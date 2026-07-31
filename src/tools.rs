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

use crate::availability::{BrowserRuntime, ProviderEntry};
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
///
/// Every provider line carries its measured state *and the provenance of that
/// state*. A check mark is reserved for providers a real request has recently
/// succeeded against; anything unestablished renders as `unknown` with the reason.
pub fn render_status(status: &OrchestratorStatus) -> String {
    let providers_text = status
        .inventory
        .iter()
        .map(|entry| {
            format!(
                "- {} `{}` ({}) — {}: {}",
                entry.availability.marker(),
                entry.id,
                entry.modality,
                entry.availability.label(),
                entry.availability.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let verified = format!(
        "**Verified available: {} of {}.**",
        status.available_providers.len(),
        status.inventory.len()
    );

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
        "# Agent Orchestrator Status\n\n\
         ## Provider Availability\n\n\
         {PROVENANCE_NOTE}\n\n\
         {providers_text}\n\n\
         {verified}\n\n\
         ## Browser Runtime\n\n\
         {}\n\n\
         ## Active Workflows\n\n\
         {}\n\n\
         ## Provider Statistics\n\n\
         {}",
        status.browser_runtime.summary(),
        status.active_workflows,
        if stats_text.is_empty() {
            "No requests yet".to_string()
        } else {
            stats_text
        }
    )
}

/// What was and was not checked. Printed on every availability report so a reader
/// never has to guess how much a green check is worth.
const PROVENANCE_NOTE: &str = "\
Probed at call time: which CDP-capable browsers are installed on this host, and \
this process's own request history.\n\
Not probed: provider login/session state — only a real request can establish that, \
so an untried provider is reported `unknown`, not available.";

/// Render the provider inventory.
///
/// Built from the compiled-in provider set and a live host probe, so it cannot
/// drift from what the binary can actually address. (It replaced a hardcoded
/// six-entry string that had already drifted: it omitted `kaggle`, which
/// `agent_status` was simultaneously reporting as available.)
pub fn render_providers(inventory: &[ProviderEntry], runtime: &BrowserRuntime) -> String {
    let rows = inventory
        .iter()
        .map(|entry| {
            format!(
                "## `{}`\n\n\
                 - modality: `{}`\n\
                 - endpoint: {}\n\
                 - availability: {} {} — {}\n",
                entry.id,
                entry.modality,
                entry.endpoint,
                entry.availability.marker(),
                entry.availability.label(),
                entry.availability.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "# AI Provider Inventory\n\n\
         {} provider(s), enumerated from the providers compiled into this binary — \
         not a hand-maintained list.\n\n\
         {PROVENANCE_NOTE}\n\n\
         {}\n\n\
         ## Notes\n\n\
         - Browser runtime: {}\n\
         - Every provider above is `browser` modality. The `api` modality \
           (ROADMAP Wave B, B1-B3: HTTP backends for xAI / OpenAI-compatible / \
           Anthropic) is **not implemented**, so no provider can report it yet.\n\
         - Capability claims (context window, tool support) are deliberately \
           omitted: agent-mcp has no way to verify them, and the previous \
           hardcoded blurbs were unverified.\n",
        inventory.len(),
        rows,
        runtime.summary()
    )
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse a provider string into the webpuppet [`Provider`] enum.
///
/// Never-silent: an unknown provider is an explicit [`Error::InvalidParams`],
/// not a silent default (house rule #2 / G2).
///
/// Every id this accepts must be one the inventory advertises, and vice versa —
/// `kaggle` used to be missing here while `agent_status` listed it as available,
/// so asking for the provider the server had just recommended failed. The
/// `parse_provider_round_trips_every_compiled_provider` test guards that.
pub fn parse_provider(s: &str) -> Result<Provider> {
    match s.to_lowercase().as_str() {
        "claude" => Ok(Provider::Claude),
        "grok" => Ok(Provider::Grok),
        "gemini" => Ok(Provider::Gemini),
        "chatgpt" | "openai" => Ok(Provider::ChatGpt),
        "perplexity" => Ok(Provider::Perplexity),
        "notebooklm" | "notebook" => Ok(Provider::NotebookLm),
        "kaggle" => Ok(Provider::Kaggle),
        _ => Err(Error::InvalidParams(format!("unknown provider: {}", s))),
    }
}

#[cfg(test)]
mod tests;
