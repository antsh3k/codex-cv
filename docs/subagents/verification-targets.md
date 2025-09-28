# Verification Targets for Subagent Workflow

Date: September 27, 2025

## Formatters & Linters

- **Rust**
  - `just fmt` (always run after touching Rust crates).
  - `just fix -p <crate>` for scoped clippy fixes.
  - `cargo fmt -p codex-subagents` to validate new crates in isolation when needed.
- **TypeScript/JavaScript**
  - `pnpm lint` (add subagent CLI checks once implemented).
  - `npm run format` to keep Markdown/JSON assets aligned with Prettier rules.
- **Docs**
  - `markdownlint-cli` (optional CI guard) to enforce heading/order conventions within `docs/subagents/`.

## Targeted Tests

- `cargo test -p codex-core` (subagent config + conversation manager tests).
- `cargo test -p codex-subagents` (registry/parser, TaskContext behavior).
- `cargo test -p codex-subagents-derive` (trybuild suites).
- `cargo test -p codex-tui` (slash command + rendering snapshots via `cargo insta`).
- `cargo test -p codex-cli` (CLI list/run commands).
- `cargo test --all-features` once shared crates (`core`, `protocol`) change.

## Packaging & CLI Smoke

- `pnpm --filter codex-cli... build` to ensure npm package bundles new artifacts.
- `cargo build -p codex-cli` for native binary integrity.
- `codex exec --subagent code-reviewer` scripted demo on sample repo (recorded for release docs).

## Telemetry & Observability

- Validate that `SubAgentStarted/Completed` events include duration, model, agent name, and `sub_conversation_id`.
- Ensure token usage metrics attribute `origin_agent` for diff patches.
- Confirm no personally identifiable information is logged in telemetry payloads.

## Sandbox & Safety Checks

- Verify `CODEX_SANDBOX` and `CODEX_SANDBOX_DISABLED` paths keep high-risk tests skipped.
- Ensure tool allowlists prevent disallowed command execution in orchestrated runs.
