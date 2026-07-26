# Integrations — compose with agent-harness and friends

`agent-mcp` is a **stdio MCP product**. Other repos consume it by reference; they
should not re-home the orchestrator or webpuppet stack.

## agent-harness

| Field | Value |
|-------|--------|
| Repo | https://github.com/tzervas/agent-harness |
| Role | Universal multi-agent harness (spawn / doctor / swarm CLI) |
| Relationship | Harness **evaluates and composes** this MCP; does not vendor it |

### Compose pattern

1. Build or install `agent-mcp` from this repo (or a release binary).
2. Register the server in the host MCP config (see [mcp.example.json](mcp.example.json)
   or [../.mcp.json.example](../.mcp.json.example)).
3. In harness docs / spawn profiles, point at the tool names:

   | Tool | Typical harness use |
   |------|---------------------|
   | `agent_status` | Health / dry-run without browser |
   | `agent_list_providers` | Catalogue before routing |
   | `agent_prompt` | Single-provider task |
   | `agent_parallel_prompt` / `agent_consensus` | Multi-provider (sequential today) |
   | `agent_workflow_*` | Multi-step automation (HITL incomplete) |

4. Offline harness tests should **mock** MCP tools — never require live browser
   keys in CI. See agent-harness `docs/INTEGRATIONS.md`.

### Example host fragment (Cursor / VS Code)

```json
{
  "servers": {
    "agent-mcp": {
      "type": "stdio",
      "command": "agent-mcp",
      "args": []
    }
  }
}
```

Optional: `"args": ["--visible"]` when debugging browser sessions.

## webpuppet-rs

| Field | Value |
|-------|--------|
| Repo | https://github.com/tzervas/webpuppet-rs |
| Cargo | git dependency `embeddenator-webpuppet` (pinned `rev` in `Cargo.toml`) |
| Role | Browser automation backend for Claude / ChatGPT / Gemini / Grok / … |

First `cargo build` fetches the git dep over the network. No sibling path checkout
is required for a standalone clone.

## tg-agent-relay

Telegram / phone bridge stays in **tg-agent-relay**. This MCP is not a Telegram
runtime. Relay and harness may call `agent-mcp` tools as consumers.

## security-mcp (optional)

Pair with [security-mcp](https://github.com/tzervas/security-mcp) when screening
untrusted text that flows through agent tools. Separate process; opt-in via host
MCP config (`security-mcp --stdio`).

## Non-goals

- Vendoring harness, relay, or webpuppet sources into this tree
- Claiming API-key multi-provider production readiness (browser path only today)
- Coupling release tags 1:1 with harness releases
