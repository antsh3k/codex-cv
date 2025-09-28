# Comprehensive Manual Demo Script: Code-Reviewer Scenario

## Overview

This manual demo script validates the complete Phase 5 subagents functionality through a realistic code-reviewer scenario. It demonstrates the full pipeline from agent definition creation to execution and validation.

**Target Workflow**: Create → Launch → Observe → Validate

## Prerequisites

### Environment Setup
```bash
# Ensure subagents feature is enabled
export CODEX_SUBAGENTS_ENABLED=true

# Set working directory
cd /path/to/codex-subagents/codex-cv

# Ensure all dependencies are available
just check-deps
cargo check --workspace
```

### Agent Definition Creation
Create `.codex/agents/code-reviewer.md` with the following content:

```markdown
---
name: code-reviewer
description: Reviews code for potential issues and style violations
model: gpt-4
tools:
  - read
  - analysis
  - git
keywords:
  - review
  - quality
  - security
---

# Code Review Instructions

Analyze the provided code changes and identify:

1. **Logic Issues**: Potential bugs, missing edge cases, incorrect assumptions
2. **Security Concerns**: Input validation, authentication bypasses, data exposure
3. **Style Violations**: Code formatting, naming conventions, documentation gaps
4. **Performance Issues**: Inefficient algorithms, memory leaks, resource usage

## Review Process

1. Read the staged changes using git diff
2. Analyze each file for the issues listed above
3. Provide specific line-by-line feedback
4. Suggest concrete improvements
5. Rate the overall code quality (1-10 scale)

## Output Format

Provide findings in this structure:
- **File**: `path/to/file.rs`
- **Line**: 42
- **Issue**: Brief description
- **Severity**: Critical/High/Medium/Low
- **Suggestion**: Specific improvement recommendation
```

## Demo Execution Steps

### Step 1: Verify Agent Discovery

**Command**:
```bash
codex subagents list
```

**Expected Output**:
```
Available subagents:

- code-reviewer: Reviews code for potential issues and style violations (model: gpt-4)
```

**Validation Points**:
- [ ] Agent appears in list
- [ ] Description is correctly displayed
- [ ] Model override (gpt-4) is shown
- [ ] No parse errors in output

### Step 2: Create Sample Code Changes

Create test changes to review:

```bash
# Create a simple Rust file with intentional issues
cat > src/sample_review.rs << 'EOF'
use std::collections::HashMap;

// TODO: This function has several issues for demonstration
pub fn process_user_data(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut data = HashMap::new();

    // Issue 1: No input validation
    let parsed = input.parse::<i32>()?;

    // Issue 2: Potential overflow
    let result = parsed * 1000000;

    // Issue 3: Inefficient string concatenation
    let mut output = String::new();
    for i in 0..result {
        output = output + &i.to_string() + ",";
    }

    // Issue 4: No error handling for HashMap operations
    data.insert("result".to_string(), output.clone());
    data.get("result").unwrap().clone();

    Ok(output)
}

// Issue 5: Missing documentation
pub fn unsafe_operation(ptr: *mut i32) {
    unsafe {
        *ptr = 42; // Issue 6: No null pointer check
    }
}
EOF

# Stage the file for review
git add src/sample_review.rs
```

### Step 3: Execute Code Review via TUI

**Launch TUI**:
```bash
codex tui
```

**TUI Interaction Steps**:
1. Type: `/use code-reviewer "Please review the staged changes"`
2. Observe the subagent execution indicators:
   - ▶ Started subagent 'code-reviewer' (gpt-4)
   - Conversation: conv-abc123
3. Wait for review completion
4. Check for completion indicator:
   - ◀ Completed subagent 'code-reviewer' (success)

**Expected Review Output**:
The code-reviewer should identify and report:
```
Code Review Results:

File: src/sample_review.rs

Line 7: Input validation missing
Severity: High
Suggestion: Add bounds checking and validate input format

Line 10: Integer overflow risk
Severity: Critical
Suggestion: Use checked arithmetic or appropriate data types

Line 13-16: Inefficient string concatenation
Severity: Medium
Suggestion: Use Vec<String> with join() or format! macro

Line 19: Unwrap without error handling
Severity: High
Suggestion: Use proper error handling with ? operator

Line 24: Missing documentation
Severity: Low
Suggestion: Add doc comments explaining function purpose

Line 27: Unsafe operation without null check
Severity: Critical
Suggestion: Add null pointer validation before dereferencing

Overall Quality Rating: 3/10
```

### Step 4: Execute Code Review via CLI

**Command**:
```bash
codex subagents run code-reviewer --prompt "Analyze the staged Git changes for security and performance issues"
```

**Expected Behavior**:
1. Command should spawn a subagent conversation
2. Output should include JSON-formatted events:
   - SessionConfigured event
   - SubAgentStarted event
   - SubAgentMessage events (user prompt)
   - SubAgentCompleted event
3. Review analysis should be displayed in conversation output

**Validation Points**:
- [ ] Command executes without errors
- [ ] SubAgent events are properly emitted
- [ ] Review content is comprehensive and relevant
- [ ] Exit code is 0 (success)

### Step 5: Error Handling Validation

**Test Invalid Agent**:
```bash
codex subagents run nonexistent-agent
```

**Expected Output**:
```
Error: unknown subagent 'nonexistent-agent'
```

**Test Disabled Feature**:
```bash
CODEX_SUBAGENTS_ENABLED=false codex subagents run code-reviewer
```

**Expected Output**:
```
Error: subagents are disabled in the current configuration
```

### Step 6: Telemetry and Event Validation

**Monitor Events** (if telemetry is enabled):
```bash
# Check for SubAgent* events in logs
tail -f ~/.codex/logs/events.log | grep -E "(SubAgentStarted|SubAgentMessage|SubAgentCompleted)"
```

**Expected Event Sequence**:
```json
{"type": "SubAgentStarted", "agent_name": "code-reviewer", "model": "gpt-4", "sub_conversation_id": "conv-123"}
{"type": "SubAgentMessage", "agent_name": "code-reviewer", "role": "user", "content": "Please review..."}
{"type": "SubAgentCompleted", "agent_name": "code-reviewer", "outcome": "success"}
```

### Step 7: Performance and Reliability Testing

**Concurrent Execution Test**:
```bash
# Launch multiple subagent sessions in parallel
for i in {1..3}; do
  codex subagents run code-reviewer --prompt "Review iteration $i" &
done
wait
```

**Validation Points**:
- [ ] All sessions complete successfully
- [ ] No resource conflicts or deadlocks
- [ ] Each session receives unique conversation IDs
- [ ] Performance remains acceptable under load

### Step 8: Clean Up Test Environment

```bash
# Remove test files
git reset HEAD src/sample_review.rs
rm -f src/sample_review.rs

# Verify agent still functions after cleanup
codex subagents list
```

## Success Criteria Checklist

### Core Functionality
- [ ] Agent discovery works correctly (`codex subagents list`)
- [ ] TUI integration executes subagents with visual feedback
- [ ] CLI execution produces expected events and output
- [ ] Error handling works for invalid agents and disabled features

### Quality Assurance
- [ ] Code review identifies all planted issues
- [ ] Review suggestions are actionable and specific
- [ ] Output format matches expected structure
- [ ] Performance is acceptable (< 30 seconds for basic review)

### Technical Validation
- [ ] SubAgent* events are properly emitted
- [ ] Conversation isolation works correctly
- [ ] Model override (gpt-4) is respected
- [ ] Tool allowlist restrictions are enforced

### User Experience
- [ ] Clear progress indicators in TUI
- [ ] Intuitive command-line interface
- [ ] Helpful error messages
- [ ] Consistent behavior across interfaces

## Troubleshooting Guide

### Common Issues

**Agent Not Found**:
- Verify `.codex/agents/code-reviewer.md` exists
- Check YAML frontmatter syntax
- Ensure agent name matches file name

**Permission Denied**:
- Check file permissions on agent definition
- Verify `CODEX_SUBAGENTS_ENABLED=true`
- Confirm authentication is configured

**Timeout Errors**:
- Increase timeout in configuration
- Check network connectivity for model API calls
- Verify model availability (gpt-4)

**Parse Errors**:
- Validate YAML frontmatter syntax
- Check for required fields (name, tools, instructions)
- Review markdown formatting

### Debugging Commands

```bash
# Enable debug logging
export CODEX_DEBUG_SUBAGENTS=1

# Check configuration
codex config show

# Validate agent parsing
codex subagents list --verbose

# Test with minimal agent
cat > .codex/agents/minimal.md << 'EOF'
---
name: minimal
---
Test agent for debugging.
EOF
```

## Recording and Documentation

### Screenshots to Capture
1. Agent list output (`codex subagents list`)
2. TUI showing subagent execution progress
3. Completed review results in conversation
4. CLI command execution with events
5. Error handling examples

### Performance Metrics to Record
- Agent discovery time
- Review execution duration
- Memory usage during execution
- Concurrent session handling

### Video Recording Checklist
- [ ] Complete workflow from start to finish
- [ ] Error scenarios and recovery
- [ ] TUI visual indicators and feedback
- [ ] CLI command execution
- [ ] Agent definition creation process

## Conclusion

This demo script validates the complete subagents functionality from agent definition through execution and monitoring. Successful completion of all steps confirms the production readiness of the Phase 5 implementation.

**Final Validation**: The target demo workflow (Create → Launch → Observe → Validate) should complete without breaking existing workflows, demonstrating the successful integration of the subagents feature while maintaining backward compatibility.