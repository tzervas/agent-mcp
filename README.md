# embeddenator-agent-mcp

Multi-agent orchestration MCP server for VS Code and GitHub Copilot.

## Overview

`embeddenator-agent-mcp` provides a Model Context Protocol (MCP) server for orchestrating prompts across multiple AI providers. It enables:

- **Intelligent Provider Routing**: Automatically select the best provider based on task type
- **Multi-Provider Querying**: Send the same prompt to multiple providers and collect their responses
- **Consensus Gathering**: Collect multiple providers' responses to the same question
- **Workflow Management**: Define multi-step automation workflows, including a human-review step

> **Status (v0.2.0).** The MCP protocol shell is now built on the official
> [`rmcp`](https://crates.io/crates/rmcp) Rust SDK (server + stdio transport); the orchestration
> logic is unchanged. This project works but is early — several items above are simpler today than
> they may sound — see [Current Limitations](#current-limitations) before relying on this in a
> workflow.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    VS Code / GitHub Copilot                      │
└───────────────────────────┬─────────────────────────────────────┘
                            │ MCP Protocol (rmcp SDK — JSON-RPC over stdio)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   embeddenator-agent-mcp                         │
│              ┌────────────┐ ┌────────────┐                       │
│              │ Workflow   │ │ Provider   │                       │
│              │ Manager    │ │ Router     │                       │
│              └────────────┘ └────────────┘                       │
└───────────────────────────┬─────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│  webpuppet    │ │   API         │ │  Self-hosted  │
│  (browser)    │ │   Providers   │ │  (future)     │
│               │ │   (future)    │ │               │
│ Claude, Grok  │ │ OpenAI API    │ │ Ollama        │
│ Gemini, etc.  │ │ Anthropic API │ │ vLLM          │
└───────────────┘ └───────────────┘ └───────────────┘
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `agent_prompt` | Send a prompt to best available provider |
| `agent_parallel_prompt` | Send same prompt to multiple providers |
| `agent_consensus` | Get consensus answer from multiple providers |
| `agent_workflow_start` | Start a multi-step workflow |
| `agent_workflow_step` | Execute next step in workflow |
| `agent_status` | Get orchestration status and stats |
| `agent_list_providers` | List available AI providers |

## Supported Providers

### Web-based (via webpuppet)

| Provider | ID | Features |
|----------|-----|----------|
| Claude (Anthropic) | `claude` | 200k context, artifacts, code execution |
| ChatGPT (OpenAI) | `chatgpt` | GPT-4o, vision, web search, code |
| Gemini (Google) | `gemini` | 2M context, Google integration |
| Grok (X/xAI) | `grok` | Real-time info, X integration |
| Perplexity AI | `perplexity` | Search-focused, sources cited |
| NotebookLM | `notebooklm` | 500k context, research assistant |

### API-based (planned)

- OpenAI API
- Anthropic API  
- Google AI API

### Self-hosted (planned)

- Ollama
- vLLM
- text-generation-webui
- LocalAI

## Installation

### Building from Source

```bash
cargo build -p embeddenator-agent-mcp --release
```

> **Build prerequisite:** this crate depends on `embeddenator-webpuppet` via a relative path
> (`../embeddenator-webpuppet` in `Cargo.toml`), not a published/vendored crate. To build, you need
> that sibling checked out next to this repo (e.g. as part of the author's local multi-repo
> workspace). A standalone clone of just this repository will not build out of the box — this is a
> known alpha-stage limitation, not a bug in this crate's own code.

### Docker (optional, service use only)

A `Dockerfile` is included for running `agent-mcp` as a standalone service (e.g. behind a
process manager, or wherever a containerized MCP stdio server is convenient). It is **not**
the publish path for this project — see [Releases](#releases) below — just a convenience for
anyone who wants a containerized runtime:

```bash
docker build -t agent-mcp .
docker run -i agent-mcp --visible
```

### VS Code Integration

Add to your VS Code `mcp.json`:

```json
{
  "servers": {
    "agent": {
      "command": "/path/to/agent-mcp",
      "args": ["--visible"]
    }
  }
}
```

## Usage

### Basic Prompt

```json
{
  "name": "agent_prompt",
  "arguments": {
    "message": "Explain quantum computing in simple terms"
  }
}
```

### Parallel Prompt

```json
{
  "name": "agent_parallel_prompt", 
  "arguments": {
    "message": "What are the best practices for API design?",
    "providers": ["claude", "chatgpt", "gemini"]
  }
}
```

### Consensus

```json
{
  "name": "agent_consensus",
  "arguments": {
    "message": "What is the capital of France?",
    "min_providers": 3
  }
}
```

### Workflow

```json
{
  "name": "agent_workflow_start",
  "arguments": {
    "name": "Research Workflow",
    "steps": [
      {
        "name": "Search",
        "type": "prompt",
        "message": "Find recent papers on quantum computing",
        "provider": "perplexity"
      },
      {
        "name": "Summarize", 
        "type": "prompt",
        "message": "Summarize the key findings",
        "provider": "claude"
      },
      {
        "name": "Verify",
        "type": "consensus",
        "message": "Verify the accuracy of this summary"
      }
    ]
  }
}
```

## CLI Options

```
agent-mcp [OPTIONS]

Options:
  --visible         Run browser in visible (non-headless) mode
  --log-level       Log level (trace, debug, info, warn, error) [default: info]
  --json-logs       Output logs as JSON
  -h, --help        Print help
  -V, --version     Print version
```

## Current Limitations

This is an early `0.2.x` project — functional, but with real gaps behind a few README/architecture
claims that describe target design rather than shipped behavior:

- **Web-based providers only, today.** All prompting goes through `embeddenator-webpuppet` browser
  automation. API-based providers (OpenAI/Anthropic/Google) and self-hosted backends
  (Ollama/vLLM/LocalAI) are listed above as "planned" — there is no code path for them yet.
- **"Parallel" prompting is sequential.** `agent_parallel_prompt` and `agent_consensus` drive one
  browser session at a time (`AgentOrchestrator::parallel_prompt` in `src/orchestrator.rs`), because
  the current backend is browser automation. True concurrent querying is future work, most likely
  once API-based providers land.
- **Consensus is a placeholder heuristic.** `agent_consensus` does not compute semantic agreement — it
  returns the longest of the collected responses as the "consensus," and the reported
  `agreement_score` is a hardcoded `0.5`, not a measured value (`AgentOrchestrator::find_consensus`).
- **Human-in-the-loop workflow steps don't resume.** A `review`/`human_review` workflow step pauses
  the workflow (`WorkflowState::Paused`) and returns an error; there is currently no API to submit a
  human response and resume the workflow. Treat this step type as not-yet-functional.
- **No content screening or rate limiting is implemented.** There is no security/content-filtering
  module and no request-rate-limiting logic in this crate today.
- **Build requires a local sibling checkout** of `embeddenator-webpuppet` (see
  [Installation](#building-from-source)) — this repo alone is not buildable as published.

None of this is hidden in the code (see the inline comments in `src/orchestrator.rs`), but it wasn't
previously called out here. Treat the feature list above as the intended design; this section is the
honest status.

## Releases

Published releases are the single channel: [GitHub Releases](https://github.com/tzervas/agent-mcp/releases)
against an annotated `vX.Y.Z` tag. Each release carries the built `agent-mcp` binary
(`cargo build --release`) plus a `agent-mcp.sha256` checksum as downloadable assets — verify
the download with `sha256sum -c agent-mcp.sha256` before trusting it. There is no crates.io
crate and no published container image for this project; the `Dockerfile` above is optional
and for local/service use only, not a publish target.

Releases are cut manually via the repo's `Release` GitHub Actions workflow
(`workflow_dispatch`, see `.github/workflows/release.yml`) — no release is auto-created on tag
push.

## License

MIT

## Status & roadmap

- [Assessment & gaps](docs/ASSESSMENT.md)
- [Product roadmap & API plans](docs/ROADMAP.md)
## Semver 2026-07-10
v0.1.0 agent-mcp (supportive mcp tooling/helper from mycelium read-only extract).
