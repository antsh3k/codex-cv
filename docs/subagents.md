# Codex Subagents

**Codex Subagents** enable you to delegate specialized tasks to focused AI agents while maintaining full integration with your main Codex conversation. Each subagent runs in an isolated context with configurable tool access and can be customized for specific workflows.

## Quick Start

### 1. Enable Subagents

Add to your `~/.codex/config.toml`:

```toml
[subagents]
enabled = true
auto_route = false  # Optional: enable automatic agent selection
```

Or use the environment variable:

```bash
export CODEX_SUBAGENTS_ENABLED=1
```

### 2. List Available Agents

```bash
# CLI
codex subagents list

# TUI
/agents
```

### 3. Use a Subagent

```bash
# CLI with prompt
codex subagents run code-reviewer -- "Review the staged changes"

# TUI
/use code-reviewer
```

## Core Concepts

### Agent Definitions

Subagents are defined using Markdown files with YAML frontmatter:

```markdown
---
name: code-reviewer
description: Reviews staged Git changes for issues
model: gpt-5-codex
tools:
  - git
  - cargo
keywords:
  - review
  - rust
---

Walk through each staged diff and report:
- logic bugs or missing edge cases
- formatting violations not covered by automated tools
- security or privacy concerns
```

### Discovery and Precedence

Agents are loaded from:

1. **Project agents**: `<repo>/.codex/agents/*.md` (highest priority)
2. **User agents**: `~/.codex/agents/*.md`
3. **Built-in examples**: Available in the Codex installation

Project agents override user agents with the same name.

### Tool Access Control

Each subagent has an explicit allowlist of tools they can use:

```yaml
tools:
  - git
  - cargo
  - npm
  - grep
```

Subagents cannot access tools not in their allowlist, providing security isolation.

## Agent Configuration

### YAML Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | ✅ | Unique identifier for the agent |
| `description` | ❌ | Human-readable summary shown in lists |
| `model` | ❌ | Model override (falls back to session default) |
| `tools` | ❌ | Allowlisted tools the agent can use |
| `keywords` | ❌ | Hints for auto-routing (when enabled) |

### Model Overrides

You can specify different models for different agents:

```yaml
---
name: code-reviewer
model: gpt-5-codex  # Use a code-specialized model
---
```

If the model is unavailable, the agent falls back to your session's default model.

## Built-in Example Agents

Codex ships with several example agents you can use immediately:

### `code-reviewer`
Reviews staged Git changes for logic bugs, style issues, and security concerns.

**Tools**: `git`, `cargo`, `npm`
**Keywords**: `review`, `rust`, `javascript`, `typescript`

### `doc-writer`
Generates comprehensive documentation for code projects.

**Tools**: `git`, `node`, `python`
**Keywords**: `documentation`, `readme`, `api`, `docs`

### `test-generator`
Creates comprehensive test suites with unit and integration tests.

**Tools**: `git`, `cargo`, `npm`, `python`
**Keywords**: `testing`, `unit-tests`, `integration`, `tdd`

### `bug-hunter`
Analyzes code to identify and propose fixes for bugs.

**Tools**: `git`, `cargo`, `npm`, `grep`
**Keywords**: `debugging`, `bugs`, `fixes`, `analysis`

## Usage Patterns

### CLI Usage

```bash
# List all available agents
codex subagents list

# Run an agent with a specific prompt
codex subagents run doc-writer -- "Generate API documentation for the user module"

# Check agent execution status
codex status  # Shows running subagent counters
```

### TUI Usage

```bash
# Launch TUI
codex tui

# In TUI, use these commands:
/agents              # List available agents
/use code-reviewer   # Launch the code reviewer
/subagent-status    # Check running agents
```

### Programmatic Usage (MCP)

External tools can interact with subagents via MCP:

```typescript
// List available agents
const agents = await mcpClient.callTool('subagents-list');

// Run a subagent
const result = await mcpClient.callTool('subagents-run', {
  conversationId: 'conv-123',
  agentName: 'code-reviewer',
  prompt: 'Review the latest changes'
});
```

## Creating Custom Agents

### Basic Agent Structure

1. **Create the agent file** in `~/.codex/agents/my-agent.md`
2. **Define the frontmatter** with name, tools, and metadata
3. **Write clear instructions** in the Markdown body
4. **Test the agent** with `/use my-agent`

### Best Practices

#### Agent Design
- **Single Purpose**: Focus each agent on one specific task
- **Clear Instructions**: Write detailed, actionable prompts
- **Tool Minimalism**: Only request tools the agent actually needs
- **Error Handling**: Include guidance for common failure scenarios

#### Security Considerations
- **Principle of Least Privilege**: Grant minimal tool access
- **Input Validation**: Design prompts that handle malicious input safely
- **Output Sanitization**: Be cautious with agent-generated code or commands

#### Performance Tips
- **Concise Prompts**: Avoid overly verbose instructions
- **Context Awareness**: Design agents that work well with limited context
- **Model Selection**: Choose appropriate models for the task complexity

### Example: Custom Agent

```markdown
---
name: api-tester
description: Tests REST API endpoints for correctness
model: gpt-5-codex
tools:
  - curl
  - node
keywords:
  - api
  - testing
  - http
---

# API Testing Instructions

Test the specified API endpoints by:

1. **Endpoint Discovery**: Identify all endpoints to test
2. **Request Generation**: Create appropriate test requests
3. **Response Validation**: Verify status codes, headers, and body content
4. **Error Scenarios**: Test error conditions and edge cases

## Test Categories

- **Happy Path**: Normal operation with valid inputs
- **Validation**: Invalid inputs and boundary conditions
- **Authentication**: Proper auth handling and rejection
- **Performance**: Response times and rate limiting

Report results in a structured format with pass/fail status and details.
```

## Monitoring and Debugging

### Status Monitoring

Check subagent activity:

```bash
# CLI status
codex status

# TUI status
/status
```

Output includes:
- Running subagent count
- Average execution duration
- Success rates per agent
- Recent activity summary

### Debug Mode

Enable detailed logging:

```bash
export CODEX_DEBUG_SUBAGENTS=1
codex tui
```

This provides:
- TaskContext serialization
- Execution flow traces
- Error details and stack traces
- Performance metrics

### Conflict Detection

Codex automatically detects when multiple agents try to modify the same files:

- **⚠ Warnings**: When different agents modify the same files sequentially
- **🚫 Blocking**: When simultaneous edits are attempted
- **Attribution**: Clear indicators of which agent made which changes

## Advanced Features

### Auto-Routing

When enabled, Codex can automatically select appropriate agents based on keywords:

```toml
[subagents]
enabled = true
auto_route = true
```

The router uses simple keyword matching to suggest agents for your requests.

### Model Fallbacks

Agent model selection follows this hierarchy:

1. Agent-specified model (in YAML frontmatter)
2. Session default model
3. Global default model

### Event Tracking

All subagent activities generate events that can be monitored:

- `SubAgentStarted`: Agent execution begins
- `SubAgentMessage`: Agent sends a message
- `SubAgentCompleted`: Agent execution ends

These events include metadata like agent name, model used, and execution duration.

## Troubleshooting

### Common Issues

#### Agent Not Found
```
Error: Agent 'my-agent' not found
```

**Solution**: Check that the agent file exists in `~/.codex/agents/` or `.codex/agents/` and has a valid name field.

#### Tool Access Denied
```
Error: Tool 'docker' not allowed for agent 'my-agent'
```

**Solution**: Add the tool to the agent's `tools` allowlist in the YAML frontmatter.

#### Parse Errors
```
Error: Invalid YAML frontmatter in my-agent.md
```

**Solution**: Validate the YAML syntax. Ensure proper `---` delimiters and correct indentation.

#### Model Not Available
```
Warning: Model 'gpt-5-special' not available, falling back to default
```

**Solution**: Verify the model name is correct or remove the model field to use the session default.

### Performance Issues

#### Slow Agent Execution
- Check if the agent's tools are available and responsive
- Reduce prompt complexity or context size
- Consider using a faster model for simple tasks

#### High Memory Usage
- Limit the number of concurrent agents
- Review agent instructions for unnecessary complexity
- Monitor TaskContext size in debug mode

### Feature Flag Issues

#### Subagents Not Available
```
Error: Subagents feature is not enabled
```

**Solution**: Set `subagents.enabled = true` in config or `CODEX_SUBAGENTS_ENABLED=1` environment variable.

## Migration and Compatibility

### Upgrading from Beta

If you used subagents in beta versions:

1. **Agent Definitions**: No changes needed for most agent files
2. **Configuration**: Migrate from `experimental.subagents` to `subagents` section
3. **Commands**: `/subagent` command renamed to `/use`

### Backward Compatibility

- Subagents are completely feature-flagged
- When disabled, Codex behaves exactly as before
- No performance impact when feature is disabled
- Existing workflows remain unchanged

### Breaking Changes

**Version 0.2.0+**:
- `tools` field now required for tool access (previously inherited all tools)
- Model fallback behavior standardized
- Event payload format updated for MCP integration

## API Reference

### MCP Tool Calls

#### `subagents-list`

Lists all available subagents.

**Parameters**: None

**Response**:
```typescript
{
  agents: Array<{
    name: string;
    description?: string;
    model?: string;
    tools: string[];
    source: string;
    parse_errors?: string[];
  }>
}
```

#### `subagents-run`

Executes a subagent in a conversation.

**Parameters**:
```typescript
{
  conversationId: string;
  agentName: string;
  prompt?: string;
}
```

**Response**:
```typescript
{
  subConversationId: string;
}
```

### CLI Commands

#### `codex subagents list`

Lists all available subagents with metadata.

**Options**:
- `--json`: Output in JSON format
- `--verbose`: Include parse errors and source paths

#### `codex subagents run <agent> [-- prompt]`

Runs a subagent with an optional prompt.

**Arguments**:
- `agent`: Name of the subagent to run
- `prompt`: Optional prompt text (use `--` to separate from options)

**Options**:
- `--conversation-id`: Specify conversation context
- `--no-wait`: Don't wait for completion (return immediately)

### TUI Commands

#### `/agents`

Shows the agent browser with all available subagents, their sources, and any parse errors.

#### `/use <agent> [prompt]`

Launches a subagent with an optional prompt.

#### `/subagent-status`

Displays current subagent execution status and recent activity.

## Best Practices Summary

### For Agent Authors

1. **Single Responsibility**: Each agent should have one clear purpose
2. **Minimal Tools**: Only request tools you actually need
3. **Clear Documentation**: Write instructions that are easy to understand
4. **Error Resilience**: Handle failures gracefully and provide fallbacks
5. **Test Thoroughly**: Validate agents with various inputs and scenarios

### For Users

1. **Start Simple**: Begin with built-in agents before creating custom ones
2. **Incremental Adoption**: Enable subagents for specific workflows first
3. **Monitor Performance**: Use status commands to track agent efficiency
4. **Review Results**: Always verify agent outputs before applying changes
5. **Share Agents**: Contribute useful agents back to the community

### For Organizations

1. **Governance**: Establish guidelines for agent creation and tool access
2. **Security Review**: Audit agent definitions and tool allowlists
3. **Training**: Provide team training on effective agent use
4. **Monitoring**: Track agent usage and performance across teams
5. **Feedback Loop**: Collect user feedback to improve agent effectiveness

## Resources

- **Specification Format**: [docs/subagents/spec-format.md](spec-format.md)
- **Demo Playbook**: [docs/subagents/demo-playbook.md](demo-playbook.md)
- **Architecture Decision Record**: [docs/subagents/adr-0001-subagents.md](adr-0001-subagents.md)
- **Example Agents**: `examples/agents/` in your Codex installation
- **GitHub Issues**: Report bugs and request features
- **Community Discord**: Share agents and get help from other users

---

*Ready to delegate your coding tasks? Start with `/agents` to explore what's available, or create your first custom agent to supercharge your workflow.*