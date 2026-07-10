//! Integration tests: a real `rmcp` client drives the `AgentMcp` `ServerHandler`
//! over an **in-memory duplex transport** — no process spawn, no stdio.
//!
//! These assert the MCP wire contract the SDK now owns for us: the `initialize`
//! handshake, `tools/list` (name + schema surface), and `tools/call` on the two
//! tools that touch **no external browser** (`agent_list_providers`, `agent_status`).
//! Browser-backed tools (`agent_prompt`, …) are intentionally *not* called here —
//! that would require a live provider session (covered by neither layer; mocked out).

use embeddenator_agent_mcp::{AgentMcp, AgentOrchestrator};
use rmcp::model::CallToolRequestParam;
use rmcp::ServiceExt;

/// Spin up the server on one end of an in-memory duplex and an `()` client on the
/// other; returns the connected client peer.
async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, ()> {
    let (server_t, client_t) = tokio::io::duplex(4096);

    tokio::spawn(async move {
        let server = AgentMcp::new(AgentOrchestrator::new())
            .serve(server_t)
            .await
            .expect("server init");
        let _ = server.waiting().await;
    });

    ().serve(client_t).await.expect("client init")
}

const EXPECTED_TOOLS: [&str; 7] = [
    "agent_prompt",
    "agent_parallel_prompt",
    "agent_consensus",
    "agent_workflow_start",
    "agent_workflow_step",
    "agent_status",
    "agent_list_providers",
];

#[tokio::test]
async fn initialize_reports_server_info_and_tool_capability() {
    let client = connect().await;
    let info = client.peer_info().expect("server info after initialize");
    assert_eq!(info.server_info.name, "embeddenator-agent-mcp");
    assert!(
        info.capabilities.tools.is_some(),
        "server must advertise the tools capability"
    );
    client.cancel().await.ok();
}

#[tokio::test]
async fn tools_list_exposes_every_tool_with_a_schema() {
    let client = connect().await;
    let tools = client.list_all_tools().await.expect("tools/list");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    for expected in EXPECTED_TOOLS {
        assert!(names.contains(&expected), "tools/list missing `{expected}`");
    }
    assert_eq!(tools.len(), EXPECTED_TOOLS.len(), "unexpected tool count");

    // agent_prompt's derived schema must mark `message` required.
    let prompt = tools
        .iter()
        .find(|t| t.name == "agent_prompt")
        .expect("agent_prompt present");
    let required = prompt
        .input_schema
        .get("required")
        .and_then(|r| r.as_array())
        .expect("agent_prompt schema has required[]");
    assert!(
        required.iter().any(|v| v.as_str() == Some("message")),
        "agent_prompt.message must be required in the derived schema"
    );

    client.cancel().await.ok();
}

#[tokio::test]
async fn call_tool_list_providers_returns_catalogue() {
    let client = connect().await;
    let result = client
        .call_tool(CallToolRequestParam {
            name: "agent_list_providers".into(),
            arguments: None,
        })
        .await
        .expect("tools/call agent_list_providers");

    assert_ne!(result.is_error, Some(true), "list_providers must succeed");
    let text = result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("text content");
    assert!(text.contains("claude") && text.contains("grok"));

    client.cancel().await.ok();
}

#[tokio::test]
async fn call_tool_status_succeeds_without_external_calls() {
    let client = connect().await;
    let result = client
        .call_tool(CallToolRequestParam {
            name: "agent_status".into(),
            arguments: None,
        })
        .await
        .expect("tools/call agent_status");

    assert_ne!(result.is_error, Some(true));
    let text = result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("text content");
    assert!(text.contains("Agent Orchestrator Status"));

    client.cancel().await.ok();
}

#[tokio::test]
async fn call_tool_unknown_name_is_never_silent_error() {
    let client = connect().await;
    // rmcp routes tools/call; an unknown tool must surface as an error, not a
    // silent empty success (house rule: never-silent).
    let outcome = client
        .call_tool(CallToolRequestParam {
            name: "agent_nonexistent".into(),
            arguments: None,
        })
        .await;
    assert!(outcome.is_err(), "unknown tool must be a protocol error");

    client.cancel().await.ok();
}
