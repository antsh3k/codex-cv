use crate::builder::SubagentBuilder;
use crate::error::ParserError;
use crate::error::SubagentValidationError;
use crate::spec::AgentSource;
use crate::spec::SubagentSpec;
use once_cell::sync::Lazy;
use regex_lite::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedAgent {
    pub spec: SubagentSpec,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    model: Option<String>,
    tools: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
}

const FRONTMATTER_DELIM: &str = "---";

pub fn parse_agent_file(path: &Path, source: AgentSource) -> Result<ParsedAgent, ParserError> {
    let contents = fs::read_to_string(path)?;
    parse_agent_str(&contents, path, source)
}

pub fn parse_agent_str(
    contents: &str,
    path: &Path,
    source: AgentSource,
) -> Result<ParsedAgent, ParserError> {
    let (frontmatter_raw, body) = split_frontmatter(contents)?;
    let frontmatter: Frontmatter =
        serde_yaml::from_str(frontmatter_raw).map_err(ParserError::InvalidFrontmatter)?;

    let name = frontmatter
        .name
        .ok_or(SubagentValidationError::MissingField("name"))?;
    validate_agent_name(&name)?;

    let instructions = body.trim().to_string();
    if instructions.is_empty() {
        return Err(SubagentValidationError::MissingField("instructions").into());
    }

    let mut builder = SubagentBuilder::new(name)
        .description(frontmatter.description)
        .model(frontmatter.model)
        .source(source)
        .source_path(path.to_path_buf())
        .instructions(instructions);

    if let Some(tools) = frontmatter.tools {
        builder = builder.tools(tools);
    }
    if let Some(keywords) = frontmatter.keywords {
        builder = builder.keywords(keywords);
    }

    let spec = builder.build()?;
    let warnings = Vec::new();

    Ok(ParsedAgent { spec, warnings })
}

pub fn validate_agent_name(name: &str) -> Result<(), SubagentValidationError> {
    static NAME_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-z][a-z0-9_-]{2,63}$").expect("compiled name regex"));
    if NAME_RE.is_match(name) {
        Ok(())
    } else {
        Err(SubagentValidationError::InvalidName {
            name: name.to_string(),
            reason: "name must start with a lowercase letter, include only lowercase letters, digits, hyphen, or underscore, and be 3-64 characters long".to_string(),
        })
    }
}

fn split_frontmatter(contents: &str) -> Result<(&str, &str), ParserError> {
    let trimmed = contents.trim_start_matches('\u{feff}');

    if !trimmed.starts_with(FRONTMATTER_DELIM) {
        return Err(ParserError::MissingFrontmatter);
    }

    let mut rest = &trimmed[FRONTMATTER_DELIM.len()..];
    rest = rest.strip_prefix('\r').unwrap_or(rest);
    rest = rest
        .strip_prefix('\n')
        .ok_or(ParserError::MissingFrontmatter)?;

    if let Some(idx) = rest.find("\n---") {
        let frontmatter = &rest[..idx];
        let mut body = &rest[idx + 4..];
        if body.starts_with('\r') {
            body = &body[1..];
        }
        if body.starts_with('\n') {
            body = &body[1..];
        }
        Ok((frontmatter.trim(), body))
    } else if let Some(frontmatter) = rest.strip_suffix("\n---") {
        Ok((frontmatter.trim(), ""))
    } else if let Some(frontmatter) = rest.strip_suffix("---") {
        Ok((frontmatter.trim(), ""))
    } else {
        Err(ParserError::MissingFrontmatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn parses_frontmatter_and_body() {
        let doc = "---\nname: reviewer\ndescription: Review diffs\ntools: [apply_patch]\n---\nBody text here.";
        let parsed = parse_agent_str(doc, Path::new("reviewer.md"), AgentSource::Project).unwrap();
        assert_eq!(parsed.spec.metadata.name, "reviewer");
        assert_eq!(parsed.spec.instructions, "Body text here.");
        assert_eq!(parsed.spec.metadata.tools, vec!["apply_patch"]);
    }
}
