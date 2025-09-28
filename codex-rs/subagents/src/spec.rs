use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentSource {
    Project,
    User,
    Builtin,
    Inline,
}

impl AgentSource {
    pub fn describe(self) -> &'static str {
        match self {
            AgentSource::Project => "project",
            AgentSource::User => "user",
            AgentSource::Builtin => "builtin",
            AgentSource::Inline => "inline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

impl SubagentMetadata {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
        }
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn model(mut self, model: Option<String>) -> Self {
        self.model = model;
        self
    }

    pub fn tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentSpec {
    pub metadata: SubagentMetadata,
    pub instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<PathBuf>,
    pub source: AgentSource,
    pub hash: String,
}

impl SubagentSpec {
    pub fn instructions_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }
}
