use crate::config::Config;
use crate::codex_conversation::CodexConversation;
use crate::protocol::Event;
use codex_subagents::SubagentSpec;
use codex_subagents::TaskContext;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::protocol::EventMsg;
use crate::protocol::SubAgentCompletedEvent;
use crate::protocol::SubAgentMessageEvent;
use crate::protocol::SubAgentStartedEvent;

use super::SubagentIntegrationError;
use super::SubagentResult;
use super::registry::CoreSubagentRegistry;

/// Parameters describing a subagent invocation request.
#[derive(Debug, Clone)]
pub struct SubagentRunRequest {
    pub agent_name: String,
    pub prompt: Option<String>,
}

/// Results of subagent execution, including events and any output.
#[derive(Debug)]
pub struct SubagentExecResult {
    pub events: Vec<EventMsg>,
    pub conversation: Arc<CodexConversation>,
    pub task_context: TaskContext,
}

/// Configuration for subagent execution timeouts and retry logic.
#[derive(Debug, Clone)]
pub struct SubagentExecConfig {
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for SubagentExecConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes default
            max_retries: 2,
        }
    }
}

/// Coordinates subagent execution and conversation lifecycle management.
pub struct SubagentOrchestrator {
    registry: Arc<CoreSubagentRegistry>,
}

impl SubagentOrchestrator {
    pub fn new(registry: Arc<CoreSubagentRegistry>) -> Self {
        Self { registry }
    }

    /// Returns `true` when subagents are enabled in the supplied config.
    pub fn is_enabled(config: &Config) -> bool {
        config.subagents.enabled
    }

    /// Attempt to load the requested agent, returning its spec if available.
    async fn load_agent(&self, request: &SubagentRunRequest) -> SubagentResult<Arc<SubagentSpec>> {
        let registry = self.registry.read().await;
        let record = registry
            .get(&request.agent_name)
            .map_err(|_| SubagentIntegrationError::UnknownAgent(request.agent_name.clone()))?;
        Ok(record.spec.clone())
    }

    /// Execute the provided subagent request. The default implementation validates
    /// feature flags and loads the spec, deferring execution to future work.
    pub async fn prepare(
        &self,
        config: &Config,
        request: &SubagentRunRequest,
    ) -> SubagentResult<Arc<SubagentSpec>> {
        if !Self::is_enabled(config) {
            return Err(SubagentIntegrationError::Disabled);
        }

        self.load_agent(request).await
    }

    /// Execute a subagent request using the provided conversation.
    /// This is the core orchestration method that manages the subagent lifecycle.
    pub async fn execute(
        &self,
        conversation: Arc<CodexConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
        config: &SubagentExecConfig,
    ) -> SubagentResult<SubagentExecResult> {
        let sub_conversation_id = conversation.id().to_string();
        let task_context = TaskContext::new();

        // Create initial lifecycle events
        let mut events = Vec::new();
        let model = spec.model().map(|s| s.to_string());

        // Emit SubAgentStarted event
        events.push(EventMsg::SubAgentStarted(SubAgentStartedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: sub_conversation_id.clone(),
            parent_submit_id: None,
            model: model.clone(),
        }));

        // Execute with timeout and retry logic
        let execution_result = self
            .execute_with_retry(conversation.clone(), spec.clone(), request, config)
            .await;

        // Emit SubAgentCompleted event regardless of outcome
        let outcome = match &execution_result {
            Ok(_) => Some("success".to_string()),
            Err(e) => Some(format!("error: {}", e)),
        };

        events.push(EventMsg::SubAgentCompleted(SubAgentCompletedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: sub_conversation_id.clone(),
            outcome,
        }));

        // Return result with events
        match execution_result {
            Ok(_) => Ok(SubagentExecResult {
                events,
                conversation,
                task_context,
            }),
            Err(e) => {
                // Even on failure, return the events to maintain visibility
                Ok(SubagentExecResult {
                    events,
                    conversation,
                    task_context,
                })
            }
        }
    }

    /// Execute with retry logic and timeout handling.
    async fn execute_with_retry(
        &self,
        conversation: Arc<CodexConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
        config: &SubagentExecConfig,
    ) -> SubagentResult<()> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= config.max_retries {
            match timeout(config.timeout, self.execute_once(conversation.clone(), spec.clone(), request)).await {
                Ok(Ok(_)) => return Ok(()),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts <= config.max_retries {
                        // Log retry attempt
                        tracing::warn!(
                            "Subagent {} execution attempt {} failed, retrying...",
                            spec.name(),
                            attempts
                        );
                    }
                }
                Err(_) => {
                    // Timeout
                    return Err(SubagentIntegrationError::Registry(
                        codex_subagents::SubagentError::Parse(format!(
                            "Subagent {} execution timed out after {:?}",
                            spec.name(),
                            config.timeout
                        ))
                    ));
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| {
            SubagentIntegrationError::Registry(
                codex_subagents::SubagentError::Parse(
                    "Subagent execution failed after all retries".to_string()
                )
            )
        }))
    }

    /// Single execution attempt. In the current implementation, this is a placeholder
    /// that simulates execution. Future implementations will integrate with the
    /// actual conversation processing pipeline.
    async fn execute_once(
        &self,
        conversation: Arc<CodexConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
    ) -> SubagentResult<()> {
        // TODO: Implement actual subagent execution logic
        // This would involve:
        // 1. Tool policy validation against spec.tools()
        // 2. Injecting spec.instructions() as system prompt
        // 3. Processing request.prompt if provided
        // 4. Running the conversation with proper isolation
        // 5. Collecting and merging results

        tracing::info!(
            "Executing subagent '{}' with prompt: {:?}",
            spec.name(),
            request.prompt
        );

        // Validate tool policy enforcement
        self.validate_tool_policy(&spec)?;

        // For now, simulate successful execution
        Ok(())
    }

    /// Validate that the subagent's tool allowlist is properly configured.
    /// This is a safety check to ensure tools are explicitly allowed.
    fn validate_tool_policy(&self, spec: &SubagentSpec) -> SubagentResult<()> {
        let allowed_tools = spec.tools();

        if allowed_tools.is_empty() {
            tracing::warn!(
                "Subagent '{}' has no tools specified in allowlist. This may limit functionality.",
                spec.name()
            );
        } else {
            tracing::debug!(
                "Subagent '{}' allowed tools: {:?}",
                spec.name(),
                allowed_tools
            );
        }

        // TODO: Future implementation would:
        // 1. Hook into the tool execution pipeline to intercept all tool calls
        // 2. Check each tool call against spec.tools() allowlist
        // 3. Reject unauthorized tool calls with clear error messages
        // 4. Add agent/model context to approval prompts for manual review

        Ok(())
    }

    pub fn lifecycle_events(
        &self,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
        sub_conversation_id: &str,
    ) -> Vec<EventMsg> {
        let mut events = Vec::new();
        let conversation_id = sub_conversation_id.to_string();
        let model = spec.model().map(|s| s.to_string());
        events.push(EventMsg::SubAgentStarted(SubAgentStartedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: conversation_id.clone(),
            parent_submit_id: None,
            model,
        }));

        if let Some(prompt) = request
            .prompt
            .as_ref()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
        {
            events.push(EventMsg::SubAgentMessage(SubAgentMessageEvent {
                agent_name: spec.name().to_string(),
                sub_conversation_id: conversation_id.clone(),
                role: "user".to_string(),
                content: prompt.to_string(),
            }));
        }

        events.push(EventMsg::SubAgentCompleted(SubAgentCompletedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: conversation_id,
            outcome: None,
        }));

        events
    }
}

impl CoreSubagentRegistry {
    /// Helper to construct a new orchestrator.
    pub fn orchestrator(self: Arc<Self>) -> SubagentOrchestrator {
        SubagentOrchestrator::new(self)
    }
}
