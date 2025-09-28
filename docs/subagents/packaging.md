# Packaging & Release Checklist

This document records the actions taken to ensure subagent support is packaging-ready.

## Build orchestration

- Verified that the `just release` pipeline runs the Rust workspace build prior to invoking any npm packaging scripts. No additional steps were required because the new crates (`codex-subagents`, `codex-subagents-derive`) are part of the default workspace members.
- `pnpm` workflows reference the compiled CLI/TUI binaries; no additional bundling changes were needed. We confirmed the `pnpm` build respects the feature flag defaults.

## Executable artifacts

- Local smoke tests executed on macOS using both the CLI and the TUI with `CODEX_SUBAGENTS_ENABLED` toggled on/off to validate reproducibility.
- Windows/Linux builds will continue to flow through CI; no OS-specific logic was added for subagents.

## Distribution assets

- `docs/subagents/spec.md` and `docs/subagents/demo.md` provide user-facing onboarding content. These files will be linked from the release notes when the feature flag graduates beyond internal.
- The TUI snapshot test (`subagent_list_snapshot`) now normalises project paths, ensuring cross-machine determinism for recorded artefacts.

## API surface

- Rust: `SubAgentCompletedEvent` exposes the optional `duration_ms` for downstream consumers.
- MCP: Existing `subagents/list` and `subagents/run` routes continue to work; duration data is available in the streaming events.
- TypeScript: The `codex-protocol-ts` bindings were regenerated to include the `duration_ms` field, ensuring npm consumers can access the telemetry metadata once published.

## Follow-up work

- Publish updated npm packages once internal validation is complete.
- Coordinate with the release management track (see `docs/release_management.md`) before flipping the feature flag by default.
