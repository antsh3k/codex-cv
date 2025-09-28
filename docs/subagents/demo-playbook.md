# Demo Validation Playbook — Code Reviewer Subagent

Date: September 27, 2025

## Scenario Overview

Showcase a `code-reviewer` subagent that reviews staged diffs, produces findings, and hands control back to the primary Codex session without regressions.

## Preconditions

1. Feature flag enabled: set `subagents.enabled = true` in `~/.codex/config.toml` or export `CODEX_SUBAGENTS_ENABLED=1`.
2. Demo repository prepared with a pending PR or staged diff containing intentional style and logic issues.
3. Seatbelt sandbox accessible (macOS: `/usr/bin/sandbox-exec`).

## Walkthrough

1. **List available agents**
   - CLI: `codex subagents list`
   - Expected output includes `code-reviewer` with `tools: ["git", "cargo", "npm"]` and source path.
   - TUI: run `/agents` and confirm project/user origin labels.
2. **Invoke the reviewer**
   - CLI: `codex subagents run code-reviewer -- prompt "Review the staged changes"`.
   - TUI: `/use code-reviewer`.
   - Observe `SubAgentStarted` telemetry and transcript banner (`code-reviewer · model: gpt-5.1` example).
3. **Subagent execution**
   - Steps: fetch diff summary → run static analysis (`just fix -p <crate>` dry-run) → produce findings.
   - Seatbelt: ensure all commands respect sandbox flags; if a command is blocked, return a fallback message.
4. **Results merge**
   - Subagent posts structured review message with severity-tagged findings.
   - UI shows nested transcript; CLI prints `SubAgentCompleted` with duration.
   - Confirm diffs include `origin_agent: "code-reviewer"` metadata.
5. **Follow-up commands**
   - Primary agent summarizes results.
   - User may run `/subagent-status` to confirm queue is empty.

## Failure Modes & Recovery

- **Registry parse error**: listing command surfaces YAML validation errors with file path.
- **Tool denied**: orchestrator returns `ToolNotAllowed` event; user can retry after editing `tools` allowlist.
- **Sandbox rejection**: command fails with seatbelt error; subagent emits fallback guidance to run manually.
- **Model override unavailable**: log warning, fall back to session model, notify user in transcript footer.

## Artifacts to Capture

- Transcript screenshot showing nested subagent run.
- CLI recording (asciinema/GIF) demonstrating list/run.
- Log snippet with telemetry payload for `SubAgentCompleted`.
- Diff viewer screenshot highlighting `origin_agent` annotation.
