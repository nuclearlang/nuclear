# AI loop: specs to deployed code, with local agents

NuclearLang ships three pieces for working with AI agents. Together they take a spec to tested, reviewed code on your
machine, and the same pipeline can run on a server.

| Piece | What it is |
|---|---|
| [Nuclear Smart Graph](https://github.com/nuclearlang/nuclear-graph) | A private library of the whole ecosystem, as cards on shelves: what it is for, how each thing is written, every kind, tool and error, what can and cannot be done, how each task is done, validated code. Plus the **MCP server** (Model Context Protocol) of NuclearLang for Claude Code, Codex or Cursor. |
| [Nuclear Developer](https://github.com/nuclearlang/nuclear-developer) | Local agents on 3B models: a coder, a tester, a security reviewer and a planner. They search the graph and check their answers with the compiler. |
| **Loop** (VS Code `Ctrl+Alt+P`, Android Studio / IntelliJ `Alt+Shift+P`, `nuclear-loop` in a terminal or CI) | The pipeline: a spec goes through plan, code, tests, security, artifacts and deploy, and every step shows who did it. |

The idea: a frontier model like Claude makes the decisions (the plan, the names, the review), and the local developer
writes each piece. A lead hands work to a junior the same way. What the junior cannot fix, Claude fixes and keeps as
an example, so the junior gets it right next time.

## 1. Set up once

```sh
curl -fsSL https://nuclearlang.github.io/install.sh | sh    # NuclearLang, then the set-up of the AI services
nuclear-loop setup                                           # the set-up alone, any time
```

The set-up reads your machine, suggests a set of models that fits it (`lite`, `medium` or `hard`, or remote agents
only), says how far each works, and installs what you choose: the Smart Graph, the Nuclear Developer, the models, the
knowledge base. Both services then start at every login. VS Code has the same set-up as a wizard (*Nuclear: Set Up
the AI Services*). Every step, the profiles and what to do when one fails: [Set up the AI services](setup.md).

```sh
nuclear-loop setup --status                              # what is installed and running
claude mcp add --scope user nuclear -- nuclear-graph mcp # optional: Claude leads
```

No program can read the graph until you allow it with `connect` or `allow` (the set-up allows the Developer and the
MCP server). Its documents are encrypted on disk and it refuses web pages. See
[its README](https://github.com/nuclearlang/nuclear-graph#who-may-read-it).

## 2. Write a spec

Open the Nuclear workspace and press `Ctrl+Alt+P`. The first page is the spec:

1. **What you want to do**: a new feature, a refactor, a bug, tests, a security review, documentation or a diagram.
   Each choice gives the description a template.
2. **The spec**: write it, or import it from Jira (*Connect…* once: site, email, API token, kept in VS Code's secret
   storage; then a key such as `PROJ-123`, or *My open issues*) or from a GitHub issue (`123`, `owner/repo#123`).
   Add the **acceptance criteria**. The run checks them, and a spec needs at least one.
3. **Context for the agents**: files suggested from the project and your skills, by the words of the spec. The
   project's agent instructions (`AGENTS.md`) are always selected.
4. **Who leads**: *Claude leads, the local developer writes*, or *Local agents only*. *Preview the plan* shows the
   pieces the local planner would make, so you can edit them before starting.

Press **Start the loop**. The spec is written to `specs/<id>/spec.md`.

Without VS Code, a spec is a markdown file in the same shape (`# Title`, `- Use case:`, `- Lead:`, `## Description`,
`## Acceptance criteria` with `- [ ]` items, optional `## Context` and `## Pieces`):

- **JetBrains** (Android Studio, IntelliJ): the *Nuclear Loop* tool window (`Alt+Shift+P`) lists every project's specs
  and runs; *New Spec* writes the file, *Run the Spec* (local agents or Claude leading) runs it, and each run unfolds
  into its stages and steps, with the trace, code and output of the selected step.
- **A terminal**: `nuclear-loop run specs/<id>/spec.md [--lead local|claude]`, `nuclear-loop runs`.

Every run is kept in `.nuclear/loop/runs/<id>.json`, the same file each editor draws.

## 3. Follow the run

The run is drawn as a graph, left to right:

- **A column per stage**: Specification → Generate → Plan → Tests first → Code → Verification → Security → Artifacts →
  Deploy. Each shows its state, its time and the agents that took part; a stage the strategy leaves out says why.
- **A node per step**, in the order it ran, and under it the steps its agent followed (recall from the graph, with
  the cards it read by name → write → repair with the compiler).
- **Click a node** and it opens beside the graph: what it did, **the decisions it took and why** (which strategy and
  by which signs, why a piece was dropped or a file went to the tester), its input, its output and its code.
- **Specification** ends with the **analysis**: which kind of request the spec is and the strategy the run follows
  ([the strategy of a request](#the-strategy-of-a-request)). The step says what it understood and why; a fact it
  lacks is asked there, before anything is written.
- **Generate** comes before any agent: `dragonfly gen resource` writes, already compiling, every resource the spec
  names (a `## Resources` section, one `dragonfly gen resource` argument line per item, or else each `GET <url>` line
  of the description: `GET https://rickandmortyapi.com/api/character` is `characters Character`), and every resource
  the analysis read in plain words ("each supplier has a name and a phone; we add and list them"), each call tried
  with `--dry-run` first. The fields come from
  the spec's example answer or its `Key | Type` table, or from the API's own answer. The planner then plans only what
  is left (often nothing), and a crash undoes the generated files like an agent's.
- **Plan** shows the work as a tree: the big parts (one per component), the tasks of each (a layer: data, domain, the
  facade, the entry points, the tests) and the pieces of each task (one file). It also says, before any code, what
  of the spec is beyond the platform or the local agents (the graph's capabilities whose trigger words the spec
  writes: "authentication" on a backend, "form" on the web; a spec too long for a local model; a plan without pieces):
  split the task, give the missing facts, or let a frontier model lead.
- **Tests first** is skipped when the generators wrote every file of the run: they wrote its tests too, and the step
  says which. A resource that reads another API gets no generated test (it would call the real one): the step says
  so, and only the acceptance step holds the criteria against the result.
- **Anything runs again.** *Run again from this step* on every step, passed or failed: it and every step after it
  run again, the steps before it are kept. *Run <stage> again* on every stage. *Run again from scratch* on the run:
  the whole pipeline from the specification. What the steps left out wrote is undone first; a file you changed since
  the run wrote it is kept, and the page says so. The attempt left behind stays under the run with its reason. From a
  terminal: `nuclear-loop retry <run> [--from <stage> | --step <id>]`, `nuclear-loop restart <run>`. A run made by an
  older version of the Loop starts from scratch at any retry.
- **Acceptance.** When the verification passes, each acceptance criterion is held against what the run left: the
  ones the result does not show met are listed as left for the reviewer (in the page and in the journal). It informs;
  it does not fail the run.

| Actor | Does | Does not |
|---|---|---|
| You | write the spec, choose the context and the lead | write a piece here |
| Claude | plans, fixes what the local coder could not, teaches the graph | write every piece |
| Planner (gemma4:e2b) | splits a spec when no frontier model leads | make architecture decisions |
| Coder (qwen2.5-coder:3b) | writes one file at a time and repairs it with the compiler | choose names, data or architecture |
| Tester | writes the tests first, from the criteria and the plan; fixes a test file the project's check rejects | delete or weaken a failing test |
| Security | reviews each new file | change the code |
| Shell | runs the project's commands | run anything not in `.nuclear/loop.json` |

How the work is split, so the local 3B models only think the business logic:

1. Before anyone writes, the transpiler has written the **project map** (`.nuclear/map.md`): every declaration with
   its kind, its description (its `//` doc comment) and its members. The agents read it instead of every file, so
   give each declaration a one-line description.
2. **Tests first**: the tester writes one test per acceptance criterion with the names the plan gives.
3. **Code**: the coder writes each piece at its place (`nuclear help layout --platform <p>`), with the card of its
   kind, the mistakes to avoid with that kind, examples of it and the project's files that declare the names its task
   writes (it reads them by itself); the compiler checks it, then the whole project.
4. **Verification**: every test runs. A failing test goes back to the coder told in Nuclear (`dragonfly explain`:
   the message, a hint and the `.nuc` lines), up to three rounds; when Claude leads, it fixes what is left. The
   step's *For the agent* tab shows what the agent read; *Output* keeps the real log.

The local agents only ever see Nuclear and the platform (backend, mobile, web), never Python, TypeScript or Dart.

**The journal.** Every run keeps `.nuclear/loop/runs/<id>.md` beside its `.json`: the files written, what is left for
the reviewer, and every step with its agent, its trace and what the agent read. At the end the local **documenter**
reads it and writes, at its top, what was built, how it was checked and what is left. A reviewer's notes go to
`<id>.review.md`.

**When it fails.** Calls to the local agents that fail for a passing reason are retried with exponential backoff
(the settings say how many and how long). When a stage fails, the local diagnoser tells why and what to do; *↻ Retry
from <stage>* (or `nuclear-loop retry <run>`, or the MCP tool `developer_retry`) runs it again from there.

**It survives the editor.** A run keeps a trace log, a heartbeat and a ledger of the files it writes: if VS Code closes
or crashes in the middle, the run is found interrupted when it opens again, what was left half written is undone, and
it goes on from that step. Every run first checks its connections (the Developer, the graph, Ollama and the models,
Claude when it leads, free disk); every few minutes every integration is checked again.

**It keeps working.** A run or a chat question does not stop when you switch tools or close its page; a notification
(and a badge on the Nuclear icon) tells you when it ends. *↻ Retry* goes on from the step that failed.

**Settings (`Ctrl+Alt+S`).** The services, the local models, the MCP server for Claude Code, Codex and Cursor, the
connections and the retries, in one page of the editor.

**The chat (`Ctrl+Alt+A`, *Nuclear agent*).** A chat with the local agents, like Claude's or Cursor's: questions
about the project are answered from its map and the graph; `/task <what to do>` is an order, run as this pipeline by
the local agents; templates come from the project's names; sessions are kept in `.nuclear/chat/`.

**From Claude Code, Codex or Cursor (MCP).** The Nuclear MCP server ([its tools and how to add it](mcp.md)) tells every
frontier model to generate first (`generate_resource`, `generate_usecase`) and to let the local developer work on
what is left: `developer_task` runs this pipeline with the local agents and answers the journal; the frontier
model validates (`nuclear_check`, `dragonfly test`), corrects only what is wrong and records it with
`developer_review`, whose corrected files become examples in the graph for the local developer.

### The strategy of a request

A whole system, one screen, a rename and a failing test are not done the same way. The graph holds a **strategy** for
each kind of request (31 rows, `strategy:<name>`; [ADR-0105](../adr/0105-strategies.md)): how the kind is recognised,
its plan with the tool of every step, and what it asks of a run. The local **analyst** picks it before anything is
generated or planned. No model chooses: the request's exact signs decide (a diagnostic code, a test's output, `rename
A to B`, a `GET <url>` line), then its meaning, and whether the project already has the names it writes.

| Family | Kinds of request |
|---|---|
| create | a whole system, a project, a component, a resource of another API, a stored resource, the app of a backend, only an interface, a repository, a use case, a route, one declaration |
| change | one more field, method or action; a rename; a move or an extraction; a break of the architecture's rules; the database's schema; the configuration; taking something out |
| fix | it does not compile; a test fails; it does the wrong thing; it fails while running; the data does not match; a screen shows the wrong thing; the machine or a service fails |
| verify and explain | tests for what exists; a security review; documentation; a question; building, running and shipping |
| beyond | what NuclearLang or the local agents cannot do: said before anything is written |

What a strategy changes in a run:

- **The stages that run.** A compile error skips *Generate*, *Plan* and *Tests first*: the compiler's exact fixes
  first (`nuclear fix .`), then each file with errors is repaired with the card of each error. A failing test goes
  straight to *Verification*. A rename plans and writes, and writes no test.
- **Where the pieces come from.** For a new feature the planner splits what the generators left. For a change of
  something that exists (a field more, a column) the pieces are the files that declare it. For a rename a tool gives
  every file that writes the name, already rewritten: no model writes.
- **What is asked first.** A stored resource without its fields, a rename without its two names, a bug without what
  should happen: the analysis stops the run with the question. Nothing is guessed.
- **The wiring.** After *Code*, each use case the coder wrote is wired by `dragonfly gen usecase`: its alias in the
  facade and, when the spec's sentence gives it (`POST orders/{id}/cancel`), its route.
- **A bug's repair** goes to the file the failure names, whoever wrote it, and never to the test.

A request that is not a run (a question, a command, something beyond the platform) stops at the analysis and says
where it belongs. The analysis read it wrong: name the strategy in the spec (`- Strategy: usecase`, under `- Use
case:`) and run it again.

```sh
nuclear-graph strategy "al cancelar un pedido pagado debería rechazarlo pero lo acepta"   # which strategy, and why
nuclear-graph card strategy:bug-behaviour                                                # its plan
```

## 4. The pipeline's commands, locally and on a server

*Pipeline settings* opens `.nuclear/loop.json`, the commands of each stage. The defaults come from the platform's
CLI (`nuclear check`, `dragonfly build`, `install`, `test`). Add `seed`, `integration` and `deploy` commands for the
deploy stage:

```json
{
  "commands": {
    "seed": ["dragonfly migrate", "python seed.py"],
    "integration": ["dragonfly test -- -m integration"],
    "deploy": ["docker compose up -d --build"]
  },
  "branches": ["main", "feature/**"]
}
```

*Export CI* (or `nuclear-loop export github|gitlab|bitbucket`) writes the same commands as
`.github/workflows/nuclear-loop.yml`, `.gitlab-ci.yml` or `bitbucket-pipelines.yml`. The agent stages run on your
machine; the server checks their result on every push of a feature branch. `nuclear-loop ci [stage...]` runs those
server stages where you are, to see what the server will see before you push.

## 5. How good is the local developer?

It is measured against Claude Sonnet on a bank of 45 tasks, 18 of them held out: 42 single files of every platform
and 3 features of several files. On single files the local 3B coder is at Sonnet's level (36 against 36). On whole
features it is behind: it passes one of the three by itself, while Sonnet passes all three. That is why Claude leads by default: it plans and fixes, the local developer writes the pieces. The
method, every round and every task are in
[the Developer's README](https://github.com/nuclearlang/nuclear-developer#evaluation-how-it-was-trained).
