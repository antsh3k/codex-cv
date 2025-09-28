# Subagents Platform Validation Guide

This document outlines the validation strategy for ensuring Codex Subagents works reliably across all supported platforms with proper feature toggle behavior.

## Platform Support Matrix

| Platform | Architecture | Status | Validation Required |
|----------|-------------|--------|-------------------|
| macOS | x86_64 | ✅ Supported | Full validation |
| macOS | aarch64 (Apple Silicon) | ✅ Supported | Full validation |
| Linux | x86_64 | ✅ Supported | Full validation |
| Linux | aarch64 | ✅ Supported | Full validation |
| Windows | x86_64 | ✅ Supported | Full validation |
| Windows | aarch64 | ✅ Supported | Full validation |

## Feature Toggle Testing Strategy

### Configuration Methods

The subagents feature can be controlled through multiple mechanisms, tested in priority order:

1. **Environment Variable** (highest priority)
   ```bash
   export CODEX_SUBAGENTS_ENABLED=1  # Enable
   export CODEX_SUBAGENTS_ENABLED=0  # Disable
   ```

2. **Configuration File** (medium priority)
   ```toml
   # ~/.codex/config.toml
   [subagents]
   enabled = true   # Enable
   enabled = false  # Disable
   ```

3. **Default Behavior** (lowest priority)
   - Default: `enabled = false` (feature disabled)

### Test Matrix

Each platform must be validated with all toggle combinations:

| Environment Var | Config File | Expected Result | Test Scenario |
|----------------|-------------|----------------|---------------|
| `1` | `true` | Enabled | Both enable |
| `1` | `false` | Enabled | Env overrides config |
| `0` | `true` | Disabled | Env overrides config |
| `0` | `false` | Disabled | Both disable |
| unset | `true` | Enabled | Config only |
| unset | `false` | Disabled | Config only |
| unset | unset | Disabled | Default behavior |

## Validation Test Suite

### Platform-Specific Validation Script

```bash
#!/bin/bash
# scripts/validate-platform.sh

set -euo pipefail

PLATFORM=$(uname -s)
ARCH=$(uname -m)
CODEX_BINARY="./target/release/codex"

echo "=== Validating Codex Subagents on $PLATFORM $ARCH ==="

# Ensure clean environment
unset CODEX_SUBAGENTS_ENABLED
rm -f ~/.codex/config.toml

# Test 1: Default disabled behavior
echo "Test 1: Default behavior (should be disabled)"
if $CODEX_BINARY subagents list 2>&1 | grep -q "feature is not enabled"; then
    echo "✅ Default disabled behavior correct"
else
    echo "❌ Default behavior incorrect"
    exit 1
fi

# Test 2: Environment variable enable
echo "Test 2: Environment variable enable"
export CODEX_SUBAGENTS_ENABLED=1
if $CODEX_BINARY subagents list | grep -q "code-reviewer"; then
    echo "✅ Environment variable enable works"
else
    echo "❌ Environment variable enable failed"
    exit 1
fi

# Test 3: Environment variable disable override
echo "Test 3: Environment variable disable override"
mkdir -p ~/.codex
cat > ~/.codex/config.toml << 'EOF'
[subagents]
enabled = true
EOF
export CODEX_SUBAGENTS_ENABLED=0
if $CODEX_BINARY subagents list 2>&1 | grep -q "feature is not enabled"; then
    echo "✅ Environment variable override works"
else
    echo "❌ Environment variable override failed"
    exit 1
fi

# Test 4: Configuration file enable
echo "Test 4: Configuration file enable"
unset CODEX_SUBAGENTS_ENABLED
cat > ~/.codex/config.toml << 'EOF'
[subagents]
enabled = true
EOF
if $CODEX_BINARY subagents list | grep -q "code-reviewer"; then
    echo "✅ Configuration file enable works"
else
    echo "❌ Configuration file enable failed"
    exit 1
fi

# Test 5: Basic agent execution
echo "Test 5: Basic agent execution"
if echo "Test prompt" | $CODEX_BINARY subagents run code-reviewer --no-wait; then
    echo "✅ Basic agent execution works"
else
    echo "❌ Basic agent execution failed"
    exit 1
fi

# Test 6: Performance impact when disabled
echo "Test 6: Performance impact when disabled"
export CODEX_SUBAGENTS_ENABLED=0
start_time=$(date +%s%N)
$CODEX_BINARY --version >/dev/null
end_time=$(date +%s%N)
disabled_duration=$((($end_time - $start_time) / 1000000))

export CODEX_SUBAGENTS_ENABLED=1
start_time=$(date +%s%N)
$CODEX_BINARY --version >/dev/null
end_time=$(date +%s%N)
enabled_duration=$((($end_time - $start_time) / 1000000))

# Allow up to 10% performance overhead
overhead_percent=$(((enabled_duration - disabled_duration) * 100 / disabled_duration))
if [ $overhead_percent -le 10 ]; then
    echo "✅ Performance impact acceptable: ${overhead_percent}%"
else
    echo "❌ Performance impact too high: ${overhead_percent}%"
    exit 1
fi

echo "✅ All platform validation tests passed on $PLATFORM $ARCH"
```

### Cross-Platform Build Validation

```bash
#!/bin/bash
# scripts/validate-builds.sh

# Build for all supported platforms
platforms=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "x86_64-pc-windows-msvc"
    "aarch64-pc-windows-msvc"
)

for platform in "${platforms[@]}"; do
    echo "Building for $platform..."

    # Cross-compile
    cargo build --release --target $platform

    # Verify subagents crates are included
    if cargo tree --target $platform | grep -q "codex-subagents"; then
        echo "✅ Subagents crate included in $platform build"
    else
        echo "❌ Subagents crate missing from $platform build"
        exit 1
    fi

    # Verify TypeScript bindings generated
    if [ -f "target/$platform/release/codex-protocol-ts" ]; then
        echo "✅ TypeScript bindings available for $platform"
    else
        echo "❌ TypeScript bindings missing for $platform"
        exit 1
    fi
done

echo "✅ All cross-platform builds validated"
```

### Binary Distribution Validation

```bash
#!/bin/bash
# scripts/validate-distribution.sh

# Test npm package includes subagent assets
echo "Validating npm package contents..."

# Stage the package
python3 codex-cli/scripts/build_npm_package.py \
    --version "0.0.0-test" \
    --staging-dir "/tmp/codex-staging"

# Verify subagent assets included
if [ -d "/tmp/codex-staging/examples/agents" ]; then
    echo "✅ Example agents included in distribution"
else
    echo "❌ Example agents missing from distribution"
    exit 1
fi

# Verify required example agents present
required_agents=("code-reviewer.md" "doc-writer.md" "test-generator.md" "bug-hunter.md")
for agent in "${required_agents[@]}"; do
    if [ -f "/tmp/codex-staging/examples/agents/$agent" ]; then
        echo "✅ $agent present"
    else
        echo "❌ $agent missing"
        exit 1
    fi
done

# Verify TypeScript types generated
if ls /tmp/codex-staging/bin/*.d.ts 1> /dev/null 2>&1; then
    echo "✅ TypeScript definitions included"
else
    echo "❌ TypeScript definitions missing"
    exit 1
fi

echo "✅ Distribution validation passed"
```

## Manual Testing Procedures

### macOS Testing

```bash
# Download and test macOS binaries
wget https://github.com/openai/codex/releases/latest/download/codex-aarch64-apple-darwin.tar.gz
tar -xzf codex-aarch64-apple-darwin.tar.gz
mv codex-aarch64-apple-darwin codex
chmod +x codex

# Test feature toggles
./scripts/validate-platform.sh

# Test with different shells
bash -c "./codex subagents list"
zsh -c "./codex subagents list"
fish -c "./codex subagents list"

# Test with different macOS versions
# - macOS 11 (Big Sur)
# - macOS 12 (Monterey)
# - macOS 13 (Ventura)
# - macOS 14 (Sonoma)
# - macOS 15 (Sequoia)
```

### Linux Testing

```bash
# Test on different Linux distributions
distributions=(
    "ubuntu:20.04"
    "ubuntu:22.04"
    "ubuntu:24.04"
    "alpine:3.19"
    "fedora:39"
    "debian:12"
    "centos:stream9"
)

for distro in "${distributions[@]}"; do
    echo "Testing on $distro..."

    docker run --rm -v $(pwd):/workspace $distro bash -c "
        cd /workspace
        ./scripts/validate-platform.sh
    "
done
```

### Windows Testing

```powershell
# PowerShell testing script
# scripts/validate-windows.ps1

$ErrorActionPreference = "Stop"

Write-Host "=== Validating Codex Subagents on Windows ==="

# Download Windows binary
Invoke-WebRequest -Uri "https://github.com/openai/codex/releases/latest/download/codex-x86_64-pc-windows-msvc.zip" -OutFile "codex-windows.zip"
Expand-Archive -Path "codex-windows.zip" -DestinationPath "."
Rename-Item -Path "codex-x86_64-pc-windows-msvc.exe" -NewName "codex.exe"

# Test default disabled
$env:CODEX_SUBAGENTS_ENABLED = $null
$output = & .\codex.exe subagents list 2>&1
if ($output -match "feature is not enabled") {
    Write-Host "✅ Default disabled behavior correct"
} else {
    Write-Host "❌ Default behavior incorrect"
    exit 1
}

# Test environment enable
$env:CODEX_SUBAGENTS_ENABLED = "1"
$output = & .\codex.exe subagents list
if ($output -match "code-reviewer") {
    Write-Host "✅ Environment variable enable works"
} else {
    Write-Host "❌ Environment variable enable failed"
    exit 1
}

# Test configuration file
$env:CODEX_SUBAGENTS_ENABLED = $null
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.codex"
@"
[subagents]
enabled = true
"@ | Out-File -FilePath "$env:USERPROFILE\.codex\config.toml" -Encoding UTF8

$output = & .\codex.exe subagents list
if ($output -match "code-reviewer") {
    Write-Host "✅ Configuration file enable works"
} else {
    Write-Host "❌ Configuration file enable failed"
    exit 1
}

Write-Host "✅ All Windows validation tests passed"
```

## Automated CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/platform-validation.yml
name: Platform Validation

on:
  push:
    branches: [main, add-subagent-analysis-docs]
  pull_request:
    branches: [main]

jobs:
  validate-platforms:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        feature-flag: [enabled, disabled]

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build Codex
      run: cargo build --release

    - name: Set feature flag
      run: |
        if [ "${{ matrix.feature-flag }}" = "enabled" ]; then
          echo "CODEX_SUBAGENTS_ENABLED=1" >> $GITHUB_ENV
        else
          echo "CODEX_SUBAGENTS_ENABLED=0" >> $GITHUB_ENV
        fi
      shell: bash

    - name: Run platform validation (Unix)
      if: runner.os != 'Windows'
      run: ./scripts/validate-platform.sh

    - name: Run platform validation (Windows)
      if: runner.os == 'Windows'
      run: .\scripts\validate-windows.ps1
      shell: powershell

    - name: Upload test results
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: test-results-${{ matrix.os }}-${{ matrix.feature-flag }}
        path: test-results/

  validate-cross-compilation:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust with targets
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        targets: |
          x86_64-apple-darwin
          aarch64-apple-darwin
          x86_64-unknown-linux-musl
          aarch64-unknown-linux-musl
          x86_64-pc-windows-msvc
          aarch64-pc-windows-msvc

    - name: Cross-compile validation
      run: ./scripts/validate-builds.sh

    - name: Distribution validation
      run: ./scripts/validate-distribution.sh
```

## Performance Benchmarks

### Baseline Measurements

```bash
#!/bin/bash
# scripts/performance-benchmarks.sh

echo "=== Performance Benchmarks ==="

# Startup time measurement
measure_startup() {
    local flag_setting=$1
    export CODEX_SUBAGENTS_ENABLED=$flag_setting

    local total_time=0
    local iterations=10

    for i in $(seq 1 $iterations); do
        start_time=$(date +%s%N)
        ./target/release/codex --version >/dev/null 2>&1
        end_time=$(date +%s%N)
        duration=$(($end_time - $start_time))
        total_time=$(($total_time + $duration))
    done

    average_time=$(($total_time / $iterations / 1000000))
    echo "$average_time"
}

# Measure with feature disabled
disabled_time=$(measure_startup "0")
echo "Average startup time (disabled): ${disabled_time}ms"

# Measure with feature enabled
enabled_time=$(measure_startup "1")
echo "Average startup time (enabled): ${enabled_time}ms"

# Calculate overhead
overhead_percent=$(((enabled_time - disabled_time) * 100 / disabled_time))
echo "Performance overhead: ${overhead_percent}%"

# Validate acceptable overhead (< 5%)
if [ $overhead_percent -le 5 ]; then
    echo "✅ Performance overhead acceptable"
else
    echo "❌ Performance overhead too high"
    exit 1
fi
```

### Memory Usage Monitoring

```bash
#!/bin/bash
# scripts/memory-benchmarks.sh

# Monitor memory usage during subagent execution
echo "=== Memory Usage Benchmarks ==="

export CODEX_SUBAGENTS_ENABLED=1

# Create test agent that does significant work
cat > /tmp/test-agent.md << 'EOF'
---
name: memory-test
tools: [git]
---
Please analyze the git history and provide a detailed summary of the last 50 commits.
EOF

mkdir -p ~/.codex/agents
cp /tmp/test-agent.md ~/.codex/agents/

# Monitor memory usage
(
    while true; do
        ps -o pid,vsz,rss,comm -p $(pgrep -f codex) 2>/dev/null || true
        sleep 1
    done
) &
monitor_pid=$!

# Execute memory-intensive subagent
./target/release/codex subagents run memory-test --no-wait

# Let it run for 30 seconds
sleep 30

# Stop monitoring
kill $monitor_pid 2>/dev/null || true

echo "✅ Memory usage monitoring complete"
```

## Validation Checklist

### Pre-Release Validation

- [ ] All platform binaries build successfully
- [ ] Feature flags work correctly on all platforms
- [ ] Performance overhead within acceptable limits (< 5%)
- [ ] Memory usage remains bounded during execution
- [ ] TypeScript bindings generate correctly
- [ ] Example agents included in distribution
- [ ] Cross-platform file path handling works
- [ ] Platform-specific dependencies resolved

### Post-Release Validation

- [ ] Download and installation work on all platforms
- [ ] Default configuration behavior correct
- [ ] Example agents execute successfully
- [ ] Error messages are clear and helpful
- [ ] Documentation matches actual behavior
- [ ] Telemetry collection functions properly

## Troubleshooting Common Issues

### Build Issues

**Problem**: Cross-compilation fails for Windows targets
**Solution**: Ensure Windows-specific dependencies and linkers are available

**Problem**: macOS binary not signed/notarized
**Solution**: Configure code signing in the release pipeline

### Runtime Issues

**Problem**: Subagents not found on Linux
**Solution**: Verify file permissions and PATH configuration

**Problem**: Feature toggle not working on Windows
**Solution**: Check environment variable case sensitivity and PowerShell escaping

### Performance Issues

**Problem**: High startup overhead
**Solution**: Profile subagent registry loading and optimize caching

**Problem**: Memory leaks during long sessions
**Solution**: Review TaskContext cleanup and conversation disposal

---

*This validation ensures subagents work reliably across all supported platforms with proper feature toggle behavior and acceptable performance characteristics.*