# ADR 001: Subagents Feature Enablement Blueprint (2025-09-27)

- **Status**: Accepted (Phase 0 groundwork)
- **Owners**: Codex subagents strike team
- **Related work**: todo.md Phase 0, README Subagents section

## Context

Codex needs a demo-ready subagents capability that lets the primary agent delegate scoped tasks to specialized child agents without regressing existing flows. We must deliver a feature-flagged rollout, keep ergonomics aligned with GPT-5-Codex defaults, and ensure UX parity across CLI, TUI, and MCP surfaces. External precedent (Anthropic Claude projects and Claude 3.5 Sonnet's subagents) shows users expect declarative agent definitions, per-agent model choices, strict tool governance, and clear transcript labeling.

## Decision

1. **Feature gates & overrides**
   - Introduce config keys under `subagents.*` with defaults disabled:
     - `subagents.enabled = false`
     - `subagents.auto_route = false`
   - Add env override `CODEX_SUBAGENTS_ENABLED` with precedence over config (accepting `"1"`, `"true"`, or `"yes"`).
   - Future env override for auto routing is deferred until heuristics mature.

2. **Specification format**
   - Agents live in Markdown files with YAML frontmatter containing `name`, `description`, optional `model`, optional `tools` allowlist, optional `keywords`, and Markdown instructions body.
   - Discovery order: project-specific agents in `<repo>/.codex/agents/*.md` override user agents in `~/.codex/agents/*.md` by `name`.

3. **Runtime architecture**
   - A new `codex-subagents` crate provides registry, parser, orchestrator traits, builder pattern, and seatbelt-aware helpers.
   - A sibling `codex-subagents-derive` proc-macro crate offers `#[derive(Subagent)]` and the `#[subagent(...)]` attribute to minimize boilerplate and avoid cyclic dependencies.
   - The conversation manager gains `spawn_subagent_conversation` that clones base session config, applies overrides (model, tools), and returns an isolated `CodexConversation`.
   - An orchestrator coordinates sequential execution, lifecycle events, retry/backoff rules, and merging of results into the parent conversation.

4. **Cross-surface alignment**
   - CLI and TUI expose `/agents`, `/use`, `/subagent-status`, and `codex subagents list/run` entry points.
   - Protocol and TypeScript bindings gain `SubAgentStarted`, `SubAgentMessage`, and `SubAgentCompleted` events, plus optional `origin_agent`/`sub_conversation_id` fields on patch events for attribution.
   - MCP exposes `subagents/list` and `subagents/run` methods mirroring the CLI.

5. **Safety controls**
   - Tool usage obeys per-agent allowlists; violations raise immediate denials without prompting users.
   - Subagents inherit sandbox and approval policies; no relaxation of seatbelt rules.
   - `CODEX_DEBUG_SUBAGENTS=1` enables TaskContext debug serialization for troubleshooting without affecting production defaults.

## Consequences

- **Backward compatibility**: With flags disabled, legacy flows are untouched. New code paths must early-return when `subagents.enabled` is false.
- **Extensibility**: Sequential orchestration keeps the MVP simple but leaves room for parallel execution in the backlog.
- **Testing commitments**: Dedicated unit tests for parser precedence, registry caching, orchestrator lifecycle, macros (`trybuild`), CLI/TUI integration, and MCP smoke tests become mandatory.
- **Documentation**: README Subagents section remains the public guide; internal docs live under `docs/subagents/` for deeper operator knowledge.

## Demo blueprint (high-level)

1. Enable feature flag via `CODEX_SUBAGENTS_ENABLED=1` or config entry.
2. Place `code-reviewer.md` in `.codex/agents/` with review-specific instructions and tool allowlist (`apply_patch`, `git_diff`).
3. Run main session; invoke `/use code-reviewer` after submitting a diff request.
4. Observe streamed `SubAgent*` events, nested transcript in the TUI, and labeled approvals.
5. Merge subagent findings back into main conversation; confirm the run is logged with `sub_conversation_id`.

## Open Questions

- Auto-routing heuristics scope (keyword matching vs richer NLP) is deferred.
- Additional env overrides (e.g., `CODEX_SUBAGENTS_AUTO_ROUTE`) remain backlog until user demand surfaces.
- Seatbelt policy hooks for long-running subagents may require future instrumentation.

