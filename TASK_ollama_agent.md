# TASK: Enhanced Model Configuration for Subagents

## Objective
Add optional model configuration to subagents so each agent can opt into a specific model/provider (Ollama, OpenAI, Anthropic, custom) while preserving current behaviour for agents that only set `model`.

## Summary
- Treat this as an incremental enhancement: reuse existing provider abstractions (`ModelProviderInfo`, `Config`) and the established subagent pipeline.
- Keep the simple `model: "gpt-4"` syntax working; the new `model_config` block is an additive, structured alternative.
- Limit scope to parser + config plumbing, light UX feedback, docs, and tests. CLI tooling, TUI dashboards, telemetry, and advanced lifecycle management remain future work.

## Key Assumptions
- The `codex-rs/ollama` crate already provides the HTTP client; no new client module needed.
- Provider metadata continues to flow through `Config`/`ModelProviderInfo`; do **not** introduce a new global provider registry or trait unless unavoidable.
- Changes should not require a feature flag or substantial refactor of conversation orchestration.

---

## Step 1 · Schema & Parsing
- [x] Extend `codex-rs/subagents/src/parser.rs` + `spec.rs` to accept either `model` *(string)* or a new structured `model_config` block.
  - [x] Add `ModelBinding` on `SubagentMetadata` with: `provider_id`, optional `model`, optional `endpoint`, optional map of `parameters`.
  - [x] Preserve shorthand semantics (`model` populates both `model` field and `ModelBinding` when present).
  - [x] Update hashing/validation in `SubagentBuilder` so cache keys change when bindings change.
- [x] Add parser tests covering: legacy string-only, structured config, invalid/empty provider input, and endpoint overrides.

## Step 2 · Config Application Path
- [x] In `ConversationManager::spawn_subagent_conversation`, recognise `ModelBinding`:
  - [x] Override `child_config.model_provider_id` and clone the referenced `ModelProviderInfo`.
  - [x] When an endpoint override is provided, synthesise a provider entry in `child_config.model_providers` before spawn.
  - [x] Continue falling back to parent defaults with a warning if the binding is missing/invalid (no hard failure).
- [x] Unit test ensuring `apply_model_binding` updates provider + model overrides.

## Step 3 · Validation & Fallbacks
- [x] Centralise validation during parsing: enforce non-empty strings, reconcile conflicting model declarations, and trim parameter keys.
- [x] Bubble actionable error messages (invalid provider/endpoint, conflicting models, empty parameter keys).
- [x] Add tests for each validation path.

## Step 4 · CLI & UX Touch-ups
- [x] Update `codex subagents list` output to show resolved provider/model when available.
- [x] Enhance `SubagentStarted` logging in `subagents_cmd.rs` to display `<provider>/<model>` (with endpoint when present).
- [x] Defer new CLI commands (`codex subagents models …`) until after core support lands.

## Step 5 · Documentation & Examples
- [x] Refresh `docs/subagents/spec.md` with the new frontmatter schema, backwards-compatibility notes, and provider referencing guidance.
- [x] Add an example agent (`docs/subagents/examples/code-reviewer-ollama.md`) showing an Ollama binding that points at the built-in `oss` provider.
- [x] Mention how to declare custom providers via `~/.codex/config.toml` and reuse them in `model_config`.

## Step 6 · Testing & Follow-up Tracking
- [x] Extend parser + conversation tests (unit + integration) to cover new flows.
- [x] Confirm existing providers cover the deterministic test path (no additional mock provider required).
- [ ] Create follow-up tickets for: advanced CLI management, TUI status widgets, adaptive fallback, telemetry, model caching/preloading.

---

## Out of Scope (Later Phases)
- Expanded CLI commands for managing models/providers.
- TUI status/monitoring widgets.
- Adaptive fallback heuristics, telemetry dashboards, and performance cost modelling.
- Model lifecycle management (caching, warming, pull orchestration refinements).

---

## Success Criteria
- Agents can opt into Ollama or other providers via frontmatter without breaking existing agents.
- Provider overrides flow through to the spawned conversation config and are visible in CLI feedback.
- Validation prevents misconfigured agents and guides users toward a fix.
- Documentation clearly explains how to adopt the new schema.
