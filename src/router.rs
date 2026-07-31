//! Provider router for intelligent prompt distribution.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use embeddenator_webpuppet::Provider;
use serde::{Deserialize, Serialize};

use crate::availability::ProviderEvidence;
use crate::error::{Error, Result};

/// Router for distributing prompts across providers.
pub struct ProviderRouter {
    /// Provider preferences and priorities.
    preferences: ProviderPreferences,
    /// Provider health status.
    health: HashMap<Provider, ProviderHealth>,
    /// Usage statistics.
    stats: HashMap<Provider, ProviderStats>,
}

impl ProviderRouter {
    /// Create a new router with default preferences.
    pub fn new() -> Self {
        Self {
            preferences: ProviderPreferences::default(),
            health: HashMap::new(),
            stats: HashMap::new(),
        }
    }

    /// Create a router with custom preferences.
    pub fn with_preferences(preferences: ProviderPreferences) -> Self {
        Self {
            preferences,
            health: HashMap::new(),
            stats: HashMap::new(),
        }
    }

    /// Select the best provider for a task.
    pub fn select_best(&self, task_type: TaskType) -> Result<Provider> {
        let available = self.candidate_providers();

        if available.is_empty() {
            return Err(Error::NoProviders("no healthy providers available".into()));
        }

        // Score each provider
        let mut best: Option<(Provider, f64)> = None;

        for provider in available {
            let score = self.score_provider(provider, &task_type);
            if best.is_none_or(|(_, s)| score > s) {
                best = Some((provider, score));
            }
        }

        best.map(|(p, _)| p)
            .ok_or_else(|| Error::NoProviders("no suitable provider found".into()))
    }

    /// Select multiple providers for parallel/consensus tasks.
    pub fn select_multiple(&self, count: usize, task_type: TaskType) -> Result<Vec<Provider>> {
        let available = self.candidate_providers();

        if available.len() < count {
            return Err(Error::NoProviders(format!(
                "need {} providers but only {} available",
                count,
                available.len()
            )));
        }

        // Score and sort providers
        let mut scored: Vec<_> = available
            .into_iter()
            .map(|p| (p, self.score_provider(p, &task_type)))
            .collect();

        scored.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored.into_iter().take(count).map(|(p, _)| p).collect())
    }

    /// Providers this router is willing to *try* for a request.
    ///
    /// This is a routing filter, **not** an availability claim. A provider is a
    /// candidate unless we have recorded enough failures to rule it out, so a
    /// never-exercised provider is a candidate by construction. Reporting that
    /// set to a user as "available providers" is what made `agent_status` lie;
    /// for a claim about availability use [`crate::availability`] instead, which
    /// probes the host and requires positive evidence.
    ///
    /// Renamed from `available_providers` for exactly that reason.
    pub fn candidate_providers(&self) -> Vec<Provider> {
        Provider::all()
            .into_iter()
            .filter(|p| self.is_healthy(*p))
            .collect()
    }

    /// Check if a provider is healthy.
    ///
    /// "Healthy" means *not known-bad*: a provider with no recorded history is
    /// healthy. This is deliberately optimistic because it gates routing, not
    /// reporting.
    pub fn is_healthy(&self, provider: Provider) -> bool {
        self.health.get(&provider).is_none_or(|h| h.is_healthy())
    }

    /// Everything this process has actually observed, per provider.
    ///
    /// Providers with no recorded history are absent from the map: callers must
    /// distinguish "no evidence" from "evidence of nothing".
    pub fn evidence(&self) -> HashMap<Provider, ProviderEvidence> {
        let mut out: HashMap<Provider, ProviderEvidence> = HashMap::new();

        for (provider, stats) in &self.stats {
            out.insert(
                *provider,
                ProviderEvidence {
                    successful_requests: stats.successful_requests,
                    failed_requests: stats.failed_requests,
                    ..Default::default()
                },
            );
        }
        for (provider, health) in &self.health {
            let entry = out.entry(*provider).or_default();
            entry.consecutive_failures = health.consecutive_failures;
            entry.last_success_age = health.last_success.map(|t| t.elapsed());
            entry.last_failure_age = health.last_failure.map(|t| t.elapsed());
        }

        out
    }

    /// Score a provider for a given task type.
    fn score_provider(&self, provider: Provider, task_type: &TaskType) -> f64 {
        let mut score = 0.0;

        // Base priority from preferences
        score += self.preferences.priority(provider) as f64;

        // Task-specific scoring
        match task_type {
            TaskType::Search => {
                if Provider::search_providers().contains(&provider) {
                    score += 50.0; // Bonus for search-capable providers
                }
            }
            TaskType::LargeContext => {
                if Provider::large_context_providers().contains(&provider) {
                    score += 30.0;
                }
            }
            TaskType::Code => {
                // Claude and ChatGPT are generally better at code
                if matches!(provider, Provider::Claude | Provider::ChatGpt) {
                    score += 20.0;
                }
            }
            TaskType::Creative => {
                // Gemini and Claude for creative tasks
                if matches!(provider, Provider::Gemini | Provider::Claude) {
                    score += 15.0;
                }
            }
            TaskType::General => {
                // No specific bonus
            }
        }

        // Health penalty
        if let Some(health) = self.health.get(&provider) {
            if health.consecutive_failures > 0 {
                score -= (health.consecutive_failures * 10) as f64;
            }
            if let Some(latency) = health.avg_latency {
                // Penalize slow providers
                score -= (latency.as_millis() / 1000) as f64;
            }
        }

        // Usage balancing (prefer less-used providers to distribute load)
        if let Some(stats) = self.stats.get(&provider) {
            let usage_penalty = (stats.total_requests % 100) as f64 * 0.1;
            score -= usage_penalty;
        }

        score
    }

    /// Record a successful request.
    pub fn record_success(&mut self, provider: Provider, latency: Duration) {
        let health = self.health.entry(provider).or_default();
        health.record_success(latency);

        let stats = self.stats.entry(provider).or_default();
        stats.total_requests += 1;
        stats.successful_requests += 1;
    }

    /// Record a failed request.
    pub fn record_failure(&mut self, provider: Provider) {
        let health = self.health.entry(provider).or_default();
        health.record_failure();

        let stats = self.stats.entry(provider).or_default();
        stats.total_requests += 1;
        stats.failed_requests += 1;
    }

    /// Get provider statistics.
    pub fn get_stats(&self) -> HashMap<Provider, ProviderStats> {
        self.stats.clone()
    }
}

impl Default for ProviderRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider preferences and priorities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Priority order (higher = more preferred).
    priorities: HashMap<String, u32>,
    /// Disabled providers.
    disabled: Vec<String>,
    /// Provider-specific settings.
    settings: HashMap<String, serde_json::Value>,
}

impl ProviderPreferences {
    /// Get priority for a provider.
    pub fn priority(&self, provider: Provider) -> u32 {
        self.priorities
            .get(&provider.to_string().to_lowercase())
            .copied()
            .unwrap_or(50) // Default priority
    }

    /// Check if a provider is disabled.
    pub fn is_disabled(&self, provider: Provider) -> bool {
        self.disabled
            .iter()
            .any(|p| p.to_lowercase() == provider.to_string().to_lowercase())
    }
}

impl Default for ProviderPreferences {
    fn default() -> Self {
        let mut priorities = HashMap::new();
        // Default priority order
        priorities.insert("claude".into(), 100);
        priorities.insert("chatgpt".into(), 90);
        priorities.insert("gemini".into(), 80);
        priorities.insert("grok".into(), 70);
        priorities.insert("perplexity".into(), 60);
        priorities.insert("notebooklm".into(), 50);

        Self {
            priorities,
            disabled: Vec::new(),
            settings: HashMap::new(),
        }
    }
}

/// Health status of a provider.
#[derive(Debug, Clone, Default)]
pub struct ProviderHealth {
    /// Last successful request time.
    pub last_success: Option<Instant>,
    /// Last failure time.
    pub last_failure: Option<Instant>,
    /// Consecutive failures.
    pub consecutive_failures: u32,
    /// Average latency.
    pub avg_latency: Option<Duration>,
}

impl ProviderHealth {
    /// Check if provider is considered healthy.
    pub fn is_healthy(&self) -> bool {
        // Unhealthy if 3+ consecutive failures in last 5 minutes
        if self.consecutive_failures >= 3 {
            if let Some(last_fail) = self.last_failure {
                if last_fail.elapsed() < Duration::from_secs(300) {
                    return false;
                }
            }
        }
        true
    }

    /// Record a successful request.
    pub fn record_success(&mut self, latency: Duration) {
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;

        // Update average latency with exponential moving average
        self.avg_latency = Some(match self.avg_latency {
            Some(avg) => Duration::from_millis(
                (avg.as_millis() as f64 * 0.9 + latency.as_millis() as f64 * 0.1) as u64,
            ),
            None => latency,
        });
    }

    /// Record a failed request.
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        self.consecutive_failures += 1;
    }
}

/// Usage statistics for a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    /// Total requests made.
    pub total_requests: u64,
    /// Successful requests.
    pub successful_requests: u64,
    /// Failed requests.
    pub failed_requests: u64,
    /// Total tokens used (if tracked).
    pub total_tokens: Option<u64>,
}

/// Type of task for routing decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    /// General purpose query.
    General,
    /// Search/research task (needs web access).
    Search,
    /// Large context task (needs big context window).
    LargeContext,
    /// Code generation/analysis.
    Code,
    /// Creative writing.
    Creative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_select_best() {
        let router = ProviderRouter::new();

        // Should work for general tasks
        let result = router.select_best(TaskType::General);
        assert!(result.is_ok());
    }

    #[test]
    fn test_router_search_preference() {
        let router = ProviderRouter::new();

        let selected = router.select_best(TaskType::Search).unwrap();
        // Should prefer search-capable providers
        assert!(Provider::search_providers().contains(&selected));
    }

    #[test]
    fn fresh_router_has_no_evidence_for_anyone() {
        // A brand-new router has observed nothing. It must say so, rather than
        // handing out defaults that read like observations.
        let router = ProviderRouter::new();
        assert!(
            router.evidence().is_empty(),
            "a router that has made no requests must report no evidence"
        );
        // ...even though every provider is still a *routing candidate*.
        assert!(!router.candidate_providers().is_empty());
    }

    #[test]
    fn evidence_reflects_recorded_outcomes() {
        let mut router = ProviderRouter::new();
        router.record_success(Provider::Claude, Duration::from_millis(120));
        router.record_failure(Provider::Grok);
        router.record_failure(Provider::Grok);

        let evidence = router.evidence();

        let claude = evidence.get(&Provider::Claude).expect("claude evidence");
        assert_eq!(claude.successful_requests, 1);
        assert_eq!(claude.failed_requests, 0);
        assert_eq!(claude.consecutive_failures, 0);
        assert!(claude.last_success_age.is_some());

        let grok = evidence.get(&Provider::Grok).expect("grok evidence");
        assert_eq!(grok.failed_requests, 2);
        assert_eq!(grok.consecutive_failures, 2);
        assert!(grok.last_success_age.is_none());

        // Untouched providers stay absent — no evidence is not zero evidence.
        assert!(!evidence.contains_key(&Provider::Gemini));
    }
}
