use std::time::Duration;

use tracing::info;

use crate::protocol::SubAgentOutcome;

/// Emit a telemetry log for a completed subagent run.
pub fn record_subagent_run(
    agent_name: &str,
    duration: Duration,
    outcome: &SubAgentOutcome,
    model: Option<&str>,
) {
    let duration_ms = duration.as_millis().min(u128::from(u64::MAX)) as u64;
    let success = matches!(outcome, SubAgentOutcome::Success);
    info!(
        target = "codex::telemetry",
        event = "subagent_run",
        agent = agent_name,
        model = model.unwrap_or("<session default>"),
        success,
        outcome = ?outcome,
        duration_ms,
    );
}
