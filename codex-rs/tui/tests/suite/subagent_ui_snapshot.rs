//! TUI snapshot tests for `/agents` command and subagent conversation rendering.

use codex_subagents::spec::SubagentSpec;
use codex_subagents::builder::SubagentBuilder;
use codex_subagents::registry::{SubagentRegistry, SubagentRecord, AgentSource, SubagentRegistryError};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

/// Mock agent data for testing UI rendering
pub struct MockAgentData {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub keywords: Vec<String>,
    pub source: AgentSource,
}

impl MockAgentData {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            source: AgentSource::Project,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_tools(mut self, tools: Vec<&str>) -> Self {
        self.tools = tools.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_source(mut self, source: AgentSource) -> Self {
        self.source = source;
        self
    }

    pub fn to_spec(&self) -> SubagentSpec {
        let mut builder = SubagentBuilder::new(&self.name)
            .instructions("Test instructions")
            .tools(self.tools.clone())
            .keywords(self.keywords.clone());

        if let Some(desc) = &self.description {
            builder = builder.description(desc.clone());
        }
        if let Some(model) = &self.model {
            builder = builder.model(model.clone());
        }

        builder.build().unwrap()
    }

    pub fn to_record(&self) -> SubagentRecord {
        SubagentRecord {
            spec: Arc::new(self.to_spec()),
            source: self.source,
        }
    }
}

/// Mock registry report for testing error scenarios
#[derive(Debug, Clone)]
pub struct MockReloadReport {
    pub loaded: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<SubagentRegistryError>,
}

impl MockReloadReport {
    pub fn new() -> Self {
        Self {
            loaded: Vec::new(),
            removed: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_loaded(mut self, agents: Vec<String>) -> Self {
        self.loaded = agents;
        self
    }

    pub fn with_errors(mut self, errors: Vec<SubagentRegistryError>) -> Self {
        self.errors = errors;
        self
    }
}

/// Agent list renderer for testing UI output
pub struct AgentListRenderer {
    agents: Vec<SubagentRecord>,
    report: MockReloadReport,
}

impl AgentListRenderer {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            report: MockReloadReport::new(),
        }
    }

    pub fn with_agents(mut self, agents: Vec<MockAgentData>) -> Self {
        self.agents = agents.into_iter().map(|a| a.to_record()).collect();
        self
    }

    pub fn with_report(mut self, report: MockReloadReport) -> Self {
        self.report = report;
        self
    }

    /// Render the agent list as it would appear in the TUI
    pub fn render_agent_list(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if self.agents.is_empty() {
            lines.push(Line::from("No subagents found"));
        } else {
            lines.push(Line::from("Available subagents:"));
            for record in &self.agents {
                let spec = record.spec.as_ref();
                let mut detail = format!("- {}", spec.name());
                if let Some(desc) = spec.description() {
                    detail.push_str(&format!(": {}", desc));
                }
                if let Some(model) = spec.model() {
                    detail.push_str(&format!(" (model: {model})"));
                }
                lines.push(detail.into());
            }
        }

        if !self.report.errors.is_empty() {
            lines.push(Line::from("Errors:"));
            for err in &self.report.errors {
                lines.push(format!("  {} — {}", err.path.display(), err.message).into());
            }
        }

        lines
    }

    /// Render detailed agent information for advanced views
    pub fn render_detailed_agent_list(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if self.agents.is_empty() {
            lines.push(Line::from("No subagents found"));
            return lines;
        }

        lines.push(Line::from("Available subagents:"));
        lines.push(Line::from(""));

        for record in &self.agents {
            let spec = record.spec.as_ref();

            // Agent name and source
            let source_indicator = match record.source {
                AgentSource::Project => "📁",
                AgentSource::User => "👤",
            };
            lines.push(Line::from(format!("{} {} ({})", source_indicator, spec.name(), record.source)));

            // Description
            if let Some(desc) = spec.description() {
                lines.push(Line::from(format!("  Description: {}", desc)));
            }

            // Model override
            if let Some(model) = spec.model() {
                lines.push(Line::from(format!("  Model: {}", model)).style(Style::default().fg(Color::Yellow)));
            }

            // Tools
            if !spec.tools().is_empty() {
                lines.push(Line::from(format!("  Tools: {}", spec.tools().join(", "))));
            }

            // Keywords
            if !spec.keywords().is_empty() {
                lines.push(Line::from(format!("  Keywords: {}", spec.keywords().join(", "))));
            }

            // Source path
            if let Some(path) = spec.source_path() {
                lines.push(Line::from(format!("  Source: {}", path.display())).style(Style::default().fg(Color::Cyan)));
            }

            lines.push(Line::from(""));
        }

        // Show errors if any
        if !self.report.errors.is_empty() {
            lines.push(Line::from("Parse Errors:").style(Style::default().fg(Color::Red)));
            for err in &self.report.errors {
                lines.push(Line::from(format!("  {} — {}", err.path.display(), err.message))
                    .style(Style::default().fg(Color::Red)));
            }
        }

        lines
    }
}

/// Mock subagent conversation events for testing UI rendering
#[derive(Debug, Clone)]
pub enum MockSubagentEvent {
    Started {
        agent_name: String,
        sub_conversation_id: String,
        model: Option<String>,
    },
    Message {
        agent_name: String,
        sub_conversation_id: String,
        role: String,
        content: String,
    },
    Completed {
        agent_name: String,
        sub_conversation_id: String,
        outcome: Option<String>,
    },
}

/// Subagent conversation renderer for testing
pub struct SubagentConversationRenderer {
    events: Vec<MockSubagentEvent>,
    main_conversation: Vec<String>,
}

impl SubagentConversationRenderer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            main_conversation: Vec::new(),
        }
    }

    pub fn with_main_conversation(mut self, messages: Vec<String>) -> Self {
        self.main_conversation = messages;
        self
    }

    pub fn with_subagent_events(mut self, events: Vec<MockSubagentEvent>) -> Self {
        self.events = events;
        self
    }

    /// Render conversation with nested subagent content
    pub fn render_conversation(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Main conversation messages
        for (i, message) in self.main_conversation.iter().enumerate() {
            lines.push(Line::from(format!("{}: {}", i + 1, message)));
        }

        // Render subagent events with visual hierarchy
        for event in &self.events {
            match event {
                MockSubagentEvent::Started { agent_name, sub_conversation_id, model } => {
                    let model_info = model.as_ref()
                        .map(|m| format!(" ({})", m))
                        .unwrap_or_default();

                    lines.push(Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("Started subagent '{}'{}",  agent_name, model_info),
                            Style::default().fg(Color::Cyan)
                        ),
                    ]));

                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("Conversation: {}", sub_conversation_id),
                            Style::default().fg(Color::Gray)
                        ),
                    ]));
                }

                MockSubagentEvent::Message { agent_name, sub_conversation_id: _, role, content } => {
                    let role_color = match role.as_str() {
                        "user" => Color::Green,
                        "assistant" => Color::Blue,
                        _ => Color::White,
                    };

                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("[{}] ", agent_name), Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}: ", role), Style::default().fg(role_color)),
                        Span::raw(content),
                    ]));
                }

                MockSubagentEvent::Completed { agent_name, sub_conversation_id: _, outcome } => {
                    let outcome_text = outcome.as_ref()
                        .map(|o| format!(" ({})", o))
                        .unwrap_or_default();

                    let style = if outcome.as_ref().map_or(false, |o| o.contains("error")) {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    lines.push(Line::from(vec![
                        Span::styled("◀ ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("Completed subagent '{}'{}",  agent_name, outcome_text),
                            style
                        ),
                    ]));
                }
            }
        }

        lines
    }

    /// Render compact conversation summary
    pub fn render_conversation_summary(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Count different types of events
        let mut agent_stats: HashMap<String, (u32, u32, u32)> = HashMap::new(); // (started, messages, completed)

        for event in &self.events {
            match event {
                MockSubagentEvent::Started { agent_name, .. } => {
                    agent_stats.entry(agent_name.clone()).or_default().0 += 1;
                }
                MockSubagentEvent::Message { agent_name, .. } => {
                    agent_stats.entry(agent_name.clone()).or_default().1 += 1;
                }
                MockSubagentEvent::Completed { agent_name, .. } => {
                    agent_stats.entry(agent_name.clone()).or_default().2 += 1;
                }
            }
        }

        if agent_stats.is_empty() {
            lines.push(Line::from("No subagent activity"));
            return lines;
        }

        lines.push(Line::from("Subagent Activity Summary:"));
        for (agent_name, (started, messages, completed)) in agent_stats {
            lines.push(Line::from(format!(
                "  {}: {} runs, {} messages, {} completed",
                agent_name, started, messages, completed
            )));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty_agent_list_rendering() {
        let renderer = AgentListRenderer::new();
        let lines = renderer.render_agent_list();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].content, "No subagents found");
    }

    #[test]
    fn test_single_agent_rendering() {
        let agent = MockAgentData::new("test-agent")
            .with_description("A test agent")
            .with_model("gpt-4");

        let renderer = AgentListRenderer::new()
            .with_agents(vec![agent]);

        let lines = renderer.render_agent_list();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].spans[0].content, "Available subagents:");
        assert_eq!(lines[1].spans[0].content, "- test-agent: A test agent (model: gpt-4)");
    }

    #[test]
    fn test_multiple_agents_rendering() {
        let agents = vec![
            MockAgentData::new("code-reviewer")
                .with_description("Reviews code for issues")
                .with_tools(vec!["read", "analysis"])
                .with_source(AgentSource::Project),
            MockAgentData::new("test-runner")
                .with_description("Runs test suites")
                .with_model("gpt-3.5-turbo")
                .with_tools(vec!["bash", "read"])
                .with_source(AgentSource::User),
            MockAgentData::new("simple-agent"),
        ];

        let renderer = AgentListRenderer::new()
            .with_agents(agents);

        let lines = renderer.render_agent_list();

        assert_eq!(lines.len(), 4); // Header + 3 agents
        assert_eq!(lines[0].spans[0].content, "Available subagents:");
        assert_eq!(lines[1].spans[0].content, "- code-reviewer: Reviews code for issues");
        assert_eq!(lines[2].spans[0].content, "- test-runner: Runs test suites (model: gpt-3.5-turbo)");
        assert_eq!(lines[3].spans[0].content, "- simple-agent");
    }

    #[test]
    fn test_agents_with_errors_rendering() {
        let agent = MockAgentData::new("working-agent")
            .with_description("This one works");

        let errors = vec![
            SubagentRegistryError {
                path: PathBuf::from("broken-agent.md"),
                message: "Invalid YAML frontmatter".to_string(),
            },
            SubagentRegistryError {
                path: PathBuf::from("missing-agent.md"),
                message: "File not found".to_string(),
            },
        ];

        let report = MockReloadReport::new()
            .with_errors(errors);

        let renderer = AgentListRenderer::new()
            .with_agents(vec![agent])
            .with_report(report);

        let lines = renderer.render_agent_list();

        assert_eq!(lines.len(), 5); // Header + 1 agent + error header + 2 errors
        assert_eq!(lines[0].spans[0].content, "Available subagents:");
        assert_eq!(lines[1].spans[0].content, "- working-agent: This one works");
        assert_eq!(lines[2].spans[0].content, "Errors:");
        assert!(lines[3].spans[0].content.contains("broken-agent.md"));
        assert!(lines[3].spans[0].content.contains("Invalid YAML frontmatter"));
        assert!(lines[4].spans[0].content.contains("missing-agent.md"));
        assert!(lines[4].spans[0].content.contains("File not found"));
    }

    #[test]
    fn test_detailed_agent_list_rendering() {
        let agent = MockAgentData::new("detailed-agent")
            .with_description("A detailed test agent")
            .with_model("gpt-4")
            .with_tools(vec!["read", "write", "analysis"])
            .with_source(AgentSource::Project);

        let renderer = AgentListRenderer::new()
            .with_agents(vec![agent]);

        let lines = renderer.render_detailed_agent_list();

        assert!(lines.len() > 5); // Should have detailed information
        assert_eq!(lines[0].spans[0].content, "Available subagents:");

        // Find the agent name line
        let agent_line = lines.iter().find(|line| line.spans[0].content.contains("detailed-agent")).unwrap();
        assert!(agent_line.spans[0].content.contains("📁")); // Project source indicator

        // Check for description line
        let desc_line = lines.iter().find(|line| line.spans[0].content.contains("Description:")).unwrap();
        assert!(desc_line.spans[0].content.contains("A detailed test agent"));

        // Check for model line
        let model_line = lines.iter().find(|line| line.spans[0].content.contains("Model:")).unwrap();
        assert!(model_line.spans[0].content.contains("gpt-4"));

        // Check for tools line
        let tools_line = lines.iter().find(|line| line.spans[0].content.contains("Tools:")).unwrap();
        assert!(tools_line.spans[0].content.contains("read, write, analysis"));
    }

    #[test]
    fn test_subagent_conversation_rendering() {
        let events = vec![
            MockSubagentEvent::Started {
                agent_name: "code-reviewer".to_string(),
                sub_conversation_id: "conv-123".to_string(),
                model: Some("gpt-4".to_string()),
            },
            MockSubagentEvent::Message {
                agent_name: "code-reviewer".to_string(),
                sub_conversation_id: "conv-123".to_string(),
                role: "user".to_string(),
                content: "Review this code".to_string(),
            },
            MockSubagentEvent::Message {
                agent_name: "code-reviewer".to_string(),
                sub_conversation_id: "conv-123".to_string(),
                role: "assistant".to_string(),
                content: "The code looks good but has a potential null pointer issue".to_string(),
            },
            MockSubagentEvent::Completed {
                agent_name: "code-reviewer".to_string(),
                sub_conversation_id: "conv-123".to_string(),
                outcome: Some("success".to_string()),
            },
        ];

        let renderer = SubagentConversationRenderer::new()
            .with_main_conversation(vec![
                "User: Please review my changes".to_string(),
                "Assistant: I'll use the code reviewer agent for this".to_string(),
            ])
            .with_subagent_events(events);

        let lines = renderer.render_conversation();

        assert!(lines.len() > 6); // Main conversation + subagent events

        // Check main conversation
        assert!(lines[0].spans[0].content.contains("Please review my changes"));
        assert!(lines[1].spans[0].content.contains("I'll use the code reviewer agent"));

        // Find and verify subagent events
        let started_line = lines.iter().find(|line|
            line.spans.iter().any(|span| span.content.contains("Started subagent"))
        ).unwrap();
        assert!(started_line.spans.iter().any(|span| span.content.contains("code-reviewer")));
        assert!(started_line.spans.iter().any(|span| span.content.contains("gpt-4")));

        let user_message_line = lines.iter().find(|line|
            line.spans.iter().any(|span| span.content.contains("Review this code"))
        ).unwrap();
        assert!(user_message_line.spans.iter().any(|span| span.content.contains("user:")));

        let assistant_message_line = lines.iter().find(|line|
            line.spans.iter().any(|span| span.content.contains("null pointer issue"))
        ).unwrap();
        assert!(assistant_message_line.spans.iter().any(|span| span.content.contains("assistant:")));

        let completed_line = lines.iter().find(|line|
            line.spans.iter().any(|span| span.content.contains("Completed subagent"))
        ).unwrap();
        assert!(completed_line.spans.iter().any(|span| span.content.contains("success")));
    }

    #[test]
    fn test_conversation_summary_rendering() {
        let events = vec![
            MockSubagentEvent::Started {
                agent_name: "agent-1".to_string(),
                sub_conversation_id: "conv-1".to_string(),
                model: None,
            },
            MockSubagentEvent::Message {
                agent_name: "agent-1".to_string(),
                sub_conversation_id: "conv-1".to_string(),
                role: "user".to_string(),
                content: "Test message 1".to_string(),
            },
            MockSubagentEvent::Message {
                agent_name: "agent-1".to_string(),
                sub_conversation_id: "conv-1".to_string(),
                role: "assistant".to_string(),
                content: "Response 1".to_string(),
            },
            MockSubagentEvent::Completed {
                agent_name: "agent-1".to_string(),
                sub_conversation_id: "conv-1".to_string(),
                outcome: Some("success".to_string()),
            },
            MockSubagentEvent::Started {
                agent_name: "agent-2".to_string(),
                sub_conversation_id: "conv-2".to_string(),
                model: None,
            },
            MockSubagentEvent::Completed {
                agent_name: "agent-2".to_string(),
                sub_conversation_id: "conv-2".to_string(),
                outcome: Some("error".to_string()),
            },
        ];

        let renderer = SubagentConversationRenderer::new()
            .with_subagent_events(events);

        let lines = renderer.render_conversation_summary();

        assert_eq!(lines.len(), 3); // Header + 2 agent summaries
        assert_eq!(lines[0].spans[0].content, "Subagent Activity Summary:");

        // Check that both agents are represented
        assert!(lines.iter().any(|line| line.spans[0].content.contains("agent-1")));
        assert!(lines.iter().any(|line| line.spans[0].content.contains("agent-2")));

        // Check counts
        let agent1_line = lines.iter().find(|line| line.spans[0].content.contains("agent-1")).unwrap();
        assert!(agent1_line.spans[0].content.contains("1 runs"));
        assert!(agent1_line.spans[0].content.contains("2 messages"));
        assert!(agent1_line.spans[0].content.contains("1 completed"));
    }

    #[test]
    fn test_empty_conversation_summary() {
        let renderer = SubagentConversationRenderer::new();
        let lines = renderer.render_conversation_summary();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].content, "No subagent activity");
    }

    #[test]
    fn test_agent_source_indicators() {
        let agents = vec![
            MockAgentData::new("project-agent")
                .with_source(AgentSource::Project),
            MockAgentData::new("user-agent")
                .with_source(AgentSource::User),
        ];

        let renderer = AgentListRenderer::new()
            .with_agents(agents);

        let lines = renderer.render_detailed_agent_list();

        let project_line = lines.iter().find(|line| line.spans[0].content.contains("project-agent")).unwrap();
        assert!(project_line.spans[0].content.contains("📁")); // Project indicator

        let user_line = lines.iter().find(|line| line.spans[0].content.contains("user-agent")).unwrap();
        assert!(user_line.spans[0].content.contains("👤")); // User indicator
    }

    #[test]
    fn test_error_rendering_edge_cases() {
        // Test with very long file paths and error messages
        let long_path = PathBuf::from("very/long/path/to/agent/file/that/might/cause/display/issues.md");
        let long_message = "This is a very long error message that might wrap in the display and we need to ensure it's handled correctly without breaking the UI layout";

        let errors = vec![
            SubagentRegistryError {
                path: long_path,
                message: long_message.to_string(),
            },
        ];

        let report = MockReloadReport::new()
            .with_errors(errors);

        let renderer = AgentListRenderer::new()
            .with_report(report);

        let lines = renderer.render_agent_list();

        assert_eq!(lines.len(), 3); // "No subagents found" + "Errors:" + error line
        assert!(lines[2].spans[0].content.contains("very/long/path"));
        assert!(lines[2].spans[0].content.contains("very long error message"));
    }
}