# agent-mcp — Tero Index (Layer 1)

> **Honesty:** Empirical/Declared — lite heading/line heuristic over markdown in agent-mcp via tero-mcp/scripts/generate_lite_index.py; source files are ground truth. Generated 2026-07-09.
> Use this index to find where to Read, not as authoritative ground truth.

- **Items:** 61
- **Flagged:** 0
- **item_tag:** `Empirical/Declared`
- **Machine index:** [`index.json`](./index.json)
- **Manifest:** [`MANIFEST.toml`](./MANIFEST.toml)

## doc (61 entries)

| Anchor | Kind | Id | Title | File:Line | Status | Summary |
|---|---|---|---|---|---|---|
| `agents` | other | — | AGENTS.md — agent-mcp | `AGENTS.md:2` | — | Use Tero + cabal-devmelopner for work here. |
| `agents--tero-layer-1-corpus-index` | section | — | Tero (Layer-1 corpus index) | `AGENTS.md:6` | — | Repo has docs/tero-index/index.json (generated/ refreshed via tero-mcp/scripts/generateliteindex.py). |
| `agents--agent-with-context` | other | — | agent with context: | `AGENTS.md:18` | — | uv run --project ../cabal-devmelopner cabal-devmelopner "task description here" --use-tero |
| `agents--working-with-cabal-devmelopner-agent-tool` | section | — | Working with cabal-devmelopner agent tool | `AGENTS.md:24` | — | This project is prepared for integration: |
| `agents--local-checks` | section | — | Local checks | `AGENTS.md:36` | — | Look for: |
| `agents--further-reading` | section | — | Further reading | `AGENTS.md:44` | — | - README.md |
| `contributing` | section | — | Contributing to This Project | `CONTRIBUTING.md:1` | — | Thank you for your interest in contributing! |
| `contributing--development-setup` | section | — | Development Setup | `CONTRIBUTING.md:5` | — | 1. Clone the repository |
| `contributing--pull-request-process` | section | — | Pull Request Process | `CONTRIBUTING.md:12` | — | 1. Fork the repository |
| `contributing--code-style` | section | — | Code Style | `CONTRIBUTING.md:20` | — | - Use cargo fmt for formatting |
| `contributing--license` | section | — | License | `CONTRIBUTING.md:27` | — | By contributing, you agree that your contributions will be licensed under the MIT License. |
| `readme` | other | — | embeddenator-agent-mcp | `README.md:1` | — | Multi-agent orchestration MCP server for VS Code and GitHub Copilot. |
| `readme--overview` | section | — | Overview | `README.md:5` | — | embeddenator-agent-mcp provides a Model Context Protocol (MCP) server for orchestrating prompts across multiple AI providers. It enables: |
| `readme--architecture` | section | — | Architecture | `README.md:18` | — | ┌─────────────────────────────────────────────────────────────────┐ |
| `readme--mcp-tools` | section | — | MCP Tools | `README.md:45` | — | — |
| `readme--supported-providers` | section | — | Supported Providers | `README.md:57` | — | — |
| `readme--web-based-via-webpuppet` | section | — | Web-based (via webpuppet) | `README.md:59` | — | — |
| `readme--api-based-planned` | section | — | API-based (planned) | `README.md:70` | — | - OpenAI API |
| `readme--self-hosted-planned` | section | — | Self-hosted (planned) | `README.md:76` | — | - Ollama |
| `readme--installation` | section | — | Installation | `README.md:83` | — | cargo build -p embeddenator-agent-mcp --release |
| `readme--building-from-source` | section | — | Building from Source | `README.md:85` | — | cargo build -p embeddenator-agent-mcp --release |
| `readme--vs-code-integration` | section | — | VS Code Integration | `README.md:97` | — | Add to your VS Code mcp.json: |
| `readme--usage` | section | — | Usage | `README.md:112` | — | { |
| `readme--basic-prompt` | section | — | Basic Prompt | `README.md:114` | — | { |
| `readme--parallel-prompt` | section | — | Parallel Prompt | `README.md:125` | — | { |
| `readme--consensus` | section | — | Consensus | `README.md:137` | — | { |
| `readme--workflow` | section | — | Workflow | `README.md:149` | — | { |
| `readme--cli-options` | section | — | CLI Options | `README.md:179` | — | agent-mcp [OPTIONS] |
| `readme--current-limitations-alpha` | section | — | Current Limitations (alpha) | `README.md:192` | — | This is a 0.1.0-alpha project — functional, but with real gaps behind a few README/architecture |
| `readme--license` | section | — | License | `README.md:219` | — | MIT |
| `readme--status-roadmap` | section | — | Status & roadmap | `README.md:223` | — | - [Assessment & gaps](docs/ASSESSMENT.md) |
| `assessment` | note | — | agent-mcp — Assessment & Gap Analysis | `docs/ASSESSMENT.md:1` | — | Date: 2026-07-08 |
| `assessment--1.-what-it-is-today` | section | — | 1. What it is today | `docs/ASSESSMENT.md:10` | — | - MCP tools for multi-provider prompting and simple workflows |
| `assessment--2.-maturity-2-5` | section | — | 2. Maturity: **2 / 5** | `docs/ASSESSMENT.md:18` | — | — |
| `assessment--3.-branches` | section | — | 3. Branches | `docs/ASSESSMENT.md:30` | — | — |
| `assessment--4.-gaps` | section | — | 4. Gaps | `docs/ASSESSMENT.md:40` | — | — |
| `assessment--5.-integration-recommendation` | section | — | 5. Integration recommendation | `docs/ASSESSMENT.md:53` | — | See [ROADMAP.md](ROADMAP.md). |
| `assessment--tero-index` | section | — | Tero index | `docs/ASSESSMENT.md:63` | — | Layer-1 citation index: [docs/tero-index/](tero-index/) (index.json, INDEX.md, MANIFEST.toml). |
| `localchecks` | section | — | Local checks (CI parity) | `docs/LOCAL_CHECKS.md:1` | — | GitHub Actions workflows in this repo are manual only (workflowdispatch). |
| `localchecks--run-everything-the-remote-job-would-run` | section | — | Run everything the remote job would run | `docs/LOCAL_CHECKS.md:6` | — | ./scripts/check.sh |
| `localchecks--tero-index` | section | — | Tero index | `docs/LOCAL_CHECKS.md:19` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root "$(pwd)" |
| `localchecks--from-a-checkout-that-can-see-the-generator-sibling-tero-mcp-recommended` | other | — | from a checkout that can see the generator (sibling tero-mcp recommended): | `docs/LOCAL_CHECKS.md:22` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root "$(pwd)" |
| `localchecks--or` | other | — | or: | `docs/LOCAL_CHECKS.md:24` | — | python3 scripts/generateteroindex.sh   # if present as a thin wrapper |
| `localchecks--remote-optional` | section | — | Remote (optional) | `docs/LOCAL_CHECKS.md:30` | — | In GitHub: Actions → CI → Run workflow. |
| `roadmap` | note | — | agent-mcp — Product Roadmap | `docs/ROADMAP.md:1` | Living (2026-07-08) | Status: Living (2026-07-08) |
| `roadmap--waves` | section | — | Waves | `docs/ROADMAP.md:10` | — | — |
| `roadmap--wave-a-honesty-alpha` | section | — | Wave A — Honesty alpha | `docs/ROADMAP.md:12` | — | — |
| `roadmap--wave-b-api-providers-primary-path` | section | — | Wave B — API providers (primary path) | `docs/ROADMAP.md:21` | — | — |
| `roadmap--wave-c-real-orchestration` | section | — | Wave C — Real orchestration | `docs/ROADMAP.md:30` | — | — |
| `roadmap--wave-d-ecosystem` | section | — | Wave D — Ecosystem | `docs/ROADMAP.md:39` | — | — |
| `roadmap--api-plan` | section | — | API plan | `docs/ROADMAP.md:49` | — | — |
| `roadmap--mcp-tools-target-stable-set` | section | — | MCP tools (target stable set) | `docs/ROADMAP.md:51` | — | — |
| `roadmap--provider-config-env-file-not-mcp-secrets` | section | — | Provider config (env / file — not MCP secrets) | `docs/ROADMAP.md:61` | — | [[providers]] |
| `roadmap--example-agent-mcp.toml` | other | — | example agent-mcp.toml | `docs/ROADMAP.md:64` | — | [[providers]] |
| `roadmap--response-envelope` | section | — | Response envelope | `docs/ROADMAP.md:78` | — | { |
| `roadmap--pr-plan` | section | — | PR plan | `docs/ROADMAP.md:95` | — | 1. Docs assessment + roadmap |
| `roadmap--non-goals` | section | — | Non-goals | `docs/ROADMAP.md:107` | — | - Replacing cabal’s primary agent loop |
| `readme-2` | other | — | Tero index (Layer 1) | `docs/tero-index/README.md:1` | — | Machine + human citation index for this repository. |
| `readme--regenerate` | section | — | Regenerate | `docs/tero-index/README.md:13` | — | python3 /path/to/tero-mcp/scripts/generateliteindex.py --root $(pwd) |
| `readme--or-if-tero-mcp-is-a-sibling` | other | — | or if tero-mcp is a sibling: | `docs/tero-index/README.md:17` | — | python3 ../tero-mcp/scripts/generateliteindex.py --root $(pwd) |
| `readme--serve-locally` | section | — | Serve locally | `docs/tero-index/README.md:21` | — | export TEROTOKENS=local-dev:refresh |

