#!/usr/bin/env node

const { program } = require('commander');
const chalk = require('chalk');
const path = require('path');
const fs = require('fs-extra');
const { spawn } = require('child_process');
const which = require('which');

// Package info
const packageJson = require('../package.json');

// Binary management
const { getBinaryPath, ensureBinaryExists } = require('../lib/binary-manager');
const { loadConfig, initializeConfig } = require('../lib/config');
const { AgentManager } = require('../lib/agent-manager');

// Global error handling
process.on('unhandledRejection', (reason, promise) => {
  console.error(chalk.red('✗ Unhandled Rejection at:'), promise, chalk.red('reason:'), reason);
  process.exit(1);
});

// Main program configuration
program
  .name('codex-subagents')
  .description('AI subagents for specialized coding tasks')
  .version(packageJson.version)
  .option('-v, --verbose', 'enable verbose logging')
  .option('--config <path>', 'path to configuration file')
  .option('--no-color', 'disable colored output');

// List available agents
program
  .command('list')
  .alias('ls')
  .description('list available subagents')
  .option('-a, --all', 'show all agents including examples')
  .option('-j, --json', 'output as JSON')
  .action(async (options) => {
    try {
      await ensureBinaryExists();
      const agentManager = new AgentManager();
      await agentManager.listAgents(options);
    } catch (error) {
      console.error(chalk.red('✗ Error listing agents:'), error.message);
      if (options.verbose) console.error(error);
      process.exit(1);
    }
  });

// Run a specific agent
program
  .command('run <agent>')
  .description('run a specific subagent')
  .option('-p, --prompt <text>', 'prompt to send to the agent')
  .option('-w, --wait', 'wait for agent completion', true)
  .option('--no-wait', 'don\'t wait for agent completion')
  .option('-o, --output <format>', 'output format (text|json)', 'text')
  .action(async (agentName, options) => {
    try {
      await ensureBinaryExists();

      // Get prompt from stdin if not provided via option
      let prompt = options.prompt;
      if (!prompt && !process.stdin.isTTY) {
        prompt = await readStdin();
      }

      // Interactive prompt if still no prompt provided
      if (!prompt) {
        const inquirer = require('inquirer');
        const answer = await inquirer.prompt([
          {
            type: 'input',
            name: 'prompt',
            message: `Enter prompt for ${chalk.cyan(agentName)}:`,
            validate: (input) => input.trim().length > 0 || 'Prompt cannot be empty'
          }
        ]);
        prompt = answer.prompt;
      }

      const binaryPath = getBinaryPath();
      const args = ['run', agentName, '--prompt', prompt];

      if (!options.wait) args.push('--no-wait');
      if (options.output === 'json') args.push('--json');
      if (program.opts().verbose) args.push('--verbose');

      console.log(chalk.blue('▶'), `Starting agent ${chalk.cyan(agentName)}...`);

      const child = spawn(binaryPath, args, {
        stdio: 'inherit',
        env: { ...process.env, RUST_LOG: program.opts().verbose ? 'debug' : 'info' }
      });

      child.on('exit', (code) => {
        if (code === 0) {
          console.log(chalk.green('✓'), `Agent ${chalk.cyan(agentName)} completed successfully`);
        } else {
          console.error(chalk.red('✗'), `Agent ${chalk.cyan(agentName)} failed with exit code ${code}`);
          process.exit(code);
        }
      });

      child.on('error', (error) => {
        console.error(chalk.red('✗ Error running agent:'), error.message);
        process.exit(1);
      });

    } catch (error) {
      console.error(chalk.red('✗ Error running agent:'), error.message);
      if (program.opts().verbose) console.error(error);
      process.exit(1);
    }
  });

// Show agent status
program
  .command('status')
  .description('show running agent status')
  .option('-w, --watch', 'watch for status changes')
  .action(async (options) => {
    try {
      await ensureBinaryExists();
      const binaryPath = getBinaryPath();
      const args = ['status'];

      if (options.watch) args.push('--watch');
      if (program.opts().verbose) args.push('--verbose');

      const child = spawn(binaryPath, args, {
        stdio: 'inherit',
        env: { ...process.env, RUST_LOG: program.opts().verbose ? 'debug' : 'info' }
      });

      child.on('error', (error) => {
        console.error(chalk.red('✗ Error checking status:'), error.message);
        process.exit(1);
      });

    } catch (error) {
      console.error(chalk.red('✗ Error checking status:'), error.message);
      if (program.opts().verbose) console.error(error);
      process.exit(1);
    }
  });

// Create a new agent
program
  .command('create <name>')
  .description('create a new subagent')
  .option('-t, --template <type>', 'agent template (basic|code-review|docs|testing)', 'basic')
  .option('-d, --description <text>', 'agent description')
  .option('--tools <list>', 'comma-separated list of allowed tools')
  .option('--model <name>', 'preferred model for this agent')
  .option('--edit', 'open agent file in editor after creation')
  .action(async (name, options) => {
    try {
      const agentManager = new AgentManager();
      await agentManager.createAgent(name, options);

      console.log(chalk.green('✓'), `Created agent ${chalk.cyan(name)}`);

      if (options.edit) {
        const editor = process.env.EDITOR || 'nano';
        const agentPath = agentManager.getAgentPath(name);
        const { spawn } = require('child_process');
        spawn(editor, [agentPath], { stdio: 'inherit' });
      }
    } catch (error) {
      console.error(chalk.red('✗ Error creating agent:'), error.message);
      if (program.opts().verbose) console.error(error);
      process.exit(1);
    }
  });

// Initialize configuration
program
  .command('init')
  .description('initialize codex-subagents configuration')
  .option('-f, --force', 'overwrite existing configuration')
  .action(async (options) => {
    try {
      await initializeConfig(options.force);
      console.log(chalk.green('✓'), 'Configuration initialized successfully');
      console.log(chalk.blue('ℹ'), 'Edit ~/.codex-subagents/config.yaml to customize settings');
      console.log(chalk.blue('ℹ'), 'Add agents to ~/.codex-subagents/agents/ directory');
    } catch (error) {
      console.error(chalk.red('✗ Error initializing configuration:'), error.message);
      if (program.opts().verbose) console.error(error);
      process.exit(1);
    }
  });

// Doctor command - check installation and dependencies
program
  .command('doctor')
  .description('check installation and diagnose issues')
  .action(async () => {
    console.log(chalk.blue('🔍 Diagnosing codex-subagents installation...\n'));

    const checks = [
      {
        name: 'Node.js version',
        check: () => {
          const version = process.version;
          const major = parseInt(version.slice(1).split('.')[0]);
          return { success: major >= 16, info: version };
        }
      },
      {
        name: 'Binary exists',
        check: async () => {
          try {
            const binaryPath = getBinaryPath();
            const exists = await fs.pathExists(binaryPath);
            return { success: exists, info: binaryPath };
          } catch (error) {
            return { success: false, info: error.message };
          }
        }
      },
      {
        name: 'Git availability',
        check: async () => {
          try {
            const gitPath = await which('git');
            return { success: true, info: gitPath };
          } catch (error) {
            return { success: false, info: 'Git not found in PATH' };
          }
        }
      },
      {
        name: 'Configuration',
        check: async () => {
          try {
            const config = await loadConfig();
            return { success: true, info: 'Configuration loaded successfully' };
          } catch (error) {
            return { success: false, info: error.message };
          }
        }
      }
    ];

    for (const check of checks) {
      try {
        const result = await check.check();
        const icon = result.success ? chalk.green('✓') : chalk.red('✗');
        console.log(`${icon} ${check.name}: ${result.info}`);
      } catch (error) {
        console.log(`${chalk.red('✗')} ${check.name}: ${error.message}`);
      }
    }

    console.log(chalk.blue('\n📖 For help and documentation, visit:'));
    console.log(chalk.blue('   https://github.com/stat-guy/codex-cv'));
  });

// Utility function to read from stdin
function readStdin() {
  return new Promise((resolve) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => data += chunk);
    process.stdin.on('end', () => resolve(data.trim()));
  });
}

// Handle unknown commands
program.on('command:*', (operands) => {
  console.error(chalk.red(`✗ Unknown command: ${operands[0]}`));
  console.log(chalk.blue('ℹ Run'), chalk.cyan('codex-subagents --help'), chalk.blue('for available commands'));
  process.exit(1);
});

// Parse command line arguments
program.parse();

// Show help if no command provided
if (!process.argv.slice(2).length) {
  console.log(chalk.blue('🤖 Codex Subagents'), chalk.gray(`v${packageJson.version}`));
  console.log(chalk.blue('   AI agents for specialized coding tasks\n'));

  program.outputHelp();

  console.log(chalk.blue('\n💡 Quick start:'));
  console.log(chalk.gray('   codex-subagents init          # Initialize configuration'));
  console.log(chalk.gray('   codex-subagents list          # List available agents'));
  console.log(chalk.gray('   codex-subagents run code-reviewer  # Run code review agent'));

  console.log(chalk.blue('\n📖 Documentation:'));
  console.log(chalk.gray('   https://github.com/stat-guy/codex-cv'));
}