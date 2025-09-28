#!/usr/bin/env node

const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const chalk = require('chalk');

async function postInstall() {
  try {
    console.log(chalk.blue('🔧 Setting up codex-subagents...'));

    // Get user's home directory
    const homeDir = os.homedir();
    const configDir = path.join(homeDir, '.codex-subagents');
    const agentsDir = path.join(configDir, 'agents');
    const configFile = path.join(configDir, 'config.yaml');

    // Create directories
    await fs.ensureDir(configDir);
    await fs.ensureDir(agentsDir);

    console.log(chalk.green('✅ Created configuration directories'));

    // Create default config if it doesn't exist
    if (!(await fs.pathExists(configFile))) {
      const defaultConfig = `# Codex Subagents Configuration
version: "1.0.0"

# AI Configuration
ai:
  provider: openai
  model: gpt-4
  apiKey: \${OPENAI_API_KEY}
  baseUrl: \${OPENAI_BASE_URL:-https://api.openai.com/v1}
  temperature: 0.1
  maxTokens: 4000

# Agent Configuration
agents:
  timeout: 300
  maxRetries: 2
  autoSave: true
  defaultTools:
    - git
    - node
    - npm
  allowCustomTools: true

# Output Configuration
output:
  format: text
  verbose: false
  colors: true
  showProgress: true
  logLevel: info

# Git Integration
git:
  autoDetectRepo: true
  requireCleanWorking: false
  createBackups: true

# Security
security:
  allowedCommands:
    - git
    - npm
    - node
    - cargo
    - python3
    - python
  blockedCommands:
    - rm
    - sudo
    - chmod
    - chown
  requireConfirmation:
    - git push
    - git reset --hard
    - rm -rf
`;

      await fs.writeFile(configFile, defaultConfig, 'utf8');
      console.log(chalk.green('✅ Created default configuration'));
    }

    // Copy example agent if agents directory is empty
    const agentFiles = await fs.readdir(agentsDir);
    const mdFiles = agentFiles.filter(f => f.endsWith('.md'));

    if (mdFiles.length === 0) {
      const exampleAgent = `---
name: example-helper
description: An example agent that helps with basic coding tasks
model: gpt-4
tools:
  - git
  - node
  - npm
keywords:
  - example
  - help
  - basic
---

# Example Helper Agent

I'm an example agent that demonstrates how to create custom subagents.

## What I can help with:

- Answer questions about your codebase
- Suggest improvements to code quality
- Help with Git operations
- Provide coding assistance

## How to customize me:

1. Edit this file: ~/.codex-subagents/agents/example-helper.md
2. Modify the YAML frontmatter to change my capabilities
3. Update this description to match your needs
4. Save the file and run: codex-subagents list

## Example usage:

\`\`\`bash
codex-subagents run example-helper --prompt "Help me understand this code"
\`\`\`

Feel free to create new agents based on this template!
`;

      const examplePath = path.join(agentsDir, 'example-helper.md');
      await fs.writeFile(examplePath, exampleAgent, 'utf8');
      console.log(chalk.green('✅ Created example agent'));
    }

    console.log(chalk.blue('\n🎉 Setup complete!\n'));

    // Show next steps
    console.log(chalk.blue('📚 Next steps:'));
    console.log('   1. Set your OpenAI API key:');
    console.log(chalk.cyan('      export OPENAI_API_KEY="your-key-here"'));
    console.log('   2. List available agents:');
    console.log(chalk.cyan('      codex-subagents list'));
    console.log('   3. Create your first custom agent:');
    console.log(chalk.cyan('      codex-subagents create my-agent'));
    console.log('   4. Run an agent:');
    console.log(chalk.cyan('      codex-subagents run example-helper'));

    // Check for API key
    if (!process.env.OPENAI_API_KEY) {
      console.log(chalk.yellow('\n⚠️  No OPENAI_API_KEY found in environment.'));
      console.log('   You\'ll need to set this before using the agents.');
    } else {
      console.log(chalk.green('\n✅ OPENAI_API_KEY found in environment'));
    }

    console.log(chalk.blue('\n📖 Documentation: https://github.com/stat-guy/codex-cv'));
    console.log(chalk.blue('🐛 Issues: https://github.com/stat-guy/codex-cv/issues'));

  } catch (error) {
    console.error(chalk.red('❌ Setup failed:'), error.message);

    // Don't fail the installation if setup fails
    console.log(chalk.yellow('⚠️  You can run setup manually with: codex-subagents init'));
  }
}

// Only run if this script is executed directly
if (require.main === module) {
  postInstall();
}

module.exports = { postInstall };