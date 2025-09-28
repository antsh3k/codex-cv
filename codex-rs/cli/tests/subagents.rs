use std::fs;
use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn write_agent(path: &Path, name: &str) -> Result<()> {
    fs::write(
        path,
        format!(
            "---\nname: {name}\ndescription: Test agent\nmodel: gpt-5-codex\n---\nPerform test task.\n"
        ),
    )?;
    Ok(())
}

fn codex_command(codex_home: &Path, cwd: &Path) -> Result<Command> {
    let mut cmd = Command::cargo_bin("codex")?;
    cmd.env("CODEX_HOME", codex_home);
    cmd.current_dir(cwd);
    Ok(cmd)
}

#[test]
fn list_displays_discovered_agents() -> Result<()> {
    let codex_home = TempDir::new()?;
    let project_dir = TempDir::new()?;
    let agents_dir = project_dir.path().join(".codex/agents");
    fs::create_dir_all(&agents_dir)?;
    write_agent(&agents_dir.join("writer.md"), "writer")?;

    let mut cmd = codex_command(codex_home.path(), project_dir.path())?;
    cmd.args(["subagents", "list"])
        .assert()
        .success()
        .stdout(contains("writer"));
    Ok(())
}

#[test]
fn run_requires_feature_flag() -> Result<()> {
    let codex_home = TempDir::new()?;
    let project_dir = TempDir::new()?;
    let agents_dir = project_dir.path().join(".codex/agents");
    fs::create_dir_all(&agents_dir)?;
    write_agent(&agents_dir.join("reviewer.md"), "reviewer")?;

    let mut cmd = codex_command(codex_home.path(), project_dir.path())?;
    let output = cmd.args(["subagents", "run", "reviewer"]).output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Subagents feature is disabled"));
    Ok(())
}
