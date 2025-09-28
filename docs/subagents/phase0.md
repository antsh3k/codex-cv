# Subagents Phase 0 Dossier

## 1. Claude Subagent Behavior Study

Observations compiled from Anthropic product updates and public documentation:

- **Declarative agent specs**: Claude Projects define agents via structured YAML/Markdown files with metadata such as name, description, capabilities, and tool access. They allow per-agent model selection (e.g., Claude 3.5 Sonnet for reasoning-heavy agents) and enforce tool allowlists for safety.
- **Orchestrator expectations**: Claude runs subagents sequentially within Projects, showing status transitions like *waiting → running → completed*, and streams intermediate reasoning to the main transcript. Subagents inherit session policies (workspace, approvals) while maintaining isolated context windows.
- **UX signals**: The UI labels subagent runs with badges (agent name + model) and collapsible transcripts; errors surface with remediation hints instead of hard failures.
- **Registry & caching**: Projects cache agent definitions but offer manual reloads to pick up file edits. Overrides prioritize project-level configuration over account-level defaults.
- **Safety posture**: Tool access checks happen before execution with clear denial messages, and users can audit a subagent run post-hoc via event logs.

These patterns reinforce our design choices: Markdown+YAML specs, strong tool gating, sequential orchestration, and explicit labeling across surfaces.

## 2. Repository Anchors

| Domain | Primary crates/modules | Notes |
| --- | --- | --- |
| Agent specs & registry | `codex-subagents` (new), `codex-subagents/src/parser.rs`, `.../registry.rs` | Owns spec parsing, caching, mtime hashing, error reporting |
| Proc macros | `codex-subagents-derive`, `codex-subagents-derive/src/lib.rs` | Provides `#[derive(Subagent)]`, `#[subagent(...)]`, `trybuild` tests in `tests/` |
| Configuration | `codex-core/src/config.rs`, `.../config_types.rs`, `codex-core/src/flags.rs` | Add `subagents.enabled`, `subagents.auto_route`, env override handling |
| Conversation orchestration | `codex-core/src/conversation_manager.rs`, `codex-core/src/conversation_history.rs` | New `spawn_subagent_conversation`, orchestrator hooks, lifecycle events |
| Protocol events | `codex-protocol/src/protocol.rs`, `codex-protocol-ts/src/index.ts` | Extend enums and serialization for `SubAgent*` events and patch attribution |
| CLI integration | `codex-cli/src/main.rs`, `codex-cli/src/lib.rs`, `codex-cli/src/proto.rs` | New subcommands `subagents list/run`, wiring for event streaming |
| TUI UX | `codex-tui/src/app.rs`, `codex-tui/src/slash_command.rs`, `codex-tui/src/views/subagents.rs` (new) | Slash commands, agent list view, transcript nesting, status counters |
| MCP surface | `codex-mcp-server/src/lib.rs`, `codex-mcp-types` schemas | Add `subagents/list` & `subagents/run` handlers and schema updates |
| Telemetry & rollout logs | `codex-core/src/rollout`, `codex-core/src/token_data.rs` | Tag `SubAgent*` events, durations, patch origin metadata |
| Tests | `codex-subagents/tests/`, `codex-core/tests/`, `codex-tui/tests/`, CLI integration tests under `codex-cli/tests/` | Ensure coverage for parser precedence, orchestrator, CLI/TUI snapshots |

Ownership assumptions: Core team maintains `codex-core`, `codex-protocol`, `codex-protocol-ts`; CLI team owns `codex-cli`; TUI team owns `codex-tui`; infra team coordinates MCP and packaging updates.

## 3. Verification Targets

- **Formatters**: `just fmt` (workspace) + `cargo fmt` fallback; ensure new crates register with workspace `just fmt` recipe.
- **Lints**: `just fix -p codex-subagents` / `codex-subagents-derive` / `codex-core` / `codex-cli` / `codex-tui` depending on touched crates; add `#![deny(missing_docs)]` only after MVP.
- **Unit tests**: parser precedence, registry reload caching, TaskContext serialization, orchestrator lifecycle (mocked conversation spawn), tool policy enforcement, proc-macro `trybuild`, CLI command parsing, TUI slash command parsing, MCP route handlers.
- **Integration tests**: end-to-end pipeline fixture (subagent definitions + simulated run), CLI snapshot for `codex subagents list`, TUI insta snapshot for `/agents`, telemetry duration/unit tests, packaging smoke (ensure new crates built in `just release` and `pnpm package`).
- **Telemetry validation**: confirm `SubAgentStarted`/`Completed` durations and token counters feed existing telemetry sinks; add unit tests around duration math.
- **Docs & packaging**: update README, author `docs/subagents.md`, ensure `pnpm build` bundles new assets, validate cross-platform builds via CI matrix once implemented.

## 4. Demo Validation Playbook

1. **Setup**
   - Enable via `CODEX_SUBAGENTS_ENABLED=1` or config (`subagents.enabled = true`).
   - Create `.codex/agents/code-reviewer.md` with review instructions, `tools: ["apply_patch", "git_diff"]`, `model: gpt-5-codex`.

2. **Launch main session**
   - Start Codex CLI/TUI normally; submit a change request requiring a code review.
   - Observe legacy behavior to confirm baseline.

3. **Invoke subagent**
   - Use `/agents` to confirm discovery and view parse status.
   - Run `/use code-reviewer`; verify CLI logs `SubAgentStarted` with matching `sub_conversation_id`.

4. **Monitor execution**
   - Ensure TUI nests the subagent transcript with cyan headers and dimmed metadata.
   - Validate approvals show "Requested by code-reviewer (model: gpt-5-codex)".
   - Confirm tool usage respects allowlist (denied operations surface friendly errors).

5. **Completion & merge**
   - On completion, check merged summary in main transcript and presence of `PatchApply` events tagged with `origin_agent="code-reviewer"`.
   - Capture rollout log segment for documentation.

6. **Fallback scenarios**
   - Disable feature flag; rerun `/use code-reviewer` to confirm warning message and legacy execution.
   - Introduce malformed agent spec to verify diagnostics appear in `/agents` and CLI listing without panic.

