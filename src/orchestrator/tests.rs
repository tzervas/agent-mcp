//! Orchestrator tests: deadlines, true concurrency, and honest status.
//!
//! These use synthetic operations rather than real providers, so they need no
//! browser and no network — but they exercise the *same* [`fan_out`] /
//! [`with_deadline`] code path that `parallel_prompt` and `prompt_provider` use,
//! which is the thing that was previously missing entirely.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A deadline generous enough that nothing in these tests hits it by accident.
const GENEROUS: Duration = Duration::from_secs(30);

// ---- agent_status must not claim availability it has not checked ---------------

#[tokio::test]
async fn fresh_orchestrator_reports_no_verified_available_providers() {
    // Before the fix this returned every compiled-in provider (seven of them,
    // marked ✅) on any host, including hosts with no browser installed at all.
    // Nothing has been probed, so nothing may be claimed.
    let orchestrator = AgentOrchestrator::new();
    let status = orchestrator.status().await;

    assert!(
        status.available_providers.is_empty(),
        "a server that has made no request cannot have verified any provider, got {:?}",
        status.available_providers
    );
    assert!(
        !status.inventory.is_empty(),
        "the inventory itself must still list the providers that exist"
    );
    for entry in &status.inventory {
        assert!(
            !entry.availability.is_available(),
            "{} was reported available without evidence",
            entry.id
        );
        assert!(
            entry.availability.detail().len() > 10,
            "{} was given a verdict with no stated reason",
            entry.id
        );
    }
}

#[tokio::test]
async fn status_inventory_covers_exactly_the_compiled_providers() {
    let status = AgentOrchestrator::new().status().await;
    assert_eq!(status.inventory.len(), Provider::all().len());
    for provider in Provider::all() {
        assert!(status.inventory.iter().any(|e| e.provider == provider));
    }
}

// ---- the configured timeout is actually applied ---------------------------------

#[tokio::test]
async fn with_deadline_maps_an_overrun_to_an_explicit_timeout() {
    let started = Instant::now();
    let outcome: Result<()> = with_deadline(Duration::from_millis(50), "slow thing", async {
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok(())
    })
    .await;

    let err = outcome.expect_err("overrunning the deadline must be an error");
    assert!(
        matches!(err, Error::Timeout(_)),
        "must be a Timeout, got {err:?}"
    );
    assert!(
        err.to_string().contains("slow thing") && err.to_string().contains("50ms"),
        "the error must name the operation and the budget it blew: {err}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the deadline must actually cut the work short"
    );
}

#[tokio::test]
async fn with_deadline_passes_work_that_finishes_in_time() {
    let out: Result<u32> = with_deadline(GENEROUS, "quick thing", async { Ok(7) }).await;
    assert_eq!(out.unwrap(), 7);
}

#[tokio::test]
async fn fan_out_applies_the_deadline_to_every_provider() {
    let providers = vec![Provider::Claude, Provider::Grok, Provider::Gemini];
    let started = Instant::now();

    let results: Vec<(Provider, Result<()>)> =
        fan_out(providers.clone(), Duration::from_millis(50), 5, |_| async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(())
        })
        .await;

    assert_eq!(results.len(), providers.len());
    for (provider, result) in &results {
        let err = result.as_ref().expect_err("must have timed out");
        assert!(
            matches!(err, Error::Timeout(_)),
            "{provider} should report Timeout, got {err:?}"
        );
        assert!(
            err.to_string().contains(provider.name()),
            "the timeout must name the provider it applies to: {err}"
        );
    }
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "a 50ms deadline must not take 30s to report, took {:?}",
        started.elapsed()
    );
}

// ---- C1: true parallel, not sequential ------------------------------------------

#[tokio::test]
async fn fan_out_runs_providers_concurrently_not_sequentially() {
    // The old parallel_prompt looped over providers and awaited each in turn, so
    // four 200ms providers took ~800ms. Run concurrently they take ~200ms.
    let providers = vec![
        Provider::Claude,
        Provider::Grok,
        Provider::Gemini,
        Provider::ChatGpt,
    ];
    let per_provider = Duration::from_millis(200);
    let started = Instant::now();

    let results: Vec<(Provider, Result<()>)> = fan_out(
        providers.clone(),
        GENEROUS,
        providers.len(),
        move |_| async move {
            tokio::time::sleep(per_provider).await;
            Ok(())
        },
    )
    .await;

    let elapsed = started.elapsed();
    assert!(results.iter().all(|(_, r)| r.is_ok()));
    assert!(
        elapsed < per_provider * 2,
        "4 x {per_provider:?} providers ran in {elapsed:?}; sequential execution \
         would take ~{:?}, so this is not actually parallel",
        per_provider * providers.len() as u32
    );
}

#[tokio::test]
async fn fan_out_honours_max_concurrent() {
    // max_concurrent = 1 forces serialisation; the wall clock proves the semaphore
    // is real rather than decorative.
    let providers = vec![Provider::Claude, Provider::Grok, Provider::Gemini];
    let per_provider = Duration::from_millis(100);

    let started = Instant::now();
    let _serial: Vec<(Provider, Result<()>)> =
        fan_out(providers.clone(), GENEROUS, 1, move |_| async move {
            tokio::time::sleep(per_provider).await;
            Ok(())
        })
        .await;
    let serial_elapsed = started.elapsed();

    assert!(
        serial_elapsed >= per_provider * providers.len() as u32,
        "max_concurrent = 1 must serialise; 3 x {per_provider:?} finished in {serial_elapsed:?}"
    );
}

#[tokio::test]
async fn fan_out_caps_simultaneous_work_at_max_concurrent() {
    let providers = vec![
        Provider::Claude,
        Provider::Grok,
        Provider::Gemini,
        Provider::ChatGpt,
        Provider::Perplexity,
    ];
    let in_flight = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let (tracker, peak_tracker) = (Arc::clone(&in_flight), Arc::clone(&peak));
    let _results: Vec<(Provider, Result<()>)> = fan_out(providers, GENEROUS, 2, move |_| {
        let tracker = Arc::clone(&tracker);
        let peak_tracker = Arc::clone(&peak_tracker);
        async move {
            let now = tracker.fetch_add(1, Ordering::SeqCst) + 1;
            peak_tracker.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(50)).await;
            tracker.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        }
    })
    .await;

    let observed_peak = peak.load(Ordering::SeqCst);
    assert!(
        observed_peak <= 2,
        "max_concurrent = 2 but {observed_peak} providers ran at once"
    );
    assert!(
        observed_peak > 1,
        "max_concurrent = 2 should still overlap work; peak was {observed_peak}"
    );
}

// ---- result envelope ------------------------------------------------------------

#[tokio::test]
async fn fan_out_returns_results_in_input_order_regardless_of_completion_order() {
    // Finishing order is deliberately the reverse of the input order.
    let providers = vec![Provider::Claude, Provider::Grok, Provider::Gemini];
    let delays: HashMap<Provider, u64> = [
        (Provider::Claude, 150),
        (Provider::Grok, 80),
        (Provider::Gemini, 10),
    ]
    .into_iter()
    .collect();

    let results: Vec<(Provider, Result<Provider>)> =
        fan_out(providers.clone(), GENEROUS, 5, move |provider| {
            let delay = delays[&provider];
            async move {
                tokio::time::sleep(Duration::from_millis(delay)).await;
                Ok(provider)
            }
        })
        .await;

    let order: Vec<Provider> = results.iter().map(|(p, _)| *p).collect();
    assert_eq!(order, providers, "results must follow the requested order");
    for (provider, result) in results {
        assert_eq!(result.unwrap(), provider, "results must not be mismatched");
    }
}

#[tokio::test]
async fn fan_out_reports_a_failing_provider_without_dropping_the_others() {
    let providers = vec![Provider::Claude, Provider::Grok];
    let results: Vec<(Provider, Result<u8>)> =
        fan_out(providers.clone(), GENEROUS, 5, |provider| async move {
            if provider == Provider::Grok {
                Err(Error::Internal("provider exploded".into()))
            } else {
                Ok(1)
            }
        })
        .await;

    assert_eq!(results.len(), 2, "a failure must not shrink the result set");
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
}

#[tokio::test]
async fn parallel_prompt_rejects_an_empty_provider_list() {
    // Never-silent: asking for nothing is a parameter error, not an empty success.
    let err = AgentOrchestrator::new()
        .parallel_prompt("hello", Vec::new())
        .await
        .expect_err("empty provider list must be rejected");
    assert!(matches!(err, Error::InvalidParams(_)), "got {err:?}");
}
