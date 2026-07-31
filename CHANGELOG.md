# Changelog

All notable changes to `embeddenator-agent-mcp` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: minor = notable change).

## [Unreleased]

### Fixed
- **`agent_status` no longer claims availability it has not checked.** It reported every
  compiled-in provider as `✅ available` on any host, having performed zero checks — including
  hosts with no browser installed, where nothing could possibly have worked. Availability is now
  tri-state (`available` / `unavailable` / `unknown`), derived from a real host probe
  (CDP-capable browser detection) plus this process's own request history, and every verdict
  carries the evidence or reason behind it. Only a recently-succeeded request earns `available`.
- **`agent_list_providers` no longer returns a hardcoded string.** The inventory is enumerated from
  the providers compiled into the binary. The old static list had already drifted: it omitted
  `kaggle`, which `agent_status` was simultaneously advertising as available.
- **`kaggle` is now accepted by `agent_prompt`.** `parse_provider` rejected it while `agent_status`
  recommended it, so asking for the provider the server had just suggested failed. A round-trip test
  over `Provider::all()` guards against the drift recurring.
- **`OrchestratorConfig::timeout` is applied.** The field existed and was never read: a knob in the
  config surface that did nothing. Provider operations now run under it via
  `orchestrator::with_deadline` / `orchestrator::fan_out`, and overrunning it is an explicit
  `Error::Timeout` naming the provider and the budget.

### Added
- ROADMAP **C1 — true parallel with deadlines**: `orchestrator::fan_out` runs one task per provider
  concurrently, each under its own deadline, bounded by `OrchestratorConfig::max_concurrent` (also
  previously unread). Results are returned in the requested order; a provider that fails, times out,
  or panics contributes an explicit `Err` entry instead of silently vanishing.
- ROADMAP **B4 — modality reporting**: every inventory row declares `browser` or `api`. Nothing is
  `api` yet, and the output says so explicitly.
- `src/availability.rs`: the measured-availability model (`Availability`, `ProviderEvidence`,
  `BrowserRuntime::probe`, `classify`, `inventory`).

### Deferred
- **Wave B B1–B3 (API provider backends) are still not implemented.** No HTTP backend for
  xAI / OpenAI-compatible / Anthropic exists; the `api-providers` and `self-hosted` Cargo features
  remain empty placeholders. This change makes the *reporting* honest about that rather than
  starting a backend that would be half-wired.
- Consensus (C2) is still the longest-response heuristic with a hardcoded `0.5` agreement score.
- Workflow human-review resume (C3) still returns an error rather than a resumable NEEDS_INPUT event.

## [0.2.1] - 2026-07-21

### Added
- Production polish (P28e): `AGENTS.md`, `CLAUDE.md`, 5-minute README path, MCP host examples
  (`docs/mcp.example.json`, `.mcp.json.example`), and [docs/INTEGRATIONS.md](docs/INTEGRATIONS.md)
  describing compose-by-reference with **agent-harness**, webpuppet-rs, and optional security-mcp.
- Fleet CI/security badges on the README, and self-hosted-runner catch-up hardening for the fleet
  gates (P26 standards).

### Changed
- Fleet policy: issues are now auto-closed only on `main`, not on feature branches.

### Fixed
- README: document **git** dependency on webpuppet-rs (was incorrectly described as a sibling path dep).
- CI: harden fleet gates so self-hosted runner catch-up runs don't produce false-negative failures.

### Deferred
- Still no live-provider (API-key) integrations — `web-providers` (browser-driven, via
  webpuppet-rs) remains the only working provider path; `api-providers`/`self-hosted` feature
  flags are placeholders with no implementation.
- Consensus gathering across providers is still a placeholder — see
  [Current Limitations](README.md#current-limitations).
- Human-in-the-loop workflow resume is still not implemented.

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
