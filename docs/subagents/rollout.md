# Rollout Plan

This section captures the recommended rollout ladder for subagents.

1. **Internal validation**
   - Keep `subagents.enabled` defaulted to `false` in `config.toml`.
   - Limit usage to pilot teams via the `CODEX_SUBAGENTS_ENABLED=1` environment flag.
   - Monitor the new telemetry stream for anomalies (see `docs/subagents/telemetry.md`).
2. **Pilot feedback loop**
   - Track issues, heuristics tweaks, and UI nits in a shared log (e.g., `docs/subagents/pilot-notes.md`).
   - Iterate on agent specs without changing the default flag until the feedback log is empty.
3. **GA preparation**
   - Update release notes with the new demo script (`docs/subagents/demo.md`).
   - Coordinate with release management to flip the default value after regression tests pass.
   - Announce the change in the onboarding docs once the default transitions to enabled.
