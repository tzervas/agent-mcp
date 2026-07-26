# Current state — agent-mcp

**Back layer. MEASURED, not aspirational.**

| Field | Value |
|-------|--------|
| **Measured at (UTC)** | 2026-07-25 |
| **Commit** | `568f32519488e6f227fa1e3139041f334cc5b7ae` (`main` tip at measurement) |
| **Crate / version** | `embeddenator-agent-mcp` **0.2.1** (`Cargo.toml`) |
| **Binary** | `agent-mcp` |
| **Toolchain used** | `rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)`, `cargo 1.98.0-nightly (a335d47ff 2026-06-26)` |
| **Resource limit** | `CARGO_BUILD_JOBS=3` for all cargo invocations |

Tag every capability **VERIFIED** (exercised in this measurement or covered by a
passing test we ran) or **UNVERIFIED** (not exercised; say why).

---

## Snapshot

Early **0.2.x** MCP server: official **rmcp** stdio shell, seven `agent_*` tools,
orchestration over **browser providers only** (webpuppet-rs git dep). Local
build, clippy, and offline tests are green. Live multi-provider prompting needs
a real browser session and is **not** covered by CI tests.

---

## Capability matrix

| Capability | Status | Evidence |
|------------|--------|----------|
| Compile / link binary `agent-mcp` | **VERIFIED** | `cargo build` exit 0 (~1m 21s) |
| Unit tests (router, workflow, tool parse/render) | **VERIFIED** | 12 passed |
| Integration: in-memory MCP handshake + offline tools | **VERIFIED** | 5 passed (`tests/integration_handshake.rs`) |
| E2E: real stdio binary handshake + `agent_list_providers` | **VERIFIED** | 1 passed (`tests/e2e_stdio.rs`) |
| `cargo fmt --check` | **VERIFIED** | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | **VERIFIED** | exit 0 |
| CLI `--help` / `--version` | **VERIFIED** | `agent-mcp 0.2.1`; options match README |
| Advertise 7 tools over MCP | **VERIFIED** | e2e asserts `tools.len() == 7` and `agent_prompt` listed |
| `agent_status` without browser | **VERIFIED** | integration test `call_tool_status_succeeds_without_external_calls` |
| `agent_list_providers` without browser | **VERIFIED** | integration + e2e; text contains `Available AI Providers` |
| Unknown tool name → error (not silent) | **VERIFIED** | `call_tool_unknown_name_is_never_silent_error` |
| Provider alias parse / reject unknown | **VERIFIED** | unit tests in `src/tools/tests.rs` |
| Workflow create / advance (in-memory state machine) | **VERIFIED** | unit tests in `src/workflow.rs` |
| Router task-type heuristics | **VERIFIED** | unit tests in `src/router.rs` |
| `agent_prompt` against live provider | **UNVERIFIED** | No test with live browser keys (by design). Needs webpuppet + authenticated Chromium session. |
| `agent_parallel_prompt` true concurrency | **UNVERIFIED** as “parallel”; **documented sequential** | Source: `parallel_prompt` loops providers one-by-one (`src/orchestrator.rs`). No live multi-provider test. |
| `agent_consensus` semantic agreement | **UNVERIFIED** as real consensus | Source: longest response + hardcoded `agreement_score: 0.5`. No live test. |
| Human-review workflow resume | **UNVERIFIED** / incomplete | `HumanReview` sets `Paused` and returns `Err(… waiting for human review)`; no resume API. |
| API-key providers (`api-providers` feature) | **UNVERIFIED** — empty feature | `Cargo.toml`: `api-providers = []` with no code path. |
| Self-hosted backends (`self-hosted` feature) | **UNVERIFIED** — empty feature | `Cargo.toml`: `self-hosted = []`. Issue **#6** open. |
| MCP client-mode chaining (`mcp-client`) | **UNVERIFIED** — empty feature | `Cargo.toml`: `mcp-client = []`. Issue **#7** open. |
| Content screening / rate limiting | **UNVERIFIED** — absent | README limitations; no module found in tree. |
| Docker / GHCR image build | **UNVERIFIED** in this docs run | `Dockerfile` present; image build not executed here. |
| crates.io publish | **UNVERIFIED** / not claimed | GitHub Releases only observed. |

---

## How this was measured

### Environment

```text
$ git rev-parse HEAD
568f32519488e6f227fa1e3139041f334cc5b7ae

$ date -u +%Y-%m-%dT%H:%M:%SZ
2026-07-25T23:21:00Z

$ rustc --version
rustc 1.98.0-nightly (4c9d2bfe4 2026-07-01)

$ cargo --version
cargo 1.98.0-nightly (a335d47ff 2026-06-26)

$ export CARGO_BUILD_JOBS=3
```

### Build

```text
$ cargo build
… (compiled deps including webpuppet @ rev bff694c, rmcp 2.2.0) …
   Compiling embeddenator-agent-mcp v0.2.1 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 21s
EXIT: 0
```

### Tests

```text
$ cargo test --all-features
    Finished `test` profile [unoptimized + debuginfo] target(s) in 38.15s

     Running unittests src/lib.rs
running 12 tests
… all ok …
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running unittests src/main.rs
running 0 tests
test result: ok. 0 passed; 0 failed; …

     Running tests/e2e_stdio.rs
running 1 test
test stdio_server_handshake_list_and_call ... ok
test result: ok. 1 passed; 0 failed; …

     Running tests/integration_handshake.rs
running 5 tests
… all ok …
test result: ok. 5 passed; 0 failed; …

   Doc-tests embeddenator-agent-mcp
running 0 tests
test result: ok. 0 passed; 0 failed; …

TEST_EXIT: 0
```

**Totals observed:** **18 tests passed** (12 lib unit + 1 e2e + 5 integration),
0 failed, 0 ignored. Bin target contributes 0 unit tests.

### Format and lint

```text
$ cargo fmt --check
FMT_EXIT: 0

$ cargo clippy --all-targets --all-features -- -D warnings
… Checking embeddenator-agent-mcp v0.2.1 …
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 05s
CLIPPY_EXIT: 0
```

### Runnable CLI examples (actually run)

```text
$ cargo run -- --help
Multi-agent orchestration MCP server for VS Code/GitHub Copilot

Usage: agent-mcp [OPTIONS]

Options:
      --visible                Run browser in visible (non-headless) mode
      --log-level <LOG_LEVEL>  Log level (trace, debug, info, warn, error) [default: info]
      --json-logs              Output logs as JSON
  -h, --help                   Print help
  -V, --version                Print version
HELP_EXIT: 0

$ cargo run -- --version
agent-mcp 0.2.1
VER_EXIT: 0
```

**Note:** `cargo run --` with no args starts the stdio MCP server and blocks
waiting for JSON-RPC on stdin (no banner on stdout). That long-lived wait was
**not** left running in this measurement; host attachment is covered by e2e
stdio tests instead.

### Full local gate script

`./scripts/check.sh` is fmt + clippy + build + test (same steps as above). The
individual steps were run successfully; the wrapper script was not re-run as a
single process after those four already passed (would duplicate ~3+ minutes of
compile work on a contended builder). Treat individual-step green as the
measurement; if the wrapper is required for your process, run:

```bash
export CARGO_BUILD_JOBS=3
./scripts/check.sh
```

### Remote CI (GitHub Actions)

Command:

```bash
gh api /repos/tzervas/agent-mcp/actions/runs?per_page=10 \
  --jq '.workflow_runs[] | {id, name, status, conclusion, head_branch, created_at}'
```

Recent runs observed (trimmed):

| When (UTC) | Workflow | Branch | Conclusion |
|------------|----------|--------|------------|
| 2026-07-23T04:52:58Z | CI | `main` | **success** (head `568f325…`) |
| 2026-07-23T04:52:58Z | fleet-ci | `main` | **success** |
| 2026-07-23T04:52:57Z | fleet-security | `main` | **success** |
| 2026-07-23T04:52:58Z | close-issues-on-main | `claude/harden-supply-chain` | **failure** |
| 2026-07-23T04:52:58Z | reopen-issues-closed-off-main | same | success |
| 2026-07-23T03:34:52Z | CI / fleet-ci on PR branch | `claude/harden-supply-chain` | **failure** (pre-merge; main tip later green) |
| 2026-07-21T23:07:24Z | fleet-ci / fleet-security | `main` | success |

**Interpretation:** trunk (`main` at measurement SHA) reports green CI, fleet-ci,
and fleet-security. Some PR-branch / issue-automation jobs failed around the same
merge window; those are separate from the green main tip.

Workflows are **not** manual-only: `ci.yml` and `fleet-ci.yml` trigger on
`push`/`pull_request` to `main`/`dev` (and related) as well as `workflow_dispatch`.
Earlier prose claiming manual-only remote CI is **stale** (corrected in
[LOCAL_CHECKS.md](LOCAL_CHECKS.md)).

---

## What the tools actually do (code-backed)

| Tool | Offline / test coverage | Runtime behavior today |
|------|-------------------------|------------------------|
| `agent_list_providers` | Yes (integration + e2e) | Static catalogue render (claude, grok, gemini, chatgpt, perplexity, notebooklm). |
| `agent_status` | Yes (integration) | Orchestrator stats; no browser required. |
| `agent_prompt` | Schema/parse only in unit tests | Calls webpuppet authenticate + prompt. **Live path UNVERIFIED** here. |
| `agent_parallel_prompt` | No live test | Sequential loop over providers; needs ≥2 valid provider names. |
| `agent_consensus` | Render unit test only | Uses parallel path then longest-text + fixed 0.5 score. |
| `agent_workflow_start` | No MCP integration test for start | Registers workflow; returns ID string. |
| `agent_workflow_step` | No MCP integration test for step | Runs prompt/parallel/consensus steps; human review errors out paused. |

---

## Known defects and gaps (observed, not fixed in this docs PR)

| Item | Severity | Evidence |
|------|----------|----------|
| “Parallel” is sequential | Product honesty / performance | `src/orchestrator.rs` comment + `for provider in providers` |
| Consensus score hardcoded `0.5` | Misleading if hosts display it as measured | `agreement_score: 0.5, // Placeholder` |
| HITL review never resumes | Broken step type | `WorkflowState::Paused` + error return; no submit API |
| Empty feature flags still advertised in Cargo features | False capability surface if someone enables them expecting code | `api-providers = []`, `self-hosted = []`, `mcp-client = []` |
| Live provider tools untested in CI | Regression risk for browser path | Test policy intentionally avoids live keys |
| `close-issues-on-main` failure on PR branch run | Process noise | Actions run `29980670391` conclusion failure |
| CHANGELOG 0.2.0 still says “rmcp SDK (0.8)” while lockfile resolves **rmcp 2.2.0** | Doc drift (CHANGELOG not edited in this docs-only pass) | `Cargo.lock` / build log vs CHANGELOG wording |
| No required status checks on `main` (fleet dispatcher measurement) | Unverified merges possible if auto-merge armed | Operator brief; do not arm auto-merge |

Open issues (signal for roadmap):

- **#5** API-based providers  
- **#6** Self-hosted backends  
- **#7** MCP client-mode chaining  

---

## Dependencies that shape reality

| Dep | Role | Pin / note |
|-----|------|------------|
| `rmcp` 2.2 | MCP shell | Features: server, transport-io, macros (+ client in dev-deps for tests) |
| `embeddenator-webpuppet` / package `webpuppet` | Browser providers | git `https://github.com/tzervas/webpuppet-rs`, rev `bff694c` |
| MSRV | `rust-version = "1.85"` | Transitive edition-2024 floor from rmcp family |

---

## Re-measure checklist

```bash
export CARGO_BUILD_JOBS=3
git rev-parse HEAD
cargo build
cargo test --all-features
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --version
gh api /repos/tzervas/agent-mcp/actions/runs?per_page=5 \
  --jq '.workflow_runs[] | {name, conclusion, head_branch, created_at}'
```

If any step fails, update this file’s date, SHA, and VERIFIED/UNVERIFIED rows
in the same PR that changes behavior — stale CURRENT-STATE is worse than none.
