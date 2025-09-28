# Codex Subagents

**AI-powered specialist agents for coding tasks** - Delegate work to focused agents for code review, documentation, testing, and debugging.

[![npm version](https://badge.fury.io/js/@stat-guy%2Fcodex-subagents.svg)](https://www.npmjs.com/package/@stat-guy/codex-subagents)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Quick Start

```bash
# Install globally
npm install -g @stat-guy/codex-subagents

# Initialize configuration
cv-agents init

# See available agents
cv-agents list

# Run a code review agent
cv-agents run code-reviewer --prompt "Review my staged changes"
```

## ✨ Features

- **🤖 Built-in Specialist Agents**: Code review, documentation, testing, and debugging
- **🎯 Task-Focused**: Each agent specializes in specific coding tasks
- **🛡️ Tool Security**: Configurable tool allowlists for safe execution
- **⚙️ Customizable**: Create your own agents with simple markdown files
- **🔄 Git Integration**: Seamlessly works with your existing Git workflow
- **🌍 Cross-Platform**: Works on macOS, Linux, and Windows

## 📦 Built-in Agents

### 🔍 **code-reviewer**
Reviews staged Git changes for logic bugs, style issues, and security concerns.

```bash
cv-agents run code-reviewer --prompt "Review these changes for potential issues"
```

### 📝 **doc-writer**
Generates comprehensive documentation for code projects, APIs, and functions.

```bash
cv-agents run doc-writer --prompt "Create API documentation for the user module"
```

### 🧪 **test-generator**
Creates comprehensive test suites with unit, integration, and edge case testing.

```bash
cv-agents run test-generator --prompt "Generate tests for the authentication service"
```

### 🐛 **bug-hunter**
Analyzes code to identify and propose fixes for bugs and performance issues.

```bash
cv-agents run bug-hunter --prompt "Find bugs in the payment processing code"
```

## 🛠️ Installation

### Prerequisites

- **Node.js** 16+
- **Git** (for repository analysis)
- **OpenAI API Key** (set `OPENAI_API_KEY` environment variable)

### Install

```bash
npm install -g @stat-guy/codex-subagents
```

### Configuration

```bash
# Initialize configuration
cv-agents init

# Edit configuration file
nano ~/.codex-subagents/config.yaml
```

### API Key Setup

```bash
# Set OpenAI API key
export OPENAI_API_KEY="your-api-key-here"

# Or add to your ~/.bashrc / ~/.zshrc
echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.bashrc
```

## 📖 Usage

### Basic Commands

```bash
# List all available agents
cv-agents list

# Run an agent interactively
cv-agents run <agent-name>

# Run with a specific prompt
cv-agents run <agent-name> --prompt "Your prompt here"

# Check agent status
cv-agents status

# Create a new custom agent
cv-agents create my-agent --template basic

# Check installation health
cv-agents doctor
```

### Advanced Usage

```bash
# Use with pipes
echo "Review this function" | cv-agents run code-reviewer

# JSON output for automation
cv-agents list --json

# Verbose logging
cv-agents run code-reviewer --verbose --prompt "Review main.js"

# Don't wait for completion
cv-agents run test-generator --no-wait --prompt "Generate tests"
```

### Aliases

For convenience, you can use the short alias `cva`:

```bash
cva list
cva run code-reviewer
cva create my-agent
```

## 🎨 Creating Custom Agents

Create agents tailored to your specific workflow:

```bash
# Create a new agent
cv-agents create my-reviewer --template code-review

# Edit the agent definition
nano ~/.codex-subagents/agents/my-reviewer.md
```

### Agent Definition Format

```markdown
---
name: my-custom-agent
description: Does something specific for my project
model: gpt-4
tools:
  - git
  - npm
  - node
keywords:
  - custom
  - specific-task
---

# My Custom Agent Instructions

I am a specialized agent that helps with...

## What I can do:
- Specific task 1
- Specific task 2

## Usage examples:
- "Help me with X"
- "Analyze Y and suggest Z"
```

### Agent Templates

Available templates:
- `basic` - Simple general-purpose agent
- `code-review` - Code review specialist
- `docs` - Documentation generator
- `testing` - Test suite creator

## ⚙️ Configuration

Configuration is stored in `~/.codex-subagents/config.yaml`:

```yaml
# AI Configuration
ai:
  provider: openai
  model: gpt-4
  apiKey: ${OPENAI_API_KEY}
  temperature: 0.1
  maxTokens: 4000

# Agent Configuration
agents:
  timeout: 300
  maxRetries: 2
  defaultTools: [git, node, npm]

# Security
security:
  allowedCommands: [git, npm, node, cargo, python3]
  blockedCommands: [rm, sudo, chmod]
  requireConfirmation: [git push, git reset --hard]
```

## 🔧 Troubleshooting

### Installation Issues

```bash
# Check installation
cv-agents doctor

# Reinstall if needed
npm uninstall -g @stat-guy/codex-subagents
npm install -g @stat-guy/codex-subagents
```

### Common Issues

**"Binary not found" Error:**
- Ensure you're on a supported platform (macOS, Linux, Windows)
- Try reinstalling the package
- Check the output of `cv-agents doctor`

**"API key not configured" Error:**
- Set your OpenAI API key: `export OPENAI_API_KEY="your-key"`
- Or update the config file: `~/.codex-subagents/config.yaml`

**"Agent not found" Error:**
- Check available agents: `cv-agents list`
- Ensure agent file exists: `ls ~/.codex-subagents/agents/`

### Getting Help

- 📖 **Documentation**: Full docs in the [repository](https://github.com/stat-guy/codex-cv)
- 🐛 **Issues**: Report bugs on [GitHub Issues](https://github.com/stat-guy/codex-cv/issues)
- 💬 **Discussions**: Ask questions in [GitHub Discussions](https://github.com/stat-guy/codex-cv/discussions)

## 🎯 Use Cases

### Code Review Workflow
```bash
# Stage your changes
git add .

# Get AI-powered code review
cv-agents run code-reviewer --prompt "Review my staged changes for bugs and improvements"

# Address feedback and commit
git commit -m "Fix issues identified by code review"
```

### Documentation Generation
```bash
# Generate project README
cv-agents run doc-writer --prompt "Create a comprehensive README for this project"

# Document specific functions
cv-agents run doc-writer --prompt "Document the authentication middleware functions"
```

### Test Creation
```bash
# Generate comprehensive tests
cv-agents run test-generator --prompt "Create unit tests for the user service module"

# Add integration tests
cv-agents run test-generator --prompt "Generate integration tests for the API endpoints"
```

### Bug Hunting
```bash
# Analyze problematic code
cv-agents run bug-hunter --prompt "Find potential bugs in the payment processing logic"

# Performance analysis
cv-agents run bug-hunter --prompt "Identify performance bottlenecks in this algorithm"
```

## 🏗️ Development

### Building from Source

```bash
git clone https://github.com/stat-guy/codex-cv.git
cd codex-cv
npm install
npm run build
npm test
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Commit your changes: `git commit -m 'Add amazing feature'`
5. Push to the branch: `git push origin feature/amazing-feature`
6. Open a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built on the foundation of AI-powered coding assistance
- Inspired by the need for specialized, focused AI agents
- Thanks to the open source community for tools and inspiration

---

**Ready to supercharge your coding workflow?**

```bash
npm install -g @stat-guy/codex-subagents
cv-agents init
cv-agents run code-reviewer
```

*Happy coding! 🚀*