//! Comprehensive unit tests for parser precedence/caching, registry reload, and edge cases.

use codex_subagents::parser::parse_document;
use codex_subagents::registry::{SubagentRegistry, AgentSource, ReloadReport};
use codex_subagents::error::{SubagentError, SubagentResult};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use tempfile::TempDir;
use pretty_assertions::assert_eq;

/// Test comprehensive parser edge cases and validation
#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parser_handles_unicode_bom() {
        let doc_with_bom = "\u{feff}---\nname: test-agent\n---\nInstructions";
        let spec = parse_document(doc_with_bom, None).unwrap();
        assert_eq!(spec.name(), "test-agent");
        assert_eq!(spec.instructions(), "Instructions");
    }

    #[test]
    fn test_parser_handles_windows_line_endings() {
        let doc_with_crlf = "---\r\nname: test-agent\r\ndescription: Test\r\n---\r\nInstructions\r\n";
        let spec = parse_document(doc_with_crlf, None).unwrap();
        assert_eq!(spec.name(), "test-agent");
        assert_eq!(spec.description(), Some("Test"));
        assert_eq!(spec.instructions(), "Instructions");
    }

    #[test]
    fn test_parser_cleans_duplicate_tools() {
        let doc = r#"---
name: test-agent
tools:
  - read
  - write
  - read
  - ""
  - "  write  "
---
Instructions"#;
        let spec = parse_document(doc, None).unwrap();
        assert_eq!(spec.tools(), &["read".to_string(), "write".to_string()]);
    }

    #[test]
    fn test_parser_cleans_duplicate_keywords() {
        let doc = r#"---
name: test-agent
keywords:
  - rust
  - review
  - rust
  - ""
  - "  review  "
---
Instructions"#;
        let spec = parse_document(doc, None).unwrap();
        assert_eq!(spec.keywords(), &["rust".to_string(), "review".to_string()]);
    }

    #[test]
    fn test_parser_validates_required_name() {
        let doc_missing_name = "---\ndescription: Test\n---\nInstructions";
        let result = parse_document(doc_missing_name, None);
        assert!(result.is_err());

        let doc_empty_name = "---\nname: \"\"\ndescription: Test\n---\nInstructions";
        let result = parse_document(doc_empty_name, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_handles_malformed_yaml() {
        let malformed_yaml = "---\nname: test\ninvalid: yaml: structure\n---\nInstructions";
        let result = parse_document(malformed_yaml, None);
        assert!(matches!(result, Err(SubagentError::FrontMatter { .. })));
    }

    #[test]
    fn test_parser_handles_missing_closing_delimiter() {
        let unclosed_frontmatter = "---\nname: test\ndescription: Test\nInstructions without closing";
        let result = parse_document(unclosed_frontmatter, None);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("closing")));
    }

    #[test]
    fn test_parser_preserves_instruction_formatting() {
        let doc = r#"---
name: test-agent
---

# Main Instructions

This is **markdown** with formatting:

1. Step one
2. Step two

```rust
fn example() {}
```

Final notes."#;
        let spec = parse_document(doc, None).unwrap();
        let instructions = spec.instructions();
        assert!(instructions.contains("# Main Instructions"));
        assert!(instructions.contains("**markdown**"));
        assert!(instructions.contains("fn example() {}"));
        assert!(!instructions.starts_with('\n')); // Should trim leading newline
    }

    #[test]
    fn test_parser_source_path_attribution() {
        let doc = "---\nname: test-agent\n---\nInstructions";
        let source_path = PathBuf::from("/project/.codex/agents/test-agent.md");
        let spec = parse_document(doc, Some(source_path.clone())).unwrap();
        assert_eq!(spec.source_path(), Some(&source_path));
    }

    #[test]
    fn test_parser_error_includes_path_context() {
        let malformed_yaml = "---\nname: test\ninvalid: yaml: structure\n---\nInstructions";
        let source_path = PathBuf::from("/project/.codex/agents/broken.md");
        let result = parse_document(malformed_yaml, Some(source_path.clone()));

        match result {
            Err(SubagentError::FrontMatter { path, .. }) => {
                assert_eq!(path, source_path);
            }
            _ => panic!("Expected FrontMatter error with path context"),
        }
    }
}

/// Test registry caching, precedence, and reload behavior
#[cfg(test)]
mod registry_tests {
    use super::*;

    fn write_agent_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{}.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    fn create_valid_agent_content(name: &str, description: &str, instructions: &str) -> String {
        format!(
            "---\nname: {}\ndescription: {}\n---\n{}",
            name, description, instructions
        )
    }

    #[test]
    fn test_registry_precedence_project_over_user() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create same-named agent in both directories
        write_agent_file(
            &user_agents,
            "duplicate-agent",
            &create_valid_agent_content("duplicate-agent", "user version", "User instructions")
        );
        write_agent_file(
            &project_agents,
            "duplicate-agent",
            &create_valid_agent_content("duplicate-agent", "project version", "Project instructions")
        );

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        assert!(report.errors.is_empty());
        assert_eq!(report.loaded.len(), 1);
        assert_eq!(report.loaded[0], "duplicate-agent");

        let record = registry.get("duplicate-agent").unwrap();
        assert_eq!(record.source, AgentSource::Project);
        assert_eq!(record.spec.description(), Some("project version"));
        assert_eq!(record.spec.instructions(), "Project instructions");
    }

    #[test]
    fn test_registry_caching_by_modification_time() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        let agent_path = write_agent_file(
            &project_agents,
            "cached-agent",
            &create_valid_agent_content("cached-agent", "original", "Original instructions")
        );

        let mut registry = SubagentRegistry::with_directories(project_agents.clone(), user_agents);

        // First load
        let report1 = registry.reload();
        assert_eq!(report1.loaded.len(), 1);
        assert_eq!(report1.loaded[0], "cached-agent");

        let record1 = registry.get("cached-agent").unwrap();
        assert_eq!(record1.spec.description(), Some("original"));

        // Second reload without file changes - should use cache
        let report2 = registry.reload();
        assert_eq!(report2.loaded.len(), 0); // No new loads due to caching
        assert_eq!(report2.removed.len(), 0);
        assert!(report2.errors.is_empty());

        // Modify file and reload - should detect change
        std::thread::sleep(Duration::from_millis(10)); // Ensure different mtime
        fs::write(&agent_path, &create_valid_agent_content("cached-agent", "updated", "Updated instructions")).unwrap();

        let report3 = registry.reload();
        assert_eq!(report3.loaded.len(), 1);
        assert_eq!(report3.loaded[0], "cached-agent");

        let record3 = registry.get("cached-agent").unwrap();
        assert_eq!(record3.spec.description(), Some("updated"));
    }

    #[test]
    fn test_registry_handles_removed_agents() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        let agent_path = write_agent_file(
            &project_agents,
            "temp-agent",
            &create_valid_agent_content("temp-agent", "temporary", "Temporary instructions")
        );

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);

        // Load agent
        let report1 = registry.reload();
        assert_eq!(report1.loaded.len(), 1);
        assert!(registry.get("temp-agent").is_ok());

        // Remove agent file
        fs::remove_file(&agent_path).unwrap();

        // Reload - should detect removal
        let report2 = registry.reload();
        assert_eq!(report2.removed.len(), 1);
        assert_eq!(report2.removed[0], "temp-agent");
        assert!(registry.get("temp-agent").is_err());
    }

    #[test]
    fn test_registry_error_handling_and_recovery() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create broken agent file
        let broken_path = write_agent_file(
            &project_agents,
            "broken-agent",
            "invalid yaml content without frontmatter"
        );

        let mut registry = SubagentRegistry::with_directories(project_agents.clone(), user_agents);

        // Load broken agent - should report error
        let report1 = registry.reload();
        assert_eq!(report1.loaded.len(), 0);
        assert_eq!(report1.errors.len(), 1);
        assert!(report1.errors[0].message.contains("front matter"));
        assert!(registry.get("broken-agent").is_err());

        // Fix the agent file
        fs::write(&broken_path, &create_valid_agent_content("broken-agent", "fixed", "Fixed instructions")).unwrap();

        // Reload - should recover from error
        let report2 = registry.reload();
        assert_eq!(report2.loaded.len(), 1);
        assert_eq!(report2.loaded[0], "broken-agent");
        assert!(report2.errors.is_empty());
        assert!(registry.get("broken-agent").is_ok());
    }

    #[test]
    fn test_registry_multiple_agents_different_sources() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create agents in different directories
        write_agent_file(
            &project_agents,
            "project-agent",
            &create_valid_agent_content("project-agent", "project-specific", "Project agent")
        );
        write_agent_file(
            &user_agents,
            "user-agent",
            &create_valid_agent_content("user-agent", "user-specific", "User agent")
        );
        write_agent_file(
            &project_agents,
            "project-agent-2",
            &create_valid_agent_content("project-agent-2", "another project", "Another project agent")
        );

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        assert_eq!(report.loaded.len(), 3);
        assert!(report.errors.is_empty());

        let agents = registry.list();
        assert_eq!(agents.len(), 3);

        // Verify sources are correctly assigned
        let project_agent = registry.get("project-agent").unwrap();
        assert_eq!(project_agent.source, AgentSource::Project);

        let user_agent = registry.get("user-agent").unwrap();
        assert_eq!(user_agent.source, AgentSource::User);

        let project_agent_2 = registry.get("project-agent-2").unwrap();
        assert_eq!(project_agent_2.source, AgentSource::Project);
    }

    #[test]
    fn test_registry_handles_missing_directories() {
        let nonexistent_project = PathBuf::from("/nonexistent/project/.codex/agents");
        let nonexistent_user = PathBuf::from("/nonexistent/user/.codex/agents");

        let mut registry = SubagentRegistry::with_directories(nonexistent_project, nonexistent_user);
        let report = registry.reload();

        // Should handle missing directories gracefully
        assert!(report.loaded.is_empty());
        assert!(report.removed.is_empty());
        assert!(report.errors.is_empty());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_registry_ignores_non_markdown_files() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        fs::create_dir_all(&project_agents).unwrap();

        // Create various file types
        fs::write(project_agents.join("agent.md"), &create_valid_agent_content("valid-agent", "valid", "Valid")).unwrap();
        fs::write(project_agents.join("README.txt"), "Not a markdown file").unwrap();
        fs::write(project_agents.join("config.json"), "{}").unwrap();
        fs::write(project_agents.join("no-extension"), "content").unwrap();

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        // Should only load the .md file
        assert_eq!(report.loaded.len(), 1);
        assert_eq!(report.loaded[0], "valid-agent");
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_registry_error_deduplication() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create multiple broken files with same error
        write_agent_file(&project_agents, "broken1", "invalid yaml");
        write_agent_file(&project_agents, "broken2", "invalid yaml");
        write_agent_file(&project_agents, "broken3", "invalid yaml");

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        // Should have errors for each file
        assert_eq!(report.errors.len(), 3);
        assert!(report.loaded.is_empty());

        // Verify errors contain path information
        for error in &report.errors {
            assert!(error.path.to_string_lossy().contains("broken"));
            assert!(error.message.contains("front matter"));
        }
    }

    #[test]
    fn test_registry_preserves_load_order() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create agents with specific naming to test order
        write_agent_file(&user_agents, "a-user", &create_valid_agent_content("a-user", "user", "User A"));
        write_agent_file(&project_agents, "b-project", &create_valid_agent_content("b-project", "project", "Project B"));
        write_agent_file(&user_agents, "c-user", &create_valid_agent_content("c-user", "user", "User C"));

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        assert_eq!(report.loaded.len(), 3);

        let agents = registry.list();
        assert_eq!(agents.len(), 3);

        // Verify user agents are processed before project agents (as per implementation)
        let user_agents_count = agents.iter().filter(|a| a.source == AgentSource::User).count();
        let project_agents_count = agents.iter().filter(|a| a.source == AgentSource::Project).count();
        assert_eq!(user_agents_count, 2);
        assert_eq!(project_agents_count, 1);
    }
}

/// Test edge cases and error scenarios
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_registry_operations() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);

        // Operations on empty registry
        assert!(registry.list().is_empty());
        assert!(registry.get("nonexistent").is_err());
        assert!(registry.last_errors().is_empty());

        let report = registry.reload();
        assert!(report.loaded.is_empty());
        assert!(report.removed.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_concurrent_registry_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create test agent
        write_agent_file(
            &project_agents,
            "concurrent-agent",
            &create_valid_agent_content("concurrent-agent", "test", "Test instructions")
        );

        let registry = Arc::new(Mutex::new(SubagentRegistry::with_directories(project_agents, user_agents)));

        // Load initial state
        {
            let mut r = registry.lock().unwrap();
            r.reload();
        }

        // Simulate concurrent read operations
        let handles: Vec<_> = (0..5).map(|i| {
            let registry_clone = Arc::clone(&registry);
            thread::spawn(move || {
                let r = registry_clone.lock().unwrap();
                let result = r.get("concurrent-agent");
                assert!(result.is_ok(), "Thread {} failed to get agent", i);
                result.unwrap().name().to_string()
            })
        }).collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads should successfully read the same agent
        for result in results {
            assert_eq!(result, "concurrent-agent");
        }
    }

    #[test]
    fn test_deep_directory_structure_ignored() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create nested directory structure
        let nested_dir = project_agents.join("subdirectory");
        write_agent_file(
            &project_agents,
            "top-level",
            &create_valid_agent_content("top-level", "valid", "Top level agent")
        );
        write_agent_file(
            &nested_dir,
            "nested",
            &create_valid_agent_content("nested", "should be ignored", "Nested agent")
        );

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();

        // Should only find top-level agent (max_depth(1) in registry)
        assert_eq!(report.loaded.len(), 1);
        assert_eq!(report.loaded[0], "top-level");
        assert!(registry.get("nested").is_err());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_registry_performance() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_root = TempDir::new().unwrap();
        let user_agents = user_root.path().to_path_buf();

        // Create many agent files
        for i in 0..50 {
            write_agent_file(
                &project_agents,
                &format!("agent-{:03}", i),
                &create_valid_agent_content(
                    &format!("agent-{:03}", i),
                    &format!("Agent number {}", i),
                    &format!("Instructions for agent {}", i)
                )
            );
        }

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);

        let start = SystemTime::now();
        let report = registry.reload();
        let elapsed = start.elapsed().unwrap();

        assert_eq!(report.loaded.len(), 50);
        assert!(report.errors.is_empty());
        assert!(elapsed < Duration::from_millis(1000)); // Should be fast

        // Test subsequent reload with caching
        let start2 = SystemTime::now();
        let report2 = registry.reload();
        let elapsed2 = start2.elapsed().unwrap();

        assert_eq!(report2.loaded.len(), 0); // Should use cache
        assert!(elapsed2 < Duration::from_millis(100)); // Should be even faster
    }
}