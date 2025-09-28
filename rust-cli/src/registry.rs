use crate::agent::{AgentSource, AgentSpec};
use crate::config::Config;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentSpec>,
    config: Config,
    builtin_agents_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub struct LoadReport {
    pub loaded: usize,
    pub errors: Vec<LoadError>,
}

#[derive(Debug)]
pub struct LoadError {
    pub path: PathBuf,
    pub message: String,
}

impl AgentRegistry {
    /// Create a new agent registry with the given configuration
    pub async fn new(config: &Config) -> Result<Self> {
        let mut registry = Self {
            agents: HashMap::new(),
            config: config.clone(),
            builtin_agents_dir: Self::find_builtin_agents_dir(),
        };

        registry.reload().await?;
        Ok(registry)
    }

    /// Reload agents from all sources
    pub async fn reload(&mut self) -> Result<LoadReport> {
        self.agents.clear();
        let mut report = LoadReport {
            loaded: 0,
            errors: Vec::new(),
        };

        // Load user agents first (so they can override builtin ones)
        if let Err(e) = self.load_user_agents(&mut report).await {
            report.errors.push(LoadError {
                path: self.config.agents_dir.clone(),
                message: format!("Failed to load user agents: {}", e),
            });
        }

        // Load builtin agents
        if let Some(builtin_dir) = self.builtin_agents_dir.clone() {
            if let Err(e) = self.load_builtin_agents(&builtin_dir, &mut report).await {
                report.errors.push(LoadError {
                    path: builtin_dir.clone(),
                    message: format!("Failed to load builtin agents: {}", e),
                });
            }
        }

        Ok(report)
    }

    /// Get all loaded agents
    pub fn list_agents(&self) -> Vec<&AgentSpec> {
        let mut agents: Vec<&AgentSpec> = self.agents.values().collect();
        agents.sort_by_key(|a| &a.name);
        agents
    }

    /// Get a specific agent by name
    pub fn get_agent(&self, name: &str) -> Option<&AgentSpec> {
        self.agents.get(name)
    }

    /// Check if an agent exists
    pub fn has_agent(&self, name: &str) -> bool {
        self.agents.contains_key(name)
    }

    /// Create a new user agent
    pub async fn create_agent(&self, spec: &AgentSpec) -> Result<PathBuf> {
        // Ensure agents directory exists
        fs::create_dir_all(&self.config.agents_dir).await?;

        let file_name = format!("{}.md", spec.name);
        let agent_path = self.config.agents_dir.join(file_name);

        if agent_path.exists() {
            return Err(anyhow!("Agent '{}' already exists", spec.name));
        }

        let content = spec.to_markdown();
        fs::write(&agent_path, content).await?;

        Ok(agent_path)
    }

    /// Delete a user agent
    pub async fn delete_agent(&self, name: &str) -> Result<()> {
        let file_name = format!("{}.md", name);
        let agent_path = self.config.agents_dir.join(file_name);

        if !agent_path.exists() {
            return Err(anyhow!("Agent '{}' not found", name));
        }

        fs::remove_file(agent_path).await?;
        Ok(())
    }

    /// Get agent templates for creating new agents
    pub fn get_templates() -> HashMap<String, &'static str> {
        let mut templates = HashMap::new();

        templates.insert("basic".to_string(), include_str!("../templates/basic.md"));
        templates.insert("code-review".to_string(), include_str!("../templates/code-review.md"));
        templates.insert("docs".to_string(), include_str!("../templates/docs.md"));
        templates.insert("testing".to_string(), include_str!("../templates/testing.md"));

        templates
    }

    /// Create agent from template
    pub fn create_from_template(name: &str, template: &str, description: Option<String>) -> Result<AgentSpec> {
        let templates = Self::get_templates();
        let template_content = templates.get(template)
            .ok_or_else(|| anyhow!("Unknown template: {}", template))?;

        // Replace placeholders in template
        let content = template_content
            .replace("{{name}}", name)
            .replace("{{description}}", &description.unwrap_or_else(|| format!("A custom agent named {}", name)));

        AgentSpec::parse_document(&content, None, AgentSource::User)
    }

    /// Load user agents from the agents directory
    async fn load_user_agents(&mut self, report: &mut LoadReport) -> Result<()> {
        let agents_dir = self.config.agents_dir.clone();
        if !agents_dir.exists() {
            // Create the directory if it doesn't exist
            fs::create_dir_all(&agents_dir).await?;
            return Ok(());
        }

        self.load_agents_from_dir(&agents_dir, AgentSource::User, report).await
    }

    /// Load builtin agents from the builtin directory
    async fn load_builtin_agents(&mut self, builtin_dir: &Path, report: &mut LoadReport) -> Result<()> {
        self.load_agents_from_dir(builtin_dir, AgentSource::Builtin, report).await
    }

    /// Load agents from a specific directory
    async fn load_agents_from_dir(&mut self, dir: &Path, source: AgentSource, report: &mut LoadReport) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.load_agent_file(&path, source).await {
                    Ok(Some(agent)) => {
                        // Only add if not already present (user agents override builtin)
                        if !self.agents.contains_key(&agent.name) {
                            self.agents.insert(agent.name.clone(), agent);
                            report.loaded += 1;
                        }
                    }
                    Ok(None) => {
                        // File exists but couldn't be parsed
                        report.errors.push(LoadError {
                            path: path.clone(),
                            message: "Invalid agent file format".to_string(),
                        });
                    }
                    Err(e) => {
                        report.errors.push(LoadError {
                            path: path.clone(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single agent file
    async fn load_agent_file(&self, path: &Path, source: AgentSource) -> Result<Option<AgentSpec>> {
        let content = fs::read_to_string(path).await?;

        match AgentSpec::parse_document(&content, Some(path.to_path_buf()), source) {
            Ok(spec) => {
                // Validate the spec
                spec.validate()?;
                Ok(Some(spec))
            }
            Err(e) => Err(e)
        }
    }

    /// Find the builtin agents directory
    fn find_builtin_agents_dir() -> Option<PathBuf> {
        // Try to find agents directory relative to the binary
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;

        // Check various possible locations
        let candidates = [
            exe_dir.join("../agents"),              // npm package structure
            exe_dir.join("../../agents"),           // alternative structure
            exe_dir.join("agents"),                 // direct sibling
            PathBuf::from("./agents"),              // current directory
        ];

        for candidate in &candidates {
            if candidate.exists() && candidate.is_dir() {
                return Some(candidate.clone());
            }
        }

        None
    }
}