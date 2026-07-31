//! Provider inventory and **measured** availability.
//!
//! # Why this module exists
//!
//! Before this module, `agent_status` reported every compiled-in provider as
//! `✅ available` and `agent_list_providers` returned a hand-maintained static
//! string. Neither had looked at anything. On a host with no browser installed
//! — the common case for a container or a headless server — the server still
//! claimed seven working providers, and the two tools disagreed with each other
//! about how many providers even existed.
//!
//! A tool that confidently reports state it never checked is worse than one that
//! reports nothing, so this module makes the *provenance* of every claim explicit:
//!
//! * [`BrowserRuntime::probe`] is a real, cheap, offline probe of the host
//!   (filesystem lookup of CDP-capable browser installations). No browser is
//!   launched and no network call is made.
//! * [`ProviderEvidence`] is this process's own observed request history, fed in
//!   from [`crate::router::ProviderRouter`].
//! * [`classify`] combines the two into a **tri-state** [`Availability`]. Anything
//!   that has not actually been established is [`Availability::Unknown`], with the
//!   reason it is unknown — never a green check.
//!
//! # What is deliberately *not* claimed
//!
//! Provider login/session state cannot be established without performing a real
//! request. A detected browser therefore yields `Unknown`, not `Available`:
//! the transport exists, the session is unverified.

use std::time::Duration;

use embeddenator_webpuppet::{BrowserDetector, Provider};

/// How long a recorded success is treated as evidence that a provider still works.
///
/// Past this age the provider drops back to [`Availability::Unknown`] rather than
/// coasting on a stale green check.
pub const EVIDENCE_TTL: Duration = Duration::from_secs(15 * 60);

/// How a provider is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    /// Driven through a real browser session (webpuppet / CDP).
    Browser,
    /// Driven through an HTTP API.
    ///
    /// **Not yet reachable.** No provider reports this modality today: the API
    /// provider path is `docs/ROADMAP.md` Wave B (B1–B3) and is unimplemented.
    /// The variant exists so the inventory can distinguish the two the moment a
    /// backend lands, rather than silently mislabelling one as the other.
    Api,
}

impl Modality {
    /// Stable machine-readable name (`browser` / `api`), per ROADMAP B4.
    pub fn as_str(self) -> &'static str {
        match self {
            Modality::Browser => "browser",
            Modality::Api => "api",
        }
    }
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The answer to "can this provider serve a request right now?", with provenance.
///
/// Tri-state on purpose. `Unknown` is a first-class outcome, not an error and not
/// a soft yes: it is what an honest answer looks like when nothing has been
/// established either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    /// Positive evidence exists: a request to this provider recently succeeded.
    Available {
        /// What that evidence is.
        evidence: String,
    },
    /// Negative evidence exists: something checked has ruled the provider out.
    Unavailable {
        /// What ruled it out.
        reason: String,
    },
    /// Nothing has established the answer either way.
    Unknown {
        /// Why it is unknown, and what would settle it.
        reason: String,
    },
}

impl Availability {
    /// One-word machine-readable state.
    pub fn label(&self) -> &'static str {
        match self {
            Availability::Available { .. } => "available",
            Availability::Unavailable { .. } => "unavailable",
            Availability::Unknown { .. } => "unknown",
        }
    }

    /// Display marker. Only a genuinely verified provider gets a check mark.
    pub fn marker(&self) -> &'static str {
        match self {
            Availability::Available { .. } => "\u{2705}",   // ✅
            Availability::Unavailable { .. } => "\u{274c}", // ❌
            Availability::Unknown { .. } => "\u{2753}",     // ❓
        }
    }

    /// The provenance string (evidence or reason).
    pub fn detail(&self) -> &str {
        match self {
            Availability::Available { evidence } => evidence,
            Availability::Unavailable { reason } | Availability::Unknown { reason } => reason,
        }
    }

    /// True only when availability was positively established.
    ///
    /// `Unknown` is **not** available. Callers that need a provider must either
    /// try it and handle failure, or surface the unknown to the user.
    pub fn is_available(&self) -> bool {
        matches!(self, Availability::Available { .. })
    }
}

/// This process's own observed history for one provider, as recorded by the router.
///
/// Every field is something that actually happened; nothing here is a default or
/// an assumption.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderEvidence {
    /// Requests that returned a response.
    pub successful_requests: u64,
    /// Requests that returned an error.
    pub failed_requests: u64,
    /// Failures since the last success.
    pub consecutive_failures: u32,
    /// Age of the most recent success, if there has ever been one.
    pub last_success_age: Option<Duration>,
    /// Age of the most recent failure, if there has ever been one.
    pub last_failure_age: Option<Duration>,
}

impl ProviderEvidence {
    /// True when this provider has never been exercised by this process.
    pub fn is_empty(&self) -> bool {
        self.successful_requests == 0 && self.failed_requests == 0
    }
}

/// Result of probing the host for a browser runtime.
///
/// This is a filesystem probe of installed, CDP-capable browsers. It launches
/// nothing and talks to no network.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserRuntime {
    /// Names of the CDP-capable browsers found on this host, in detection order.
    pub cdp_browsers: Vec<String>,
}

impl BrowserRuntime {
    /// Probe the host. Cheap and synchronous.
    pub fn probe() -> Self {
        let cdp_browsers = BrowserDetector::detect_cdp_capable()
            .into_iter()
            .filter(|install| install.is_valid())
            .map(|install| match install.version {
                Some(v) => format!("{} {}", install.browser_type.name(), v),
                None => install.browser_type.name().to_string(),
            })
            .collect();
        Self { cdp_browsers }
    }

    /// A runtime with no browsers, for tests and for hosts where probing is skipped.
    pub fn none() -> Self {
        Self {
            cdp_browsers: Vec::new(),
        }
    }

    /// True when at least one CDP-capable browser was found.
    pub fn has_browser(&self) -> bool {
        !self.cdp_browsers.is_empty()
    }

    /// Human-readable summary of what the probe found.
    pub fn summary(&self) -> String {
        if self.cdp_browsers.is_empty() {
            "no CDP-capable browser found on this host".to_string()
        } else {
            format!(
                "CDP-capable browsers found: {}",
                self.cdp_browsers.join(", ")
            )
        }
    }
}

/// One row of the provider inventory.
#[derive(Debug, Clone)]
pub struct ProviderEntry {
    /// The provider itself.
    pub provider: Provider,
    /// Stable id accepted by the `provider` tool argument.
    pub id: &'static str,
    /// Endpoint this provider is driven against.
    pub endpoint: &'static str,
    /// How it is reached.
    pub modality: Modality,
    /// Measured availability, with provenance.
    pub availability: Availability,
}

/// Decide a provider's availability from what is actually known.
///
/// Evaluation order, most-specific evidence first:
///
/// 1. recent repeated failures → `Unavailable` (measured);
/// 2. a recent success → `Available` (measured);
/// 3. no browser runtime on the host → `Unavailable` (measured — a browser-modality
///    provider cannot run without one);
/// 4. a stale success → `Unknown` (the evidence expired);
/// 5. otherwise → `Unknown` (the transport exists, the session is unverified).
///
/// There is no branch that returns `Available` without evidence.
pub fn classify(
    modality: Modality,
    evidence: &ProviderEvidence,
    runtime: &BrowserRuntime,
) -> Availability {
    if modality == Modality::Api {
        return Availability::Unavailable {
            reason: "API provider backends are not implemented (ROADMAP Wave B, B1-B3)".to_string(),
        };
    }

    // 1. Repeated recent failures are hard negative evidence.
    if evidence.consecutive_failures >= 3 {
        if let Some(age) = evidence.last_failure_age {
            if age < Duration::from_secs(300) {
                return Availability::Unavailable {
                    reason: format!(
                        "{} consecutive failures, most recent {}s ago",
                        evidence.consecutive_failures,
                        age.as_secs()
                    ),
                };
            }
        }
    }

    // 2. A recent success is the only thing that earns a green check.
    if let Some(age) = evidence.last_success_age {
        if age < EVIDENCE_TTL {
            return Availability::Available {
                evidence: format!(
                    "last request succeeded {}s ago ({} ok / {} failed this session)",
                    age.as_secs(),
                    evidence.successful_requests,
                    evidence.failed_requests
                ),
            };
        }
    }

    // 3. Browser-modality providers are ruled out by a host with no browser.
    if !runtime.has_browser() {
        return Availability::Unavailable {
            reason: "no CDP-capable browser detected on this host, so no browser-modality \
                     provider can run here"
                .to_string(),
        };
    }

    // 4. Evidence existed but expired.
    if let Some(age) = evidence.last_success_age {
        return Availability::Unknown {
            reason: format!(
                "last success was {}s ago, older than the {}s evidence window; \
                 session may have expired",
                age.as_secs(),
                EVIDENCE_TTL.as_secs()
            ),
        };
    }

    // 5. Transport present, session unverified. This is the honest default.
    Availability::Unknown {
        reason: format!(
            "{}; this provider has not been used by this process, so its login/session \
             state is unverified — only a real request can settle it",
            runtime.summary()
        ),
    }
}

/// Build the full provider inventory from the compiled-in provider set.
///
/// The set comes from [`Provider::all`], not a hand-maintained list, so the
/// inventory cannot drift from what the binary can actually address.
pub fn inventory(
    evidence: &std::collections::HashMap<Provider, ProviderEvidence>,
    runtime: &BrowserRuntime,
) -> Vec<ProviderEntry> {
    let default_evidence = ProviderEvidence::default();
    Provider::all()
        .into_iter()
        .map(|provider| {
            // Every provider compiled into this binary today is browser-driven;
            // the API path is unimplemented (ROADMAP Wave B).
            let modality = Modality::Browser;
            let ev = evidence.get(&provider).unwrap_or(&default_evidence);
            ProviderEntry {
                provider,
                id: provider.name(),
                endpoint: provider.base_url(),
                modality,
                availability: classify(modality, ev, runtime),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
