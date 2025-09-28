# Subagent Specification Format

Subagents are defined in Markdown files that begin with a YAML frontmatter block followed by free-form instructions. The parser in `codex-subagents` expects the following schema:

```yaml
---
name: reviewer               # required; `[a-z][a-z0-9_-]{2,63}`
description: Reviews diffs   # optional; short human summary
model: gpt-5-codex           # optional; shorthand for setting a model
model_config:                # optional; structured model binding
  provider: openai           # matches a provider id in config.toml
  model: gpt-4o              # overrides the runtime model
  endpoint: https://proxy.example.dev/v1
  parameters:                # optional free-form JSON values
    temperature: 0.1
tools: [apply_patch]         # optional; allowlist of tool identifiers
keywords: [review, lint]     # optional; used for keyword auto-routing
---
```

The Markdown body that follows is stored verbatim as the agent instructions. Example file:

```markdown
---
name: reviewer
description: Reviews diffs for style/security issues
model_config:
  provider: openai
  model: gpt-4o
  parameters:
    temperature: 0.1
tools: [apply_patch, git_diff]
keywords: [review, diff, lint]
---

You are a meticulous reviewer. Inspect every change, reference security best practices, and avoid approving untested code. Summarize high-risk findings with remediation steps.
```

## `model` vs `model_config`

- The legacy `model` string remains supported. When present, it is treated as shorthand for `model_config.model` with no provider override.
- `model_config` unlocks per-agent model bindings:
  - `provider`: references a provider id available in the merged `model_providers` map (built-ins plus overrides from `~/.codex/config.toml`).
  - `model`: optional; when omitted the session default is used.
  - `endpoint`: optional; overrides the provider's `base_url` for this agent only.
  - `parameters`: optional map of provider-specific settings stored on the spec for future use.
- If both `model` and `model_config.model` are provided they must match.
- Leave the entire block out to inherit the session's model/provider unchanged.

## File discovery

The registry looks in two locations:

1. Project agents: `<repo>/.codex/agents/*.md`
2. User agents: `~/.codex/agents/*.md`

When both locations define the same `name`, the project version overrides the user definition. Files are cached by modification time so `reload()` is cheap.

## Validation rules

- The `name` must start with a lowercase letter and only contain lowercase letters, digits, `_`, or `-`.
- Empty strings are rejected for `tools`, `keywords`, and `model_config` keys that expect strings.
- Duplicate entries in `tools` or `keywords` are rejected.
- Conflicting model declarations (`model` vs `model_config.model`) are rejected.
- Instructions must not be empty after trimming.
- Parse errors are recorded and surfaced by CLI/TUI listings.

## Optional metadata

- `model`/`model_config`: override the session model/provider for this agent.
- `tools`: enforce a strict allowlist before tool execution.
- `keywords`: feed simple keyword-based routing when `subagents.auto_route = true`.
- Additional metadata can be added in the future without breaking backward compatibility; unknown keys are currently ignored.
