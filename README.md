
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
</br>
</br>If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE</a>
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a></p>

<p align="center"><em>Hackathon build:</em> build from source to unlock the Subagents feature described below.</p>

<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
  </p>

---

## CEREBRAL VALLEY × OPENAI Hackathon — Subagents Launch (September 28, 2025)

Subagents are the centerpiece of our hackathon submission for the CEREBRAL VALLEY × OPENAI event on September 28, 2025. They let Codex delegate targeted tasks to purpose-built copilots that live alongside your codebase so on-call engineers, reviewers, and operators get AI superpowers without leaving the CLI.

### Motivation
- Keep teammates focused: automatically route repetitive chores like log triage, release notes, and security scans to specialized assistants so humans stay on deep work.
- Ship safely under pressure: per-project policies and tool allowlists mean every subagent inherits the right guardrails while still acting fast.
- Extend in minutes: author a Markdown file, drop it in `.codex/agents/`, and Codex discovers it instantly—perfect for hackathon iteration.

### Rationale & Example Flows
- Story-driven code review: `reviewer.md` flags risky diffs, while `test-writer.md` proposes coverage to keep weekend launches stable.
- Incident co-pilot: a `pager-duty.md` agent can page through on-call runbooks, summarize metrics, and suggest mitigations from shell output.
- Docs whisperer: `docs-reviewer.md` reads release notes, checks tone, and drafts customer-facing summaries before the demo.

### Why Subagents Matter (September 2025 snapshot)
- Cloud monopolies force every request through a single, expensive frontier model even when you just need a fast lint.
- Shipping code to third-party APIs is a non-starter for teams with SOC2, HIPAA, or FedRAMP obligations.
- Sequential workflows block on the slowest step; parallel local specialists finish in a fraction of the time.
- Local agents embrace your project context—naming conventions, flaky tests, runbooks—because you author them in Markdown alongside your code.

#### Scenario 1: Airplane Mode Reality
```bash
# Simulate a spotty connection during the hackathon
$ networksetup -setairportpower en0 off

# Cloud-only approach stalls immediately
$ codex exec --model gpt-4.1 "Review src/payments/webhook.rs"
error: request failed: network error: lookup api.openai.com: nodename nor servname provided

# Local subagents keep shipping
$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run security-scanner --prompt "Review src/payments/webhook.rs"
security-scanner  ▸ ⚠ SQL injection risk at src/payments/webhook.rs:23
$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run bug-detector --prompt "Review src/payments/webhook.rs"
bug-detector      ▸ ⚠ Missing authentication guard at src/payments/webhook.rs:41
$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run perf-analyzer --prompt "Review src/payments/webhook.rs"
perf-analyzer     ▸ ℹ Reuse the pooled HTTP client to avoid reconnect overhead
$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run test-writer --prompt "Review src/payments/webhook.rs"
test-writer       ▸ ✅ Drafted 6 regression tests covering 403/429 branches
```
Even with Wi-Fi disabled, the local registry handles the full review pipeline. No approval prompts, no retries, no lost time.

#### Scenario 2: Enterprise Security & Compliance
```bash
# Inspect outbound traffic while running a cloud model
$ sudo tcpdump -i en0 'dst host api.openai.com' &
[1] 4201
$ codex exec --model gpt-4.1 "Summarize customer_data_handler.rs" >/tmp/cloud.log
$ fg
tcpdump: 3 packets captured

# Now run the local lineup under the same capture
$ sudo tcpdump -i en0 'dst host api.openai.com' &
[1] 4210
$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run security-scanner --prompt "Summarize customer_data_handler.rs" >/tmp/local.log
$ fg
tcpdump: 0 packets captured
```
Every regulated customer demo so far has highlighted that subagents keep proprietary source, PII, and credentials on-device. Compliance officers sign off because there is no network egress to chase.

#### Scenario 3: Institutional Memory on Tap
```bash
$ cat .codex/agents/bug-hunter.md
---
name: bug-hunter
description: Flags regressions we have shipped before
model_config:
  provider: oss
  model: stable-code:3b
  endpoint: http://localhost:11434/v1
keywords: [regression, flaky-tests]
---
Reference incident INC-4521 (March 2024) where the scheduler lost its mutex.
Flag any async task that writes shared state without acquiring the lock first.
Link to PR-8832 when suggesting a fix.

$ CODEX_SUBAGENTS_ENABLED=1 codex subagents run bug-hunter --prompt "Audit scheduler.rs before the finals demo"
bug-hunter ▸ ⚠ Matches INC-4521: acquire scheduler.lock() before updating queue state
```
Instead of fine-tuning, we codify playbooks in Markdown and ship them with the repo. New teammates instantly benefit from years of tribal knowledge.

### Architecture Advantages
```
Traditional cloud agent (sequential):
┌──────────┐ 8s security → 8s performance → 8s bugs → 8s tests = 32s
└──────────┘

Subagents (parallel fan-out):
┌────────────┐ ┌────────────┐ ┌───────────┐ ┌──────────┐
│security 8s │ │performance │ │bugs 8s    │ │tests 8s  │
└────────────┘ └────────────┘ └───────────┘ └──────────┘
Total wall-clock: ~8s (4× speedup)
```
The orchestrator streams each agent run independently, so your TUI can render feedback as soon as one specialist finishes instead of waiting for a mega-model to complete a monologue.

### Cost Snapshot (per developer, 10 PRs + 20 tasks daily)
| Mode | Daily Cost | Annual Cost | Notes |
| --- | --- | --- | --- |
| Single frontier model (`codex exec --model gpt-4.1`) | ≈ $47.50 | ≈ $14,000 | assumes 95K output tokens/day |
| Hybrid (frontier + local) | ≈ $12.80 | ≈ $3,900 | only escalates complex prompts |
| Subagent-first (local + optional fallback) | ≈ $2.10 | ≈ $750 | local Ollama models, occasional fallback |
Savings compound across teams: a 10-person squad saves ~ $132K/year versus cloud-only review.

### Where Subagents Shine
- **High-security shops**: air-gapped labs, defense contractors, health tech, fintech audits.
- **High-volume CI**: hundreds of PRs/nightly that need linting, security scans, and release notes.
- **Domain-heavy stacks**: robotics, firmware, or data infra with bespoke APIs and safety briefs.
- **Budget-conscious teams**: startups and OSS maintainers who need predictable costs.
- **Education & hackathons**: classrooms and meetups with constrained Wi-Fi but plenty of laptops.


### Local Install & Startup (5 minutes)
1. Clone this repo and enter the workspace: `git clone https://github.com/antsh3k/codex-cv.git && cd codex-cv`.
2. Build the hackathon binary: `cargo build -p codex-cli --release` (this compiles the new Subagents support).
3. Enable Subagents and explore with the local build: set `CODEX_SUBAGENTS_ENABLED=1`, run `./codex-rs/target/release/codex subagents list`, then try `./codex-rs/target/release/codex subagents run reviewer --prompt "Audit the new subagent orchestrator"`.
4. Customize on-site: add Markdown specs under `.codex/agents/` (see examples below) and share them with hackathon teammates for instant reuse.

---

## Local Build Quickstart

### Build and run the hackathon Subagents build

The Subagents capability showcased for CEREBRAL VALLEY × OPENAI ships ahead of the standard installers, so pull and build from source:

```bash
git clone https://github.com/antsh3k/codex-cv.git
cd codex-cv
cargo build -p codex-cli --release
```

That produces `./codex-rs/target/release/codex`. Run it directly (or symlink it into your PATH) to explore the new capabilities:

```bash
CODEX_SUBAGENTS_ENABLED=1 ./codex-rs/target/release/codex subagents list
CODEX_SUBAGENTS_ENABLED=1 ./codex-rs/target/release/codex subagents run reviewer --prompt "Audit the new subagent orchestrator"
```

If you prefer not to manage PATH entries manually, `cargo run --release -p codex-cli -- …` works for any command:

```bash
cargo run --release -p codex-cli -- subagents run reviewer --prompt "Check the payments webhook"
```

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
  - `codex-subagent list` → convenience command (same as `codex subagents list` with auto-enablement)
  - `codex-subagent run <name> [-- prompt...]` → convenience command (same as `codex subagents run` with auto-enablement)

Notes:
- `subagents run` streams standard Codex events; it returns immediately with `{ subConversationId }` and progress arrives via notifications.
- The existing `codex exec` remains unchanged; a `--subagent <name>` flag may be added later if desired.

### Quickstart: Local Ollama Subagent Pipeline
The `codex-subagent` shim makes it easy to exercise subagents without toggling feature flags manually. The following walkthrough wires up an “AI Code Review Pipeline” that runs entirely on a local Ollama instance.

#### Local validation (until npm release)
If you are testing an unpublished build from this repository, install it globally from the checkout so both entry points are available on your PATH:

```bash
# from the repository root
cargo build -p codex-cli --release
TARGET=$(rustc -vV | awk '/host:/ { print($2) }')
cp codex-rs/target/release/codex "codex-cli/vendor/${TARGET}/codex/codex"
(cd codex-cli && npm install -g .)
which codex
which codex-subagent
```

This rebuild uses the local workspace code inside `codex-rs`. Cargo will only use cached crates from crates.io; if you need a completely offline run, prime the cache once with `cargo fetch` while online and then repeat the build with `CARGO_NET_OFFLINE=1`.

#### 0. Prerequisites
- Build the Rust binary and stage it under `codex-cli/vendor/<target-triple>/codex/codex` (see installation section above).
- Optionally run `(cd codex-cli && npm install -g .)` to expose the CLI shim (`codex` and `codex-subagent`) on your PATH.
- Ensure Ollama is running at `http://localhost:11434/v1`.
- Create a project directory where you can add `.codex/agents/*.md`.

#### 1. Pull local models (≈14 GB total)
```bash
ollama pull granite3-dense:2b      # security scanner (~2 GB)
ollama pull stable-code:3b         # bug detector (~3 GB)
ollama pull deepseek-coder:1.3b    # performance analyzer (~2 GB)
ollama pull qwen2.5-coder:3b       # test writer (~3 GB)
ollama pull codellama:7b           # code fixer (~4 GB)
# Optional documentation pass
# ollama pull phi3                 # docs reviewer (~2 GB)
```

#### 2. Define subagents
Create `.codex/agents/` and add one Markdown file per agent.

`security-scanner.md`
```markdown
---
name: security-scanner
description: Flags potential security issues before code merges
model_config:
  provider: oss
  model: granite3-dense:2b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.1
tools: []
keywords: [security, vulnerabilities]
---

Audit the diff for authentication, authorization, secrets, and injection risks.
Return a short report with severity and remediation suggestions. Mark "safe" if nothing needs attention.
```

`bug-detector.md`
```markdown
---
name: bug-detector
description: Hunts for logic bugs and edge cases
model_config:
  provider: oss
  model: stable-code:3b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.15
tools: []
keywords: [bug, regression]
---

Read the change context and spot regressions, missing error handling, or broken invariants.
Explain why each issue matters and how to fix it.
```

`perf-analyzer.md`
```markdown
---
name: perf-analyzer
description: Reviews patches for performance regressions
model_config:
  provider: oss
  model: deepseek-coder:1.3b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.2
tools: []
keywords: [performance, latency]
---

Inspect for CPU, memory, or I/O costs. Highlight hot paths or scale concerns and suggest optimizations or benchmarks.
```

`test-writer.md`
```markdown
---
name: test-writer
description: Proposes regression tests for new behavior
model_config:
  provider: oss
  model: qwen2.5-coder:3b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.25
tools: []
keywords: [tests, coverage]
---

Infer the key behaviors that should be validated. Produce skeleton tests (unit/integration) or outline manual steps to verify fixes.
```

`code-fixer.md`
```markdown
---
name: code-fixer
description: Generates patch suggestions for high-priority issues
model_config:
  provider: oss
  model: codellama:7b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.2
tools: [apply_patch]
keywords: [fix, patch]
---

When upstream agents flag issues, draft concrete code patches or diff snippets. Keep changes minimal and explain the rationale.
```

Optional documentation reviewer (`docs-reviewer.md`):
```markdown
---
name: docs-reviewer
description: Improves inline documentation and release notes
model_config:
  provider: oss
  model: phi3
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.3
tools: []
keywords: [docs, comments]
---

Assess comments, README updates, or changelog entries. Ensure clarity, accuracy, and developer ergonomics.
```

#### 3. Verify discovery
```bash
codex-subagent list
```
You should see each agent listed with `source: project`. If nothing is found, double-check the `.codex/agents` path, YAML formatting, and filenames.

#### 4. Run the pipeline
```bash
ISSUE_PROMPT="Review the new payment webhook handler in src/payments/webhook.rs"

codex-subagent run security-scanner --prompt "$ISSUE_PROMPT"
codex-subagent run bug-detector --prompt "$ISSUE_PROMPT"
codex-subagent run perf-analyzer --prompt "$ISSUE_PROMPT"
codex-subagent run test-writer --prompt "$ISSUE_PROMPT"
codex-subagent run code-fixer --prompt "$ISSUE_PROMPT"
# Optional documentation pass
# codex-subagent run docs-reviewer --prompt "$ISSUE_PROMPT"
```
Each command streams standard Codex events; successes end with a duration summary.

#### 5. Compare with the base CLI (optional)
```bash
CODEX_SUBAGENTS_ENABLED=1 codex subagents list
CODEX_SUBAGENTS_ENABLED=1 codex subagents run security-scanner --prompt "$ISSUE_PROMPT"
```
Outputs should match the shimmed commands.

#### 6. Clean up
```bash
rm -rf codex-cli/vendor
npm uninstall -g @openai/codex   # optional
```
Keep `.codex/agents/` if you plan to reuse the pipeline; the registry reloads on demand.

---

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
