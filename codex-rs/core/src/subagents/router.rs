use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct RouteIntent<'a> {
    pub text: &'a str,
    pub explicit_agent: Option<&'a str>,
    pub auto_route: bool,
    pub candidates: &'a [RouteCandidate<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct RouteCandidate<'a> {
    pub name: &'a str,
    pub keywords: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentRoute {
    pub agent_name: Option<String>,
    pub reason: Option<String>,
}

impl SubagentRoute {
    fn none_with_reason(reason: impl Into<String>) -> Self {
        Self {
            agent_name: None,
            reason: Some(reason.into()),
        }
    }

    fn matched(agent_name: &str, reason: impl Into<String>) -> Self {
        Self {
            agent_name: Some(agent_name.to_string()),
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Default)]
pub struct SubagentRouter;

impl SubagentRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn route<'a>(&self, intent: RouteIntent<'a>) -> SubagentRoute {
        if intent.candidates.is_empty() {
            return SubagentRoute::none_with_reason("No registered subagents.");
        }

        match detect_explicit(&intent) {
            ExplicitRequest::MissingFromCommand => {
                return SubagentRoute::none_with_reason("Provide an agent name after `/use`.");
            }
            ExplicitRequest::Named { raw, source } => {
                return resolve_explicit(raw, source, intent.candidates);
            }
            ExplicitRequest::None => {}
        }

        if !intent.auto_route {
            return SubagentRoute::none_with_reason("Auto-routing disabled.");
        }

        auto_route(intent)
    }
}

enum ExplicitRequest<'a> {
    None,
    MissingFromCommand,
    Named {
        raw: &'a str,
        source: ExplicitSource,
    },
}

enum ExplicitSource {
    Direct,
    SlashCommand,
}

fn detect_explicit<'a>(intent: &'a RouteIntent<'a>) -> ExplicitRequest<'a> {
    if let Some(agent) = intent.explicit_agent {
        let trimmed = agent.trim();
        return if trimmed.is_empty() {
            ExplicitRequest::MissingFromCommand
        } else {
            ExplicitRequest::Named {
                raw: trimmed,
                source: ExplicitSource::Direct,
            }
        };
    }

    parse_slash_use_command(intent.text)
}

fn parse_slash_use_command<'a>(text: &'a str) -> ExplicitRequest<'a> {
    let trimmed = text.trim_start();
    let Some(rest) = trimmed.strip_prefix("/use") else {
        return ExplicitRequest::None;
    };

    let argument = rest.trim_start();
    if argument.is_empty() {
        return ExplicitRequest::MissingFromCommand;
    }

    if let Some(without_quote) = argument.strip_prefix('"')
        && let Some(end) = without_quote.find('"')
    {
        let content = &without_quote[..end];
        return if content.trim().is_empty() {
            ExplicitRequest::MissingFromCommand
        } else {
            ExplicitRequest::Named {
                raw: content.trim(),
                source: ExplicitSource::SlashCommand,
            }
        };
    }

    let name = argument
        .trim_matches('"')
        .split_whitespace()
        .next()
        .unwrap_or_default();
    if name.trim().is_empty() {
        ExplicitRequest::MissingFromCommand
    } else {
        ExplicitRequest::Named {
            raw: name.trim(),
            source: ExplicitSource::SlashCommand,
        }
    }
}

fn resolve_explicit(
    requested: &str,
    source: ExplicitSource,
    candidates: &[RouteCandidate<'_>],
) -> SubagentRoute {
    if let Some(candidate) = find_candidate(candidates, requested) {
        let reason = match source {
            ExplicitSource::Direct => format!("selected explicitly ('{requested}')"),
            ExplicitSource::SlashCommand => format!("requested via `/use {requested}`"),
        };
        SubagentRoute::matched(candidate.name, reason)
    } else {
        SubagentRoute::none_with_reason(format!("Unknown subagent '{requested}'."))
    }
}

fn auto_route(intent: RouteIntent<'_>) -> SubagentRoute {
    let text_lower = intent.text.to_ascii_lowercase();
    let text_tokens = tokenize_set(intent.text);

    let mut top: Vec<CandidateScore<'_>> = Vec::new();
    let mut best_score = 0usize;

    for candidate in intent.candidates {
        let (score, reasons) = score_candidate(candidate, &text_lower, &text_tokens);
        if score == 0 {
            continue;
        }

        if score > best_score {
            best_score = score;
            top.clear();
            top.push(CandidateScore { candidate, reasons });
        } else if score == best_score {
            top.push(CandidateScore { candidate, reasons });
        }
    }

    if top.is_empty() {
        return SubagentRoute::none_with_reason("No confident keyword match.");
    }

    if top.len() > 1 {
        let names: Vec<&str> = top.iter().map(|entry| entry.candidate.name).collect();
        return SubagentRoute::none_with_reason(format!(
            "Multiple agents matched: {}",
            names.join(", "),
        ));
    }

    let best = top.remove(0);
    let reason = match best.reasons.len() {
        0 => format!("matched agent name '{}'", best.candidate.name),
        1 => format!("matched {}", best.reasons[0]),
        _ => format!("matched {}", best.reasons.join(", ")),
    };
    SubagentRoute::matched(best.candidate.name, reason)
}

struct CandidateScore<'a> {
    candidate: &'a RouteCandidate<'a>,
    reasons: Vec<String>,
}

fn find_candidate<'a>(
    candidates: &'a [RouteCandidate<'a>],
    requested: &str,
) -> Option<&'a RouteCandidate<'a>> {
    let requested_slug = slugify(requested);
    candidates.iter().find(|candidate| {
        slugify(candidate.name) == requested_slug
            || candidate
                .keywords
                .iter()
                .any(|keyword| slugify(keyword) == requested_slug)
    })
}

fn score_candidate(
    candidate: &RouteCandidate<'_>,
    text_lower: &str,
    text_tokens: &HashSet<String>,
) -> (usize, Vec<String>) {
    let mut score = 0usize;
    let mut reasons = Vec::new();

    let name_tokens = tokenize(candidate.name);
    if !name_tokens.is_empty() && name_tokens.iter().all(|token| text_tokens.contains(token)) {
        score += 3;
        reasons.push(format!("agent name '{}'", candidate.name));
    } else {
        let lower_name = candidate.name.to_ascii_lowercase();
        if text_lower.contains(&lower_name) {
            score += 2;
            reasons.push(format!("agent name '{}'", candidate.name));
        }
    }

    let mut seen_keywords = HashSet::new();
    for keyword in candidate.keywords.iter() {
        let trimmed = keyword.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = trimmed.to_ascii_lowercase();
        if !seen_keywords.insert(normalized.clone()) {
            continue;
        }

        let keyword_tokens = tokenize(trimmed);
        if !keyword_tokens.is_empty()
            && keyword_tokens
                .iter()
                .all(|token| text_tokens.contains(token))
        {
            score += 1;
            reasons.push(format!("keyword '{trimmed}'"));
            continue;
        }

        if text_lower.contains(&normalized) {
            score += 1;
            reasons.push(format!("keyword '{trimmed}'"));
        }
    }

    (score, reasons)
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim();
            if token.is_empty() {
                None
            } else {
                Some(token.to_ascii_lowercase())
            }
        })
        .collect()
}

fn tokenize_set(text: &str) -> HashSet<String> {
    tokenize(text).into_iter().collect()
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        }
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    struct Fixture {
        spec_keywords: Vec<String>,
        tester_keywords: Vec<String>,
        reviewer_keywords: Vec<String>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                spec_keywords: vec![
                    "requirements".into(),
                    "spec parser".into(),
                    "acceptance criteria".into(),
                ],
                tester_keywords: vec!["tests".into(), "verification".into(), "pass/fail".into()],
                reviewer_keywords: vec!["review".into(), "lint".into(), "security".into()],
            }
        }

        fn candidates(&self) -> Vec<RouteCandidate<'_>> {
            vec![
                RouteCandidate {
                    name: "spec-parser",
                    keywords: &self.spec_keywords,
                },
                RouteCandidate {
                    name: "tester",
                    keywords: &self.tester_keywords,
                },
                RouteCandidate {
                    name: "reviewer",
                    keywords: &self.reviewer_keywords,
                },
            ]
        }
    }

    fn router() -> SubagentRouter {
        SubagentRouter::new()
    }

    #[test]
    fn explicit_agent_from_intent() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "run whatever",
            explicit_agent: Some("tester"),
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name.as_deref(), Some("tester"));
        assert_eq!(
            result.reason.as_deref(),
            Some("selected explicitly ('tester')"),
        );
    }

    #[test]
    fn explicit_agent_via_slash_command() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "/use spec-parser please",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name.as_deref(), Some("spec-parser"));
        assert_eq!(
            result.reason.as_deref(),
            Some("requested via `/use spec-parser`"),
        );
    }

    #[test]
    fn slash_command_missing_name() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "  /use   ",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name, None);
        assert_eq!(
            result.reason.as_deref(),
            Some("Provide an agent name after `/use`."),
        );
    }

    #[test]
    fn slash_command_unknown_agent() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "/use summarizer",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name, None);
        assert_eq!(
            result.reason.as_deref(),
            Some("Unknown subagent 'summarizer'."),
        );
    }

    #[test]
    fn auto_route_by_keyword() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "Please parse the requirements spec for me.",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name.as_deref(), Some("spec-parser"));
        assert_eq!(
            result.reason.as_deref(),
            Some("matched keyword 'requirements'"),
        );
    }

    #[test]
    fn auto_route_disabled_returns_none() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "tester please run",
            explicit_agent: None,
            auto_route: false,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name, None);
        assert_eq!(result.reason.as_deref(), Some("Auto-routing disabled."));
    }

    #[test]
    fn auto_route_ambiguous_keywords() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "We need tests and a review.",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name, None);
        assert_eq!(
            result.reason.as_deref(),
            Some("Multiple agents matched: tester, reviewer"),
        );
    }

    #[test]
    fn auto_route_by_name_tokens() {
        let fixture = Fixture::new();
        let candidates = fixture.candidates();
        let result = router().route(RouteIntent {
            text: "Spec parser should handle this",
            explicit_agent: None,
            auto_route: true,
            candidates: &candidates,
        });

        assert_eq!(result.agent_name.as_deref(), Some("spec-parser"));
        let reason = result.reason.expect("reason present");
        assert!(reason.contains("matched agent name 'spec-parser'"));
    }
}
