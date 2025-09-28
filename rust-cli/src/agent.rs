use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parsed metadata describing a subagent definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSpec {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub keywords: Vec<String>,
    pub instructions: String,
    pub source_path: Option<PathBuf>,
    pub source: AgentSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AgentSource {
    Builtin,
    User,
}

impl std::fmt::Display for AgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSource::Builtin => write!(f, "builtin"),
            AgentSource::User => write!(f, "user"),
        }
    }
}

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

impl AgentSpec {
    pub fn new(name: impl Into<String>, instructions: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            instructions: instructions.into(),
            source_path: None,
            source: AgentSource::User,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn tools(&self) -> &[String] {
        &self.tools
    }

    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub fn source_path(&self) -> Option<&PathBuf> {
        self.source_path.as_ref()
    }

    /// Parse agent definition from markdown content
    pub fn parse_document(contents: &str, source_path: Option<PathBuf>, source: AgentSource) -> Result<Self> {
        let normalized = contents.replace('\r', "");
        let trimmed = normalized.trim_start_matches(['\u{feff}', '\n']);

        if !trimmed.starts_with("---") {
            return Err(anyhow!("Agent definition must start with YAML front matter delimited by `---`"));
        }

        let after_delim = trimmed
            .strip_prefix("---")
            .ok_or_else(|| anyhow!("Missing front matter delimiter"))?;

        let (front_matter_raw, body) = split_front_matter(after_delim)
            .ok_or_else(|| anyhow!("Missing closing front matter delimiter `---`"))?;

        let mut front_matter: RawFrontMatter = serde_yaml::from_str(front_matter_raw)
            .map_err(|e| anyhow!("YAML parsing error: {}", e))?;

        // Clean up lists
        front_matter.tools = clean_list(front_matter.tools);
        front_matter.keywords = clean_list(front_matter.keywords);

        let instructions = body.trim_start_matches('\n').trim().to_string();

        Ok(AgentSpec {
            name: front_matter.name,
            description: front_matter.description,
            model: front_matter.model,
            tools: front_matter.tools,
            keywords: front_matter.keywords,
            instructions,
            source_path,
            source,
        })
    }

    /// Validate agent specification
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Agent name cannot be empty"));
        }

        if !self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(anyhow!("Agent name must contain only alphanumeric characters, hyphens, and underscores"));
        }

        if self.instructions.trim().is_empty() {
            return Err(anyhow!("Agent instructions cannot be empty"));
        }

        Ok(())
    }

    /// Convert to YAML frontmatter format for saving
    pub fn to_markdown(&self) -> String {
        let mut yaml = String::from("---\n");
        yaml.push_str(&format!("name: {}\n", self.name));

        if let Some(desc) = &self.description {
            yaml.push_str(&format!("description: {}\n", desc));
        }

        if let Some(model) = &self.model {
            yaml.push_str(&format!("model: {}\n", model));
        }

        if !self.tools.is_empty() {
            yaml.push_str("tools:\n");
            for tool in &self.tools {
                yaml.push_str(&format!("  - {}\n", tool));
            }
        }

        if !self.keywords.is_empty() {
            yaml.push_str("keywords:\n");
            for keyword in &self.keywords {
                yaml.push_str(&format!("  - {}\n", keyword));
            }
        }

        yaml.push_str("---\n\n");
        yaml.push_str(&self.instructions);
        yaml
    }
}

fn split_front_matter(after_delim: &str) -> Option<(&str, &str)> {
    let after_delim = after_delim.strip_prefix('\n').unwrap_or(after_delim);
    let closing = after_delim.find("\n---\n")?;
    let (front_matter, rest) = after_delim.split_at(closing);
    let body = rest.strip_prefix("\n---\n").unwrap_or("");
    Some((front_matter, body))
}

fn clean_list(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for v in values {
        let trimmed = v.trim().to_string();
        if !trimmed.is_empty() && !seen.contains(&trimmed) {
            seen.insert(trimmed.clone());
            result.push(trimmed);
        }
    }

    result
}