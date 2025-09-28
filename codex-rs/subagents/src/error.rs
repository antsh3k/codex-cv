use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubagentValidationError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid agent name `{name}`: {reason}")]
    InvalidName { name: String, reason: String },
    #[error("duplicate tool entry `{0}`")]
    DuplicateTool(String),
    #[error("duplicate keyword entry `{0}`")]
    DuplicateKeyword(String),
    #[error("tools must be non-empty strings")]
    EmptyTool,
    #[error("keywords must be non-empty strings")]
    EmptyKeyword,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("no YAML frontmatter block found")]
    MissingFrontmatter,
    #[error("failed to parse YAML frontmatter: {0}")]
    InvalidFrontmatter(serde_yaml::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Validation(#[from] SubagentValidationError),
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("failed to read {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("failed to parse {path}: {source}")]
    Parse { path: PathBuf, source: ParserError },
}

#[derive(Debug, Error)]
pub enum TaskContextError {
    #[error("slot downcast failed for type `{expected}`")]
    SlotDowncast { expected: &'static str },
    #[error("internal state poisoned")]
    Poisoned,
    #[error("failed to serialize debug snapshot: {0}")]
    Serialization(serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct AgentParseError {
    pub path: PathBuf,
    pub message: String,
}

impl AgentParseError {
    pub fn new(path: PathBuf, message: impl Into<String>) -> Self {
        Self {
            path,
            message: message.into(),
        }
    }
}
