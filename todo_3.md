# Subagent Feature Build Plan

## Purpose
Deliver a sequential, feature-flagged subagent workflow that allows Codex to delegate work to specialized agents (Spec Parser, Code Writer, Tester, Reviewer) while maintaining full backward compatibility with the current single-agent path.

## Scope
- v2 focuses on sequential orchestration, typed shared context, CLI/TUI integration, telemetry, and optional per-subagent model overrides. Parallel execution, hot reload, caching, and other advanced behaviors remain post-v2 enhancements.
- All work must adhere to Codex conventions: `codex-*` crates, builder patterns, `Result`-based APIs, `just` workflows, seatbelt-aware subprocess helpers, and sandbox env checks.

---

## Phase 0 – Design Validation
- [ ] **Audit orchestrator touchpoints**
  - Review existing orchestrator entry points in `codex-rs/core` to choose between extending the current coordinator or introducing a new `codex-orchestrator` crate.
  - Identify required updates in CLI/TUI, MCP server, protocol layers, and telemetry pipelines.
- [ ] **Finalize data models**
  - Define typed structs: `RequestEnvelope`, `RequirementsSpec`, `ProposedChanges`, `TestPlan`, `TestResults`, `ReviewFindings`, `TaskContext` (typed slots + guards).
  - Document schema in an ADR that also covers trait hierarchy (`Subagent`, `TypedSubagent`, `ContextualSubagent`), registry mechanism (inventory pattern), and feature gating strategy.
- [ ] **Confirm dependencies and telemetry**
  - Select YAML/Markdown parsing crates for agent definitions.
  - Coordinate required telemetry schema changes with observability owners.

---

## Phase 1 – Framework Scaffolding
- [ ] **Create `codex-subagents` crate**
  - Implement traits (`Subagent`, `TypedSubagent<I, O>`, `ContextualSubagent`) and `SubagentBuilder` supporting optional `model` overrides and defaulting to the session model when unspecified.
  - Provide derive/attribute macros (`#[derive(Subagent)]`, `#[subagent(...)]`) that auto-register agents, wire logging, retries (default 30 s), metrics, and enforce tool allowlists.
  - Add seatbelt-aware subprocess/model invocation helpers that respect `CODEX_SANDBOX_*` env vars.
- [ ] **Implement `TaskContext`**
  - Typed slots for each artifact plus namespaced scratchpads; enforce read/write guards to maintain consistency.
  - Support serialization for debug dumps when `CODEX_DEBUG_SUBAGENTS=1`; include diagnostic history and orchestrator status flags.
- [ ] **Agent configuration loading**
  - Parse project and user agent definitions from `.codex/agents/*.md` (YAML frontmatter + Markdown body).
  - Apply precedence rules (project overrides user), warm cache entries by mtime hash, and surface invalid configs in `/agents` output.
  - Extend `codex-rs/core/src/config.rs` with `subagents.enabled`, `subagents.auto_route`, and optional defaults (e.g., fallback model, tool policies).

---

## Phase 2 – Orchestrator & Pipeline Skeleton
- [ ] **Build orchestrator module**
  - Create `codex-orchestrator` (or extend existing coordinator) to manage sequential execution, retries, timeout escalation, and lifecycle events.
  - Initialize `TaskContext`, invoke subagents in configured order, collate outputs, and merge results back into the main session.
- [ ] **Implement registry and router**
  - Add `registry.rs` to expose loaded `SubagentSpec` objects (including optional `model` field) and `router.rs` to map user turns to subagents via explicit commands or keyword heuristics.
  - Expose APIs for dependency checks and pipeline composition.
- [ ] **Protocol and messaging updates**
  - Extend `codex-rs/core/src/protocol/mod.rs` (and TypeScript bindings) with `SubAgentStarted`, `SubAgentMessage`, `SubAgentCompleted`, carrying parent IDs, chosen model, and agent metadata.
  - Update MCP message processors and exec-mode JSON contracts to relay new event types.
- [ ] **Feature flag integration**
  - Gate the orchestrator path behind CLI flag `--use-subagents` and configuration entries; ensure graceful fallback to legacy flow when disabled.

---

## Phase 3 – Core Subagents
- [ ] **Spec Parser (`codex-rs/subagents/spec_parser`)**
  - Implement prompt templates, schema validation (IDs, success criteria), and unit fixtures mapping sample requests to `RequirementsSpec` snapshots.
- [ ] **Code Writer**
  - Integrate repository context adapters (file tree summaries, diff utilities), produce `ProposedChanges`, and verify formatting via dry-run `just fmt`/`cargo fmt`.
- [ ] **Tester**
  - Generate `TestPlan`, execute tests through sandbox-safe harnesses, capture logs/test outcomes, and provide clear messaging when execution is blocked by sandbox policies.
- [ ] **Reviewer**
  - Apply style/security heuristics, leverage lint tooling or LLM prompts as needed, and emit actionable `ReviewFindings` with severity taxonomy.
- [ ] **Shared enforcement**
  - Ensure subagents declare tool allowlists; integrate with approval tracker to enforce permissions before execution.
  - Confirm model override preference flows from `SubagentSpec` to `SubagentBuilder` to orchestrator when instantiating each agent.

---

## Phase 4 – UX & Interface Integration
- [ ] **CLI/TUI updates**
  - Implement `/agents`, `/use`, `/subagent-status`; adjust `codex-rs/tui/src/slash_command.rs`, status panes, history renderer (nest subagent transcripts with Stylize helpers like `.cyan()` and `.dim()`).
  - Add CLI commands: `codex --list-subagents`, `codex exec --subagent <name>`; ensure model information is visible in status outputs.
- [ ] **Approval and diff tracking**
  - Tag approval prompts/events with `agent_name` and selected `model`; enforce allowlists prior to approval prompts.
  - Extend `turn_diff_tracker` to record subagent origin and guard against conflicting edits.
- [ ] **Rollout persistence & telemetry**
  - Persist nested runs in rollout logs (read-only fallback if subagent runs cannot be resumed individually).
  - Emit structured telemetry for start/completion events including model usage and outcome; update dashboards accordingly.
- [ ] **MCP/external integrations**
  - Surface `subagents/list` and `subagents/run` tools in `codex-rs/mcp-server` and update notifications/docs (`docs/config.md`, notifications config) to explain new event types and model override behavior.

---

## Phase 5 – Testing & Hardening
- [ ] **Unit tests**
  - Cover trait helpers, macro expansions (`trybuild`), parser precedence/errors, router matching, orchestrator lifecycle (mocked spawn with different models).
- [ ] **Integration tests**
  - Execute end-to-end pipeline on fixture repo; ensure sequential execution, rollback handling, and correct model selection (custom vs default).
  - Add TUI snapshot for `/agents` list and CLI acceptance tests for `codex exec --subagent`.
- [ ] **Sandbox & approval audits**
  - Validate tool allowlist enforcement server-side; cross-check sandbox approvals align with `CODEX_SANDBOX_*` expectations.
- [ ] **Telemetry verification**
  - Confirm logs and metrics emit expected fields (`event="subagent_start"`, `subagent`, `model`, `duration_ms`, `outcome`).
  - Ensure dashboards reflect success/failure ratios and per-model usage patterns.

---

## Rollout
- [ ] **Feature flag control**: keep `subagents.enabled` false by default; enable for internal testers once regression suite passes.
- [ ] **Pilot feedback loop**: gather UX/input from power users, iterate on heuristics and UI clarity.
- [ ] **Documentation**: publish `docs/subagents.md`, quickstart, troubleshooting guide, and migration instructions for adopting subagents (manual invocation and auto routing).
- [ ] **General availability switch**: enable by default after telemetry and feedback indicate stability; announce change in release notes and update onboarding materials.

---

## Risks & Mitigations
- **Tool conflicts**: enforce allowlists + approval tagging to prevent unauthorized tool use.
- **Context budgeting**: isolate subagent transcripts and tune summarization to protect main session context.
- **Concurrency**: keep v2 sequential; orchestrator rejects concurrent subagent runs until current run completes.
- **User clarity**: clearly label agent name and model in UI/CLI; document delegation behavior thoroughly.
- **Maintenance overhead**: house each agent in dedicated modules with targeted tests to simplify future updates.

---

## Post-v2 Enhancements (Backlog)
- Parallel subagent execution with dependency graph resolution.
- Hot reload/watch mode for subagent development.
- Built-in profiling and caching layers configurable per agent.
- Marketplace/registry for community-contributed subagents with versioning and approval workflow.
- Human-in-the-loop checkpoints between subagent stages.

