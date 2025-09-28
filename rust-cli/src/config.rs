use anyhow::{anyhow, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub ai: AiConfig,
    pub agents: AgentsConfig,
    pub output: OutputConfig,
    pub git: GitConfig,
    pub security: SecurityConfig,

    // Runtime paths
    #[serde(skip)]
    pub config_dir: PathBuf,
    #[serde(skip)]
    pub agents_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub timeout: u64,
    pub max_retries: u32,
    pub auto_save: bool,
    pub default_tools: Vec<String>,
    pub allow_custom_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub verbose: bool,
    pub colors: bool,
    pub show_progress: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_detect_repo: bool,
    pub require_clean_working: bool,
    pub create_backups: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub require_confirmation: Vec<String>,
}

impl Config {
    /// Load configuration from file or create default
    pub async fn load(config_path: Option<&Path>) -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_file = config_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| config_dir.join("config.yaml"));

        if config_file.exists() {
            let contents = fs::read_to_string(&config_file).await?;
            let mut config: Config = serde_yaml::from_str(&contents)?;

            // Set runtime paths
            config.config_dir = config_dir.clone();
            config.agents_dir = config_dir.join("agents");

            // Merge environment variables
            config.merge_env_vars();

            Ok(config)
        } else {
            // Create default configuration
            let config = Self::default_config(config_dir)?;
            config.save().await?;
            Ok(config)
        }
    }

    /// Get the configuration directory path
    pub fn get_config_dir() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        Ok(home.join(".codex-subagents"))
    }

    /// Get the agents directory path
    pub fn get_agents_dir(&self) -> &PathBuf {
        &self.agents_dir
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir).await?;
        fs::create_dir_all(&self.agents_dir).await?;

        let config_file = self.config_dir.join("config.yaml");
        let yaml = serde_yaml::to_string(self)?;
        fs::write(config_file, yaml).await?;

        Ok(())
    }

    /// Create default configuration
    fn default_config(config_dir: PathBuf) -> Result<Self> {
        let agents_dir = config_dir.join("agents");

        Ok(Config {
            version: "1.0.0".to_string(),
            ai: AiConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                base_url: std::env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                temperature: 0.1,
                max_tokens: 4000,
            },
            agents: AgentsConfig {
                timeout: 300,
                max_retries: 2,
                auto_save: true,
                default_tools: vec![
                    "git".to_string(),
                    "node".to_string(),
                    "npm".to_string(),
                ],
                allow_custom_tools: true,
            },
            output: OutputConfig {
                format: "text".to_string(),
                verbose: false,
                colors: true,
                show_progress: true,
                log_level: "info".to_string(),
            },
            git: GitConfig {
                auto_detect_repo: true,
                require_clean_working: false,
                create_backups: true,
            },
            security: SecurityConfig {
                allowed_commands: vec![
                    "git".to_string(),
                    "npm".to_string(),
                    "node".to_string(),
                    "cargo".to_string(),
                    "python3".to_string(),
                    "python".to_string(),
                ],
                blocked_commands: vec![
                    "rm".to_string(),
                    "sudo".to_string(),
                    "chmod".to_string(),
                    "chown".to_string(),
                ],
                require_confirmation: vec![
                    "git push".to_string(),
                    "git reset --hard".to_string(),
                    "rm -rf".to_string(),
                ],
            },
            config_dir,
            agents_dir,
        })
    }

    /// Merge environment variables into configuration
    fn merge_env_vars(&mut self) {
        // Override API key from environment
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                self.ai.api_key = api_key;
            }
        }

        // Override base URL from environment
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            if !base_url.is_empty() {
                self.ai.base_url = base_url;
            }
        }

        // Override model from environment
        if let Ok(model) = std::env::var("CODEX_MODEL") {
            if !model.is_empty() {
                self.ai.model = model;
            }
        }

        // Override verbose from environment
        if let Ok(_) = std::env::var("CODEX_VERBOSE") {
            self.output.verbose = true;
        }
    }

    /// Check if configuration is valid
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check API key
        if self.ai.api_key.is_empty() {
            issues.push("No API key configured. Set OPENAI_API_KEY environment variable or update config.".to_string());
        }

        // Check model format
        if self.ai.model.is_empty() {
            issues.push("AI model is not configured".to_string());
        }

        // Check paths exist
        if !self.config_dir.exists() {
            issues.push(format!("Configuration directory does not exist: {}", self.config_dir.display()));
        }

        if !self.agents_dir.exists() {
            issues.push(format!("Agents directory does not exist: {}", self.agents_dir.display()));
        }

        Ok(issues)
    }

    /// Get a configuration value by dot-separated path
    pub fn get_value(&self, path: &str) -> Option<serde_yaml::Value> {
        let config_value = serde_yaml::to_value(self).ok()?;

        let keys: Vec<&str> = path.split('.').collect();
        let mut current = &config_value;

        for key in keys {
            current = current.get(key)?;
        }

        Some(current.clone())
    }

    /// Set a configuration value by dot-separated path
    pub async fn set_value(&mut self, path: &str, value: serde_yaml::Value) -> Result<()> {
        // This is a simplified implementation - in practice you'd want more robust path handling
        match path {
            "ai.model" => {
                if let Some(model) = value.as_str() {
                    self.ai.model = model.to_string();
                }
            }
            "ai.temperature" => {
                if let Some(temp) = value.as_f64() {
                    self.ai.temperature = temp as f32;
                }
            }
            "output.verbose" => {
                if let Some(verbose) = value.as_bool() {
                    self.output.verbose = verbose;
                }
            }
            _ => return Err(anyhow!("Unsupported configuration path: {}", path)),
        }

        self.save().await?;
        Ok(())
    }
}