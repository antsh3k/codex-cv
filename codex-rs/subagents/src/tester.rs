use crate::AgentSource;
use crate::DiagnosticLevel;
use crate::Subagent;
use crate::SubagentBuilder;
use crate::SubagentSpec;
use crate::TaskContext;
use crate::TaskContextError;
use crate::TypedSubagent;
use crate::pipeline::ProposedChanges;
use crate::pipeline::TestOutcome;
use crate::pipeline::TestPlan;
use crate::pipeline::TestResults;
use crate::pipeline::TestStatus;
use crate::pipeline::merge_test_results;
use crate::pipeline::plan_tests_for_changes;
use anyhow::Result;
use anyhow::anyhow;
use serde::Serialize;
use std::borrow::Cow;

const TESTER_PROMPT: &str = r#"
You are the tester subagent. Given proposed changes, produce an executable test plan.
Attempt to run each task in a sandbox-safe manner and report its outcome. When execution
is blocked, surface a user-facing message explaining why.
"#;

#[derive(Default)]
pub struct TesterSubagent;

#[derive(Debug, Clone, Serialize)]
pub struct TesterOutput {
    pub results: TestResults,
}

impl TesterSubagent {
    pub fn subagent_spec() -> SubagentSpec {
        SubagentBuilder::new("tester")
            .description(Some("Plans and executes verification steps".to_string()))
            .model(Some("gpt-5-codex".to_string()))
            .tools(["just", "cargo"])
            .keywords(["test", "verify", "qa"])
            .instructions(TESTER_PROMPT)
            .source(AgentSource::Builtin)
            .build()
            .expect("valid tester spec")
    }

    fn load_changes(ctx: &TaskContext) -> Result<ProposedChanges> {
        ctx.get_typed::<ProposedChanges>()?
            .ok_or_else(|| anyhow!("ProposedChanges not present; run code-writer first"))
    }

    fn execute_plan(plan: &TestPlan) -> Vec<TestOutcome> {
        let sandbox = std::env::var("CODEX_SANDBOX").unwrap_or_default();
        let mode = if sandbox.is_empty() {
            None
        } else {
            Some(sandbox.as_str())
        };
        Self::execute_plan_for_mode(plan, mode)
    }

    fn execute_plan_for_mode(plan: &TestPlan, sandbox: Option<&str>) -> Vec<TestOutcome> {
        let mut outcomes = Vec::new();
        let blocked_reason = if sandbox == Some("seatbelt") {
            "Sandbox prohibits executing arbitrary commands"
        } else {
            "Execution deferred to interactive shell"
        };
        for task in &plan.tasks {
            outcomes.push(TestOutcome::new(
                &task.name,
                TestStatus::Blocked,
                Some(blocked_reason.to_string()),
            ));
        }
        outcomes
    }
}

impl Subagent for TesterSubagent {
    fn spec(&self) -> Cow<'_, SubagentSpec> {
        Cow::Owned(Self::subagent_spec())
    }
}

impl TypedSubagent for TesterSubagent {
    type Input = ProposedChanges;
    type Output = TesterOutput;

    fn prepare(&self, ctx: &TaskContext) -> Result<Self::Input> {
        Self::load_changes(ctx)
    }

    fn execute(&self, ctx: &mut TaskContext, input: Self::Input) -> Result<Self::Output> {
        let plan = plan_tests_for_changes(&input);
        ctx.push_diagnostic(
            DiagnosticLevel::Info,
            format!("Prepared {} test tasks", plan.tasks.len()),
        )?;
        let outcomes = Self::execute_plan(&plan);
        let results = merge_test_results(&plan, outcomes);
        Ok(TesterOutput { results })
    }

    fn finalize(&self, ctx: &mut TaskContext, output: Self::Output) -> Result<()> {
        ctx.insert_typed(output.results.clone())?;
        ctx.set_scratchpad(
            "subagents.tester.output",
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
    use crate::pipeline::plan_tests_for_changes;

    #[test]
    fn produces_blocked_results_in_sandbox() {
        let mut ctx = TaskContext::new();
        let changes = ProposedChanges::new(
            "r",
            vec![ProposedChange::new(
                "REQ-001",
                "Work",
                vec![ChangeFile::new("src/lib.rs", ChangeType::Modify, "")],
                vec![],
            )],
        );
        ctx.insert_typed(changes.clone()).unwrap();

        let agent = TesterSubagent;
        let input = agent.prepare(&ctx).unwrap();
        let output = agent.execute(&mut ctx, input).unwrap();
        for outcome in &output.results.outcomes {
            assert!(matches!(outcome.status, TestStatus::Blocked));
        }
        let plan = plan_tests_for_changes(&changes);
        let seatbelt_outcomes = TesterSubagent::execute_plan_for_mode(&plan, Some("seatbelt"));
        for outcome in &seatbelt_outcomes {
            assert!(matches!(outcome.status, TestStatus::Blocked));
            assert!(
                outcome
                    .details
                    .as_ref()
                    .is_some_and(|detail| detail.contains("Sandbox"))
            );
        }
        agent.finalize(&mut ctx, output).unwrap();
    }
}
