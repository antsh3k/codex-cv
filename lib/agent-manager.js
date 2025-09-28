const fs = require('fs-extra');
const path = require('path');
const yaml = require('yaml');
const chalk = require('chalk');
const { getAgentsDir, loadConfig } = require('./config');

/**
 * Manages agent definitions and operations
 */
class AgentManager {
  constructor() {
    this.builtinAgentsDir = path.join(__dirname, '..', 'agents');
  }

  /**
   * Get path for a specific agent
   */
  getAgentPath(agentName) {
    const agentsDir = getAgentsDir();
    return path.join(agentsDir, `${agentName}.md`);
  }

  /**
   * Get path for a builtin agent
   */
  getBuiltinAgentPath(agentName) {
    return path.join(this.builtinAgentsDir, `${agentName}.md`);
  }

  /**
   * List all available agents
   */
  async listAgents(options = {}) {
    const agents = await this.loadAllAgents(options.all);

    if (options.json) {
      console.log(JSON.stringify(agents, null, 2));
      return;
    }

    // Text output
    if (agents.length === 0) {
      console.log(chalk.yellow('No agents found.'));
      console.log(chalk.blue('ℹ Run'), chalk.cyan('codex-subagents create <name>'), chalk.blue('to create your first agent'));
      return;
    }

    console.log(chalk.blue('📋 Available Agents:\n'));

    // Separate builtin and custom agents
    const builtinAgents = agents.filter(a => a.source === 'builtin');
    const customAgents = agents.filter(a => a.source === 'user');

    if (builtinAgents.length > 0) {
      console.log(chalk.blue('Built-in Agents:'));
      for (const agent of builtinAgents) {
        this.printAgentInfo(agent);
      }
      console.log();
    }

    if (customAgents.length > 0) {
      console.log(chalk.blue('Custom Agents:'));
      for (const agent of customAgents) {
        this.printAgentInfo(agent);
      }
    }

    console.log(chalk.blue('\n💡 Usage:'));
    console.log(chalk.gray('   codex-subagents run <agent-name>'));
    console.log(chalk.gray('   codex-subagents create <new-agent-name>'));
  }

  /**
   * Print information about a single agent
   */
  printAgentInfo(agent) {
    const nameColor = agent.source === 'builtin' ? 'cyan' : 'green';
    const prefix = agent.source === 'builtin' ? '  📦' : '  👤';

    console.log(prefix, chalk[nameColor](agent.name));

    if (agent.description) {
      console.log('     ', chalk.gray(agent.description));
    }

    if (agent.tools && agent.tools.length > 0) {
      console.log('     ', chalk.blue('Tools:'), agent.tools.join(', '));
    }

    if (agent.model) {
      console.log('     ', chalk.blue('Model:'), agent.model);
    }

    if (agent.keywords && agent.keywords.length > 0) {
      console.log('     ', chalk.blue('Keywords:'), agent.keywords.join(', '));
    }

    if (agent.parseErrors && agent.parseErrors.length > 0) {
      console.log('     ', chalk.red('⚠ Parse Errors:'), agent.parseErrors.join('; '));
    }

    console.log();
  }

  /**
   * Load all agents from both builtin and user directories
   */
  async loadAllAgents(includeBuiltin = true) {
    const agents = [];

    // Load user agents
    try {
      const userAgentsDir = getAgentsDir();
      if (await fs.pathExists(userAgentsDir)) {
        const userFiles = await fs.readdir(userAgentsDir);
        for (const file of userFiles) {
          if (file.endsWith('.md')) {
            const agentPath = path.join(userAgentsDir, file);
            const agent = await this.loadAgent(agentPath, 'user');
            if (agent) agents.push(agent);
          }
        }
      }
    } catch (error) {
      console.warn(chalk.yellow('⚠ Failed to load user agents:'), error.message);
    }

    // Load builtin agents
    if (includeBuiltin) {
      try {
        if (await fs.pathExists(this.builtinAgentsDir)) {
          const builtinFiles = await fs.readdir(this.builtinAgentsDir);
          for (const file of builtinFiles) {
            if (file.endsWith('.md')) {
              const agentPath = path.join(this.builtinAgentsDir, file);
              const agent = await this.loadAgent(agentPath, 'builtin');
              if (agent) {
                // Only add builtin if no user agent with same name exists
                const existingAgent = agents.find(a => a.name === agent.name);
                if (!existingAgent) {
                  agents.push(agent);
                }
              }
            }
          }
        }
      } catch (error) {
        console.warn(chalk.yellow('⚠ Failed to load builtin agents:'), error.message);
      }
    }

    return agents.sort((a, b) => a.name.localeCompare(b.name));
  }

  /**
   * Load a single agent from a file
   */
  async loadAgent(agentPath, source = 'user') {
    try {
      const content = await fs.readFile(agentPath, 'utf8');
      const agent = this.parseAgentDefinition(content, agentPath, source);
      return agent;
    } catch (error) {
      return {
        name: path.basename(agentPath, '.md'),
        source,
        filePath: agentPath,
        parseErrors: [`Failed to read file: ${error.message}`]
      };
    }
  }

  /**
   * Parse agent definition from markdown content
   */
  parseAgentDefinition(content, filePath, source = 'user') {
    const parseErrors = [];
    let name = path.basename(filePath, '.md');

    try {
      // Extract YAML frontmatter
      const frontmatterMatch = content.match(/^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)$/);

      if (!frontmatterMatch) {
        parseErrors.push('No YAML frontmatter found');
        return { name, source, filePath, parseErrors };
      }

      const [, frontmatterYaml, instructions] = frontmatterMatch;
      const frontmatter = yaml.parse(frontmatterYaml);

      // Validate required fields
      if (!frontmatter.name) {
        parseErrors.push('Missing required field: name');
      } else {
        name = frontmatter.name;
      }

      // Validate tools format
      if (frontmatter.tools && !Array.isArray(frontmatter.tools)) {
        parseErrors.push('Tools must be an array');
      }

      // Validate keywords format
      if (frontmatter.keywords && !Array.isArray(frontmatter.keywords)) {
        parseErrors.push('Keywords must be an array');
      }

      return {
        name,
        description: frontmatter.description || '',
        model: frontmatter.model || '',
        tools: frontmatter.tools || [],
        keywords: frontmatter.keywords || [],
        instructions: instructions.trim(),
        source,
        filePath,
        parseErrors
      };

    } catch (error) {
      parseErrors.push(`YAML parsing error: ${error.message}`);
      return { name, source, filePath, parseErrors };
    }
  }

  /**
   * Create a new agent
   */
  async createAgent(name, options = {}) {
    // Validate name
    if (!name.match(/^[a-z0-9-]+$/)) {
      throw new Error('Agent name must contain only lowercase letters, numbers, and hyphens');
    }

    const agentPath = this.getAgentPath(name);

    // Check if agent already exists
    if (await fs.pathExists(agentPath)) {
      throw new Error(`Agent '${name}' already exists at ${agentPath}`);
    }

    // Get template content
    const template = this.getAgentTemplate(options.template || 'basic', {
      name,
      description: options.description || `A custom agent named ${name}`,
      tools: options.tools ? options.tools.split(',').map(t => t.trim()) : ['git'],
      model: options.model || ''
    });

    // Ensure agents directory exists
    const agentsDir = getAgentsDir();
    await fs.ensureDir(agentsDir);

    // Write agent file
    await fs.writeFile(agentPath, template, 'utf8');

    return agentPath;
  }

  /**
   * Get agent template content
   */
  getAgentTemplate(templateType, vars) {
    const templates = {
      basic: `---
name: ${vars.name}
description: ${vars.description}
${vars.model ? `model: ${vars.model}` : ''}
tools:
${vars.tools.map(tool => `  - ${tool}`).join('\n')}
keywords:
  - custom
  - ${vars.name}
---

# ${vars.name.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase())} Agent

I am a custom agent designed to help with specific tasks.

## What I can do:

- Analyze code and provide suggestions
- Help with Git operations
- Assist with project tasks

## How to use me:

\`\`\`bash
codex-subagents run ${vars.name} --prompt "Your request here"
\`\`\`

Customize this description and the YAML frontmatter above to match your specific needs.
`,

      'code-review': `---
name: ${vars.name}
description: ${vars.description || 'Reviews code for quality, bugs, and improvements'}
${vars.model ? `model: ${vars.model}` : ''}
tools:
  - git
  - node
  - npm
keywords:
  - review
  - code-quality
  - bugs
---

# Code Review Agent

I specialize in reviewing code for quality, potential bugs, and improvement opportunities.

## What I analyze:

- **Code Quality**: Style, readability, and maintainability
- **Bug Detection**: Logic errors, edge cases, and potential failures
- **Performance**: Optimization opportunities and bottlenecks
- **Security**: Potential vulnerabilities and security best practices
- **Documentation**: Code comments and documentation quality

## Review Process:

1. Analyze the provided code or diff
2. Identify issues by severity (Critical, High, Medium, Low)
3. Provide specific recommendations with examples
4. Suggest best practices and improvements

## Usage:

\`\`\`bash
# Review staged changes
codex-subagents run ${vars.name} --prompt "Review my staged changes"

# Review specific file
codex-subagents run ${vars.name} --prompt "Review src/main.js for potential issues"
\`\`\`
`,

      docs: `---
name: ${vars.name}
description: ${vars.description || 'Generates comprehensive documentation'}
${vars.model ? `model: ${vars.model}` : ''}
tools:
  - git
  - node
keywords:
  - documentation
  - readme
  - api-docs
---

# Documentation Generator

I specialize in creating clear, comprehensive documentation for code projects.

## Documentation Types:

- **API Documentation**: Function and method documentation
- **README Files**: Project overviews and getting started guides
- **Code Comments**: Inline documentation for complex code
- **Architecture Docs**: High-level system documentation
- **User Guides**: How-to guides and tutorials

## Documentation Standards:

- Clear, concise language
- Practical examples and code snippets
- Proper formatting and structure
- Up-to-date and accurate information

## Usage:

\`\`\`bash
# Generate README for project
codex-subagents run ${vars.name} --prompt "Create a README for this project"

# Document specific function
codex-subagents run ${vars.name} --prompt "Document the calculateTotal function"
\`\`\`
`,

      testing: `---
name: ${vars.name}
description: ${vars.description || 'Creates comprehensive test suites'}
${vars.model ? `model: ${vars.model}` : ''}
tools:
  - git
  - node
  - npm
  - python3
keywords:
  - testing
  - unit-tests
  - integration-tests
---

# Test Generator

I specialize in creating comprehensive test suites for reliable code.

## Test Types:

- **Unit Tests**: Test individual functions and methods
- **Integration Tests**: Test component interactions
- **Edge Case Tests**: Test boundary conditions and error scenarios
- **Performance Tests**: Test execution speed and resource usage

## Testing Frameworks:

- **JavaScript**: Jest, Mocha, Vitest
- **Python**: pytest, unittest
- **Rust**: cargo test
- **Go**: built-in testing package

## Test Strategy:

1. Analyze code structure and dependencies
2. Identify critical paths and edge cases
3. Generate comprehensive test cases
4. Include proper setup and teardown
5. Ensure good test coverage

## Usage:

\`\`\`bash
# Generate tests for a function
codex-subagents run ${vars.name} --prompt "Create tests for the userAuth module"

# Create integration tests
codex-subagents run ${vars.name} --prompt "Generate integration tests for the API endpoints"
\`\`\`
`
    };

    return templates[templateType] || templates.basic;
  }

  /**
   * Delete an agent
   */
  async deleteAgent(name) {
    const agentPath = this.getAgentPath(name);

    if (!(await fs.pathExists(agentPath))) {
      throw new Error(`Agent '${name}' not found`);
    }

    await fs.remove(agentPath);
  }

  /**
   * Check if an agent exists
   */
  async agentExists(name) {
    const userAgentPath = this.getAgentPath(name);
    const builtinAgentPath = this.getBuiltinAgentPath(name);

    return (await fs.pathExists(userAgentPath)) || (await fs.pathExists(builtinAgentPath));
  }

  /**
   * Get agent by name
   */
  async getAgent(name) {
    // Try user agent first
    const userAgentPath = this.getAgentPath(name);
    if (await fs.pathExists(userAgentPath)) {
      return await this.loadAgent(userAgentPath, 'user');
    }

    // Try builtin agent
    const builtinAgentPath = this.getBuiltinAgentPath(name);
    if (await fs.pathExists(builtinAgentPath)) {
      return await this.loadAgent(builtinAgentPath, 'builtin');
    }

    throw new Error(`Agent '${name}' not found`);
  }

  /**
   * Validate agent definition
   */
  validateAgent(agent) {
    const errors = [];

    if (!agent.name) {
      errors.push('Agent name is required');
    }

    if (agent.name && !agent.name.match(/^[a-z0-9-]+$/)) {
      errors.push('Agent name must contain only lowercase letters, numbers, and hyphens');
    }

    if (!agent.instructions || agent.instructions.trim().length === 0) {
      errors.push('Agent instructions cannot be empty');
    }

    if (agent.tools && !Array.isArray(agent.tools)) {
      errors.push('Tools must be an array');
    }

    if (agent.keywords && !Array.isArray(agent.keywords)) {
      errors.push('Keywords must be an array');
    }

    if (errors.length > 0) {
      throw new Error(`Agent validation failed:\n${errors.join('\n')}`);
    }

    return true;
  }
}

module.exports = {
  AgentManager
};