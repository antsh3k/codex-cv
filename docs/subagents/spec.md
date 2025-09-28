# Subagent Specification Format

Subagents are defined in Markdown files that begin with a YAML frontmatter block followed by free-form instructions. The parser in `codex-subagents` expects the following schema:

```yaml
---
name: reviewer               # required; `[a-z][a-z0-9_-]{2,63}`
description: Reviews diffs   # optional; short human summary
model: gpt-5-codex           # optional; falls back to session model if omitted
tools: [apply_patch]         # optional; allowlist of tool identifiers
keywords: [review, lint]     # optional; used for keyword auto-routing
---
```

The Markdown body that follows is stored verbatim as the agent instructions. Example file:

```markdown
---
name: reviewer
description: Reviews diffs for style/security issues
model: gpt-5-codex
tools: [apply_patch, git_diff]
keywords: [review, diff, lint]
---

You are a meticulous reviewer. Inspect every change, reference security best practices, and avoid approving untested code. Summarize high-risk findings with remediation steps.
```

## File discovery

The registry looks in two locations:

1. Project agents: `<repo>/.codex/agents/*.md`
2. User agents: `~/.codex/agents/*.md`

When both locations define the same `name`, the project version overrides the user definition. Files are cached by modification time so `reload()` is cheap.

## Validation rules

- The `name` must start with a lowercase letter and only contain lowercase letters, digits, `_`, or `-`.
- Empty strings are rejected for `tools` and `keywords`.
- Duplicate entries in `tools` or `keywords` are rejected.
- Instructions must not be empty after trimming.
- Parse errors are recorded and surfaced by CLI/TUI listings.

## Optional metadata

- `model`: overrides the session model when present; invalid models are handled at orchestration time.
- `tools`: enforce a strict allowlist before tool execution.
- `keywords`: feed simple keyword-based routing when `subagents.auto_route = true`.
- Additional metadata can be added in the future without breaking backward compatibility; unknown keys are currently ignored.
