//! Pipeline transformation utilities for converting between subagent data types.

use crate::error::{SubagentError, SubagentResult};
use super::types::*;
use std::collections::HashMap;

/// Pipeline transformation engine for converting between subagent data types
pub struct PipelineTransformer {
    /// Validation rules for transformations
    validation_enabled: bool,
}

impl PipelineTransformer {
    /// Create a new pipeline transformer
    pub fn new() -> Self {
        Self {
            validation_enabled: true,
        }
    }

    /// Create a transformer with validation disabled (for testing)
    pub fn without_validation() -> Self {
        Self {
            validation_enabled: false,
        }
    }

    /// Transform requirements into a test plan structure
    pub fn requirements_to_test_plan(
        &self,
        requirements: &RequirementsSpec,
        changes_id: String,
    ) -> SubagentResult<TestPlan> {
        if self.validation_enabled {
            self.validate_requirements(requirements)?;
        }

        let test_plan_id = format!("test-plan-{}", requirements.id);
        let mut test_plan = TestPlan::new(
            test_plan_id,
            requirements.id.clone(),
            changes_id,
            "Comprehensive testing strategy based on acceptance criteria".to_string(),
        );

        // Generate test cases from acceptance criteria
        for criterion in &requirements.acceptance_criteria {
            if criterion.testable {
                let test_case = TestCase {
                    id: format!("test-{}", criterion.id),
                    name: format!("Test: {}", criterion.description),
                    description: criterion.description.clone(),
                    test_type: self.infer_test_type(criterion),
                    execution_command: criterion.test_scenario.clone(),
                    expected_outcome: format!("Criterion '{}' is satisfied", criterion.description),
                    acceptance_criterion_id: Some(criterion.id.clone()),
                };
                test_plan.add_test_case(test_case);
            }
        }

        Ok(test_plan)
    }

    /// Transform proposed changes into review scope
    pub fn changes_to_review_scope(
        &self,
        changes: &ProposedChanges,
    ) -> SubagentResult<ReviewFindings> {
        if self.validation_enabled {
            self.validate_changes(changes)?;
        }

        let review_id = format!("review-{}", changes.id);
        let mut review = ReviewFindings::new(review_id, changes.id.clone());

        // Set initial review status based on risk level
        review.status = match changes.impact.risk_level {
            RiskLevel::Low => ReviewStatus::Approved,
            RiskLevel::Medium => ReviewStatus::ApprovedWithComments,
            RiskLevel::High | RiskLevel::Critical => ReviewStatus::RequestChanges,
        };

        // Pre-populate findings based on impact assessment
        if !changes.impact.breaking_changes.is_empty() {
            let finding = ReviewFinding {
                id: "breaking-changes".to_string(),
                severity: FindingSeverity::Major,
                category: FindingCategory::Correctness,
                description: format!("Breaking changes detected: {}",
                    changes.impact.breaking_changes.join(", ")),
                file_path: changes.changes.first()
                    .map(|c| c.file_path.clone())
                    .unwrap_or_default(),
                line_range: None,
                suggestion: Some("Review breaking changes carefully".to_string()),
                rule_id: Some("breaking-change-detection".to_string()),
            };
            review.add_finding(finding);
        }

        Ok(review)
    }

    /// Create a complete pipeline state from individual components
    pub fn create_pipeline_state(
        &self,
        requirements: Option<RequirementsSpec>,
        changes: Option<ProposedChanges>,
        test_plan: Option<TestPlan>,
        review: Option<ReviewFindings>,
    ) -> SubagentResult<PipelineState> {
        let stage = self.determine_pipeline_stage(&requirements, &changes, &test_plan, &review);

        let execution_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let metadata = PipelineMetadata {
            execution_id: execution_id.clone(),
            started_at: now.clone(),
            updated_at: now,
            initiated_by: "codex-subagents".to_string(),
            custom_fields: HashMap::new(),
        };

        Ok(PipelineState {
            stage,
            requirements,
            changes,
            test_plan,
            review,
            metadata,
        })
    }

    /// Merge test results into existing test plan
    pub fn merge_test_results(
        &self,
        test_plan: &mut TestPlan,
        results: TestResults,
    ) -> SubagentResult<()> {
        if self.validation_enabled {
            self.validate_test_results(&results)?;
        }

        test_plan.results = Some(results);
        Ok(())
    }

    /// Update pipeline state with new data
    pub fn update_pipeline_state(
        &self,
        mut state: PipelineState,
        requirements: Option<RequirementsSpec>,
        changes: Option<ProposedChanges>,
        test_plan: Option<TestPlan>,
        review: Option<ReviewFindings>,
    ) -> SubagentResult<PipelineState> {
        // Update components if provided
        if let Some(req) = requirements {
            state.requirements = Some(req);
        }
        if let Some(ch) = changes {
            state.changes = Some(ch);
        }
        if let Some(tp) = test_plan {
            state.test_plan = Some(tp);
        }
        if let Some(rev) = review {
            state.review = Some(rev);
        }

        // Update stage and metadata
        state.stage = self.determine_pipeline_stage(
            &state.requirements,
            &state.changes,
            &state.test_plan,
            &state.review,
        );
        state.metadata.updated_at = chrono::Utc::now().to_rfc3339();

        Ok(state)
    }

    /// Extract summary information from pipeline state
    pub fn extract_summary(&self, state: &PipelineState) -> PipelineSummary {
        PipelineSummary {
            stage: state.stage.clone(),
            requirements_count: state.requirements.as_ref()
                .map(|r| r.acceptance_criteria.len())
                .unwrap_or(0),
            changes_count: state.changes.as_ref()
                .map(|c| c.changes.len())
                .unwrap_or(0),
            test_cases_count: state.test_plan.as_ref()
                .map(|t| t.test_cases.len())
                .unwrap_or(0),
            review_findings_count: state.review.as_ref()
                .map(|r| r.findings.len())
                .unwrap_or(0),
            has_blocking_issues: state.review.as_ref()
                .map(|r| r.has_blocking_findings())
                .unwrap_or(false),
        }
    }

    // Private helper methods

    fn validate_requirements(&self, requirements: &RequirementsSpec) -> SubagentResult<()> {
        if requirements.id.is_empty() {
            return Err(SubagentError::Parse("Requirements ID cannot be empty".to_string()));
        }
        if requirements.title.is_empty() {
            return Err(SubagentError::Parse("Requirements title cannot be empty".to_string()));
        }
        if requirements.acceptance_criteria.is_empty() {
            return Err(SubagentError::Parse("Requirements must have at least one acceptance criterion".to_string()));
        }
        Ok(())
    }

    fn validate_changes(&self, changes: &ProposedChanges) -> SubagentResult<()> {
        if changes.id.is_empty() {
            return Err(SubagentError::Parse("Changes ID cannot be empty".to_string()));
        }
        if changes.changes.is_empty() {
            return Err(SubagentError::Parse("Must have at least one file change".to_string()));
        }
        Ok(())
    }

    fn validate_test_results(&self, results: &TestResults) -> SubagentResult<()> {
        if results.test_case_results.is_empty() {
            return Err(SubagentError::Parse("Test results must contain at least one test case result".to_string()));
        }
        Ok(())
    }

    fn infer_test_type(&self, criterion: &AcceptanceCriterion) -> TestType {
        let description_lower = criterion.description.to_lowercase();

        if description_lower.contains("unit") || description_lower.contains("function") {
            TestType::Unit
        } else if description_lower.contains("integration") || description_lower.contains("api") {
            TestType::Integration
        } else if description_lower.contains("performance") || description_lower.contains("speed") {
            TestType::Performance
        } else if description_lower.contains("security") || description_lower.contains("auth") {
            TestType::Security
        } else {
            TestType::Functional
        }
    }

    fn determine_pipeline_stage(
        &self,
        requirements: &Option<RequirementsSpec>,
        changes: &Option<ProposedChanges>,
        test_plan: &Option<TestPlan>,
        review: &Option<ReviewFindings>,
    ) -> PipelineStage {
        match (requirements, changes, test_plan, review) {
            (None, None, None, None) => PipelineStage::Specification,
            (Some(_), None, None, None) => PipelineStage::CodeGeneration,
            (Some(_), Some(_), None, None) => PipelineStage::Testing,
            (Some(_), Some(_), Some(_), None) => PipelineStage::Review,
            (Some(_), Some(_), Some(_), Some(_)) => PipelineStage::Complete,
            _ => PipelineStage::Specification, // Default fallback
        }
    }
}

impl Default for PipelineTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary view of pipeline state for quick status checks
#[derive(Debug, Clone)]
pub struct PipelineSummary {
    pub stage: PipelineStage,
    pub requirements_count: usize,
    pub changes_count: usize,
    pub test_cases_count: usize,
    pub review_findings_count: usize,
    pub has_blocking_issues: bool,
}

/// Utility functions for common pipeline operations
pub mod utils {
    use super::*;

    /// Generate a unique ID with prefix
    pub fn generate_id(prefix: &str) -> String {
        format!("{}-{}", prefix, uuid::Uuid::new_v4())
    }

    /// Check if pipeline stage represents completion
    pub fn is_complete_stage(stage: &PipelineStage) -> bool {
        matches!(stage, PipelineStage::Complete)
    }

    /// Extract file paths from proposed changes
    pub fn extract_file_paths(changes: &ProposedChanges) -> Vec<&std::path::PathBuf> {
        changes.changes.iter().map(|c| &c.file_path).collect()
    }

    /// Count test cases by status
    pub fn count_tests_by_status(results: &TestResults, status: &TestStatus) -> usize {
        results.test_case_results.values()
            .filter(|r| &r.status == status)
            .count()
    }

    /// Get high severity findings
    pub fn get_high_severity_findings(review: &ReviewFindings) -> Vec<&ReviewFinding> {
        review.findings.iter()
            .filter(|f| matches!(f.severity, FindingSeverity::Major | FindingSeverity::Critical | FindingSeverity::Blocker))
            .collect()
    }
}

// Add required dependencies to Cargo.toml
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_transformer_basic() {
        let transformer = PipelineTransformer::new();

        let requirements = RequirementsSpec::new(
            "req-1".to_string(),
            "Test Requirements".to_string(),
            "Basic test requirements".to_string(),
        );

        let test_plan = transformer.requirements_to_test_plan(&requirements, "changes-1".to_string()).unwrap();
        assert_eq!(test_plan.requirements_id, "req-1");
        assert_eq!(test_plan.changes_id, "changes-1");
    }

    #[test]
    fn test_pipeline_stage_determination() {
        let transformer = PipelineTransformer::new();

        let stage = transformer.determine_pipeline_stage(&None, &None, &None, &None);
        assert_eq!(stage, PipelineStage::Specification);
    }
}