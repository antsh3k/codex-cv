//! Code Writer subagent for generating structured code changes from requirements specifications.

use crate::error::{SubagentError, SubagentResult};
use crate::pipeline::{
    RequirementsSpec, ProposedChanges, FileChange, ChangeType, ImpactAssessment, RiskLevel
};
use crate::spec::SubagentSpec;
use crate::task_context::TaskContext;
use crate::traits::{Subagent, TypedSubagent, ContextualSubagent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Repository context for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContext {
    /// Root directory of the repository
    pub root_path: PathBuf,
    /// Programming language(s) detected
    pub languages: Vec<String>,
    /// Key architectural patterns identified
    pub patterns: Vec<String>,
    /// Existing file summaries for context
    pub file_summaries: HashMap<PathBuf, String>,
    /// Dependencies and frameworks in use
    pub dependencies: Vec<String>,
    /// Code style and formatting rules
    pub style_config: Option<StyleConfig>,
}

/// Code style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Indentation style (spaces/tabs)
    pub indentation: String,
    /// Line length limit
    pub max_line_length: Option<usize>,
    /// Naming conventions
    pub naming_conventions: HashMap<String, String>,
    /// Formatter command (e.g., "cargo fmt", "prettier")
    pub formatter_command: Option<String>,
}

/// Input for the Code Writer subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeWriterRequest {
    /// Requirements specification to implement
    pub requirements: RequirementsSpec,
    /// Repository context for code generation
    pub repository_context: RepositoryContext,
    /// Specific target files to focus on (optional)
    pub target_files: Option<Vec<PathBuf>>,
    /// Code generation preferences
    pub preferences: CodeGenerationPreferences,
}

/// Code generation preferences and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationPreferences {
    /// Prefer existing patterns over new ones
    pub prefer_existing_patterns: bool,
    /// Maximum number of files to modify
    pub max_files_to_modify: Option<usize>,
    /// Include comprehensive comments
    pub include_comments: bool,
    /// Generate tests alongside implementation
    pub generate_tests: bool,
    /// Risk tolerance level
    pub risk_tolerance: RiskLevel,
}

/// Output from the Code Writer subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeWriterResponse {
    /// Proposed code changes
    pub changes: ProposedChanges,
    /// Implementation strategy and rationale
    pub implementation_strategy: String,
    /// Code quality metrics and analysis
    pub quality_metrics: CodeQualityMetrics,
    /// Recommendations for next steps
    pub recommendations: Vec<String>,
}

/// Code quality metrics for the proposed changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// Estimated cyclomatic complexity
    pub complexity_estimate: String,
    /// Code reuse score
    pub reuse_score: f32,
    /// Maintainability rating
    pub maintainability: String,
    /// Test coverage potential
    pub test_coverage_potential: f32,
    /// Performance impact assessment
    pub performance_impact: String,
}

/// Code Writer subagent implementation
pub struct CodeWriterSubagent {
    spec: SubagentSpec,
    /// Code analysis utilities
    analyzer: CodeAnalyzer,
    /// Template system for code generation
    template_engine: CodeTemplateEngine,
}

impl CodeWriterSubagent {
    /// Create a new Code Writer subagent
    pub fn new(spec: SubagentSpec) -> Self {
        Self {
            spec,
            analyzer: CodeAnalyzer::new(),
            template_engine: CodeTemplateEngine::new(),
        }
    }

    /// Analyze repository context and generate code strategy
    fn analyze_implementation_strategy(
        &self,
        requirements: &RequirementsSpec,
        context: &RepositoryContext,
        preferences: &CodeGenerationPreferences,
    ) -> SubagentResult<ImplementationStrategy> {
        let mut strategy = ImplementationStrategy::new();

        // Analyze existing codebase patterns
        strategy.architectural_approach = self.determine_architectural_approach(context, requirements)?;

        // Identify target files for modification
        strategy.target_files = self.identify_target_files(requirements, context, preferences)?;

        // Determine implementation order
        strategy.implementation_order = self.plan_implementation_order(&strategy.target_files, requirements)?;

        // Assess risk and complexity
        strategy.risk_assessment = self.assess_implementation_risk(&strategy, preferences)?;

        Ok(strategy)
    }

    /// Generate structured code changes based on strategy
    fn generate_code_changes(
        &self,
        requirements: &RequirementsSpec,
        strategy: &ImplementationStrategy,
        context: &RepositoryContext,
        preferences: &CodeGenerationPreferences,
    ) -> SubagentResult<ProposedChanges> {
        let changes_id = format!("changes-{}", uuid::Uuid::new_v4());
        let mut proposed_changes = ProposedChanges::new(
            changes_id,
            requirements.id.clone(),
            strategy.rationale.clone(),
        );

        // Generate file changes in implementation order
        for (index, file_path) in strategy.implementation_order.iter().enumerate() {
            let change = self.generate_file_change(
                file_path,
                requirements,
                context,
                preferences,
                index,
                &strategy.architectural_approach,
            )?;
            proposed_changes.add_change(change);
        }

        // Set impact assessment
        proposed_changes.impact = self.create_impact_assessment(&strategy, &proposed_changes)?;

        // Record analyzed files
        proposed_changes.analyzed_files = context.file_summaries.keys().cloned().collect();

        Ok(proposed_changes)
    }

    /// Generate change for a specific file
    fn generate_file_change(
        &self,
        file_path: &PathBuf,
        requirements: &RequirementsSpec,
        context: &RepositoryContext,
        preferences: &CodeGenerationPreferences,
        implementation_index: usize,
        architectural_approach: &str,
    ) -> SubagentResult<FileChange> {
        // Determine change type
        let change_type = if file_path.exists() {
            ChangeType::Modify
        } else {
            ChangeType::Create
        };

        // Generate content based on file type and requirements
        let content = self.generate_file_content(
            file_path,
            requirements,
            context,
            architectural_approach,
            preferences,
        )?;

        // Create rationale for this specific change
        let reason = self.generate_change_reason(
            file_path,
            &change_type,
            requirements,
            implementation_index,
        );

        Ok(FileChange {
            file_path: file_path.clone(),
            change_type,
            content,
            line_range: None, // Full file changes for now
            reason,
        })
    }

    /// Generate content for a specific file
    fn generate_file_content(
        &self,
        file_path: &PathBuf,
        requirements: &RequirementsSpec,
        context: &RepositoryContext,
        architectural_approach: &str,
        preferences: &CodeGenerationPreferences,
    ) -> SubagentResult<String> {
        // This would involve sophisticated code generation logic
        // For now, implement a template-based approach

        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let template_context = TemplateContext {
            requirements: requirements.clone(),
            file_path: file_path.clone(),
            architectural_approach: architectural_approach.to_string(),
            repository_context: context.clone(),
            preferences: preferences.clone(),
        };

        match file_extension {
            "rs" => self.template_engine.generate_rust_content(&template_context),
            "ts" | "js" => self.template_engine.generate_typescript_content(&template_context),
            "py" => self.template_engine.generate_python_content(&template_context),
            "go" => self.template_engine.generate_go_content(&template_context),
            _ => self.template_engine.generate_generic_content(&template_context),
        }
    }

    /// Determine architectural approach based on codebase analysis
    fn determine_architectural_approach(
        &self,
        context: &RepositoryContext,
        requirements: &RequirementsSpec,
    ) -> SubagentResult<String> {
        // Analyze existing patterns
        let existing_patterns = &context.patterns;

        if existing_patterns.contains(&"MVC".to_string()) {
            Ok("Follow existing MVC pattern".to_string())
        } else if existing_patterns.contains(&"Microservices".to_string()) {
            Ok("Extend microservices architecture".to_string())
        } else if existing_patterns.contains(&"Modular".to_string()) {
            Ok("Follow modular design principles".to_string())
        } else {
            Ok("Apply clean architecture principles".to_string())
        }
    }

    /// Identify target files for implementation
    fn identify_target_files(
        &self,
        requirements: &RequirementsSpec,
        context: &RepositoryContext,
        preferences: &CodeGenerationPreferences,
    ) -> SubagentResult<Vec<PathBuf>> {
        let mut target_files = Vec::new();

        // Start with explicitly related files
        target_files.extend(requirements.related_files.iter().cloned());

        // Add inferred files based on requirements content
        let inferred_files = self.infer_target_files(requirements, context)?;
        target_files.extend(inferred_files);

        // Remove duplicates and apply constraints
        target_files.sort();
        target_files.dedup();

        // Apply max files constraint
        if let Some(max_files) = preferences.max_files_to_modify {
            target_files.truncate(max_files);
        }

        Ok(target_files)
    }

    /// Infer target files based on requirements analysis
    fn infer_target_files(
        &self,
        requirements: &RequirementsSpec,
        context: &RepositoryContext,
    ) -> SubagentResult<Vec<PathBuf>> {
        let mut inferred = Vec::new();

        // Analyze requirements for file hints
        let description_lower = requirements.description.to_lowercase();

        // Look for common patterns in requirements
        if description_lower.contains("api") || description_lower.contains("endpoint") {
            inferred.extend(self.find_api_files(context));
        }

        if description_lower.contains("database") || description_lower.contains("model") {
            inferred.extend(self.find_model_files(context));
        }

        if description_lower.contains("ui") || description_lower.contains("component") {
            inferred.extend(self.find_ui_files(context));
        }

        if description_lower.contains("test") {
            inferred.extend(self.find_test_files(context));
        }

        Ok(inferred)
    }

    /// Find API-related files in the codebase
    fn find_api_files(&self, context: &RepositoryContext) -> Vec<PathBuf> {
        context
            .file_summaries
            .keys()
            .filter(|path| {
                let path_str = path.to_string_lossy().to_lowercase();
                path_str.contains("api") || path_str.contains("route") || path_str.contains("handler")
            })
            .cloned()
            .collect()
    }

    /// Find model/database files in the codebase
    fn find_model_files(&self, context: &RepositoryContext) -> Vec<PathBuf> {
        context
            .file_summaries
            .keys()
            .filter(|path| {
                let path_str = path.to_string_lossy().to_lowercase();
                path_str.contains("model") || path_str.contains("entity") || path_str.contains("schema")
            })
            .cloned()
            .collect()
    }

    /// Find UI-related files in the codebase
    fn find_ui_files(&self, context: &RepositoryContext) -> Vec<PathBuf> {
        context
            .file_summaries
            .keys()
            .filter(|path| {
                let path_str = path.to_string_lossy().to_lowercase();
                path_str.contains("component") || path_str.contains("view") || path_str.contains("ui")
            })
            .cloned()
            .collect()
    }

    /// Find test files in the codebase
    fn find_test_files(&self, context: &RepositoryContext) -> Vec<PathBuf> {
        context
            .file_summaries
            .keys()
            .filter(|path| {
                let path_str = path.to_string_lossy();
                path_str.contains("test") || path_str.ends_with("_test.rs") || path_str.ends_with(".test.ts")
            })
            .cloned()
            .collect()
    }

    /// Plan implementation order based on dependencies
    fn plan_implementation_order(
        &self,
        target_files: &[PathBuf],
        requirements: &RequirementsSpec,
    ) -> SubagentResult<Vec<PathBuf>> {
        let mut ordered = target_files.to_vec();

        // Sort by implementation priority:
        // 1. Models/data structures first
        // 2. Core logic
        // 3. API/interfaces
        // 4. UI components
        // 5. Tests last

        ordered.sort_by(|a, b| {
            let a_priority = self.get_file_priority(a);
            let b_priority = self.get_file_priority(b);
            a_priority.cmp(&b_priority)
        });

        Ok(ordered)
    }

    /// Get implementation priority for a file (lower = earlier)
    fn get_file_priority(&self, file_path: &PathBuf) -> usize {
        let path_str = file_path.to_string_lossy().to_lowercase();

        if path_str.contains("model") || path_str.contains("entity") || path_str.contains("type") {
            1 // Models first
        } else if path_str.contains("service") || path_str.contains("logic") || path_str.contains("core") {
            2 // Core logic
        } else if path_str.contains("api") || path_str.contains("route") || path_str.contains("handler") {
            3 // API layer
        } else if path_str.contains("component") || path_str.contains("view") || path_str.contains("ui") {
            4 // UI components
        } else if path_str.contains("test") {
            5 // Tests last
        } else {
            3 // Default to middle priority
        }
    }

    /// Assess implementation risk and complexity
    fn assess_implementation_risk(
        &self,
        strategy: &ImplementationStrategy,
        preferences: &CodeGenerationPreferences,
    ) -> SubagentResult<RiskAssessment> {
        let mut risk = RiskAssessment::new();

        // Assess based on number of files
        risk.file_count_risk = match strategy.target_files.len() {
            0..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            6..=10 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        // Assess based on architectural complexity
        risk.architectural_risk = if strategy.architectural_approach.contains("new") {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        };

        // Overall risk assessment
        risk.overall_risk = match (risk.file_count_risk.clone(), risk.architectural_risk.clone()) {
            (RiskLevel::Low, RiskLevel::Low) => RiskLevel::Low,
            (RiskLevel::Low, RiskLevel::Medium) | (RiskLevel::Medium, RiskLevel::Low) => RiskLevel::Medium,
            (RiskLevel::Medium, RiskLevel::Medium) => RiskLevel::Medium,
            _ => RiskLevel::High,
        };

        Ok(risk)
    }

    /// Create impact assessment for proposed changes
    fn create_impact_assessment(
        &self,
        strategy: &ImplementationStrategy,
        changes: &ProposedChanges,
    ) -> SubagentResult<ImpactAssessment> {
        let mut impact = ImpactAssessment {
            risk_level: strategy.risk_assessment.overall_risk.clone(),
            affected_components: Vec::new(),
            breaking_changes: Vec::new(),
            performance_notes: None,
            security_notes: None,
        };

        // Identify affected components
        for change in &changes.changes {
            let component = self.identify_component(&change.file_path);
            if !impact.affected_components.contains(&component) {
                impact.affected_components.push(component);
            }
        }

        // Check for potential breaking changes
        impact.breaking_changes = self.identify_breaking_changes(changes);

        // Add performance notes if relevant
        if changes.changes.len() > 5 {
            impact.performance_notes = Some("Large number of file changes may impact build time".to_string());
        }

        // Add security notes if relevant
        if changes.changes.iter().any(|c| self.is_security_sensitive(&c.file_path)) {
            impact.security_notes = Some("Changes affect security-sensitive files".to_string());
        }

        Ok(impact)
    }

    /// Identify component name from file path
    fn identify_component(&self, file_path: &PathBuf) -> String {
        let path_str = file_path.to_string_lossy();

        if let Some(parent) = file_path.parent() {
            if let Some(parent_name) = parent.file_name() {
                return parent_name.to_string_lossy().to_string();
            }
        }

        if let Some(stem) = file_path.file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Identify potential breaking changes
    fn identify_breaking_changes(&self, changes: &ProposedChanges) -> Vec<String> {
        let mut breaking_changes = Vec::new();

        for change in &changes.changes {
            match change.change_type {
                ChangeType::Delete => {
                    breaking_changes.push(format!("File deletion: {}", change.file_path.display()));
                }
                ChangeType::Rename { from: _ } => {
                    breaking_changes.push(format!("File rename: {}", change.file_path.display()));
                }
                _ => {}
            }

            // Check for API/interface changes
            if self.is_public_interface(&change.file_path) {
                breaking_changes.push(format!("Public interface change: {}", change.file_path.display()));
            }
        }

        breaking_changes
    }

    /// Check if file represents a public interface
    fn is_public_interface(&self, file_path: &PathBuf) -> bool {
        let path_str = file_path.to_string_lossy().to_lowercase();
        path_str.contains("api") || path_str.contains("public") || path_str.contains("interface")
    }

    /// Check if file is security-sensitive
    fn is_security_sensitive(&self, file_path: &PathBuf) -> bool {
        let path_str = file_path.to_string_lossy().to_lowercase();
        path_str.contains("auth") || path_str.contains("security") || path_str.contains("crypto")
    }

    /// Generate reason for a specific file change
    fn generate_change_reason(
        &self,
        file_path: &PathBuf,
        change_type: &ChangeType,
        requirements: &RequirementsSpec,
        implementation_index: usize,
    ) -> String {
        let action = match change_type {
            ChangeType::Create => "Create",
            ChangeType::Modify => "Modify",
            ChangeType::Delete => "Delete",
            ChangeType::Rename { from: _ } => "Rename",
        };

        format!(
            "{} {} to implement requirement '{}' (step {} of implementation)",
            action,
            file_path.display(),
            requirements.title,
            implementation_index + 1
        )
    }

    /// Calculate code quality metrics
    fn calculate_quality_metrics(
        &self,
        changes: &ProposedChanges,
        strategy: &ImplementationStrategy,
    ) -> CodeQualityMetrics {
        CodeQualityMetrics {
            complexity_estimate: if changes.changes.len() <= 3 {
                "Low".to_string()
            } else if changes.changes.len() <= 6 {
                "Medium".to_string()
            } else {
                "High".to_string()
            },
            reuse_score: if strategy.architectural_approach.contains("existing") {
                0.8
            } else {
                0.4
            },
            maintainability: if changes.impact.risk_level == RiskLevel::Low {
                "Good".to_string()
            } else {
                "Moderate".to_string()
            },
            test_coverage_potential: 0.7, // Placeholder
            performance_impact: "Minimal".to_string(),
        }
    }
}

impl Subagent for CodeWriterSubagent {
    fn spec(&self) -> &SubagentSpec {
        &self.spec
    }

    fn spec_mut(&mut self) -> &mut SubagentSpec {
        &mut self.spec
    }
}

impl TypedSubagent for CodeWriterSubagent {
    type Request = CodeWriterRequest;
    type Response = CodeWriterResponse;

    fn run(
        &mut self,
        ctx: &mut TaskContext,
        request: Self::Request,
    ) -> SubagentResult<Self::Response> {
        ctx.info("Starting code generation analysis");

        // Analyze implementation strategy
        let strategy = self.analyze_implementation_strategy(
            &request.requirements,
            &request.repository_context,
            &request.preferences,
        )?;

        ctx.info(&format!(
            "Identified {} target files for implementation",
            strategy.target_files.len()
        ));

        // Generate code changes
        let changes = self.generate_code_changes(
            &request.requirements,
            &strategy,
            &request.repository_context,
            &request.preferences,
        )?;

        ctx.info(&format!(
            "Generated {} file changes with {} risk level",
            changes.changes.len(),
            match changes.impact.risk_level {
                RiskLevel::Low => "low",
                RiskLevel::Medium => "medium",
                RiskLevel::High => "high",
                RiskLevel::Critical => "critical",
            }
        ));

        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(&changes, &strategy);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&changes, &strategy, &request.preferences);

        Ok(CodeWriterResponse {
            changes,
            implementation_strategy: strategy.rationale,
            quality_metrics,
            recommendations,
        })
    }

    fn generate_recommendations(
        &self,
        changes: &ProposedChanges,
        strategy: &ImplementationStrategy,
        preferences: &CodeGenerationPreferences,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if changes.impact.risk_level != RiskLevel::Low {
            recommendations.push("Consider implementing changes incrementally to reduce risk".to_string());
        }

        if preferences.generate_tests && !changes.changes.iter().any(|c| c.file_path.to_string_lossy().contains("test")) {
            recommendations.push("Add test files to ensure proper coverage".to_string());
        }

        if changes.changes.len() > 5 {
            recommendations.push("Consider breaking this into smaller, focused change sets".to_string());
        }

        if !changes.impact.breaking_changes.is_empty() {
            recommendations.push("Review breaking changes carefully and update dependent code".to_string());
        }

        recommendations
    }
}

impl ContextualSubagent<RepositoryContext> for CodeWriterSubagent {
    fn prepare(&mut self, ctx: &mut TaskContext, context: &RepositoryContext) -> SubagentResult<()> {
        ctx.info("Preparing Code Writer with repository context");

        // Validate repository context
        if context.file_summaries.is_empty() {
            ctx.warning("No file summaries provided in repository context");
        }

        if context.languages.is_empty() {
            ctx.warning("No programming languages detected in repository context");
        }

        // Update analyzer with context
        self.analyzer.set_context(context.clone());

        ctx.info(&format!(
            "Prepared for {} languages with {} file summaries",
            context.languages.len(),
            context.file_summaries.len()
        ));

        Ok(())
    }
}

/// Internal implementation strategy
#[derive(Debug, Clone)]
struct ImplementationStrategy {
    pub architectural_approach: String,
    pub target_files: Vec<PathBuf>,
    pub implementation_order: Vec<PathBuf>,
    pub risk_assessment: RiskAssessment,
    pub rationale: String,
}

impl ImplementationStrategy {
    fn new() -> Self {
        Self {
            architectural_approach: String::new(),
            target_files: Vec::new(),
            implementation_order: Vec::new(),
            risk_assessment: RiskAssessment::new(),
            rationale: "Generated implementation strategy".to_string(),
        }
    }
}

/// Risk assessment for implementation
#[derive(Debug, Clone)]
struct RiskAssessment {
    pub file_count_risk: RiskLevel,
    pub architectural_risk: RiskLevel,
    pub overall_risk: RiskLevel,
}

impl RiskAssessment {
    fn new() -> Self {
        Self {
            file_count_risk: RiskLevel::Low,
            architectural_risk: RiskLevel::Low,
            overall_risk: RiskLevel::Low,
        }
    }
}

/// Code analysis utilities
#[derive(Debug, Clone)]
struct CodeAnalyzer {
    context: Option<RepositoryContext>,
}

impl CodeAnalyzer {
    fn new() -> Self {
        Self { context: None }
    }

    fn set_context(&mut self, context: RepositoryContext) {
        self.context = Some(context);
    }
}

/// Template engine for code generation
#[derive(Debug, Clone)]
struct CodeTemplateEngine {
    // Template configuration would go here
}

impl CodeTemplateEngine {
    fn new() -> Self {
        Self {}
    }

    fn generate_rust_content(&self, context: &TemplateContext) -> SubagentResult<String> {
        // This would generate Rust code based on the template context
        // For now, return a placeholder
        Ok(format!(
            "// Generated Rust code for {}\n// Requirements: {}\n\n// TODO: Implement based on requirements",
            context.file_path.display(),
            context.requirements.title
        ))
    }

    fn generate_typescript_content(&self, context: &TemplateContext) -> SubagentResult<String> {
        Ok(format!(
            "// Generated TypeScript code for {}\n// Requirements: {}\n\n// TODO: Implement based on requirements",
            context.file_path.display(),
            context.requirements.title
        ))
    }

    fn generate_python_content(&self, context: &TemplateContext) -> SubagentResult<String> {
        Ok(format!(
            "# Generated Python code for {}\n# Requirements: {}\n\n# TODO: Implement based on requirements",
            context.file_path.display(),
            context.requirements.title
        ))
    }

    fn generate_go_content(&self, context: &TemplateContext) -> SubagentResult<String> {
        Ok(format!(
            "// Generated Go code for {}\n// Requirements: {}\n\n// TODO: Implement based on requirements",
            context.file_path.display(),
            context.requirements.title
        ))
    }

    fn generate_generic_content(&self, context: &TemplateContext) -> SubagentResult<String> {
        Ok(format!(
            "Generated code for {}\nRequirements: {}\n\nTODO: Implement based on requirements",
            context.file_path.display(),
            context.requirements.title
        ))
    }
}

/// Template context for code generation
#[derive(Debug, Clone)]
struct TemplateContext {
    pub requirements: RequirementsSpec,
    pub file_path: PathBuf,
    pub architectural_approach: String,
    pub repository_context: RepositoryContext,
    pub preferences: CodeGenerationPreferences,
}

impl Default for CodeGenerationPreferences {
    fn default() -> Self {
        Self {
            prefer_existing_patterns: true,
            max_files_to_modify: Some(10),
            include_comments: true,
            generate_tests: true,
            risk_tolerance: RiskLevel::Medium,
        }
    }
}

impl Default for RepositoryContext {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            languages: vec!["rust".to_string()],
            patterns: vec!["modular".to_string()],
            file_summaries: HashMap::new(),
            dependencies: Vec::new(),
            style_config: None,
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
            .name("code-writer".to_string())
            .description("Test code writer".to_string())
            .instructions("Generate code".to_string())
            .build()
            .unwrap()
    }

    fn create_test_requirements() -> RequirementsSpec {
        let mut req = RequirementsSpec::new(
            "req-1".to_string(),
            "Test API Endpoint".to_string(),
            "Create a REST API endpoint for user management".to_string(),
        );

        req.add_criterion(AcceptanceCriterion {
            id: "ac-1".to_string(),
            description: "API must handle GET requests".to_string(),
            priority: Priority::High,
            testable: true,
            test_scenario: None,
        });

        req
    }

    #[test]
    fn test_code_writer_creation() {
        let spec = create_test_spec();
        let writer = CodeWriterSubagent::new(spec);
        assert_eq!(writer.name(), "code-writer");
    }

    #[test]
    fn test_file_priority_ordering() {
        let spec = create_test_spec();
        let writer = CodeWriterSubagent::new(spec);

        assert_eq!(writer.get_file_priority(&PathBuf::from("src/models/user.rs")), 1);
        assert_eq!(writer.get_file_priority(&PathBuf::from("src/api/routes.rs")), 3);
        assert_eq!(writer.get_file_priority(&PathBuf::from("tests/user_test.rs")), 5);
    }

    #[test]
    fn test_target_file_inference() {
        let spec = create_test_spec();
        let writer = CodeWriterSubagent::new(spec);

        let mut context = RepositoryContext::default();
        context.file_summaries.insert(PathBuf::from("src/api/routes.rs"), "API routes".to_string());
        context.file_summaries.insert(PathBuf::from("src/models/user.rs"), "User model".to_string());

        let requirements = create_test_requirements();
        let inferred = writer.infer_target_files(&requirements, &context).unwrap();

        assert!(inferred.iter().any(|p| p.to_string_lossy().contains("api")));
    }

    #[test]
    fn test_breaking_change_detection() {
        let spec = create_test_spec();
        let writer = CodeWriterSubagent::new(spec);

        let mut changes = ProposedChanges::new(
            "changes-1".to_string(),
            "req-1".to_string(),
            "Test changes".to_string(),
        );

        changes.add_change(FileChange {
            file_path: PathBuf::from("src/api/public.rs"),
            change_type: ChangeType::Modify,
            content: "new content".to_string(),
            line_range: None,
            reason: "test".to_string(),
        });

        let breaking_changes = writer.identify_breaking_changes(&changes);
        assert!(!breaking_changes.is_empty());
    }
}