use crate::error::SubagentValidationError;
use crate::parser::validate_agent_name;
use crate::spec::AgentSource;
use crate::spec::SubagentMetadata;
use crate::spec::SubagentSpec;
use sha1::Digest;
use sha1::Sha1;
use std::path::PathBuf;

#[derive(Debug)]
pub struct SubagentBuilder {
    name: Option<String>,
    description: Option<String>,
    model: Option<String>,
    tools: Vec<String>,
    keywords: Vec<String>,
    instructions: Option<String>,
    source: AgentSource,
    source_path: Option<PathBuf>,
}

impl Default for SubagentBuilder {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            instructions: None,
            source: AgentSource::Inline,
            source_path: None,
        }
    }
}

impl SubagentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            instructions: None,
            source: AgentSource::Inline,
            source_path: None,
        }
    }

    pub fn description(mut self, description: impl Into<Option<String>>) -> Self {
        self.description = description.into();
        self
    }

    pub fn model(mut self, model: impl Into<Option<String>>) -> Self {
        self.model = model.into();
        self
    }

    pub fn tools<I, S>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    pub fn source(mut self, source: AgentSource) -> Self {
        self.source = source;
        self
    }

    pub fn source_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    pub fn build(self) -> Result<SubagentSpec, SubagentValidationError> {
        let name = self
            .name
            .ok_or(SubagentValidationError::MissingField("name"))?;
        validate_agent_name(&name)?;

        let instructions = self
            .instructions
            .ok_or(SubagentValidationError::MissingField("instructions"))?;

        let tools = normalize_unique(self.tools, false)?;
        let keywords = normalize_unique(self.keywords, true)?;

        let metadata = SubagentMetadata::new(name.clone())
            .description(self.description)
            .model(self.model)
            .tools(tools)
            .keywords(keywords);

        let mut hasher = Sha1::new();
        hasher.update(name.as_bytes());
        hasher.update(instructions.as_bytes());
        if let Some(model) = metadata.model.as_deref() {
            hasher.update(model.as_bytes());
        }
        for tool in &metadata.tools {
            hasher.update(tool.as_bytes());
        }
        for keyword in &metadata.keywords {
            hasher.update(keyword.as_bytes());
        }
        let hash = format!("{:x}", hasher.finalize());

        Ok(SubagentSpec {
            metadata,
            instructions,
            source_path: self.source_path,
            source: self.source,
            hash,
        })
    }
}

fn normalize_unique(
    items: Vec<String>,
    allow_empty: bool,
) -> Result<Vec<String>, SubagentValidationError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut output = Vec::new();
    for raw in items {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            if allow_empty {
                return Err(SubagentValidationError::EmptyKeyword);
            } else {
                return Err(SubagentValidationError::EmptyTool);
            }
        }
        if !seen.insert(trimmed.clone()) {
            if allow_empty {
                return Err(SubagentValidationError::DuplicateKeyword(trimmed));
            } else {
                return Err(SubagentValidationError::DuplicateTool(trimmed));
            }
        }
        output.push(trimmed);
    }
    Ok(output)
}
