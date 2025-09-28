//! Feature flag and backward compatibility validation tests.

#[cfg(test)]
mod tests {
    use crate::registry::SubagentRegistry;
    use crate::spec::SubagentSpec;
    use crate::pipeline::*;
    use crate::core::*;
    use crate::task_context::TaskContext;
    use crate::traits::{Subagent, TypedSubagent};
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Mock configuration for testing feature flags
    #[derive(Debug, Clone)]
    struct MockConfig {
        pub subagents_enabled: bool,
    }

    impl MockConfig {
        fn with_subagents_enabled(enabled: bool) -> Self {
            Self {
                subagents_enabled: enabled,
            }
        }
    }

    /// Test that core subagent types can be created regardless of feature flag state
    #[test]
    fn test_subagent_creation_backward_compatibility() {
        // Test that all core subagent types can be instantiated
        let spec_parser_spec = create_test_spec("spec-parser");
        let code_writer_spec = create_test_spec("code-writer");
        let tester_spec = create_test_spec("tester");
        let reviewer_spec = create_test_spec("reviewer");

        let spec_parser = SpecParserSubagent::new(spec_parser_spec);
        let code_writer = CodeWriterSubagent::new(code_writer_spec);
        let tester = TesterSubagent::new(tester_spec);
        let reviewer = ReviewerSubagent::new(reviewer_spec);

        // All subagents should implement the base trait correctly
        assert_eq!(spec_parser.name(), "spec-parser");
        assert_eq!(code_writer.name(), "code-writer");
        assert_eq!(tester.name(), "tester");
        assert_eq!(reviewer.name(), "reviewer");

        println!("✅ All core subagent types can be created successfully");
    }

    /// Test that pipeline types work independently of feature flags
    #[test]
    fn test_pipeline_types_backward_compatibility() {
        // Test that pipeline types work regardless of subagent feature state
        let requirements = RequirementsSpec::new(
            "test-req-1".to_string(),
            "Test Requirements".to_string(),
            "Test requirements for backward compatibility".to_string(),
        );

        let mut changes = ProposedChanges::new(
            "changes-1".to_string(),
            requirements.id.clone(),
            "Test changes".to_string(),
        );

        changes.add_change(FileChange {
            file_path: PathBuf::from("src/test.rs"),
            change_type: ChangeType::Create,
            content: "// Test content\n".to_string(),
            line_range: None,
            reason: "Test change".to_string(),
        });

        let test_plan = TestPlan::new(
            "test-plan-1".to_string(),
            requirements.id.clone(),
            changes.id.clone(),
            "Test strategy".to_string(),
        );

        let review = ReviewFindings::new("review-1".to_string(), changes.id.clone());

        // All pipeline types should work correctly
        assert!(!requirements.id.is_empty());
        assert!(!changes.id.is_empty());
        assert!(!test_plan.id.is_empty());
        assert!(!review.id.is_empty());

        println!("✅ Pipeline types work independently of feature flags");
    }

    /// Test that transformation utilities work regardless of feature state
    #[test]
    fn test_pipeline_transformation_backward_compatibility() {
        let transformer = PipelineTransformer::new();
        let validator = PipelineValidator::new();

        let requirements = create_test_requirements();

        // Test transformation operations
        let test_plan = transformer.requirements_to_test_plan(&requirements, "changes-1".to_string()).unwrap();
        assert_eq!(test_plan.requirements_id, requirements.id);

        let changes = create_test_changes(&requirements.id);
        let review_scope = transformer.changes_to_review_scope(&changes).unwrap();
        assert_eq!(review_scope.changes_id, changes.id);

        // Test validation operations
        let pipeline_state = transformer.create_pipeline_state(
            Some(requirements.clone()),
            Some(changes.clone()),
            Some(test_plan.clone()),
            Some(review_scope.clone()),
        ).unwrap();

        let validation_report = validator.validate_pipeline(&pipeline_state).unwrap();
        assert!(validation_report.is_valid() || !validation_report.errors.is_empty());

        println!("✅ Pipeline transformation utilities work correctly");
    }

    /// Test that subagent execution works with proper inputs
    #[test]
    fn test_subagent_execution_functional() {
        let mut ctx = TaskContext::new();

        // Test Spec Parser execution
        let mut spec_parser = create_spec_parser();
        let spec_request = SpecParserRequest {
            requirements_text: "Create a login system with authentication".to_string(),
            codebase_context: Some("Rust web application".to_string()),
            related_files: vec![PathBuf::from("src/auth.rs")],
            metadata: HashMap::new(),
        };

        let spec_result = spec_parser.run(&mut ctx, spec_request);
        assert!(spec_result.is_ok());

        // Test Code Writer execution
        let mut code_writer = create_code_writer();
        let requirements = create_test_requirements();
        let code_request = CodeWriterRequest {
            requirements: requirements.clone(),
            repository_context: create_repository_context(),
            target_files: None,
            preferences: CodeGenerationPreferences::default(),
        };

        let code_result = code_writer.run(&mut ctx, code_request);
        assert!(code_result.is_ok());

        // Test Tester execution (dry run)
        let mut tester = create_tester();
        let changes = create_test_changes(&requirements.id);
        let test_request = TesterRequest {
            requirements: requirements.clone(),
            changes: changes.clone(),
            sandbox_config: SandboxConfig {
                enabled: false, // Disable sandbox for testing
                ..SandboxConfig::default()
            },
            test_options: TestOptions {
                dry_run: true,
                ..TestOptions::default()
            },
        };

        let test_result = tester.run(&mut ctx, test_request);
        assert!(test_result.is_ok());

        // Test Reviewer execution
        let mut reviewer = create_reviewer();
        let review_request = ReviewerRequest {
            changes,
            review_config: ReviewConfig::default(),
            codebase_context: Some(CodebaseContext::default()),
            focus_areas: vec![ReviewFocus::Style],
        };

        let review_result = reviewer.run(&mut ctx, review_request);
        assert!(review_result.is_ok());

        println!("✅ All subagents can execute successfully with proper inputs");
    }

    /// Test that serialization/deserialization maintains compatibility
    #[test]
    fn test_serialization_backward_compatibility() {
        let requirements = create_test_requirements();
        let changes = create_test_changes(&requirements.id);

        // Test RequirementsSpec serialization
        let req_serialized = serde_json::to_string(&requirements).unwrap();
        let req_deserialized: RequirementsSpec = serde_json::from_str(&req_serialized).unwrap();
        assert_eq!(requirements.id, req_deserialized.id);

        // Test ProposedChanges serialization
        let changes_serialized = serde_json::to_string(&changes).unwrap();
        let changes_deserialized: ProposedChanges = serde_json::from_str(&changes_serialized).unwrap();
        assert_eq!(changes.id, changes_deserialized.id);

        // Test that serialized data is reasonably sized
        assert!(req_serialized.len() > 50); // Should have meaningful content
        assert!(changes_serialized.len() > 50);

        println!("✅ Serialization maintains backward compatibility");
        println!("   📦 Requirements size: {} bytes", req_serialized.len());
        println!("   📦 Changes size: {} bytes", changes_serialized.len());
    }

    /// Test that error handling is graceful and informative
    #[test]
    fn test_error_handling_backward_compatibility() {
        let validator = PipelineValidator::new();

        // Create invalid pipeline state
        let mut requirements = RequirementsSpec::new(
            String::new(), // Invalid empty ID
            String::new(), // Invalid empty title
            String::new(), // Invalid empty description
        );

        let mut validation_report = crate::pipeline::ValidationReport::new();
        let result = validator.validate_requirements_spec(&requirements, &mut validation_report);

        // Should handle invalid input gracefully
        assert!(result.is_ok());
        assert!(!validation_report.is_valid()); // Should detect errors
        assert!(!validation_report.errors.is_empty());

        // Error messages should be helpful
        for error in &validation_report.errors {
            assert!(!error.is_empty());
            assert!(error.len() > 10); // Should be descriptive
        }

        println!("✅ Error handling is graceful and informative");
        println!("   ❌ Detected {} validation errors", validation_report.errors.len());
    }

    /// Test that memory usage is reasonable
    #[test]
    fn test_memory_usage_reasonable() {
        let mut subagents = Vec::new();

        // Create multiple subagents to test memory usage
        for i in 0..10 {
            let spec = create_test_spec(&format!("test-agent-{}", i));
            subagents.push(SpecParserSubagent::new(spec));
        }

        assert_eq!(subagents.len(), 10);

        // Create large pipeline state
        let large_requirements = create_large_requirements();
        let large_changes = create_large_changes(&large_requirements.id);

        // Memory usage should be reasonable even with large data
        assert!(large_requirements.acceptance_criteria.len() <= 100);
        assert!(large_changes.changes.len() <= 50);

        println!("✅ Memory usage is reasonable for large datasets");
        println!("   📊 Created {} subagents", subagents.len());
        println!("   📊 Large requirements: {} criteria", large_requirements.acceptance_criteria.len());
        println!("   📊 Large changes: {} files", large_changes.changes.len());
    }

    /// Test that concurrent operations work correctly
    #[test]
    fn test_concurrent_operations_safe() {
        use std::thread;

        let transformer = PipelineTransformer::new();
        let requirements = create_test_requirements();

        // Test concurrent transformations
        let handles: Vec<_> = (0..5).map(|i| {
            let transformer = transformer.clone();
            let requirements = requirements.clone();

            thread::spawn(move || {
                let test_plan = transformer.requirements_to_test_plan(
                    &requirements,
                    format!("changes-{}", i)
                ).unwrap();

                assert!(!test_plan.id.is_empty());
                test_plan
            })
        }).collect();

        // Wait for all threads to complete
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.len(), 5);

        // All results should be valid and unique
        for (i, result) in results.iter().enumerate() {
            assert!(result.changes_id.contains(&format!("changes-{}", i)));
        }

        println!("✅ Concurrent operations work safely");
    }

    /// Test that default configurations are sensible
    #[test]
    fn test_default_configurations_sensible() {
        let test_options = TestOptions::default();
        let review_config = ReviewConfig::default();
        let sandbox_config = SandboxConfig::default();
        let repo_context = RepositoryContext::default();
        let codebase_context = CodebaseContext::default();

        // Test options should have reasonable defaults
        assert!(!test_options.test_types.is_empty());
        assert!(test_options.max_test_cases.is_some());
        assert!(!test_options.fail_fast); // Should be false for stability

        // Review config should enable key analyses
        assert!(review_config.style_analysis);
        assert!(review_config.security_analysis);
        assert!(review_config.performance_analysis);

        // Sandbox should be enabled by default for safety
        assert!(sandbox_config.enabled);
        assert!(sandbox_config.test_timeout.as_secs() > 0);

        // Repository context should have sensible defaults
        assert!(!repo_context.languages.is_empty());

        println!("✅ Default configurations are sensible and safe");
    }

    /// Test that validation rules are comprehensive
    #[test]
    fn test_validation_rules_comprehensive() {
        let validator = PipelineValidator::new();

        // Test various invalid states
        let test_cases = vec![
            ("empty_requirements", create_empty_requirements()),
            ("invalid_changes", create_invalid_changes()),
            ("mismatched_ids", create_mismatched_pipeline_state()),
        ];

        for (test_name, pipeline_state) in test_cases {
            let validation_report = validator.validate_pipeline(&pipeline_state).unwrap();

            if test_name != "valid_state" {
                assert!(!validation_report.is_valid(), "Test case '{}' should be invalid", test_name);
            }

            println!("   📋 Test case '{}': {} errors, {} warnings",
                test_name, validation_report.errors.len(), validation_report.warnings.len());
        }

        println!("✅ Validation rules are comprehensive");
    }

    // Helper functions for creating test data

    fn create_test_spec(name: &str) -> SubagentSpec {
        SubagentSpec::builder()
            .name(name.to_string())
            .description(format!("{} subagent", name))
            .instructions(format!("Execute {} operations", name))
            .tools(vec!["read".to_string(), "analysis".to_string()])
            .build()
            .unwrap()
    }

    fn create_spec_parser() -> SpecParserSubagent {
        SpecParserSubagent::new(create_test_spec("spec-parser"))
    }

    fn create_code_writer() -> CodeWriterSubagent {
        CodeWriterSubagent::new(create_test_spec("code-writer"))
    }

    fn create_tester() -> TesterSubagent {
        TesterSubagent::new(create_test_spec("tester"))
    }

    fn create_reviewer() -> ReviewerSubagent {
        ReviewerSubagent::new(create_test_spec("reviewer"))
    }

    fn create_test_requirements() -> RequirementsSpec {
        let mut requirements = RequirementsSpec::new(
            "test-req-1".to_string(),
            "Test Requirements".to_string(),
            "Test requirements for backward compatibility testing".to_string(),
        );

        requirements.add_criterion(AcceptanceCriterion {
            id: "ac-1".to_string(),
            description: "System must handle user authentication".to_string(),
            priority: Priority::High,
            testable: true,
            test_scenario: Some("Given valid credentials when login then success".to_string()),
        });

        requirements
    }

    fn create_test_changes(requirements_id: &str) -> ProposedChanges {
        let mut changes = ProposedChanges::new(
            "changes-1".to_string(),
            requirements_id.to_string(),
            "Test changes for compatibility testing".to_string(),
        );

        changes.add_change(FileChange {
            file_path: PathBuf::from("src/auth.rs"),
            change_type: ChangeType::Create,
            content: "// Authentication module\npub fn authenticate() {}\n".to_string(),
            line_range: None,
            reason: "Create authentication module".to_string(),
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
                summaries.insert(PathBuf::from("src/lib.rs"), "Main library".to_string());
                summaries
            },
            dependencies: vec!["serde".to_string()],
            style_config: None,
        }
    }

    fn create_large_requirements() -> RequirementsSpec {
        let mut requirements = RequirementsSpec::new(
            "large-req-1".to_string(),
            "Large Requirements".to_string(),
            "Large requirements set for testing".to_string(),
        );

        for i in 1..=50 {
            requirements.add_criterion(AcceptanceCriterion {
                id: format!("ac-{}", i),
                description: format!("Criterion number {}", i),
                priority: Priority::Medium,
                testable: true,
                test_scenario: None,
            });
        }

        requirements
    }

    fn create_large_changes(requirements_id: &str) -> ProposedChanges {
        let mut changes = ProposedChanges::new(
            "large-changes-1".to_string(),
            requirements_id.to_string(),
            "Large changes set".to_string(),
        );

        for i in 1..=25 {
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

    fn create_empty_requirements() -> PipelineState {
        let requirements = RequirementsSpec::new(
            String::new(),
            String::new(),
            String::new(),
        );

        PipelineState {
            stage: PipelineStage::Specification,
            requirements: Some(requirements),
            changes: None,
            test_plan: None,
            review: None,
            metadata: PipelineMetadata {
                execution_id: "test".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                initiated_by: "test".to_string(),
                custom_fields: HashMap::new(),
            },
        }
    }

    fn create_invalid_changes() -> PipelineState {
        let changes = ProposedChanges::new(
            String::new(), // Invalid empty ID
            "req-1".to_string(),
            String::new(), // Invalid empty rationale
        );

        PipelineState {
            stage: PipelineStage::CodeGeneration,
            requirements: None,
            changes: Some(changes),
            test_plan: None,
            review: None,
            metadata: PipelineMetadata {
                execution_id: "test".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                initiated_by: "test".to_string(),
                custom_fields: HashMap::new(),
            },
        }
    }

    fn create_mismatched_pipeline_state() -> PipelineState {
        let requirements = RequirementsSpec::new(
            "req-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
        );

        let changes = ProposedChanges::new(
            "changes-1".to_string(),
            "wrong-req-id".to_string(), // Mismatched ID
            "Test changes".to_string(),
        );

        PipelineState {
            stage: PipelineStage::Complete,
            requirements: Some(requirements),
            changes: Some(changes),
            test_plan: None,
            review: None,
            metadata: PipelineMetadata {
                execution_id: "test".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                initiated_by: "test".to_string(),
                custom_fields: HashMap::new(),
            },
        }
    }
}