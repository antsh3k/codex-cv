//! Codex subagent framework primitives (traits, parser, registry, and context utilities).

mod builder;
pub mod core;
mod error;
#[cfg(test)]
mod feature_flag_tests;
#[cfg(test)]
mod integration_tests;
mod parser;
pub mod pipeline;
mod registry;
mod spec;
mod task_context;
mod traits;

pub use builder::SubagentBuilder;
pub use error::SubagentError;
pub use error::SubagentResult;
pub use parser::parse_document;
pub use registry::AgentSource;
pub use registry::ReloadReport;
pub use registry::SubagentRecord;
pub use registry::SubagentRegistry;
pub use registry::SubagentRegistryError;
pub use spec::SubagentSpec;
pub use task_context::DiagnosticLevel;
pub use task_context::TaskContext;
pub use task_context::TaskDiagnostic;
pub use task_context::TaskScratchpadGuard;
pub use traits::ContextualSubagent;
pub use traits::Subagent;
pub use traits::TypedSubagent;

// Re-export pipeline types for convenient access
pub use pipeline::{
    RequirementsSpec, ProposedChanges, TestPlan, ReviewFindings,
    PipelineState, PipelineStage, PipelineTransformer, PipelineValidator,
    ValidationReport, AcceptanceCriterion, Priority, FileChange, ChangeType,
    TestCase, TestType, TestResults, TestStatus, ReviewFinding, FindingSeverity,
    FindingCategory, ReviewStatus
};
