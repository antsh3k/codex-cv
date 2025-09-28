const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const yaml = require('yaml');
const chalk = require('chalk');

/**
 * Configuration management for codex-subagents
 */
class ConfigManager {
  constructor() {
    this.homeDir = os.homedir();
    this.configDir = path.join(this.homeDir, '.codex-subagents');
    this.configFile = path.join(this.configDir, 'config.yaml');
    this.agentsDir = path.join(this.configDir, 'agents');

    // Default configuration
    this.defaultConfig = {
      version: '1.0.0',

      // AI Configuration
      ai: {
        provider: 'openai',
        model: 'gpt-4',
        apiKey: process.env.OPENAI_API_KEY || '',
        baseUrl: process.env.OPENAI_BASE_URL || 'https://api.openai.com/v1',
        temperature: 0.1,
        maxTokens: 4000
      },

      // Agent Configuration
      agents: {
        timeout: 300,         // 5 minutes default timeout
        maxRetries: 2,        // Retry failed executions
        autoSave: true,       // Save conversation history
        defaultTools: ['git', 'node', 'npm'],  // Default allowed tools
        allowCustomTools: true
      },

      // Output Configuration
      output: {
        format: 'text',       // text | json | markdown
        verbose: false,
        colors: true,
        showProgress: true,
        logLevel: 'info'      // debug | info | warn | error
      },

      // Git Integration
      git: {
        autoDetectRepo: true,
        requireCleanWorking: false,
        createBackups: true
      },

      // Security
      security: {
        allowedCommands: ['git', 'npm', 'node', 'cargo', 'python3', 'python'],
        blockedCommands: ['rm', 'sudo', 'chmod', 'chown'],
        requireConfirmation: ['git push', 'git reset --hard', 'rm -rf']
      }
    };
  }

  /**
   * Get the path to the configuration directory
   */
  getConfigDir() {
    return this.configDir;
  }

  /**
   * Get the path to the configuration file
   */
  getConfigFile() {
    return this.configFile;
  }

  /**
   * Get the path to the agents directory
   */
  getAgentsDir() {
    return this.agentsDir;
  }

  /**
   * Initialize configuration with default values
   */
  async initializeConfig(force = false) {
    // Create config directory if it doesn't exist
    await fs.ensureDir(this.configDir);
    await fs.ensureDir(this.agentsDir);

    // Check if config file already exists
    if (await fs.pathExists(this.configFile) && !force) {
      throw new Error(`Configuration already exists at ${this.configFile}. Use --force to overwrite.`);
    }

    // Write default configuration
    await this.saveConfig(this.defaultConfig);

    // Create example agent if agents directory is empty
    const agentFiles = await fs.readdir(this.agentsDir);
    if (agentFiles.length === 0) {
      await this.createExampleAgent();
    }

    return this.configFile;
  }

  /**
   * Load configuration from file
   */
  async loadConfig() {
    try {
      // Create default config if it doesn't exist
      if (!(await fs.pathExists(this.configFile))) {
        await this.initializeConfig();
      }

      const configContent = await fs.readFile(this.configFile, 'utf8');
      const config = yaml.parse(configContent);

      // Merge with defaults to ensure all fields are present
      return this.mergeWithDefaults(config);
    } catch (error) {
      if (error.code === 'ENOENT') {
        // Config file doesn't exist, create it
        await this.initializeConfig();
        return this.defaultConfig;
      }
      throw new Error(`Failed to load configuration: ${error.message}`);
    }
  }

  /**
   * Save configuration to file
   */
  async saveConfig(config) {
    await fs.ensureDir(this.configDir);

    const configYaml = yaml.stringify(config, {
      indent: 2,
      lineWidth: 120,
      minContentWidth: 0
    });

    await fs.writeFile(this.configFile, configYaml, 'utf8');
  }

  /**
   * Update specific configuration values
   */
  async updateConfig(updates) {
    const config = await this.loadConfig();
    const updatedConfig = this.deepMerge(config, updates);
    await this.saveConfig(updatedConfig);
    return updatedConfig;
  }

  /**
   * Get a specific configuration value
   */
  async getConfigValue(keyPath, defaultValue = undefined) {
    const config = await this.loadConfig();
    return this.getNestedValue(config, keyPath, defaultValue);
  }

  /**
   * Set a specific configuration value
   */
  async setConfigValue(keyPath, value) {
    const config = await this.loadConfig();
    this.setNestedValue(config, keyPath, value);
    await this.saveConfig(config);
  }

  /**
   * Validate configuration structure
   */
  validateConfig(config) {
    const errors = [];

    // Check required fields
    if (!config.ai) {
      errors.push('Missing ai configuration section');
    } else {
      if (!config.ai.provider) {
        errors.push('Missing ai.provider');
      }
      if (!config.ai.model) {
        errors.push('Missing ai.model');
      }
    }

    if (!config.agents) {
      errors.push('Missing agents configuration section');
    }

    if (errors.length > 0) {
      throw new Error(`Configuration validation failed:\n${errors.join('\n')}`);
    }

    return true;
  }

  /**
   * Create an example agent for new installations
   */
  async createExampleAgent() {
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

    const examplePath = path.join(this.agentsDir, 'example-helper.md');
    await fs.writeFile(examplePath, exampleAgent, 'utf8');
  }

  /**
   * Merge configuration with defaults
   */
  mergeWithDefaults(config) {
    return this.deepMerge(this.defaultConfig, config || {});
  }

  /**
   * Deep merge two objects
   */
  deepMerge(target, source) {
    const result = { ...target };

    for (const key in source) {
      if (source[key] !== null && typeof source[key] === 'object' && !Array.isArray(source[key])) {
        result[key] = this.deepMerge(target[key] || {}, source[key]);
      } else {
        result[key] = source[key];
      }
    }

    return result;
  }

  /**
   * Get nested value from object using dot notation
   */
  getNestedValue(obj, keyPath, defaultValue = undefined) {
    const keys = keyPath.split('.');
    let current = obj;

    for (const key of keys) {
      if (current === null || current === undefined || !(key in current)) {
        return defaultValue;
      }
      current = current[key];
    }

    return current;
  }

  /**
   * Set nested value in object using dot notation
   */
  setNestedValue(obj, keyPath, value) {
    const keys = keyPath.split('.');
    let current = obj;

    for (let i = 0; i < keys.length - 1; i++) {
      const key = keys[i];
      if (!(key in current) || typeof current[key] !== 'object') {
        current[key] = {};
      }
      current = current[key];
    }

    current[keys[keys.length - 1]] = value;
  }

  /**
   * Get configuration suitable for display
   */
  async getDisplayConfig() {
    const config = await this.loadConfig();

    // Hide sensitive information
    const displayConfig = JSON.parse(JSON.stringify(config));
    if (displayConfig.ai && displayConfig.ai.apiKey) {
      displayConfig.ai.apiKey = displayConfig.ai.apiKey ? '***HIDDEN***' : '';
    }

    return displayConfig;
  }

  /**
   * Check if configuration is valid and complete
   */
  async checkConfigHealth() {
    try {
      const config = await this.loadConfig();
      this.validateConfig(config);

      const issues = [];

      // Check API key
      if (!config.ai.apiKey) {
        issues.push('No API key configured. Set OPENAI_API_KEY environment variable or update config.');
      }

      // Check agents directory
      const agentFiles = await fs.readdir(this.agentsDir);
      const agentCount = agentFiles.filter(f => f.endsWith('.md')).length;

      return {
        valid: true,
        configFile: this.configFile,
        agentsDir: this.agentsDir,
        agentCount,
        issues,
        config: await this.getDisplayConfig()
      };
    } catch (error) {
      return {
        valid: false,
        error: error.message,
        configFile: this.configFile,
        agentsDir: this.agentsDir
      };
    }
  }
}

// Export singleton instance and functions
const configManager = new ConfigManager();

module.exports = {
  ConfigManager,
  configManager,
  loadConfig: () => configManager.loadConfig(),
  initializeConfig: (force) => configManager.initializeConfig(force),
  updateConfig: (updates) => configManager.updateConfig(updates),
  getConfigValue: (keyPath, defaultValue) => configManager.getConfigValue(keyPath, defaultValue),
  setConfigValue: (keyPath, value) => configManager.setConfigValue(keyPath, value),
  checkConfigHealth: () => configManager.checkConfigHealth(),
  getConfigDir: () => configManager.getConfigDir(),
  getAgentsDir: () => configManager.getAgentsDir()
};