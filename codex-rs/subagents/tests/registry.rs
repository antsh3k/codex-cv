use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use pretty_assertions::assert_eq;

use codex_subagents::AgentSource;
use codex_subagents::SubagentRegistry;
use tempfile::tempdir;

fn write_agent_file(path: &Path, name: &str, description: &str) {
    let mut file = fs::File::create(path).expect("create agent file");
    writeln!(
        file,
        "---\nname: {name}\ndescription: {description}\n---\nInstructions for {name}."
    )
    .expect("write agent file");
}

#[test]
fn project_agents_override_user_agents() {
    let user_dir = tempdir().expect("user dir");
    let project_dir = tempdir().expect("project dir");

    let user_agent = user_dir.path().join("reviewer.md");
    let project_agent = project_dir.path().join("reviewer.md");

    write_agent_file(&user_agent, "reviewer", "User reviewer");
    write_agent_file(&project_agent, "reviewer", "Project reviewer");

    let mut registry = SubagentRegistry::new(project_dir.path(), user_dir.path());
    let snapshot = registry.reload().expect("reload registry").clone();

    assert_eq!(
        snapshot.agents.len(),
        1,
        "expected project agent to override user"
    );
    let handle = &snapshot.agents[0];
    assert_eq!(handle.spec.metadata.name, "reviewer");
    assert_eq!(
        handle.spec.metadata.description.as_deref(),
        Some("Project reviewer")
    );
    assert_eq!(handle.spec.source, AgentSource::Project);
    assert!(snapshot.parse_errors.is_empty());
}

#[test]
fn registry_picks_up_modified_agents() {
    let user_dir = tempdir().expect("user dir");
    let project_dir = tempdir().expect("project dir");

    let project_agent = project_dir.path().join("writer.md");
    write_agent_file(&project_agent, "writer", "Initial version");

    let mut registry = SubagentRegistry::new(project_dir.path(), user_dir.path());
    let first = registry.reload().expect("first load").clone();
    let handle = &first.agents[0];
    assert_eq!(
        handle.spec.metadata.description.as_deref(),
        Some("Initial version")
    );

    // Ensure filesystem timestamp advances before rewriting the file so mtime differs.
    thread::sleep(Duration::from_millis(5));
    write_agent_file(&project_agent, "writer", "Updated version");

    let second = registry.reload().expect("second load").clone();
    let handle = &second.agents[0];
    assert_eq!(
        handle.spec.metadata.description.as_deref(),
        Some("Updated version")
    );
}
