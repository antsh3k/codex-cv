use crate::agent::{AgentSource, AgentSpec};
use crate::config::Config;
use crate::registry::AgentRegistry;
use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Input, Select};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// List available agents
pub async fn list_agents(registry: &AgentRegistry, format: &str, show_all: bool) -> Result<()> {
    let agents = registry.list_agents();

    match format {
        "json" => {
            let json_output = json!({
                "agents": agents.iter().map(|agent| {
                    json!({
                        "name": agent.name,
                        "description": agent.description,
                        "model": agent.model,
                        "tools": agent.tools,
                        "keywords": agent.keywords,
                        "source": agent.source.to_string()
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            if agents.is_empty() {
                println!("{}", "No agents found.".yellow());
                println!("{} {}", "ℹ Run".blue(), "codex-subagents create <name>".cyan());
                println!("   to create your first agent");
                return Ok(());
            }

            println!("{}\n", "📋 Available Agents:".blue().bold());

            // Separate builtin and user agents
            let builtin_agents: Vec<_> = agents.iter().filter(|a| a.source == AgentSource::Builtin).collect();
            let user_agents: Vec<_> = agents.iter().filter(|a| a.source == AgentSource::User).collect();

            if !builtin_agents.is_empty() {
                println!("{}", "Built-in Agents:".blue());
                for agent in builtin_agents {
                    print_agent_info(agent);
                }
                println!();
            }

            if !user_agents.is_empty() {
                println!("{}", "Custom Agents:".blue());
                for agent in user_agents {
                    print_agent_info(agent);
                }
            }

            println!("{}", "\n💡 Usage:".blue());
            println!("   {}", "codex-subagents run <agent-name>".cyan());
            println!("   {}", "codex-subagents create <new-agent-name>".cyan());
        }
    }

    Ok(())
}

/// Print information about a single agent
fn print_agent_info(agent: &AgentSpec) {
    let name_color = match agent.source {
        AgentSource::Builtin => "cyan",
        AgentSource::User => "green",
    };
    let prefix = match agent.source {
        AgentSource::Builtin => "  📦",
        AgentSource::User => "  👤",
    };

    println!("{} {}", prefix, agent.name.color(name_color));

    if let Some(description) = agent.description() {
        println!("      {}", description.bright_black());
    }

    if !agent.tools.is_empty() {
        println!("      {}: {}", "Tools".blue(), agent.tools.join(", "));
    }

    if let Some(model) = agent.model() {
        println!("      {}: {}", "Model".blue(), model);
    }

    if !agent.keywords.is_empty() {
        println!("      {}: {}", "Keywords".blue(), agent.keywords.join(", "));
    }

    println!();
}

/// Run a specific agent
pub async fn run_agent(
    registry: &AgentRegistry,
    config: &Config,
    agent_name: &str,
    prompt: Option<String>,
    timeout_seconds: u64,
) -> Result<()> {
    let agent = registry.get_agent(agent_name)
        .ok_or_else(|| anyhow!("Agent '{}' not found", agent_name))?;

    println!("{} {}", "🚀 Running agent:".blue(), agent_name.cyan());

    // Get prompt if not provided
    let prompt = match prompt {
        Some(p) => p,
        None => {
            let prompt_input: String = Input::new()
                .with_prompt("Enter your prompt")
                .interact_text()?;
            prompt_input
        }
    };

    if prompt.trim().is_empty() {
        return Err(anyhow!("Prompt cannot be empty"));
    }

    println!("{} {}", "📝 Prompt:".blue(), prompt.bright_black());
    println!("{} {}", "🤖 Model:".blue(), agent.model().unwrap_or(&config.ai.model).bright_black());
    println!();

    // Validate API key
    if config.ai.api_key.is_empty() {
        return Err(anyhow!("No API key configured. Set OPENAI_API_KEY environment variable or run 'codex-subagents init'"));
    }

    // For now, simulate agent execution
    // In a full implementation, this would make actual API calls
    println!("{}", "⏳ Executing agent...".yellow());

    let result = timeout(
        Duration::from_secs(timeout_seconds),
        simulate_agent_execution(agent, &prompt, config)
    ).await;

    match result {
        Ok(Ok(response)) => {
            println!("{}\n", "✅ Agent completed successfully:".green());
            println!("{}", response);
        }
        Ok(Err(e)) => {
            eprintln!("{} {}", "❌ Agent execution failed:".red(), e);
            return Err(e);
        }
        Err(_) => {
            eprintln!("{} Agent execution timed out after {} seconds", "⏱️".yellow(), timeout_seconds);
            return Err(anyhow!("Agent execution timeout"));
        }
    }

    Ok(())
}

/// Show status of configuration and agents
pub async fn show_status(registry: &AgentRegistry, config: &Config) -> Result<()> {
    println!("{}", "📊 Codex Subagents Status".blue().bold());
    println!();

    // Configuration status
    println!("{}", "⚙️ Configuration:".blue());
    println!("   Config dir: {}", config.config_dir.display().to_string().bright_black());
    println!("   Agents dir: {}", config.agents_dir.display().to_string().bright_black());
    println!("   AI Provider: {}", config.ai.provider.bright_black());
    println!("   Model: {}", config.ai.model.bright_black());
    println!("   API Key: {}", if config.ai.api_key.is_empty() { "❌ Not configured".red() } else { "✅ Configured".green() });
    println!();

    // Agent status
    let agents = registry.list_agents();
    println!("{}", "🤖 Agents:".blue());
    println!("   Total agents: {}", agents.len().to_string().bright_black());

    let builtin_count = agents.iter().filter(|a| a.source == AgentSource::Builtin).count();
    let user_count = agents.iter().filter(|a| a.source == AgentSource::User).count();

    println!("   Built-in: {}", builtin_count.to_string().bright_black());
    println!("   Custom: {}", user_count.to_string().bright_black());
    println!();

    // Validation
    let issues = config.validate()?;
    if issues.is_empty() {
        println!("{}", "✅ All checks passed".green());
    } else {
        println!("{}", "⚠️ Issues found:".yellow());
        for issue in issues {
            println!("   - {}", issue.yellow());
        }
    }

    Ok(())
}

/// Create a new agent
pub async fn create_agent(
    registry: &AgentRegistry,
    name: &str,
    template: &str,
    description: Option<String>,
) -> Result<()> {
    // Validate name
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow!("Agent name must contain only alphanumeric characters, hyphens, and underscores"));
    }

    if registry.has_agent(name) {
        return Err(anyhow!("Agent '{}' already exists", name));
    }

    println!("{} {}", "🛠️ Creating agent:".blue(), name.cyan());

    // Create agent from template
    let agent = AgentRegistry::create_from_template(name, template, description)?;

    // Save to file
    let path = registry.create_agent(&agent).await?;

    println!("{} {}", "✅ Agent created:".green(), path.display().to_string().bright_black());
    println!();
    println!("{}", "📝 Next steps:".blue());
    println!("   1. Edit the agent file to customize its behavior");
    println!("   2. Test with: {}", format!("codex-subagents run {}", name).cyan());

    Ok(())
}

/// Initialize configuration
pub async fn initialize_config(force: bool) -> Result<()> {
    let config_dir = Config::get_config_dir()?;
    let config_file = config_dir.join("config.yaml");

    if config_file.exists() && !force {
        return Err(anyhow!("Configuration already exists. Use --force to overwrite."));
    }

    println!("{}", "🔧 Initializing configuration...".blue());

    // Create default configuration
    let config = Config::load(None).await?;
    config.save().await?;

    println!("{} {}", "✅ Configuration created:".green(), config_file.display().to_string().bright_black());
    println!("   Agents directory: {}", config.agents_dir.display().to_string().bright_black());
    println!();

    // Check for API key
    if config.ai.api_key.is_empty() {
        println!("{}", "⚠️ No API key found.".yellow());
        println!("   Set your OpenAI API key with:");
        println!("   {}", "export OPENAI_API_KEY=\"your-key-here\"".cyan());
    } else {
        println!("{}", "✅ API key configured".green());
    }

    println!();
    println!("{}", "🚀 Ready to use! Try:".blue());
    println!("   {}", "codex-subagents list".cyan());
    println!("   {}", "codex-subagents create my-agent".cyan());

    Ok(())
}

/// Check health of the installation
pub async fn check_health(registry: &AgentRegistry, config: &Config) -> Result<()> {
    println!("{}", "🏥 Health Check".blue().bold());
    println!();

    let mut all_good = true;

    // Check configuration
    println!("{}", "⚙️ Configuration:".blue());
    let issues = config.validate()?;
    if issues.is_empty() {
        println!("   {}", "✅ Configuration is valid".green());
    } else {
        all_good = false;
        for issue in issues {
            println!("   {} {}", "❌".red(), issue);
        }
    }

    // Check directories
    println!("   {}", "📁 Directories:".blue());
    if config.config_dir.exists() {
        println!("     {} Config directory exists", "✅".green());
    } else {
        println!("     {} Config directory missing", "❌".red());
        all_good = false;
    }

    if config.agents_dir.exists() {
        println!("     {} Agents directory exists", "✅".green());
    } else {
        println!("     {} Agents directory missing", "❌".red());
        all_good = false;
    }

    // Check agents
    println!();
    println!("{}", "🤖 Agents:".blue());
    let agents = registry.list_agents();
    if agents.is_empty() {
        println!("   {} No agents found (this is OK for new installations)", "⚠️".yellow());
    } else {
        println!("   {} {} agents loaded successfully", "✅".green(), agents.len());
    }

    // Check API connectivity (simplified)
    println!();
    println!("{}", "🌐 API Connectivity:".blue());
    if config.ai.api_key.is_empty() {
        println!("   {} No API key configured", "❌".red());
        all_good = false;
    } else {
        println!("   {} API key configured", "✅".green());
        // In a real implementation, you would test the API connection here
        println!("   {} API connectivity not tested (would require actual API call)", "ℹ️".blue());
    }

    println!();
    if all_good {
        println!("{}", "🎉 Everything looks good!".green().bold());
    } else {
        println!("{}", "⚠️ Some issues found. Please address them for optimal functionality.".yellow());
    }

    Ok(())
}

/// Simulate agent execution (placeholder for actual implementation)
async fn simulate_agent_execution(agent: &AgentSpec, prompt: &str, _config: &Config) -> Result<String> {
    // In a real implementation, this would:
    // 1. Prepare the conversation context with agent instructions
    // 2. Make API calls to the configured AI service
    // 3. Handle tool calls if the agent uses tools
    // 4. Return the actual response

    // For now, return a simulated response
    tokio::time::sleep(Duration::from_millis(1500)).await;

    Ok(format!(
        "Agent '{}' executed successfully.\n\nPrompt: {}\n\nInstructions:\n{}\n\n[This is a simulated response. In the full implementation, this would be the actual AI response.]",
        agent.name,
        prompt,
        agent.instructions
    ))
}