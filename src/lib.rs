//! Multi-Agent Orchestration MCP Server
//!
//! This crate provides an MCP server for orchestrating multi-agent workflows
//! in VS Code and GitHub Copilot environments. It enables:
//!
//! - Automated prompt routing to multiple AI providers
//! - Multi-step workflow state management, including a (currently non-resumable)
//!   human-review step
//! - Sub-agent delegation to webpuppet for web-based AI interactions
//!
//! This is a `0.1.0-alpha` crate — see the "Current Limitations (alpha)" section of the repo
//! README for what's still a placeholder or not yet implemented (e.g. "parallel"/"consensus"
//! tools run sequentially against one browser session today; there is no content-screening or
//! rate-limiting module yet).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    VS Code / GitHub Copilot                      │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ MCP Protocol (JSON-RPC over stdio)
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   embeddenator-agent-mcp                         │
//! │              ┌────────────┐ ┌────────────┐                       │
//! │              │ Workflow   │ │ Provider   │                       │
//! │              │ Manager    │ │ Router     │                       │
//! │              └────────────┘ └────────────┘                       │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!         ┌───────────────────┼───────────────────┐
//!         ▼                   ▼                   ▼
//! ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
//! │  webpuppet    │ │   API         │ │  Self-hosted  │
//! │  (browser)    │ │   Providers   │ │  (future)     │
//! │               │ │   (future)    │ │               │
//! │ Claude, Grok  │ │ OpenAI API    │ │ Ollama        │
//! │ Gemini, etc.  │ │ Anthropic API │ │ vLLM          │
//! └───────────────┘ └───────────────┘ └───────────────┘
//! ```
//!
//! # MCP Tools
//!
//! | Tool | Description |
//! |------|-------------|
//! | `agent_prompt` | Send a prompt to best available provider |
//! | `agent_parallel_prompt` | Send same prompt to multiple providers (sequential today) |
//! | `agent_consensus` | Collect responses from multiple providers (longest-response heuristic, not semantic agreement) |
//! | `agent_workflow_start` | Start a multi-step workflow |
//! | `agent_workflow_step` | Execute next step in workflow |
//! | `agent_status` | Get orchestration status and stats |
//! | `agent_list_providers` | List available AI providers |

pub mod error;
pub mod orchestrator;
pub mod protocol;
pub mod router;
pub mod server;
pub mod tools;
pub mod workflow;

pub use error::{Error, Result};
pub use orchestrator::AgentOrchestrator;
pub use protocol::{McpRequest, McpResponse};
pub use router::ProviderRouter;
pub use server::AgentMcpServer;
pub use workflow::{Workflow, WorkflowStep, WorkflowState};
