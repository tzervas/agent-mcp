# ADR 0001 — Adopt official rmcp SDK for the MCP protocol shell

| Field | Value |
|-------|--------|
| Status | Accepted |
| Date | 2026-07-10 |
| Evidence | PR **#15** (`2bf8f30`), release **v0.2.0**, CHANGELOG 0.2.0 |

## Context

agent-mcp originally shipped a hand-rolled MCP/JSON-RPC layer (`src/protocol.rs`)
and match-based method dispatch in `src/server.rs`. That duplicated work the
official Rust MCP SDK already owns (framing, initialize/ping, tools/list schema,
tools/call routing).

## Decision

Use [`rmcp`](https://crates.io/crates/rmcp) as the **only** MCP protocol shell:

- `ServerHandler` + `#[tool_router]` / `#[tool]` methods
- `serve_stdio` with `transport-io` (stdin/stdout)
- Features: `server`, `transport-io`, `macros` — **not** OAuth/`auth` or HTTP/`reqwest`

Orchestration modules (`orchestrator`, `router`, `workflow`) and the seven
`agent_*` tool **names and behaviors** stay in this crate.

## Consequences

| Positive | Negative / trade-off |
|----------|----------------------|
| Less custom protocol surface to secure and test | MSRV raised to **1.85** (edition-2024 transitive floor) |
| Schema derivation via `schemars` | Tool input schemas may differ from hand-written JSON Schema (e.g. free-string `provider`) |
| stdio-only matches Cursor / VS Code / Claude Desktop hosts | No in-process HTTP MCP transport in this product |
| Clear product boundary: protocol vs orchestration | Dependency must track rmcp CVEs (see PR **#24** lockfile + version bump) |

## Alternatives considered

| Alternative | Why not |
|-------------|---------|
| Keep hand-rolled protocol | Ongoing framing/schema debt; diverges from ecosystem |
| Enable rmcp HTTP/auth features “for later” | False capability signal; product is stdio-only |
| Rewrite orchestration in the same PR | Unnecessary risk; behavior was already the honesty problem, not framing |

## Follow-ups

- Live API providers remain a separate decision ([ROADMAP.md](../ROADMAP.md) Wave B).
- Pin/bump rmcp via lockfile; do not reintroduce path deps for protocol.
