use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::anyhow;
use clap::Parser;
use clap::Subcommand;
use codex_common::CliConfigOverrides;
use codex_core::AuthManager;
use codex_core::ConversationManager;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::protocol::EventMsg;
use codex_core::protocol::SubAgentOutcome;
use codex_core::subagents::SubagentInvocation;
use codex_core::subagents::SubagentOrchestrator;
use codex_subagents::RegistrySnapshot;
use codex_subagents::SubagentRegistry;
use owo_colors::OwoColorize;

#[derive(Debug, Parser)]
pub(crate) struct SubagentsCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub command: SubagentsCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SubagentsCommand {
    /// List discovered subagents from the project and user registries.
    List,

    /// Run a subagent manually with an optional prompt payload.
    Run {
        #[arg(value_name = "NAME")]
        name: String,

        /// Optional prompt text forwarded to the subagent.
        #[arg(long = "prompt", value_name = "TEXT")]
        prompt: Option<String>,
    },
}

impl SubagentsCli {
    pub(crate) async fn run(self) -> anyhow::Result<()> {
        let config = load_config(&self.config_overrides)?;
        let project_agents = config.cwd.join(".codex/agents");
        let user_agents = config.codex_home.join("agents");

        match self.command {
            SubagentsCommand::List => {
                let snapshot = load_snapshot(project_agents, user_agents)?;
                render_snapshot(&snapshot);
                Ok(())
            }
            SubagentsCommand::Run { name, prompt } => {
                if !config.subagents.enabled {
                    anyhow::bail!(
                        "Subagents feature is disabled in this configuration. Enable `subagents.enabled` to run subagents."
                    );
                }

                let snapshot = load_snapshot(project_agents, user_agents)?;
                let handle = snapshot
                    .agents
                    .iter()
                    .find(|agent| agent.spec.metadata.name.eq_ignore_ascii_case(&name))
                    .ok_or_else(|| anyhow!("Subagent '{}' not found.", name))?;

                let auth_manager = AuthManager::shared(config.codex_home.clone());
                let conversation_manager = Arc::new(ConversationManager::new(auth_manager));
                let orchestrator = SubagentOrchestrator::new(conversation_manager);
                let spec = handle.spec.clone();
                let agent_display = spec.metadata.name.clone();

                println!(
                    "{} Starting subagent {}",
                    "→".cyan(),
                    agent_display.cyan().bold()
                );

                let run_state = orchestrator
                    .run_subagent(
                        &config,
                        SubagentInvocation {
                            spec: &spec,
                            parent_submit_id: format!("cli-subagent-{agent_display}"),
                        },
                        prompt,
                        |msg| match msg {
                            EventMsg::SubAgentStarted(ev) => {
                                let runtime_model =
                                    describe_model(&spec.metadata, ev.model.as_deref());
                                println!(
                                    "  {} {}",
                                    "started".dimmed(),
                                    format!("model: {runtime_model}").dimmed()
                                );
                            }
                            EventMsg::SubAgentMessage(ev) => {
                                for (idx, line) in ev.message.lines().enumerate() {
                                    if idx == 0 {
                                        println!("  {line}");
                                    } else {
                                        println!("    {line}");
                                    }
                                }
                            }
                            EventMsg::SubAgentCompleted(ev) => match ev.outcome {
                                SubAgentOutcome::Success => {
                                    let mut message = "Subagent completed successfully".to_string();
                                    if let Some(ms) = ev.duration_ms {
                                        message.push_str(&format!(" in {}", format_duration(ms)));
                                    }
                                    println!("{} {}", "✓".green(), message.green());
                                }
                                SubAgentOutcome::Error => {
                                    let mut base = "Subagent failed".to_string();
                                    if let Some(ms) = ev.duration_ms {
                                        base.push_str(&format!(" after {}", format_duration(ms)));
                                    }
                                    if let Some(message) = ev.error.as_ref() {
                                        println!(
                                            "{} {}",
                                            "✗".red(),
                                            format!("{base}: {message}").red()
                                        );
                                    } else {
                                        println!("{} {}", "✗".red(), base.red());
                                    }
                                }
                            },
                            EventMsg::Error(err) => {
                                println!(
                                    "{} {}",
                                    "✗".red(),
                                    format!("Subagent error: {}", err.message).red()
                                );
                            }
                            EventMsg::StreamError(stream_err) => {
                                println!(
                                    "{} {}",
                                    "!".magenta(),
                                    format!("Stream warning: {}", stream_err.message).magenta()
                                );
                            }
                            _ => {}
                        },
                    )
                    .await?;

                let duration_ms = run_state.duration.as_millis().min(u128::from(u64::MAX)) as u64;
                println!(
                    "{}",
                    format!("Duration: {}", format_duration(duration_ms)).dimmed()
                );

                match run_state.outcome {
                    SubAgentOutcome::Error => {
                        let detail = run_state
                            .error
                            .or(run_state.last_message.clone())
                            .unwrap_or_else(|| "unknown error".to_string());
                        Err(anyhow!(detail))
                    }
                    SubAgentOutcome::Success => {
                        if let Some(message) = run_state.last_message {
                            println!("{}", format!("Last message: {message}").dimmed());
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}

fn load_config(overrides: &CliConfigOverrides) -> anyhow::Result<Config> {
    let cli_overrides = overrides
        .parse_overrides()
        .map_err(|err| anyhow::anyhow!("failed to parse -c overrides: {err}"))?;
    let config_overrides = ConfigOverrides::default();
    Config::load_with_cli_overrides(cli_overrides, config_overrides)
        .context("failed to load Codex configuration")
}

fn load_snapshot(
    project_agents: PathBuf,
    user_agents: PathBuf,
) -> anyhow::Result<RegistrySnapshot> {
    let mut registry = SubagentRegistry::new(project_agents, user_agents);
    let snapshot = registry
        .reload()
        .context("failed to load subagent registry")?;
    Ok(snapshot.clone())
}

fn format_duration(ms: u64) -> String {
    if ms >= 60_000 {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1_000;
        let millis = ms % 1_000;
        if millis == 0 {
            format!("{minutes}m {seconds:02}s")
        } else {
            format!("{minutes}m {seconds:02}.{millis:03}s")
        }
    } else if ms >= 1_000 {
        if ms.is_multiple_of(1_000) {
            format!("{}s", ms / 1_000)
        } else {
            format!("{:.1}s", (ms as f64) / 1_000.0)
        }
    } else {
        format!("{ms}ms")
    }
}

fn describe_model(
    metadata: &codex_subagents::SubagentMetadata,
    runtime_model: Option<&str>,
) -> String {
    let provider = metadata
        .model_config
        .as_ref()
        .and_then(|binding| binding.provider_id.as_deref());
    let endpoint = metadata
        .model_config
        .as_ref()
        .and_then(|binding| binding.endpoint.as_deref());
    let chosen_model = runtime_model.or(metadata.model.as_deref());

    let mut summary = match (provider, chosen_model) {
        (Some(provider), Some(model)) => format!("{provider}/{model}"),
        (Some(provider), None) => format!("{provider} (session default)"),
        (None, Some(model)) => model.to_string(),
        (None, None) => "<session default>".to_string(),
    };

    if let Some(endpoint) = endpoint {
        summary = format!("{summary} @ {endpoint}");
    }

    summary
}

fn render_snapshot(snapshot: &RegistrySnapshot) {
    if snapshot.agents.is_empty() {
        println!("{}", "No subagents found.".yellow());
    } else {
        println!(
            "{}",
            format!("Discovered {} subagent(s):", snapshot.agents.len()).bold()
        );
        for handle in &snapshot.agents {
            let metadata = &handle.spec.metadata;
            let source = handle.spec.source.describe();
            let model_summary = describe_model(metadata, None);
            let tools = if metadata.tools.is_empty() {
                "(none)".to_string()
            } else {
                metadata.tools.join(", ")
            };
            println!("  • {} [{}]", metadata.name.cyan().bold(), source);
            if let Some(desc) = metadata.description.as_ref() {
                println!("      {desc}");
            }
            println!("      model: {model_summary}");
            println!("      tools: {tools}");
            if !metadata.keywords.is_empty() {
                println!("      keywords: {}", metadata.keywords.join(", "));
            }
            for warning in &handle.warnings {
                println!("      {} {}", "warning:".yellow(), warning);
            }
        }
    }

    if !snapshot.parse_errors.is_empty() {
        println!("\n{}", "Parse errors:".red().bold());
        for err in &snapshot.parse_errors {
            println!("  - {}\n      {}", err.path.display(), err.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_duration_formats_human_readable_values() {
        assert_eq!(format_duration(45), "45ms");
        assert_eq!(format_duration(1_000), "1s");
        assert_eq!(format_duration(1_250), "1.2s");
        assert_eq!(format_duration(75_000), "1m 15s");
    }
}
