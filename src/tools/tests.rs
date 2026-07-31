//! Unit tests for the tool input schemas, rendering, and helpers.
//!
//! White-box (`use super::*`): these exercise the pure, deterministic half of the
//! tool surface — provider parsing, result rendering, and schema derivation — with
//! table-driven fixtures. The MCP wire behaviour is covered by the integration and
//! e2e tests under `tests/`. No browser / provider calls happen here.

use super::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::availability::{self, Availability, ProviderEvidence};
use crate::orchestrator::{ConsensusResult, OrchestratorStatus};
use crate::router::ProviderStats;
use crate::workflow::ProviderResponse;
use embeddenator_webpuppet::Provider;

/// A host that has a usable browser, so availability turns on request history
/// rather than on the host being empty.
fn host_with_browser() -> BrowserRuntime {
    BrowserRuntime {
        cdp_browsers: vec!["chromium 120".into()],
    }
}

/// The inventory a freshly started server really has: transport present,
/// nothing exercised, so every entry is `unknown`.
fn unprobed_inventory() -> Vec<ProviderEntry> {
    availability::inventory(&HashMap::new(), &host_with_browser())
}

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

#[test]
fn parse_provider_round_trips_every_compiled_provider() {
    // Drift guard. `agent_status` enumerates `Provider::all()`, so anything it can
    // report must also be something `agent_prompt` will accept. `kaggle` failed
    // this before the fix: status advertised it, parse_provider rejected it.
    for provider in Provider::all() {
        let parsed = parse_provider(provider.name()).unwrap_or_else(|e| {
            panic!(
                "`{}` is a compiled-in provider that the inventory advertises, \
                 but parse_provider rejects it: {e}",
                provider.name()
            )
        });
        assert_eq!(parsed, provider);
    }
}

// ---- render_providers -------------------------------------------------------

#[test]
fn render_providers_lists_every_compiled_provider() {
    let inventory = unprobed_inventory();
    let out = render_providers(&inventory, &host_with_browser());
    for provider in Provider::all() {
        assert!(
            out.contains(provider.name()),
            "provider inventory missing `{}`",
            provider.name()
        );
    }
}

#[test]
fn render_providers_declares_modality_per_provider() {
    // ROADMAP B4. The old hardcoded catalogue said nothing about how a provider
    // is reached, so a caller could not tell browser automation from an API call.
    let inventory = unprobed_inventory();
    let out = render_providers(&inventory, &host_with_browser());
    assert_eq!(
        out.matches("modality: `browser`").count(),
        inventory.len(),
        "every provider row must declare its modality"
    );
    assert!(
        out.contains("`api` modality") && out.contains("not implemented"),
        "the api modality must be named and marked unimplemented"
    );
}

#[test]
fn render_providers_never_claims_unprobed_availability() {
    let inventory = unprobed_inventory();
    let out = render_providers(&inventory, &host_with_browser());
    assert!(
        !out.contains('\u{2705}'),
        "nothing has been probed, so no row may carry a check mark:\n{out}"
    );
    assert_eq!(
        out.matches("availability: \u{2753} unknown").count(),
        inventory.len(),
        "every unprobed provider row must be reported as unknown"
    );
}

#[test]
fn render_providers_states_its_own_provenance() {
    let out = render_providers(&unprobed_inventory(), &host_with_browser());
    assert!(
        out.contains("Not probed:"),
        "the inventory must say what it did not check"
    );
    assert!(
        out.contains("not a hand-maintained list"),
        "the inventory must say where its contents come from"
    );
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

/// Status for a host with a browser but no request history.
fn unprobed_status(
    active_workflows: usize,
    stats: HashMap<Provider, ProviderStats>,
) -> OrchestratorStatus {
    OrchestratorStatus {
        available_providers: Vec::new(),
        inventory: unprobed_inventory(),
        browser_runtime: host_with_browser(),
        active_workflows,
        provider_stats: stats,
    }
}

#[test]
fn render_status_empty_stats_says_no_requests() {
    let out = render_status(&unprobed_status(0, HashMap::new()));
    assert!(out.contains("No requests yet"));
    assert!(out.contains("claude"));
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
    let out = render_status(&unprobed_status(2, stats));
    assert!(out.contains("3 total"));
    assert!(out.contains("2 success"));
    assert!(out.contains("1 failed"));
}

#[test]
fn render_status_never_marks_an_unprobed_provider_available() {
    // The bug this whole change exists for: the old renderer printed
    // "- ✅ <provider>" for all seven providers on a host with no browser at all.
    let out = render_status(&unprobed_status(0, HashMap::new()));
    assert!(
        !out.contains('\u{2705}'),
        "no provider was probed, so no check mark may appear:\n{out}"
    );
    assert!(
        out.contains("Verified available: 0 of"),
        "the count of *verified* providers must be stated explicitly:\n{out}"
    );
}

#[test]
fn render_status_gives_a_reason_for_every_provider_line() {
    let out = render_status(&unprobed_status(0, HashMap::new()));
    for provider in Provider::all() {
        let line = out
            .lines()
            .find(|l| l.contains(&format!("`{}`", provider.name())))
            .unwrap_or_else(|| panic!("no status line for {}", provider.name()));
        assert!(
            line.contains("unknown:")
                || line.contains("unavailable:")
                || line.contains("available:"),
            "line for {} must state a labelled state: {line}",
            provider.name()
        );
        // "state: reason" — there must actually be a reason after the colon.
        let reason = line.rsplit(": ").next().unwrap_or_default();
        assert!(
            reason.len() > 10,
            "line for {} states a verdict with no provenance: {line}",
            provider.name()
        );
    }
}

#[test]
fn render_status_uses_a_check_mark_only_for_evidenced_providers() {
    let mut evidence = HashMap::new();
    evidence.insert(
        Provider::Claude,
        ProviderEvidence {
            successful_requests: 1,
            last_success_age: Some(Duration::from_secs(4)),
            ..Default::default()
        },
    );
    let inventory = availability::inventory(&evidence, &host_with_browser());
    let available: Vec<_> = inventory
        .iter()
        .filter(|e| e.availability.is_available())
        .map(|e| e.provider)
        .collect();
    assert_eq!(available, vec![Provider::Claude]);

    let status = OrchestratorStatus {
        available_providers: available,
        inventory,
        browser_runtime: host_with_browser(),
        active_workflows: 0,
        provider_stats: HashMap::new(),
    };
    let out = render_status(&status);
    assert_eq!(
        out.matches('\u{2705}').count(),
        1,
        "exactly the one evidenced provider gets a check mark:\n{out}"
    );
    assert!(out.contains("Verified available: 1 of"));
}

#[test]
fn render_status_reports_a_browserless_host_as_unavailable() {
    let inventory = availability::inventory(&HashMap::new(), &BrowserRuntime::none());
    let status = OrchestratorStatus {
        available_providers: Vec::new(),
        inventory,
        browser_runtime: BrowserRuntime::none(),
        active_workflows: 0,
        provider_stats: HashMap::new(),
    };
    let out = render_status(&status);
    assert!(!out.contains('\u{2705}'));
    assert!(out.contains("no CDP-capable browser found on this host"));
    for provider in Provider::all() {
        assert!(
            out.contains(&format!("`{}`", provider.name())),
            "{} must still be listed, just not as available",
            provider.name()
        );
    }
    // Sanity: the classifier really did rule them out, not merely stay quiet.
    assert!(matches!(
        availability::classify(
            crate::availability::Modality::Browser,
            &ProviderEvidence::default(),
            &BrowserRuntime::none()
        ),
        Availability::Unavailable { .. }
    ));
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
