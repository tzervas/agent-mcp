# Development path — agent-mcp

**Back-layer history.** How this repository got to its current shape, reconstructed
from git history, merged PRs, and releases. Cite short SHAs / PR numbers so a
reader can verify. Where a claim is inference rather than a direct citation, it
is labeled.

Companion: [CURRENT-STATE.md](CURRENT-STATE.md) (measured today) ·
[ROADMAP.md](ROADMAP.md) (planned work).

---

## Origin (split baseline)

| Evidence | What it shows |
|----------|----------------|
| `f019eb5` — *split baseline: workspace restructure + subcrates + benchmarks + tests* | Earliest commit on this line: product extracted as a standalone crate rather than remaining buried in a monorepo workspace. |
| Repo `created_at` `2026-01-04` (`gh api /repos/tzervas/agent-mcp`) | Public GitHub home established early 2026. |

**Inferred from those commits and the later honesty pass:** the product was always
meant as an **MCP host-facing orchestration shell** over browser-driven providers
(via what became [webpuppet-rs](https://github.com/tzervas/webpuppet-rs)), not as
a primary agent loop that owns model weights or a full IDE extension.

Early layout shipped a **hand-rolled MCP/JSON-RPC shell** (`src/protocol.rs`,
custom `match method` dispatch in `src/server.rs`) and seven `agent_*` tools
backed by webpuppet. That shell is gone as of v0.2.0; the tools and orchestration
modules remain the continuous product surface.

---

## Decision timeline

### 1. Standalone crate + manual-only CI (Jan–Jul early)

| Commit / PR | Decision |
|-------------|----------|
| `0adefde` *Add manual-only CI workflow* | Prefer local gates; remote CI not the only truth. |
| `ebbb1b9` *add standardized GitHub workflows* | Fleet-style workflow pack lands. |
| Dependabot PRs `#1`, `#3`, `#4` | Keep Actions action pins current. |

**Shape chosen:** Rust binary `agent-mcp`, crate name `embeddenator-agent-mcp`,
stdio MCP only (no HTTP server). **Rejected (later made explicit by dep cleanup):**
shipping an in-tree HTTP/axum transport for this product — stdio is the host
attachment model for Cursor / VS Code / Claude Desktop.

### 2. Honesty alpha (2026-07-08) — docs and dependency truth

| PR | Title | Effect |
|----|-------|--------|
| **#8** | public-ready pass (honest README, secret scan, gap issues) | README Current Limitations; gap issues **#5** (API providers), **#6** (self-hosted), **#7** (MCP client chaining). |
| **#9** | repoint webpuppet from path dep to **webpuppet-rs git dep** | Standalone clone builds without a sibling checkout. |

**Decision:** tell the truth about parallel/consensus/HITL rather than paper over
gaps. **Decision:** pin `embeddenator-webpuppet` / package `webpuppet` as a **git
dependency** (`rev` in `Cargo.toml`), not a path dependency.

**Rejected alternative (evidence: PR #9 commit messages and README fix in 0.2.1):**
requiring `../webpuppet-rs` on disk. Path deps fail in CI and for first-time
clones (`cargo metadata` aborts before build). Fleet contract later restated the
same rule globally.

Supporting docs from the same wave: `87ed3e4` assessment + roadmap; `9c1f780`
Layer-1 Tero index.

### 3. Local check parity and branch hygiene (2026-07-09)

| PR | Effect |
|----|--------|
| **#12** | `scripts/check.sh` + refresh Tero; comment era of “manual-only remote CI.” |
| **#10–#14** | Promote docs through `dev` / `integration` / `main`. |

**Decision:** `./scripts/check.sh` is the day-to-day gate (fmt, clippy `-D warnings`,
build, test). Remote workflows remain useful but must not be the only story.

> **Note for readers of older docs:** after PR **#18** (self-hosted fleet CI) and
> push/PR triggers on `fleet-ci.yml` / `ci.yml`, “manual-only CI” is **no longer
> accurate**. See [LOCAL_CHECKS.md](LOCAL_CHECKS.md) and [CURRENT-STATE.md](CURRENT-STATE.md).

### 4. Semver baseline and rmcp protocol shell (2026-07-10) — major product turn

| Tag / PR | What landed |
|----------|-------------|
| `v0.1.0-alpha.1`, `v0.1.0` | Semver baseline under 0.x (commitizen / major-version-zero policy). |
| **#15** / `2bf8f30` | **Breaking for the shell:** adopt official [`rmcp`](https://crates.io/crates/rmcp) SDK. |
| **#16** | Release **v0.2.0** from `dev`. |
| **#17** | Align publish flow to GitHub Releases. |

**Decision (rmcp adoption) — evidence: PR #15 body and CHANGELOG 0.2.0:**

| Chose | Rejected | Why (stated in tree) |
|-------|----------|----------------------|
| `rmcp` `ServerHandler` + `#[tool_router]` / `#[tool]` + `serve_stdio` (`transport-io`) | Hand-rolled JSON-RPC types in `src/protocol.rs` and match-based dispatch | Framing, handshake, and `tools/list` schema derivation belong in the official SDK; less custom protocol debt. |
| Features: `server`, `transport-io`, `macros` only | OAuth/`auth` + `reqwest` HTTP server features | Product is **stdio-only** for MCP hosts. |
| Keep orchestration (`orchestrator`, `router`, `workflow`) and the seven tool *names/behavior* | Rewrite business logic in the same PR | Isolate protocol migration risk. |
| MSRV **1.85** (`rust-version`) | Stay on older toolchain | Transitive floor from edition-2024 `rmcp` (0.8-era crates at merge; lockfile later on **rmcp 2.2** after supply-chain bump). |
| Drop `axum` / `tower` / unused `http`/`stdio` cargo features | Keep unused HTTP surface “for later” | Dead code and false capability signal. |

**Tool surface continuity:** same seven tools. **Schema caveat (CHANGELOG):**
inputs are now `schemars`-derived; `provider` is a free string with server-side
rejection (no client-side enum dropdown). That was accepted as the cost of
generated schemas.

**Tests added with #15:** unit (parse/render/schema), integration (in-memory
`rmcp` client), e2e (spawn real binary over stdio). **Hard rule:** no live
browser keys in automated tests.

### 5. Fleet CI and production polish (2026-07-16 – 0.2.1)

| PR | Effect |
|----|--------|
| **#18** | Route linux x64 jobs to **self-hosted podman** fleet. |
| **#19**, **#21** | P26 standards: CI/security badges, issue-close **only on main**. |
| **#20** | P28e: `AGENTS.md`, `CLAUDE.md`, 5-minute README, MCP host examples, [INTEGRATIONS.md](INTEGRATIONS.md). |
| **#22** | Harden fleet gates for self-hosted catch-up. |
| **#23** | Release **v0.2.1**. |

**Decision:** compose with [agent-harness](https://github.com/tzervas/agent-harness)
**by reference**, not by vendoring. **Decision:** leave mycelium isolated
([AGENTS.md](../AGENTS.md)). **Decision:** issues stay open on feature/`dev`
merges (`Refs #n`); `Closes #n` only when merging to `main`
([FLEET_STANDARDS.md](FLEET_STANDARDS.md)).

### 6. Supply-chain harden (2026-07-23)

| PR | Effect |
|----|--------|
| **#24** `568f325` | Track **Cargo.lock**; clear **rmcp CVE-2026-42559** (HIGH); fix Dockerfile DS-0026. |

**Decision:** commit the lockfile so builds are reproducible and known CVEs can
be cleared with an intentional dep bump. This closed an earlier assessment gap
that treated missing lock tracking as medium risk.

---

## Architectural spine (what stayed constant)

Across the protocol rewrite, these product choices held:

1. **MCP product boundary** — this repo owns `agent_*` tools + stdio server; browser
   automation lives in webpuppet-rs; swarm CLI lives in agent-harness; Telegram
   runtime lives in tg-agent-relay.
2. **Browser providers only in practice** — `api-providers`, `self-hosted`, and
   `mcp-client` feature flags exist as empty placeholders in `Cargo.toml`
   (no implementation bodies).
3. **Honesty about orchestration quality** — sequential “parallel,” longest-text
   “consensus,” HITL pause without resume — documented in README and enforced in
   code comments (`src/orchestrator.rs`).
4. **0.x versioning** — no 1.x until a human authorizes production readiness
   (fleet branch/release contract).

---

## Releases (GitHub)

| Tag | Published | Role |
|-----|-----------|------|
| `v0.1.0-alpha.1` | 2026-07-10 | Semver baseline |
| `v0.1.0` | 2026-07-10 | Named 0.1 line |
| `v0.2.0` | 2026-07-10 | rmcp shell |
| `v0.2.1` | 2026-07-21 | Fleet polish / docs / CI harden |

Registry publication (crates.io) is **not** claimed here; GitHub Release ≠ crates.io
(fleet contract §4).

---

## What this path did *not* choose

| Non-goal | Signal |
|----------|--------|
| Cabal primary agent loop | [ASSESSMENT.md](ASSESSMENT.md) §5 — different layer |
| Silent browser automation as default production path | ROADMAP non-goals; honesty sections |
| Fake agreement scores presented as measured | Hardcoded `0.5` remains; docs call it out |
| Coupling release tags 1:1 with harness | [INTEGRATIONS.md](INTEGRATIONS.md) |

---

## How to re-check this narrative

```bash
git log --oneline | head -50
gh pr list --state merged --limit 30 --repo tzervas/agent-mcp
gh api /repos/tzervas/agent-mcp/releases --jq '.[].tag_name'
```
