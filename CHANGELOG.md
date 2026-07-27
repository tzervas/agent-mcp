# Changelog

All notable changes to `embeddenator-agent-mcp` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: minor = notable change).

## [Unreleased]

## [0.3.0] - 2026-07-26

### Security
- **Cleared rmcp CVE-2026-42559 (HIGH, "rmcp Streamable HTTP server transport has a DNS rebinding
  vulnerability")** by upgrading `rmcp` 0.8.5 -> 2.2.0 (568f325). This crate builds rmcp with
  `["server", "transport-io", "macros"]` only, so the vulnerable streamable-HTTP server transport
  was never compiled in and the DNS-rebinding path itself was unreachable — but the advisory still
  failed the repo's own `trivy fs --severity HIGH,CRITICAL --exit-code 1` gate on the dependency
  version, so the SDK is upgraded rather than suppressed.
- `Cargo.lock` is now tracked (was gitignored). This crate ships a binary (`[[bin]] agent-mcp`), so
  the lockfile is part of the product: builds are now reproducible (`cargo build --locked` /
  `--locked` in the Dockerfile and the release workflow), and `trivy fs` can finally see this
  crate's dependency graph at all — committing the lockfile is what surfaced the CVE above in the
  first place.

### Changed
- **rmcp 0.8.5 -> 2.2.0 (major SDK jump; source migration, no orchestration logic touched):**
  `model::Content` -> `model::ContentBlock`; `ServerInfo`/`Implementation` are now
  `#[non_exhaustive]` and built through `ServerInfo::new(..).with_protocol_version(..)
  .with_server_info(..).with_instructions(..)` instead of struct literals; the dead
  `tool_router: ToolRouter<Self>` field is removed (`#[tool_handler]` resolves it via the generated
  `Self::tool_router()`); `CallToolRequestParam` -> `CallToolRequestParams::new(name)` in tests.
  **The MCP wire contract is unchanged across this jump**: `initialize` still answers
  protocolVersion `"2024-11-05"` with the same `serverInfo` shape, `tools/list` still returns the
  same 7 tools with camelCase `inputSchema`, and `tools/call` still returns `content`/`isError`.
  `src/server.rs` pins `ProtocolVersion::V_2024_11_05` explicitly rather than tracking rmcp's new
  `LATEST` default (2025-11-25) — the negotiated protocol version does not move as a side effect of
  this dependency bump.
- `thiserror` moved to the 2.0 line for the crate's own direct dependency (usage is plain
  `#[error(...)]`/`#[from]`, unchanged in 2.0); the 1.0 line stays in the graph transitively via
  `chromiumoxide`/`tungstenite`.
- CI: `actions/checkout` 4 -> 7, `astral-sh/setup-uv` 5 -> 7 (dependabot, major bumps).
- CI: the release workflow now builds with `--locked`, matching the Dockerfile, so a shipped
  release artifact can never drift from the scanned lockfile.

### Fixed
- Dockerfile: declare `HEALTHCHECK NONE` explicitly (trivy DS-0026). This image runs a stdio MCP
  server with no listening port; a `HEALTHCHECK CMD` could only spawn a second `agent-mcp` process,
  telling us nothing about the one serving the session, and anything probing stdin/stdout would
  corrupt the JSON-RPC stream. `HEALTHCHECK NONE` states that intent instead of leaving trivy (or a
  reader) to guess it was an oversight.

### Governance
- Add `.cz.toml` (commitizen config) with `major_version_zero = true`. Without it, commitizen
  computes the next version from conventional-commit types alone, and this release's `harden:`
  commit + the rmcp major dependency jump would otherwise mint `1.0.0` on the next `cz bump` — an
  accidental 1.0 this project is not ready to claim. `version_files` covers `Cargo.toml` and
  `README.md`; `Cargo.lock` is deliberately excluded (see comment in `.cz.toml`).

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
