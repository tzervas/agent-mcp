//! Tool input schemas and result rendering for agent-mcp.
//!
//! This module is the *data* half of the MCP tool surface: the input structs
//! (each `#[derive(JsonSchema, Deserialize)]` so `rmcp` can derive the
//! `tools/list` schema) and the pure functions that render orchestrator results
//! into MCP text content. The tool *methods* — the `#[tool]`-annotated handlers
//! that `rmcp` routes `tools/call` to — live on the `AgentMcp` handler in
//! [`crate::server`]. The orchestration business logic they call into
//! ([`crate::orchestrator`], [`crate::router`], [`crate::workflow`]) is unchanged.

use schemars::JsonSchema;
use serde::Deserialize;

use embeddenator_webpuppet::Provider;

use crate::error::{Error, Result};
use crate::orchestrator::{ConsensusResult, OrchestratorStatus};

// =============================================================================
// Tool input schemas
// =============================================================================

/// Arguments for the `agent_prompt` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PromptArgs {
    /// The prompt message to send.
    pub message: String,
    /// Optional: specific provider to use (claude, grok, gemini, chatgpt,
    /// perplexity, notebooklm). If omitted, the best available is chosen.
    #[serde(default)]
    pub provider: Option<String>,
    /// Optional: system context or instructions.
    #[serde(default)]
    #[allow(dead_code)]
    pub context: Option<String>,
}

/// Arguments for the `agent_parallel_prompt` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParallelPromptArgs {
    /// The prompt message to send.
    pub message: String,
    /// List of providers to query (at least 2 valid ones required).
    pub providers: Vec<String>,
}

/// Arguments for the `agent_consensus` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConsensusArgs {
    /// The question to get consensus on.
    pub message: String,
    /// Minimum providers to query (default: 3).
    #[serde(default)]
    pub min_providers: Option<usize>,
}

/// A single step definition inside an `agent_workflow_start` request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowStepDef {
    /// Human-readable step name.
    pub name: String,
    /// Step kind: one of `prompt`, `parallel`, `consensus`, `review`.
    #[serde(rename = "type")]
    pub step_type: String,
    /// The prompt/message for this step.
    pub message: String,
    /// Optional provider (for `prompt` steps).
    #[serde(default)]
    #[allow(dead_code)]
    pub provider: Option<String>,
    /// Optional provider list (for `parallel` steps).
    #[serde(default)]
    pub providers: Option<Vec<String>>,
}

/// Arguments for the `agent_workflow_start` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowStartArgs {
    /// Name of the workflow.
    pub name: String,
    /// Ordered workflow steps to execute.
    pub steps: Vec<WorkflowStepDef>,
}

/// Arguments for the `agent_workflow_step` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowStepArgs {
    /// ID of the workflow to advance.
    pub workflow_id: String,
}

// =============================================================================
// Result rendering (pure)
// =============================================================================

/// Render a single-provider prompt response.
pub fn render_prompt(provider: &Provider, text: &str) -> String {
    format!("**Response from {}:**\n\n{}", provider, text)
}

/// Render the results of a parallel prompt.
pub fn render_parallel(
    results: &[(Provider, Result<embeddenator_webpuppet::PromptResponse>)],
) -> String {
    let body = results
        .iter()
        .map(|(provider, result)| match result {
            Ok(resp) => format!("## {}\n\n{}", provider, resp.text),
            Err(e) => format!("## {} (Error)\n\n{}", provider, e),
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");
    format!("# Parallel Responses\n\n{}", body)
}

/// Render a consensus result.
pub fn render_consensus(result: &ConsensusResult) -> String {
    let responses_text = result
        .responses
        .iter()
        .map(|r| {
            let marker = if r.selected { "\u{2713}" } else { "\u{25cb}" };
            format!(
                "{} **{}**: {}",
                marker,
                r.provider,
                r.text.chars().take(200).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "# Consensus Result\n\n**Agreement Score:** {:.0}%\n\n## Consensus Answer\n\n{}\n\n## Individual Responses\n\n{}",
        result.agreement_score * 100.0,
        result.consensus_text,
        responses_text
    )
}

/// Render orchestrator status.
pub fn render_status(status: &OrchestratorStatus) -> String {
    let providers_text = status
        .available_providers
        .iter()
        .map(|p| format!("- \u{2705} {}", p))
        .collect::<Vec<_>>()
        .join("\n");

    let stats_text = status
        .provider_stats
        .iter()
        .map(|(p, s)| {
            format!(
                "- **{}**: {} total, {} success, {} failed",
                p, s.total_requests, s.successful_requests, s.failed_requests
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "# Agent Orchestrator Status\n\n## Available Providers\n\n{}\n\n## Active Workflows\n\n{}\n\n## Provider Statistics\n\n{}",
        providers_text,
        status.active_workflows,
        if stats_text.is_empty() {
            "No requests yet".to_string()
        } else {
            stats_text
        }
    )
}

/// Render the static provider catalogue.
pub fn render_providers() -> String {
    let providers = [
        (
            "claude",
            "Claude (Anthropic)",
            "200k context, artifacts, code execution",
        ),
        ("grok", "Grok (X/xAI)", "Real-time info, X integration"),
        (
            "gemini",
            "Gemini (Google)",
            "2M context, Google integration",
        ),
        (
            "chatgpt",
            "ChatGPT (OpenAI)",
            "GPT-4o, vision, web search, code",
        ),
        (
            "perplexity",
            "Perplexity AI",
            "Search-focused, sources cited",
        ),
        (
            "notebooklm",
            "NotebookLM (Google)",
            "500k context, research assistant",
        ),
    ];

    let text = providers
        .iter()
        .map(|(id, name, caps)| format!("## {} (`{}`)\n\n{}\n", name, id, caps))
        .collect::<Vec<_>>()
        .join("\n");

    format!("# Available AI Providers\n\n{}", text)
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse a provider string into the webpuppet [`Provider`] enum.
///
/// Never-silent: an unknown provider is an explicit [`Error::InvalidParams`],
/// not a silent default (house rule #2 / G2).
pub fn parse_provider(s: &str) -> Result<Provider> {
    match s.to_lowercase().as_str() {
        "claude" => Ok(Provider::Claude),
        "grok" => Ok(Provider::Grok),
        "gemini" => Ok(Provider::Gemini),
        "chatgpt" | "openai" => Ok(Provider::ChatGpt),
        "perplexity" => Ok(Provider::Perplexity),
        "notebooklm" | "notebook" => Ok(Provider::NotebookLm),
        _ => Err(Error::InvalidParams(format!("unknown provider: {}", s))),
    }
}

#[cfg(test)]
mod tests;
