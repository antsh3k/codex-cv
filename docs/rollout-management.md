# Rollout Management for Codex Subagents

This document provides operational procedures for managing the subagents feature rollout across all deployment phases.

## Feature Flag Management

### Default Configuration Status

**Current Configuration** ✅
```rust
// codex-rs/core/src/config.rs
impl Default for SubagentsConfig {
    fn default() -> Self {
        Self {
            enabled: false,    // ✅ Correct: Default disabled
            auto_route: false, // ✅ Correct: Conservative default
        }
    }
}
```

### Internal Enablement Strategy

#### Phase 1: OpenAI Internal Teams

**Target**: Engineering teams working on Codex CLI development

**Enablement Method**:
```bash
# Individual developer enablement
export CODEX_SUBAGENTS_ENABLED=1
echo 'export CODEX_SUBAGENTS_ENABLED=1' >> ~/.bashrc

# Or via configuration file
cat >> ~/.codex/config.toml << 'EOF'
[subagents]
enabled = true
auto_route = false
EOF
```

**Prerequisites**:
- [ ] All regression tests passing for 2+ weeks
- [ ] Performance metrics within acceptable ranges
- [ ] Documentation complete and validated
- [ ] Internal training materials prepared

**Validation Checklist**:
```bash
# 1. Verify feature flag works
codex subagents list  # Should show example agents

# 2. Test core workflows
/use code-reviewer "Review my staged changes"

# 3. Validate performance
time codex --version  # Should be < 5% overhead

# 4. Check error handling
export CODEX_SUBAGENTS_ENABLED=0
codex subagents list  # Should show "feature not enabled"
```

#### Phase 2: Post-Regression Validation

**Automated Monitoring**:
```yaml
# .github/workflows/regression-monitoring.yml
name: Post-Release Regression Monitoring
on:
  schedule:
    - cron: '0 */4 * * *'  # Every 4 hours

jobs:
  regression-check:
    runs-on: ubuntu-latest
    steps:
    - name: Performance baseline check
      run: |
        # Measure startup time with feature disabled
        export CODEX_SUBAGENTS_ENABLED=0
        baseline=$(time codex --version 2>&1 | grep real)

        # Measure with feature enabled
        export CODEX_SUBAGENTS_ENABLED=1
        enabled_time=$(time codex --version 2>&1 | grep real)

        # Alert if > 5% regression
        ./scripts/check-performance-regression.py "$baseline" "$enabled_time"

    - name: Functional validation
      run: |
        export CODEX_SUBAGENTS_ENABLED=1
        # Test basic agent functionality
        codex subagents list | grep -q "code-reviewer"
        echo "Test prompt" | codex subagents run code-reviewer --no-wait

    - name: Alert on failure
      if: failure()
      run: |
        curl -X POST "$SLACK_WEBHOOK" -d '{"text":"🚨 Subagents regression detected"}'
```

**Internal Rollout Criteria**:
- ✅ Zero critical bugs for 1 week
- ✅ Performance within 5% of baseline
- ✅ All example agents functional
- ✅ Positive internal team feedback

## Pilot Feedback Collection System

### Feedback Categories

| Category | Description | Priority Mapping |
|----------|-------------|------------------|
| **Crashes** | Application crashes or hangs | Critical |
| **Data Loss** | Lost work or corrupted files | Critical |
| **Security** | Security vulnerabilities | Critical |
| **Performance** | Slow response or high resource usage | High |
| **Usability** | Confusing UX or workflows | Medium |
| **Enhancement** | Feature requests or improvements | Low |

### Feedback Collection Methods

#### 1. Structured Feedback Form

```markdown
## Subagents Pilot Feedback

**User Info:**
- Name: [Your name]
- Team: [Engineering/Product/Other]
- Codex Version: [Run `codex --version`]
- Platform: [macOS/Linux/Windows]

**Feedback Type:**
- [ ] Bug Report
- [ ] Performance Issue
- [ ] Usability Problem
- [ ] Feature Request
- [ ] Documentation Issue

**Details:**
- **Summary**: [Brief description]
- **Steps to Reproduce**: [What you did]
- **Expected Behavior**: [What should happen]
- **Actual Behavior**: [What actually happened]
- **Agent Name**: [Which agent if applicable]
- **Impact**: [How this affects your workflow]

**Screenshots/Logs:**
[Attach any relevant files]
```

#### 2. Automated Issue Tracking

**Integration with GitHub Issues**:
```bash
# scripts/create-feedback-issue.sh
#!/bin/bash

gh issue create \
  --title "[Subagents Pilot] $1" \
  --body-file feedback-template.md \
  --label "subagents-pilot,needs-triage" \
  --assignee subagents-team

echo "Created feedback issue: $1"
```

**Usage**:
```bash
./scripts/create-feedback-issue.sh "Agent execution hangs on large repositories"
```

#### 3. Telemetry-Based Detection

**Automatic Error Reporting**:
```rust
// In subagent orchestrator
if let Err(error) = subagent_result {
    // Log structured error for analysis
    tracing::error!(
        error = %error,
        agent_name = %agent_name,
        user_id = %user_id,
        codex_version = %version,
        platform = %platform,
        "Subagent execution failed"
    );

    // Increment failure metrics
    metrics::counter!("subagent.execution.failed").increment(1);
}
```

### Feedback Analysis Dashboard

**Weekly Metrics Collection**:
```python
#!/usr/bin/env python3
# scripts/generate-pilot-metrics.py

import json
from collections import defaultdict

def analyze_pilot_feedback():
    # Collect from multiple sources
    github_issues = get_github_issues(label="subagents-pilot")
    telemetry_errors = get_telemetry_errors(last_week=True)
    manual_feedback = get_manual_feedback()

    metrics = {
        "total_users": count_active_pilot_users(),
        "adoption_rate": calculate_adoption_rate(),
        "issues_by_category": categorize_issues(github_issues),
        "error_rates": calculate_error_rates(telemetry_errors),
        "satisfaction_score": calculate_satisfaction(manual_feedback),
        "top_pain_points": identify_pain_points(github_issues, manual_feedback)
    }

    return metrics

def generate_report(metrics):
    """Generate weekly pilot status report."""
    return f"""
# Subagents Pilot Status - Week {get_week_number()}

## Key Metrics
- **Active Pilot Users**: {metrics['total_users']}
- **Weekly Adoption Rate**: {metrics['adoption_rate']}%
- **User Satisfaction**: {metrics['satisfaction_score']}/5.0

## Issue Summary
- Critical: {metrics['issues_by_category']['critical']}
- High: {metrics['issues_by_category']['high']}
- Medium: {metrics['issues_by_category']['medium']}
- Low: {metrics['issues_by_category']['low']}

## Top Pain Points
{format_pain_points(metrics['top_pain_points'])}

## Action Items
{generate_action_items(metrics)}
"""

if __name__ == "__main__":
    metrics = analyze_pilot_feedback()
    report = generate_report(metrics)
    print(report)

    # Auto-send to Slack
    send_to_slack(report)
```

### Iteration Process

#### Weekly Review Cycle

**Monday: Data Collection**
```bash
# Automated weekly data collection
python3 scripts/generate-pilot-metrics.py > reports/week-$(date +%V).md

# Manual feedback review
gh issue list --label "subagents-pilot" --state open
```

**Wednesday: Team Review**
- Review metrics and feedback
- Prioritize critical issues
- Plan fixes and improvements
- Update documentation if needed

**Friday: Release Preparation**
- Deploy fixes to pilot users
- Update feedback tracking
- Communicate changes to pilot group

#### Heuristics and UI Improvements

**Common Improvement Areas**:

1. **Agent Discovery**: Make it easier to find the right agent
   ```bash
   # Before: Users confused about which agent to use
   /agents  # Shows basic list

   # After: Enhanced with descriptions and use cases
   /agents --verbose  # Shows detailed info with examples
   ```

2. **Error Messages**: Clearer feedback when things go wrong
   ```rust
   // Before: Generic error
   "Agent execution failed"

   // After: Actionable error with suggestions
   "Agent 'code-reviewer' failed: Git repository not found.
    Try running this command from within a Git repository,
    or use 'git init' to initialize one."
   ```

3. **Progress Visibility**: Show what agents are doing
   ```bash
   # Enhanced status display
   /status
   # Shows: "🔄 code-reviewer: Analyzing 15 files (2m remaining)"
   ```

## Default Enablement Transition

### Stability Confirmation Criteria

**Quantitative Metrics**:
- ✅ Error rate < 1% across all agent executions
- ✅ Performance overhead < 3% system-wide
- ✅ User satisfaction score > 4.0/5.0
- ✅ Zero critical bugs for 4+ weeks
- ✅ 70%+ pilot user retention

**Qualitative Indicators**:
- ✅ Positive feedback from diverse user types
- ✅ Success stories from real workflows
- ✅ Documentation validated with new users
- ✅ Support team confidence in troubleshooting

### Transition Plan

#### Phase 1: Opt-in Default (2 weeks)
```toml
# New user configuration gets this by default
[subagents]
enabled = true
auto_route = false  # Still conservative
```

**Implementation**:
```rust
// Update default configuration
impl Default for SubagentsConfig {
    fn default() -> Self {
        Self {
            enabled: true,     // ✅ Changed: Now enabled by default
            auto_route: false, // ✅ Unchanged: Still conservative
        }
    }
}
```

**Monitoring During Transition**:
```bash
# Track new user adoption
codex-analytics --metric "new_user_subagent_usage" --period weekly

# Monitor support ticket volume
support-dashboard --filter "subagents" --alert-threshold 20%
```

#### Phase 2: Full Default Enablement (2 weeks)
- All new installations get subagents enabled
- Existing users can still disable if desired
- Enhanced onboarding includes subagents introduction

#### Phase 3: Announcement and Documentation

**Release Notes Update**:
```markdown
## 🎉 Subagents Now Enabled by Default

Codex Subagents is now enabled by default for all users!

**What's New:**
- Delegate specialized tasks to focused AI agents
- Built-in agents for code review, documentation, testing, and debugging
- Create custom agents for your specific workflows

**Getting Started:**
- Try `/agents` to see available agents
- Use `/use code-reviewer` to review your staged changes
- Visit [docs/subagents.md](docs/subagents.md) for complete documentation

**Need to Disable?**
Add to your `~/.codex/config.toml`:
```toml
[subagents]
enabled = false
```
```

**Onboarding Content Updates**:
```bash
# Update getting-started documentation
docs/getting-started.md:
- Add subagents section after basic usage
- Include example workflow with /use command
- Link to full subagents documentation

# Update CLI help text
codex --help:
- Mention subagents in feature list
- Add link to /agents command documentation

# Update TUI welcome screen
- Add hint about /agents command
- Show example agent execution
```

## Success Metrics and KPIs

### Adoption Metrics
- **Feature Usage Rate**: % of active users who try subagents monthly
- **Agent Execution Volume**: Number of agent runs per user per week
- **Custom Agent Creation**: % of users who create custom agents
- **Retention Rate**: % of users who continue using after first week

### Quality Metrics
- **Success Rate**: % of agent executions that complete successfully
- **User Satisfaction**: Average rating from feedback surveys
- **Time to Value**: How quickly new users achieve useful results
- **Support Burden**: Volume of subagents-related support tickets

### Performance Metrics
- **Execution Speed**: Average time from agent invocation to completion
- **System Impact**: CPU/memory overhead when feature is enabled
- **Error Recovery**: Time to resolve critical issues
- **Scalability**: Performance under high concurrent usage

### Target Thresholds

| Metric | Alpha Target | Beta Target | GA Target |
|--------|-------------|-------------|-----------|
| Adoption Rate | 60% | 70% | 40% |
| Success Rate | 95% | 98% | 99% |
| Satisfaction | 3.5/5 | 4.0/5 | 4.2/5 |
| Performance Overhead | < 10% | < 5% | < 3% |
| Error Resolution | < 24h | < 12h | < 4h |

---

*This rollout management strategy ensures safe, measured deployment with comprehensive feedback collection and continuous improvement throughout the enablement process.*