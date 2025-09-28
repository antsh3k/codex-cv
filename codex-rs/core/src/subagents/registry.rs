use crate::config::Config;
use codex_subagents::ReloadReport;
use codex_subagents::SubagentRecord;
use codex_subagents::SubagentRegistry;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tokio::sync::RwLockReadGuard;

use super::SubagentIntegrationError;
use super::SubagentResult;

/// Thread-safe wrapper around the shared [`SubagentRegistry`].
pub struct CoreSubagentRegistry {
    inner: RwLock<SubagentRegistry>,
}

impl CoreSubagentRegistry {
    /// Create a registry using the given project root.
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let registry = SubagentRegistry::new(project_root);
        Self {
            inner: RwLock::new(registry),
        }
    }

    /// Create a registry using explicit project/user agent directories.
    pub fn with_directories(project_dir: PathBuf, user_dir: PathBuf) -> Self {
        let registry = SubagentRegistry::with_directories(project_dir, user_dir);
        Self {
            inner: RwLock::new(registry),
        }
    }

    /// Construct a registry from a resolved configuration.
    pub fn from_config(config: &Config) -> Self {
        // The registry expects the project root; re-use the cwd from configuration.
        Self::new(config.cwd.clone())
    }

    /// Acquire a read guard for the underlying registry.
    pub async fn read(&self) -> RwLockReadGuard<'_, SubagentRegistry> {
        self.inner.read().await
    }

    /// Reload agent definitions from disk, returning a summary report.
    pub async fn reload(&self) -> ReloadReport {
        let mut guard = self.inner.write().await;
        guard.reload()
    }

    /// Convenience helper to fetch an agent by name.
    pub async fn get(&self, name: &str) -> SubagentResult<SubagentRecord> {
        let guard = self.read().await;
        guard
            .get(name)
            .map_err(|_| SubagentIntegrationError::UnknownAgent(name.to_string()))
    }

    /// List all registered agents ordered by precedence.
    pub async fn list(&self) -> Vec<SubagentRecord> {
        self.read().await.list()
    }

    /// Return the last known load errors.
    pub async fn last_errors(&self) -> Vec<codex_subagents::SubagentRegistryError> {
        self.read().await.last_errors().to_vec()
    }
}
