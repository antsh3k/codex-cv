# codex-cv Subagents Initiative — GPT-5-Codex Optimized Plan

## Mission
Launch a subagent framework that showcases GPT-5-Codex strengths: deep code navigation, adaptive reasoning, collaborative planning, and reliable tool execution—while paving the way for npm distribution of `codex-cv`.

## Phase 0 · Prime Codex for Success
- [ ] Lock architecture ADR framing the hybrid strategy (v3 demo core + as_v2 ergonomics + v1 safety + v2 surface alignment)
- [ ] Identify greppable anchors (modules, structs, commands) for rapid navigation during implementation
- [ ] Enumerate verification targets (just fmt, just fix, targeted cargo tests, npm packaging smoke)
- [ ] Create failure reproduction + validation playbook for the demo scenario

## Phase 1 · Demo Backbone (Fast Iteration)
- [ ] Scaffold `core/src/subagents/` module with registry, router, orchestrator, parser stubs
- [ ] Implement YAML+Markdown loader with change detection (mtime hash) and project/user precedence
- [ ] Extend `ConversationManager::spawn_subagent_conversation` for sequential delegation + transcript tagging
- [ ] Introduce protocol events (`SubAgentStarted|Message|Completed`) and update Rust/TS bindings
- [ ] Wire `/agents`, `/use`, `/subagent-status`, CLI shortcuts, and TUI status panes
- [ ] Script demo flow (code-reviewer.md fixture + step-by-step walkthrough)
- [ ] Add focused tests (parser precedence, router keyword routing, orchestrator lifecycle)

## Phase 2 · Developer Ergonomics & Observability
- [ ] Create `codex-subagents` crate exposing `TaskContext`, typed payloads, and domain-specific errors
- [ ] Implement derive macros (`#[subagent]`, `TypedSubagent`, `ContextualSubagent`) with trybuild coverage
- [ ] Build sequential pipeline helpers (`RequirementsSpec → ProposedChanges → TestResults → ReviewFindings`)
- [ ] Provide `SubagentBuilder` for configuring model overrides, retries, and timeouts
- [ ] Generate scaffolding command `codex subagent new` (templates, starter tests, docs link)
- [ ] Integrate structured logging/metrics (start/complete events, duration, retry counts) with CLI inspection

## Phase 3 · Safety, Tooling, Rollout Guardrails
- [ ] Add tool permission allowlists enforced server-side pre-call plus inheritance defaults
- [ ] Support per-agent approval policy & sandbox overrides honoring global constraints
- [ ] Gate functionality with feature flag `CODEX_CV_SUBAGENTS_ENABLED` + config toggles
- [ ] Emit telemetry hooks for subagent lifecycle and failure diagnostics (toggle via env)
- [ ] Backlog caching/preloading optimizations with acceptance criteria (defer implementation)

## Phase 4 · Cross-Surface Integration & Risk Controls
- [ ] Update MCP server (`subagents/list`, `subagents/run`) and regenerate TS protocol clients
- [ ] Persist subagent provenance in rollout/session logs; ensure turn replay resilience
- [ ] Annotate diff tracker/apply-patch outputs with agent origin for audit + conflict detection
- [ ] Enforce sequential execution with queued edits and user warnings on conflicts
- [ ] Refresh help/docs in CLI/TUI, onboarding content, and slash command usage

## Phase 5 · Packaging & Release Readiness
- [ ] Align `pnpm`/`just` pipelines to build Rust artifacts before npm packing
- [ ] Expose subagent metadata and commands via npm TypeScript API layer
- [ ] Validate reproducible binaries across macOS/Linux/Windows for `codex-cv`
- [ ] Draft release notes, migration guide, and demo assets for GitHub + npm launch
- [ ] Stage release ladder (alpha tag → beta → GA) with verification checkpoints

## Future Hook · Expanded Model Configuration
- [ ] Design `~/.codex/config.toml` extension allowing users to register additional subagent models (ties into `model_providers.*`)
- [ ] Outline CLI/TUI affordances for selecting configured models during agent creation/editing
- [ ] Define validation + fallback logic for unsupported/unauthorized models (implementation deferred post-MVP)

---
Use this tracker to align high-leverage GPT-5-Codex workflows: clear code pointers, lightweight prompting, agent-assisted verification, and incremental delivery.
