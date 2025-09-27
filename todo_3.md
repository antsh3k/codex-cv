# Subagent Feature Build Plan

## Purpose
Deliver a sequential, feature-flagged subagent workflow that allows Codex to delegate work to specialized agents (Spec Parser, Code Writer, Tester, Reviewer) while maintaining full backward compatibility with the current single-agent path.

## Scope
- v2 focuses on sequential orchestration, typed shared context, CLI/TUI integration, and telemetry. Parallel execution, hot reload, and caching remain post-v2 enhancements.
- All work must adhere to Codex conventions: `codex-*` crates, builder patterns, `Result`-based APIs, `just` workflows, seatbelt-aware subprocess helpers, and sandbox env checks.

---

## Phase 0 – Design Validation (Week 0–1)
1. **Architecture Audit**
   - Review existing orchestrator entry points in `codex-rs/core` to choose between extending the current coordinator or introducing a new `codex-orchestrator` crate.
   - Identify touchpoints in CLI/TUI, MCP server, and protocol layers.
2. **Data Model Finalization**
   - Define typed structs: `RequestEnvelope`, `RequirementsSpec`, `ProposedChanges`, `TestPlan`, `TestResults`, `ReviewFindings`, and `TaskContext` with slot/guard semantics.
   - Write an ADR covering trait hierarchy (`Subagent`, `TypedSubagent`, `ContextualSubagent`), registry approach (inventory pattern), and feature gating strategy.
3. **Dependency Planning**
   - Confirm YAML/Markdown parsing dependencies (from `v_1_change.md`) for agent files.
   - Align telemetry schema updates with observability owners.

---

## Phase 1 – Framework Scaffolding (Week 2)
1. **Create `codex-subagents` Crate**
   - Expose traits (`Subagent`, `TypedSubagent<I, O>`, `ContextualSubagent`) and `SubagentBuilder`.
   - Implement derive/attribute macros (`#[derive(Subagent)]`, `#[subagent(...)]`) providing logging, retries (default 30s timeout), and registry hooks.
   - Provide seatbelt-aware subprocess helpers and model invocation wrappers respecting `CODEX_SANDBOX_*` (per `v_1_change.md`).
2. **Implement `TaskContext`**
   - Typed slots + namespaced scratchpads with read/write guards; support serialization for debug dumps (`CODEX_DEBUG_SUBAGENTS=1`).
   - Include diagnostic history, timestamps, and orchestrator status flags.
3. **Configuration Loading**
   - Parse project/user agent files from `.codex/agents/*.md` using YAML frontmatter + Markdown body (`v_3_change.md`).
   - Respect precedence (project overrides user, duplicates logged) and warm cache by mtime hash.
   - Extend `codex-rs/core/src/config.rs` with `subagents.enabled` and `subagents.auto_route` flags.

---

## Phase 2 – Orchestrator & Pipeline Skeleton (Week 3)
1. **Orchestrator Module**
   - Create `codex-orchestrator` (or equivalent module) managing sequential execution, retries, and lifecycle events.
   - Initialize `TaskContext`, call subagents in order, and collate outputs.
2. **Registry & Router**
   - Implement registry (`registry.rs`) and router (`router.rs`) that match user turns to subagents via explicit commands or keyword heuristics (`v_3_change.md`).
   - Provide API for orchestrator to request subagent metadata and dependency checks.
3. **Protocol Extensions**
   - Add event variants in `codex-rs/core/src/protocol/mod.rs`: `SubAgentStarted`, `SubAgentMessage`, `SubAgentCompleted` with parent context IDs.
   - Update TypeScript `protocol-ts` and MCP message processors to surface new events (`v_3_change.md`).
4. **Feature Flag Wiring**
   - Gate orchestrator path behind CLI flag `--use-subagents` and config entry (`as_v_2_change.md`). Fallback to legacy path when disabled.

---

## Phase 3 – Core Subagents (Week 4–6)
1. **Spec Parser** (`codex-rs/subagents/spec_parser`)
   - Prompt templates, schema validation (ensure every requirement has ID and success criteria).
   - Unit fixtures converting sample requests into `RequirementsSpec` snapshots.
2. **Code Writer**
   - Integrate repository context adapters (file tree summaries, diff utilities).
   - Produce `ProposedChanges` with diff metadata, run formatting checks (dry-run `cargo fmt`/`just fmt`).
3. **Tester**
   - Generate tests (`TestPlan`), execute via sandbox-safe harness, capture logs and pass/fail/error states.
   - Provide fallback messaging when execution blocked by sandbox.
4. **Reviewer**
   - Apply style/security heuristics, map findings to severity taxonomy, emit actionable `ReviewFindings`.
   - Reuse lint tools where available; fallback to LLM prompts when necessary.
5. **Shared Validation**
   - Ensure each subagent declares tool allowlists; hook into approval tracker for enforcement (from `v_1_change.md`).

---

## Phase 4 – UX & Interface Integration (Week 7)
1. **CLI/TUI Enhancements**
   - Add `/agents`, `/use`, `/subagent-status` commands; update `codex-rs/tui/src/slash_command.rs`, status panes, history renderer (nest subagent output with Stylize helpers `.cyan()`, `.dim()`).
   - CLI flags: `codex --list-subagents`, `codex exec --subagent <name>`.
2. **Approval & Diff Tracking**
   - Tag approval prompts with `agent_name`; enforce tool allowlists before seeking approval.
   - Extend `turn_diff_tracker` to record subagent origin and avoid conflicting edits.
3. **Rollout Persistence**
   - Store nested runs in rollout logs for replay; initially read-only fallback (subagent runs not resumable individually).
4. **MCP & External Interfaces**
   - Expose `subagents/list` and `subagents/run` tools in `codex-rs/mcp-server`.
   - Update docs (`docs/config.md`, notifications) with new event types.

---

## Phase 5 – Testing & Hardening (Week 8)
1. **Unit Tests**
   - Trait helpers, macro expansion (`trybuild`), parser precedence/error handling, router matching, orchestrator lifecycle (mocked spawn).
2. **Integration Tests**
   - End-to-end pipeline on fixture repo; ensure sequential execution and rollback paths.
   - TUI snapshot for `/agents` list; CLI acceptance for `codex exec --subagent` command.
3. **Sandbox & Approval Audits**
   - Verify tool allowlists enforced server-side; ensure sandbox and approval policies align with `v_1_change.md` security guidance.
4. **Telemetry Validation**
   - Confirm logs/metrics emit expected data (`event="subagent_start"`, `duration_ms`, outcome counts).
   - Build dashboards for success rate, retries, average latency.

---

## Rollout Plan
1. **Feature Flag**: default `subagents.enabled = false`; enable for internal testing.
2. **Pilot**: Enable for power users, gather feedback, iterate on UX.
3. **Documentation**: Publish `docs/subagents.md`, quickstart, troubleshooting, migration guide from manual flows (`v_2_change.md`).
4. **General Availability**: Flip default after telemetry shows stable success/error ratios and UX issues addressed.

---

## Risks & Mitigations
- **Tool Conflicts**: Mitigate with allowlists and approval tagging (from `v_1_change.md`).
- **Context Budgeting**: Subagent conversations maintain isolated buffers; summarization tuned per agent to protect main context.
- **Concurrency**: Keep v2 sequential; orchestrator rejects overlapping subagent runs until completion (per `v_3_change.md`).
- **User Clarity**: UI labels “Requested by <agent>”; documentation and CLI help explain delegation behavior.
- **Maintenance Load**: Modular subagents live under `codex-rs/subagents/<name>` with dedicated tests to simplify updates.

---

## Post-v2 Enhancements (Backlog)
- Parallel subagent execution with dependency graph resolution.
- Hot reload/watch mode for subagent development.
- Built-in profiling and caching layers per agent.
- Marketplace/registry for community subagents with versioning and approval workflows.
- Human-in-the-loop checkpoints between subagent stages.

