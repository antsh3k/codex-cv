use codex_subagents::Subagent;
use codex_subagents::SubagentBuilder;
use codex_subagents_derive::Subagent as SubagentDerive;

#[derive(SubagentDerive)]
struct DemoAgent {
    spec: codex_subagents::SubagentSpec,
}

#[test]
fn derive_exposes_spec() {
    let spec = SubagentBuilder::new("demo")
        .instructions("body")
        .build()
        .unwrap();
    let mut agent = DemoAgent { spec };
    assert_eq!(agent.name(), "demo");
    assert_eq!(agent.instructions(), "body");

    agent.spec_mut().description = Some("desc".to_string());
    assert_eq!(agent.spec().description(), Some("desc"));
}
