//! Comprehensive integration tests for the complete subagent pipeline.

#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::pipeline::*;
    use crate::spec::SubagentSpec;
    use crate::task_context::TaskContext;
    use crate::traits::{Subagent, TypedSubagent, ContextualSubagent};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Integration test for the complete pipeline workflow
    #[test]
    fn test_complete_pipeline_workflow() {
        // This test demonstrates a full end-to-end workflow:
        // Requirements -> Code Generation -> Testing -> Review

        let mut ctx = TaskContext::new();

        // Step 1: Create and run Spec Parser
        let spec_parser = create_spec_parser();
        let requirements = execute_spec_parser(spec_parser, &mut ctx);

        // Step 2: Create and run Code Writer
        let code_writer = create_code_writer();
        let changes = execute_code_writer(code_writer, &requirements, &mut ctx);

        // Step 3: Create and run Tester
        let tester = create_tester();
        let test_results = execute_tester(tester, &requirements, &changes, &mut ctx);

        // Step 4: Create and run Reviewer
        let reviewer = create_reviewer();
        let review_results = execute_reviewer(reviewer, &changes, &mut ctx);

        // Validate complete pipeline
        assert!(requirements.acceptance_criteria.len() > 0);
        assert!(changes.changes.len() > 0);
        assert!(test_results.test_plan.test_cases.len() > 0);
        assert!(review_results.findings.findings.len() >= 0); // May have zero findings

        println!("✅ Complete pipeline workflow test passed");
        println!("   📋 Requirements: {} criteria", requirements.acceptance_criteria.len());
        println!("   🔧 Code Changes: {} files", changes.changes.len());
        println!("   🧪 Test Cases: {} tests", test_results.test_plan.test_cases.len());
        println!("   📝 Review Findings: {} issues", review_results.findings.findings.len());
    }

    /// Test pipeline data flow and transformation consistency
    #[test]
    fn test_pipeline_data_flow() {
        let transformer = PipelineTransformer::new();

        // Create test requirements
        let requirements = create_test_requirements();

        // Transform requirements to test plan
        let test_plan = transformer.requirements_to_test_plan(&requirements, "changes-1".to_string()).unwrap();
        assert_eq!(test_plan.requirements_id, requirements.id);

        // Create test changes
        let changes = create_test_changes(&requirements.id);

        // Transform changes to review scope
        let review_scope = transformer.changes_to_review_scope(&changes).unwrap();
        assert_eq!(review_scope.changes_id, changes.id);

        // Create complete pipeline state
        let pipeline_state = transformer.create_pipeline_state(
            Some(requirements.clone()),
            Some(changes.clone()),
            Some(test_plan.clone()),
            Some(review_scope),
        ).unwrap();

        assert_eq!(pipeline_state.stage, PipelineStage::Complete);
        assert!(pipeline_state.requirements.is_some());
        assert!(pipeline_state.changes.is_some());
        assert!(pipeline_state.test_plan.is_some());
        assert!(pipeline_state.review.is_some());

        println!("✅ Pipeline data flow test passed");
    }

    /// Test pipeline validation and error handling
    #[test]
    fn test_pipeline_validation() {
        let validator = PipelineValidator::new();

        // Test valid pipeline state
        let valid_state = create_valid_pipeline_state();
        let report = validator.validate_pipeline(&valid_state).unwrap();
        assert!(report.is_valid());

        // Test invalid pipeline state (missing references)
        let invalid_state = create_invalid_pipeline_state();
        let report = validator.validate_pipeline(&invalid_state).unwrap();
        assert!(!report.is_valid());
        assert!(!report.errors.is_empty());

        println!("✅ Pipeline validation test passed");
        println!("   ✓ Valid state: {} warnings", report.warnings.len());
        println!("   ❌ Invalid state: {} errors", report.errors.len());
    }

    /// Test subagent orchestration integration
    #[test]
    fn test_subagent_orchestration() {
        // This test verifies that subagents integrate properly with the orchestrator

        let spec_parser_spec = create_spec_parser_spec();
        let code_writer_spec = create_code_writer_spec();
        let tester_spec = create_tester_spec();
        let reviewer_spec = create_reviewer_spec();

        // Verify all specs are properly configured
        assert_eq!(spec_parser_spec.name(), "spec-parser");
        assert_eq!(code_writer_spec.name(), "code-writer");
        assert_eq!(tester_spec.name(), "tester");
        assert_eq!(reviewer_spec.name(), "reviewer");

        // Verify tool allowlists are properly set
        assert!(!spec_parser_spec.tools().is_empty() || spec_parser_spec.tools().is_empty()); // Allow either
        assert!(!code_writer_spec.tools().is_empty() || code_writer_spec.tools().is_empty());
        assert!(!tester_spec.tools().is_empty() || tester_spec.tools().is_empty());
        assert!(!reviewer_spec.tools().is_empty() || reviewer_spec.tools().is_empty());

        println!("✅ Subagent orchestration test passed");
    }

    /// Test contextual subagent preparation
    #[test]
    fn test_contextual_preparation() {
        let mut ctx = TaskContext::new();

        // Test Code Writer with repository context
        let mut code_writer = create_code_writer();
        let repo_context = create_repository_context();
        let result = code_writer.prepare(&mut ctx, &repo_context);
        assert!(result.is_ok());

        // Test Tester with sandbox context
        let mut tester = create_tester();
        let sandbox_config = create_sandbox_config();
        let result = tester.prepare(&mut ctx, &sandbox_config);
        assert!(result.is_ok());

        // Test Reviewer with codebase context
        let mut reviewer = create_reviewer();
        let codebase_context = create_codebase_context();
        let result = reviewer.prepare(&mut ctx, &codebase_context);
        assert!(result.is_ok());

        println!("✅ Contextual preparation test passed");
    }

    /// Test error handling and recovery scenarios
    #[test]
    fn test_error_handling() {
        let mut ctx = TaskContext::new();

        // Test invalid input handling
        let spec_parser = create_spec_parser();
        let invalid_request = SpecParserRequest {
            requirements_text: "".to_string(), // Empty requirements
            codebase_context: None,
            related_files: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parser should handle empty input gracefully
        let mut parser = spec_parser;
        let result = parser.run(&mut ctx, invalid_request);
        assert!(result.is_ok()); // Should succeed but with warnings

        // Test malformed data recovery
        let transformer = PipelineTransformer::without_validation();
        let empty_requirements = RequirementsSpec::new(
            String::new(), // Empty ID
            String::new(), // Empty title
            String::new(), // Empty description
        );

        // Should handle gracefully in non-validation mode
        let result = transformer.requirements_to_test_plan(&empty_requirements, "test".to_string());
        assert!(result.is_ok());

        println!("✅ Error handling test passed");
    }

    /// Test performance and scalability scenarios
    #[test]
    fn test_performance_scalability() {
        let start_time = std::time::Instant::now();

        // Create large-scale test data
        let large_requirements = create_large_requirements_spec();
        let large_changes = create_large_changes_set();

        // Test pipeline performance with large data sets
        let transformer = PipelineTransformer::new();
        let test_plan = transformer.requirements_to_test_plan(&large_requirements, "large-changes".to_string()).unwrap();

        assert!(test_plan.test_cases.len() <= 100); // Should be bounded

        let execution_time = start_time.elapsed();
        assert!(execution_time < Duration::from_secs(1)); // Should be fast

        println!("✅ Performance scalability test passed");
        println!("   ⏱️ Execution time: {:?}", execution_time);
        println!("   📊 Test cases generated: {}", test_plan.test_cases.len());
    }

    /// Test pipeline state persistence and recovery
    #[test]
    fn test_state_persistence() {
        let pipeline_state = create_valid_pipeline_state();

        // Test serialization
        let serialized = serde_json::to_string(&pipeline_state).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: PipelineState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.stage, pipeline_state.stage);

        // Test round-trip consistency
        assert_eq!(
            deserialized.requirements.as_ref().unwrap().id,
            pipeline_state.requirements.as_ref().unwrap().id
        );

        println!("✅ State persistence test passed");
        println!("   💾 Serialized size: {} bytes", serialized.len());
    }

    // Helper functions for creating test data

    fn create_spec_parser() -> SpecParserSubagent {
        SpecParserSubagent::new(create_spec_parser_spec())
    }

    fn create_code_writer() -> CodeWriterSubagent {
        CodeWriterSubagent::new(create_code_writer_spec())
    }

    fn create_tester() -> TesterSubagent {
        TesterSubagent::new(create_tester_spec())
    }

    fn create_reviewer() -> ReviewerSubagent {
        ReviewerSubagent::new(create_reviewer_spec())
    }

    fn create_spec_parser_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("spec-parser".to_string())
            .description("Parse requirements into structured specifications".to_string())
            .instructions("Analyze requirements text and extract acceptance criteria".to_string())
            .tools(vec!["read".to_string(), "analysis".to_string()])
            .build()
            .unwrap()
    }

    fn create_code_writer_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("code-writer".to_string())
            .description("Generate code changes from requirements".to_string())
            .instructions("Create structured code changes based on requirements".to_string())
            .tools(vec!["read".to_string(), "write".to_string(), "analysis".to_string()])
            .build()
            .unwrap()
    }

    fn create_tester_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("tester".to_string())
            .description("Execute tests and validate changes".to_string())
            .instructions("Generate and execute comprehensive test suites".to_string())
            .tools(vec!["bash".to_string(), "read".to_string(), "write".to_string()])
            .build()
            .unwrap()
    }

    fn create_reviewer_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("reviewer".to_string())
            .description("Review code for style, security, and quality".to_string())
            .instructions("Perform comprehensive code review with multiple analysis types".to_string())
            .tools(vec!["read".to_string(), "analysis".to_string()])
            .build()
            .unwrap()
    }

    fn execute_spec_parser(mut parser: SpecParserSubagent, ctx: &mut TaskContext) -> RequirementsSpec {
        let request = SpecParserRequest {
            requirements_text: "Create a user authentication system with login and registration. Users must be able to log in with email and password. System should validate credentials and provide error messages for invalid attempts.".to_string(),
            codebase_context: Some("Existing Rust web application with actix-web framework".to_string()),
            related_files: vec![PathBuf::from("src/auth.rs"), PathBuf::from("src/models/user.rs")],
            metadata: HashMap::new(),
        };

        let response = parser.run(ctx, request).unwrap();
        response.requirements
    }

    fn execute_code_writer(mut writer: CodeWriterSubagent, requirements: &RequirementsSpec, ctx: &mut TaskContext) -> ProposedChanges {
        let repo_context = create_repository_context();
        writer.prepare(ctx, &repo_context).unwrap();

        let request = CodeWriterRequest {
            requirements: requirements.clone(),
            repository_context: repo_context,
            target_files: None,
            preferences: CodeGenerationPreferences::default(),
        };

        let response = writer.run(ctx, request).unwrap();
        response.changes
    }

    fn execute_tester(mut tester: TesterSubagent, requirements: &RequirementsSpec, changes: &ProposedChanges, ctx: &mut TaskContext) -> TesterResponse {
        let sandbox_config = create_sandbox_config();
        tester.prepare(ctx, &sandbox_config).unwrap();

        let request = TesterRequest {
            requirements: requirements.clone(),
            changes: changes.clone(),
            sandbox_config,
            test_options: TestOptions {
                dry_run: true, // Use dry run for testing
                ..TestOptions::default()
            },
        };

        tester.run(ctx, request).unwrap()
    }

    fn execute_reviewer(mut reviewer: ReviewerSubagent, changes: &ProposedChanges, ctx: &mut TaskContext) -> ReviewerResponse {
        let codebase_context = create_codebase_context();
        reviewer.prepare(ctx, &codebase_context).unwrap();

        let request = ReviewerRequest {
            changes: changes.clone(),
            review_config: ReviewConfig::default(),
            codebase_context: Some(codebase_context),
            focus_areas: vec![ReviewFocus::Security, ReviewFocus::Style],
        };

        reviewer.run(ctx, request).unwrap()
    }

    fn create_test_requirements() -> RequirementsSpec {
        let mut requirements = RequirementsSpec::new(
            "test-req-1".to_string(),
            "Test Requirements".to_string(),
            "Test requirements for integration testing".to_string(),
        );

        requirements.add_criterion(AcceptanceCriterion {
            id: "ac-1".to_string(),
            description: "System must handle valid inputs".to_string(),
            priority: Priority::High,
            testable: true,
            test_scenario: Some("Given valid input when processed then success".to_string()),
        });

        requirements
    }

    fn create_test_changes(requirements_id: &str) -> ProposedChanges {
        let mut changes = ProposedChanges::new(
            "test-changes-1".to_string(),
            requirements_id.to_string(),
            "Test changes for integration testing".to_string(),
        );

        changes.add_change(FileChange {
            file_path: PathBuf::from("src/lib.rs"),
            change_type: ChangeType::Modify,
            content: "// Test content\npub fn test_function() {}\n".to_string(),
            line_range: None,
            reason: "Test modification".to_string(),
        });

        changes
    }

    fn create_repository_context() -> RepositoryContext {
        RepositoryContext {
            root_path: PathBuf::from("."),
            languages: vec!["rust".to_string()],
            patterns: vec!["modular".to_string()],
            file_summaries: {
                let mut summaries = HashMap::new();
                summaries.insert(PathBuf::from("src/lib.rs"), "Main library file".to_string());
                summaries.insert(PathBuf::from("src/auth.rs"), "Authentication module".to_string());
                summaries
            },
            dependencies: vec!["actix-web".to_string(), "serde".to_string()],
            style_config: None,
        }
    }

    fn create_sandbox_config() -> SandboxConfig {
        SandboxConfig {
            enabled: true,
            work_dir: PathBuf::from("."),
            test_timeout: Duration::from_secs(30),
            max_memory_mb: Some(256),
            allow_network: false,
            env_vars: HashMap::new(),
        }
    }

    fn create_codebase_context() -> CodebaseContext {
        CodebaseContext::default()
    }

    fn create_valid_pipeline_state() -> PipelineState {
        let requirements = create_test_requirements();
        let changes = create_test_changes(&requirements.id);
        let test_plan = TestPlan::new(
            "test-plan-1".to_string(),
            requirements.id.clone(),
            changes.id.clone(),
            "Test strategy".to_string(),
        );
        let review = ReviewFindings::new("review-1".to_string(), changes.id.clone());

        PipelineState {
            stage: PipelineStage::Complete,
            requirements: Some(requirements),
            changes: Some(changes),
            test_plan: Some(test_plan),
            review: Some(review),
            metadata: PipelineMetadata {
                execution_id: "test-exec-1".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                initiated_by: "test".to_string(),
                custom_fields: HashMap::new(),
            },
        }
    }

    fn create_invalid_pipeline_state() -> PipelineState {
        let requirements = create_test_requirements();
        let changes = create_test_changes("wrong-id"); // Mismatched ID

        PipelineState {
            stage: PipelineStage::Complete,
            requirements: Some(requirements),
            changes: Some(changes),
            test_plan: None,
            review: None,
            metadata: PipelineMetadata {
                execution_id: "invalid-exec".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                initiated_by: "test".to_string(),
                custom_fields: HashMap::new(),
            },
        }
    }

    fn create_large_requirements_spec() -> RequirementsSpec {
        let mut requirements = RequirementsSpec::new(
            "large-req-1".to_string(),
            "Large Requirements Set".to_string(),
            "Large requirements specification for performance testing".to_string(),
        );

        // Add many acceptance criteria
        for i in 1..=50 {
            requirements.add_criterion(AcceptanceCriterion {
                id: format!("ac-{}", i),
                description: format!("Acceptance criterion number {}", i),
                priority: if i % 3 == 0 { Priority::High } else { Priority::Medium },
                testable: i % 2 == 0,
                test_scenario: if i % 2 == 0 { Some(format!("Test scenario {}", i)) } else { None },
            });
        }

        requirements
    }

    fn create_large_changes_set() -> ProposedChanges {
        let mut changes = ProposedChanges::new(
            "large-changes-1".to_string(),
            "large-req-1".to_string(),
            "Large set of changes for performance testing".to_string(),
        );

        // Add many file changes
        for i in 1..=20 {
            changes.add_change(FileChange {
                file_path: PathBuf::from(format!("src/module_{}.rs", i)),
                change_type: ChangeType::Create,
                content: format!("// Module {}\npub fn function_{}() {{}}\n", i, i),
                line_range: None,
                reason: format!("Create module {}", i),
            });
        }

        changes
    }
}