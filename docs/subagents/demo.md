# Subagents Demo Walkthrough

This run-book provides a reproducible script for showcasing the delegated subagent workflow end to end. It assumes a macOS or Linux environment with the Codex CLI/TUI built from this workspace.

## 1. Prepare the environment

1. Ensure the feature flag is enabled either via CLI overrides or configuration:
   ```bash
   export CODEX_SUBAGENTS_ENABLED=1
   ```
2. Create a demo workspace with the sample repository you want to review.
3. Add a project-level agent definition at `<repo>/.codex/agents/code-reviewer.md`:
   ```markdown
   ---
   name: code-reviewer
   description: Performs focused code review for diffs touching Rust modules
   model: gpt-5-codex
   tools: ["apply_patch", "git_diff", "git_status", "plan"]
   keywords: ["review", "audit"]
   ---
   You are an expert Rust reviewer. Analyse the proposed change, highlight risky behaviour,
   and suggest actionable follow-ups. Focus on safety, telemetry, and ergonomics.
   ```
4. Optionally add a user-level override in `~/.codex/agents` to demonstrate precedence.

## 2. CLI dry run

1. List agents to show discovery, precedence, and warnings:
   ```bash
   codex subagents list -c subagents.enabled=true
   ```
   Expect the output to highlight the `code-reviewer` agent (source: project), the configured model, tools, keywords, and any parse errors.
2. Attempt to run the agent with the feature flag disabled to demonstrate the guardrail:
   ```bash
   codex subagents run code-reviewer --prompt "Review the latest diff"
   ```
   The command fails with `Subagents feature is disabled...`.
3. Re-run with the flag enabled to stream lifecycle events. You should capture:
   - `started` message with the resolved model
   - streamed messages from the reviewer subagent
   - final success message including the formatted duration line (e.g., `completed in 12.4s`).

## 3. TUI walkthrough

1. Launch the TUI against the same workspace with the feature flag set. The `/agents` slash command should render:
   - Project/user agent counts
   - Source, model, tool allowlist, keywords, and parse warnings
2. Trigger a review request (e.g., paste `Review the changes under src/` and send).
3. Use `/use code-reviewer` to delegate the turn. Observe:
   - History entries labelled `subagent <name> started/completed`
   - Nested transcript lines (`↳`) streaming from the subagent
   - Status header showing `Subagents: 0 active • 1 done • 0 failed`
   - Duration line in the completion card (`duration: 1.2s`) and the status overlay (`elapsed` for active runs)
4. When the subagent requests approvals (e.g., `apply_patch`), verify the modal header reads `Requested by code-reviewer (model: gpt-5-codex)`.

## 4. Capture artefacts

- Record a short terminal session showing the CLI lifecycle events ending with the duration summary.
- Capture TUI screenshots of `/agents`, the nested transcript, and the approval overlay.
- Save the generated `code-reviewer.md` spec and the resulting review snippet for inclusion in release notes.

## 5. Cleanup checklist

- Unset `CODEX_SUBAGENTS_ENABLED` to confirm legacy behaviour is unchanged.
- Remove the demo agent from `.codex/agents` if it should not ship with the repository.
- Copy the captured transcript snippets into `docs/subagents/demo.md` when preparing release notes.
