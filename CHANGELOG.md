# Changelog

All notable changes to `embeddenator-agent-mcp` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: minor = notable change).

## [0.2.0] - 2026-07-10

### Changed

- **Adopt the official [`rmcp`](https://crates.io/crates/rmcp) MCP SDK (0.8) for the protocol shell.**
  The hand-rolled JSON-RPC framing (`src/protocol.rs`) and the `match method {…}` dispatch
  (`src/server.rs`) are replaced by an `rmcp` `ServerHandler`: `#[tool_router]` + `#[tool]` methods
  derive the `tools/list` schema (via `schemars`) and route `tools/call`, and `serve_stdio` uses the
  `transport-io` `(stdin, stdout)` transport. Features enabled: `server`, `transport-io`, `macros`
  (OAuth/`auth` + `reqwest`/HTTP stay off — this server is stdio-only). The orchestration business
  logic (`orchestrator.rs`, `router.rs`, `workflow.rs`) and the seven tools' behavior are unchanged.
- The public tool surface is unchanged in the ways that matter: same 7 tool names, same tool
  behavior, same rendered output. **Caveat:** the tool *input JSON Schemas* are now derived by
  `schemars` from the Rust arg structs, so their shape may differ from the hand-written originals —
  notably the `provider` field no longer carries a client-side `enum` constraint (valid values are
  documented in the field description, and unknown providers are still rejected server-side, never
  silently). MCP hosts that rendered a provider dropdown from the old schema will now see a free
  string.

### Removed

- `src/protocol.rs` (hand-written MCP types — now from `rmcp::model`).
- `AgentMcpServer` / `McpRequest` / `McpResponse` public items (replaced by `AgentMcp` + `serve_stdio`).
- Dead dependencies dropped along with the hand-rolled shell they served: `async-trait` and
  `futures` (both only used by the now-removed `Tool` trait), and the unused `http`/`stdio` cargo
  features with their `axum`/`tower`/`tower-http` dependencies (the server is stdio-only; `rmcp`
  owns the transport).

### Added

- Test coverage across three layers: unit (tool parsing/rendering/schema, table-driven), integration
  (an `rmcp` client driving `initialize`/`tools/list`/`tools/call` over an in-memory duplex
  transport — no external calls), and e2e (spawning the real stdio binary and running a full MCP
  handshake + tool call). Browser-backed tools are never invoked in tests (no live-key dependency).

### MSRV

- **Bumped the minimum supported Rust version to 1.85** (recorded as `rust-version = "1.85"`).
  `rmcp` 0.8.5 is an edition-2024 crate, so the transitive compiler floor is 1.85 (edition 2024
  stabilized there). This crate itself stays on edition 2021. This is a deliberate, documented
  toolchain change, not a silent pin bump.

## [0.1.0] - 2026-07-10

- Baseline extraction: multi-agent orchestration MCP server (browser-driven providers via
  `embeddenator-webpuppet`), hand-rolled MCP/JSON-RPC shell, seven `agent_*` tools.

[0.2.0]: https://github.com/tzervas/agent-mcp/releases/tag/v0.2.0
[0.1.0]: https://github.com/tzervas/agent-mcp/releases/tag/v0.1.0
