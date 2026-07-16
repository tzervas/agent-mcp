//! MCP server shell for agent orchestration, built on the official `rmcp` SDK.
//!
//! This is the *only* MCP-protocol code in the crate. `rmcp` owns the JSON-RPC
//! framing, the `initialize`/`ping` handshake, and the `tools/list` schema
//! derivation (via `schemars`) + `tools/call` routing. Our job is just to:
//!
//! 1. declare the tool surface — one `#[tool]` method per tool, on the
//!    `#[tool_router]`-annotated `impl AgentMcp` block below;
//! 2. wire the handshake defaults in [`ServerHandler::get_info`];
//! 3. hand `rmcp` the stdio transport in [`serve_stdio`].
//!
//! Each tool method deserializes its typed args ([`crate::tools`]), calls
//! straight into the **unchanged** orchestration logic ([`AgentOrchestrator`]),
//! and renders the result as MCP text content. No hand-rolled JSON-RPC, no
//! `match method {…}` dispatch — that shell was replaced wholesale.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};

use crate::error::Error;
use crate::orchestrator::AgentOrchestrator;
use crate::tools::{
    parse_provider, render_consensus, render_parallel, render_prompt, render_providers,
    render_status, ConsensusArgs, ParallelPromptArgs, PromptArgs, WorkflowStartArgs,
    WorkflowStepArgs,
};
use crate::workflow::{Workflow, WorkflowStep};

/// The agent-orchestration MCP server.
///
/// Holds a cheaply-cloneable [`AgentOrchestrator`] handle (all `Arc`-backed) and
/// the `rmcp` [`ToolRouter`] generated from the `#[tool]` methods below.
#[derive(Clone)]
pub struct AgentMcp {
    orchestrator: Arc<AgentOrchestrator>,
    tool_router: ToolRouter<Self>,
}

impl AgentMcp {
    /// Create a new server around the given orchestrator.
    pub fn new(orchestrator: AgentOrchestrator) -> Self {
        Self {
            orchestrator: Arc::new(orchestrator),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl AgentMcp {
    /// Send a prompt to an AI provider (best available if none specified).
    #[tool(
        name = "agent_prompt",
        description = "Send a prompt to an AI provider. If no provider specified, uses the best available."
    )]
    async fn agent_prompt(
        &self,
        Parameters(args): Parameters<PromptArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let response = if let Some(provider_str) = args.provider {
            let provider = parse_provider(&provider_str)?;
            self.orchestrator
                .prompt_provider(provider, args.message)
                .await?
        } else {
            self.orchestrator.prompt(args.message).await?
        };

        Ok(CallToolResult::success(vec![Content::text(render_prompt(
            &response.provider,
            &response.text,
        ))]))
    }

    /// Send the same prompt to multiple providers in parallel.
    #[tool(
        name = "agent_parallel_prompt",
        description = "Send the same prompt to multiple AI providers in parallel."
    )]
    async fn agent_parallel_prompt(
        &self,
        Parameters(args): Parameters<ParallelPromptArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let providers: Vec<_> = args
            .providers
            .iter()
            .filter_map(|p| parse_provider(p).ok())
            .collect();

        if providers.len() < 2 {
            return Err(Error::InvalidParams("need at least 2 valid providers".into()).into());
        }

        let results = self
            .orchestrator
            .parallel_prompt(args.message, providers)
            .await?;

        Ok(CallToolResult::success(vec![Content::text(
            render_parallel(&results),
        )]))
    }

    /// Get a consensus answer across multiple providers.
    #[tool(
        name = "agent_consensus",
        description = "Get a consensus answer from multiple AI providers."
    )]
    async fn agent_consensus(
        &self,
        Parameters(args): Parameters<ConsensusArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let min_providers = args.min_providers.unwrap_or(3);
        let result = self
            .orchestrator
            .consensus_prompt(args.message, min_providers)
            .await?;

        Ok(CallToolResult::success(vec![Content::text(
            render_consensus(&result),
        )]))
    }

    /// Start a new multi-step workflow.
    #[tool(
        name = "agent_workflow_start",
        description = "Start a new multi-step workflow."
    )]
    async fn agent_workflow_start(
        &self,
        Parameters(args): Parameters<WorkflowStartArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut workflow = Workflow::new(args.name);

        for step_def in args.steps {
            let step = match step_def.step_type.as_str() {
                "prompt" => WorkflowStep::prompt(step_def.name, step_def.message),
                "parallel" => WorkflowStep::parallel(
                    step_def.name,
                    step_def.message,
                    step_def.providers.unwrap_or_default(),
                ),
                "consensus" => WorkflowStep::consensus(step_def.name, step_def.message),
                "review" => WorkflowStep::review(step_def.name, step_def.message),
                other => {
                    return Err(
                        Error::InvalidParams(format!("unknown step type: {}", other)).into(),
                    )
                }
            };
            workflow.add_step(step);
        }

        let id = self.orchestrator.start_workflow(workflow).await?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "# Workflow Started\n\n**ID:** `{}`\n\nUse `agent_workflow_step` with this ID to execute steps.",
            id
        ))]))
    }

    /// Execute the next step in a workflow.
    #[tool(
        name = "agent_workflow_step",
        description = "Execute the next step in a workflow."
    )]
    async fn agent_workflow_step(
        &self,
        Parameters(args): Parameters<WorkflowStepArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let result = self
            .orchestrator
            .execute_workflow_step(&args.workflow_id)
            .await?;

        let workflow = self
            .orchestrator
            .get_workflow(&args.workflow_id)
            .await
            .ok_or_else(|| Error::Workflow("workflow not found".into()))?;

        let status = if workflow.is_complete() {
            "\u{2705} Workflow Complete".to_string()
        } else {
            format!("Step {}/{}", workflow.current_step, workflow.steps.len())
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "# Workflow Step Result\n\n**Status:** {}\n**Duration:** {}ms\n\n## Output\n\n{}",
            status, result.duration_ms, result.output
        ))]))
    }

    /// Report orchestrator status (providers, workflows, per-provider stats).
    #[tool(
        name = "agent_status",
        description = "Get the status of the agent orchestrator."
    )]
    async fn agent_status(&self) -> Result<CallToolResult, ErrorData> {
        let status = self.orchestrator.status().await;
        Ok(CallToolResult::success(vec![Content::text(render_status(
            &status,
        ))]))
    }

    /// List the available AI providers and their capabilities.
    #[tool(
        name = "agent_list_providers",
        description = "List all available AI providers and their capabilities."
    )]
    async fn agent_list_providers(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            render_providers(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for AgentMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "embeddenator-agent-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Implementation::from_build_env()
            },
            instructions: Some(
                "Multi-agent orchestration over browser-driven AI providers. \
                 Use agent_list_providers to discover providers, agent_prompt / \
                 agent_parallel_prompt / agent_consensus to query them, and the \
                 agent_workflow_* tools to run multi-step workflows."
                    .into(),
            ),
        }
    }
}

/// Serve the agent MCP server over stdio until the client disconnects.
///
/// `rmcp` performs the `initialize` handshake and routes every request; this
/// call returns once the peer closes the transport.
pub async fn serve_stdio(orchestrator: AgentOrchestrator) -> anyhow::Result<()> {
    let service = AgentMcp::new(orchestrator)
        .serve(rmcp::transport::io::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
