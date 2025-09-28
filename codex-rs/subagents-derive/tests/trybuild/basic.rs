use codex_subagents::Subagent;
use codex_subagents_derive::Subagent;

#[derive(Subagent)]
#[subagent(
    name = "code_reviewer",
    description = "Reviews code changes",
    model = "gpt-5-codex",
    tools = ["apply_patch", "git_diff"],
    keywords = ["review", "diff"],
    instructions = "Review the provided diff and point out issues."
)]
struct CodeReviewer;

fn main() {
    let agent = CodeReviewer;
    let spec = agent.spec();
    assert_eq!(spec.metadata.name, "code_reviewer");
    assert_eq!(spec.metadata.tools.len(), 2);
}
