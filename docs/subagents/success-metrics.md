# Success Metrics — Subagents MVP

Date: September 27, 2025

## Demo Readiness

- ✅ Scripted `code-reviewer` run completes without manual intervention on macOS and Linux demo environments.
- ✅ Transcript screenshot captures nested subagent output and diff attribution.
- ✅ CLI recording published alongside demo instructions.

## Developer Experience

- Onboarding doc enables a new contributor to add a subagent spec and run it locally within 30 minutes.
- End-to-end formatter + lint cycle (`just fmt`, `just fix -p`, targeted `cargo test`) finishes under 12 minutes on a cold machine.
- `codex subagent new <name>` scaffolds a runnable agent with passing unit tests in under 5 seconds.

## Safety & Reliability

- Tool allowlist violations are detected 100% of the time and reported as `ToolNotAllowed` events.
- All orchestrated runs respect sandbox flags; zero incidents of escaping `CODEX_SANDBOX=seatbelt`.
- Telemetry coverage: 95%+ of subagent runs emit duration and model metadata; 0% contain PII.

## Rollout Health

- Internal preview: 5 pilot repos run the feature for one week with no high-severity incidents.
- Beta gate: error rate <2% across subagent runs (excluding intentional sandbox denials).
- GA: feature flag default toggled after instrumentation confirms success metrics for two consecutive release trains.
