# Subagent Specification Format

Date: September 27, 2025

Each subagent is described using a Markdown file with YAML front matter. The front matter captures metadata, while the Markdown body provides the instruction prompt delivered to the agent at runtime.

```markdown
---
name: code-reviewer
description: Reviews staged Git changes for issues
model: gpt-5-codex
tools:
  - git
  - cargo
keywords:
  - review
  - rust
---
Walk through each staged diff and report:

- logic bugs or missing edge cases
- formatting violations not covered by automated tools
- security or privacy concerns
```
```

## Fields

- `name` *(required)*: Stable identifier used for lookups and slash commands.
- `description` *(optional)*: Short human-readable summary displayed in lists.
- `model` *(optional)*: Preferred model override. Invalid values fall back to the session default.
- `tools` *(optional)*: Allowlisted tool identifiers the subagent may invoke.
- `keywords` *(optional)*: Hints used by the router for auto-routing heuristics.

The Markdown body is preserved verbatim (after trimming leading blank lines) and handed to the orchestrator as the agent’s instructions. Trailing whitespace is trimmed.

## Lookup Order

The registry merges:

1. Project agents: `<repo>/.codex/agents/*.md`
2. User agents: `~/.codex/agents/*.md`

Project entries override user entries with the same `name`. Files are cached by modification time and re-parsed on reload.

## Validation Rules

- Documents must start with `---` and include a closing `---` delimiter for the front matter block.
- `name` cannot be empty and must be unique after precedence rules are applied.
- Duplicate `tools` or `keywords` are deduplicated while preserving order of first appearance.
- Parse errors are reported by the registry with the source path and surfaced in `/agents` UI summaries.

## Debugging Tips

Set `CODEX_DEBUG_SUBAGENTS=1` before running the CLI to emit JSON snapshots of the shared `TaskContext`, including scratchpads and diagnostic history.
