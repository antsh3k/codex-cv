use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::ConversationManager;
use crate::NewConversation;
use crate::config::Config;
use crate::error::Result as CodexResult;
use crate::protocol::ErrorEvent;
use crate::protocol::EventMsg;
use crate::protocol::InputItem;
use crate::protocol::Op;
use crate::protocol::StreamErrorEvent;
use crate::protocol::SubAgentCompletedEvent;
use crate::protocol::SubAgentMessageEvent;
use crate::protocol::SubAgentOutcome;
use crate::protocol::SubAgentStartedEvent;
use crate::protocol::TaskCompleteEvent;
use crate::protocol::TurnAbortReason;
use crate::protocol::TurnAbortedEvent;
use codex_protocol::mcp_protocol::ConversationId;
use codex_subagents::SubagentSpec;
use codex_subagents::TaskContext;
use codex_subagents::TaskContextError;

#[derive(Debug)]
pub struct SubagentInvocation<'a> {
    pub spec: &'a SubagentSpec,
    pub parent_submit_id: String,
}

#[derive(Debug, Clone)]
pub struct SubagentRunState {
    pub conversation_id: ConversationId,
    pub model: Option<String>,
    pub outcome: SubAgentOutcome,
    pub error: Option<String>,
    pub last_message: Option<String>,
    pub duration: Duration,
}

#[derive(Clone)]
pub struct SubagentOrchestrator {
    conversation_manager: Arc<ConversationManager>,
}

impl SubagentOrchestrator {
    pub fn new(conversation_manager: Arc<ConversationManager>) -> Self {
        Self {
            conversation_manager,
        }
    }

    pub async fn spawn_child(
        &self,
        parent_config: &Config,
        invocation: &SubagentInvocation<'_>,
    ) -> CodexResult<NewConversation> {
        self.conversation_manager
            .spawn_subagent_conversation(parent_config, invocation.spec)
            .await
    }

    pub fn initialize_task_context(spec: &SubagentSpec) -> Result<TaskContext, TaskContextError> {
        let ctx = TaskContext::new();
        ctx.insert_typed(spec.clone())?;
        Ok(ctx)
    }

    pub fn build_started_event(
        invocation: &SubagentInvocation<'_>,
        conversation_id: ConversationId,
        model: Option<String>,
    ) -> EventMsg {
        EventMsg::SubAgentStarted(SubAgentStartedEvent {
            agent_name: invocation.spec.metadata.name.clone(),
            parent_submit_id: invocation.parent_submit_id.clone(),
            sub_conversation_id: conversation_id,
            model,
        })
    }

    pub fn build_message_event(
        spec: &SubagentSpec,
        conversation_id: ConversationId,
        message: impl Into<String>,
    ) -> EventMsg {
        EventMsg::SubAgentMessage(SubAgentMessageEvent {
            agent_name: spec.metadata.name.clone(),
            sub_conversation_id: conversation_id,
            message: message.into(),
        })
    }

    pub fn build_completed_event(
        spec: &SubagentSpec,
        conversation_id: ConversationId,
        outcome: SubAgentOutcome,
        error: Option<String>,
        model: Option<String>,
        duration: Duration,
    ) -> EventMsg {
        let duration_ms = duration.as_millis();
        let duration_ms = duration_ms.min(u128::from(u64::MAX)) as u64;
        EventMsg::SubAgentCompleted(SubAgentCompletedEvent {
            agent_name: spec.metadata.name.clone(),
            sub_conversation_id: conversation_id,
            outcome,
            error,
            model,
            duration_ms: Some(duration_ms),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_subagent<F>(
        &self,
        parent_config: &Config,
        invocation: SubagentInvocation<'_>,
        prompt: Option<String>,
        mut on_event: F,
    ) -> CodexResult<SubagentRunState>
    where
        F: FnMut(EventMsg) + Send,
    {
        let SubagentInvocation {
            spec,
            parent_submit_id,
        } = invocation;
        let invocation_ref = SubagentInvocation {
            spec,
            parent_submit_id: parent_submit_id.clone(),
        };

        let started_at = Instant::now();

        let NewConversation {
            conversation_id,
            conversation,
            session_configured,
        } = self.spawn_child(parent_config, &invocation_ref).await?;

        let model = Some(session_configured.model.clone());
        on_event(Self::build_started_event(
            &invocation_ref,
            conversation_id,
            model.clone(),
        ));

        let default_prompt = "Please execute your standard workflow.".to_string();
        let (prompt_text, prompt_preview) = match prompt {
            Some(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    (default_prompt.clone(), None)
                } else {
                    let preview = trimmed.to_string();
                    (text, Some(preview))
                }
            }
            None => (default_prompt.clone(), None),
        };

        if let Some(preview) = prompt_preview.as_ref() {
            let shortened = if preview.len() > 200 {
                format!("prompt: {}…", &preview[..200])
            } else {
                format!("prompt: {preview}")
            };
            on_event(Self::build_message_event(spec, conversation_id, shortened));
        }

        conversation
            .submit(Op::UserInput {
                items: vec![InputItem::Text { text: prompt_text }],
            })
            .await?;

        let mut last_message: Option<String> = None;
        let mut outcome = SubAgentOutcome::Success;
        let mut error_text: Option<String> = None;

        loop {
            match conversation.next_event().await {
                Ok(event) => match event.msg {
                    EventMsg::AgentMessage(agent_event) => {
                        last_message = Some(agent_event.message.clone());
                        on_event(Self::build_message_event(
                            spec,
                            conversation_id,
                            agent_event.message,
                        ));
                    }
                    EventMsg::AgentMessageDelta(_) => {}
                    EventMsg::TaskComplete(TaskCompleteEvent { last_agent_message }) => {
                        if let Some(message) = last_agent_message
                            .filter(|msg| !msg.trim().is_empty())
                            .filter(|msg| Some(msg) != last_message.as_ref())
                        {
                            last_message = Some(message.clone());
                            on_event(Self::build_message_event(spec, conversation_id, message));
                        }
                        break;
                    }
                    EventMsg::TurnAborted(TurnAbortedEvent { reason }) => {
                        outcome = SubAgentOutcome::Error;
                        let message = match reason {
                            TurnAbortReason::Interrupted => "Subagent turn interrupted".to_string(),
                            TurnAbortReason::Replaced => {
                                "Subagent turn replaced by another task".to_string()
                            }
                            TurnAbortReason::ReviewEnded => {
                                "Subagent review thread ended".to_string()
                            }
                        };
                        error_text = Some(message.clone());
                        last_message = Some(message.clone());
                        on_event(Self::build_message_event(spec, conversation_id, message));
                        break;
                    }
                    EventMsg::Error(ErrorEvent { message }) => {
                        outcome = SubAgentOutcome::Error;
                        error_text = Some(message.clone());
                        let rendered = format!("error: {message}");
                        last_message = Some(rendered.clone());
                        on_event(Self::build_message_event(spec, conversation_id, rendered));
                    }
                    EventMsg::StreamError(StreamErrorEvent { message }) => {
                        let rendered = format!("stream error: {message}");
                        last_message = Some(rendered.clone());
                        on_event(Self::build_message_event(spec, conversation_id, rendered));
                    }
                    EventMsg::ShutdownComplete => {
                        break;
                    }
                    _ => {}
                },
                Err(err) => {
                    outcome = SubAgentOutcome::Error;
                    let message = format!("subagent conversation error: {err}");
                    error_text = Some(message.clone());
                    last_message = Some(message.clone());
                    on_event(Self::build_message_event(spec, conversation_id, message));
                    break;
                }
            }
        }

        let duration = started_at.elapsed();
        on_event(Self::build_completed_event(
            spec,
            conversation_id,
            outcome.clone(),
            error_text.clone(),
            model.clone(),
            duration,
        ));

        crate::telemetry::record_subagent_run(
            &spec.metadata.name,
            duration,
            &outcome,
            model.as_deref(),
        );

        self.conversation_manager
            .remove_conversation(&conversation_id)
            .await;

        Ok(SubagentRunState {
            conversation_id,
            model,
            outcome,
            error: error_text,
            last_message,
            duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::mcp_protocol::ConversationId;
    use codex_subagents::SubagentBuilder;
    use pretty_assertions::assert_eq;

    fn make_spec(name: &str) -> codex_subagents::SubagentSpec {
        SubagentBuilder::new(name)
            .instructions("Do the thing")
            .build()
            .expect("spec")
    }

    #[test]
    fn completed_event_carries_duration() {
        let spec = make_spec("tester");
        let event = SubagentOrchestrator::build_completed_event(
            &spec,
            ConversationId::default(),
            SubAgentOutcome::Success,
            None,
            Some("gpt-5".to_string()),
            Duration::from_millis(1_250),
        );
        let EventMsg::SubAgentCompleted(payload) = event else {
            panic!("expected subagent completed event");
        };
        assert_eq!(payload.duration_ms, Some(1_250));
    }

    #[test]
    fn completed_event_saturates_large_durations() {
        let spec = make_spec("tester");
        let event = SubagentOrchestrator::build_completed_event(
            &spec,
            ConversationId::default(),
            SubAgentOutcome::Success,
            None,
            None,
            Duration::from_secs(u64::MAX),
        );
        let EventMsg::SubAgentCompleted(payload) = event else {
            panic!("expected subagent completed event");
        };
        assert_eq!(payload.duration_ms, Some(u64::MAX));
    }
}
