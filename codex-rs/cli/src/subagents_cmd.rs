use std::fmt::Write as _;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use codex_common::CliConfigOverrides;
use codex_core::AuthManager;
use codex_core::ConversationManager;
use codex_core::NewConversation;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::error::CodexErr;
use codex_core::subagents::CoreSubagentRegistry;
use codex_core::subagents::SubagentRunRequest;
use owo_colors::OwoColorize;
use serde_json::to_string_pretty;
use tokio::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Parser)]
pub struct SubagentsCli {
    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List available subagent definitions.
    List,

    /// Run a subagent using the configured feature flag settings.
    Run(RunArgs),
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    /// Name of the subagent to execute.
    pub agent: String,

    /// Optional inline prompt forwarded to the subagent conversation.
    #[arg(long = "prompt", short = 'p')]
    pub prompt: Option<String>,
}

pub async fn run_cli(cli: SubagentsCli) -> Result<()> {
    let SubagentsCli {
        config_overrides,
        command,
    } = cli;

    let overrides_vec = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;

    let config = Config::load_with_cli_overrides(overrides_vec, ConfigOverrides::default())?;
    let registry = CoreSubagentRegistry::from_config(&config);
    let registry = std::sync::Arc::new(registry);

    match command {
        Command::List => {
            let report = registry.reload().await;
            let agents = registry.list().await;

            if agents.is_empty() {
                println!("No subagents discovered. Add Markdown specs under .codex/agents.");
            } else {
                println!("Available subagents:\n");
                for record in agents {
                    let spec = record.spec.clone();
                    let mut line = String::new();
                    let _ = write!(line, "- {}", spec.name().green());
                    if let Some(desc) = spec.description() {
                        let _ = write!(line, ": {desc}");
                    }
                    if let Some(model) = spec.model() {
                        let _ = write!(line, " (model: {model})");
                    }
                    println!("{line}");
                }
            }

            if !report.errors.is_empty() {
                eprintln!("\nErrors:");
                for error in report.errors {
                    eprintln!("  {} — {}", error.path.display(), error.message.red());
                }
            }
        }
        Command::Run(args) => {
            let RunArgs { agent, prompt } = args;
            let request = SubagentRunRequest {
                agent_name: agent.clone(),
                prompt,
            };

            let auth_manager = AuthManager::shared(config.codex_home.clone());
            let manager = ConversationManager::new(auth_manager);
            match manager.spawn_subagent_conversation(config, request).await {
                Ok((new_conversation, _spec, lifecycle_events)) => {
                    let NewConversation {
                        conversation_id,
                        conversation,
                        session_configured,
                    } = new_conversation;

                    println!(
                        "spawned subagent conversation {}",
                        conversation_id.to_string().cyan()
                    );
                    println!(
                        "{}",
                        to_string_pretty(&codex_core::protocol::Event {
                            id: String::new(),
                            msg: codex_core::protocol::EventMsg::SessionConfigured(
                                session_configured
                            ),
                        })?
                    );

                    for event in lifecycle_events {
                        println!("{}", to_string_pretty(&event)?);
                    }

                    let mut attempts = 0;
                    while attempts < 10 {
                        attempts += 1;
                        match tokio::time::timeout(
                            Duration::from_millis(250),
                            conversation.next_event(),
                        )
                        .await
                        {
                            Ok(Ok(event)) => {
                                println!("{}", to_string_pretty(&event)?);
                            }
                            Ok(Err(err)) => {
                                eprintln!("error: {err:#}");
                                break;
                            }
                            Err(_) => {
                                sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
                Err(err) => match err {
                    CodexErr::SubagentsDisabled => {
                        eprintln!("{}", "Subagents are disabled. Enable subagents.enabled=true or CODEX_SUBAGENTS_ENABLED=1".red());
                    }
                    CodexErr::UnknownSubagent(name) => {
                        eprintln!("{}", format!("Unknown subagent `{name}`").red());
                    }
                    other => {
                        eprintln!("Subagent run failed: {other:#}");
                    }
                },
            }
        }
    }

    Ok(())
}
