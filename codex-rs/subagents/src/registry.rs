use crate::error::SubagentError;
use crate::error::SubagentResult;
use crate::parser::parse_document;
use crate::spec::SubagentSpec;
use indexmap::IndexMap;
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use walkdir::WalkDir;

static DEFAULT_HOME: Lazy<PathBuf> = Lazy::new(|| PathBuf::from(".codex/agents"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSource {
    Project,
    User,
}

impl fmt::Display for AgentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentSource::Project => write!(f, "project"),
            AgentSource::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubagentRecord {
    pub spec: Arc<SubagentSpec>,
    pub source: AgentSource,
}

impl SubagentRecord {
    pub fn name(&self) -> &str {
        self.spec.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubagentRegistryError {
    pub path: PathBuf,
    pub message: String,
}

impl SubagentRegistryError {
    fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReloadReport {
    pub loaded: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<SubagentRegistryError>,
}

pub struct SubagentRegistry {
    project_dir: PathBuf,
    user_dir: PathBuf,
    cache: HashMap<PathBuf, CachedDoc>,
    agents: IndexMap<String, SubagentRecord>,
    last_errors: Vec<SubagentRegistryError>,
}

impl SubagentRegistry {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let project_dir = project_root.join(".codex/agents");
        let user_dir = dirs::home_dir()
            .map(|h| h.join(".codex/agents"))
            .unwrap_or_else(|| DEFAULT_HOME.clone());
        Self::with_directories(project_dir, user_dir)
    }

    pub fn with_directories(project_dir: PathBuf, user_dir: PathBuf) -> Self {
        Self {
            project_dir,
            user_dir,
            cache: HashMap::new(),
            agents: IndexMap::new(),
            last_errors: Vec::new(),
        }
    }

    pub fn reload(&mut self) -> ReloadReport {
        let mut report = ReloadReport::default();
        let mut new_cache = HashMap::new();
        let mut resolved: IndexMap<String, SubagentRecord> = IndexMap::new();

        let old_names: IndexSet<String> = self.agents.keys().cloned().collect();

        for (source, dir) in [
            (AgentSource::User, &self.user_dir),
            (AgentSource::Project, &self.project_dir),
        ] {
            if !dir.exists() {
                continue;
            }

            let walker = WalkDir::new(dir).max_depth(1).follow_links(false);
            for entry in walker.into_iter() {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        report
                            .errors
                            .push(SubagentRegistryError::new(dir, format!("{err}")));
                        continue;
                    }
                };

                if entry.depth() == 0 || !entry.file_type().is_file() {
                    continue;
                }

                if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                let path = entry.path().to_path_buf();
                let metadata = match entry.metadata() {
                    Ok(meta) => meta,
                    Err(err) => {
                        let error = SubagentRegistryError::new(&path, err.to_string());
                        new_cache.insert(
                            path.clone(),
                            CachedDoc::failure(None, source, error.clone()),
                        );
                        report.errors.push(error);
                        continue;
                    }
                };

                let modified = metadata.modified().ok();
                let cached = self.cache.get(&path);
                let use_cached = cached
                    .filter(|doc| doc.modified == modified && doc.source == source)
                    .cloned();

                let mut doc = if let Some(doc) = use_cached {
                    doc
                } else {
                    let parsed = load_document(&path);
                    let mut cached_doc = match parsed {
                        Ok(spec) => {
                            report.loaded.push(spec.name().to_string());
                            CachedDoc::success(modified, source, spec)
                        }
                        Err(err) => {
                            let error = registry_error_from(err, &path);
                            report.errors.push(error.clone());
                            CachedDoc::failure(modified, source, error)
                        }
                    };
                    cached_doc.modified = modified;
                    cached_doc
                };

                // Always use the canonical path in cache key
                doc.modified = modified;
                new_cache.insert(path.clone(), doc.clone());

                if let CachedParsed::Success(spec) = &doc.parsed {
                    resolved.insert(
                        spec.name().to_string(),
                        SubagentRecord {
                            spec: spec.clone(),
                            source,
                        },
                    );
                }
            }
        }

        let new_names: IndexSet<String> = resolved.keys().cloned().collect();
        for name in &old_names {
            if !new_names.contains(name) {
                report.removed.push(name.clone());
            }
        }

        // Deduplicate errors
        let mut error_set: IndexSet<SubagentRegistryError> = IndexSet::new();
        for doc in new_cache.values() {
            if let CachedParsed::Failure(err) = &doc.parsed {
                error_set.insert(err.clone());
            }
        }
        for err in report.errors.drain(..) {
            error_set.insert(err);
        }

        report.errors = error_set.into_iter().collect();

        let mut loaded_set: IndexSet<String> = IndexSet::new();
        for name in report.loaded.drain(..) {
            loaded_set.insert(name);
        }
        report.loaded = loaded_set.into_iter().collect();

        let mut removed_set: IndexSet<String> = IndexSet::new();
        for name in report.removed.drain(..) {
            removed_set.insert(name);
        }
        report.removed = removed_set.into_iter().collect();

        self.cache = new_cache;
        self.agents = resolved;
        self.last_errors = report.errors.clone();

        report
    }

    pub fn list(&self) -> Vec<SubagentRecord> {
        self.agents.values().cloned().collect()
    }

    pub fn get(&self, name: &str) -> SubagentResult<SubagentRecord> {
        self.agents
            .get(name)
            .cloned()
            .ok_or_else(|| SubagentError::NotFound(name.to_string()))
    }

    pub fn last_errors(&self) -> &[SubagentRegistryError] {
        &self.last_errors
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    pub fn user_dir(&self) -> &Path {
        &self.user_dir
    }
}

#[derive(Clone)]
struct CachedDoc {
    modified: Option<SystemTime>,
    source: AgentSource,
    parsed: CachedParsed,
}

#[derive(Clone)]
enum CachedParsed {
    Success(Arc<SubagentSpec>),
    Failure(SubagentRegistryError),
}

impl CachedDoc {
    fn success(modified: Option<SystemTime>, source: AgentSource, spec: SubagentSpec) -> Self {
        Self {
            modified,
            source,
            parsed: CachedParsed::Success(Arc::new(spec)),
        }
    }

    fn failure(
        modified: Option<SystemTime>,
        source: AgentSource,
        error: SubagentRegistryError,
    ) -> Self {
        Self {
            modified,
            source,
            parsed: CachedParsed::Failure(error),
        }
    }
}

fn load_document(path: &Path) -> SubagentResult<SubagentSpec> {
    let contents = fs::read_to_string(path).map_err(|err| SubagentError::Io {
        path: path.to_path_buf(),
        source: err,
    })?;
    parse_document(&contents, Some(path.to_path_buf()))
}

fn registry_error_from(err: SubagentError, fallback: &Path) -> SubagentRegistryError {
    match err {
        SubagentError::Io { path, source } => SubagentRegistryError::new(path, source.to_string()),
        SubagentError::FrontMatter { path, source } => {
            SubagentRegistryError::new(path, source.to_string())
        }
        SubagentError::Registry { path, message } => SubagentRegistryError::new(path, message),
        SubagentError::InvalidSpec(message) => {
            SubagentRegistryError::new(fallback.to_path_buf(), message)
        }
        SubagentError::MissingField { field } => {
            SubagentRegistryError::new(fallback.to_path_buf(), format!("missing field `{field}`"))
        }
        SubagentError::NotFound(name) => SubagentRegistryError::new(
            fallback.to_path_buf(),
            format!("subagent `{name}` not found"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    fn write_agent(dir: &Path, name: &str, yaml: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{name}.md"));
        let doc = format!("---\n{yaml}\n---\n{body}\n");
        fs::write(&path, doc).unwrap();
        path
    }

    #[test]
    fn registry_prefers_project_over_user() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_agents_root = TempDir::new().unwrap();
        let user_agents = user_agents_root.path().to_path_buf();

        write_agent(
            &user_agents,
            "code-reviewer",
            "name: code-reviewer\ndescription: user",
            "User instructions",
        );
        write_agent(
            &project_agents,
            "code-reviewer",
            "name: code-reviewer\ndescription: project",
            "Project instructions",
        );

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();
        assert!(report.errors.is_empty());
        let record = registry.get("code-reviewer").unwrap();
        assert_eq!(record.source, AgentSource::Project);
        assert_eq!(record.spec.description(), Some("project"));
        assert_eq!(record.spec.instructions(), "Project instructions");
    }

    #[test]
    fn registry_reports_parse_errors() {
        let project_root = TempDir::new().unwrap();
        let project_agents = project_root.path().join(".codex/agents");
        let user_agents_root = TempDir::new().unwrap();
        let user_agents = user_agents_root.path().to_path_buf();

        fs::create_dir_all(&project_agents).unwrap();
        fs::write(project_agents.join("broken.md"), "not yaml").unwrap();

        let mut registry = SubagentRegistry::with_directories(project_agents, user_agents);
        let report = registry.reload();
        assert_eq!(report.loaded, Vec::<String>::new());
        assert_eq!(report.removed, Vec::<String>::new());
        assert_eq!(report.errors.len(), 1);
    }
}
