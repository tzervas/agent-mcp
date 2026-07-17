# CLAUDE.md — agent-mcp

Multi-agent orchestration **MCP server** for VS Code / Copilot-style hosts.
Binary: `agent-mcp`. Crate: `embeddenator-agent-mcp`. Protocol shell: `rmcp` (stdio).

## Commands

```bash
# Build (git dep on webpuppet-rs — network fetch on first build)
cargo build
cargo build --release

# Tests (unit + rmcp integration + stdio e2e; no live browser keys)
cargo test --all-features

# Full local gate
./scripts/check.sh
./scripts/check.sh --fix

# Run MCP over stdio
cargo run --
# visible browser (non-headless webpuppet sessions):
cargo run -- --visible

# Help / version
cargo run -- --help
cargo run -- --version
```

## Layout

| Path | Role |
|------|------|
| `src/main.rs` | CLI (`--visible`, log level, JSON logs) |
| `src/server.rs` | `rmcp` `ServerHandler` + `serve_stdio` |
| `src/tools.rs` | Tool arg structs + pure render helpers |
| `src/orchestrator.rs` | Provider orchestration (webpuppet) |
| `src/router.rs` | Provider selection heuristics |
| `src/workflow.rs` | Multi-step workflow state machine |
| `tests/integration_handshake.rs` | In-memory MCP client handshake |
| `tests/e2e_stdio.rs` | Spawn real binary over stdio |
| `docs/mcp.example.json` | Claude Desktop MCP snippet |
| `.mcp.json.example` | Cursor / VS Code MCP snippet |

## Agent rules

- Prefer `./scripts/check.sh` before claiming complete.
- Do **not** call browser-backed tools in automated tests without explicit env/keys.
- Keep honesty: parallel ≈ sequential; consensus is placeholder; HITL resume missing.
- Compose with [agent-harness](https://github.com/tzervas/agent-harness) by reference — see [docs/INTEGRATIONS.md](docs/INTEGRATIONS.md).
- Leave mycelium isolated; see [AGENTS.md](AGENTS.md).

## Status

Early `0.2.x`. Not production multi-provider orchestration yet — usable MCP shell + browser path.
