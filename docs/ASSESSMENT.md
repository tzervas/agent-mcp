# agent-mcp — Assessment & gap analysis

**Date:** 2026-07-25 (refreshed; original pass 2026-07-08)  
**Crate:** multi-agent / multi-provider orchestration MCP (`embeddenator-agent-mcp` **0.2.1**)  
**Role:** Route / parallel / consensus / workflow prompts across providers via MCP  
**Overlap:** cabal-style `Provider` ABC (API brain) — **different layer**

Measured detail: [CURRENT-STATE.md](CURRENT-STATE.md). History: [DEVELOPMENT-PATH.md](DEVELOPMENT-PATH.md).

---

## 1. What it is today

- MCP tools for multi-provider prompting and simple workflows (seven `agent_*` tools)
- Protocol shell: official **rmcp** SDK, **stdio only** (since v0.2.0, PR #15)
- Backend **tied to browser automation (webpuppet-rs git dep)** — not first-class API keys
- Orchestration quality still alpha: parallel is sequential; consensus is longest-text +
  fixed score; human-review steps pause without resume
- Offline MCP handshake and catalogue/status tools are tested; **live** prompt paths are not

---

## 2. Maturity (qualitative)

Not a numeric score with false precision. Relative to a production multi-API orchestrator:

| Area | Notes |
|------|--------|
| Tool names / MCP shell | Solid offline path; rmcp + e2e stdio tests green (measured 2026-07-25) |
| Real multi-API providers | Missing — empty `api-providers` feature; issue **#5** |
| Orchestration quality | Alpha — see honesty gaps in CURRENT-STATE |
| Packaging | Git dep on webpuppet-rs (pinned rev); **Cargo.lock tracked** (PR #24); AGENTS/CLAUDE + INTEGRATIONS present |
| Cabal dependency | **Optional later only** — do not block this product on cabal |

---

## 3. Branches / integration

| Branch pattern | Notes |
|----------------|--------|
| `main` | Release trunk; badges and fleet-ci target it |
| `dev` | Integration branch (fleet standards); feature PRs should prefer `dev` when active |
| Historical `integration` | Used in 2026-07 doc promotion PRs #10–#14 |

Default branch observed via API: **`main`**.

---

## 4. Gaps

| Gap | Sev | Notes |
|-----|-----|--------|
| No first-class API-key providers (xAI/OpenAI/Anthropic HTTP) | High | Issue **#5**; Wave B in [ROADMAP.md](ROADMAP.md) |
| Browser auth / keyring complexity | High for headless | Lives mostly in webpuppet-rs |
| Consensus / parallel semantics dishonest if taken literally | High | Documented; still placeholder code |
| Workflow HITL incomplete | Med | Pause without resume |
| Live browser tools untested in CI | Med | Intentional (no live keys); needs mocks for coverage |
| MCP client chaining / self-hosted | Med | Issues **#7**, **#6**; empty features |
| Overlap confusion with cabal Provider | Docs | This assessment + INTEGRATIONS |
| ~~Cargo.lock untracked~~ | ~~Med~~ | **Closed** by PR **#24** |

---

## 5. Integration recommendation

| Consumer | Fit |
|----------|-----|
| cabal primary model | **No** — use cabal Provider + API keys |
| cabal optional “consult N web UIs” | Maybe later via MCP once honesty/API waves land |
| Prefer raw webpuppet-mcp | If only browser prompt needed and no orchestration |
| agent-harness | Compose by reference — [INTEGRATIONS.md](INTEGRATIONS.md) |

See [ROADMAP.md](ROADMAP.md).

## Tero index

Layer-1 citation index: [docs/tero-index/](tero-index/) (`index.json`, `INDEX.md`, `MANIFEST.toml`).
