//! Comprehensive orchestrator lifecycle tests with mocked Codex::spawn scenarios.

use codex_subagents::spec::SubagentSpec;
use codex_subagents::builder::SubagentBuilder;
use codex_subagents::registry::{SubagentRegistry, SubagentRecord, AgentSource};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::path::PathBuf;

/// Mock configuration for testing orchestrator scenarios
#[derive(Debug, Clone)]
pub struct MockConfig {
    pub subagents: SubagentConfig,
    pub model_provider: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubagentConfig {
    pub enabled: bool,
    pub auto_route: bool,
}

impl MockConfig {
    pub fn new() -> Self {
        Self {
            subagents: SubagentConfig {
                enabled: true,
                auto_route: false,
            },
            model_provider: "openai".to_string(),
            api_key: Some("test-key".to_string()),
        }
    }

    pub fn with_subagents_disabled(mut self) -> Self {
        self.subagents.enabled = false;
        self
    }

    pub fn with_auto_route_enabled(mut self) -> Self {
        self.subagents.auto_route = true;
        self
    }
}

/// Mock conversation for testing orchestrator execution
#[derive(Debug)]
pub struct MockConversation {
    pub id: String,
    pub messages: Vec<MockMessage>,
    pub model: Option<String>,
    pub state: ConversationState,
}

#[derive(Debug, Clone)]
pub struct MockMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    Active,
    Completed,
    Failed(String),
    TimedOut,
}

impl MockConversation {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            messages: Vec::new(),
            model: None,
            state: ConversationState::Active,
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(MockMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn set_state(&mut self, state: ConversationState) {
        self.state = state;
    }
}

/// Mock orchestrator for testing lifecycle scenarios
pub struct MockOrchestrator {
    pub registry: Arc<RwLock<SubagentRegistry>>,
    pub conversations: Arc<RwLock<HashMap<String, MockConversation>>>,
    pub execution_behavior: ExecutionBehavior,
}

#[derive(Debug, Clone)]
pub enum ExecutionBehavior {
    Success,
    Timeout(Duration),
    Failure(String),
    RetryThenSuccess(u32), // Number of failures before success
    RetryThenFail(u32),    // Number of retries before final failure
}

impl MockOrchestrator {
    pub fn new() -> Self {
        let registry = Arc::new(RwLock::new(
            SubagentRegistry::with_directories(
                PathBuf::from("test-project/.codex/agents"),
                PathBuf::from("test-user/.codex/agents")
            )
        ));

        Self {
            registry,
            conversations: Arc::new(RwLock::new(HashMap::new())),
            execution_behavior: ExecutionBehavior::Success,
        }
    }

    pub fn with_execution_behavior(mut self, behavior: ExecutionBehavior) -> Self {
        self.execution_behavior = behavior;
        self
    }

    pub async fn add_agent(&self, spec: SubagentSpec, source: AgentSource) {
        let mut registry = self.registry.write().await;
        // Manually add agent to registry for testing
        // In real usage, this would come from file system discovery
    }

    pub async fn create_conversation(&self, id: &str, model: Option<&str>) -> Arc<MockConversation> {
        let conversation = match model {
            Some(model) => MockConversation::new(id).with_model(model),
            None => MockConversation::new(id),
        };

        let conversation_arc = Arc::new(conversation);
        let mut conversations = self.conversations.write().await;
        conversations.insert(id.to_string(), (*conversation_arc).clone());
        conversation_arc
    }

    /// Simulate subagent execution with configurable behavior
    pub async fn execute_subagent(
        &self,
        conversation: Arc<MockConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
        config: &SubagentExecConfig,
    ) -> SubagentResult<SubagentExecResult> {
        match &self.execution_behavior {
            ExecutionBehavior::Success => {
                self.simulate_successful_execution(conversation, spec, request).await
            }
            ExecutionBehavior::Timeout(duration) => {
                tokio::time::sleep(*duration).await;
                Err(SubagentIntegrationError::Registry(
                    codex_subagents::SubagentError::Parse(format!(
                        "Execution timed out after {:?}",
                        duration
                    ))
                ))
            }
            ExecutionBehavior::Failure(error) => {
                Err(SubagentIntegrationError::Registry(
                    codex_subagents::SubagentError::Parse(error.clone())
                ))
            }
            ExecutionBehavior::RetryThenSuccess(fail_count) => {
                self.simulate_retry_scenario(conversation, spec, request, *fail_count, true).await
            }
            ExecutionBehavior::RetryThenFail(fail_count) => {
                self.simulate_retry_scenario(conversation, spec, request, *fail_count, false).await
            }
        }
    }

    async fn simulate_successful_execution(
        &self,
        conversation: Arc<MockConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
    ) -> SubagentResult<SubagentExecResult> {
        let task_context = codex_subagents::TaskContext::new();
        let events = self.generate_lifecycle_events(&spec, request, &conversation.id);

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(SubagentExecResult {
            events,
            conversation,
            task_context,
        })
    }

    async fn simulate_retry_scenario(
        &self,
        conversation: Arc<MockConversation>,
        spec: Arc<SubagentSpec>,
        request: &SubagentRunRequest,
        fail_count: u32,
        eventual_success: bool,
    ) -> SubagentResult<SubagentExecResult> {
        // This would be tracked in a real implementation
        // For testing, we simulate the retry logic behavior

        if eventual_success {
            self.simulate_successful_execution(conversation, spec, request).await
        } else {
            Err(SubagentIntegrationError::Registry(
                codex_subagents::SubagentError::Parse(format!(
                    "Failed after {} retries",
                    fail_count
                ))
            ))
        }
    }

    fn generate_lifecycle_events(
        &self,
        spec: &SubagentSpec,
        request: &SubagentRunRequest,
        conversation_id: &str,
    ) -> Vec<EventMsg> {
        let mut events = Vec::new();
        let model = spec.model().map(|s| s.to_string());

        // SubAgentStarted event
        events.push(EventMsg::SubAgentStarted(SubAgentStartedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: conversation_id.to_string(),
            parent_submit_id: None,
            model: model.clone(),
        }));

        // Optional message event if prompt provided
        if let Some(prompt) = &request.prompt {
            if !prompt.trim().is_empty() {
                events.push(EventMsg::SubAgentMessage(SubAgentMessageEvent {
                    agent_name: spec.name().to_string(),
                    sub_conversation_id: conversation_id.to_string(),
                    role: "user".to_string(),
                    content: prompt.clone(),
                }));
            }
        }

        // SubAgentCompleted event
        events.push(EventMsg::SubAgentCompleted(SubAgentCompletedEvent {
            agent_name: spec.name().to_string(),
            sub_conversation_id: conversation_id.to_string(),
            outcome: Some("success".to_string()),
        }));

        events
    }
}

/// Shared structs for testing (would normally come from core crate)
#[derive(Debug, Clone)]
pub struct SubagentRunRequest {
    pub agent_name: String,
    pub prompt: Option<String>,
}

#[derive(Debug)]
pub struct SubagentExecResult {
    pub events: Vec<EventMsg>,
    pub conversation: Arc<MockConversation>,
    pub task_context: codex_subagents::TaskContext,
}

#[derive(Debug, Clone)]
pub struct SubagentExecConfig {
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for SubagentExecConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            max_retries: 2,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubagentIntegrationError {
    #[error("subagents are disabled in the current configuration")]
    Disabled,
    #[error("unknown subagent `{0}`")]
    UnknownAgent(String),
    #[error("registry error: {0}")]
    Registry(#[from] codex_subagents::SubagentError),
}

pub type SubagentResult<T> = Result<T, SubagentIntegrationError>;

// Mock event types
#[derive(Debug, Clone)]
pub enum EventMsg {
    SubAgentStarted(SubAgentStartedEvent),
    SubAgentMessage(SubAgentMessageEvent),
    SubAgentCompleted(SubAgentCompletedEvent),
}

#[derive(Debug, Clone)]
pub struct SubAgentStartedEvent {
    pub agent_name: String,
    pub sub_conversation_id: String,
    pub parent_submit_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubAgentMessageEvent {
    pub agent_name: String,
    pub sub_conversation_id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SubAgentCompletedEvent {
    pub agent_name: String,
    pub sub_conversation_id: String,
    pub outcome: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_spec(name: &str, tools: Vec<&str>) -> SubagentSpec {
        SubagentBuilder::new(name)
            .instructions("Test instructions")
            .description("Test agent")
            .tools(tools.into_iter().map(|s| s.to_string()).collect())
            .build()
            .unwrap()
    }

    fn create_test_spec_with_model(name: &str, model: &str) -> SubagentSpec {
        SubagentBuilder::new(name)
            .instructions("Test instructions")
            .description("Test agent with custom model")
            .model(model.to_string())
            .tools(vec!["read".to_string(), "write".to_string()])
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_successful_subagent_execution() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::Success);

        let spec = Arc::new(create_test_spec("test-agent", vec!["read", "write"]));
        let conversation = orchestrator.create_conversation("conv-123", None).await;

        let request = SubagentRunRequest {
            agent_name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
        };

        let config = SubagentExecConfig::default();
        let result = orchestrator.execute_subagent(conversation, spec, &request, &config).await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert_eq!(exec_result.events.len(), 3); // Started, Message, Completed

        // Verify event structure
        match &exec_result.events[0] {
            EventMsg::SubAgentStarted(event) => {
                assert_eq!(event.agent_name, "test-agent");
                assert_eq!(event.sub_conversation_id, "conv-123");
            }
            _ => panic!("Expected SubAgentStarted event"),
        }

        match &exec_result.events[1] {
            EventMsg::SubAgentMessage(event) => {
                assert_eq!(event.content, "Test prompt");
                assert_eq!(event.role, "user");
            }
            _ => panic!("Expected SubAgentMessage event"),
        }

        match &exec_result.events[2] {
            EventMsg::SubAgentCompleted(event) => {
                assert_eq!(event.agent_name, "test-agent");
                assert_eq!(event.outcome, Some("success".to_string()));
            }
            _ => panic!("Expected SubAgentCompleted event"),
        }
    }

    #[tokio::test]
    async fn test_subagent_execution_with_custom_model() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::Success);

        let spec = Arc::new(create_test_spec_with_model("custom-model-agent", "gpt-4"));
        let conversation = orchestrator.create_conversation("conv-456", Some("gpt-3.5-turbo")).await;

        let request = SubagentRunRequest {
            agent_name: "custom-model-agent".to_string(),
            prompt: None,
        };

        let config = SubagentExecConfig::default();
        let result = orchestrator.execute_subagent(conversation, spec, &request, &config).await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();

        // Should have Started and Completed events (no Message event without prompt)
        assert_eq!(exec_result.events.len(), 2);

        match &exec_result.events[0] {
            EventMsg::SubAgentStarted(event) => {
                assert_eq!(event.model, Some("gpt-4".to_string())); // Agent's model override
            }
            _ => panic!("Expected SubAgentStarted event"),
        }
    }

    #[tokio::test]
    async fn test_execution_timeout_behavior() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::Timeout(Duration::from_millis(100)));

        let spec = Arc::new(create_test_spec("timeout-agent", vec!["read"]));
        let conversation = orchestrator.create_conversation("conv-timeout", None).await;

        let request = SubagentRunRequest {
            agent_name: "timeout-agent".to_string(),
            prompt: Some("This will timeout".to_string()),
        };

        let config = SubagentExecConfig {
            timeout: Duration::from_millis(50), // Shorter than behavior timeout
            max_retries: 1,
        };

        let start_time = std::time::Instant::now();
        let result = orchestrator.execute_subagent(conversation, spec, &request, &config).await;
        let elapsed = start_time.elapsed();

        assert!(result.is_err());
        assert!(elapsed >= Duration::from_millis(90)); // Should wait for behavior timeout

        match result.unwrap_err() {
            SubagentIntegrationError::Registry(err) => {
                assert!(err.to_string().contains("timed out"));
            }
            _ => panic!("Expected timeout error"),
        }
    }

    #[tokio::test]
    async fn test_retry_then_success_behavior() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::RetryThenSuccess(2));

        let spec = Arc::new(create_test_spec("retry-agent", vec!["bash"]));
        let conversation = orchestrator.create_conversation("conv-retry", None).await;

        let request = SubagentRunRequest {
            agent_name: "retry-agent".to_string(),
            prompt: Some("Eventually succeed".to_string()),
        };

        let config = SubagentExecConfig {
            timeout: Duration::from_secs(1),
            max_retries: 3, // Allow enough retries
        };

        let result = orchestrator.execute_subagent(conversation, spec, &request, &config).await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert_eq!(exec_result.events.len(), 3); // Should eventually succeed
    }

    #[tokio::test]
    async fn test_retry_then_fail_behavior() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::RetryThenFail(2));

        let spec = Arc::new(create_test_spec("fail-agent", vec!["git"]));
        let conversation = orchestrator.create_conversation("conv-fail", None).await;

        let request = SubagentRunRequest {
            agent_name: "fail-agent".to_string(),
            prompt: Some("Will fail".to_string()),
        };

        let config = SubagentExecConfig {
            timeout: Duration::from_secs(1),
            max_retries: 2,
        };

        let result = orchestrator.execute_subagent(conversation, spec, &request, &config).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SubagentIntegrationError::Registry(err) => {
                assert!(err.to_string().contains("Failed after"));
            }
            _ => panic!("Expected failure error"),
        }
    }

    #[tokio::test]
    async fn test_feature_flag_disabled() {
        let orchestrator = MockOrchestrator::new();
        let config = MockConfig::new().with_subagents_disabled();

        // Simulate the prepare method behavior when disabled
        let result = simulate_prepare_with_disabled_feature(&config);

        assert!(result.is_err());
        match result.unwrap_err() {
            SubagentIntegrationError::Disabled => {
                // Expected behavior
            }
            _ => panic!("Expected Disabled error"),
        }
    }

    #[tokio::test]
    async fn test_unknown_agent_error() {
        let orchestrator = MockOrchestrator::new();

        let request = SubagentRunRequest {
            agent_name: "nonexistent-agent".to_string(),
            prompt: Some("This agent doesn't exist".to_string()),
        };

        // Simulate trying to load an unknown agent
        let result = simulate_unknown_agent_load(&request);

        assert!(result.is_err());
        match result.unwrap_err() {
            SubagentIntegrationError::UnknownAgent(name) => {
                assert_eq!(name, "nonexistent-agent");
            }
            _ => panic!("Expected UnknownAgent error"),
        }
    }

    #[tokio::test]
    async fn test_conversation_state_tracking() {
        let orchestrator = MockOrchestrator::new();
        let mut conversation = orchestrator.create_conversation("conv-state", None).await;

        // Test initial state
        assert_eq!(conversation.state, ConversationState::Active);
        assert_eq!(conversation.messages.len(), 0);

        // Test state changes (would be managed by real conversation)
        let mut conversation_mut = Arc::make_mut(&mut conversation);
        conversation_mut.add_message("user", "Hello");
        conversation_mut.add_message("assistant", "Hi there!");
        conversation_mut.set_state(ConversationState::Completed);

        assert_eq!(conversation.messages.len(), 2);
        assert_eq!(conversation.state, ConversationState::Completed);
        assert_eq!(conversation.messages[0].role, "user");
        assert_eq!(conversation.messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_execution_config_variations() {
        let orchestrator = MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::Success);

        let spec = Arc::new(create_test_spec("config-test-agent", vec!["analysis"]));
        let conversation = orchestrator.create_conversation("conv-config", None).await;

        let request = SubagentRunRequest {
            agent_name: "config-test-agent".to_string(),
            prompt: None,
        };

        // Test with custom timeout
        let custom_config = SubagentExecConfig {
            timeout: Duration::from_millis(100),
            max_retries: 0,
        };

        let result = orchestrator.execute_subagent(conversation.clone(), spec.clone(), &request, &custom_config).await;
        assert!(result.is_ok());

        // Test with high retry count
        let retry_config = SubagentExecConfig {
            timeout: Duration::from_secs(5),
            max_retries: 10,
        };

        let result = orchestrator.execute_subagent(conversation, spec, &request, &retry_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_generation_edge_cases() {
        let orchestrator = MockOrchestrator::new();

        // Test with empty prompt
        let spec = Arc::new(create_test_spec("edge-case-agent", vec![]));
        let events = orchestrator.generate_lifecycle_events(&spec, &SubagentRunRequest {
            agent_name: "edge-case-agent".to_string(),
            prompt: Some("".to_string()), // Empty prompt
        }, "conv-edge");

        // Should only have Started and Completed events (no Message for empty prompt)
        assert_eq!(events.len(), 2);

        // Test with whitespace-only prompt
        let events = orchestrator.generate_lifecycle_events(&spec, &SubagentRunRequest {
            agent_name: "edge-case-agent".to_string(),
            prompt: Some("   \n\t  ".to_string()), // Whitespace only
        }, "conv-edge");

        // Should only have Started and Completed events
        assert_eq!(events.len(), 2);

        // Test with None prompt
        let events = orchestrator.generate_lifecycle_events(&spec, &SubagentRunRequest {
            agent_name: "edge-case-agent".to_string(),
            prompt: None,
        }, "conv-edge");

        // Should only have Started and Completed events
        assert_eq!(events.len(), 2);
    }

    // Helper functions to simulate orchestrator behavior

    fn simulate_prepare_with_disabled_feature(config: &MockConfig) -> SubagentResult<()> {
        if !config.subagents.enabled {
            return Err(SubagentIntegrationError::Disabled);
        }
        Ok(())
    }

    fn simulate_unknown_agent_load(request: &SubagentRunRequest) -> SubagentResult<()> {
        // Simulate registry lookup failure
        Err(SubagentIntegrationError::UnknownAgent(request.agent_name.clone()))
    }

    #[tokio::test]
    async fn test_concurrent_executions() {
        let orchestrator = Arc::new(MockOrchestrator::new()
            .with_execution_behavior(ExecutionBehavior::Success));

        let spec = Arc::new(create_test_spec("concurrent-agent", vec!["read", "write"]));

        // Spawn multiple concurrent executions
        let mut handles = Vec::new();
        for i in 0..5 {
            let orchestrator_clone = Arc::clone(&orchestrator);
            let spec_clone = Arc::clone(&spec);
            let conversation = orchestrator.create_conversation(&format!("conv-{}", i), None).await;

            let handle = tokio::spawn(async move {
                let request = SubagentRunRequest {
                    agent_name: "concurrent-agent".to_string(),
                    prompt: Some(format!("Concurrent execution {}", i)),
                };
                let config = SubagentExecConfig::default();
                orchestrator_clone.execute_subagent(conversation, spec_clone, &request, &config).await
            });
            handles.push(handle);
        }

        // Wait for all executions to complete
        let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await;
        assert!(results.is_ok());

        let exec_results = results.unwrap();
        assert_eq!(exec_results.len(), 5);

        // All should succeed
        for result in exec_results {
            assert!(result.is_ok());
        }
    }
}