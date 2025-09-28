use crate::AgentSource;
use crate::DiagnosticLevel;
use crate::Subagent;
use crate::SubagentBuilder;
use crate::SubagentSpec;
use crate::TaskContext;
use crate::TaskContextError;
use crate::TypedSubagent;
use crate::pipeline::ProposedChanges;
use crate::pipeline::ReviewFinding;
use crate::pipeline::ReviewFindings;
use crate::pipeline::Severity;
use crate::pipeline::TestResults;
use crate::pipeline::review_findings_from_results;
use anyhow::Result;
use anyhow::anyhow;
use serde::Serialize;
use std::borrow::Cow;

const REVIEWER_PROMPT: &str = r#"
You are the reviewer subagent. Inspect proposed changes and available test results to
identify risks. Classify findings with severity (info, low, medium, high, critical) and
recommend fixes when possible. Highlight missing tests and potential security issues.
"#;

#[derive(Default)]
pub struct ReviewerSubagent;

#[derive(Debug, Clone, Serialize)]
pub struct ReviewerOutput {
    pub findings: ReviewFindings,
}

impl ReviewerSubagent {
    pub fn subagent_spec() -> SubagentSpec {
        SubagentBuilder::new("reviewer")
            .description(Some("Performs quality and safety review".to_string()))
            .model(Some("gpt-5-codex".to_string()))
            .keywords(["review", "lint", "qa"])
            .instructions(REVIEWER_PROMPT)
            .source(AgentSource::Builtin)
            .build()
            .expect("valid reviewer spec")
    }

    fn load_changes(ctx: &TaskContext) -> Result<ProposedChanges> {
        ctx.get_typed::<ProposedChanges>()?
            .ok_or_else(|| anyhow!("ProposedChanges not available"))
    }

    fn load_results(ctx: &TaskContext) -> Result<TestResults> {
        ctx.get_typed::<TestResults>()?
            .ok_or_else(|| anyhow!("TestResults not available"))
    }

    fn apply_static_heuristics(changes: &ProposedChanges, findings: &mut ReviewFindings) {
        for change in &changes.changes {
            if change.summary.to_lowercase().contains("unsafe") {
                findings.findings.push(ReviewFinding::new(
                    Severity::High,
                    format!(
                        "Requirement {} mentions unsafe operations",
                        change.requirement_id
                    ),
                    Some(change.requirement_id.clone()),
                    Some("Consider refactoring to safe APIs".to_string()),
                ));
            }
        }
    }
}

impl Subagent for ReviewerSubagent {
    fn spec(&self) -> Cow<'_, SubagentSpec> {
        Cow::Owned(Self::subagent_spec())
    }
}

impl TypedSubagent for ReviewerSubagent {
    type Input = (ProposedChanges, TestResults);
    type Output = ReviewerOutput;

    fn prepare(&self, ctx: &TaskContext) -> Result<Self::Input> {
        let changes = Self::load_changes(ctx)?;
        let results = Self::load_results(ctx)?;
        Ok((changes, results))
    }

    fn execute(&self, ctx: &mut TaskContext, input: Self::Input) -> Result<Self::Output> {
        let (changes, results) = input;
        let mut findings = review_findings_from_results(&changes, &results);
        Self::apply_static_heuristics(&changes, &mut findings);
        ctx.push_diagnostic(
            DiagnosticLevel::Info,
            format!("Generated {} review findings", findings.findings.len()),
        )?;
        Ok(ReviewerOutput { findings })
    }

    fn finalize(&self, ctx: &mut TaskContext, output: Self::Output) -> Result<()> {
        ctx.insert_typed(output.findings.clone())?;
        ctx.set_scratchpad(
            "subagents.reviewer.output",
            serde_json::to_value(&output).map_err(TaskContextError::Serialization)?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskContext;
    use crate::pipeline::ChangeFile;
    use crate::pipeline::ChangeType;
    use crate::pipeline::ProposedChange;
    use crate::pipeline::TestOutcome;
    use crate::pipeline::TestStatus;
    use crate::pipeline::merge_test_results;
    use crate::pipeline::plan_tests_for_changes;

    #[test]
    fn highlights_blocked_tests() {
        let mut ctx = TaskContext::new();
        let changes = ProposedChanges::new(
            "r",
            vec![ProposedChange::new(
                "REQ-001",
                "Unsafe API usage",
                vec![ChangeFile::new("src/lib.rs", ChangeType::Modify, "")],
                vec![],
            )],
        );
        let plan = plan_tests_for_changes(&changes);
        let results = merge_test_results(
            &plan,
            vec![TestOutcome::new(
                "cargo test",
                TestStatus::Blocked,
                Some("blocked".to_string()),
            )],
        );
        ctx.insert_typed(changes).unwrap();
        ctx.insert_typed(results).unwrap();

        let agent = ReviewerSubagent;
        let input = agent.prepare(&ctx).unwrap();
        let output = agent.execute(&mut ctx, input).unwrap();
        assert!(!output.findings.findings.is_empty());
        agent.finalize(&mut ctx, output).unwrap();
    }
}
