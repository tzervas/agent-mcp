//! Integration tests: a real `rmcp` client drives the `AgentMcp` `ServerHandler`
//! over an **in-memory duplex transport** — no process spawn, no stdio.
//!
//! These assert the MCP wire contract the SDK now owns for us: the `initialize`
//! handshake, `tools/list` (name + schema surface), and `tools/call` on the two
//! tools that touch **no external browser** (`agent_list_providers`, `agent_status`).
//! Browser-backed tools (`agent_prompt`, …) are intentionally *not* called here —
//! that would require a live provider session (covered by neither layer; mocked out).

use embeddenator_agent_mcp::{AgentMcp, AgentOrchestrator};
use rmcp::model::CallToolRequestParams;
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

/// Call a tool and return its text content.
async fn call_text(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    tool: &'static str,
) -> String {
    let result = client
        .call_tool(CallToolRequestParams::new(tool))
        .await
        .unwrap_or_else(|e| panic!("tools/call {tool}: {e}"));
    assert_ne!(result.is_error, Some(true), "{tool} must succeed");
    result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .expect("text content")
}

/// The provider ids this binary can actually address.
fn compiled_provider_ids() -> Vec<&'static str> {
    embeddenator_webpuppet::Provider::all()
        .into_iter()
        .map(|p| p.name())
        .collect()
}

#[tokio::test]
async fn call_tool_list_providers_returns_catalogue() {
    let client = connect().await;
    let text = call_text(&client, "agent_list_providers").await;
    assert!(text.contains("claude") && text.contains("grok"));
    client.cancel().await.ok();
}

#[tokio::test]
async fn list_providers_reflects_the_binary_not_a_hardcoded_string() {
    // Regression: the tool used to return a hand-written six-entry string. It had
    // already drifted from reality — `kaggle` is compiled in and `agent_status`
    // reported it, but the catalogue never mentioned it.
    let client = connect().await;
    let text = call_text(&client, "agent_list_providers").await;

    for id in compiled_provider_ids() {
        assert!(
            text.contains(id),
            "inventory is missing `{id}`, so it is not enumerated from the binary:\n{text}"
        );
    }
    assert!(
        text.contains("not a hand-maintained list"),
        "the tool must state where its contents come from:\n{text}"
    );

    client.cancel().await.ok();
}

#[tokio::test]
async fn list_providers_declares_modality_per_provider() {
    // ROADMAP B4: modality is `api` | `browser`. Nothing is `api` yet, and the
    // tool has to say so rather than leave the caller to guess.
    let client = connect().await;
    let text = call_text(&client, "agent_list_providers").await;
    assert!(text.contains("modality: `browser`"), "{text}");
    assert!(
        text.contains("`api` modality") && text.contains("not implemented"),
        "the unimplemented api path must be named as unimplemented:\n{text}"
    );
    client.cancel().await.ok();
}

#[tokio::test]
async fn call_tool_status_succeeds_without_external_calls() {
    let client = connect().await;
    let text = call_text(&client, "agent_status").await;
    assert!(text.contains("Agent Orchestrator Status"));
    client.cancel().await.ok();
}

#[tokio::test]
async fn status_never_reports_availability_it_has_not_probed() {
    // This is the bug the change exists for. Over the wire, on a server that has
    // made zero requests, the old implementation returned:
    //     ## Available Providers
    //     - ✅ grok
    //     - ✅ claude          (... seven of them, on a host with no browser)
    let client = connect().await;
    let text = call_text(&client, "agent_status").await;

    assert!(
        !text.contains('\u{2705}'),
        "nothing has been probed, so no provider may carry a check mark:\n{text}"
    );
    assert!(
        text.contains("Verified available: 0 of"),
        "the number of *verified* providers must be stated:\n{text}"
    );
    assert!(
        text.contains("Not probed:"),
        "the report must say what it did not check:\n{text}"
    );
    // Every provider is still listed — this refuses to over-claim, it does not hide.
    for id in compiled_provider_ids() {
        assert!(text.contains(id), "agent_status omits `{id}`:\n{text}");
    }

    client.cancel().await.ok();
}

#[tokio::test]
async fn status_and_list_providers_agree_on_the_provider_set() {
    // The two tools disagreed before: agent_status listed 7 providers,
    // agent_list_providers listed 6.
    let client = connect().await;
    let status = call_text(&client, "agent_status").await;
    let providers = call_text(&client, "agent_list_providers").await;

    for id in compiled_provider_ids() {
        assert!(status.contains(id), "agent_status omits `{id}`");
        assert!(providers.contains(id), "agent_list_providers omits `{id}`");
    }

    client.cancel().await.ok();
}

#[tokio::test]
async fn call_tool_unknown_name_is_never_silent_error() {
    let client = connect().await;
    // rmcp routes tools/call; an unknown tool must surface as an error, not a
    // silent empty success (house rule: never-silent).
    let outcome = client
        .call_tool(CallToolRequestParams::new("agent_nonexistent"))
        .await;
    assert!(outcome.is_err(), "unknown tool must be a protocol error");

    client.cancel().await.ok();
}
