# Documentation index — agent-mcp

Project-management and deep docs for
[`embeddenator-agent-mcp`](https://github.com/tzervas/agent-mcp) (binary `agent-mcp`).

## Project management (start here for status)

| Doc | Layer | Question it answers |
|-----|-------|---------------------|
| [DEVELOPMENT-PATH.md](DEVELOPMENT-PATH.md) | Back | How we got here — decisions, PRs, releases |
| [CURRENT-STATE.md](CURRENT-STATE.md) | Back | What works **today**, measured (VERIFIED / UNVERIFIED) |
| [ROADMAP.md](ROADMAP.md) | Back | Planned work and **what would unblock** each item |

## Product & integration

| Doc | Topic |
|-----|--------|
| [ASSESSMENT.md](ASSESSMENT.md) | Gap analysis vs cabal / maturity notes |
| [INTEGRATIONS.md](INTEGRATIONS.md) | Compose with agent-harness, webpuppet-rs, relay |
| [LOCAL_CHECKS.md](LOCAL_CHECKS.md) | `./scripts/check.sh` and CI parity |
| [FLEET_STANDARDS.md](FLEET_STANDARDS.md) | Fleet workflows, issue-close policy, badges |
| [mcp.example.json](mcp.example.json) | Claude Desktop MCP snippet |

## Front layer

| Doc | Topic |
|-----|--------|
| [../README.md](../README.md) | What it is, 5-minute path, tools, limitations |
| [../AGENTS.md](../AGENTS.md) | Agent/contributor product boundaries |
| [../CLAUDE.md](../CLAUDE.md) | Cargo command cheat sheet |

## Architecture decisions

| ADR | Decision |
|-----|----------|
| [adr/0001-rmcp-protocol-shell.md](adr/0001-rmcp-protocol-shell.md) | Official rmcp SDK for stdio MCP shell (v0.2.0) |

## Memory index (optional)

If present, Layer-1 Tero corpus: [tero-index/](tero-index/).

## Shape

- **Front** (README): concise, start in under a minute.  
- **Back** (`docs/`): measured state, history, roadmap depth.  

Do not treat aspirational architecture diagrams as current capability — prefer
[CURRENT-STATE.md](CURRENT-STATE.md).
