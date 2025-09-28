use crate::error::AgentParseError;
use crate::error::ParserError;
use crate::error::RegistryError;
use crate::parser::ParsedAgent;
use crate::parser::parse_agent_file;
use crate::spec::AgentSource;
use crate::spec::SubagentSpec;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub spec: SubagentSpec,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RegistrySnapshot {
    pub agents: Vec<AgentHandle>,
    pub parse_errors: Vec<AgentParseError>,
    pub generated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
struct CachedEntry {
    modified: Option<SystemTime>,
    spec: SubagentSpec,
    warnings: Vec<String>,
}

pub struct SubagentRegistry {
    project_dir: PathBuf,
    user_dir: PathBuf,
    cache: HashMap<PathBuf, CachedEntry>,
    agents: BTreeMap<String, AgentHandle>,
    parse_errors: Vec<AgentParseError>,
    last_snapshot: Option<RegistrySnapshot>,
}

impl SubagentRegistry {
    pub fn new(project_dir: impl Into<PathBuf>, user_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: project_dir.into(),
            user_dir: user_dir.into(),
            cache: HashMap::new(),
            agents: BTreeMap::new(),
            parse_errors: Vec::new(),
            last_snapshot: None,
        }
    }

    pub fn reload(&mut self) -> Result<&RegistrySnapshot, RegistryError> {
        let mut agents = BTreeMap::new();
        let mut cache = HashMap::new();
        let mut parse_errors = Vec::new();

        let user_dir = self.user_dir.clone();
        let project_dir = self.project_dir.clone();

        self.scan_dir(
            &user_dir,
            AgentSource::User,
            &mut agents,
            &mut cache,
            &mut parse_errors,
        )?;
        self.scan_dir(
            &project_dir,
            AgentSource::Project,
            &mut agents,
            &mut cache,
            &mut parse_errors,
        )?;

        self.agents = agents;
        self.cache = cache;
        self.parse_errors = parse_errors;

        let snapshot = RegistrySnapshot {
            agents: self.agents.values().cloned().collect(),
            parse_errors: self.parse_errors.clone(),
            generated_at: OffsetDateTime::now_utc(),
        };
        self.last_snapshot = Some(snapshot);
        Ok(self.last_snapshot.as_ref().expect("snapshot set"))
    }

    fn scan_dir(
        &mut self,
        dir: &Path,
        source: AgentSource,
        agents: &mut BTreeMap<String, AgentHandle>,
        cache: &mut HashMap<PathBuf, CachedEntry>,
        parse_errors: &mut Vec<AgentParseError>,
    ) -> Result<(), RegistryError> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                return Err(RegistryError::Io {
                    path: dir.to_path_buf(),
                    source: err,
                });
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    parse_errors.push(AgentParseError::new(
                        dir.to_path_buf(),
                        format!("failed to read directory entry: {err}"),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            if !is_agent_file(&path) {
                continue;
            }

            match self.load_agent(&path, source) {
                Ok(Some(parsed)) => {
                    let modified = get_modified_time(&path);
                    cache.insert(
                        path.clone(),
                        CachedEntry {
                            modified,
                            spec: parsed.spec.clone(),
                            warnings: parsed.warnings.clone(),
                        },
                    );
                    agents.insert(
                        parsed.spec.metadata.name.clone(),
                        AgentHandle {
                            spec: parsed.spec,
                            warnings: parsed.warnings,
                        },
                    );
                }
                Ok(None) => {}
                Err(RegistryError::Parse {
                    path: err_path,
                    source,
                }) => {
                    parse_errors.push(AgentParseError::new(err_path, format!("{source}")));
                }
                Err(other) => return Err(other),
            }
        }

        Ok(())
    }

    fn load_agent(
        &mut self,
        path: &Path,
        source: AgentSource,
    ) -> Result<Option<ParsedAgent>, RegistryError> {
        let modified = get_modified_time(path);
        if let Some(entry) = self.cache.get(path)
            && entry.modified == modified
        {
            return Ok(Some(ParsedAgent {
                spec: entry.spec.clone(),
                warnings: entry.warnings.clone(),
            }));
        }

        match parse_agent_file(path, source) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(ParserError::Io(err)) => Err(RegistryError::Io {
                path: path.to_path_buf(),
                source: err,
            }),
            Err(other) => Err(RegistryError::Parse {
                path: path.to_path_buf(),
                source: other,
            }),
        }
    }

    pub fn agents(&self) -> impl Iterator<Item = &AgentHandle> {
        self.agents.values()
    }

    pub fn get(&self, name: &str) -> Option<&AgentHandle> {
        self.agents.get(name)
    }

    pub fn parse_errors(&self) -> &[AgentParseError] {
        &self.parse_errors
    }

    pub fn snapshot(&self) -> Option<&RegistrySnapshot> {
        self.last_snapshot.as_ref()
    }
}

fn is_agent_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "md" | "markdown"))
        .unwrap_or(false)
}

fn get_modified_time(path: &Path) -> Option<SystemTime> {
    match fs::metadata(path) {
        Ok(metadata) => metadata.modified().ok(),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::write;

    #[test]
    fn loads_user_and_project_agents_with_override() {
        let temp = tempfile::tempdir().unwrap();
        let project_dir = temp.path().join("project");
        let user_dir = temp.path().join("user");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&user_dir).unwrap();

        let user_agent = r"---
name: reviewer
model: user-model
tools: [apply_patch]
---
User reviewer instructions.
";
        let project_agent = r"---
name: reviewer
model: project-model
tools: [apply_patch, git_diff]
---
Project reviewer instructions.
";
        write(user_dir.join("reviewer.md"), user_agent).unwrap();
        write(project_dir.join("reviewer.md"), project_agent).unwrap();

        let mut registry = SubagentRegistry::new(&project_dir, &user_dir);
        let snapshot = registry.reload().unwrap();
        assert_eq!(snapshot.agents.len(), 1);
        let agent = &snapshot.agents[0];
        assert_eq!(agent.spec.metadata.model.as_deref(), Some("project-model"));
        assert_eq!(agent.spec.metadata.tools, vec!["apply_patch", "git_diff"]);
        assert_eq!(agent.spec.source, AgentSource::Project);
    }
}
