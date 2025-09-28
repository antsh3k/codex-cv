//! CLI acceptance tests for `codex subagents list` and `run` commands.

use std::path::Path;
use std::fs;
use anyhow::Result;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value as JsonValue;
use tempfile::TempDir;

/// Helper to create a codex command with proper environment setup
fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::cargo_bin("codex")?;
    cmd.env("CODEX_HOME", codex_home);
    cmd.env("CODEX_SUBAGENTS_ENABLED", "true"); // Enable subagents for testing
    Ok(cmd)
}

/// Helper to create agent definition files for testing
fn create_agent_file(agents_dir: &Path, name: &str, content: &str) -> Result<()> {
    fs::create_dir_all(agents_dir)?;
    let file_path = agents_dir.join(format!("{}.md", name));
    fs::write(file_path, content)?;
    Ok(())
}

/// Helper to create a valid agent definition
fn create_valid_agent_content(name: &str, description: &str, tools: &[&str]) -> String {
    let tools_yaml = if tools.is_empty() {
        String::new()
    } else {
        format!("tools:\n{}", tools.iter().map(|t| format!("  - {}", t)).collect::<Vec<_>>().join("\n"))
    };

    format!(
        r#"---
name: {}
description: {}
{}
---
Test instructions for {} agent."#,
        name, description, tools_yaml, name
    )
}

/// Helper to create an invalid agent definition (malformed YAML)
fn create_invalid_agent_content() -> String {
    r#"---
name: broken-agent
invalid: yaml: structure
description: This will fail to parse
---
This agent has malformed YAML and should cause a parse error."#.to_string()
}

#[test]
fn subagents_list_shows_empty_state() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("No subagents discovered"));
    assert!(stdout.contains("Add Markdown specs under .codex/agents"));

    Ok(())
}

#[test]
fn subagents_list_shows_single_agent() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create a simple agent
    create_agent_file(
        &agents_dir,
        "test-agent",
        &create_valid_agent_content("test-agent", "A simple test agent", &["read", "write"])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Available subagents:"));
    assert!(stdout.contains("- test-agent: A simple test agent"));

    Ok(())
}

#[test]
fn subagents_list_shows_multiple_agents_with_details() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create multiple agents with different configurations
    create_agent_file(
        &agents_dir,
        "code-reviewer",
        &format!(
            r#"---
name: code-reviewer
description: Reviews code for potential issues
model: gpt-4
tools:
  - read
  - analysis
  - git
keywords:
  - review
  - quality
---
Analyze the provided code and identify potential bugs, security issues, and style violations."#
        )
    )?;

    create_agent_file(
        &agents_dir,
        "test-runner",
        &create_valid_agent_content("test-runner", "Executes test suites", &["bash", "read"])
    )?;

    create_agent_file(
        &agents_dir,
        "simple-agent",
        &create_valid_agent_content("simple-agent", "A minimal agent", &[])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Check header
    assert!(stdout.contains("Available subagents:"));

    // Check each agent is listed correctly
    assert!(stdout.contains("- code-reviewer: Reviews code for potential issues (model: gpt-4)"));
    assert!(stdout.contains("- test-runner: Executes test suites"));
    assert!(stdout.contains("- simple-agent: A minimal agent"));

    Ok(())
}

#[test]
fn subagents_list_shows_parse_errors() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create a valid agent
    create_agent_file(
        &agents_dir,
        "working-agent",
        &create_valid_agent_content("working-agent", "This agent works fine", &["read"])
    )?;

    // Create an invalid agent
    create_agent_file(
        &agents_dir,
        "broken-agent",
        &create_invalid_agent_content()
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should show the working agent
    assert!(stdout.contains("Available subagents:"));
    assert!(stdout.contains("- working-agent: This agent works fine"));

    // Should show parse errors
    assert!(stderr.contains("Errors:"));
    assert!(stderr.contains("broken-agent.md"));
    assert!(stderr.contains("error")); // Should contain some error description

    Ok(())
}

#[test]
fn subagents_list_handles_missing_directory() -> Result<()> {
    let codex_home = TempDir::new()?;
    // Don't create .codex/agents directory

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("No subagents discovered"));

    Ok(())
}

#[test]
fn subagents_run_unknown_agent_fails() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "run", "nonexistent-agent"]).output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("nonexistent-agent") || stderr.contains("not found") || stderr.contains("unknown"));

    Ok(())
}

#[test]
fn subagents_run_with_disabled_feature_fails() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create a valid agent
    create_agent_file(
        &agents_dir,
        "test-agent",
        &create_valid_agent_content("test-agent", "Test agent", &["read"])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.env_remove("CODEX_SUBAGENTS_ENABLED"); // Disable subagents
    let output = cmd.args(["subagents", "run", "test-agent"]).output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("disabled") || stderr.contains("not enabled"));

    Ok(())
}

#[test]
fn subagents_run_with_prompt_option() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create a valid agent
    create_agent_file(
        &agents_dir,
        "test-agent",
        &create_valid_agent_content("test-agent", "Test agent", &["read", "analysis"])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    // Note: This test may fail in CI without proper auth setup
    // but it tests the argument parsing and basic flow
    let output = cmd.args([
        "subagents",
        "run",
        "test-agent",
        "--prompt",
        "Test prompt message"
    ]).output()?;

    // The exact behavior depends on auth and network setup
    // but we can check that the command structure is valid
    let stderr = String::from_utf8(output.stderr)?;

    // Should not fail due to argument parsing issues
    assert!(!stderr.contains("argument") && !stderr.contains("usage:"));

    Ok(())
}

#[test]
fn subagents_run_short_prompt_option() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create a valid agent
    create_agent_file(
        &agents_dir,
        "test-agent",
        &create_valid_agent_content("test-agent", "Test agent", &["read"])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args([
        "subagents",
        "run",
        "test-agent",
        "-p",
        "Short prompt"
    ]).output()?;

    let stderr = String::from_utf8(output.stderr)?;
    // Should parse arguments correctly
    assert!(!stderr.contains("argument") && !stderr.contains("usage:"));

    Ok(())
}

#[test]
fn subagents_help_shows_correct_usage() -> Result<()> {
    let codex_home = TempDir::new()?;

    // Test main subagents help
    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "--help"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("list"));
    assert!(stdout.contains("run"));

    // Test subagents run help
    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "run", "--help"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("agent"));
    assert!(stdout.contains("prompt"));

    Ok(())
}

#[test]
fn subagents_list_with_project_and_user_agents() -> Result<()> {
    let codex_home = TempDir::new()?;

    // Create project-level agents
    let project_agents_dir = codex_home.path().join(".codex/agents");
    create_agent_file(
        &project_agents_dir,
        "project-agent",
        &create_valid_agent_content("project-agent", "Project-specific agent", &["git", "read"])
    )?;

    // Create user-level agents directory (this simulates the user's home directory structure)
    let user_agents_dir = codex_home.path().join("user/.codex/agents");
    create_agent_file(
        &user_agents_dir,
        "user-agent",
        &create_valid_agent_content("user-agent", "User-specific agent", &["bash"])
    )?;

    // Test that project agent is found
    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("- project-agent: Project-specific agent"));

    // Note: User agent may not be found unless we set up the user home directory properly
    // This test primarily validates project-level agent discovery

    Ok(())
}

#[test]
fn subagents_list_ignores_non_markdown_files() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");
    fs::create_dir_all(&agents_dir)?;

    // Create a valid agent
    create_agent_file(
        &agents_dir,
        "valid-agent",
        &create_valid_agent_content("valid-agent", "Valid agent", &["read"])
    )?;

    // Create non-markdown files that should be ignored
    fs::write(agents_dir.join("README.txt"), "This is not an agent")?;
    fs::write(agents_dir.join("config.json"), r#"{"not": "an agent"}"#)?;
    fs::write(agents_dir.join("script.sh"), "#!/bin/bash\necho 'not an agent'")?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should only show the valid agent
    assert!(stdout.contains("Available subagents:"));
    assert!(stdout.contains("- valid-agent: Valid agent"));

    // Should not reference the non-markdown files
    assert!(!stdout.contains("README.txt"));
    assert!(!stdout.contains("config.json"));
    assert!(!stdout.contains("script.sh"));

    Ok(())
}

#[test]
fn subagents_run_handles_agent_with_spaces_in_name() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create an agent with spaces in name (though this might not be recommended)
    create_agent_file(
        &agents_dir,
        "agent-with-spaces",
        &create_valid_agent_content("agent with spaces", "Agent with spaces in name", &["read"])
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "run", "agent with spaces"]).output()?;

    let stderr = String::from_utf8(output.stderr)?;
    // Should handle the spaces correctly in argument parsing
    assert!(!stderr.contains("usage:"));

    Ok(())
}

#[test]
fn subagents_list_performance_with_many_agents() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create many agents to test performance
    for i in 1..=20 {
        create_agent_file(
            &agents_dir,
            &format!("agent-{:02}", i),
            &create_valid_agent_content(
                &format!("agent-{:02}", i),
                &format!("Test agent number {}", i),
                &["read", "write"]
            )
        )?;
    }

    let start = std::time::Instant::now();
    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;
    let elapsed = start.elapsed();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should list all agents
    assert!(stdout.contains("Available subagents:"));
    for i in 1..=20 {
        assert!(stdout.contains(&format!("agent-{:02}", i)));
    }

    // Should complete in reasonable time (less than 5 seconds)
    assert!(elapsed.as_secs() < 5, "Command took too long: {:?}", elapsed);

    Ok(())
}

#[test]
fn subagents_error_output_goes_to_stderr() -> Result<()> {
    let codex_home = TempDir::new()?;
    let agents_dir = codex_home.path().join(".codex/agents");

    // Create an agent that will cause errors
    create_agent_file(&agents_dir, "broken", &create_invalid_agent_content())?;

    let mut cmd = codex_command(codex_home.path())?;
    let output = cmd.args(["subagents", "list"]).output()?;

    assert!(output.status.success()); // Command succeeds but has errors

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Normal output goes to stdout
    assert!(stdout.contains("No subagents discovered") || stdout.contains("Available subagents:"));

    // Errors go to stderr
    assert!(stderr.contains("Errors:"));
    assert!(stderr.contains("broken.md"));

    Ok(())
}