# AGENTS.md — agent-mcp

Multi-agent orchestration MCP server (`embeddenator-agent-mcp` crate; binary `agent-mcp`).
stdio-only, built on the official [`rmcp`](https://crates.io/crates/rmcp) SDK.

**Leave mycelium isolated.** Coordinate via this repo, webpuppet-rs, agent-harness, and cabal — not mycelium.

## Product role

| Concern | Owner |
|---------|--------|
| MCP tools (`agent_*`) + rmcp shell | **this repo** |
| Browser-backed providers | [webpuppet-rs](https://github.com/tzervas/webpuppet-rs) (git dep) |
| Swarm / spawn CLI composition | [agent-harness](https://github.com/tzervas/agent-harness) (compose by reference) |
| Telegram / relay runtime | [tg-agent-relay](https://github.com/tzervas/tg-agent-relay) (separate product) |

## Local checks

```bash
./scripts/check.sh          # fmt + clippy -D warnings + build + test
./scripts/check.sh --fix # apply rustfmt
cargo test --all-features
cargo run -- --help
```

MSRV: **Rust 1.85+** (`rmcp` 0.8 is edition-2024).

## Tero (optional)

If a `docs/tero-index/index.json` is present, prefer tero queries before large greps.
Cabal auto-detects a local index when run from this tree (`TERO_INDEX_PATH` override).

## PR flow

- Feature branch off `main` (or `dev` when used as integration)
- Run `./scripts/check.sh` green before PR
- Prefer merge via PR; do not force-push protected branches

## Honesty bar

- Browser providers only today; API/self-hosted are planned
- `agent_parallel_prompt` is sequential under the hood
- Consensus is a placeholder heuristic (longest response / fixed score)
- Human-review workflow steps do not resume yet

See README **Current Limitations** and [docs/ASSESSMENT.md](docs/ASSESSMENT.md).

## Further reading

- [README.md](README.md) — 5-minute path + tools
- [docs/INTEGRATIONS.md](docs/INTEGRATIONS.md) — harness / compose notes
- [docs/ROADMAP.md](docs/ROADMAP.md)
- [CLAUDE.md](CLAUDE.md) — cargo command cheat sheet
