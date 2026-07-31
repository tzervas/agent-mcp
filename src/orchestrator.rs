//! Agent orchestrator for multi-provider prompt execution.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;

use embeddenator_webpuppet::{PromptRequest, PromptResponse, Provider, WebPuppet};

use crate::availability::{self, BrowserRuntime, ProviderEntry};
use crate::error::{Error, Result};
use crate::router::{ProviderRouter, TaskType};
use crate::workflow::{
    ProviderResponse, StepConfig, StepResult, StepState, Workflow, WorkflowState,
};

/// Run one fallible async operation under a wall-clock deadline.
///
/// Overrunning the deadline is an explicit [`Error::Timeout`] naming the operation
/// and the budget it blew, never a silent hang and never a degraded success.
///
/// This and [`fan_out`] are the only places a deadline is applied, so
/// [`OrchestratorConfig::timeout`] has exactly one meaning across the crate.
pub async fn with_deadline<T, F>(deadline: Duration, what: &str, fut: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(deadline, fut).await {
        Ok(result) => result,
        Err(_) => Err(Error::Timeout(format!(
            "{what} exceeded the configured deadline of {}ms",
            deadline.as_millis()
        ))),
    }
}

/// Fan one operation out across providers **concurrently**, each under its own
/// deadline, with at most `max_concurrent` in flight (ROADMAP C1).
///
/// Results come back in the order the providers were given, regardless of the
/// order they finish in. A provider that overruns its deadline yields
/// [`Error::Timeout`]; a provider whose task panics or is cancelled yields
/// [`Error::Internal`] rather than vanishing from the results.
///
/// The `max_concurrent` permit is acquired *before* the deadline starts, so a
/// provider queued behind the concurrency limit still gets its full budget.
pub async fn fan_out<T, F, Fut>(
    providers: Vec<Provider>,
    deadline: Duration,
    max_concurrent: usize,
    op: F,
) -> Vec<(Provider, Result<T>)>
where
    T: Send + 'static,
    F: Fn(Provider) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
{
    let count = providers.len();
    let semaphore = Arc::new(Semaphore::new(max_concurrent.max(1)));
    let op = Arc::new(op);

    let mut tasks: JoinSet<(usize, Provider, Result<T>)> = JoinSet::new();
    for (index, provider) in providers.iter().copied().enumerate() {
        let semaphore = Arc::clone(&semaphore);
        let op = Arc::clone(&op);
        tasks.spawn(async move {
            // The semaphore is never closed, so this only fails while the runtime
            // is tearing down; treat that as a loud error, not a skipped provider.
            let _permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    return (
                        index,
                        provider,
                        Err(Error::Internal(format!(
                            "concurrency limiter closed before {provider} could start: {e}"
                        ))),
                    )
                }
            };
            let result = match tokio::time::timeout(deadline, op(provider)).await {
                Ok(result) => result,
                Err(_) => Err(Error::Timeout(format!(
                    "provider {provider} did not respond within the configured {}ms deadline",
                    deadline.as_millis()
                ))),
            };
            (index, provider, result)
        });
    }

    let mut slots: Vec<Option<(Provider, Result<T>)>> = (0..count).map(|_| None).collect();
    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((index, provider, result)) => slots[index] = Some((provider, result)),
            Err(e) => tracing::error!("provider task failed to join: {e}"),
        }
    }

    // Never-silent: a slot we did not fill becomes an explicit error, not a gap.
    slots
        .into_iter()
        .zip(providers)
        .map(|(slot, provider)| {
            slot.unwrap_or_else(|| {
                (
                    provider,
                    Err(Error::Internal(format!(
                        "provider task for {provider} panicked or was cancelled"
                    ))),
                )
            })
        })
        .collect()
}

/// Orchestrator for multi-agent prompt execution.
pub struct AgentOrchestrator {
    /// WebPuppet instance for browser automation.
    puppet: Arc<RwLock<Option<WebPuppet>>>,
    /// Provider router for intelligent distribution.
    router: Arc<RwLock<ProviderRouter>>,
    /// Active workflows.
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    /// Configuration.
    config: OrchestratorConfig,
}

impl AgentOrchestrator {
    /// Create a new orchestrator.
    pub fn new() -> Self {
        Self {
            puppet: Arc::new(RwLock::new(None)),
            router: Arc::new(RwLock::new(ProviderRouter::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            config: OrchestratorConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            puppet: Arc::new(RwLock::new(None)),
            router: Arc::new(RwLock::new(ProviderRouter::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get or create WebPuppet instance.
    async fn get_puppet(&self) -> Result<WebPuppet> {
        let guard = self.puppet.read().await;
        if guard.is_some() {
            drop(guard);
        }

        // Create new puppet
        let puppet = WebPuppet::builder()
            .with_all_providers()
            .headless(self.config.headless)
            .build()
            .await?;

        Ok(puppet)
    }

    /// Send a prompt to the best available provider.
    pub async fn prompt(&self, message: impl Into<String>) -> Result<PromptResponse> {
        let router = self.router.read().await;
        let provider = router.select_best(TaskType::General)?;
        drop(router);

        self.prompt_provider(provider, message).await
    }

    /// Send a prompt to a specific provider.
    pub async fn prompt_provider(
        &self,
        provider: Provider,
        message: impl Into<String>,
    ) -> Result<PromptResponse> {
        let message = message.into();
        let start = Instant::now();
        let deadline = self.config.timeout;

        let puppet = with_deadline(deadline, "browser session setup", self.get_puppet()).await?;

        // Authentication and the prompt itself share one budget: the caller asked
        // for an answer within `timeout`, not for each hop to get its own.
        let result = with_deadline(deadline, &format!("prompt to {provider}"), async {
            puppet.authenticate(provider).await?;
            let response = puppet
                .prompt(provider, PromptRequest::new(&message))
                .await?;
            Ok(response)
        })
        .await;

        // Record result in router
        let mut router = self.router.write().await;
        match &result {
            Ok(_) => router.record_success(provider, start.elapsed()),
            Err(_) => router.record_failure(provider),
        }
        drop(router);

        // Cleanup
        puppet.close().await.ok();

        result
    }

    /// Send a prompt to multiple providers, **truly concurrently** (ROADMAP C1).
    ///
    /// Every provider runs in its own task under its own copy of the configured
    /// deadline, with at most `max_concurrent` in flight. A provider that fails,
    /// times out, or panics contributes an `Err` entry; it never removes itself
    /// from the results and never blocks the others.
    ///
    /// Returns one entry per requested provider, in the requested order.
    pub async fn parallel_prompt(
        &self,
        message: impl Into<String>,
        providers: Vec<Provider>,
    ) -> Result<Vec<(Provider, Result<PromptResponse>)>> {
        if providers.is_empty() {
            return Err(Error::InvalidParams(
                "parallel_prompt needs at least one provider".into(),
            ));
        }

        let message = message.into();
        let deadline = self.config.timeout;

        // One shared browser session, acquired once and under the same deadline.
        // Failing here is a hard failure: no provider could have run.
        let puppet =
            Arc::new(with_deadline(deadline, "browser session setup", self.get_puppet()).await?);
        let session = Arc::clone(&puppet);

        // Each task times its own work, so the router records measured latency
        // rather than a stand-in derived from the deadline.
        let timed = fan_out(
            providers,
            deadline,
            self.config.max_concurrent,
            move |provider| {
                let puppet = Arc::clone(&puppet);
                let message = message.clone();
                async move {
                    let started = Instant::now();
                    puppet.authenticate(provider).await?;
                    let response = puppet
                        .prompt(provider, PromptRequest::new(&message))
                        .await?;
                    Ok((response, started.elapsed()))
                }
            },
        )
        .await;

        // Feed every outcome back into the router so the availability probe has
        // real evidence to report later.
        let mut router = self.router.write().await;
        let mut results = Vec::with_capacity(timed.len());
        for (provider, result) in timed {
            match result {
                Ok((response, latency)) => {
                    router.record_success(provider, latency);
                    results.push((provider, Ok(response)));
                }
                Err(e) => {
                    router.record_failure(provider);
                    results.push((provider, Err(e)));
                }
            }
        }
        drop(router);

        session.close().await.ok();

        Ok(results)
    }

    /// Get consensus from multiple providers.
    pub async fn consensus_prompt(
        &self,
        message: impl Into<String>,
        min_providers: usize,
    ) -> Result<ConsensusResult> {
        let message = message.into();

        // Select providers
        let router = self.router.read().await;
        let providers = router.select_multiple(min_providers.max(3), TaskType::General)?;
        drop(router);

        // Get responses in parallel
        let results = self.parallel_prompt(&message, providers).await?;

        // Collect successful responses
        let responses: Vec<_> = results
            .into_iter()
            .filter_map(|(p, r)| r.ok().map(|resp| (p, resp)))
            .collect();

        if responses.len() < min_providers {
            return Err(Error::NoProviders(format!(
                "only {} providers responded, need {}",
                responses.len(),
                min_providers
            )));
        }

        // Simple consensus: find common themes
        // In a real implementation, this would use semantic similarity
        let consensus = self.find_consensus(&responses);

        Ok(consensus)
    }

    /// Find consensus among responses (simple implementation).
    fn find_consensus(&self, responses: &[(Provider, PromptResponse)]) -> ConsensusResult {
        // For now, just return the longest response as "consensus"
        // A real implementation would use semantic similarity
        let best = responses
            .iter()
            .max_by_key(|(_, r)| r.text.len())
            .map(|(p, r)| (*p, r.clone()));

        let provider_responses: Vec<_> = responses
            .iter()
            .map(|(p, r)| ProviderResponse {
                provider: p.to_string(),
                text: r.text.clone(),
                selected: best.as_ref().is_some_and(|(bp, _)| bp == p),
                confidence: None,
            })
            .collect();

        ConsensusResult {
            consensus_text: best.map(|(_, r)| r.text).unwrap_or_default(),
            responses: provider_responses,
            agreement_score: 0.5, // Placeholder
        }
    }

    /// Start a new workflow.
    pub async fn start_workflow(&self, workflow: Workflow) -> Result<String> {
        let id = workflow.id.clone();
        let mut workflows = self.workflows.write().await;
        workflows.insert(id.clone(), workflow);
        Ok(id)
    }

    /// Execute the next step in a workflow.
    pub async fn execute_workflow_step(&self, workflow_id: &str) -> Result<StepResult> {
        let mut workflows = self.workflows.write().await;
        let workflow = workflows
            .get_mut(workflow_id)
            .ok_or_else(|| Error::Workflow(format!("workflow not found: {}", workflow_id)))?;

        if workflow.is_complete() {
            return Err(Error::InvalidState("workflow already complete".into()));
        }

        // Get step config (clone to avoid borrow issues)
        let step_config = workflow
            .current()
            .ok_or_else(|| Error::InvalidState("no current step".into()))?
            .config
            .clone();

        // Mark step as running
        if let Some(step) = workflow.current_mut() {
            step.start();
        }
        workflow.state = WorkflowState::Running;

        let start = Instant::now();
        let result = match &step_config {
            StepConfig::Prompt {
                message,
                provider,
                context,
            } => {
                let provider = provider
                    .as_ref()
                    .and_then(|p| match p.to_lowercase().as_str() {
                        "claude" => Some(Provider::Claude),
                        "grok" => Some(Provider::Grok),
                        "gemini" => Some(Provider::Gemini),
                        "chatgpt" => Some(Provider::ChatGpt),
                        "perplexity" => Some(Provider::Perplexity),
                        "notebooklm" => Some(Provider::NotebookLm),
                        _ => None,
                    });

                // Note: context is currently not used in prompt_provider
                // Future: pass context as system message
                let _context_for_future = context;

                let response = if let Some(p) = provider {
                    self.prompt_provider(p, message.clone()).await?
                } else {
                    self.prompt(message.clone()).await?
                };

                StepResult {
                    output: response.text,
                    provider: Some(response.provider.to_string()),
                    responses: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    metadata: HashMap::new(),
                }
            }
            StepConfig::ParallelPrompt { message, providers } => {
                let providers: Vec<_> = providers
                    .iter()
                    .filter_map(|p| match p.to_lowercase().as_str() {
                        "claude" => Some(Provider::Claude),
                        "grok" => Some(Provider::Grok),
                        "gemini" => Some(Provider::Gemini),
                        "chatgpt" => Some(Provider::ChatGpt),
                        "perplexity" => Some(Provider::Perplexity),
                        "notebooklm" => Some(Provider::NotebookLm),
                        _ => None,
                    })
                    .collect();

                let results = self.parallel_prompt(message.clone(), providers).await?;

                let responses: Vec<_> = results
                    .iter()
                    .filter_map(|(p, r)| {
                        r.as_ref().ok().map(|resp| ProviderResponse {
                            provider: p.to_string(),
                            text: resp.text.clone(),
                            selected: false,
                            confidence: None,
                        })
                    })
                    .collect();

                let output = responses
                    .iter()
                    .map(|r| format!("**{}**:\n{}", r.provider, r.text))
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");

                StepResult {
                    output,
                    provider: None,
                    responses: Some(responses),
                    duration_ms: start.elapsed().as_millis() as u64,
                    metadata: HashMap::new(),
                }
            }
            StepConfig::Consensus {
                message,
                min_providers,
            } => {
                let consensus = self
                    .consensus_prompt(message.clone(), *min_providers)
                    .await?;

                StepResult {
                    output: consensus.consensus_text,
                    provider: None,
                    responses: Some(consensus.responses),
                    duration_ms: start.elapsed().as_millis() as u64,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert(
                            "agreement_score".into(),
                            serde_json::json!(consensus.agreement_score),
                        );
                        m
                    },
                }
            }
            StepConfig::HumanReview { prompt: _ } => {
                // Set step to waiting and return
                let step = workflow.current_mut().unwrap();
                step.state = StepState::WaitingForHuman;
                workflow.state = WorkflowState::Paused;

                return Err(Error::Workflow("waiting for human review".into()));
            }
            _ => {
                return Err(Error::Workflow("unsupported step type".into()));
            }
        };

        // Mark step complete and advance
        let step = workflow.current_mut().unwrap();
        step.complete(result.clone());
        workflow.advance()?;

        Ok(result)
    }

    /// Get a workflow by ID.
    pub async fn get_workflow(&self, id: &str) -> Option<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.get(id).cloned()
    }

    /// Get orchestrator status, with a **freshly probed** provider inventory.
    ///
    /// The probe is a host-local browser detection plus this process's own request
    /// history; it launches no browser and makes no network call, so it is cheap
    /// enough to run on every status call and always reflects the current host.
    pub async fn status(&self) -> OrchestratorStatus {
        let router = self.router.read().await;
        let workflows = self.workflows.read().await;

        let browser_runtime = BrowserRuntime::probe();
        let inventory = availability::inventory(&router.evidence(), &browser_runtime);
        let available_providers = inventory
            .iter()
            .filter(|entry| entry.availability.is_available())
            .map(|entry| entry.provider)
            .collect();

        OrchestratorStatus {
            available_providers,
            inventory,
            browser_runtime,
            active_workflows: workflows.len(),
            provider_stats: router.get_stats(),
        }
    }

    /// The provider inventory on its own, for `agent_list_providers`.
    pub async fn provider_inventory(&self) -> (Vec<ProviderEntry>, BrowserRuntime) {
        let router = self.router.read().await;
        let browser_runtime = BrowserRuntime::probe();
        let inventory = availability::inventory(&router.evidence(), &browser_runtime);
        (inventory, browser_runtime)
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AgentOrchestrator {
    fn clone(&self) -> Self {
        Self {
            puppet: self.puppet.clone(),
            router: self.router.clone(),
            workflows: self.workflows.clone(),
            config: self.config.clone(),
        }
    }
}

/// Orchestrator configuration.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Run browsers in headless mode.
    pub headless: bool,
    /// Wall-clock budget for a single provider operation.
    ///
    /// Applied by [`with_deadline`] and [`fan_out`]; overrunning it is an explicit
    /// [`Error::Timeout`]. (Before this was wired up the field existed but was
    /// never read — a knob in the config surface that did nothing.)
    pub timeout: Duration,
    /// Maximum provider requests in flight at once during a fan-out.
    ///
    /// Applied by [`fan_out`] via a semaphore.
    pub max_concurrent: usize,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            headless: true,
            timeout: Duration::from_secs(120),
            max_concurrent: 5,
        }
    }
}

/// Result of a consensus operation.
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    /// The consensus text.
    pub consensus_text: String,
    /// All provider responses.
    pub responses: Vec<ProviderResponse>,
    /// Agreement score (0.0 - 1.0).
    pub agreement_score: f64,
}

#[cfg(test)]
mod tests;

/// Orchestrator status.
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    /// Providers whose availability was **positively established**.
    ///
    /// This is a strict subset of [`Self::inventory`]: a provider only appears
    /// here after a real request to it has recently succeeded. On a freshly
    /// started server this is empty, because nothing has been established yet.
    /// Do not read it as "the providers that exist" — that is the inventory.
    pub available_providers: Vec<Provider>,
    /// Every compiled-in provider with its measured availability and provenance.
    pub inventory: Vec<ProviderEntry>,
    /// What the host browser probe found.
    pub browser_runtime: BrowserRuntime,
    /// Number of active workflows.
    pub active_workflows: usize,
    /// Provider statistics.
    pub provider_stats: HashMap<Provider, crate::router::ProviderStats>,
}
