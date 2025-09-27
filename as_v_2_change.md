# Subagent Framework (v2) — Developer-First Design

## Purpose & Scope
- Establish a first-class workflow for delegating Codex tasks to focused subagents (Spec Parser, Code Writer, Tester, Reviewer) without regressing existing single-agent behavior.
- Deliver an idiomatic Rust experience that matches Codex conventions (`codex-*` crates, builder patterns, `Result`-based APIs, `just` workflows, seatbelt awareness).
- Limit v2 to sequential execution with a shared task context; async orchestration and marketplace concepts remain future candidates, not v2 scope.

## Design Pillars
- **Trivial to start, powerful to extend.** Provide a zero-boilerplate path for simple helpers while exposing optional traits for typed/contextual agents.
- **Predictable data contracts.** Subagents exchange typed structs (`RequirementsSpec`, `ProposedChanges`, `TestPlan`, `ReviewFindings`) managed by a shared `TaskContext`.
- **Observability-first.** Every run emits structured logs, traces, and metrics to make debugging and replay straightforward.
- **Safe-by-default.** Respect sandbox env vars, make dependencies explicit, and route filesystem/network access through vetted adapters.

## Quick Start (Simple Subagent)
```rust
use codex_subagents::prelude::*;

#[subagent(name = "hello_world")]
fn hello_world(input: &str) -> Result<String> {
    Ok(format!("Processed: {}", input))
}
```
- Declarative attribute registers the subagent and wires in logging/metrics.
- Returns plain strings; the framework handles retries, timeouts, and serialization.
- Ideal for lightweight helpers, matching the "simple by default" principle.

## Progressive Capability Path
1. **Function Subagents** — quickest entry point (`fn(&str) -> Result<String>`).
2. **Typed Subagents** — opt into `TypedSubagent<I, O>` for structured payloads:
   ```rust
   #[derive(Subagent)]
   #[subagent(name = "spec_parser")]
   pub struct SpecParser;
   
   impl TypedSubagent<RequestEnvelope, RequirementsSpec> for SpecParser {
       fn run(&self, input: RequestEnvelope) -> Result<RequirementsSpec> { /* ... */ }
   }
   ```
3. **Contextual Subagents** — implement `ContextualSubagent` when direct `TaskContext` mutation is required (e.g., coordinating multiple artifacts).
4. **Custom Builders** — `SubagentBuilder::new(CodeWriter::default()).with_model("gpt-4o-mini").with_timeout(Duration::from_secs(60)).finish()?;` keeps advanced tuning explicit.

## Architectural Overview
- **Orchestrator (new `codex-orchestrator` module or expansion of existing coordinator)**
  - Chooses the active pipeline, initializes `TaskContext`, and invokes registered subagents sequentially.
  - Handles retries, timeout escalation, and failure propagation.
  - Emits lifecycle events consumable by the CLI/TUI.
- **TaskContext (new `codex-subagents` crate)**
  - Typed slots for `request`, `requirements`, `proposed_changes`, `test_results`, `review_findings`, plus namespaced scratchpads for subagent-specific metadata.
  - Read/write guards ensure consistent state updates.
- **Registry & Configuration**
  - Attribute macros (`#[subagent(name = "code_writer", requires = ["spec_parser"], description = "Generates code diffs")]`) declare metadata and dependencies.
  - Registry discovered at compile time via inventory; orchestrator respects config order defined in `codex.toml`/`subagents.toml`.
- **Execution Pipeline (default sequential)**
  1. Spec Parser → `RequirementsSpec`
  2. Code Writer → `ProposedChanges`
  3. Tester → `TestPlan` + `TestResults`
  4. Reviewer → `ReviewFindings`
  - Pipelines are configurable; orchestrator can skip/replace subagents per config flags or feature gates.

## Developer Experience
- **Scaffolding command**: `codex subagent new <name>` creates `codex-rs/subagents/<name>/mod.rs`, registers the crate feature, and generates baseline tests + docs stub.
- **Macro ergonomics**: derive macros/attributes provide logging, metrics, retries (default 30s timeout) without handwritten boilerplate.
- **Testing harness**: `cargo test -p codex-subagents -- my_agent::tests` for unit tests; orchestrator integration tests live under `codex-orchestrator/tests`.
- **Sandbox compliance**: helper APIs expose seatbelt-aware subprocess runners; network usage requires explicit opt-in and respects `CODEX_SANDBOX_*` guards.
- **Getting started guide**: documented walk-through (CLI scaffold → implement → run → integrate) ensures new contributors can build a subagent in under an hour.

## Key Data Structures
- `RequestEnvelope` — captures raw user input, metadata, repository context pointers.
- `RequirementsSpec` — goals, constraints, success metrics, open questions.
- `ProposedChanges` — structured list of file edits with diff metadata and rationale.
- `TestPlan` / `TestResults` — tests to run, execution outcome, logs, coverage hints.
- `ReviewFindings` — severity, category, message, recommended follow-up.
- `TaskContext` — aggregates the above plus diagnostic history, timestamps, and orchestrator status flags.

## Implementation Roadmap
1. **Design Validation (Week 0–1)**
   - Audit existing orchestration entry points and decide whether to extend or fork into `codex-orchestrator`.
   - Finalize schemas for `TaskContext` and typed payloads; publish ADR covering trait hierarchy and registry choice.
2. **Framework Scaffolding (Week 2)**
   - Create `codex-subagents` crate with traits, derive macros, builder utilities, error types, logging helpers.
   - Add workspace feature gate `subagents`; integrate config loader with default disabled state.
3. **Pipeline Skeleton (Week 3)**
   - Implement orchestrator shell, placeholder subagents, lifecycle logging, metrics wiring.
   - Add CLI flag `--use-subagents` / config entry to toggle the new flow.
4. **Core Subagents (Week 4–6)**
   - Spec Parser: prompt templates, schema validation, unit fixtures.
   - Code Writer: repo context adapters, diff generation, formatting checks.
   - Tester: test synthesis, sandbox-aware execution harness, result capture.
   - Reviewer: style/security heuristics, severity taxonomy, actionable output.
5. **Integration & UX (Week 7)**
   - End-to-end pipeline test against a sample repo; capture artifacts for regression.
   - Stream subagent status to TUI/CLI; implement graceful fallback when a step fails or is disabled.
6. **Docs & Enablement (Week 8)**
   - Author `docs/subagents.md`, quickstart tutorial, and examples under `examples/subagents/`.
   - Publish migration notes for adding bespoke subagents.

## Testing & Quality Strategy
- Unit tests for trait helpers, macro expansion (`trybuild`), data model serialization.
- Mock orchestrator tests with synthetic subagents to exercise retry/timeout/error branches.
- Snapshot tests for Spec Parser/Reviewer outputs to guarantee format stability.
- Integration harness that runs the full pipeline on a fixture repo; auto-skip when sandbox envs block subprocesses.
- CI steps: `cargo test -p codex-subagents`, targeted `just fix -p codex-subagents`, orchestrator integration suite behind feature flag.

## Observability & Tooling
- Structured logs (`event="subagent_start"`, `subagent="spec_parser"`, `duration_ms`, `outcome`).
- Metrics exported via existing telemetry crates; dashboards for success rate, retries, latency.
- Optional debug dumps of `TaskContext` snapshots when `CODEX_DEBUG_SUBAGENTS=1`.
- CLI helpers: `codex subagent run <name> --input fixture.json` for local iteration; `codex subagent inspect <name>` to view metadata/config.

## Future Enhancements (Post-v2)
- Parallel execution with dependency graph resolution once data contracts stabilize.
- Hot reload / watch mode for subagent development.
- Built-in profiling and caching layers configurable per subagent.
- Marketplace/registry for community-contributed subagents with versioning.
- Human-in-the-loop checkpoints between subagent stages.

## Alignment With Codex Standards
- New crates follow naming convention (`codex-subagents`, `codex-orchestrator`).
- APIs mirror patterns used in `codex-core` (builder pattern, `thiserror`, `anyhow`, `Result`).
- Tooling integrates with existing `just fmt` / `just fix -p <crate>` workflows and respects seatbelt env checks.
- Documentation style matches current guides: concise sections, actionable instructions, focused code samples.

## Open Items Before Build
- Decide orchestrator placement (new crate vs extending existing coordinator).
- Confirm configuration format (`codex.toml` extension vs new `subagents.toml`).
- Align resource limits (memory, model tokens) with infra team.
- Coordinate telemetry schema updates with observability owners.
- Validate CLI surface (`codex subagent ...`) with DX stakeholders.

## Definition of Done (v2)
- Feature-flagged orchestrator executes Spec Parser → Code Writer → Tester → Reviewer sequentially and returns a consolidated report.
- Developers can scaffold, implement, test, and register a new subagent in under one hour using provided tooling.
- Documentation, tests, and telemetry updates merged; legacy workflows remain unchanged when `--use-subagents` is disabled.
