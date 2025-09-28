use crate::AgentSource;
use crate::DiagnosticLevel;
use crate::Subagent;
use crate::SubagentBuilder;
use crate::SubagentSpec;
use crate::TaskContext;
use crate::TaskContextError;
use crate::TypedSubagent;
use crate::pipeline::FormatterRun;
use crate::pipeline::ProposedChanges;
use crate::pipeline::RequirementsSpec;
use crate::pipeline::derive_changes_from_spec;
use anyhow::Result;
use anyhow::anyhow;
use serde::Serialize;
use std::borrow::Cow;

const CODE_WRITER_PROMPT: &str = r#"
You are the code writer subagent. Given a structured requirements specification, outline
proposed code changes with clear file-level actions and rationale. Prefer concise bullet
summaries per requirement. Run formatters (just fmt, cargo fmt -- --check) when possible
and report their outcome.
"#;

#[derive(Default)]
pub struct CodeWriterSubagent;

#[derive(Debug, Clone, Serialize)]
pub struct CodeWriterOutput {
    pub changes: ProposedChanges,
    pub formatters: Vec<FormatterRun>,
}

impl CodeWriterSubagent {
    pub fn subagent_spec() -> SubagentSpec {
        SubagentBuilder::new("code-writer")
            .description(Some(
                "Drafts implementation plans and code changes".to_string(),
            ))
            .model(Some("gpt-5-codex".to_string()))
            .tools(["apply_patch", "just", "cargo"])
            .keywords(["implement", "code", "write"])
            .instructions(CODE_WRITER_PROMPT)
            .source(AgentSource::Builtin)
            .build()
            .expect("valid code writer spec")
    }

    fn load_requirements(ctx: &TaskContext) -> Result<RequirementsSpec> {
        ctx.get_typed::<RequirementsSpec>()?
            .ok_or_else(|| anyhow!("RequirementsSpec not present; run spec-parser first"))
    }

    fn simulate_formatters() -> Vec<FormatterRun> {
        let sandbox = std::env::var("CODEX_SANDBOX").unwrap_or_default();
        let mut runs = Vec::new();
        if sandbox == "seatbelt" {
            runs.push(FormatterRun::skipped(
                "just fmt",
                "Formatter skipped in seatbelt sandbox",
            ));
            runs.push(FormatterRun::skipped(
                "cargo fmt -- --check",
                "Formatter skipped in seatbelt sandbox",
            ));
        } else {
            runs.push(FormatterRun::skipped(
                "just fmt",
                "Justfile invocation deferred to orchestrator",
            ));
            runs.push(FormatterRun::skipped(
                "cargo fmt -- --check",
                "Lint run deferred",
            ));
        }
        runs
    }
}

impl Subagent for CodeWriterSubagent {
    fn spec(&self) -> Cow<'_, SubagentSpec> {
        Cow::Owned(Self::subagent_spec())
    }
}

impl TypedSubagent for CodeWriterSubagent {
    type Input = RequirementsSpec;
    type Output = CodeWriterOutput;

    fn prepare(&self, ctx: &TaskContext) -> Result<Self::Input> {
        Self::load_requirements(ctx)
    }

    fn execute(&self, ctx: &mut TaskContext, input: Self::Input) -> Result<Self::Output> {
        let changes = derive_changes_from_spec(&input);
        ctx.push_diagnostic(
            DiagnosticLevel::Info,
            format!("Drafted {} planned changes", changes.changes.len()),
        )?;
        let formatters = Self::simulate_formatters();
        Ok(CodeWriterOutput {
            changes,
            formatters,
        })
    }

    fn finalize(&self, ctx: &mut TaskContext, output: Self::Output) -> Result<()> {
        ctx.insert_typed(output.changes.clone())?;
        ctx.set_scratchpad(
            "subagents.code_writer.output",
            serde_json::to_value(&output).map_err(TaskContextError::Serialization)?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskContext;
    use crate::pipeline::AcceptanceCriterion;
    use crate::pipeline::Requirement;
    use crate::pipeline::RequirementsSpec;
    use pretty_assertions::assert_eq;

    #[test]
    fn generates_plan_from_requirements() {
        let mut ctx = TaskContext::new();
        let spec = RequirementsSpec::new(
            "Demo",
            "Overview",
            vec![Requirement::new(
                "REQ-001",
                "Implement",
                vec![AcceptanceCriterion::new(None, "works")],
                vec!["src/lib.rs".to_string()],
            )],
        );
        ctx.insert_typed(spec.clone()).unwrap();

        let agent = CodeWriterSubagent;
        let input = agent.prepare(&ctx).unwrap();
        assert_eq!(input.requirements.len(), 1);
        let output = agent.execute(&mut ctx, input).unwrap();
        assert_eq!(output.changes.changes.len(), 1);
        agent.finalize(&mut ctx, output).unwrap();
    }
}
