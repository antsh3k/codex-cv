mod builder;
mod code_writer;
mod error;
mod parser;
pub mod pipeline;
mod registry;
mod reviewer;
mod spec;
mod spec_parser;
mod task_context;
mod tester;

pub use builder::SubagentBuilder;
pub use code_writer::CodeWriterOutput;
pub use code_writer::CodeWriterSubagent;
pub use error::AgentParseError;
pub use error::ParserError;
pub use error::RegistryError;
pub use error::SubagentValidationError;
pub use error::TaskContextError;
pub use parser::ParsedAgent;
pub use parser::parse_agent_file;
pub use parser::parse_agent_str;
pub use parser::validate_agent_name;
pub use registry::AgentHandle;
pub use registry::RegistrySnapshot;
pub use registry::SubagentRegistry;
pub use reviewer::ReviewerOutput;
pub use reviewer::ReviewerSubagent;
pub use spec::AgentSource;
pub use spec::SubagentMetadata;
pub use spec::SubagentSpec;
pub use spec_parser::SpecParserOutput;
pub use spec_parser::SpecParserSeed;
pub use spec_parser::SpecParserSubagent;
pub use task_context::DiagnosticEntry;
pub use task_context::DiagnosticLevel;
pub use task_context::TaskContext;
pub use task_context::TaskContextSnapshot;
pub use tester::TesterOutput;
pub use tester::TesterSubagent;

use std::borrow::Cow;

/// Marker trait for any subagent implementation.
pub trait Subagent: Send + Sync {
    fn spec(&self) -> Cow<'_, SubagentSpec>;
}

/// Trait for subagents that expose strongly typed inputs and outputs.
pub trait TypedSubagent: Subagent {
    type Input: Send + Sync + 'static;
    type Output: Send + Sync + 'static;

    fn prepare(&self, ctx: &TaskContext) -> anyhow::Result<Self::Input>;
    fn execute(&self, ctx: &mut TaskContext, input: Self::Input) -> anyhow::Result<Self::Output>;
    fn finalize(&self, ctx: &mut TaskContext, output: Self::Output) -> anyhow::Result<()>;
}

/// Trait for subagents that require additional context seeding prior to execution.
pub trait ContextualSubagent: Subagent {
    fn seed_context(&self, ctx: &mut TaskContext) -> anyhow::Result<()>;
}

/// Helper function to interpret sandbox metadata values.
pub fn seatbelt_active(value: Option<&str>) -> bool {
    matches!(value, Some("seatbelt"))
}
