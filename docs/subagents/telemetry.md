# Telemetry Notes

The subagent pipeline now emits duration metrics that can be reused by existing telemetry sinks.

## Logged fields

Each `SubAgentCompleted` event logs the following payload via `tracing`:

- `event = "subagent_run"`
- `agent` – canonical name from the spec metadata
- `model` – resolved model label
- `success` – boolean derived from the outcome
- `duration_ms` – saturated wall-clock duration in milliseconds

## Dashboard sketch

Until the formal dashboards are wired up, the quickest way to visualise usage is:

1. Forward `codex::telemetry` logs to the central logging stack (e.g., Honeycomb).
2. Create a derived dataset with the following aggregations:
   - Average duration per agent over time.
   - P95 duration per agent.
   - Failure rate grouped by `agent` and `model`.
3. Add a table view of the last 50 runs to aid triage when investigating incidents.

## Future automation

- Plumb the telemetry stream into the existing token usage dashboard so the duration chart can sit alongside cost metrics.
- Consider emitting counters for tool usage per agent if the orchestration layer requires deeper audits.
