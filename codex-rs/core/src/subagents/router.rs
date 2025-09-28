use codex_subagents::SubagentSpec;
use std::borrow::Cow;

/// Simple keyword-driven router placeholder.
#[derive(Debug, Default)]
pub struct SubagentRouter;

impl SubagentRouter {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to select an agent based on the provided user input.
    /// The MVP implementation performs a basic keyword search.
    pub fn route<'a>(&self, input: &'a str, agents: &'a [SubagentSpec]) -> Option<Cow<'a, str>> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        let lower = trimmed.to_ascii_lowercase();
        agents.iter().find_map(|spec| {
            if spec
                .keywords()
                .iter()
                .any(|kw| lower.contains(&kw.to_ascii_lowercase()))
            {
                Some(Cow::Borrowed(spec.name()))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_subagents::SubagentBuilder;

    #[test]
    fn matches_keywords_case_insensitive() {
        let spec = SubagentBuilder::new("reviewer")
            .instructions("body")
            .keywords(["Rust", "security"])
            .build()
            .unwrap();
        let router = SubagentRouter::new();
        assert_eq!(
            router.route("Please review this rust module", &[spec.clone()]),
            Some(Cow::Borrowed("reviewer"))
        );
    }

    #[test]
    fn ignores_empty_input() {
        let spec = SubagentBuilder::new("demo")
            .instructions("body")
            .keywords(["demo"])
            .build()
            .unwrap();
        let router = SubagentRouter::new();
        assert!(router.route("   ", &[spec.clone()]).is_none());
    }

    #[test]
    fn returns_none_when_no_match() {
        let spec = SubagentBuilder::new("lint")
            .instructions("body")
            .keywords(["lint"])
            .build()
            .unwrap();
        let router = SubagentRouter::new();
        assert!(router.route("run tests", &[spec]).is_none());
    }
}
