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
//! The MCP shell is built on the official [`rmcp`] SDK (server + stdio transport); the
//! orchestration logic ([`orchestrator`], [`router`], [`workflow`]) is transport-agnostic.
//! See the "Current Limitations" section of the repo README for what's still a placeholder or
//! not yet implemented (e.g. consensus is a longest-response heuristic with a hardcoded agreement
//! score; there is no content-screening or rate-limiting module yet; API-provider backends do not
//! exist). Parallel prompting *is* genuinely concurrent — see [`orchestrator::fan_out`] — though it
//! still shares one browser session.
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
//! | `agent_parallel_prompt` | Send same prompt to multiple providers, concurrently, with deadlines |
//! | `agent_consensus` | Collect responses from multiple providers (longest-response heuristic, not semantic agreement) |
//! | `agent_workflow_start` | Start a multi-step workflow |
//! | `agent_workflow_step` | Execute next step in workflow |
//! | `agent_status` | Probed provider availability, workflow count, and stats |
//! | `agent_list_providers` | Enumerate compiled-in providers with modality + probed availability |
//!
//! # Honest availability
//!
//! `agent_status` and `agent_list_providers` never assert availability they have
//! not established. Availability is tri-state ([`Availability`]) and every value
//! carries the evidence or the reason behind it; a provider that has not been
//! exercised is `unknown`, not `available`. See [`availability`].

pub mod availability;
pub mod error;
pub mod orchestrator;
pub mod router;
pub mod server;
pub mod tools;
pub mod workflow;

pub use availability::{Availability, BrowserRuntime, Modality, ProviderEntry};
pub use error::{Error, Result};
pub use orchestrator::AgentOrchestrator;
pub use router::ProviderRouter;
pub use server::{serve_stdio, AgentMcp};
pub use workflow::{Workflow, WorkflowState, WorkflowStep};
