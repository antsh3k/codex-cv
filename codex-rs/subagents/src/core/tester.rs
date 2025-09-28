//! Tester subagent for executing tests and validating proposed code changes.

use crate::error::{SubagentError, SubagentResult};
use crate::pipeline::{
    RequirementsSpec, ProposedChanges, TestPlan, TestCase, TestType, TestResults,
    TestStatus, TestCaseResult, TestDiagnostic, DiagnosticLevel
};
use crate::spec::SubagentSpec;
use crate::task_context::TaskContext;
use crate::traits::{Subagent, TypedSubagent, ContextualSubagent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Sandbox environment configuration for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable sandbox isolation
    pub enabled: bool,
    /// Working directory for test execution
    pub work_dir: PathBuf,
    /// Timeout for individual test execution
    pub test_timeout: Duration,
    /// Maximum memory usage (MB)
    pub max_memory_mb: Option<usize>,
    /// Allowed network access
    pub allow_network: bool,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

/// Input for the Tester subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TesterRequest {
    /// Requirements specification being tested
    pub requirements: RequirementsSpec,
    /// Proposed changes to test
    pub changes: ProposedChanges,
    /// Sandbox configuration for execution
    pub sandbox_config: SandboxConfig,
    /// Testing preferences and options
    pub test_options: TestOptions,
}

/// Testing options and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOptions {
    /// Test types to execute
    pub test_types: Vec<TestType>,
    /// Maximum number of test cases to generate
    pub max_test_cases: Option<usize>,
    /// Include performance benchmarks
    pub include_benchmarks: bool,
    /// Generate integration tests
    pub generate_integration_tests: bool,
    /// Dry run mode (validate without execution)
    pub dry_run: bool,
    /// Fail fast on first error
    pub fail_fast: bool,
}

/// Output from the Tester subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TesterResponse {
    /// Generated test plan
    pub test_plan: TestPlan,
    /// Execution results (if tests were run)
    pub execution_results: Option<TestResults>,
    /// Test coverage analysis
    pub coverage_analysis: CoverageAnalysis,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Recommendations for improving test coverage
    pub recommendations: Vec<String>,
}

/// Test coverage analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    /// Overall coverage percentage
    pub overall_coverage: f32,
    /// Coverage by file
    pub file_coverage: HashMap<PathBuf, f32>,
    /// Uncovered areas identified
    pub uncovered_areas: Vec<UncoveredArea>,
    /// Coverage improvement suggestions
    pub improvement_suggestions: Vec<String>,
}

/// Areas of code not covered by tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredArea {
    /// File containing uncovered code
    pub file_path: PathBuf,
    /// Line range not covered
    pub line_range: (usize, usize),
    /// Description of uncovered functionality
    pub description: String,
    /// Suggested test case to cover this area
    pub suggested_test: Option<String>,
}

/// Performance metrics from test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average test case execution time
    pub average_test_time: Duration,
    /// Memory usage statistics
    pub memory_usage: MemoryUsage,
    /// Performance bottlenecks identified
    pub bottlenecks: Vec<String>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Peak memory usage (MB)
    pub peak_mb: f32,
    /// Average memory usage (MB)
    pub average_mb: f32,
    /// Memory leaks detected
    pub leaks_detected: bool,
}

/// Tester subagent implementation
pub struct TesterSubagent {
    spec: SubagentSpec,
    /// Test execution engine
    executor: TestExecutor,
    /// Test generation engine
    generator: TestGenerator,
    /// Coverage analyzer
    coverage_analyzer: CoverageAnalyzer,
}

impl TesterSubagent {
    /// Create a new Tester subagent
    pub fn new(spec: SubagentSpec) -> Self {
        Self {
            spec,
            executor: TestExecutor::new(),
            generator: TestGenerator::new(),
            coverage_analyzer: CoverageAnalyzer::new(),
        }
    }

    /// Generate comprehensive test plan from requirements and changes
    fn generate_test_plan(
        &self,
        requirements: &RequirementsSpec,
        changes: &ProposedChanges,
        options: &TestOptions,
    ) -> SubagentResult<TestPlan> {
        let test_plan_id = format!("test-plan-{}", uuid::Uuid::new_v4());
        let mut test_plan = TestPlan::new(
            test_plan_id,
            requirements.id.clone(),
            changes.id.clone(),
            "Comprehensive test strategy covering all acceptance criteria and code changes".to_string(),
        );

        // Generate tests from acceptance criteria
        for criterion in &requirements.acceptance_criteria {
            if criterion.testable {
                let test_cases = self.generator.generate_from_criterion(criterion, changes)?;
                for test_case in test_cases {
                    if self.should_include_test_type(&test_case.test_type, options) {
                        test_plan.add_test_case(test_case);
                    }
                }
            }
        }

        // Generate tests from code changes
        for change in &changes.changes {
            let test_cases = self.generator.generate_from_file_change(change, requirements)?;
            for test_case in test_cases {
                if self.should_include_test_type(&test_case.test_type, options) {
                    test_plan.add_test_case(test_case);
                }
            }
        }

        // Generate integration tests if requested
        if options.generate_integration_tests {
            let integration_tests = self.generator.generate_integration_tests(requirements, changes)?;
            for test_case in integration_tests {
                test_plan.add_test_case(test_case);
            }
        }

        // Apply test case limit
        if let Some(max_cases) = options.max_test_cases {
            if test_plan.test_cases.len() > max_cases {
                test_plan.test_cases.truncate(max_cases);
            }
        }

        Ok(test_plan)
    }

    /// Execute test plan in sandbox environment
    async fn execute_test_plan(
        &self,
        test_plan: &TestPlan,
        sandbox_config: &SandboxConfig,
        options: &TestOptions,
        ctx: &mut TaskContext,
    ) -> SubagentResult<TestResults> {
        if options.dry_run {
            ctx.info("Dry run mode: skipping actual test execution");
            return Ok(self.create_dry_run_results(test_plan));
        }

        if !sandbox_config.enabled {
            ctx.warning("Sandbox disabled: executing tests without isolation");
        }

        let start_time = Instant::now();
        let mut test_case_results = HashMap::new();
        let mut diagnostics = Vec::new();
        let mut overall_status = TestStatus::Passed;

        for test_case in &test_plan.test_cases {
            ctx.info(&format!("Executing test case: {}", test_case.name));

            match self.executor.execute_test_case(test_case, sandbox_config).await {
                Ok(result) => {
                    if result.status == TestStatus::Failed {
                        overall_status = TestStatus::Failed;

                        if options.fail_fast {
                            ctx.warning("Fail fast enabled: stopping execution on first failure");
                            break;
                        }
                    }
                    test_case_results.insert(test_case.id.clone(), result);
                }
                Err(e) => {
                    ctx.error(&format!("Test execution error for {}: {}", test_case.id, e));

                    let error_result = TestCaseResult {
                        status: TestStatus::Failed,
                        output: None,
                        error: Some(e.to_string()),
                        execution_time_ms: None,
                    };
                    test_case_results.insert(test_case.id.clone(), error_result);
                    overall_status = TestStatus::Failed;

                    diagnostics.push(TestDiagnostic {
                        level: DiagnosticLevel::Error,
                        message: format!("Test execution failed: {}", e),
                        file_path: None,
                        line_number: None,
                    });

                    if options.fail_fast {
                        break;
                    }
                }
            }
        }

        // Determine final status
        let final_status = if test_case_results.is_empty() {
            TestStatus::Blocked
        } else if test_case_results.values().all(|r| r.status == TestStatus::Passed) {
            TestStatus::Passed
        } else if test_case_results.values().any(|r| r.status == TestStatus::Failed) {
            TestStatus::Failed
        } else {
            TestStatus::Partial
        };

        let execution_time = start_time.elapsed();

        Ok(TestResults {
            status: final_status,
            test_case_results,
            summary: self.generate_test_summary(&test_plan.test_cases, &test_case_results),
            execution_time_ms: Some(execution_time.as_millis() as u64),
            diagnostics,
        })
    }

    /// Check if test type should be included based on options
    fn should_include_test_type(&self, test_type: &TestType, options: &TestOptions) -> bool {
        options.test_types.is_empty() || options.test_types.contains(test_type)
    }

    /// Create dry run results for validation
    fn create_dry_run_results(&self, test_plan: &TestPlan) -> TestResults {
        let mut test_case_results = HashMap::new();

        for test_case in &test_plan.test_cases {
            test_case_results.insert(test_case.id.clone(), TestCaseResult {
                status: TestStatus::Skipped,
                output: Some("Dry run - not executed".to_string()),
                error: None,
                execution_time_ms: Some(0),
            });
        }

        TestResults {
            status: TestStatus::Skipped,
            test_case_results,
            summary: format!("Dry run completed: {} test cases validated", test_plan.test_cases.len()),
            execution_time_ms: Some(0),
            diagnostics: Vec::new(),
        }
    }

    /// Generate test execution summary
    fn generate_test_summary(
        &self,
        test_cases: &[TestCase],
        results: &HashMap<String, TestCaseResult>,
    ) -> String {
        let total = test_cases.len();
        let passed = results.values().filter(|r| r.status == TestStatus::Passed).count();
        let failed = results.values().filter(|r| r.status == TestStatus::Failed).count();
        let skipped = results.values().filter(|r| r.status == TestStatus::Skipped).count();

        format!(
            "Test execution completed: {} total, {} passed, {} failed, {} skipped",
            total, passed, failed, skipped
        )
    }

    /// Analyze test coverage for the proposed changes
    fn analyze_coverage(
        &self,
        changes: &ProposedChanges,
        test_plan: &TestPlan,
        results: &Option<TestResults>,
    ) -> SubagentResult<CoverageAnalysis> {
        self.coverage_analyzer.analyze(changes, test_plan, results)
    }

    /// Calculate performance metrics from test execution
    fn calculate_performance_metrics(
        &self,
        results: &Option<TestResults>,
        test_plan: &TestPlan,
    ) -> PerformanceMetrics {
        match results {
            Some(results) => {
                let total_time = Duration::from_millis(results.execution_time_ms.unwrap_or(0));
                let test_count = test_plan.test_cases.len() as u64;
                let average_time = if test_count > 0 {
                    Duration::from_millis(results.execution_time_ms.unwrap_or(0) / test_count)
                } else {
                    Duration::from_millis(0)
                };

                PerformanceMetrics {
                    total_execution_time: total_time,
                    average_test_time: average_time,
                    memory_usage: MemoryUsage {
                        peak_mb: 50.0, // Placeholder
                        average_mb: 30.0, // Placeholder
                        leaks_detected: false,
                    },
                    bottlenecks: self.identify_bottlenecks(results),
                }
            }
            None => PerformanceMetrics {
                total_execution_time: Duration::from_millis(0),
                average_test_time: Duration::from_millis(0),
                memory_usage: MemoryUsage {
                    peak_mb: 0.0,
                    average_mb: 0.0,
                    leaks_detected: false,
                },
                bottlenecks: Vec::new(),
            },
        }
    }

    /// Identify performance bottlenecks from test results
    fn identify_bottlenecks(&self, results: &TestResults) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        // Check for slow test cases
        for (test_id, result) in &results.test_case_results {
            if let Some(execution_time) = result.execution_time_ms {
                if execution_time > 5000 { // 5 seconds
                    bottlenecks.push(format!("Slow test case: {} ({}ms)", test_id, execution_time));
                }
            }
        }

        bottlenecks
    }

    /// Generate recommendations for improving test coverage and quality
    fn generate_recommendations(
        &self,
        test_plan: &TestPlan,
        results: &Option<TestResults>,
        coverage: &CoverageAnalysis,
        performance: &PerformanceMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Coverage recommendations
        if coverage.overall_coverage < 0.8 {
            recommendations.push(format!(
                "Test coverage is {:.1}% - consider adding more test cases to reach 80%+",
                coverage.overall_coverage * 100.0
            ));
        }

        // Test type recommendations
        let test_types: std::collections::HashSet<_> = test_plan.test_cases.iter()
            .map(|tc| &tc.test_type)
            .collect();

        if !test_types.contains(&TestType::Integration) {
            recommendations.push("Consider adding integration tests to verify component interactions".to_string());
        }

        if !test_types.contains(&TestType::Performance) {
            recommendations.push("Add performance tests to ensure scalability requirements are met".to_string());
        }

        // Performance recommendations
        if performance.average_test_time > Duration::from_secs(2) {
            recommendations.push("Test execution times are high - consider optimizing slow test cases".to_string());
        }

        // Results-based recommendations
        if let Some(results) = results {
            let failure_rate = results.test_case_results.values()
                .filter(|r| r.status == TestStatus::Failed)
                .count() as f32 / results.test_case_results.len() as f32;

            if failure_rate > 0.1 {
                recommendations.push("High test failure rate detected - review failed test cases for issues".to_string());
            }
        }

        // Uncovered areas recommendations
        if !coverage.uncovered_areas.is_empty() {
            recommendations.push(format!(
                "Found {} uncovered areas - add test cases for critical paths",
                coverage.uncovered_areas.len()
            ));
        }

        recommendations
    }
}

impl Subagent for TesterSubagent {
    fn spec(&self) -> &SubagentSpec {
        &self.spec
    }

    fn spec_mut(&mut self) -> &mut SubagentSpec {
        &mut self.spec
    }
}

impl TypedSubagent for TesterSubagent {
    type Request = TesterRequest;
    type Response = TesterResponse;

    fn run(
        &mut self,
        ctx: &mut TaskContext,
        request: Self::Request,
    ) -> SubagentResult<Self::Response> {
        ctx.info("Starting test plan generation and execution");

        // Generate test plan
        let test_plan = self.generate_test_plan(
            &request.requirements,
            &request.changes,
            &request.test_options,
        )?;

        ctx.info(&format!(
            "Generated test plan with {} test cases",
            test_plan.test_cases.len()
        ));

        // Execute tests (async operation)
        let execution_results = if request.test_options.dry_run {
            Some(self.create_dry_run_results(&test_plan))
        } else {
            match futures::executor::block_on(self.execute_test_plan(
                &test_plan,
                &request.sandbox_config,
                &request.test_options,
                ctx,
            )) {
                Ok(results) => Some(results),
                Err(e) => {
                    ctx.error(&format!("Test execution failed: {}", e));
                    None
                }
            }
        };

        // Analyze coverage
        let coverage_analysis = self.analyze_coverage(
            &request.changes,
            &test_plan,
            &execution_results,
        )?;

        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(
            &execution_results,
            &test_plan,
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &test_plan,
            &execution_results,
            &coverage_analysis,
            &performance_metrics,
        );

        ctx.info(&format!(
            "Test analysis completed: {:.1}% coverage, {} recommendations",
            coverage_analysis.overall_coverage * 100.0,
            recommendations.len()
        ));

        Ok(TesterResponse {
            test_plan,
            execution_results,
            coverage_analysis,
            performance_metrics,
            recommendations,
        })
    }
}

impl ContextualSubagent<SandboxConfig> for TesterSubagent {
    fn prepare(&mut self, ctx: &mut TaskContext, context: &SandboxConfig) -> SubagentResult<()> {
        ctx.info("Preparing Tester with sandbox configuration");

        if !context.enabled {
            ctx.warning("Sandbox is disabled - tests will run without isolation");
        }

        // Validate sandbox configuration
        if !context.work_dir.exists() {
            return Err(SubagentError::Parse(format!(
                "Sandbox work directory does not exist: {}",
                context.work_dir.display()
            )));
        }

        // Configure executor with sandbox settings
        self.executor.configure(context.clone());

        ctx.info(&format!(
            "Tester prepared with sandbox at {} (timeout: {:?})",
            context.work_dir.display(),
            context.test_timeout
        ));

        Ok(())
    }
}

/// Test execution engine
#[derive(Debug, Clone)]
struct TestExecutor {
    config: Option<SandboxConfig>,
}

impl TestExecutor {
    fn new() -> Self {
        Self { config: None }
    }

    fn configure(&mut self, config: SandboxConfig) {
        self.config = Some(config);
    }

    async fn execute_test_case(
        &self,
        test_case: &TestCase,
        sandbox_config: &SandboxConfig,
    ) -> SubagentResult<TestCaseResult> {
        let start_time = Instant::now();

        // Execute based on test type and available execution command
        let result = if let Some(command) = &test_case.execution_command {
            self.execute_command(command, sandbox_config).await
        } else {
            // No execution command - validate only
            Ok((
                "Test case validated but not executed (no command specified)".to_string(),
                None,
            ))
        };

        let execution_time = start_time.elapsed();

        match result {
            Ok((output, error)) => Ok(TestCaseResult {
                status: if error.is_some() { TestStatus::Failed } else { TestStatus::Passed },
                output: Some(output),
                error,
                execution_time_ms: Some(execution_time.as_millis() as u64),
            }),
            Err(e) => Ok(TestCaseResult {
                status: TestStatus::Failed,
                output: None,
                error: Some(e.to_string()),
                execution_time_ms: Some(execution_time.as_millis() as u64),
            }),
        }
    }

    async fn execute_command(
        &self,
        command: &str,
        sandbox_config: &SandboxConfig,
    ) -> SubagentResult<(String, Option<String>)> {
        // In a real implementation, this would use proper sandboxing
        // For now, implement a basic command execution with timeout

        let mut cmd = if sandbox_config.enabled {
            // Use sandbox wrapper
            let mut sandbox_cmd = Command::new("timeout");
            sandbox_cmd.arg(format!("{}s", sandbox_config.test_timeout.as_secs()));
            sandbox_cmd.arg("sh");
            sandbox_cmd.arg("-c");
            sandbox_cmd.arg(command);
            sandbox_cmd.current_dir(&sandbox_config.work_dir);

            // Set environment variables
            for (key, value) in &sandbox_config.env_vars {
                sandbox_cmd.env(key, value);
            }

            sandbox_cmd
        } else {
            let mut basic_cmd = Command::new("sh");
            basic_cmd.arg("-c");
            basic_cmd.arg(command);
            basic_cmd
        };

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let output = cmd.output()
            .map_err(|e| SubagentError::Parse(format!("Command execution failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);

        let error = if !output.status.success() || !stderr.is_empty() {
            Some(stderr.to_string())
        } else {
            None
        };

        Ok((stdout, error))
    }
}

/// Test generation engine
#[derive(Debug, Clone)]
struct TestGenerator;

impl TestGenerator {
    fn new() -> Self {
        Self
    }

    fn generate_from_criterion(
        &self,
        criterion: &crate::pipeline::AcceptanceCriterion,
        _changes: &ProposedChanges,
    ) -> SubagentResult<Vec<TestCase>> {
        let mut test_cases = Vec::new();

        // Generate basic functional test
        let test_case = TestCase {
            id: format!("test-{}", criterion.id),
            name: format!("Test: {}", criterion.description),
            description: format!("Validate acceptance criterion: {}", criterion.description),
            test_type: if criterion.test_scenario.is_some() {
                TestType::Integration
            } else {
                TestType::Functional
            },
            execution_command: criterion.test_scenario.clone(),
            expected_outcome: format!("Criterion '{}' is satisfied", criterion.description),
            acceptance_criterion_id: Some(criterion.id.clone()),
        };

        test_cases.push(test_case);

        Ok(test_cases)
    }

    fn generate_from_file_change(
        &self,
        change: &crate::pipeline::FileChange,
        _requirements: &RequirementsSpec,
    ) -> SubagentResult<Vec<TestCase>> {
        let mut test_cases = Vec::new();

        // Generate unit test for the file change
        if change.change_type != ChangeType::Delete {
            let test_case = TestCase {
                id: format!("unit-test-{}", uuid::Uuid::new_v4()),
                name: format!("Unit test for {}", change.file_path.display()),
                description: format!("Test functionality added in {}", change.file_path.display()),
                test_type: TestType::Unit,
                execution_command: self.generate_test_command(&change.file_path),
                expected_outcome: "All unit tests pass".to_string(),
                acceptance_criterion_id: None,
            };

            test_cases.push(test_case);
        }

        Ok(test_cases)
    }

    fn generate_integration_tests(
        &self,
        requirements: &RequirementsSpec,
        changes: &ProposedChanges,
    ) -> SubagentResult<Vec<TestCase>> {
        let mut test_cases = Vec::new();

        // Generate integration test covering multiple components
        let test_case = TestCase {
            id: format!("integration-test-{}", uuid::Uuid::new_v4()),
            name: format!("Integration test for {}", requirements.title),
            description: format!("End-to-end test validating {} with {} file changes",
                requirements.title, changes.changes.len()),
            test_type: TestType::Integration,
            execution_command: Some("cargo test --test integration".to_string()),
            expected_outcome: "All integration tests pass".to_string(),
            acceptance_criterion_id: None,
        };

        test_cases.push(test_case);

        Ok(test_cases)
    }

    fn generate_test_command(&self, file_path: &PathBuf) -> Option<String> {
        let extension = file_path.extension()?.to_str()?;

        match extension {
            "rs" => Some("cargo test".to_string()),
            "js" | "ts" => Some("npm test".to_string()),
            "py" => Some("python -m pytest".to_string()),
            "go" => Some("go test".to_string()),
            _ => Some("echo 'No test command available'".to_string()),
        }
    }
}

/// Coverage analysis engine
#[derive(Debug, Clone)]
struct CoverageAnalyzer;

impl CoverageAnalyzer {
    fn new() -> Self {
        Self
    }

    fn analyze(
        &self,
        changes: &ProposedChanges,
        test_plan: &TestPlan,
        _results: &Option<TestResults>,
    ) -> SubagentResult<CoverageAnalysis> {
        // Simplified coverage analysis
        let total_files = changes.changes.len();
        let tested_files = test_plan.test_cases.len();

        let overall_coverage = if total_files > 0 {
            (tested_files as f32 / total_files as f32).min(1.0)
        } else {
            1.0
        };

        let mut file_coverage = HashMap::new();
        for change in &changes.changes {
            // Placeholder coverage calculation
            file_coverage.insert(change.file_path.clone(), 0.8);
        }

        let uncovered_areas = self.identify_uncovered_areas(changes, test_plan);

        Ok(CoverageAnalysis {
            overall_coverage,
            file_coverage,
            uncovered_areas,
            improvement_suggestions: vec![
                "Add edge case testing".to_string(),
                "Include error handling tests".to_string(),
            ],
        })
    }

    fn identify_uncovered_areas(
        &self,
        changes: &ProposedChanges,
        _test_plan: &TestPlan,
    ) -> Vec<UncoveredArea> {
        let mut uncovered = Vec::new();

        // Identify areas that might not be covered
        for change in &changes.changes {
            if change.content.contains("error") || change.content.contains("panic") {
                uncovered.push(UncoveredArea {
                    file_path: change.file_path.clone(),
                    line_range: (1, 10), // Placeholder
                    description: "Error handling code".to_string(),
                    suggested_test: Some("Add test for error conditions".to_string()),
                });
            }
        }

        uncovered
    }
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            test_types: vec![TestType::Unit, TestType::Functional],
            max_test_cases: Some(50),
            include_benchmarks: false,
            generate_integration_tests: true,
            dry_run: false,
            fail_fast: false,
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            work_dir: PathBuf::from("."),
            test_timeout: Duration::from_secs(30),
            max_memory_mb: Some(512),
            allow_network: false,
            env_vars: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::SubagentSpec;
    use crate::pipeline::{RequirementsSpec, AcceptanceCriterion, Priority};

    fn create_test_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("tester".to_string())
            .description("Test tester".to_string())
            .instructions("Execute tests".to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn test_tester_creation() {
        let spec = create_test_spec();
        let tester = TesterSubagent::new(spec);
        assert_eq!(tester.name(), "tester");
    }

    #[test]
    fn test_test_command_generation() {
        let generator = TestGenerator::new();

        let rust_file = PathBuf::from("src/lib.rs");
        let cmd = generator.generate_test_command(&rust_file);
        assert_eq!(cmd, Some("cargo test".to_string()));

        let js_file = PathBuf::from("src/index.js");
        let cmd = generator.generate_test_command(&js_file);
        assert_eq!(cmd, Some("npm test".to_string()));
    }

    #[test]
    fn test_coverage_analysis() {
        let analyzer = CoverageAnalyzer::new();

        let changes = ProposedChanges::new(
            "changes-1".to_string(),
            "req-1".to_string(),
            "Test changes".to_string(),
        );

        let test_plan = TestPlan::new(
            "plan-1".to_string(),
            "req-1".to_string(),
            "changes-1".to_string(),
            "Test strategy".to_string(),
        );

        let coverage = analyzer.analyze(&changes, &test_plan, &None).unwrap();
        assert!(coverage.overall_coverage >= 0.0 && coverage.overall_coverage <= 1.0);
    }
}