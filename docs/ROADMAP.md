# agent-mcp — Product Roadmap

**Status:** Living (2026-07-08)  
**North star:** Honest multi-provider **orchestration MCP** — real API backends first, browser providers optional and explicit, metrics-backed consensus.

Companion: [ASSESSMENT.md](ASSESSMENT.md).

---

## Waves

### Wave A — Honesty alpha

| ID | Work |
|----|------|
| A1 | Document actual parallel/consensus behavior |
| A2 | Feature-flag browser providers vs API providers |
| A3 | CI build without requiring local browser |
| A4 | Align README path deps with git deps |

### Wave B — API providers (primary path)

| ID | Work |
|----|------|
| B1 | Provider trait: `complete(messages) -> text` |
| B2 | xAI / OpenAI-compatible / Anthropic HTTP backends |
| B3 | Env-based keys only (`XAI_API_KEY`, etc.) — never tool args |
| B4 | `list_providers` reports modality: `api` \| `browser` |

### Wave C — Real orchestration

| ID | Work |
|----|------|
| C1 | True parallel (`tokio::join` / futures) with deadlines |
| C2 | Consensus: vote / embed-similarity / judge-model (documented) |
| C3 | Workflow resume + human step (NEEDS_INPUT event) |
| C4 | Structured traces for each hop |

### Wave D — Ecosystem

| ID | Work |
|----|------|
| D1 | Optional security-mcp screen on I/O |
| D2 | cabal-devmelopner adapter docs (MCP client) |
| D3 | 0.2.0 stable MCP schema freeze |

---

## API plan

### MCP tools (target stable set)

| Tool | Purpose | Key args |
|------|---------|----------|
| `agent_list_providers` | Inventory | — |
| `agent_prompt` | Single provider complete | `provider`, `prompt`, `system?` |
| `agent_parallel_prompt` | N providers | `providers[]`, `prompt`, `timeout_ms` |
| `agent_consensus` | Aggregate | `providers[]`, `prompt`, `strategy` |
| `workflow_start` / `workflow_step` / `workflow_status` | Multi-step | workflow id, step input |

### Provider config (env / file — not MCP secrets)

```toml
# example agent-mcp.toml
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

### Response envelope

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

## PR plan

1. Docs assessment + roadmap  
2. Honesty + CI without browser  
3. API provider trait + xAI/OpenAI  
4. True parallel + timeouts  
5. Consensus strategies  
6. Workflow HITL  
7. Schema freeze 0.2.0  

---

## Non-goals

- Replacing cabal’s primary agent loop  
- Silent browser automation as default  
- Fake consensus scores  
