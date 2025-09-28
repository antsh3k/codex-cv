# Creating Standalone `codex-subagents` NPM Package

## Project Overview
Create a standalone npm package called `codex-subagents` that users can install globally and use independently of the main Codex CLI. This will provide immediate access to our subagents framework.

**Package Name**: `codex-subagents`
**CLI Command**: `codex-subagents` (alias: `cs`)
**Repository**: https://github.com/stat-guy/codex-cv
**Installation**: `npm i -g codex-subagents`

---

## Phase 1: Package Architecture & Configuration

### ✅ Design standalone npm package architecture for codex-subagents
- [x] Define project structure with bin/, lib/, agents/, docs/ directories
- [x] Plan cross-platform binary distribution strategy
- [x] Design CLI interface with main commands (list, run, status, create, init, doctor)

### ✅ Create new package.json and bin configuration for codex-subagents command
- [x] Set up package.json with proper bin configuration
- [x] Configure npm scripts for building and testing
- [x] Add dependencies for CLI framework (commander, chalk, inquirer)
- [x] Set up files array for npm distribution

### ✅ Create JavaScript CLI wrapper for the Rust binary
- [x] Create bin/codex-subagents.js as main entry point
- [x] Implement lib/binary-manager.js for cross-platform binary handling
- [x] Implement lib/config.js for configuration management
- [x] Implement lib/agent-manager.js for agent operations
- [x] Add proper error handling and user-friendly messages

---

## Phase 2: Rust Implementation

### ☐ Extract core subagents Rust code into standalone binary
- [ ] Create rust-cli/ directory with minimal Cargo.toml
- [ ] Extract subagents framework from existing codebase
- [ ] Create standalone CLI that doesn't depend on full Codex
- [ ] Implement core commands: list, run, status
- [ ] Add configuration file support (YAML/TOML)

### ✅ Set up cross-platform build pipeline for npm distribution
- [x] Configure Cargo.toml for cross-compilation targets
- [x] Set up build script for multiple platforms (macOS, Linux, Windows)
- [x] Create vendor/ directory structure for platform-specific binaries
- [x] Implement binary selection logic in JavaScript wrapper
- [x] Create GitHub Actions workflow for automated builds
- [x] Set up npm packaging and publishing automation

---

## Phase 3: Asset Packaging

### ☐ Package example agents and documentation for distribution
- [ ] Copy example agents to agents/ directory
- [ ] Create user-friendly documentation in docs/
- [ ] Add README.md with installation and usage instructions
- [ ] Create LICENSE file (MIT)
- [ ] Set up .npmignore to exclude development files

### ☐ Create post-install setup scripts
- [ ] Implement scripts/postinstall.js for initial setup
- [ ] Create default configuration directory (~/.codex-subagents/)
- [ ] Copy example agents to user directory
- [ ] Set up initial configuration file

---

## Phase 4: Automation & CI/CD

### ☐ Configure GitHub Actions for automated building and publishing
- [ ] Create .github/workflows/build.yml for cross-platform builds
- [ ] Create .github/workflows/publish.yml for npm publishing
- [ ] Set up automated testing pipeline
- [ ] Configure release automation with version tagging

### ☐ Set up development and testing infrastructure
- [ ] Create test/ directory with test scripts
- [ ] Add integration tests for CLI commands
- [ ] Set up local development scripts
- [ ] Create contributor documentation

---

## Phase 5: Testing & Validation

### ☐ Test installation and usage flow end-to-end
- [ ] Test npm install -g codex-subagents locally
- [ ] Verify all CLI commands work correctly
- [ ] Test on multiple platforms (macOS, Linux, Windows)
- [ ] Validate agent creation and execution
- [ ] Test configuration management

### ☐ User experience validation
- [ ] Create getting started guide
- [ ] Test with fresh user environment
- [ ] Validate error messages and help text
- [ ] Ensure proper cleanup on uninstall

---

## Phase 6: Publishing & Distribution

### ☐ Push to stat-guy/codex-cv repository and publish to npm
- [ ] Push final code to GitHub repository
- [ ] Create release tags and release notes
- [ ] Publish to npm registry
- [ ] Update repository README with installation instructions
- [ ] Announce release and gather feedback

### ☐ Post-launch support
- [ ] Monitor for issues and user feedback
- [ ] Set up issue templates in GitHub
- [ ] Create troubleshooting documentation
- [ ] Plan for future updates and maintenance

---

## Implementation Priority

### High Priority (Must Have)
- [x] Package.json configuration
- [x] CLI wrapper architecture
- [ ] Core Rust binary extraction
- [ ] Basic agent functionality (list, run)
- [ ] Cross-platform builds

### Medium Priority (Should Have)
- [ ] Configuration management
- [ ] Agent creation tools
- [ ] GitHub Actions automation
- [ ] Comprehensive testing

### Low Priority (Nice to Have)
- [ ] Advanced CLI features
- [ ] Extensive documentation
- [ ] Community features
- [ ] Plugin system

---

## Technical Notes

### Dependencies
- **Node.js**: >= 16.0.0
- **Rust**: Latest stable for building
- **Platform Support**: macOS (x64, arm64), Linux (x64, arm64), Windows (x64, arm64)

### File Structure
```
codex-subagents/
├── package.json                    ✅ Created
├── bin/codex-subagents.js          ✅ Created
├── lib/                           ⏳ In Progress
│   ├── binary-manager.js          ☐ TODO
│   ├── config.js                  ☐ TODO
│   └── agent-manager.js           ☐ TODO
├── rust-cli/                      ☐ TODO
│   ├── Cargo.toml                 ☐ TODO
│   └── src/main.rs                ☐ TODO
├── agents/                        ☐ TODO
│   ├── code-reviewer.md           ☐ TODO
│   ├── doc-writer.md              ☐ TODO
│   ├── test-generator.md          ☐ TODO
│   └── bug-hunter.md              ☐ TODO
├── docs/                          ☐ TODO
├── scripts/                       ☐ TODO
├── test/                          ☐ TODO
├── .github/workflows/             ☐ TODO
├── README.md                      ☐ TODO
└── LICENSE                        ☐ TODO
```

---

## Success Criteria

- [ ] Users can install with `npm i -g codex-subagents`
- [ ] CLI responds to `codex-subagents --help`
- [ ] Built-in agents work out of the box
- [ ] Cross-platform compatibility confirmed
- [ ] Documentation is clear and complete
- [ ] GitHub repository is properly organized
- [ ] npm package is published and accessible

---

*This document will be updated as tasks are completed. Check off items as they're finished and add notes for any issues or changes.*