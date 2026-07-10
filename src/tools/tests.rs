//! Unit tests for the tool input schemas, rendering, and helpers.
//!
//! White-box (`use super::*`): these exercise the pure, deterministic half of the
//! tool surface — provider parsing, result rendering, and schema derivation — with
//! table-driven fixtures. The MCP wire behaviour is covered by the integration and
//! e2e tests under `tests/`. No browser / provider calls happen here.

use super::*;
use std::collections::HashMap;

use crate::orchestrator::{ConsensusResult, OrchestratorStatus};
use crate::router::ProviderStats;
use crate::workflow::ProviderResponse;
use embeddenator_webpuppet::Provider;

// ---- parse_provider (parameterized) ----------------------------------------

#[test]
fn parse_provider_accepts_known_aliases() {
    let cases = [
        ("claude", Provider::Claude),
        ("CLAUDE", Provider::Claude),
        ("grok", Provider::Grok),
        ("gemini", Provider::Gemini),
        ("chatgpt", Provider::ChatGpt),
        ("openai", Provider::ChatGpt),
        ("perplexity", Provider::Perplexity),
        ("notebooklm", Provider::NotebookLm),
        ("notebook", Provider::NotebookLm),
    ];
    for (input, expected) in cases {
        let got = parse_provider(input).unwrap_or_else(|_| panic!("{input} should parse"));
        assert_eq!(got, expected, "parsing {input}");
    }
}

#[test]
fn parse_provider_rejects_unknown_never_silent() {
    // Never-silent (G2): unknown input is an explicit error, not a silent default.
    let err = parse_provider("bard").unwrap_err();
    assert!(
        matches!(err, Error::InvalidParams(_)),
        "unknown provider must be InvalidParams, got {err:?}"
    );
    assert!(err.to_string().contains("bard"));
}

// ---- render_providers -------------------------------------------------------

#[test]
fn render_providers_lists_every_provider() {
    let out = render_providers();
    for id in [
        "claude",
        "grok",
        "gemini",
        "chatgpt",
        "perplexity",
        "notebooklm",
    ] {
        assert!(out.contains(id), "provider catalogue missing `{id}`");
    }
}

// ---- render_prompt ----------------------------------------------------------

#[test]
fn render_prompt_includes_provider_and_text() {
    let out = render_prompt(&Provider::Claude, "hello world");
    assert!(out.contains("hello world"));
    assert!(out.to_lowercase().contains("claude"));
}

// ---- render_consensus -------------------------------------------------------

fn consensus_fixture() -> ConsensusResult {
    ConsensusResult {
        consensus_text: "the answer is 42".into(),
        agreement_score: 0.75,
        responses: vec![
            ProviderResponse {
                provider: "claude".into(),
                text: "42".into(),
                selected: true,
                confidence: Some(0.9),
            },
            ProviderResponse {
                provider: "grok".into(),
                text: "forty-two".into(),
                selected: false,
                confidence: None,
            },
        ],
    }
}

#[test]
fn render_consensus_shows_score_and_selection_marker() {
    let out = render_consensus(&consensus_fixture());
    assert!(out.contains("75%"), "agreement score should render as 75%");
    assert!(out.contains("the answer is 42"));
    assert!(
        out.contains('\u{2713}'),
        "selected response gets a check mark"
    );
    assert!(
        out.contains('\u{25cb}'),
        "unselected response gets a hollow marker"
    );
}

// ---- render_status ----------------------------------------------------------

#[test]
fn render_status_empty_stats_says_no_requests() {
    let status = OrchestratorStatus {
        available_providers: vec![Provider::Claude],
        active_workflows: 0,
        provider_stats: HashMap::new(),
    };
    let out = render_status(&status);
    assert!(out.contains("No requests yet"));
    assert!(out.contains("claude") || out.contains("Claude"));
}

#[test]
fn render_status_renders_provider_stats() {
    let mut stats = HashMap::new();
    stats.insert(
        Provider::Claude,
        ProviderStats {
            total_requests: 3,
            successful_requests: 2,
            failed_requests: 1,
            total_tokens: None,
        },
    );
    let status = OrchestratorStatus {
        available_providers: vec![Provider::Claude],
        active_workflows: 2,
        provider_stats: stats,
    };
    let out = render_status(&status);
    assert!(out.contains("3 total"));
    assert!(out.contains("2 success"));
    assert!(out.contains("1 failed"));
}

// ---- schema derivation ------------------------------------------------------

#[test]
fn prompt_args_schema_marks_message_required() {
    // rmcp derives the tools/list schema from these types via schemars; assert the
    // required/optional split the client sees is what we intend.
    let schema = schemars::schema_for!(PromptArgs);
    let json = serde_json::to_value(&schema).unwrap();
    let required = json
        .get("required")
        .and_then(|r| r.as_array())
        .expect("schema should have a required array");
    let required: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    assert!(required.contains(&"message"), "message must be required");
    assert!(
        !required.contains(&"provider"),
        "provider is optional and must not be required"
    );
}
