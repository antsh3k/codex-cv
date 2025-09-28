#!/usr/bin/env node

const { spawn } = require('child_process');
const fs = require('fs-extra');
const path = require('path');
const chalk = require('chalk');

// Target platforms for cross-compilation
const TARGETS = [
  {
    target: 'x86_64-apple-darwin',
    platform: 'darwin',
    arch: 'x64',
    ext: ''
  },
  {
    target: 'aarch64-apple-darwin',
    platform: 'darwin',
    arch: 'arm64',
    ext: ''
  },
  {
    target: 'x86_64-unknown-linux-musl',
    platform: 'linux',
    arch: 'x64',
    ext: ''
  },
  {
    target: 'aarch64-unknown-linux-musl',
    platform: 'linux',
    arch: 'arm64',
    ext: ''
  },
  {
    target: 'x86_64-pc-windows-msvc',
    platform: 'win32',
    arch: 'x64',
    ext: '.exe'
  },
  {
    target: 'aarch64-pc-windows-msvc',
    platform: 'win32',
    arch: 'arm64',
    ext: '.exe'
  }
];

const ROOT_DIR = path.join(__dirname, '..');
const RUST_CLI_DIR = path.join(ROOT_DIR, 'rust-cli');
const VENDOR_DIR = path.join(ROOT_DIR, 'vendor');
const BINARY_NAME = 'codex-subagents';

async function main() {
  console.log(chalk.blue('🔨 Building cross-platform binaries...\n'));

  // Ensure vendor directory exists
  await fs.ensureDir(VENDOR_DIR);

  // Check if Rust is installed
  try {
    await execCommand('cargo', ['--version'], { cwd: RUST_CLI_DIR });
  } catch (error) {
    console.error(chalk.red('❌ Cargo not found. Please install Rust: https://rustup.rs/'));
    process.exit(1);
  }

  // Install targets
  console.log(chalk.blue('📦 Installing compilation targets...'));
  for (const { target } of TARGETS) {
    try {
      await execCommand('rustup', ['target', 'add', target]);
      console.log(chalk.green(`✅ Added target: ${target}`));
    } catch (error) {
      console.warn(chalk.yellow(`⚠️ Failed to add target ${target}: ${error.message}`));
    }
  }

  console.log();

  // Build for each target
  const results = [];
  for (const targetInfo of TARGETS) {
    const { target, ext } = targetInfo;
    console.log(chalk.blue(`🔧 Building for ${target}...`));

    try {
      // Build the binary
      await execCommand('cargo', [
        'build',
        '--release',
        '--target',
        target
      ], { cwd: RUST_CLI_DIR });

      // Copy binary to vendor directory
      const sourcePath = path.join(RUST_CLI_DIR, 'target', target, 'release', `${BINARY_NAME}${ext}`);
      const targetDir = path.join(VENDOR_DIR, target);
      const targetPath = path.join(targetDir, `${BINARY_NAME}${ext}`);

      await fs.ensureDir(targetDir);
      await fs.copy(sourcePath, targetPath);

      // Make binary executable on Unix systems
      if (!ext) {
        await fs.chmod(targetPath, 0o755);
      }

      console.log(chalk.green(`✅ Built and copied to vendor/${target}/`));
      results.push({ ...targetInfo, success: true, path: targetPath });

    } catch (error) {
      console.error(chalk.red(`❌ Failed to build ${target}: ${error.message}`));
      results.push({ ...targetInfo, success: false, error: error.message });
    }
  }

  // Summary
  console.log(chalk.blue('\n📊 Build Summary:'));
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);

  console.log(chalk.green(`✅ Successful builds: ${successful.length}`));
  if (failed.length > 0) {
    console.log(chalk.red(`❌ Failed builds: ${failed.length}`));
    failed.forEach(f => console.log(chalk.red(`  - ${f.target}: ${f.error}`)));
  }

  // Write build manifest
  const manifest = {
    buildTime: new Date().toISOString(),
    targets: results,
    binaryName: BINARY_NAME,
    version: require('../package.json').version
  };

  await fs.writeJson(path.join(VENDOR_DIR, 'build-manifest.json'), manifest, { spaces: 2 });
  console.log(chalk.blue(`\n📄 Build manifest written to vendor/build-manifest.json`));

  if (failed.length > 0) {
    console.log(chalk.yellow('\n⚠️ Some builds failed. You may need to install additional tools:'));
    console.log('- For Windows targets: Install Visual Studio Build Tools');
    console.log('- For Linux targets: Install musl-tools (apt-get install musl-tools)');
    console.log('- For cross-compilation: Install cross (cargo install cross)');
  } else {
    console.log(chalk.green('\n🎉 All builds completed successfully!'));
  }
}

function execCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
      ...options
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => stdout += data.toString());
    child.stderr.on('data', (data) => stderr += data.toString());

    child.on('exit', (code) => {
      if (code === 0) {
        resolve({ stdout, stderr });
      } else {
        reject(new Error(`Command failed with code ${code}: ${stderr || stdout}`));
      }
    });

    child.on('error', (error) => {
      reject(error);
    });
  });
}

if (require.main === module) {
  main().catch(error => {
    console.error(chalk.red('❌ Build failed:'), error.message);
    process.exit(1);
  });
}

module.exports = { main };