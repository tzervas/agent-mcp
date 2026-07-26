# agent-mcp — Assessment & Gap Analysis

**Date:** 2026-07-08  
**Crate:** multi-agent / multi-provider orchestration MCP (alpha)  
**Role:** Route / parallel / consensus / workflow prompts across providers  
**Overlap:** cabal-devmelopner `Provider` ABC (API brain) — **different layer**

---

## 1. What it is today

- MCP tools for multi-provider prompting and simple workflows  
- Backend heavily tied to **browser automation (webpuppet)** rather than first-class API keys  
- Alpha quality: parallel may be sequential; consensus heuristics weak; human-review resume incomplete  

---

## 2. Maturity: **2 / 5**

| Area | Notes |
|------|--------|
| Tool names / MCP shell | Present |
| Real multi-API providers | Weak / browser path |
| Orchestration quality | Alpha |
| Packaging | Git dep on webpuppet-rs (pinned rev); AGENTS/CLAUDE + INTEGRATIONS docs present |
| Cabal dependency | **Optional later only** |

---

## 3. Branches

| Branch | Notes |
|--------|--------|
| `main`/`dev`/`integration` | Aligned (webpuppet git dep fix landed) |
| `claude/finish-agent-mcp` | Honesty docs candidate |
| `claude/fix-webpuppet-dep` | Likely merged-equivalent |

---

## 4. Gaps

| Gap | Sev |
|-----|-----|
| No first-class API-key providers (xAI/OpenAI/Anthropic HTTP) | High |
| Browser auth / keyring complexity | High for headless |
| Consensus/parallel semantics dishonest | High |
| Workflow HITL incomplete | Med |
| Cargo.lock / release binaries | Med |
| Overlap confusion with cabal Provider | Docs |

---

## 5. Integration recommendation

| Consumer | Fit |
|----------|-----|
| cabal primary model | **No** — use cabal Provider + API keys |
| cabal optional “consult N web UIs” | Maybe Wave D via MCP |
| Prefer raw webpuppet-mcp | If only browser prompt needed |

See [ROADMAP.md](ROADMAP.md).

## Tero index

Layer-1 citation index: [docs/tero-index/](tero-index/) (`index.json`, `INDEX.md`, `MANIFEST.toml`).
