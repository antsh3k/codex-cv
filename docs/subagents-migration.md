# Subagents Migration Guide

This guide helps you migrate to Codex Subagents and understand compatibility considerations.

## Overview

Codex Subagents is designed to be **completely backward compatible**. When the feature is disabled (default), Codex behaves exactly as before with no performance impact.

## Enabling Subagents

### For New Users

Add to your `~/.codex/config.toml`:

```toml
[subagents]
enabled = true
auto_route = false  # Optional: enable automatic agent selection
```

Or use environment variable:

```bash
export CODEX_SUBAGENTS_ENABLED=1
```

### For Beta Users

If you used experimental subagents in earlier versions:

#### Configuration Migration

**Old configuration:**
```toml
[experimental]
subagents = true
```

**New configuration:**
```toml
[subagents]
enabled = true
auto_route = false
```

#### Command Changes

- `/subagent <name>` → `/use <name>`
- `/subagent-list` → `/agents`
- `/subagent-status` → `/subagent-status` (unchanged)

#### Agent Definition Changes

Most agent files require no changes, but review these updates:

**Tool Access (Breaking Change in v0.2.0+)**

Previously, agents inherited all available tools. Now, tools must be explicitly allowlisted:

```yaml
# Old (inherited all tools automatically)
---
name: my-agent
---

# New (explicit tool allowlist required)
---
name: my-agent
tools:
  - git
  - cargo
  - npm
---
```

**Model Fallback Behavior**

Model selection is now standardized:
1. Agent-specified model (in YAML frontmatter)
2. Session default model
3. Global default model

**Event Payload Updates**

If you use MCP or consume Codex events programmatically, note these changes:

- `SubAgentStarted` events now include `sub_conversation_id`
- `PatchApplyBegin/End` events include optional `originating_subagent` field
- New MCP endpoints: `subagents-list` and `subagents-run`

## Compatibility Matrix

| Codex Version | Subagents Support | Breaking Changes |
|---------------|-------------------|------------------|
| 0.1.x | ❌ Not available | N/A |
| 0.2.0 | ✅ Beta | Tool allowlists required |
| 0.3.0+ | ✅ Stable | Command renames |

## Migration Scenarios

### Scenario 1: First-Time Enablement

**Before:**
```bash
codex tui
# Standard Codex experience
```

**After:**
```bash
# Enable in config
echo -e "\n[subagents]\nenabled = true" >> ~/.codex/config.toml

codex tui
# Now has access to /agents, /use commands
/agents  # See available agents
/use code-reviewer  # Delegate code review
```

**Impact:** None on existing workflows. New capabilities added.

### Scenario 2: Beta User Upgrade

**Before (Beta):**
```toml
[experimental]
subagents = true
```

**Migration Steps:**

1. **Update configuration:**
   ```bash
   # Remove old config
   sed -i '' '/\[experimental\]/,/subagents = true/d' ~/.codex/config.toml

   # Add new config
   echo -e "\n[subagents]\nenabled = true" >> ~/.codex/config.toml
   ```

2. **Update agent definitions** (if any custom agents exist):
   ```bash
   # Add tools field to each agent in ~/.codex/agents/
   # Example update:
   cat > ~/.codex/agents/my-agent.md << 'EOF'
   ---
   name: my-agent
   description: My custom agent
   tools:
     - git
     - cargo
   ---
   Original agent instructions here.
   EOF
   ```

3. **Update command usage:**
   - Replace `/subagent` with `/use`
   - Replace `/subagent-list` with `/agents`

**Validation:**
```bash
codex tui
/agents  # Should show your agents
/use my-agent  # Should work with tool allowlist
```

### Scenario 3: Team/Organization Rollout

**Planning Considerations:**

1. **Staged Rollout:** Start with a subset of developers
2. **Training:** Provide team training on agent creation and usage
3. **Governance:** Establish guidelines for agent creation and sharing
4. **Monitoring:** Track adoption and performance metrics

**Rollout Steps:**

1. **Pilot Phase:**
   ```bash
   # Enable for pilot users only
   export CODEX_SUBAGENTS_ENABLED=1
   ```

2. **Team Configuration:**
   ```toml
   # Shared team config template
   [subagents]
   enabled = true
   auto_route = false
   ```

3. **Shared Agents:**
   ```bash
   # Create team agents repository
   mkdir -p team-codex-agents
   # Share agents via git or shared filesystem
   ln -s /shared/team-agents ~/.codex/agents
   ```

## Troubleshooting Migration Issues

### Common Problems

#### Feature Not Available
```
Error: Unknown command '/use'
```

**Solution:** Ensure `subagents.enabled = true` in config or set `CODEX_SUBAGENTS_ENABLED=1`.

#### Tools Access Denied
```
Error: Tool 'docker' not allowed for agent 'my-agent'
```

**Solution:** Add missing tools to agent's YAML frontmatter:
```yaml
tools:
  - git
  - docker
  - npm
```

#### Legacy Command Not Found
```
Error: Unknown command '/subagent'
```

**Solution:** Use new command syntax: `/use <agent-name>`.

#### Configuration Conflicts
```
Error: Invalid configuration format
```

**Solution:** Remove old `[experimental]` section and use new `[subagents]` section format.

### Rollback Procedure

If you need to disable subagents:

1. **Disable in configuration:**
   ```toml
   [subagents]
   enabled = false
   ```

2. **Or remove configuration entirely:**
   ```bash
   sed -i '' '/\[subagents\]/,/enabled = true/d' ~/.codex/config.toml
   ```

3. **Or use environment override:**
   ```bash
   export CODEX_SUBAGENTS_ENABLED=0
   ```

**Result:** Codex returns to pre-subagents behavior with no functional changes.

## Performance Considerations

### Resource Usage

**When Enabled:**
- Minimal overhead when not actively using subagents
- Additional memory usage during subagent execution
- Extra CPU for agent parsing and validation

**When Disabled:**
- Zero performance impact
- No additional memory usage
- No startup time increase

### Optimization Tips

1. **Selective Enablement:** Only enable for users who need subagents
2. **Agent Cleanup:** Remove unused custom agents periodically
3. **Tool Minimalism:** Use minimal tool allowlists for better security and performance
4. **Model Selection:** Choose appropriate models for agent complexity

## Security Considerations

### Tool Access Changes

**Impact:** Subagents can only use explicitly allowlisted tools, improving security isolation.

**Migration:** Review all custom agents and add necessary tools to their `tools` field.

### Audit Trail

**New Capability:** All subagent actions are logged with attribution for audit purposes.

**Benefit:** Better visibility into which agent made which changes.

## Support and Resources

### Getting Help

- **Documentation:** [docs/subagents.md](subagents.md)
- **GitHub Issues:** Report migration problems
- **Community Discord:** Ask questions and share experiences

### Best Practices

1. **Start Small:** Enable for individual developers first
2. **Test Thoroughly:** Validate agents work as expected after migration
3. **Document Changes:** Keep track of configuration and agent updates
4. **Train Users:** Provide training on new commands and capabilities
5. **Monitor Usage:** Track adoption and performance metrics

### Example Migration Timeline

**Week 1:** Pilot enablement with 2-3 developers
**Week 2:** Expand to team leads and create first shared agents
**Week 3:** Team-wide rollout with training session
**Week 4:** Full organization deployment with monitoring

---

*Need help with migration? Check the [troubleshooting section](#troubleshooting-migration-issues) or reach out to the community for support.*