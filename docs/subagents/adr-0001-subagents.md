# ADR-0001: Subagent Framework Strategy

## Status

Accepted — September 27, 2025

## Context

The codex-cv initiative needs a demo-ready but production-safe subagent experience that can scale from a scripted walkthrough to broader distribution. We must preserve existing single-agent behavior by default, respect sandbox constraints (`CODEX_SANDBOX_DISABLED`, Seatbelt), and design a path toward CLI/TUI/MCP parity. Prior art from Claude subagents highlights the importance of deterministic orchestration, explicit tool allowlists, and clear telemetry for sub-runs.

## Decision

We will deliver a sequential, feature-flagged architecture with these pillars:

- **Config + flag controls**: Introduce `subagents.enabled` and `subagents.auto_route` configuration keys (default `false`) in `codex-core`, plus an env override `CODEX_SUBAGENTS_ENABLED=true`. When disabled, the system reverts to legacy assistant orchestration without touching the new code paths.
- **Spec format**: Define agents via Markdown files with YAML frontmatter (`name`, optional `description`, `model`, `tools`, `keywords`). Project-level specs in `<repo>/.codex/agents/` override user-level specs in `~/.codex/agents/`.
- **Registry + parser**: Centralize parsing and precedence rules inside a new `codex-subagents` crate to avoid scattering concerns across surfaces.
- **Orchestration contract**: Add a seatbelt-aware orchestrator that spawns isolated `CodexConversation` instances. Every sub-run emits `SubAgentStarted`, `SubAgentMessage`, and `SubAgentCompleted` events with `sub_conversation_id` and `origin_agent` metadata for auditing.
- **UX affordances**: Surface list/run flows via CLI (`codex subagents list`, `codex subagents run`) and TUI slash commands (`/agents`, `/use`, `/subagent-status`). Nested transcripts are rendered with lightweight styling helpers.
- **Telemetry hooks**: Reuse existing token usage events, adding per-agent duration metrics and patch attribution for diff viewers.

## Consequences

- Sequential routing simplifies deterministic auditing and sandbox enforcement but defers parallel execution to a future phase.
- CLI/TUI/MCP consumers share a single registry/orchestrator implementation, reducing drift.
- Explicit configuration keys and env overrides unblock incremental rollout while keeping the feature off by default.
- Reusing the existing approval system with additional labels avoids bespoke tooling and keeps diff attribution intact.

## Alternatives Considered

- **Per-surface implementations** risked divergent behavior and higher maintenance cost.
- **Parallel execution first** would introduce coordination overhead without a compelling demo requirement.
- **Only YAML specification** lacked Markdown ergonomics for rich instruction bodies and was rejected in favor of Markdown with YAML frontmatter.

## Follow-Up

- Finalize subagent macro ergonomics (`codex-subagents-derive`).
- Document default models, timeouts, and retry behavior once orchestrator implementation stabilizes.
- Revisit parallel execution and caching after the sequential MVP ships.
