use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, Duration};

use codex_protocol::mcp_protocol::ConversationId;
use crate::protocol::TokenUsage;
use crate::protocol::SubAgentStartedEvent;
use crate::protocol::SubAgentCompletedEvent;

/// Tracks telemetry metrics for subagent executions using existing infrastructure.
/// This piggybacks on the existing token usage tracking without creating new subsystems.
#[derive(Debug, Clone)]
pub struct SubagentTelemetryTracker {
    inner: Arc<Mutex<TelemetryTrackerInner>>,
}

#[derive(Debug)]
struct TelemetryTrackerInner {
    /// Active subagent sessions with start times
    active_sessions: HashMap<String, SubagentSession>,
    /// Completed sessions with full metrics
    completed_sessions: Vec<SubagentMetrics>,
}

#[derive(Debug, Clone)]
struct SubagentSession {
    agent_name: String,
    sub_conversation_id: ConversationId,
    model: Option<String>,
    start_time: SystemTime,
}

/// Complete metrics for a finished subagent execution.
#[derive(Debug, Clone)]
pub struct SubagentMetrics {
    pub agent_name: String,
    pub sub_conversation_id: ConversationId,
    pub model: Option<String>,
    pub duration: Duration,
    pub token_usage: Option<TokenUsage>,
    pub success: bool,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
}

/// Summary statistics for telemetry dashboards.
#[derive(Debug, Clone)]
pub struct SubagentTelemetrySummary {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub total_tokens: u64,
    pub average_tokens_per_execution: u64,
    pub agent_performance: HashMap<String, AgentPerformanceMetrics>,
}

#[derive(Debug, Clone)]
pub struct AgentPerformanceMetrics {
    pub executions: usize,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub total_tokens: u64,
    pub average_tokens: u64,
}

impl SubagentTelemetryTracker {
    /// Create a new telemetry tracker.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TelemetryTrackerInner {
                active_sessions: HashMap::new(),
                completed_sessions: Vec::new(),
            })),
        }
    }

    /// Record the start of a subagent execution.
    /// Should be called when processing SubAgentStartedEvent.
    pub fn record_subagent_started(&self, event: &SubAgentStartedEvent) {
        let mut inner = self.inner.lock().unwrap();

        let session = SubagentSession {
            agent_name: event.agent_name.clone(),
            sub_conversation_id: ConversationId::from_string(event.sub_conversation_id.clone()).unwrap_or_else(|_| {
                // Fallback to a generated ID if parsing fails
                ConversationId::from_string(uuid::Uuid::new_v4().to_string()).unwrap()
            }),
            model: event.model.clone(),
            start_time: SystemTime::now(),
        };

        inner.active_sessions.insert(event.sub_conversation_id.clone(), session);
    }

    /// Record the completion of a subagent execution with performance metrics.
    /// Should be called when processing SubAgentCompletedEvent.
    pub fn record_subagent_completed(
        &self,
        event: &SubAgentCompletedEvent,
        token_usage: Option<TokenUsage>,
    ) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(session) = inner.active_sessions.remove(&event.sub_conversation_id) {
            let end_time = SystemTime::now();
            let duration = end_time
                .duration_since(session.start_time)
                .unwrap_or_else(|_| Duration::from_secs(0));

            let success = event.outcome.as_deref() != Some("error") && event.outcome.as_deref() != Some("failure");

            let metrics = SubagentMetrics {
                agent_name: session.agent_name,
                sub_conversation_id: session.sub_conversation_id,
                model: session.model,
                duration,
                token_usage,
                success,
                start_time: session.start_time,
                end_time,
            };

            inner.completed_sessions.push(metrics);

            // Limit the number of stored completed sessions to prevent unbounded growth
            if inner.completed_sessions.len() > 1000 {
                inner.completed_sessions.drain(0..100); // Remove oldest 100 entries
            }
        }
    }

    /// Get detailed metrics for a specific subagent by name.
    pub fn get_agent_metrics(&self, agent_name: &str) -> Vec<SubagentMetrics> {
        let inner = self.inner.lock().unwrap();
        inner
            .completed_sessions
            .iter()
            .filter(|m| m.agent_name == agent_name)
            .cloned()
            .collect()
    }

    /// Get summary statistics for all subagent executions.
    pub fn get_summary(&self) -> SubagentTelemetrySummary {
        let inner = self.inner.lock().unwrap();

        if inner.completed_sessions.is_empty() {
            return SubagentTelemetrySummary {
                total_executions: 0,
                successful_executions: 0,
                total_duration: Duration::from_secs(0),
                average_duration: Duration::from_secs(0),
                total_tokens: 0,
                average_tokens_per_execution: 0,
                agent_performance: HashMap::new(),
            };
        }

        let total_executions = inner.completed_sessions.len();
        let successful_executions = inner.completed_sessions.iter().filter(|m| m.success).count();

        let total_duration: Duration = inner.completed_sessions
            .iter()
            .map(|m| m.duration)
            .sum();

        let average_duration = total_duration.div_f64(total_executions as f64);

        let total_tokens: u64 = inner.completed_sessions
            .iter()
            .filter_map(|m| m.token_usage.as_ref())
            .map(|usage| usage.total_tokens)
            .sum();

        let average_tokens_per_execution = if total_executions > 0 {
            total_tokens / total_executions as u64
        } else {
            0
        };

        // Compute per-agent performance metrics
        let mut agent_stats: HashMap<String, Vec<&SubagentMetrics>> = HashMap::new();
        for metrics in &inner.completed_sessions {
            agent_stats.entry(metrics.agent_name.clone())
                .or_default()
                .push(metrics);
        }

        let agent_performance = agent_stats
            .into_iter()
            .map(|(agent_name, metrics)| {
                let executions = metrics.len();
                let successful = metrics.iter().filter(|m| m.success).count();
                let success_rate = successful as f64 / executions as f64;

                let agent_total_duration: Duration = metrics.iter().map(|m| m.duration).sum();
                let average_duration = agent_total_duration.div_f64(executions as f64);

                let agent_total_tokens: u64 = metrics
                    .iter()
                    .filter_map(|m| m.token_usage.as_ref())
                    .map(|usage| usage.total_tokens)
                    .sum();

                let average_tokens = if executions > 0 {
                    agent_total_tokens / executions as u64
                } else {
                    0
                };

                let performance = AgentPerformanceMetrics {
                    executions,
                    success_rate,
                    average_duration,
                    total_tokens: agent_total_tokens,
                    average_tokens,
                };

                (agent_name, performance)
            })
            .collect();

        SubagentTelemetrySummary {
            total_executions,
            successful_executions,
            total_duration,
            average_duration,
            total_tokens,
            average_tokens_per_execution,
            agent_performance,
        }
    }

    /// Get the number of currently active subagent sessions.
    pub fn active_session_count(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.active_sessions.len()
    }

    /// Clear all telemetry data (useful for testing or session reset).
    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.active_sessions.clear();
        inner.completed_sessions.clear();
    }

    /// Get recent subagent executions for debugging/monitoring.
    pub fn get_recent_executions(&self, limit: usize) -> Vec<SubagentMetrics> {
        let inner = self.inner.lock().unwrap();
        inner
            .completed_sessions
            .iter()
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for SubagentTelemetryTracker {
    fn default() -> Self {
        Self::new()
    }
}