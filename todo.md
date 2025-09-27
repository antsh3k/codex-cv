# Codex Subagents Initiative — Unified Build Plan

## Mission
Deliver a sequential, feature-flagged subagent workflow that showcases Codex’s ability to delegate work to specialized agents while remaining backward compatible, demo-ready, and aligned with GPT-5-Codex ergonomics. The system must support Markdown+YAML agent definitions, optional per-agent model overrides, clear UX affordances across CLI/TUI/MCP, and pave the path toward broader distribution (e.g., npm packaging).

## Success Criteria
- Demo scenario: create/specify/launch a `code-reviewer` agent, observe delegated execution, and merge results back into the main conversation without regressions.
- Feature gated: legacy behavior unchanged when `subagents.enabled` is false.
- Tooling compliant: `just fmt`, scoped `just fix -p`, targeted `cargo test`, telemetry hooks, sandbox env respect.
- Packaging readiness: pipelines and docs updated so the feature can ship in future npm releases.

---

## Phase 0 — Design & Research Alignment
- [ ] **Lock guiding ADR** capturing hybrid strategy (demo blueprint + ergonomic framework + safety controls + surface alignment).
- [ ] **Study Claude subagent behavior** to distill practical implications (registry, orchestrator, UX, context isolation, model/tool overrides).
- [ ] **Map repository anchors** (modules, structs, commands) for fast navigation and code ownership notes.
- [ ] **Define verification targets** (formatters, lints, targeted tests, packaging smoke, telemetry validation).
- [ ] **Draft demo validation playbook** describing the end-to-end walkthrough, failure modes, and recovery steps.

---

## Phase 1 — Framework Scaffolding & Registry
- [ ] **Create `codex-subagents` crate** with traits (`Subagent`, `TypedSubagent`, `ContextualSubagent`), `SubagentBuilder` (optional `model` override with session fallback), error types, seatbelt-aware helpers, and derive/attribute macros (`#[derive(Subagent)]`, `#[subagent(...)]`) covered by `trybuild` tests.
- [ ] **Implement `TaskContext`** (typed slots, namespaced scratchpads, read/write guards, diagnostic history, `CODEX_DEBUG_SUBAGENTS=1` serialization).
- [ ] **Build agent parser & registry** under `codex-rs/core/src/subagents/`:
  - `parser.rs` to parse YAML frontmatter + Markdown body, validate naming, capture metadata (name, description, tools, model, instructions, scope, path).
  - `registry.rs` to discover `.codex/agents/*.md` (user + project precedence), cache by mtime hash, expose reload hooks, log invalid files for `/agents` display.
  - `mod.rs` to expose public API; ensure project overrides user definitions by name.
- [ ] **Add configuration entries** in `codex-rs/core/src/config.rs` (`subagents.enabled`, `subagents.auto_route`, fallback model/tool defaults) with feature flag defaulting to false.

---

## Phase 2 — Orchestrator, Protocol, and Routing
- [ ] **Extend Conversation Manager** with `spawn_subagent_conversation` that clones base config, injects agent prompt, applies model/tool overrides, and spawns isolated `CodexConversation` instances.
- [ ] **Implement orchestrator module** (`orchestrator.rs`) to run sequential pipelines, manage retries/timeout escalation, emit lifecycle events, pool `TaskContext`, and merge outputs into the main session.
- [ ] **Router logic** (`router.rs`) to handle `/use <agent>` slash command, natural-language mentions (“Use the reviewer agent”), and optional keyword auto-routing (toggle via config).
- [ ] **Protocol updates**: add `EventMsg::SubAgentStarted`, `SubAgentMessage`, `SubAgentCompleted` (include parent submit ID, agent name, chosen model) in Rust and TypeScript bindings; propagate to MCP message processor and exec-mode JSON contracts.
- [ ] **Approval/tool policy wiring**: introduce allowlist enforcement before tool execution; attach agent/model labels to approval prompts; deny disallowed tool access automatically.
- [ ] **Feature flag integration**: gate orchestrator path behind CLI flag `--use-subagents` and config entries; ensure fallback path exercises legacy flow when disabled.

---

## Phase 3 — Core Subagents & Pipeline Helpers
- [ ] **Spec Parser subagent** (`codex-rs/subagents/spec_parser`): prompt templates, schema validation (IDs, acceptance criteria), fixture-based unit tests producing `RequirementsSpec` snapshots.
- [ ] **Code Writer subagent**: integrate repo context adapters (file summaries, diff utilities), generate `ProposedChanges`, run dry-run formatters (`just fmt`, `cargo fmt`) to verify formatting, produce rationale notes.
- [ ] **Tester subagent**: synthesize `TestPlan`, execute via sandbox-safe harness, capture pass/fail/error states, and provide fallback messaging when execution blocked.
- [ ] **Reviewer subagent**: run style/security heuristics, incorporate lint tooling/LLM prompts, emit `ReviewFindings` with severity taxonomy.
- [ ] **Shared helpers**: provide pipeline utilities to transform `RequirementsSpec → ProposedChanges → TestResults → ReviewFindings` and ensure model overrides flow `SubagentSpec → SubagentBuilder → Orchestrator`.

---

## Phase 4 — UX, CLI/TUI, and External Interfaces
- [ ] **Slash commands & CLI**: add `/agents`, `/use`, `/subagent-status` (update `codex-rs/tui/src/slash_command.rs`), CLI shortcuts (`codex --list-subagents`, `codex exec --subagent <name>`), and textual fallbacks for headless mode.
- [ ] **TUI rendering**: create agents command view listing specs (source, tools, model, parse errors); nest subagent transcripts in history (`.cyan()`/`.dim()` styling), add status pane counters, and label approvals with “Requested by <agent> (model: …)”.
- [ ] **Diff tracker & apply-patch**: annotate diffs with originating agent for audits/conflict detection and enforce sequential edits (queue warnings on conflicts).
- [ ] **MCP/notifications**: expose `subagents/list` + `subagents/run` tools, propagate new events in notifications, update docs (`docs/config.md`, onboarding) with configuration and UX instructions.
- [ ] **Structured telemetry**: emit start/completion events (`subagent`, `model`, `duration_ms`, `outcome`, retry counts) and ensure CLI inspection surfaces recent runs.

---

## Phase 5 — Testing, Hardening, and Demo Prep
- [ ] **Unit tests** for parser precedence/caching, registry reload, tool policy enforcement, orchestrator lifecycle (mocked `Codex::spawn` with custom models), macro expansion.
- [ ] **Integration tests**: fixture repo run across full pipeline, TUI snapshot for `/agents`, CLI acceptance for `codex exec --subagent`, telemetry assertions.
- [ ] **Sandbox & approval audits**: verify allowlists, ensure sandbox env (`CODEX_SANDBOX_*`) respected, confirm approval policies align with instructions.
- [ ] **Manual demo script**: rehearse code-reviewer.md scenario (commands, approvals, transcript review) and capture recordings/screens for documentation.
- [ ] **Telemetry dashboards**: build charts for success/failure rates, average latency, per-model usage.

---

## Phase 6 — Packaging & Release Enablement
- [ ] **Pipeline alignment**: ensure `pnpm`/`just` workflows build Rust artifacts before npm packaging; include subagent assets in distribution.
- [ ] **API exposure**: surface subagent metadata/commands through npm TypeScript API where relevant.
- [ ] **Reproducible builds**: validate binaries across macOS/Linux/Windows with subagent feature toggled on/off.
- [ ] **Documentation & release notes**: publish `docs/subagents.md`, update README, provide migration guidance, record demo assets.
- [ ] **Rollout ladder**: plan alpha → beta → GA toggles with verification checkpoints and telemetry gates.

---

## Rollout Management
- [ ] Maintain feature flag (`subagents.enabled`) default false; enable internally post-regression tests.
- [ ] Gather pilot feedback, iterate on heuristics/UI clarity, track issues in dedicated log.
- [ ] Flip default after stability confirmed; announce in release notes and update onboarding content.

---

## Risk & Mitigation Checklist
- Sequential execution only (queue delegations, warn on overlap).
- Graceful fallback on spawn failure (surface message, resume main agent).
- No sandbox env modifications beyond documented helpers.
- Tool allowlists enforced server-side before command execution.
- Clear labeling of agent name/model across UI/CLI to avoid user confusion.

---

## Post-v2 Backlog & Future Hooks
- Parallel subagent execution with dependency graph resolver.
- Hot reload/watch mode for subagent development.
- Caching/profiling layers configurable per agent.
- Marketplace or registry for community agents with versioning and approval workflow.
- Extended model configuration in `~/.codex/config.toml` (user-defined providers, validation, fallback logic).
- Human-in-the-loop checkpoints between subagent stages.

