# codex-cv Subagents Initiative — Implementation Tracker

## 🎯 Objective

Deliver a demo-ready subagent framework for codex-cv that balances fast validation with long-term maintainability, developer ergonomics, and safe rollout—while preparing the npm `codex-cv` package for distribution.

## Phase 0 · Alignment & Planning

- [x] Record final architecture ADR (demo-first spine + typed pipeline + safety guardrails + cross-surface requirements). See `docs/subagents/adr-0001-subagents.md`.
- [x] Define success metrics (demo checklist, DX onboarding time, safety gate coverage). See `docs/subagents/success-metrics.md`.
- [x] Confirm feature flag + rollout plan with stakeholders (initial internal preview → beta → GA). Plan captured in `docs/subagents/rollout-plan.md`.

## Phase 1 · Demo Backbone (v3 emphasis)

- [ ] Create `codex-rs/core/src/subagents/` module (registry, router, orchestrator, parser)
- [x] Implement YAML+Markdown agent loader with project/user precedence and cache invalidation (see `codex-rs/subagents/src/parser.rs` and `codex-rs/subagents/src/registry.rs`).
- [x] Extend `ConversationManager` with `spawn_subagent_conversation` (sequential runs only). Implemented in `codex-rs/core/src/conversation_manager.rs`.
- [ ] Add protocol events (`SubAgentStarted`, `SubAgentMessage`, `SubAgentCompleted`) and server routing
- [ ] Implement `/agents`, `/use`, `/subagent-status` flows in TUI + CLI shortcuts (`codex exec --subagent`)
- [ ] Produce scripted demo scenario (code-reviewer flow) with fixtures and manual walkthrough doc
- [ ] Snapshot/unit tests: parser precedence, router keyword match, orchestrator lifecycle

## Phase 2 · Developer Ergonomics (as_v_2 emphasis)

- [ ] Stand up `codex-subagents` crate with `TaskContext`, typed payload structs, error types
- [ ] Implement derive/attribute macros (`#[subagent]`, `TypedSubagent`, `ContextualSubagent`)
- [ ] Build sequential orchestrator pipeline using typed contracts (`RequirementsSpec → ProposedChanges → …`)
- [ ] Expose builder API (`SubagentBuilder`) for advanced tuning (timeouts, model override, retries)
- [ ] Add scaffolding command `codex subagent new <name>` with templates + baseline tests
- [ ] Integrate structured logging/metrics hooks and CLI inspection commands
- [ ] Write contributor guide + examples under `docs/subagents/`

## Phase 3 · Safety & Governance (v1 emphasis)

- [ ] Implement tool permission model (allowlist/inherit) enforced server-side pre-execution
- [ ] Wire per-agent sandbox + approval policy overrides, respecting global session constraints
- [ ] Add feature flag (`CODEX_CV_SUBAGENTS_ENABLED`) + config plumbing for gradual rollout
- [ ] Introduce telemetry/observability (structured events, metrics shipping, debug env toggles)
- [ ] Plan caching/preload roadmap (defer heavy optimizations, document follow-up stories)

## Phase 4 · Cross-Surface Integration (v2 emphasis)

- [ ] Update protocol bindings (`codex_protocol`, TypeScript `protocol-ts`) and regenerate clients
- [ ] Ensure MCP server exposes `subagents/list` + `subagents/run` tools with new payloads
- [ ] Extend rollout/session persistence to record nested subagent runs (read-only resume)
- [ ] Audit diff tracker (`turn_diff_tracker`) and apply-patch pipeline for subagent provenance tags
- [ ] Harden concurrency story (sequential enforcement, queued edits, conflict warnings)
- [ ] Refresh CLI/TUI help, slash command docs, onboarding copy

## Phase 5 · Packaging & Release Readiness

- [ ] Update build scripts (`pnpm`, `just`) to compile Rust targets prior to npm bundle creation
- [ ] Expose subagent metadata + `/agents` workflows via npm-facing TypeScript API surface
- [ ] Ensure reproducible binaries for macOS/Linux/Windows in `codex-cv` package pipeline
- [ ] Draft release notes + migration guide (agent files layout, feature flag usage, safety notes)
- [ ] Prepare staged npm release plan (alpha tag → beta → latest) with GitHub distribution steps

## Future Feature Placeholder · Expanded Model Configuration

- [ ] Design config extension allowing `~/.codex/config.toml` to declare additional subagent-eligible models (integrates with `model_providers.*`)
- [ ] Spec UI/CLI affordances for selecting configured models when creating/editing agents
- [ ] Align validation + fallback behavior for unsupported providers (defer implementation to post-MVP)

---

**Status Legend**: unchecked = pending · checked = complete. Update after each work session to maintain shared situational awareness.
