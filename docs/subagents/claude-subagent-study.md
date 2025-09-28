# Claude Subagent Behavior — Key Findings

Date: September 27, 2025

## Observed Workflow

- Subagents are defined in Anthropic's Claude project as named personas with Markdown instructions and optional tool bindings.
- Users invoke subagents explicitly (`/subagent <name>`) or via keyword triggers; Claude reflects the active persona in the transcript header.
- Execution is sequential. Requests queue and the root conversation regains control only after the subagent finishes.
- Tool access is constrained per persona. Attempts to call disallowed tools fail fast with user-facing error cards.

## Implications for codex-cv

1. **Registry precedence** — Claude favors workspace definitions over global defaults, mirroring our project-over-user override requirement.
2. **Context isolation** — Each subagent run resets scratch context while still inheriting the overarching conversation history relevant to the task; we must serialize a `TaskContext` with typed slots and diagnostic history.
3. **Model overrides** — Claude allows personas to pin a specific model. We will treat an invalid override as a warning and fall back to the session model, logging telemetry for visibility.
4. **Tool prompts** — Approval dialogs display both the persona name and active model. Provide similar labels so humans can audit delegated actions.
5. **Lifecycle surfacing** — Subagent state transitions are visible in the UI (starting, streaming, completed). We will emit the corresponding protocol events and render them in TUI/CLI views.

## Risks to Address

- **Error recovery**: Claude lets subagents raise errors without derailing the main chat. Our orchestrator must surface failures as structured events and rejoin the parent session gracefully.
- **Sandbox awareness**: Claude's hosted environment dodges local sandbox limitations. We must respect `CODEX_SANDBOX` flags when executing commands or tests.
- **Telemetry volume**: Persona-level metrics can explode cardinality. We will bucket durations and emit only necessary identifiers (agent name, model, conversation ids).

## Action Items

- Mirror persona labeling in approval prompts.
- Cache registry entries by mtime to avoid re-parsing on every slash command.
- Plan for keyword auto-routing guarded by `subagents.auto_route` to avoid surprising users.
