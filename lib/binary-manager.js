const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const { spawn } = require('child_process');
const chalk = require('chalk');

/**
 * Manages the Rust binary for different platforms
 */
class BinaryManager {
  constructor() {
    this.packageRoot = path.join(__dirname, '..');
    this.vendorDir = path.join(this.packageRoot, 'vendor');
  }

  /**
   * Get the expected binary path for the current platform
   */
  getBinaryPath() {
    const platform = this.getPlatformTriple();
    const binaryName = process.platform === 'win32' ? 'codex-subagents.exe' : 'codex-subagents';
    return path.join(this.vendorDir, platform, binaryName);
  }

  /**
   * Get the platform triple (target triple) for the current system
   */
  getPlatformTriple() {
    const platform = process.platform;
    const arch = process.arch;

    // Map Node.js platform/arch to Rust target triples
    const platformMap = {
      'darwin': {
        'x64': 'x86_64-apple-darwin',
        'arm64': 'aarch64-apple-darwin'
      },
      'linux': {
        'x64': 'x86_64-unknown-linux-musl',
        'arm64': 'aarch64-unknown-linux-musl'
      },
      'win32': {
        'x64': 'x86_64-pc-windows-msvc',
        'arm64': 'aarch64-pc-windows-msvc'
      }
    };

    if (!platformMap[platform] || !platformMap[platform][arch]) {
      throw new Error(`Unsupported platform: ${platform} (${arch})`);
    }

    return platformMap[platform][arch];
  }

  /**
   * Check if the binary exists for the current platform
   */
  async binaryExists() {
    try {
      const binaryPath = this.getBinaryPath();
      const exists = await fs.pathExists(binaryPath);

      if (exists) {
        // Check if binary is executable
        try {
          await fs.access(binaryPath, fs.constants.X_OK);
          return true;
        } catch (error) {
          // Try to make it executable
          await fs.chmod(binaryPath, 0o755);
          return true;
        }
      }

      return false;
    } catch (error) {
      return false;
    }
  }

  /**
   * Ensure the binary exists and is ready to use
   */
  async ensureBinaryExists() {
    if (await this.binaryExists()) {
      return;
    }

    // Try to find binary in development environment
    const devBinaryPath = await this.findDevelopmentBinary();
    if (devBinaryPath) {
      console.log(chalk.yellow('⚠'), 'Using development binary:', devBinaryPath);
      return devBinaryPath;
    }

    // Binary not found - provide helpful error message
    const platform = this.getPlatformTriple();
    const expectedPath = this.getBinaryPath();

    throw new Error(`
Binary not found for platform ${platform}

Expected location: ${expectedPath}

This might happen if:
1. The package was not installed correctly
2. Your platform is not supported
3. The binary was corrupted during download

Try:
1. Reinstalling: npm uninstall -g codex-subagents && npm install -g codex-subagents
2. Running: codex-subagents doctor
3. Checking the repository: https://github.com/stat-guy/codex-cv

Supported platforms:
- macOS (x64, arm64)
- Linux (x64, arm64)
- Windows (x64, arm64)
`);
  }

  /**
   * Try to find a development binary (for local development)
   */
  async findDevelopmentBinary() {
    const possiblePaths = [
      // In rust-cli subdirectory
      path.join(this.packageRoot, 'rust-cli', 'target', 'release', 'codex-subagents'),
      path.join(this.packageRoot, 'rust-cli', 'target', 'release', 'codex-subagents.exe'),

      // In parent directory (if this is a dev environment)
      path.join(this.packageRoot, '..', 'target', 'release', 'codex-subagents'),
      path.join(this.packageRoot, '..', 'target', 'release', 'codex-subagents.exe'),

      // In codex-rs directory (original development location)
      path.join(this.packageRoot, 'codex-rs', 'target', 'release', 'codex'),
      path.join(this.packageRoot, 'codex-rs', 'target', 'release', 'codex.exe'),
    ];

    for (const possiblePath of possiblePaths) {
      try {
        if (await fs.pathExists(possiblePath)) {
          // Verify it's executable
          await fs.access(possiblePath, fs.constants.X_OK);
          return possiblePath;
        }
      } catch (error) {
        // Continue checking other paths
      }
    }

    return null;
  }

  /**
   * Execute the binary with given arguments
   */
  async executeBinary(args = [], options = {}) {
    await this.ensureBinaryExists();

    const binaryPath = this.getBinaryPath();
    const devBinaryPath = await this.findDevelopmentBinary();
    const actualBinaryPath = devBinaryPath || binaryPath;

    return new Promise((resolve, reject) => {
      const child = spawn(actualBinaryPath, args, {
        stdio: options.stdio || 'inherit',
        env: {
          ...process.env,
          ...options.env,
          // Enable subagents by default for our standalone tool
          CODEX_SUBAGENTS_ENABLED: '1',
          // Set log level
          RUST_LOG: options.verbose ? 'debug' : 'info'
        }
      });

      child.on('exit', (code) => {
        if (code === 0) {
          resolve({ code, stdout: '', stderr: '' });
        } else {
          reject(new Error(`Binary exited with code ${code}`));
        }
      });

      child.on('error', (error) => {
        reject(new Error(`Failed to execute binary: ${error.message}`));
      });
    });
  }

  /**
   * Get version information from the binary
   */
  async getBinaryVersion() {
    try {
      await this.ensureBinaryExists();

      const binaryPath = this.getBinaryPath();
      const devBinaryPath = await this.findDevelopmentBinary();
      const actualBinaryPath = devBinaryPath || binaryPath;

      return new Promise((resolve, reject) => {
        const child = spawn(actualBinaryPath, ['--version'], {
          stdio: ['ignore', 'pipe', 'pipe'],
          env: { ...process.env, CODEX_SUBAGENTS_ENABLED: '1' }
        });

        let stdout = '';
        let stderr = '';

        child.stdout.on('data', (data) => stdout += data.toString());
        child.stderr.on('data', (data) => stderr += data.toString());

        child.on('exit', (code) => {
          if (code === 0) {
            resolve(stdout.trim());
          } else {
            reject(new Error(`Failed to get version: ${stderr}`));
          }
        });

        child.on('error', (error) => {
          reject(new Error(`Failed to execute binary: ${error.message}`));
        });
      });
    } catch (error) {
      return 'unknown';
    }
  }

  /**
   * Check if the binary is working correctly
   */
  async checkBinaryHealth() {
    try {
      const version = await this.getBinaryVersion();
      const binaryPath = this.getBinaryPath();
      const exists = await this.binaryExists();

      return {
        healthy: true,
        version,
        path: binaryPath,
        exists,
        platform: this.getPlatformTriple()
      };
    } catch (error) {
      return {
        healthy: false,
        error: error.message,
        path: this.getBinaryPath(),
        exists: await this.binaryExists(),
        platform: this.getPlatformTriple()
      };
    }
  }
}

// Export singleton instance and individual functions
const binaryManager = new BinaryManager();

module.exports = {
  BinaryManager,
  binaryManager,
  getBinaryPath: () => binaryManager.getBinaryPath(),
  ensureBinaryExists: () => binaryManager.ensureBinaryExists(),
  executeBinary: (args, options) => binaryManager.executeBinary(args, options),
  getBinaryVersion: () => binaryManager.getBinaryVersion(),
  checkBinaryHealth: () => binaryManager.checkBinaryHealth()
};