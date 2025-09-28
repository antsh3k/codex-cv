use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

mod agent;
mod config;
mod registry;
mod cli;

use agent::AgentSpec;
use config::Config;
use registry::AgentRegistry;

#[derive(Parser)]
#[command(name = "codex-subagents")]
#[command(about = "AI-powered specialist agents for coding tasks")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// List available subagent definitions
    List {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show all agents including builtin
        #[arg(long)]
        all: bool,
    },

    /// Run a subagent with the given prompt
    Run {
        /// Name of the agent to run
        agent: String,

        /// Prompt to send to the agent
        #[arg(short, long)]
        prompt: Option<String>,

        /// Timeout in seconds
        #[arg(short, long, default_value = "300")]
        timeout: u64,
    },

    /// Show status of agents and configuration
    Status,

    /// Create a new custom agent
    Create {
        /// Name of the new agent
        name: String,

        /// Agent template to use
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Description for the agent
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Initialize configuration and setup
    Init {
        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Check installation and configuration health
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("codex_subagents={}", log_level))
        .init();

    // Load configuration
    let config = Config::load(cli.config.as_deref()).await?;

    // Initialize registry
    let registry = AgentRegistry::new(&config).await?;

    match cli.command {
        Commands::List { format, all } => {
            cli::list_agents(&registry, &format, all).await?;
        }

        Commands::Run { agent, prompt, timeout } => {
            cli::run_agent(&registry, &config, &agent, prompt, timeout).await?;
        }

        Commands::Status => {
            cli::show_status(&registry, &config).await?;
        }

        Commands::Create { name, template, description } => {
            cli::create_agent(&registry, &name, &template, description).await?;
        }

        Commands::Init { force } => {
            cli::initialize_config(force).await?;
        }

        Commands::Doctor => {
            cli::check_health(&registry, &config).await?;
        }
    }

    Ok(())
}