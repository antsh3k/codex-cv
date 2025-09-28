use crate::builder::SubagentBuilder;
use crate::error::SubagentError;
use crate::error::SubagentResult;
use crate::spec::SubagentSpec;
use indexmap::IndexSet;
use serde::Deserialize;
use std::path::PathBuf;

const FRONT_MATTER_DELIM: &str = "---";

#[derive(Debug, Deserialize)]
struct RawFrontMatter {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

pub fn parse_document(
    contents: &str,
    source_path: Option<PathBuf>,
) -> SubagentResult<SubagentSpec> {
    let normalized = contents.replace('\r', "");
    let trimmed = normalized.trim_start_matches(['\u{feff}', '\n']);

    if !trimmed.starts_with(FRONT_MATTER_DELIM) {
        return Err(SubagentError::InvalidSpec(
            "subagent definition must start with YAML front matter delimited by `---`".into(),
        ));
    }

    let after_delim = trimmed
        .strip_prefix(FRONT_MATTER_DELIM)
        .expect("checked starts_with");

    let (front_matter_raw, body) = split_front_matter(after_delim).ok_or_else(|| {
        SubagentError::InvalidSpec("missing closing front matter delimiter `---`".into())
    })?;

    let path_for_error = source_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("<inline>"));
    let mut front_matter: RawFrontMatter =
        serde_yaml::from_str(front_matter_raw).map_err(|source| SubagentError::FrontMatter {
            path: path_for_error.clone(),
            source,
        })?;

    front_matter.tools = clean_list(front_matter.tools);
    front_matter.keywords = clean_list(front_matter.keywords);

    let instructions = body.trim_start_matches('\n').trim().to_string();

    let mut builder = SubagentBuilder::new(front_matter.name)
        .instructions(instructions)
        .keywords(front_matter.keywords)
        .tools(front_matter.tools);

    if let Some(description) = front_matter.description {
        builder = builder.description(description);
    }
    if let Some(model) = front_matter.model {
        builder = builder.model(model);
    }
    if let Some(path) = source_path {
        builder = builder.source_path(path);
    }

    builder.build()
}

fn split_front_matter(after_delim: &str) -> Option<(&str, &str)> {
    let after_delim = after_delim.strip_prefix('\n').unwrap_or(after_delim);
    let closing = after_delim.find("\n---\n")?;
    let (front_matter, rest) = after_delim.split_at(closing);
    let body = rest.strip_prefix("\n---\n").unwrap_or("");
    Some((front_matter, body))
}

fn clean_list(values: Vec<String>) -> Vec<String> {
    let mut unique = IndexSet::new();
    for v in values {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_full_document() {
        let doc = r#"---
name: code-reviewer
description: Finds issues in Git diffs
model: gpt-5-codex
keywords:
  - review
  - rust
  - review
---
Analyze the staged diff and report any issues.
"#;
        let spec =
            parse_document(doc, Some(PathBuf::from("/repo/.codex/agents/reviewer.md"))).unwrap();

        assert_eq!(spec.name(), "code-reviewer");
        assert_eq!(spec.description(), Some("Finds issues in Git diffs"));
        assert_eq!(spec.model(), Some("gpt-5-codex"));
        assert_eq!(spec.keywords(), &["review".to_string(), "rust".to_string()]);
        assert_eq!(
            spec.instructions(),
            "Analyze the staged diff and report any issues."
        );
        assert_eq!(
            spec.source_path().unwrap(),
            PathBuf::from("/repo/.codex/agents/reviewer.md")
        );
    }

    #[test]
    fn errors_without_front_matter() {
        let err = parse_document("# missing front matter", None).unwrap_err();
        assert!(
            matches!(err, SubagentError::InvalidSpec(message) if message.contains("front matter"))
        );
    }
}
