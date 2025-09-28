# Subagents Sandbox & Approval Audit

This note captures the hardening checks completed for the delegated subagent pipeline.

## Tool allowlists

- Verified that `SubagentRegistry` propagates the `tools` array into the cloned configuration via `apply_tool_policy_from_spec`.
- Added unit coverage (`core/src/conversation_manager.rs::apply_tool_policy_respects_allowlist`) to confirm `apply_patch`, `plan`, and `web_search` are explicitly enabled while unrelated tools remain disabled.
- Manual spot-check: ran a subagent with a restricted spec and observed denial messaging (`Tool "local_shell" is not allowed for code-reviewer`).

## Sandbox environment guarantees

- Confirmed that the orchestrated child conversations inherit the parent sandbox policy. The spawned sessions expose `turn_context.subagent_name`, allowing downstream code to continue honoring `CODEX_SANDBOX` guards.
- Inspected `TurnDiffTracker` output to ensure `origin_agent` is preserved in `PatchApplyBegin/End`, enabling audit trails when reviewing shell executions.

## Approval styling and traceability

- TUI: approval overlays now label requests as `Requested by <agent> (model: ...)`, enabling differentiation between main agent and delegate.
- CLI: stream events include the agent name with the lifecycle messages so that logs can be audited without ambiguity.

## Telemetry & rollouts

- Each `SubAgentCompleted` event now carries a saturated `duration_ms`. The CLI/TUI render the human-readable duration and the telemetry helper logs the raw duration to the `codex::telemetry` target.
- Rollout persistence captures `SubAgentStarted/Completed` to maintain an immutable trail.

## Pending follow-ups

- Seatbelt policy auto-injection is limited to sequential execution; parallel runs will require an additional review.
- No changes were made to the upstream sandbox policies (`CODEX_SANDBOX_ENV_VAR`), respecting the constraint of not touching seatbelt helpers.
