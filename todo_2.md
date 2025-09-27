 ———

  Purpose

  Deliver a hackathon-ready, working demo of Claude-style sub-
  agents inside Codex. The prototype must showcase:

  - Definition of sub-agents via Markdown + YAML (project/user
  precedence).
  - Launching a delegated Codex conversation with isolated
  context, model/tool overrides.
  - Sub-agent execution traced in the TUI/CLI with clear UX
  affordances.
  - Reintegration of sub-agent output into the main conversation.

  The implementation should reflect OpenAI engineering standards:
  incremental commits, tests, just fmt, scoped just fix -p, and
  respectful handling of sandbox environment variables.

  ———

  1. Evaluate Claude Sub-Agent Behavior

  - [ ] Capture core traits from Claude docs (already reviewed):
      - Each sub-agent has name, description, tool allowlist,
  model override, dedicated prompt, separate context window.
      - Stored at .claude/agents/*.md (project precedence) and
  ~/.claude/agents/*.md.
      - Managed via /agents command; delegates auto or explicit
  request (e.g., “Use the debugger subagent…”).
      - Main agent waits for sub-agent completion, then merges
  result.
  - [ ] Distill practical implications for Codex:
      - Need registry loader for .codex/agents/*.md.
      - Need orchestrator capable of spawning additional
  CodexConversations.
      - Need UX for listing/invoking agents and reflecting
  delegated work.

  ———

  2. Architecture & Codebase Preparation

  - [ ] Understand current Codex construction:
      - codex-rs/core/src/conversation_manager.rs – single
  conversation lifecycle.
      - codex-rs/core/src/project_doc.rs – AGENTS.md layering (for
  instruction reuse).
      - codex-rs/tui/src/slash_command.rs – slash-command
  enumeration.
      - codex-rs/core/src/protocol/ – event definitions.
      - codex-rs/mcp-server/src/ – external protocol bridging.
  - [ ] Decide feature gating:
      - Add subagents.enabled boolean (default false) and
  subagents.auto_route boolean to Config.
  - [ ] Create new module structure:
      - codex-rs/core/src/subagents/
          - mod.rs – public API.
          - registry.rs – discovery + caching.
          - parser.rs – Markdown/YAML parsing.
          - router.rs – match logic for explicit/heuristic
  delegation.
          - orchestrator.rs – spawn & manage sub-agent
  conversations.

  ———

  3. Implement Sub-Agent Registry

  - [ ] Define SubAgentSpec struct capturing:
      - name, description, tools: Option<Vec<String>>, model:
  Option<String>, instructions: String, scope (project/user),
  path.
  - [ ] Parser tasks:
      - Use serde_yaml to parse YAML frontmatter.
      - Accept Markdown body post-frontmatter as system prompt.
      - Validate naming (lowercase + hyphen), required fields.
  - [ ] Discovery logic:
      - Search ~/.codex/agents/ (user scope) and <repo>/.codex/
  agents/ (project scope).
      - Precedence: project overrides user by name.
      - Cache results keyed by (path, mtime).
      - Provide reload_if_stale() to refresh.
  - [ ] Errors:
      - Log invalid files; surface in TUI /agents view.

  ———

  4. Conversation & Orchestration Layer

  - [ ] Extend ConversationManager:
      - Add spawn_subagent_conversation(spec: &SubAgentSpec,
  parent: &CodexConversation) -> CodexResult<SubConversation>:
          - Clone base Config, inject sub-agent prompt
  (config.user_instructions replacement?).
          - Apply model override if present.
          - Provide allowed tool list to later enforcement.
          - Call Codex::spawn to create isolated
  CodexConversation.
  - [ ] Orchestrator:
      - Maintain mapping Vec<ActiveSubAgent> containing
  conversation_id, agent_name, parent_submit_id.
      - Provide run_subagent(agent_name, task_prompt,
  history_slice) that:
          - Emits SubAgentStarted.
          - Streams user prompt to spawned conversation.
          - Waits for EventMsg::TaskComplete (or equivalent).
          - Returns final message summary + recorded actions.
          - Emits SubAgentCompleted.
  - [ ] Router:
      - Recognize explicit /use <agent> ... command.
      - Recognize explicit textual mention Use the <agent_name>
  agent.
      - Optionally stub auto-routing with keyword match on
  description (but allow toggling off).

  ———

  5. Protocol & Event Surface

  - [ ] Update codex-rs/core/src/protocol/mod.rs and codex-rs/protocol/src/...:
      - New enum variants:
          - EventMsg::SubAgentStarted { parent_submit_id, agent_name }
          - EventMsg::SubAgentMessage { agent_name, content_delta } (optional for
  streaming)
          - EventMsg::SubAgentCompleted { agent_name, summary }
      - Ensure serialization updates propagate to TypeScript protocol-ts.
  - [ ] Modify codex-rs/core/src/tasks/regular.rs (and compact.rs) to:
      - Recognize when events belong to sub-agent.
      - Tag approvals with agent_name.
      - Apply tool allowlist check before presenting approval prompt; deny automatically
  if blocked.
  - [ ] turn_diff_tracker.rs:
      - Record diff events with origin_agent metadata for auditing.
  - [ ] Update documentation references in docs/config.md, docs/advanced.md summarizing new
  events/config.

  ———

  6. Tool Permission Enforcement

  - [ ] Introduce ToolPolicy in subagents/orchestrator.rs or shared module:
      - Build from spec.
      - Provide fn can_use(tool_id: &str) -> bool.
      - When main spool receives ToolCall, check if agent is sub-agent and enforce
  allowlist before scheduling.
  - [ ] Ensure inherit default results in None (meaning full access) to minimize initial
  friction.

  ———

  7. TUI & CLI Enhancements

  - [ ] codex-rs/tui/src/slash_command.rs:
      - Add commands: Agents, Use, optionally SubagentStatus.
      - Provide descriptions per docs.
  - [ ] codex-rs/tui/src/commands/agents.rs (new):
      - Display table of available specs (source, description, tools, model).
      - Show parse errors.
  - [ ] Composer view updates:
      - history_cell.rs: render nested block for sub-agent conversation with indentation +
  Stylize color (e.g., .cyan()). Show summary when collapsed.
      - status/helpers.rs: include active sub-agent count.
  - [ ] Approval UI:
      - In codex-rs/tui/src/approvals/, add label Requested by <agent-name>.
  - [ ] CLI:
      - codex-cli: add codex --list-subagents, codex --use-subagent <name> "<prompt>".
      - codex exec: new flag --subagent.
  - [ ] Provide textual fallback when TUI disabled (headless mode): log start/completion
  events with clear formatting.

  ———

  8. External Interfaces & Telemetry

  - [ ] codex-rs/mcp-server/src/codex_message_processor.rs:
      - Extend tools/list to include subagents/list metadata.
      - Accept new subagents/run tool with payload { agent, prompt, optional context }.
      - Forward new event payloads in codex/event.
  - [ ] Notifications:
      - Update docs/config.md to mention tui.notifications and notify accepting subagent-
  started, subagent-completed.
  - [ ] Telemetry:
      - Add structured logging (via tracing) for start/complete + duration.

  ———

  9. Tests & Validation

  - [ ] Unit Tests:
      - Parser: valid/invalid YAML, precedence, caching.
      - Registry: reload when file changed.
      - ToolPolicy: enforcement tests.
      - Orchestrator: spawn stub Codex::spawn (likely using mock or feature gating) to
  ensure right config injection.
  - [ ] Integration Tests:
      - TUI snapshot tests for /agents list and sub-agent transcript (skip under seatbelt
  as required).
      - CLI acceptance: cargo test -p codex-cli --test subagent_exec (scoped).
  - [ ] Manual Demo Checklist:
      1. Create user-level agent ~/.codex/agents/code-reviewer.md.
      2. Launch codex with sample repo.
      3. Run /agents to show listing.
      4. Invoke /use code-reviewer Review src/lib.rs.
      5. Observe sub-agent start/completion events; show result summary in transcript.
      6. Approve sample read-only command (if any) with label showing agent.
      7. Exit; demonstrate codex --list-subagents.
  - [ ] Ensure just fmt and just fix -p codex-core / codex-tui run clean.

  ———

  10. Documentation & Enablement

  - [ ] Add docs/subagents.md summarizing usage, file format, and demo steps.
  - [ ] Update README.md (Codex root) with short “Sub-agents (experimental)” blurb
  referencing new doc.
  - [ ] Provide sample agent files under examples/subagents/.
  - [ ] Note feature gate (subagents.enabled) and default off state.

  ———

  11. Risk Mitigation

  - Sequential execution only – orchestrator should queue delegations; log warning if user
  attempts parallel sub-agents.
  - Fallback plan: if sub-agent spawn fails, surface message and continue with main agent.
  - Keep existing behavior unchanged when feature disabled; guard code paths with config
  checks.
  - Avoid modifications to sandbox environment support variables per instructions.

  ———

  12. Demo Prep

  - [ ] Prepare scripted walkthrough slides (optional).
  - [ ] Record TUI session to verify clarity.
  - [ ] Gather metrics (number of commands run, approvals) for storytelling.

  ———

  References

  - Claude Sub-agent docs.
  - v_3_change.md prototype design (primary blueprint).
  - OpenAI Codex coding standards from repo instructions (AGENTS.md, formatting/testing
  directives).

  ———
