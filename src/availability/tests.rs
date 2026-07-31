//! Unit tests for measured availability.
//!
//! Every test here is about *provenance*: which claims the classifier is allowed
//! to make, and which it must refuse to make. No browser is launched and no
//! network call is made.

use super::*;
use std::collections::HashMap;

fn browser_present() -> BrowserRuntime {
    BrowserRuntime {
        cdp_browsers: vec!["chromium 120".to_string()],
    }
}

// ---- the core honesty property -------------------------------------------------

#[test]
fn unexercised_provider_with_a_browser_is_unknown_not_available() {
    // The whole point of the fix: a browser being installed says the *transport*
    // exists. It says nothing about whether we are logged in to the provider.
    let got = classify(
        Modality::Browser,
        &ProviderEvidence::default(),
        &browser_present(),
    );
    assert!(
        matches!(got, Availability::Unknown { .. }),
        "an unprobed provider must be Unknown, got {got:?}"
    );
    assert!(!got.is_available(), "Unknown must not count as available");
    assert!(
        got.detail().contains("unverified"),
        "the reason must say what is unverified, got {:?}",
        got.detail()
    );
}

#[test]
fn no_browser_on_host_makes_every_provider_unavailable() {
    let got = classify(
        Modality::Browser,
        &ProviderEvidence::default(),
        &BrowserRuntime::none(),
    );
    assert!(
        matches!(got, Availability::Unavailable { .. }),
        "no browser must rule the provider out, got {got:?}"
    );
    assert!(got.detail().contains("CDP-capable browser"));
}

#[test]
fn only_a_recent_success_earns_available() {
    let evidence = ProviderEvidence {
        successful_requests: 2,
        failed_requests: 0,
        consecutive_failures: 0,
        last_success_age: Some(Duration::from_secs(30)),
        last_failure_age: None,
    };
    let got = classify(Modality::Browser, &evidence, &browser_present());
    assert!(
        got.is_available(),
        "a recent success is evidence, got {got:?}"
    );
    assert!(got.detail().contains("30s ago"));
}

#[test]
fn stale_success_expires_back_to_unknown() {
    let evidence = ProviderEvidence {
        successful_requests: 1,
        last_success_age: Some(EVIDENCE_TTL + Duration::from_secs(1)),
        ..Default::default()
    };
    let got = classify(Modality::Browser, &evidence, &browser_present());
    assert!(
        matches!(got, Availability::Unknown { .. }),
        "expired evidence must not keep the green check, got {got:?}"
    );
}

#[test]
fn repeated_recent_failures_are_unavailable() {
    let evidence = ProviderEvidence {
        successful_requests: 0,
        failed_requests: 4,
        consecutive_failures: 4,
        last_success_age: None,
        last_failure_age: Some(Duration::from_secs(5)),
    };
    let got = classify(Modality::Browser, &evidence, &browser_present());
    assert!(
        matches!(got, Availability::Unavailable { .. }),
        "4 consecutive failures must be Unavailable, got {got:?}"
    );
    assert!(got.detail().contains('4'));
}

#[test]
fn failures_beat_a_stale_success_but_not_a_fresh_one() {
    // Fresh success wins over an old failure streak.
    let evidence = ProviderEvidence {
        successful_requests: 1,
        failed_requests: 3,
        consecutive_failures: 3,
        last_success_age: Some(Duration::from_secs(2)),
        last_failure_age: Some(Duration::from_secs(600)), // outside the 300s window
    };
    assert!(classify(Modality::Browser, &evidence, &browser_present()).is_available());
}

#[test]
fn api_modality_is_reported_unimplemented_not_available() {
    let got = classify(
        Modality::Api,
        &ProviderEvidence::default(),
        &browser_present(),
    );
    assert!(!got.is_available());
    assert!(got.detail().contains("Wave B"));
}

// ---- inventory ------------------------------------------------------------------

#[test]
fn inventory_covers_every_compiled_provider_and_never_drifts() {
    // Regression guard for the observed drift: `agent_status` listed 7 providers
    // while the static `agent_list_providers` catalogue listed 6 (kaggle missing).
    let entries = inventory(&HashMap::new(), &BrowserRuntime::none());
    let all = Provider::all();
    assert_eq!(
        entries.len(),
        all.len(),
        "inventory must have exactly one row per compiled provider"
    );
    for provider in all {
        let entry = entries
            .iter()
            .find(|e| e.provider == provider)
            .unwrap_or_else(|| panic!("inventory missing {provider}"));
        assert_eq!(entry.id, provider.name());
        assert_eq!(entry.endpoint, provider.base_url());
    }
}

#[test]
fn inventory_declares_modality_for_every_entry() {
    // ROADMAP B4: list_providers reports modality `api` | `browser`.
    let entries = inventory(&HashMap::new(), &BrowserRuntime::none());
    assert!(!entries.is_empty());
    for entry in &entries {
        assert_eq!(
            entry.modality.as_str(),
            "browser",
            "no API backend exists yet, so every provider must declare browser modality"
        );
    }
}

#[test]
fn inventory_marks_nothing_available_without_evidence() {
    let runtime = browser_present();
    let entries = inventory(&HashMap::new(), &runtime);
    assert!(
        entries.iter().all(|e| !e.availability.is_available()),
        "no provider may be reported available before a single request has been made"
    );
}

// ---- the host probe itself ------------------------------------------------------

#[test]
fn browser_probe_runs_offline_and_reports_what_it_found() {
    // We cannot assert *which* browsers exist on an arbitrary CI host, but we can
    // assert the probe completes without launching anything and that its summary
    // agrees with its own data.
    let runtime = BrowserRuntime::probe();
    if runtime.has_browser() {
        assert!(runtime.summary().contains("CDP-capable browsers found"));
    } else {
        assert!(runtime.summary().contains("no CDP-capable browser"));
    }
}
