
<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install codex</code></p>

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
</br>
</br>If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE</a>
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a></p>

<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
  </p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager. If you use npm:

```shell
npm install -g @openai/codex
```

Alternatively, if you use Homebrew:

```shell
brew install codex
```

Then simply run `codex` to get started:

```shell
codex
```

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

<p align="center">
  <img src="./.github/codex-cli-login.png" alt="Codex CLI login" width="80%" />
  </p>

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](./docs/authentication.md#usage-based-billing-alternative-use-an-openai-api-key). If you previously used an API key for usage-based billing, see the [migration steps](./docs/authentication.md#migrating-from-usage-based-billing-api-key). If you're having trouble with login, please comment on [this issue](https://github.com/openai/codex/issues/1243).

### Model Context Protocol (MCP)

Codex CLI supports [MCP servers](./docs/advanced.md#model-context-protocol-mcp). Enable by adding an `mcp_servers` section to your `~/.codex/config.toml`.


### Configuration

Codex CLI supports a rich set of configuration options, with preferences stored in `~/.codex/config.toml`. For full configuration options, see [Configuration](./docs/config.md).

---

### Docs & FAQ

- [**Getting started**](./docs/getting-started.md)
  - [CLI usage](./docs/getting-started.md#cli-usage)
  - [Running with a prompt as input](./docs/getting-started.md#running-with-a-prompt-as-input)
  - [Example prompts](./docs/getting-started.md#example-prompts)
  - [Memory with AGENTS.md](./docs/getting-started.md#memory-with-agentsmd)
  - [Configuration](./docs/config.md)
- [**Sandbox & approvals**](./docs/sandbox.md)
- [**Authentication**](./docs/authentication.md)
  - [Auth methods](./docs/authentication.md#forcing-a-specific-auth-method-advanced)
  - [Login on a "Headless" machine](./docs/authentication.md#connecting-on-a-headless-machine)
- [**Advanced**](./docs/advanced.md)
  - [Non-interactive / CI mode](./docs/advanced.md#non-interactive--ci-mode)
  - [Tracing / verbose logging](./docs/advanced.md#tracing--verbose-logging)
  - [Model Context Protocol (MCP)](./docs/advanced.md#model-context-protocol-mcp)
- [**Zero data retention (ZDR)**](./docs/zdr.md)
- [**Contributing**](./docs/contributing.md)
- [**Install & build**](./docs/install.md)
  - [System Requirements](./docs/install.md#system-requirements)
  - [DotSlash](./docs/install.md#dotslash)
  - [Build from source](./docs/install.md#build-from-source)
- [**FAQ**](./docs/faq.md)
- [**Open source fund**](./docs/open-source-fund.md)

---

## Subagents (Design Overview)

The Subagents feature lets Codex delegate specific tasks to lightweight, user‑defined agents described in Markdown files with YAML frontmatter. It is feature‑flagged and opt‑in by default so existing workflows remain unchanged.

Key points below are chosen for simplicity, clarity, and ease of iteration.

### Enablement & Discovery
- Feature flag: `subagents.enabled = false` by default (can be overridden by `CODEX_SUBAGENTS_ENABLED=1`).
- Auto‑routing: `subagents.auto_route = false` by default. When true, simple keyword heuristics may trigger a subagent automatically.
- Discovery locations (project overrides user by name):
  - Project: `<repo>/.codex/agents/*.md`
  - User: `~/.codex/agents/*.md`
- Registry caches by mtime hash and reloads when you run `/agents` (TUI) or `codex subagents list` (CLI).

### Agent Spec Format
Agent files are Markdown with YAML frontmatter. Minimal example:

```markdown
---
name: reviewer
description: Reviews diffs and flags style/security issues
model: gpt-5-codex   # optional; falls back to session model if omitted
tools: [apply_patch, local_shell]  # allowlist; empty means no tools
keywords: [review, code review, lints]  # optional; used by auto_route heuristics
---

Detailed instructions for the reviewer agent go here in Markdown.
```

To bind a subagent to a different provider—such as a local Ollama instance exposed through the built-in `oss` provider—use the structured `model_config` block:

```markdown
---
name: code-reviewer-ollama
description: Reviews diffs using a local gpt-oss model
model_config:
  provider: oss
  model: gpt-oss:20b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.0
tools: [apply_patch]
---

You run entirely against the local Ollama server. Flag risky changes and suggest fixes.
```

Fields:
- `name` (string, required): unique per registry; project definitions override user ones when names collide.
- `description` (string): short summary for listings.
- `model` (string, optional): per‑agent model override. If invalid or missing, the session model is used.
- `model_config` (object, optional): structured provider binding for advanced scenarios. Supports `provider` (e.g., `oss` for Ollama), `model`, `endpoint` overrides, and a free-form `parameters` map. The legacy `model` string is treated as shorthand for `model_config.model`.
- `tools` (string[]; optional): strict allowlist of built‑in tools (e.g., `local_shell`, `apply_patch`, `view_image`). MCP tools can be referenced later using `mcp:<server>:<tool>`; initial implementation may focus on built‑ins for simplicity.
- `keywords` (string[]; optional): phrases used by simple NL routing when `subagents.auto_route=true`.

### Orchestration & Routing
- Subagents run sequentially. The orchestrator creates an isolated sub‑conversation that inherits parent policies and cwd:
  - Inherit: `approval_policy`, `sandbox_policy`, `cwd`.
  - Override allowed: `model` (from agent spec) and tool allowlist (from agent spec).
- Routing options:
  - Slash: `/use <agent>` runs a named agent explicitly.
  - Auto‑route (optional): string/keyword heuristics (e.g., "use the <name> agent" or presence of `keywords`) when `subagents.auto_route=true`. No model‑assisted intent in the MVP.

### Protocol Events (new)
Three new events are introduced and streamed like other `EventMsg` items:
- `SubAgentStarted { agent_name, parent_submit_id, sub_conversation_id, model }`
- `SubAgentMessage { agent_name, message }` (mirrors `AgentMessageEvent` but labels the agent)
- `SubAgentCompleted { agent_name, outcome }` where `outcome` is `"success"` or `"error"`.

Diff attribution: existing patch events gain optional origin metadata so UIs can label changes:
- `PatchApplyBegin { ..., origin_agent?: string, sub_conversation_id?: string }`
- `PatchApplyEnd { ..., origin_agent?: string, sub_conversation_id?: string }`

These fields are optional to preserve backwards compatibility.

### CLI/TUI Surface
- TUI slash commands:
  - `/agents` → list available subagents and parse errors
  - `/use <name>` → run a specific subagent
  - `/subagent-status` → show recent runs with status and durations
- CLI subcommands:
  - `codex subagents list` → prints discovered agents with source, model, and tools
  - `codex subagents run <name> [-- prompt...]` → runs a specific subagent non‑interactively

Notes:
- `subagents run` streams standard Codex events; it returns immediately with `{ subConversationId }` and progress arrives via notifications.
- The existing `codex exec` remains unchanged; a `--subagent <name>` flag may be added later if desired.

### MCP Methods (optional, simple shape)
- `subagents/list` → returns: `{ agents: Array<{ name, description?, model?, tools, source, parse_errors? }> }`
- `subagents/run` → params: `{ conversationId, agentName, prompt? }` → result: `{ subConversationId }`, with progress via `codex/event` notifications using the new `SubAgent*` events.

### Tool Policy & Safety
- Enforcement is server‑side and strict: only tools in the agent `tools` allowlist are available during that subagent’s run. Unlisted tools (including local shell) are denied.
- No sandbox changes beyond existing policies. Subagents inherit the parent `approval_policy` and `sandbox_policy` as‑is.

### Telemetry
- Piggyback on existing token usage and event stream: include agent labels in `SubAgentStarted/Completed` and compute per‑agent durations in the CLI/TUI. No new telemetry subsystem is introduced.

### Macros & Crates
- Derive macros live in a sibling proc‑macro crate to avoid cycles: `codex-subagents-derive` (e.g., `#[derive(Subagent)]`, `#[subagent(name = "...", model = "...")]`). The core implementation lives in `codex-subagents`.

### Rollout & History
- Subagent events are recorded in the rollout log with their `sub_conversation_id`. Resuming or forking preserves the parent transcript and labels sub‑segments clearly in UIs.

### Errors & UX Messages
- If subagents are disabled, surface a clear hint: "Subagents are disabled. Enable with `subagents.enabled = true` or set `CODEX_SUBAGENTS_ENABLED=1`."
- If an agent is not found or has parse errors, `/agents` and `subagents list` display diagnostics and the orchestrator skips execution gracefully.

This design keeps the MVP minimal while providing clear extension points (parallel execution, richer routing, MCP tool addressing, and advanced policies) without affecting current behavior.

## License

This repository is licensed under the [Apache-2.0 License](LICENSE).
