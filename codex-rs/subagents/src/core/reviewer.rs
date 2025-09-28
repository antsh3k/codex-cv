//! Reviewer subagent for automated code review with style and security analysis.

use crate::error::{SubagentError, SubagentResult};
use crate::pipeline::{
    ProposedChanges, ReviewFindings, ReviewFinding, FindingSeverity, FindingCategory,
    ReviewStatus, ConfidenceLevel, FileChange, ChangeType
};
use crate::spec::SubagentSpec;
use crate::task_context::TaskContext;
use crate::traits::{Subagent, TypedSubagent, ContextualSubagent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Review configuration and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// Enable style analysis
    pub style_analysis: bool,
    /// Enable security analysis
    pub security_analysis: bool,
    /// Enable performance analysis
    pub performance_analysis: bool,
    /// Enable maintainability analysis
    pub maintainability_analysis: bool,
    /// Severity threshold for blocking findings
    pub blocking_threshold: FindingSeverity,
    /// Custom lint rules to apply
    pub custom_rules: Vec<LintRule>,
    /// Formatter configurations
    pub formatters: HashMap<String, FormatterConfig>,
}

/// Custom lint rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Pattern to match (regex)
    pub pattern: String,
    /// Severity of violations
    pub severity: FindingSeverity,
    /// Category of the rule
    pub category: FindingCategory,
    /// Suggested fix (optional)
    pub suggested_fix: Option<String>,
}

/// Formatter configuration for specific file types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    /// Command to run formatter
    pub command: String,
    /// File extensions this formatter applies to
    pub extensions: Vec<String>,
    /// Whether to run in check mode only
    pub check_only: bool,
}

/// Input for the Reviewer subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerRequest {
    /// Proposed changes to review
    pub changes: ProposedChanges,
    /// Review configuration and preferences
    pub review_config: ReviewConfig,
    /// Existing codebase context for comparison
    pub codebase_context: Option<CodebaseContext>,
    /// Focus areas for the review
    pub focus_areas: Vec<ReviewFocus>,
}

/// Codebase context for review comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseContext {
    /// Style guide rules
    pub style_guide: StyleGuide,
    /// Security policies
    pub security_policies: Vec<SecurityPolicy>,
    /// Performance benchmarks
    pub performance_benchmarks: HashMap<String, f32>,
    /// Code quality metrics baseline
    pub quality_baseline: QualityBaseline,
}

/// Style guide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleGuide {
    /// Language-specific style rules
    pub language_rules: HashMap<String, Vec<StyleRule>>,
    /// Naming conventions
    pub naming_conventions: HashMap<String, String>,
    /// Formatting preferences
    pub formatting_preferences: HashMap<String, String>,
}

/// Individual style rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Enforcement level
    pub enforcement: EnforcementLevel,
}

/// Security policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy identifier
    pub id: String,
    /// Policy description
    pub description: String,
    /// Patterns to detect violations
    pub violation_patterns: Vec<String>,
    /// Severity of violations
    pub severity: FindingSeverity,
}

/// Quality baseline metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBaseline {
    /// Maximum cyclomatic complexity
    pub max_complexity: usize,
    /// Maximum function length
    pub max_function_length: usize,
    /// Maximum file length
    pub max_file_length: usize,
    /// Required test coverage percentage
    pub min_test_coverage: f32,
}

/// Rule enforcement levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnforcementLevel {
    Error,
    Warning,
    Info,
    Disabled,
}

/// Review focus areas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewFocus {
    Security,
    Performance,
    Maintainability,
    Style,
    Documentation,
    Testing,
    Architecture,
}

/// Output from the Reviewer subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerResponse {
    /// Review findings and analysis
    pub findings: ReviewFindings,
    /// Detailed analysis reports
    pub analysis_reports: AnalysisReports,
    /// Actionable recommendations
    pub recommendations: Vec<ReviewRecommendation>,
    /// Review metrics and statistics
    pub metrics: ReviewMetrics,
}

/// Detailed analysis reports by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReports {
    /// Style analysis results
    pub style_report: Option<StyleAnalysisReport>,
    /// Security analysis results
    pub security_report: Option<SecurityAnalysisReport>,
    /// Performance analysis results
    pub performance_report: Option<PerformanceAnalysisReport>,
    /// Maintainability analysis results
    pub maintainability_report: Option<MaintainabilityAnalysisReport>,
}

/// Style analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleAnalysisReport {
    /// Overall style compliance score
    pub compliance_score: f32,
    /// Formatting issues found
    pub formatting_issues: Vec<FormattingIssue>,
    /// Naming convention violations
    pub naming_violations: Vec<NamingViolation>,
    /// Code organization suggestions
    pub organization_suggestions: Vec<String>,
}

/// Formatting issue details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingIssue {
    /// File where issue was found
    pub file_path: PathBuf,
    /// Line number
    pub line_number: usize,
    /// Issue description
    pub description: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Naming convention violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingViolation {
    /// File where violation was found
    pub file_path: PathBuf,
    /// Line number
    pub line_number: usize,
    /// Identifier name that violates convention
    pub identifier: String,
    /// Expected naming pattern
    pub expected_pattern: String,
    /// Suggested correction
    pub suggested_name: Option<String>,
}

/// Security analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysisReport {
    /// Overall security risk score
    pub risk_score: f32,
    /// Vulnerabilities detected
    pub vulnerabilities: Vec<SecurityVulnerability>,
    /// Security best practices violations
    pub best_practice_violations: Vec<String>,
    /// Recommended security improvements
    pub security_improvements: Vec<String>,
}

/// Security vulnerability details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    /// Vulnerability type
    pub vulnerability_type: String,
    /// File where vulnerability was found
    pub file_path: PathBuf,
    /// Line number
    pub line_number: usize,
    /// Vulnerability description
    pub description: String,
    /// Severity level
    pub severity: FindingSeverity,
    /// Remediation suggestion
    pub remediation: Option<String>,
}

/// Performance analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisReport {
    /// Performance impact score
    pub impact_score: f32,
    /// Performance concerns identified
    pub concerns: Vec<PerformanceConcern>,
    /// Optimization opportunities
    pub optimizations: Vec<String>,
    /// Benchmark comparisons
    pub benchmark_comparisons: HashMap<String, f32>,
}

/// Performance concern details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConcern {
    /// File where concern was found
    pub file_path: PathBuf,
    /// Line range
    pub line_range: (usize, usize),
    /// Concern description
    pub description: String,
    /// Estimated impact
    pub impact: String,
    /// Suggested optimization
    pub optimization: Option<String>,
}

/// Maintainability analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityAnalysisReport {
    /// Maintainability index score
    pub maintainability_index: f32,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Code smells detected
    pub code_smells: Vec<CodeSmell>,
    /// Refactoring suggestions
    pub refactoring_suggestions: Vec<String>,
}

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic_complexity: usize,
    /// Halstead complexity
    pub halstead_complexity: f32,
    /// Lines of code
    pub lines_of_code: usize,
    /// Comment ratio
    pub comment_ratio: f32,
}

/// Code smell detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    /// Smell type
    pub smell_type: String,
    /// File where smell was found
    pub file_path: PathBuf,
    /// Line range
    pub line_range: (usize, usize),
    /// Description
    pub description: String,
    /// Refactoring suggestion
    pub refactoring_suggestion: Option<String>,
}

/// Review recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecommendation {
    /// Recommendation category
    pub category: FindingCategory,
    /// Priority level
    pub priority: Priority,
    /// Recommendation description
    pub description: String,
    /// Implementation steps
    pub implementation_steps: Vec<String>,
    /// Expected impact
    pub expected_impact: String,
}

/// Review metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMetrics {
    /// Total findings count
    pub total_findings: usize,
    /// Findings by severity
    pub findings_by_severity: HashMap<FindingSeverity, usize>,
    /// Findings by category
    pub findings_by_category: HashMap<FindingCategory, usize>,
    /// Review completion time
    pub review_time_ms: u64,
    /// Files reviewed
    pub files_reviewed: usize,
    /// Lines reviewed
    pub lines_reviewed: usize,
}

/// Priority levels for recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Reviewer subagent implementation
pub struct ReviewerSubagent {
    spec: SubagentSpec,
    /// Style analyzer
    style_analyzer: StyleAnalyzer,
    /// Security analyzer
    security_analyzer: SecurityAnalyzer,
    /// Performance analyzer
    performance_analyzer: PerformanceAnalyzer,
    /// Maintainability analyzer
    maintainability_analyzer: MaintainabilityAnalyzer,
}

impl ReviewerSubagent {
    /// Create a new Reviewer subagent
    pub fn new(spec: SubagentSpec) -> Self {
        Self {
            spec,
            style_analyzer: StyleAnalyzer::new(),
            security_analyzer: SecurityAnalyzer::new(),
            performance_analyzer: PerformanceAnalyzer::new(),
            maintainability_analyzer: MaintainabilityAnalyzer::new(),
        }
    }

    /// Perform comprehensive code review
    fn perform_review(
        &self,
        changes: &ProposedChanges,
        config: &ReviewConfig,
        context: &Option<CodebaseContext>,
        focus_areas: &[ReviewFocus],
        ctx: &mut TaskContext,
    ) -> SubagentResult<ReviewFindings> {
        let review_id = format!("review-{}", uuid::Uuid::new_v4());
        let mut review = ReviewFindings::new(review_id, changes.id.clone());

        ctx.info("Starting comprehensive code review");

        // Analyze each file change
        for change in &changes.changes {
            self.analyze_file_change(change, config, context, focus_areas, &mut review, ctx)?;
        }

        // Determine overall review status
        review.status = self.determine_review_status(&review, config);

        // Set confidence level based on analysis completeness
        review.confidence = self.calculate_confidence(&review, config, focus_areas);

        // Generate review summary
        review.summary = self.generate_review_summary(&review, changes);

        ctx.info(&format!(
            "Review completed: {} findings, status: {:?}",
            review.findings.len(),
            review.status
        ));

        Ok(review)
    }

    /// Analyze a single file change
    fn analyze_file_change(
        &self,
        change: &FileChange,
        config: &ReviewConfig,
        context: &Option<CodebaseContext>,
        focus_areas: &[ReviewFocus],
        review: &mut ReviewFindings,
        ctx: &mut TaskContext,
    ) -> SubagentResult<()> {
        ctx.debug(&format!("Analyzing file: {}", change.file_path.display()));

        // Skip analysis for deleted files
        if change.change_type == ChangeType::Delete {
            return Ok(());
        }

        // Style analysis
        if config.style_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Style)) {
            let style_findings = self.style_analyzer.analyze(change, context)?;
            review.findings.extend(style_findings);
        }

        // Security analysis
        if config.security_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Security)) {
            let security_findings = self.security_analyzer.analyze(change, context)?;
            review.findings.extend(security_findings);
        }

        // Performance analysis
        if config.performance_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Performance)) {
            let performance_findings = self.performance_analyzer.analyze(change, context)?;
            review.findings.extend(performance_findings);
        }

        // Maintainability analysis
        if config.maintainability_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Maintainability)) {
            let maintainability_findings = self.maintainability_analyzer.analyze(change, context)?;
            review.findings.extend(maintainability_findings);
        }

        // Apply custom lint rules
        let custom_findings = self.apply_custom_rules(change, &config.custom_rules)?;
        review.findings.extend(custom_findings);

        Ok(())
    }

    /// Apply custom lint rules to a file change
    fn apply_custom_rules(
        &self,
        change: &FileChange,
        rules: &[LintRule],
    ) -> SubagentResult<Vec<ReviewFinding>> {
        let mut findings = Vec::new();

        for rule in rules {
            if let Ok(regex) = regex::Regex::new(&rule.pattern) {
                for (line_num, line) in change.content.lines().enumerate() {
                    if regex.is_match(line) {
                        let finding = ReviewFinding {
                            id: format!("custom-{}-{}", rule.id, line_num + 1),
                            severity: rule.severity.clone(),
                            category: rule.category.clone(),
                            description: format!("{}: {}", rule.description, line.trim()),
                            file_path: change.file_path.clone(),
                            line_range: Some((line_num + 1, line_num + 1)),
                            suggestion: rule.suggested_fix.clone(),
                            rule_id: Some(rule.id.clone()),
                        };
                        findings.push(finding);
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Determine overall review status based on findings
    fn determine_review_status(
        &self,
        review: &ReviewFindings,
        config: &ReviewConfig,
    ) -> ReviewStatus {
        // Check for blocking findings
        let has_blocking = review.findings.iter().any(|f| {
            match config.blocking_threshold {
                FindingSeverity::Info => false,
                FindingSeverity::Minor => matches!(f.severity,
                    FindingSeverity::Major | FindingSeverity::Critical | FindingSeverity::Blocker),
                FindingSeverity::Major => matches!(f.severity,
                    FindingSeverity::Critical | FindingSeverity::Blocker),
                FindingSeverity::Critical => matches!(f.severity, FindingSeverity::Blocker),
                FindingSeverity::Blocker => false,
            }
        });

        if has_blocking {
            ReviewStatus::RequestChanges
        } else if review.findings.is_empty() {
            ReviewStatus::Approved
        } else {
            ReviewStatus::ApprovedWithComments
        }
    }

    /// Calculate confidence level for the review
    fn calculate_confidence(
        &self,
        review: &ReviewFindings,
        config: &ReviewConfig,
        focus_areas: &[ReviewFocus],
    ) -> ConfidenceLevel {
        let mut confidence_score = 0.0;
        let mut max_score = 0.0;

        // Style analysis confidence
        if config.style_analysis {
            max_score += 1.0;
            if focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Style) {
                confidence_score += 1.0;
            }
        }

        // Security analysis confidence
        if config.security_analysis {
            max_score += 1.0;
            if focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Security) {
                confidence_score += 1.0;
            }
        }

        // Performance analysis confidence
        if config.performance_analysis {
            max_score += 1.0;
            if focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Performance) {
                confidence_score += 1.0;
            }
        }

        // Maintainability analysis confidence
        if config.maintainability_analysis {
            max_score += 1.0;
            if focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Maintainability) {
                confidence_score += 1.0;
            }
        }

        let ratio = if max_score > 0.0 { confidence_score / max_score } else { 1.0 };

        if ratio >= 0.8 {
            ConfidenceLevel::High
        } else if ratio >= 0.5 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }

    /// Generate review summary
    fn generate_review_summary(
        &self,
        review: &ReviewFindings,
        changes: &ProposedChanges,
    ) -> String {
        let total_findings = review.findings.len();
        let files_reviewed = changes.changes.len();

        let severity_counts: HashMap<FindingSeverity, usize> = review.findings.iter()
            .fold(HashMap::new(), |mut acc, finding| {
                *acc.entry(finding.severity.clone()).or_insert(0) += 1;
                acc
            });

        let mut summary = format!(
            "Code review completed for {} files with {} findings. ",
            files_reviewed, total_findings
        );

        if total_findings == 0 {
            summary.push_str("No issues found - code is ready for merge.");
        } else {
            let mut severity_parts = Vec::new();

            if let Some(count) = severity_counts.get(&FindingSeverity::Blocker) {
                if *count > 0 {
                    severity_parts.push(format!("{} blocker(s)", count));
                }
            }

            if let Some(count) = severity_counts.get(&FindingSeverity::Critical) {
                if *count > 0 {
                    severity_parts.push(format!("{} critical", count));
                }
            }

            if let Some(count) = severity_counts.get(&FindingSeverity::Major) {
                if *count > 0 {
                    severity_parts.push(format!("{} major", count));
                }
            }

            if !severity_parts.is_empty() {
                summary.push_str(&format!("Found: {}. ", severity_parts.join(", ")));
            }

            match review.status {
                ReviewStatus::RequestChanges => summary.push_str("Changes required before merge."),
                ReviewStatus::ApprovedWithComments => summary.push_str("Approved with minor comments."),
                _ => summary.push_str("Review completed."),
            }
        }

        summary
    }

    /// Generate detailed analysis reports
    fn generate_analysis_reports(
        &self,
        changes: &ProposedChanges,
        config: &ReviewConfig,
        context: &Option<CodebaseContext>,
        focus_areas: &[ReviewFocus],
    ) -> SubagentResult<AnalysisReports> {
        let mut reports = AnalysisReports {
            style_report: None,
            security_report: None,
            performance_report: None,
            maintainability_report: None,
        };

        if config.style_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Style)) {
            reports.style_report = Some(self.style_analyzer.generate_report(changes, context)?);
        }

        if config.security_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Security)) {
            reports.security_report = Some(self.security_analyzer.generate_report(changes, context)?);
        }

        if config.performance_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Performance)) {
            reports.performance_report = Some(self.performance_analyzer.generate_report(changes, context)?);
        }

        if config.maintainability_analysis && (focus_areas.is_empty() || focus_areas.contains(&ReviewFocus::Maintainability)) {
            reports.maintainability_report = Some(self.maintainability_analyzer.generate_report(changes, context)?);
        }

        Ok(reports)
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        findings: &ReviewFindings,
        reports: &AnalysisReports,
    ) -> Vec<ReviewRecommendation> {
        let mut recommendations = Vec::new();

        // Security recommendations
        if findings.findings.iter().any(|f| f.category == FindingCategory::Security) {
            recommendations.push(ReviewRecommendation {
                category: FindingCategory::Security,
                priority: Priority::High,
                description: "Address security findings before deployment".to_string(),
                implementation_steps: vec![
                    "Review all security-related findings".to_string(),
                    "Implement suggested security improvements".to_string(),
                    "Run security scan verification".to_string(),
                ],
                expected_impact: "Reduced security risk and improved compliance".to_string(),
            });
        }

        // Performance recommendations
        if let Some(perf_report) = &reports.performance_report {
            if perf_report.impact_score > 0.5 {
                recommendations.push(ReviewRecommendation {
                    category: FindingCategory::Performance,
                    priority: Priority::Medium,
                    description: "Optimize performance-critical sections".to_string(),
                    implementation_steps: perf_report.optimizations.clone(),
                    expected_impact: "Improved application performance and user experience".to_string(),
                });
            }
        }

        // Maintainability recommendations
        if let Some(maint_report) = &reports.maintainability_report {
            if maint_report.maintainability_index < 0.7 {
                recommendations.push(ReviewRecommendation {
                    category: FindingCategory::Maintainability,
                    priority: Priority::Medium,
                    description: "Improve code maintainability".to_string(),
                    implementation_steps: maint_report.refactoring_suggestions.clone(),
                    expected_impact: "Easier maintenance and reduced technical debt".to_string(),
                });
            }
        }

        recommendations
    }

    /// Calculate review metrics
    fn calculate_metrics(
        &self,
        findings: &ReviewFindings,
        changes: &ProposedChanges,
        review_time_ms: u64,
    ) -> ReviewMetrics {
        let total_findings = findings.findings.len();

        let findings_by_severity = findings.findings.iter()
            .fold(HashMap::new(), |mut acc, finding| {
                *acc.entry(finding.severity.clone()).or_insert(0) += 1;
                acc
            });

        let findings_by_category = findings.findings.iter()
            .fold(HashMap::new(), |mut acc, finding| {
                *acc.entry(finding.category.clone()).or_insert(0) += 1;
                acc
            });

        let files_reviewed = changes.changes.len();
        let lines_reviewed = changes.changes.iter()
            .map(|c| c.content.lines().count())
            .sum();

        ReviewMetrics {
            total_findings,
            findings_by_severity,
            findings_by_category,
            review_time_ms,
            files_reviewed,
            lines_reviewed,
        }
    }
}

impl Subagent for ReviewerSubagent {
    fn spec(&self) -> &SubagentSpec {
        &self.spec
    }

    fn spec_mut(&mut self) -> &mut SubagentSpec {
        &mut self.spec
    }
}

impl TypedSubagent for ReviewerSubagent {
    type Request = ReviewerRequest;
    type Response = ReviewerResponse;

    fn run(
        &mut self,
        ctx: &mut TaskContext,
        request: Self::Request,
    ) -> SubagentResult<Self::Response> {
        let start_time = std::time::Instant::now();

        ctx.info("Starting automated code review");

        // Perform comprehensive review
        let findings = self.perform_review(
            &request.changes,
            &request.review_config,
            &request.codebase_context,
            &request.focus_areas,
            ctx,
        )?;

        // Generate detailed analysis reports
        let analysis_reports = self.generate_analysis_reports(
            &request.changes,
            &request.review_config,
            &request.codebase_context,
            &request.focus_areas,
        )?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&findings, &analysis_reports);

        // Calculate metrics
        let review_time_ms = start_time.elapsed().as_millis() as u64;
        let metrics = self.calculate_metrics(&findings, &request.changes, review_time_ms);

        ctx.info(&format!(
            "Review completed: {} findings across {} files in {}ms",
            metrics.total_findings,
            metrics.files_reviewed,
            review_time_ms
        ));

        Ok(ReviewerResponse {
            findings,
            analysis_reports,
            recommendations,
            metrics,
        })
    }
}

impl ContextualSubagent<CodebaseContext> for ReviewerSubagent {
    fn prepare(&mut self, ctx: &mut TaskContext, context: &CodebaseContext) -> SubagentResult<()> {
        ctx.info("Preparing Reviewer with codebase context");

        // Configure analyzers with context
        self.style_analyzer.configure(&context.style_guide);
        self.security_analyzer.configure(&context.security_policies);
        self.performance_analyzer.configure(&context.performance_benchmarks);
        self.maintainability_analyzer.configure(&context.quality_baseline);

        ctx.info(&format!(
            "Reviewer prepared with {} security policies and {} style rules",
            context.security_policies.len(),
            context.style_guide.language_rules.len()
        ));

        Ok(())
    }
}

// Analyzer implementations would be more complex in practice
// These are simplified implementations for the framework

#[derive(Debug, Clone)]
struct StyleAnalyzer {
    style_guide: Option<StyleGuide>,
}

impl StyleAnalyzer {
    fn new() -> Self {
        Self { style_guide: None }
    }

    fn configure(&mut self, style_guide: &StyleGuide) {
        self.style_guide = Some(style_guide.clone());
    }

    fn analyze(&self, change: &FileChange, _context: &Option<CodebaseContext>) -> SubagentResult<Vec<ReviewFinding>> {
        let mut findings = Vec::new();

        // Simple style checks (in practice, this would be much more sophisticated)
        for (line_num, line) in change.content.lines().enumerate() {
            // Check line length
            if line.len() > 120 {
                findings.push(ReviewFinding {
                    id: format!("style-line-length-{}", line_num + 1),
                    severity: FindingSeverity::Minor,
                    category: FindingCategory::Style,
                    description: format!("Line too long ({} characters)", line.len()),
                    file_path: change.file_path.clone(),
                    line_range: Some((line_num + 1, line_num + 1)),
                    suggestion: Some("Break long line into multiple lines".to_string()),
                    rule_id: Some("max-line-length".to_string()),
                });
            }

            // Check for trailing whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                findings.push(ReviewFinding {
                    id: format!("style-trailing-whitespace-{}", line_num + 1),
                    severity: FindingSeverity::Minor,
                    category: FindingCategory::Style,
                    description: "Trailing whitespace found".to_string(),
                    file_path: change.file_path.clone(),
                    line_range: Some((line_num + 1, line_num + 1)),
                    suggestion: Some("Remove trailing whitespace".to_string()),
                    rule_id: Some("no-trailing-whitespace".to_string()),
                });
            }
        }

        Ok(findings)
    }

    fn generate_report(&self, changes: &ProposedChanges, _context: &Option<CodebaseContext>) -> SubagentResult<StyleAnalysisReport> {
        Ok(StyleAnalysisReport {
            compliance_score: 0.85, // Placeholder
            formatting_issues: Vec::new(),
            naming_violations: Vec::new(),
            organization_suggestions: vec![
                "Consider grouping related functions together".to_string(),
            ],
        })
    }
}

#[derive(Debug, Clone)]
struct SecurityAnalyzer {
    policies: Vec<SecurityPolicy>,
}

impl SecurityAnalyzer {
    fn new() -> Self {
        Self { policies: Vec::new() }
    }

    fn configure(&mut self, policies: &[SecurityPolicy]) {
        self.policies = policies.to_vec();
    }

    fn analyze(&self, change: &FileChange, _context: &Option<CodebaseContext>) -> SubagentResult<Vec<ReviewFinding>> {
        let mut findings = Vec::new();

        // Basic security checks
        for (line_num, line) in change.content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Check for hardcoded secrets
            if line_lower.contains("password") && line_lower.contains("=") {
                findings.push(ReviewFinding {
                    id: format!("security-hardcoded-secret-{}", line_num + 1),
                    severity: FindingSeverity::Critical,
                    category: FindingCategory::Security,
                    description: "Potential hardcoded password detected".to_string(),
                    file_path: change.file_path.clone(),
                    line_range: Some((line_num + 1, line_num + 1)),
                    suggestion: Some("Use environment variables or secure configuration".to_string()),
                    rule_id: Some("no-hardcoded-secrets".to_string()),
                });
            }

            // Check for SQL injection risks
            if line_lower.contains("query") && line_lower.contains("+") {
                findings.push(ReviewFinding {
                    id: format!("security-sql-injection-{}", line_num + 1),
                    severity: FindingSeverity::Major,
                    category: FindingCategory::Security,
                    description: "Potential SQL injection vulnerability".to_string(),
                    file_path: change.file_path.clone(),
                    line_range: Some((line_num + 1, line_num + 1)),
                    suggestion: Some("Use parameterized queries".to_string()),
                    rule_id: Some("prevent-sql-injection".to_string()),
                });
            }
        }

        Ok(findings)
    }

    fn generate_report(&self, _changes: &ProposedChanges, _context: &Option<CodebaseContext>) -> SubagentResult<SecurityAnalysisReport> {
        Ok(SecurityAnalysisReport {
            risk_score: 0.3, // Placeholder
            vulnerabilities: Vec::new(),
            best_practice_violations: Vec::new(),
            security_improvements: vec![
                "Consider adding input validation".to_string(),
                "Implement proper error handling".to_string(),
            ],
        })
    }
}

#[derive(Debug, Clone)]
struct PerformanceAnalyzer {
    benchmarks: HashMap<String, f32>,
}

impl PerformanceAnalyzer {
    fn new() -> Self {
        Self { benchmarks: HashMap::new() }
    }

    fn configure(&mut self, benchmarks: &HashMap<String, f32>) {
        self.benchmarks = benchmarks.clone();
    }

    fn analyze(&self, change: &FileChange, _context: &Option<CodebaseContext>) -> SubagentResult<Vec<ReviewFinding>> {
        let mut findings = Vec::new();

        // Basic performance checks
        for (line_num, line) in change.content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Check for inefficient loops
            if line_lower.contains("for") && line_lower.contains("len(") {
                findings.push(ReviewFinding {
                    id: format!("performance-inefficient-loop-{}", line_num + 1),
                    severity: FindingSeverity::Minor,
                    category: FindingCategory::Performance,
                    description: "Potentially inefficient loop pattern".to_string(),
                    file_path: change.file_path.clone(),
                    line_range: Some((line_num + 1, line_num + 1)),
                    suggestion: Some("Consider using iterators or optimized patterns".to_string()),
                    rule_id: Some("efficient-loops".to_string()),
                });
            }
        }

        Ok(findings)
    }

    fn generate_report(&self, _changes: &ProposedChanges, _context: &Option<CodebaseContext>) -> SubagentResult<PerformanceAnalysisReport> {
        Ok(PerformanceAnalysisReport {
            impact_score: 0.2, // Placeholder
            concerns: Vec::new(),
            optimizations: vec![
                "Consider caching expensive computations".to_string(),
            ],
            benchmark_comparisons: HashMap::new(),
        })
    }
}

#[derive(Debug, Clone)]
struct MaintainabilityAnalyzer {
    baseline: Option<QualityBaseline>,
}

impl MaintainabilityAnalyzer {
    fn new() -> Self {
        Self { baseline: None }
    }

    fn configure(&mut self, baseline: &QualityBaseline) {
        self.baseline = Some(baseline.clone());
    }

    fn analyze(&self, change: &FileChange, _context: &Option<CodebaseContext>) -> SubagentResult<Vec<ReviewFinding>> {
        let mut findings = Vec::new();

        let line_count = change.content.lines().count();

        // Check file length
        if let Some(baseline) = &self.baseline {
            if line_count > baseline.max_file_length {
                findings.push(ReviewFinding {
                    id: "maintainability-file-too-long".to_string(),
                    severity: FindingSeverity::Minor,
                    category: FindingCategory::Maintainability,
                    description: format!("File is too long ({} lines)", line_count),
                    file_path: change.file_path.clone(),
                    line_range: None,
                    suggestion: Some("Consider breaking file into smaller modules".to_string()),
                    rule_id: Some("max-file-length".to_string()),
                });
            }
        }

        Ok(findings)
    }

    fn generate_report(&self, _changes: &ProposedChanges, _context: &Option<CodebaseContext>) -> SubagentResult<MaintainabilityAnalysisReport> {
        Ok(MaintainabilityAnalysisReport {
            maintainability_index: 0.75, // Placeholder
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 5,
                halstead_complexity: 10.5,
                lines_of_code: 150,
                comment_ratio: 0.15,
            },
            code_smells: Vec::new(),
            refactoring_suggestions: vec![
                "Extract common functionality into shared utilities".to_string(),
            ],
        })
    }
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            style_analysis: true,
            security_analysis: true,
            performance_analysis: true,
            maintainability_analysis: true,
            blocking_threshold: FindingSeverity::Major,
            custom_rules: Vec::new(),
            formatters: HashMap::new(),
        }
    }
}

impl Default for CodebaseContext {
    fn default() -> Self {
        Self {
            style_guide: StyleGuide {
                language_rules: HashMap::new(),
                naming_conventions: HashMap::new(),
                formatting_preferences: HashMap::new(),
            },
            security_policies: Vec::new(),
            performance_benchmarks: HashMap::new(),
            quality_baseline: QualityBaseline {
                max_complexity: 10,
                max_function_length: 50,
                max_file_length: 500,
                min_test_coverage: 0.8,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::SubagentSpec;
    use crate::pipeline::{ProposedChanges, FileChange, ChangeType};

    fn create_test_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("reviewer".to_string())
            .description("Test reviewer".to_string())
            .instructions("Review code".to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn test_reviewer_creation() {
        let spec = create_test_spec();
        let reviewer = ReviewerSubagent::new(spec);
        assert_eq!(reviewer.name(), "reviewer");
    }

    #[test]
    fn test_style_analysis() {
        let analyzer = StyleAnalyzer::new();

        let change = FileChange {
            file_path: PathBuf::from("test.rs"),
            change_type: ChangeType::Modify,
            content: "let very_long_line_that_exceeds_the_maximum_allowed_length_and_should_trigger_a_style_violation_in_our_analysis = true;    \n".to_string(),
            line_range: None,
            reason: "test".to_string(),
        };

        let findings = analyzer.analyze(&change, &None).unwrap();
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id.as_ref().map_or(false, |id| id == "max-line-length")));
        assert!(findings.iter().any(|f| f.rule_id.as_ref().map_or(false, |id| id == "no-trailing-whitespace")));
    }

    #[test]
    fn test_security_analysis() {
        let analyzer = SecurityAnalyzer::new();

        let change = FileChange {
            file_path: PathBuf::from("config.rs"),
            change_type: ChangeType::Modify,
            content: "let password = \"hardcoded_secret\";\nlet query = \"SELECT * FROM users WHERE id = \" + user_id;\n".to_string(),
            line_range: None,
            reason: "test".to_string(),
        };

        let findings = analyzer.analyze(&change, &None).unwrap();
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.category == FindingCategory::Security));
    }

    #[test]
    fn test_review_status_determination() {
        let spec = create_test_spec();
        let reviewer = ReviewerSubagent::new(spec);

        let config = ReviewConfig::default();

        // Test with no findings
        let mut review = ReviewFindings::new("test".to_string(), "changes".to_string());
        let status = reviewer.determine_review_status(&review, &config);
        assert_eq!(status, ReviewStatus::Approved);

        // Test with blocker finding
        review.add_finding(ReviewFinding {
            id: "test".to_string(),
            severity: FindingSeverity::Blocker,
            category: FindingCategory::Security,
            description: "Blocker issue".to_string(),
            file_path: PathBuf::from("test.rs"),
            line_range: None,
            suggestion: None,
            rule_id: None,
        });

        let status = reviewer.determine_review_status(&review, &config);
        assert_eq!(status, ReviewStatus::RequestChanges);
    }
}