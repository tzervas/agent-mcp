//! End-to-end test: spawn the **real compiled `agent-mcp` binary** as a child
//! process and drive a full MCP handshake over its stdio, exactly as an MCP host
//! (VS Code / Copilot) would.
//!
//! This exercises the actual `serve_stdio` entrypoint + `rmcp` framing end to end.
//! It only calls `tools/list` and the browser-free `agent_list_providers` tool, so
//! no external provider/browser session is ever launched (VR-5/G2: no live-key
//! dependency in tests).

use rmcp::model::CallToolRequestParam;
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use rmcp::ServiceExt;

/// Path to the freshly-built binary, provided by Cargo for integration tests.
const BIN: &str = env!("CARGO_BIN_EXE_agent-mcp");

#[tokio::test]
async fn stdio_server_handshake_list_and_call() -> anyhow::Result<()> {
    let transport = TokioChildProcess::new(tokio::process::Command::new(BIN).configure(|cmd| {
        // Keep the child quiet on stderr; stdout carries the MCP protocol.
        cmd.arg("--log-level").arg("error");
    }))?;

    let client = ().serve(transport).await?;

    // Handshake happened during serve(); confirm server identity.
    let info = client.peer_info().expect("server info after initialize");
    assert_eq!(info.server_info.name, "embeddenator-agent-mcp");

    // tools/list over the wire.
    let tools = client.list_all_tools().await?;
    assert!(
        tools.iter().any(|t| t.name == "agent_prompt"),
        "agent_prompt should be listed"
    );
    assert_eq!(tools.len(), 7, "all 7 tools should be advertised");

    // tools/call on a browser-free tool.
    let result = client
        .call_tool(CallToolRequestParam {
            name: "agent_list_providers".into(),
            arguments: None,
        })
        .await?;
    assert_ne!(result.is_error, Some(true));
    let text = result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .unwrap_or_default();
    assert!(text.contains("Available AI Providers"));

    client.cancel().await?;
    Ok(())
}
