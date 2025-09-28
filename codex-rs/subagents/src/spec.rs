use std::path::Path;
use std::path::PathBuf;

/// Parsed metadata describing a subagent definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentSpec {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub keywords: Vec<String>,
    pub instructions: String,
    pub source_path: Option<PathBuf>,
}

impl SubagentSpec {
    pub fn new(name: impl Into<String>, instructions: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            instructions: instructions.into(),
            source_path: None,
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

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_model(mut self, model: Option<String>) -> Self {
        self.model = model;
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_source_path(mut self, source_path: Option<PathBuf>) -> Self {
        self.source_path = source_path;
        self
    }
}
