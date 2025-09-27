# Subagent Integration Plan for codex-cv

This document captures a preliminary assessment of what it would take to add Claude Code–style subagents to codex-cv, including required architecture changes, risks, and estimated effort.

## Concept Recap

- Subagents are specialized AI personas with their own context windows, tool allowlists, and system prompts.
- They can be invoked automatically (based on task heuristics) or explicitly by the user.
- Each subagent’s workspace should stay independent to avoid polluting the main conversation history.

## Required Changes

### 1. Agent Registry and Storage

- Extend Composer configuration loading to discover agent profiles at `~/.codex-cv/agents/*.md` and project-level `.codex-cv/agents/*.md`, using Markdown with YAML frontmatter.
- Define an `AgentProfile` type in `codex-core` that captures `name`, `description`, `tools`, `model`, and system prompt.
- Add caching or file watching so updates to agent files are picked up without restarting the session.

### 2. Protocol and Frontend Updates

- Update `codex_protocol::{config_types, protocol}` plus the TypeScript `protocol-ts` crate to describe agent metadata, delegation events, and subagent transcript boundaries.
- Teach the TUI/CLI to expose an `/agents` workflow: list, create, edit, and delete agents, surface when a subagent is active, and allow explicit invocation.
- Refresh documentation, onboarding flows, and help output to explain subagent management.

### 3. Conversation Orchestration

- Refactor `ConversationManager`/`ModelClient` to launch sub-conversations with isolated message buffers, token budgets, and summarization logic.
- Ensure subagents inherit or override tool access via new allowlist hooks in `plan_tool`, `tool_apply_patch`, and related modules.
- Implement result folding so artifacts (diffs, execution logs, summaries) return to the main conversation cleanly.

### 4. Sandbox and Approval Semantics

- Extend approval-policy handling so each agent can default to its own `AskForApproval` level while still respecting session-wide overrides.
- Enforce sandbox restrictions per agent, ensuring that powerful tools stay gated to trusted personas.

### 5. Model Routing

- Allow agent profiles to choose between `gpt-5`, `gpt-5-codex`, OSS, or inherited models by plumbing selections through `model_provider_info`, config overrides, and runtime client setup.
- Verify fallback behavior when preferred providers are unavailable.

### 6. Persistence and Rollouts

- Modify rollout/session logging to capture agent boundaries, ensuring the viewer can replay both main and delegated conversations.
- Audit diff tracking (`turn_diff_tracker`, `apply_patch`) for concurrent edits; introduce queuing or workspace snapshots to avoid collision when multiple agents write files.

## Risks and Open Questions

- **Isolation:** Current apply-patch and diff tracking assumes one active editor. Concurrent agent edits risk conflicts without coordination.
- **Context Budgeting:** Summarization and truncation live in shared modules; they need isolation so delegated work does not consume the main context window.
- **Approval UX:** Users must understand when a subagent runs commands autonomously; misconfigured defaults could surprise them.
- **Testing Footprint:** New flows will require integration tests across CLI, TUI, and MCP server modes to ensure delegation works everywhere.

## Effort Estimate

- Backend architecture: ~4–6 weeks for agent registry, orchestration refactor, protocol changes, and tool gating.
- Frontend & UX work: ~2–3 weeks for `/agents` UI, documentation, and settings.
- Validation and hardening: ~2 weeks for integration tests, snapshot updates, and sandbox/approval audits.

Staging proposal:

1. Implement agent profile parsing and manual invocation.
2. Layer in automatic delegation heuristics once manual flows stabilize.
3. Address concurrency, approval smoothing, and rollout visibility before general release.
