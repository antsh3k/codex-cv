use std::path::PathBuf;

use thiserror::Error;

pub type SubagentResult<T, E = SubagentError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum SubagentError {
    #[error("invalid subagent spec: {0}")]
    InvalidSpec(String),

    #[error("missing required field `{field}`")]
    MissingField { field: String },

    #[error("I/O error while reading {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse front matter in {path:?}: {source}")]
    FrontMatter {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("registry error for {path:?}: {message}")]
    Registry { path: PathBuf, message: String },

    #[error("unknown subagent `{0}`")]
    NotFound(String),
}

impl SubagentError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn registry(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Registry {
            path: path.into(),
            message: message.into(),
        }
    }
}
