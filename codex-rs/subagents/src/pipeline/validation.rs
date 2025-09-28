//! Validation utilities for pipeline data integrity and consistency checks.

use crate::error::{SubagentError, SubagentResult};
use super::types::*;
use std::collections::HashSet;

/// Comprehensive validation engine for pipeline data integrity
pub struct PipelineValidator {
    /// Whether to perform strict validation
    strict_mode: bool,
}

impl PipelineValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            strict_mode: true,
        }
    }

    /// Create a validator with lenient validation for development
    pub fn lenient() -> Self {
        Self {
            strict_mode: false,
        }
    }

    /// Validate a complete pipeline state for consistency
    pub fn validate_pipeline(&self, state: &PipelineState) -> SubagentResult<ValidationReport> {
        let mut report = ValidationReport::new();

        // Validate individual components
        if let Some(requirements) = &state.requirements {
            self.validate_requirements_spec(requirements, &mut report)?;
        }

        if let Some(changes) = &state.changes {
            self.validate_proposed_changes(changes, &mut report)?;
        }

        if let Some(test_plan) = &state.test_plan {
            self.validate_test_plan(test_plan, &mut report)?;
        }

        if let Some(review) = &state.review {
            self.validate_review_findings(review, &mut report)?;
        }

        // Cross-component validation
        self.validate_component_references(state, &mut report)?;
        self.validate_stage_consistency(state, &mut report)?;

        Ok(report)
    }

    /// Validate requirements specification structure and content
    pub fn validate_requirements_spec(
        &self,
        requirements: &RequirementsSpec,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Required field validation
        if requirements.id.trim().is_empty() {
            report.add_error("Requirements ID cannot be empty");
        }

        if requirements.title.trim().is_empty() {
            report.add_error("Requirements title cannot be empty");
        }

        if requirements.description.trim().is_empty() {
            report.add_warning("Requirements description is empty");
        }

        // Acceptance criteria validation
        if requirements.acceptance_criteria.is_empty() {
            if self.strict_mode {
                report.add_error("Requirements must have at least one acceptance criterion");
            } else {
                report.add_warning("No acceptance criteria defined");
            }
        }

        // Check for duplicate criterion IDs
        let mut criterion_ids = HashSet::new();
        for criterion in &requirements.acceptance_criteria {
            if !criterion_ids.insert(&criterion.id) {
                report.add_error(&format!("Duplicate acceptance criterion ID: {}", criterion.id));
            }

            if criterion.description.trim().is_empty() {
                report.add_error(&format!("Acceptance criterion '{}' has empty description", criterion.id));
            }
        }

        Ok(())
    }

    /// Validate proposed changes structure and safety
    pub fn validate_proposed_changes(
        &self,
        changes: &ProposedChanges,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Required field validation
        if changes.id.trim().is_empty() {
            report.add_error("Changes ID cannot be empty");
        }

        if changes.requirements_id.trim().is_empty() {
            report.add_error("Changes must reference a requirements ID");
        }

        if changes.changes.is_empty() {
            report.add_error("Must have at least one file change");
        }

        if changes.rationale.trim().is_empty() {
            report.add_warning("Change rationale is empty");
        }

        // File change validation
        let mut file_paths = HashSet::new();
        for change in &changes.changes {
            if !file_paths.insert(&change.file_path) {
                report.add_error(&format!(
                    "Duplicate file change for path: {}",
                    change.file_path.display()
                ));
            }

            if change.content.trim().is_empty() && change.change_type != ChangeType::Delete {
                report.add_warning(&format!(
                    "File change for '{}' has empty content",
                    change.file_path.display()
                ));
            }

            if change.reason.trim().is_empty() {
                report.add_warning(&format!(
                    "File change for '{}' has no reason provided",
                    change.file_path.display()
                ));
            }
        }

        // Risk assessment validation
        self.validate_impact_assessment(&changes.impact, report)?;

        Ok(())
    }

    /// Validate test plan structure and coverage
    pub fn validate_test_plan(
        &self,
        test_plan: &TestPlan,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Required field validation
        if test_plan.id.trim().is_empty() {
            report.add_error("Test plan ID cannot be empty");
        }

        if test_plan.requirements_id.trim().is_empty() {
            report.add_error("Test plan must reference a requirements ID");
        }

        if test_plan.changes_id.trim().is_empty() {
            report.add_error("Test plan must reference a changes ID");
        }

        if test_plan.test_cases.is_empty() {
            if self.strict_mode {
                report.add_error("Test plan must have at least one test case");
            } else {
                report.add_warning("Test plan has no test cases");
            }
        }

        // Test case validation
        let mut test_case_ids = HashSet::new();
        for test_case in &test_plan.test_cases {
            if !test_case_ids.insert(&test_case.id) {
                report.add_error(&format!("Duplicate test case ID: {}", test_case.id));
            }

            if test_case.name.trim().is_empty() {
                report.add_error(&format!("Test case '{}' has empty name", test_case.id));
            }

            if test_case.description.trim().is_empty() {
                report.add_warning(&format!("Test case '{}' has empty description", test_case.id));
            }

            if test_case.expected_outcome.trim().is_empty() {
                report.add_warning(&format!("Test case '{}' has no expected outcome", test_case.id));
            }
        }

        // Validate test results if present
        if let Some(results) = &test_plan.results {
            self.validate_test_results(results, test_plan, report)?;
        }

        Ok(())
    }

    /// Validate review findings structure and completeness
    pub fn validate_review_findings(
        &self,
        review: &ReviewFindings,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Required field validation
        if review.id.trim().is_empty() {
            report.add_error("Review ID cannot be empty");
        }

        if review.changes_id.trim().is_empty() {
            report.add_error("Review must reference a changes ID");
        }

        if review.summary.trim().is_empty() {
            report.add_warning("Review summary is empty");
        }

        // Findings validation
        let mut finding_ids = HashSet::new();
        for finding in &review.findings {
            if !finding_ids.insert(&finding.id) {
                report.add_error(&format!("Duplicate review finding ID: {}", finding.id));
            }

            if finding.description.trim().is_empty() {
                report.add_error(&format!("Review finding '{}' has empty description", finding.id));
            }

            // Validate severity and status consistency
            if finding.severity == FindingSeverity::Blocker && review.status == ReviewStatus::Approved {
                report.add_error("Cannot approve review with blocker-level findings");
            }
        }

        // Check for required high-severity finding justification
        let has_critical_findings = review.findings.iter()
            .any(|f| matches!(f.severity, FindingSeverity::Critical | FindingSeverity::Blocker));

        if has_critical_findings && review.confidence == ConfidenceLevel::High {
            report.add_warning("High confidence with critical findings may need justification");
        }

        Ok(())
    }

    /// Validate cross-component references and consistency
    fn validate_component_references(
        &self,
        state: &PipelineState,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Validate requirements → changes reference
        if let (Some(requirements), Some(changes)) = (&state.requirements, &state.changes) {
            if changes.requirements_id != requirements.id {
                report.add_error("Changes requirements_id does not match requirements ID");
            }
        }

        // Validate changes → test plan reference
        if let (Some(changes), Some(test_plan)) = (&state.changes, &state.test_plan) {
            if test_plan.changes_id != changes.id {
                report.add_error("Test plan changes_id does not match changes ID");
            }
        }

        // Validate requirements → test plan reference
        if let (Some(requirements), Some(test_plan)) = (&state.requirements, &state.test_plan) {
            if test_plan.requirements_id != requirements.id {
                report.add_error("Test plan requirements_id does not match requirements ID");
            }
        }

        // Validate changes → review reference
        if let (Some(changes), Some(review)) = (&state.changes, &state.review) {
            if review.changes_id != changes.id {
                report.add_error("Review changes_id does not match changes ID");
            }
        }

        Ok(())
    }

    /// Validate pipeline stage matches available components
    fn validate_stage_consistency(
        &self,
        state: &PipelineState,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        let expected_stage = match (&state.requirements, &state.changes, &state.test_plan, &state.review) {
            (None, None, None, None) => PipelineStage::Specification,
            (Some(_), None, None, None) => PipelineStage::CodeGeneration,
            (Some(_), Some(_), None, None) => PipelineStage::Testing,
            (Some(_), Some(_), Some(_), None) => PipelineStage::Review,
            (Some(_), Some(_), Some(_), Some(_)) => PipelineStage::Complete,
            _ => {
                report.add_warning("Inconsistent component availability for pipeline stage");
                return Ok(());
            }
        };

        if state.stage != expected_stage {
            report.add_warning(&format!(
                "Pipeline stage {:?} does not match component availability (expected {:?})",
                state.stage, expected_stage
            ));
        }

        Ok(())
    }

    /// Validate impact assessment for safety
    fn validate_impact_assessment(
        &self,
        impact: &ImpactAssessment,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Check for breaking changes documentation
        if !impact.breaking_changes.is_empty() && impact.risk_level == RiskLevel::Low {
            report.add_warning("Breaking changes present but risk level is Low");
        }

        // Validate component coverage
        if impact.affected_components.is_empty() && self.strict_mode {
            report.add_warning("No affected components specified");
        }

        Ok(())
    }

    /// Validate test results against test plan
    fn validate_test_results(
        &self,
        results: &TestResults,
        test_plan: &TestPlan,
        report: &mut ValidationReport,
    ) -> SubagentResult<()> {
        // Check that all test cases have results
        let test_case_ids: HashSet<_> = test_plan.test_cases.iter().map(|tc| &tc.id).collect();
        let result_ids: HashSet<_> = results.test_case_results.keys().collect();

        for test_case_id in &test_case_ids {
            if !result_ids.contains(test_case_id) {
                report.add_warning(&format!("Missing test result for test case: {}", test_case_id));
            }
        }

        for result_id in &result_ids {
            if !test_case_ids.contains(result_id) {
                report.add_warning(&format!("Test result for unknown test case: {}", result_id));
            }
        }

        // Validate overall status consistency
        let all_passed = results.test_case_results.values()
            .all(|r| r.status == TestStatus::Passed);
        let any_failed = results.test_case_results.values()
            .any(|r| r.status == TestStatus::Failed);

        match (results.status, all_passed, any_failed) {
            (TestStatus::Passed, false, _) => {
                report.add_error("Overall test status is Passed but not all test cases passed");
            }
            (TestStatus::Failed, true, false) => {
                report.add_error("Overall test status is Failed but all test cases passed");
            }
            _ => {} // Other combinations are valid
        }

        Ok(())
    }
}

impl Default for PipelineValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Report containing validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationReport {
    /// Create a new empty validation report
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Add an info message to the report
    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total number of issues (errors + warnings)
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Get a summary string of the validation results
    pub fn summary(&self) -> String {
        if self.is_valid() {
            if self.warnings.is_empty() {
                "Validation passed".to_string()
            } else {
                format!("Validation passed with {} warnings", self.warnings.len())
            }
        } else {
            format!(
                "Validation failed: {} errors, {} warnings",
                self.errors.len(),
                self.warnings.len()
            )
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());

        report.add_warning("Test warning");
        assert!(report.is_valid());

        report.add_error("Test error");
        assert!(!report.is_valid());
        assert_eq!(report.issue_count(), 2);
    }

    #[test]
    fn test_requirements_validation() {
        let validator = PipelineValidator::new();
        let mut report = ValidationReport::new();

        let requirements = RequirementsSpec::new(
            "req-1".to_string(),
            "Test Requirements".to_string(),
            "Test description".to_string(),
        );

        validator.validate_requirements_spec(&requirements, &mut report).unwrap();
        assert!(!report.is_valid()); // Should fail due to no acceptance criteria in strict mode
    }
}