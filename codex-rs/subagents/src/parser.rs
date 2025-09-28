use crate::builder::SubagentBuilder;
use crate::error::ParserError;
use crate::error::SubagentValidationError;
use crate::spec::AgentSource;
use crate::spec::ModelBinding;
use crate::spec::SubagentSpec;
use once_cell::sync::Lazy;
use regex_lite::Regex;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
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
    #[serde(default)]
    model_config: Option<FrontmatterModelConfig>,
    tools: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct FrontmatterModelConfig {
    provider: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
    #[serde(default)]
    parameters: BTreeMap<String, JsonValue>,
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

    let simple_model = normalize_optional_string(frontmatter.model);
    let model_binding = match frontmatter.model_config {
        Some(cfg) => Some(parse_model_config(cfg, &simple_model)?),
        None => simple_model.clone().map(|model| ModelBinding {
            provider_id: None,
            model: Some(model),
            endpoint: None,
            parameters: BTreeMap::new(),
        }),
    };

    let mut builder = SubagentBuilder::new(name)
        .description(frontmatter.description)
        .model(simple_model.clone())
        .model_config(model_binding)
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

fn parse_model_config(
    raw: FrontmatterModelConfig,
    simple_model: &Option<String>,
) -> Result<ModelBinding, ParserError> {
    let provider_id = raw
        .provider
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(SubagentValidationError::InvalidModelProvider)
            } else {
                Ok(trimmed.to_string())
            }
        })
        .transpose()?;

    let endpoint = raw
        .endpoint
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(SubagentValidationError::InvalidModelEndpoint)
            } else {
                Ok(trimmed.to_string())
            }
        })
        .transpose()?;

    let mut parameters = BTreeMap::new();
    for (key, value) in raw.parameters.into_iter() {
        if key.trim().is_empty() {
            return Err(SubagentValidationError::InvalidModelParameterKey.into());
        }
        parameters.insert(key.trim().to_string(), value);
    }

    let mut binding = ModelBinding {
        provider_id,
        model: normalize_optional_string(raw.model),
        endpoint,
        parameters,
    };

    if let Some(model) = simple_model.as_ref() {
        if let Some(binding_model) = binding.model.as_ref() {
            if binding_model != model {
                return Err(SubagentValidationError::ConflictingModelDefinitions {
                    model: model.clone(),
                    model_config: binding_model.clone(),
                }
                .into());
            }
        } else {
            binding.model = Some(model.clone());
        }
    }

    Ok(binding)
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
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
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn parses_frontmatter_and_body() {
        let doc = "---\nname: reviewer\ndescription: Review diffs\ntools: [apply_patch]\n---\nBody text here.";
        let parsed = parse_agent_str(doc, Path::new("reviewer.md"), AgentSource::Project).unwrap();
        assert_eq!(parsed.spec.metadata.name, "reviewer");
        assert_eq!(parsed.spec.instructions, "Body text here.");
        assert_eq!(parsed.spec.metadata.tools, vec!["apply_patch"]);
        assert_eq!(parsed.spec.metadata.model.as_deref(), None);
        assert!(parsed.spec.metadata.model_config.is_none());
    }

    #[test]
    fn parses_structured_model_config() {
        let doc = r"---
name: code-reviewer
model: gpt-4o
model_config:
  provider: openai
  endpoint: https://proxy.mycompany.dev/v1
  parameters:
    temperature: 0.1
---
Follow the usual review checklist.";

        let parsed =
            parse_agent_str(doc, Path::new("code-reviewer.md"), AgentSource::Project).unwrap();
        let metadata = &parsed.spec.metadata;
        assert_eq!(metadata.model.as_deref(), Some("gpt-4o"));
        let binding = metadata.model_config.as_ref().expect("binding");
        assert_eq!(binding.provider_id.as_deref(), Some("openai"));
        assert_eq!(binding.model.as_deref(), Some("gpt-4o"));
        assert_eq!(
            binding.endpoint.as_deref(),
            Some("https://proxy.mycompany.dev/v1")
        );
        assert_eq!(binding.parameters.get("temperature"), Some(&json!(0.1)));
    }

    #[test]
    fn rejects_conflicting_models() {
        let doc = r"---
name: mismatch
model: gpt-4
model_config:
  model: gpt-4o
---
text";
        let err = parse_agent_str(doc, Path::new("mismatch.md"), AgentSource::Project).unwrap_err();
        assert!(matches!(
            err,
            ParserError::Validation(SubagentValidationError::ConflictingModelDefinitions { .. })
        ));
    }

    #[test]
    fn rejects_empty_provider() {
        let doc = r#"---
name: bad
model_config:
  provider: "   "
---
text"#;
        let err = parse_agent_str(doc, Path::new("bad.md"), AgentSource::Project).unwrap_err();
        assert!(matches!(
            err,
            ParserError::Validation(SubagentValidationError::InvalidModelProvider)
        ));
    }
}
