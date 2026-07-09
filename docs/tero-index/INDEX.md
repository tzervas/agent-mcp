# agent-mcp — Tero Index (Layer 1)

> **Honesty:** Empirical/Declared — lite heading/line heuristic over markdown in agent-mcp via tero-mcp/scripts/generate_lite_index.py; source files are ground truth. Generated 2026-07-09.
> Use this index to find where to Read, not as authoritative ground truth.

- **Items:** 44
- **Flagged:** 0
- **item_tag:** `Empirical/Declared`
- **Machine index:** [`index.json`](./index.json)
- **Manifest:** [`MANIFEST.toml`](./MANIFEST.toml)

## doc (44 entries)

| Anchor | Kind | Id | Title | File:Line | Status | Summary |
|---|---|---|---|---|---|---|
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

