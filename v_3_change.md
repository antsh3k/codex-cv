 Design Overview

  - Goal: let Codex delegate work to configurable “sub-agents” (separate CodexConversations with their own prompts/tool scopes) and
  produce a demoable end-to-end flow.
  - Non-goals: efficiency optimizations, full parity with Claude, UI polish beyond basic operability, sandbox/approval model redesign.

  User Workflow

  - Sub-agent definitions live in .codex/agents/*.md (user) and <repo>/.codex/agents/*.md (project); project files override on name
  clash.
  - Run /agents (new slash command) to list/create/edit/delete agents; files follow Markdown + YAML header (name, description, tools,
  model, optional instructions body).
  - Invoke explicitly with /use <agent> <prompt> or by tagging in a message (Use the code-reviewer agent…). Optional auto-routing turns
  on via config flag.

  Architecture & Flow

  - Introduce codex-rs/core/src/subagents module:
      - registry.rs loads YAML headers, caches SubAgentSpec.
      - router.rs matches UserTurn to specs (explicit mention or keyword match on description).
      - orchestrator.rs owns a primary CodexConversation plus a map of spawned subagent conversations keyed by (parent_id, agent_name,
  run_id).
  - ConversationManager (codex-rs/core/src/conversation_manager.rs) grows an spawn_subagent_conversation(spec, parent_ctx) that:
      - Calls Codex::spawn with cloned base config, overrides system prompt, tool caps, model.
      - Registers a transient ConversationId tagged with the parent run.
      - Pipes the subagent transcript/events back through the orchestrator.
  - Execution loop:
      1. Main conversation receives user input.
      2. Router decides: handle inline or delegate.
      3. Delegate path: orchestrator spawns subagent, streams task prompt + limited history, waits for completion, then posts synthesized
  summary into the main transcript (EventMsg::SubAgentResult).
      4. Main agent resumes with merged result.

  Configuration & Storage

  - File schema (YAML frontmatter + Markdown body) parsed in codex-rs/core/src/subagents/parser.rs.
  - Precedence: project > user; duplicate names logged.
  - Registry warm cache keyed by mtime hash; invalid configs surfaced in /agents view and ignored.
  - Config flag subagents.auto_route and subagents.enabled added to CodexConfig (codex-rs/core/src/config.rs).

  Runtime Changes

  - Extend EventMsg (codex-rs/core/src/protocol/mod.rs) with:
      - SubAgentStarted { parent_id, agent_name }
      - SubAgentMessage { agent_name, delta | message }
      - SubAgentCompleted { agent_name, summary }
  - Approval tracker (codex-rs/core/src/tasks/regular.rs) tags requests with agent_name, enforces tool allowlist before prompting user.
  - Diff tracker / apply patch pipeline (codex-rs/core/src/turn_diff_tracker.rs) records subagent origin for audit trail.
  - Rollout persistence (codex-rs/core/src/rollout/mod.rs) stores nested runs to support resume/fork (initially read-only fallback:
  subagent runs are not resumable individually).

  TUI/CLI Updates

  - Add /agents, /use, /subagent-status; modify SlashCommand enum (codex-rs/tui/src/slash_command.rs) and popups.
  - New panes under status view (codex-rs/tui/src/status) showing active subagent runs and registry entries.
  - Transcript renderer (codex-rs/tui/src/history_cell.rs) nests subagent output in collapsible blocks, color-coded via Stylize
  (.cyan(), .dim()).
  - Approval dialogs (codex-rs/tui/src/approvals) include “Requested by <agent>”.
  - CLI codex exec gets --subagent <name> shortcut; codex --list-subagents.

  External Interfaces

  - MCP server (codex-rs/mcp-server/src/codex_message_processor.rs) surfaces subagents/list, subagents/run tools, plus new codex/event
  payload variants above.
  - Notifications config (docs/config.md) documents new types (subagent-started, subagent-completed).
  - Exec mode JSON contract adds optional subagent field on actions to keep compatibility (absence = main agent).

  Testing & Demo Plan

  - Unit tests: parser (invalid YAML, precedence), router matching, orchestrator spawn lifecycle (mocked Codex::spawn).
  - Integration: TUI smoke test (golden snapshot) for /agents list, CLI acceptance for codex exec --subagent.
  - Manual demo:
      1. Create code-reviewer.md.
      2. Run codex.
      3. Type /use code-reviewer Review src/lib.rs.
      4. Show nested transcript and approval prompt labeled with subagent.
      5. Confirm resulting summary in main thread.

  Risks & Mitigations

  - Safety: ensure tool allowlists enforced server-side; default inherit inherits full set, others must be whitelisted.
  - State explosion: keep subagent runs sequential for v1; orchestrator rejects concurrent delegations until main run idle.
  - Complexity: isolate new logic in subagents module; keep existing single-agent path untouched when feature disabled.
  - Protocol churn: gate new event types behind subagents.enabled; fall back to textual summary if consumer ignores them.

  Implementation Targets

  - Core: codex-rs/core/src/conversation_manager.rs, new subagents module, protocol enums.
  - Config/UI: codex-rs/core/src/config.rs, codex-rs/tui/src/* (slash command, status, history, approvals), docs/config.md.
  - Interfaces: codex-rs/mcp-server/src/*, codex-rs/exec/src/*, notification docs.

  This plan prioritizes a functional prototype: sequential delegation, basic UI, and clear API signals so the demo shows Codex spinning
  up a specialist agent and folding the result back into the main conversation.