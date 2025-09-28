# Repository Anchors for Subagents Implementation

Date: September 27, 2025

## Rust Workspace Highlights (`codex-rs/`)

- `core/`
  - `src/config.rs`: central configuration definitions; will host `subagents.*` keys.
  - `src/conversation/manager.rs`: conversation orchestration; extension point for `spawn_subagent_conversation`.
  - `src/events/`: event definitions for protocol bridging; extend with `SubAgent*` variants and diff attribution metadata.
- `tui/`
  - `src/slash_command.rs`: slash command parser; add `/agents`, `/use`, `/subagent-status`.
  - `src/ui/history/`: transcript rendering; annotate subagent runs, label approvals, and nest transcripts.
- `cli/`
  - `src/commands/`: CLI subcommand dispatch; integrate `codex subagents list/run`.
- `protocol/` and `protocol-ts/`
  - Rust and TypeScript protocol definitions; update `EventMsg` union and regenerate bindings.
- `mcp-server/` and `mcp-types/`
  - MCP tool exposure; add `subagents/list` and `subagents/run` methods.

## Planned New Crates

- `codex-subagents`
  - Registry parser (`parser.rs`, `registry.rs`).
  - Traits: `Subagent`, `TypedSubagent`, `ContextualSubagent`.
  - Runtime structures: `TaskContext`, `SubagentBuilder`, error enums.
- `codex-subagents-derive`
  - Proc macros for `#[derive(Subagent)]` and attribute helpers.
  - `tests/trybuild/` for compile-time validation.

## TypeScript / npm Surface (`codex-cli/`)

- `packages/codex-cli/src/commands/`: CLI entrypoints to map new subagent flows.
- `packages/codex-cli/src/protocol/`: generated types; ensure compatibility with Rust protocol changes.

## Documentation & Tooling

- `docs/`: central location for spec, demo playbook, contributor guide.
- `justfile`: gating commands (`just fmt`, `just fix -p <crate>`, targeted tests) that must be wired into subagent pipeline.

## Ownership & Collaboration Notes

- Core orchestration: Runtime team (@core-runtime) — coordinate reviews for `codex-core` changes.
- TUI/CLI integration: DX experience team (@dx-surface) — align on slash command UX and CLI flags.
- Protocol bindings: Platform team (@protocols) — schedule simultaneous Rust/TypeScript updates.
- Docs/demo: Developer Enablement (@enablement) — maintain the onboarding materials under `docs/subagents/`.

Keep this map updated as modules land to preserve shared situational awareness.
