//! Core integration layer for Codex subagents.

mod orchestrator;
mod registry;
mod router;

pub use orchestrator::SubagentOrchestrator;
pub use orchestrator::SubagentRunRequest;
pub use orchestrator::SubagentExecResult;
pub use orchestrator::SubagentExecConfig;
pub use registry::CoreSubagentRegistry;
pub use router::SubagentRouter;

/// Shared errors surfaced by the subagent integration.
#[derive(Debug, thiserror::Error)]
pub enum SubagentIntegrationError {
    #[error("subagents are disabled in the current configuration")]
    Disabled,

    #[error("unknown subagent `{0}`")]
    UnknownAgent(String),

    #[error("registry reload failed: {0}")]
    Registry(#[from] codex_subagents::SubagentError),
}

/// Convenience result alias for subagent integration helpers.
pub type SubagentResult<T> = Result<T, SubagentIntegrationError>;
