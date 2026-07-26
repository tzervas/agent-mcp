# agent-mcp — Product roadmap

**Status:** Living (refreshed 2026-07-25 against commit `568f325`)  
**North star:** Honest multi-provider **orchestration MCP** — real API backends as
primary path, browser providers optional and explicit, metrics-backed consensus.

Companions: [CURRENT-STATE.md](CURRENT-STATE.md) (measured) ·
[DEVELOPMENT-PATH.md](DEVELOPMENT-PATH.md) (history) · [ASSESSMENT.md](ASSESSMENT.md).

**Rules for this file:** no invented dates or completion percentages. Each item
states **what would unblock it**. Speculative items are marked *proposed, not
committed*.

---

## Done or largely done (do not re-plan as net-new)

| Item | Evidence |
|------|----------|
| Document actual parallel/consensus/HITL behavior | README Current Limitations; `src/orchestrator.rs` comments; ASSESSMENT |
| Git dep on webpuppet-rs (no sibling path) | PR **#9**; `Cargo.toml` rev pin |
| CI / local build without live browser keys | Unit + integration + e2e; tests never call live providers |
| Official rmcp protocol shell | PR **#15**, tag **v0.2.0** |
| Track Cargo.lock; clear known rmcp HIGH CVE | PR **#24** |
| Compose-by-reference docs (harness / relay) | [INTEGRATIONS.md](INTEGRATIONS.md), PR **#20** |
| Fleet CI + security badges on trunk | PR **#18–#22**, README badges |

---

## Near-term — honesty and unblocking API work

### H1 — Keep CURRENT-STATE honest on every behavior PR

| | |
|--|--|
| **What** | When parallel/consensus/HITL/provider modality changes, update [CURRENT-STATE.md](CURRENT-STATE.md) + README limitations in the same PR. |
| **Why** | Stale capability claims hide bugs (fleet contract §5a). |
| **Unblock** | Process only — already measurable via `./scripts/check.sh` + this suite. |

### H2 — Feature-flag honesty for empty features

| | |
|--|--|
| **What** | `api-providers`, `self-hosted`, `mcp-client` are empty `[]` features. Either implement or document in user-facing README that enabling them does nothing. |
| **Why** | Cargo features look like capabilities. |
| **Unblock** | Decision: implement Wave B first vs. rename/remove flags until code exists. |

### H3 — Align CHANGELOG rmcp version wording with lockfile

| | |
|--|--|
| **What** | CHANGELOG 0.2.0 still says “rmcp SDK (0.8)” while builds resolve **rmcp 2.2.0** (post-CVE bump). |
| **Why** | Version drift trains distrust of release notes. |
| **Unblock** | Docs/changelog edit in a non-docs-only or allowed changelog PR (this PM suite deliberately did not touch `CHANGELOG.md`). |

---

## Wave B — API providers (primary path)

Tracked by issue **#5**.

| ID | Work | Why | What would unblock it |
|----|------|-----|------------------------|
| B1 | Provider trait: `complete(messages) -> text` (or equivalent) | Decouple orchestration from browser session lifecycle | Design decision on trait ownership (this crate vs shared); avoid path deps on sibling crates |
| B2 | xAI / OpenAI-compatible / Anthropic HTTP backends | Real multi-provider without Chromium | B1; env-based secrets only (`XAI_API_KEY`, etc.) — never tool args |
| B3 | `list_providers` reports modality `api` \| `browser` | Hosts can filter offline-safe vs session-backed | B2 for at least one API backend |
| B4 | Integration tests with mocked HTTP (no live keys required) | Gate regressions without fleet secrets | B1–B2; wiremock/httpmock or similar |

**Dependency:** none on browser auth once B2 lands; may keep webpuppet behind
`web-providers` (already default).

---

## Wave C — Real orchestration

| ID | Work | Why | What would unblock it |
|----|------|-----|------------------------|
| C1 | True parallel (`tokio::join` / `FuturesUnordered`) with deadlines | Name matches behavior; lower latency for API backends | **B2** (browser path likely stays sequential per session) |
| C2 | Consensus strategies: vote / embed-similarity / judge-model (documented) | Replace longest-text + hardcoded `0.5` | C1 useful; strategy enum in tool schema; tests with fixtures |
| C3 | Workflow HITL resume (`NEEDS_INPUT` + submit) | `review` step is currently a dead end | API design for resume tool or step arg; persist workflow state across process restarts (decision) |
| C4 | Structured traces per hop (latency, provider, error) | Debug multi-provider failures | Logging/envelope shape (see API plan below) |

---

## Wave D — Ecosystem

| ID | Work | Why | What would unblock it |
|----|------|-----|------------------------|
| D1 | Optional security-mcp screen on I/O | Untrusted text through agent tools | Host config docs only at first; optional process pairing ([INTEGRATIONS.md](INTEGRATIONS.md)) |
| D2 | agent-harness spawn-profile examples kept in sync | Compose-by-reference must not rot | Coordinate with harness releases; no vendoring |
| D3 | MCP schema freeze announcement for a 0.2.x line | Hosts pin tool shapes | Human decision after B/C stabilize; **still 0.x — no 1.x without human authorization** |
| D4 | Self-hosted backends (Ollama, vLLM, LocalAI, …) | Issue **#6** | OpenAI-compatible client from B2 likely reuses; local process discovery |
| D5 | MCP client-mode chaining | Issue **#7**; empty `mcp-client` feature | rmcp client features + allowlist of downstream servers; security review |

Items D4–D5 are **committed as open issues**, not speculative. Ordering after B is
recommended but not mandated by this doc alone.

---

## Known defects to close (from measurement)

See [CURRENT-STATE.md](CURRENT-STATE.md) for full evidence. Short list:

| Defect | Unblock |
|--------|---------|
| Sequential “parallel” | C1 + API providers |
| Placeholder consensus score | C2 |
| HITL pause without resume | C3 |
| Live browser path untested in CI | Policy choice: mock webpuppet trait **or** optional secret-gated job (prefer mocks) |
| `close-issues-on-main` red on some non-main events | Workflow/condition fix in `.github/` (out of scope for docs-only PRs) |

---

## API plan (target shape — proposed, not committed)

### MCP tools (stable *names* today; schemas may still move under 0.x)

| Tool | Purpose | Key args |
|------|---------|----------|
| `agent_list_providers` | Inventory | — |
| `agent_prompt` | Single provider complete | `provider?`, `message`, `context?` |
| `agent_parallel_prompt` | N providers | `providers[]`, `message` |
| `agent_consensus` | Aggregate | `message`, `min_providers?`, future `strategy` |
| `agent_workflow_start` / `agent_workflow_step` | Multi-step | workflow id, step defs |
| `agent_status` | Stats | — |

### Provider config (env / file — not MCP secrets)

```toml
# example agent-mcp.toml — proposed, not shipped
[[providers]]
name = "grok"
kind = "openai_compat"
base_url = "https://api.x.ai/v1"
model = "grok-4.5"
api_key_env = "XAI_API_KEY"

[[providers]]
name = "browser-chatgpt"
kind = "webpuppet"
enabled = false
```

### Response envelope (proposed)

```json
{
  "provider": "grok",
  "modality": "api",
  "text": "...",
  "latency_ms": 1200,
  "usage": { "input_tokens": 0, "output_tokens": 0 }
}
```

Parallel: `{ "results": [ ... ], "errors": [ ... ] }`  
Consensus: `{ "aggregate": "...", "strategy": "judge", "members": [ ... ] }`

---

## Suggested PR sequence (engineering, not calendar)

1. Provider trait + one OpenAI-compatible backend + mock tests (**#5** / B1–B4)  
2. True parallel + timeouts for API modality (C1)  
3. Consensus strategies with explicit strategy field (C2)  
4. Workflow HITL resume (C3)  
5. Self-hosted via same HTTP client (**#6** / D4)  
6. Optional MCP client chaining (**#7** / D5)  
7. Schema freeze note on a 0.2.x release (D3) — still not 1.x  

---

## Non-goals

- Replacing cabal’s primary agent loop  
- Silent browser automation as the unmarked default for production  
- Fake consensus scores  
- Agent-driven **1.x.x** release (human authorization required; fleet contract §3)  
- Vendoring harness, relay, or webpuppet into this tree  

---

## Signals used to build this roadmap

- Open issues: `#5`, `#6`, `#7` (`gh issue list`)  
- Empty Cargo features and TODO-shaped comments in `src/orchestrator.rs`  
- Gaps measured in [CURRENT-STATE.md](CURRENT-STATE.md) (2026-07-25)  
- Prior wave plan in this file (2026-07-08) retained where still accurate  
