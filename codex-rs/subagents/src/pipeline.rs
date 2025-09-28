use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementsSpec {
    pub title: String,
    pub overview: String,
    pub requirements: Vec<Requirement>,
}

impl RequirementsSpec {
    pub fn new(
        title: impl Into<String>,
        overview: impl Into<String>,
        requirements: Vec<Requirement>,
    ) -> Self {
        Self {
            title: title.into(),
            overview: overview.into(),
            requirements,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Requirement {
    pub id: String,
    pub summary: String,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_hints: Vec<String>,
}

impl Requirement {
    pub fn new(
        id: impl Into<String>,
        summary: impl Into<String>,
        acceptance_criteria: Vec<AcceptanceCriterion>,
        file_hints: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            summary: summary.into(),
            acceptance_criteria,
            file_hints,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcceptanceCriterion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub text: String,
}

impl AcceptanceCriterion {
    pub fn new(id: Option<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposedChanges {
    pub rationale: String,
    pub changes: Vec<ProposedChange>,
}

impl ProposedChanges {
    pub fn new(rationale: impl Into<String>, changes: Vec<ProposedChange>) -> Self {
        Self {
            rationale: rationale.into(),
            changes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposedChange {
    pub requirement_id: String,
    pub summary: String,
    pub files: Vec<ChangeFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl ProposedChange {
    pub fn new(
        requirement_id: impl Into<String>,
        summary: impl Into<String>,
        files: Vec<ChangeFile>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            summary: summary.into(),
            files,
            notes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeFile {
    pub path: String,
    pub change_type: ChangeType,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rationale: String,
}

impl ChangeFile {
    pub fn new(
        path: impl Into<String>,
        change_type: ChangeType,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            change_type,
            rationale: rationale.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Create,
    Modify,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestPlan {
    pub summary: String,
    pub tasks: Vec<TestTask>,
}

impl TestPlan {
    pub fn new(summary: impl Into<String>, tasks: Vec<TestTask>) -> Self {
        Self {
            summary: summary.into(),
            tasks,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestTask {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_requirements: Vec<String>,
}

impl TestTask {
    pub fn new(
        name: impl Into<String>,
        command: impl Into<String>,
        related_requirements: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            related_requirements,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestResults {
    pub plan: TestPlan,
    pub outcomes: Vec<TestOutcome>,
}

impl TestResults {
    pub fn new(plan: TestPlan, outcomes: Vec<TestOutcome>) -> Self {
        Self { plan, outcomes }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestOutcome {
    pub task_name: String,
    pub status: TestStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl TestOutcome {
    pub fn new(task_name: impl Into<String>, status: TestStatus, details: Option<String>) -> Self {
        Self {
            task_name: task_name.into(),
            status,
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Passed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewFindings {
    pub summary: String,
    pub findings: Vec<ReviewFinding>,
}

impl ReviewFindings {
    pub fn new(summary: impl Into<String>, findings: Vec<ReviewFinding>) -> Self {
        Self {
            summary: summary.into(),
            findings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewFinding {
    pub severity: Severity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_requirement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
}

impl ReviewFinding {
    pub fn new(
        severity: Severity,
        message: impl Into<String>,
        related_requirement: Option<String>,
        suggested_fix: Option<String>,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            related_requirement,
            suggested_fix,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormatterRun {
    pub command: String,
    pub status: FormatterStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl FormatterRun {
    pub fn skipped(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: FormatterStatus::Skipped,
            output: Some(reason.into()),
        }
    }

    pub fn success(command: impl Into<String>, output: Option<String>) -> Self {
        Self {
            command: command.into(),
            status: FormatterStatus::Succeeded,
            output,
        }
    }

    pub fn failed(command: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: FormatterStatus::Failed,
            output: Some(output.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormatterStatus {
    Succeeded,
    Failed,
    Skipped,
}

pub fn derive_changes_from_spec(spec: &RequirementsSpec) -> ProposedChanges {
    let mut changes = Vec::new();
    for requirement in &spec.requirements {
        let files = derive_files_for_requirement(requirement);
        let notes = requirement
            .acceptance_criteria
            .iter()
            .map(|criterion| format!("Acceptance: {}", criterion.text))
            .collect();
        changes.push(ProposedChange::new(
            &requirement.id,
            &requirement.summary,
            files,
            notes,
        ));
    }
    ProposedChanges::new(
        format!("Derived from {} requirements", spec.requirements.len()),
        changes,
    )
}

fn derive_files_for_requirement(requirement: &Requirement) -> Vec<ChangeFile> {
    if requirement.file_hints.is_empty() {
        return vec![ChangeFile::new(
            "src/lib.rs",
            ChangeType::Modify,
            format!("Default target inferred for {}", requirement.id),
        )];
    }

    requirement
        .file_hints
        .iter()
        .map(|hint| {
            ChangeFile::new(
                hint,
                ChangeType::Modify,
                format!("Referenced by {}", requirement.id),
            )
        })
        .collect()
}

pub fn plan_tests_for_changes(changes: &ProposedChanges) -> TestPlan {
    let mut tasks = Vec::new();
    let mut related = Vec::new();
    for change in &changes.changes {
        related.push(change.requirement_id.clone());
    }
    if !changes.changes.is_empty() {
        tasks.push(TestTask::new("cargo test", "cargo test", related.clone()));
        tasks.push(TestTask::new("fmt check", "cargo fmt -- --check", related));
    }

    let summary = if tasks.is_empty() {
        "No tests required for documentation-only changes".to_string()
    } else {
        format!("Planned {} automated tasks", tasks.len())
    };
    TestPlan::new(summary, tasks)
}

pub fn merge_test_results(plan: &TestPlan, statuses: Vec<TestOutcome>) -> TestResults {
    let mut outcomes = Vec::new();
    for task in &plan.tasks {
        if let Some(existing) = statuses.iter().find(|status| status.task_name == task.name) {
            outcomes.push(existing.clone());
        } else {
            outcomes.push(TestOutcome::new(
                &task.name,
                TestStatus::Blocked,
                Some("No execution result available".to_string()),
            ));
        }
    }
    TestResults::new(plan.clone(), outcomes)
}

pub fn review_findings_from_results(
    changes: &ProposedChanges,
    results: &TestResults,
) -> ReviewFindings {
    let mut findings = Vec::new();

    for change in &changes.changes {
        if change
            .notes
            .iter()
            .any(|note| note.to_lowercase().contains("security"))
        {
            findings.push(ReviewFinding::new(
                Severity::High,
                format!(
                    "Security-related requirement {} needs dedicated review",
                    change.requirement_id
                ),
                Some(change.requirement_id.clone()),
                None,
            ));
        }
    }

    for outcome in &results.outcomes {
        if outcome.status == TestStatus::Failed {
            findings.push(ReviewFinding::new(
                Severity::Critical,
                format!("Test {} failed", outcome.task_name),
                None,
                outcome.details.clone(),
            ));
        }
        if outcome.status == TestStatus::Blocked {
            findings.push(ReviewFinding::new(
                Severity::Medium,
                format!("Test {} did not run", outcome.task_name),
                None,
                outcome.details.clone(),
            ));
        }
    }

    findings.sort_by(|a, b| b.severity.cmp(&a.severity));

    let summary = if findings.is_empty() {
        "No review findings".to_string()
    } else {
        format!("Identified {} findings", findings.len())
    };

    ReviewFindings::new(summary, findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn derives_changes_with_default_file() {
        let spec = RequirementsSpec::new(
            "Spec",
            "Overview",
            vec![Requirement::new(
                "REQ-001",
                "Add feature",
                vec![
                    AcceptanceCriterion::new(None, "Works"),
                    AcceptanceCriterion::new(None, "Tested"),
                ],
                vec![],
            )],
        );

        let changes = derive_changes_from_spec(&spec);
        assert_eq!(changes.changes.len(), 1);
        let change = &changes.changes[0];
        assert_eq!(change.files.len(), 1);
        assert_eq!(change.files[0].path, "src/lib.rs");
    }

    #[test]
    fn plans_tests_for_changes() {
        let changes = ProposedChanges::new(
            "rationale",
            vec![ProposedChange::new(
                "REQ-001",
                "do it",
                vec![ChangeFile::new("src/main.rs", ChangeType::Modify, "reason")],
                vec![],
            )],
        );

        let plan = plan_tests_for_changes(&changes);
        assert_eq!(plan.tasks.len(), 2);
        assert!(plan.summary.contains("Planned"));
    }

    #[test]
    fn review_findings_cover_failed_tests() {
        let changes = ProposedChanges::new(
            "r",
            vec![ProposedChange::new(
                "REQ-001",
                "Work",
                vec![ChangeFile::new("src/lib.rs", ChangeType::Modify, "")],
                vec![],
            )],
        );
        let plan = plan_tests_for_changes(&changes);
        let results = merge_test_results(
            &plan,
            vec![TestOutcome::new(
                "cargo test",
                TestStatus::Failed,
                Some("compile error".to_string()),
            )],
        );
        let findings = review_findings_from_results(&changes, &results);
        assert!(findings.summary.contains("Identified"));
        assert!(!findings.findings.is_empty());
        assert!(matches!(findings.findings[0].severity, Severity::Critical));
    }
}
