---
adr: 0102
title: The AI loop — Nuclear Smart Graph, Nuclear Developer and the Loop tool
status: Accepted
scope: nuclear
---

# ADR-0102: The AI loop — Nuclear Smart Graph, Nuclear Developer and the Loop tool

- Status: Accepted
- Scope: `nuclear-graph`, `nuclear-developer`, `nuclear-vscode`, `nuclear-jetbrains`, `nuclear` (project definitions, backend
  `dependencies(...)`), `dragonfly-backend` (`[library.error]`)
- Date: 2026-11-10

## Context

The user asked for spec-driven work (SDD) inside the editor: a spec, written in the editor or coming from Jira or
another tracker, goes through the whole cycle on the developer's machine (spec, plan, code, tests, security,
artifacts, deploy), drawn as a pipeline where every step shows who did it, and the same pipeline can run on a server
(GitHub, GitLab, Bitbucket). A frontier model (Claude) must not do all the work: it plans and hands each piece to a
local model of at most 3B parameters that knows NuclearLang, as a lead hands work to a junior. The local model cannot
think much nor hold much context; a local knowledge base (RAG) must make up for it, and only programs the user allows
may read that base.

The machine has no dedicated GPU (Radeon 880M, 15 GB shared memory): training a 3B model's weights takes days per
round. The user chose **retrieval plus distillation** instead: a frontier model answers a task bank once, its answers
that pass the compiler become examples in the knowledge base, and the local model is measured against it until its
answers are equivalent.

## Decision

1. **Nuclear Smart Graph** (`nuclear-graph`, its own repository): the knowledge base of the whole ecosystem in Chroma,
   with vectors from a local embeddings model (EmbeddingGemma on Ollama). Four collections: `knowledge` (every spec,
   guide, context file, ADR, dialect word, example and the CLIs' help, made by `ingest` and refreshed by hash),
   `examples` (validated task → solution pairs), `prompts` (the validated prompt of each use case), `memory` (what an
   agent keeps about a project).
2. **The graph is private.** Only its server process opens the store. It listens on 127.0.0.1 and answers only with a
   token; each program asks for access and the user allows it (`nuclear-graph allow`, `connect`); tokens are kept as
   hashes, with scopes `read`, `write`, `admin`; requests with an `Origin` header (a web page) are refused; documents
   are encrypted at rest with AES-256-GCM, the key and the owner's token live in the desktop keyring (`secret-tool`),
   the data folder is 0700; only `ingest` (the owner) writes `knowledge`.
3. **One MCP server for NuclearLang** (`nuclear-graph mcp`, stdio): the graph (`graph_search`, `graph_prompt`,
   `graph_remember`), the compiler (`nuclear_check`, `nuclear_build`, `nuclear_help`, `nuclear_brief`, `nuclear_fmt`,
   `nuclear_defs`), the `dragonfly` commands of a project (an allow list) and the local developer
   (`developer_code`, `developer_test`, `developer_security`). It reads the graph as the client `mcp`.
4. **Nuclear Developer** (`nuclear-developer`, its own repository) is a Dragonfly backend written in Nuclear (fastapi
   dialect): four LangGraph graphs (coder, tester, security reviewer, planner) on `qwen2.5-coder:3b` and `gemma4:e2b`.
   A small model follows steps better than it chooses tools, so each graph runs fixed steps: recall from the graph
   (the use case's prompt, knowledge, solved examples, example files) → write → `nuclear check` → repair with the
   compiler's errors (at most three attempts). Each HTTP request runs on its own thread. Its tools reach the graph and
   the compiler through `nuclear_graph.client` and `nuclear_graph.toolchain`, whose type definitions the project keeps
   in `defs/`.
5. **The evaluation is the training loop** (`nuclear-developer/eval`): a bank of tasks with acceptance texts, split
   into `train` (their frontier answers become graph examples) and `test` (held out); an answer passes when `nuclear
   check` reports no errors and it has every acceptance text; the local developer is equivalent to the frontier model
   on a task when both pass. A failure is fixed by adding a general rule or knowledge to the graph, never the answer.
6. **The Loop tool** in the VS Code workspace (`Ctrl+Alt+P`): the spec form (use case, the spec written or imported
   from Jira or a GitHub issue, acceptance criteria, the context files suggested from the project and the user's
   skills, who leads), the runs, and each run as a pipeline drawn like a C4 diagram (level 1: the stages; a click
   zooms into a stage's steps; another into a step's trace, code and output). The engine (`loopEngine.ts`) has no VS
   Code dependency. Claude leads through `claude -p` with the MCP and the local developer writes; what the coder
   cannot fix goes to Claude, which keeps the fix as an example. The pieces that compile alone but not together get
   one local repair pass, then Claude when it leads.
7. **The pipeline's commands are data** (`.nuclear/loop.json`, defaults from the platform's CLI): the same commands
   are exported as a GitHub Actions, GitLab CI or Bitbucket Pipelines file. The agent stages run on the developer's
   machine; the server checks their result.
8. Generic mechanisms this needed, in the platform: `nuclear build|check` load a project's own type definitions
   (`<project>/defs/<language>/<name>/<version>/` and `build/defs.lock`, `project::with_project_defs`, also used by
   the LSP); a backend `config.nuc` has `dependencies(name = "version")` (a version that starts with `.` or `/` is a
   folder; fastapi and nestjs write it to `pyproject.toml` / `package.json`); `e.message` of a caught error is right in
   both backend dialects.
9. **The Loop without VS Code** (2026-11-11): `nuclear-loop` is the same engine bundled for Node (`run` a spec file,
   `ci` the server stages of `.nuclear/loop.json`, `export` the CI file, `runs`); a spec is its markdown file, so
   JetBrains drives it from a *Nuclear Loop* tool window (specs, runs, stages, steps). The Loop is for VS Code and
   Android Studio / IntelliJ only (decided by the user, 2026-11-11: no Zed features for now). `install.sh` installs
   `nuclear-loop`; `install.sh --ai` also the graph and the Developer.
10. **The evaluation covers every platform and several files** (2026-11-11): tasks of the mobile and web platforms,
    and `[[feature]]` tasks that are planned, written piece by piece and checked together in a fresh `dragonfly new`
    project. What a frontier model got wrong too is fixed where every agent reads it (the web platform got its brief;
    `dragonfly new`'s mobile skeleton did not compile), never in the task.
11. **The local agents write only Nuclear** (decided by the user, 2026-11-11). They name a platform (`backend`, `mobile`,
    `web`), never a dialect: `nuclear check|build|help|spec --platform <p>` uses the platform's default dialect underneath
    (the Nuclear of a platform is the same in all of them), `nuclear spec --brief --platform <p>` gives the base and the
    platform's sheet (`spec/brief/<platform>.md`) without the target language's, and the graph's `nuclear` collection
    holds only what they read (examples, the kinds' words, the base and platform specs, the language guide; no dialect
    templates, runtimes or target-language code blocks). The full `knowledge` collection is the frontier model's.
12. **The tools write; the agents think the business logic** (decided by the user, 2026-11-11). Before an agent writes,
    the transpiler has written the project map (`.nuclear/map.md`: each declaration with its kind, its doc comment and
    its members) and the platform says where each kind goes (`[layout]`) and its smallest file (`[skeletons]`,
    `dragonfly make <kind> <component> <Name>`). The Loop runs: spec → plan → **tests first** (the tester, from the
    criteria, the planned pieces and the map) → code (piece by piece) → **verification** (build and every test; a failure
    goes back to the coder told in Nuclear by `dragonfly explain`, at most 3 rounds) → security → artifacts → deploy.
    Measured and discarded: giving the local coder the `dragonfly make` skeleton as the existing file (29 of 42 tasks
    against 37: a 3B model keeps the skeleton's placeholders); `dragonfly make` stays for people and frontier models.
13. **Logs for the agents, beside the real ones.** A failing build or test is passed through `dragonfly explain --json`
    (Dart, Python and Node stacks → the message, a hint and the `.nuc` lines with their code); the agent reads that
    (the step's `agent` field, the panel's *For the agent* tab), the raw output stays as it is.
14. **Over MCP the local developer works first** (asked by the user, 2026-11-11). The server's instructions tell any
    frontier model (Sonnet, Opus, Haiku...) to start every task that writes `.nuc` with `developer_task` (the Loop with
    the local agents only, `nuclear-loop run --lead local`), then only validate and correct, and record the review with
    `developer_review` (`<run>.review.md`; corrected files to the graph's `examples`). Each run has a journal
    (`.nuclear/loop/runs/<id>.md`, `journalMarkdown`) and the local documenter (`POST /developer/document`) writes its
    summary of what it did at the top.
15. **The agents' prompts are files** (asked by the user, 2026-11-11): every prompt a local model reads is a markdown file
    of the graph (`prompts/agents/<step>.md`, `{{name}}` placeholders, an empty section dropped), served as the prompt
    `agent.<step>`; the agents' code holds no prompt text.
16. **A chat with the local agents in the editor** (asked by the user, 2026-11-11): the workspace tool *Nuclear agent*
    (VS Code) talks to the Developer's assistant (`POST /developer/chat`) about the active project; `/task` runs the
    Loop with the local agents; templates from the project's map; sessions in `.nuclear/chat/`.
17. **Failures are diagnosed, retried and retryable** (asked by the user, 2026-11-11). A call to the local agents that
    fails for a passing reason (unreachable, 408/425/429/5xx) is retried with exponential backoff and full jitter
    (`min(maxMs, baseMs·2^(n-1))·random`, `Retry-After` wins; 5 tries, 1 s, 30 s by default): jitter spreads the
    retries, the cap bounds a long outage, and a deterministic failure (a 4xx, a compiler error, a failing test) is
    never retried blindly — it goes to the repair rounds or the diagnoser. The diagnoser (`POST /developer/diagnose`)
    classifies by rules, gathers evidence with the tools and explains with the local model. The user retries a failed
    run from its failed stage (panel, chat, `nuclear-loop retry`, MCP `developer_retry`); the earlier attempt is kept.
18. **The AI loop's configuration lives in the editor's settings** (asked by the user, 2026-11-11): `nuclear.ai.*` and
    the *Settings and integrations* page (services, models under the 3B rule, MCP in Claude Code/Codex/Cursor,
    connections, retries); a project's `.nuclear/loop.json` wins for its pipeline.
19. **Work outlives its page; notifications tell** (asked by the user, 2026-11-11): Loop runs and chat jobs belong to the
    extension (a page only draws them); when one ends a notification (toast with *Open*, a badge on the Nuclear icon, a
    count on the tool's tab, the list in its pane) says so unless the tool is on screen. A retry goes on from the step
    that failed (`start(from, step)`, the passed steps kept); the chat shows the agent's steps folded, a context meter,
    a question history (↑ / ↓) and a retry of a failed question. Every pipeline command runs with the installers' bin
    folders on its PATH (`toolEnv`): an editor started from the desktop has no `~/.local/bin`.
20. **The knowledge base is one folder, its own repository** (asked by the user, 2026-11-11: the chat did not know the
    MCP). `nuclear-developer/data` (`nuclear-knowledge`): every document, example and tool help of the project, copied
    byte for byte by `nuclear-graph knowledge build` into the sections of `sections.toml` in reading order, nothing
    left out (`99-other`), checked by `knowledge check` and a test. The graph ingests it as the whole knowledge; the
    local agents read every section but the target languages, the dialects underneath, the runtimes and the ADRs.
21. **A run survives the editor** (asked by the user, 2026-11-11: VS Code was restarted and the run stayed "running"
    for ever, nothing working on it). `loopSafety.ts`: the run's state is written atomically; every event goes to an
    append-only trace log (`.nuclear/loop/runs/<id>.log`); its owner (pid, host) beats every 10 s; every project file it
    writes is kept in a ledger with what was there before. A run whose beat is stale or whose process is gone is
    orphaned: `recoverRun` fails its unfinished steps, undoes what they wrote and the run goes on from the step that
    failed (`resumeOrphans` on activation, `nuclear.ai.resumeInterrupted`; `nuclear-loop retry`). A call to the local
    agents is watched: cut and retried when it takes longer than `callTimeoutMinutes` or when, for `stallSeconds`, the
    Developer stops answering or Ollama holds no model; a command is killed after `commandTimeoutMinutes`. Two
    processes never run one run (its fresh owner refuses the second). A chat question left without an answer is
    marked interrupted with *Retry*.
22. **Every external connection is checked** (asked by the user, 2026-11-11): before every task (a run's first step
    *Check the connections*: the Developer, the graph, Ollama, the models installed, Claude Code when it leads, the free
    disk; a chat question: the Developer, the graph, Ollama, the assistant's model) and every `checkEveryMinutes`
    (`connections.ts`: also Codex, OpenCode, GitHub, Jira when configured), a change being a notification. Every
    integration is a setting with its default (`nuclear.ai.claude.*`, `codex.*`, `opencode.*`, `cursor.mcpFile`,
    `github.command`, `jira.site|email|criteriaField`, `watch.*`); only secrets (the Jira token) live in the secret
    storage.

## Consequences

- The local developer is at the frontier model's level on single files (36 of 42 against 36 after eleven rounds) and
  behind it on whole features (1 of 3 by itself; Sonnet 3 of 3): Claude leads by default (`nuclear-developer/eval/results/REPORT.md`). The references cost under USD 1.
- Security is per OS user: a program running as the same user could ask the keyring for the key as the graph does.
  The graph protects against other users, web pages and programs that only reach it over the network or MCP; full
  isolation needs an OS sandbox.
- Vectors are stored in clear (Chroma needs them to search); they leak the topic of a document, not its text.
- Not done yet: weight training (LoRA; what it needs: `specs/007-lora-nuclear-coder/plan.md`), the graph and the Developer in the release installers (they install
  from the source tree), live Jira and GitHub imports and the Claude-led runs inside a running editor (Untested in
  STATUS.md).
