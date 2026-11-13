# The MCP server of NuclearLang

MCP is the **Model Context Protocol**: an AI client (Claude Code, Codex, Cursor, OpenCode) starts a program and calls
the tools it offers. `nuclear-graph mcp` is that program for NuclearLang. It speaks MCP over stdio and gives a
frontier model four things: the **generators** that write the standard code, the **local developer** that writes what
is left, the **Smart Graph** to look things up instead of guessing, and the **compiler** to check everything.

The server also tells the model how to work, so every client behaves the same: generate first, let the local
developer write the rest, then validate and correct.

## 1. Set up

The server comes with the AI services: [set them up](setup.md) first (the set-up also allows MCP clients to read
the graph). Then add the server to your client:

```sh
claude mcp add --scope user nuclear -- nuclear-graph mcp   # Claude Code
codex mcp add nuclear -- nuclear-graph mcp                 # Codex
```

Cursor reads `~/.cursor/mcp.json`, OpenCode `~/.config/opencode/opencode.json`:

```json
{ "mcpServers": { "nuclear": { "command": "nuclear-graph", "args": ["mcp"] } } }
```

```json
{ "mcp": { "nuclear": { "type": "local", "command": ["nuclear-graph", "mcp"], "enabled": true } } }
```

In VS Code, *Settings and integrations* (`Ctrl+Alt+S`) adds it to each client with a button.

A machine set up for **remote agents only** has no Smart Graph and no local developer: the server then offers the
generators, the compiler, the command line and the internet tools, without the `graph_*` and `developer_*` tools, and
tells the model to write what the generators do not.

What each group of tools needs running:

| Tools | They need |
|---|---|
| `project_inspect`, `generate_*`, `nuclear_*`, `dragonfly` | only the installed `nuclear` and `dragonfly` commands |
| `api_inspect`, `web_search`, `web_read` | the internet |
| `graph_*` | the Smart Graph (127.0.0.1:7878), which a local profile of the set-up installs and starts |
| `developer_*` | the Nuclear Developer (127.0.0.1:7879), the graph and Ollama with the profile's models |

## 2. How a model works with it

The server's instructions tell every client to call `graph_strategy` first, with the user's own words: it answers which
kind of request it is and the plan of that kind ([the strategies](loop.md#the-strategy-of-a-request)). For a feature
the plan is this order:

1. **Inspect.** `project_inspect`: what the project already has, so nothing is declared twice.
2. **Generate.** For every resource the spec names (a model and what is done with it), `generate_resource`. Its
   fields come from the API's answer (`from_json`; `api_inspect` shows its shape first), from the spec (`from_spec`)
   or from `fields`. When neither gives them, the model looks up the API's documentation (`web_search`, `web_read`)
   and, if it is still unclear, asks the user: it never guesses. A use case that only calls a repository method is
   `generate_usecase` with `calls`.
3. **The local developer writes the rest.** `developer_task` with what the generators did not write (a use case with
   its own rules, a view's details): the local agents plan, write the tests first, write the code, run the tests and
   the security review, and answer a journal of everything they did.
4. **Wire.** Each use case the local developer wrote, with `generate_usecase` (its alias in the facade, its route).
5. **Validate.** `nuclear_check` of the project must say `ok` and `dragonfly test` must pass.
6. **Correct only what failed**, never what passed. Before writing `.nuc` the model reads `nuclear_brief` and
   searches the graph. A run that failed for a passing cause is run again with `developer_retry`.
7. **Review.** `developer_review` with what was corrected and why: the journal keeps it and the corrected files
   become examples the local developer finds next time.

The model writes the code itself only when the local developer is not running, and says so.

## 3. The tools

### Generators

They write finished code that compiles, never a skeleton to fill in. Each is a `dragonfly gen` command:
[the generators' guide](https://github.com/nuclearlang/dragonfly/blob/main/docs/dragonfly/generators.md) has the
options, the examples and what each platform gets.

| Tool | What it does |
|---|---|
| `project_inspect(project)` | the project's platform and dialect, its components with their declarations by kind, the generators of its platform |
| `generate_component(project, component)` | a new component with its facade |
| `generate_resource(project, component, name, from_json \| from_spec \| fields, source, path, actions, screen, plural, dry_run)` | a resource: models, repository, a use case per action, the facade's aliases, the controller (backend) or a screen with its state manager (mobile, web), the tests |
| `generate_usecase(project, component, name, calls, route, dry_run)` | a use case wired into its component: its alias in the facade and, with `route` (backend), its route. With `calls` the use case is written too, because it only calls that repository method; without it the use case is one the project already has |
| `generate_screen(project, component, screen, route)` | mobile and web: a screen with its state manager and state model, for a view that is not a resource's list |
| `api_inspect(url, project, component, name)` | the shape of a public API's JSON answer; with a project, the files `generate_resource` would write from it |

```
generate_resource(project, "orders", "Order", fields="id:int total:double status:string", source="db")
generate_usecase(project, "orders", "PageOrders", calls="OrderRepository.page", route="GET orders/pages/{page}")
generate_usecase(project, "orders", "CancelOrder", route="POST orders/{id}/cancel")   # CancelOrder is in the project
```

Nothing is overwritten and nothing is declared twice; `dry_run` shows the files first. A generator that cannot write
something finished refuses and says what to do: `generate_usecase` without `calls`, for a use case the project does
not declare, answers where to write it.

### The local developer

| Tool | What it does |
|---|---|
| `developer_task(project, title, description, criteria, use_case)` | **the first step of every task that writes `.nuc`**: the local agents do the whole task in the project (plan, tests first, code, verification, security, artifacts) and answer the run's journal. `use_case`: `feature`, `bug`, `refactor`, `test` |
| `developer_retry(project, run, from_stage, from_step, scratch)` | a run again: from the step that failed, every step that passed kept; the whole of `from_stage`; from one step (`from_step`: its id in the journal, passed or failed) and every step after it; or from scratch (`scratch`: every stage again). What the steps left out wrote is undone first |
| `developer_review(project, run, notes, files)` | **the last step**: the lead's corrections, kept in the journal; each corrected file goes to the graph's examples |
| `developer_code(task, platform, path, kind, context, project)` | one `.nuc` file outside a task, written, checked and repaired by the local model. It returns the file; it does not write it |
| `developer_test(task, code, platform)` | the Nuclear tests of a piece of code |
| `developer_security(code, platform)` | the security findings of a piece of code |

### The Smart Graph

| Tool | What it does |
|---|---|
| `graph_strategy(request, platform)` | **the first call for any request**: which kind of request it is (a whole system, a component, one screen, a use case, a rename, a failing test, a question...), the signs that say so, and the plan of that kind, step by step with the tool of each step |
| `graph_answer(question, platform)` | the exact answer of a question that has one (every command of a tool, the tools of this server and their parameters, the commands that write a kind, a diagnostic by its code, the stages of the Loop, a version...): the rows of the graph's database, whole, never recalled; then the cards the shelves answer by meaning. For "which", "list", "what are the" questions |
| `graph_search(query, shelf, k, platform, kind, type)` | the cards that answer a question, by meaning and by exact name (a diagnostic code, a command, a kind), each with the rows it links to. `shelf`: `concepts`, `language`, `platforms`, `tools`, `errors`, `capabilities`, `playbooks`, `examples`, `docs`, `memory`; empty searches all |
| `graph_card(id)` | one row by its id: `error:N0114`, `kind:backend.usecase`, `tool:dragonfly.gen.usecase`, `mcp:generate_usecase`, `playbook:add-a-use-case` |
| `graph_prompt(use_case)` | the validated prompt of `feature`, `refactor`, `bug`, `docs`, `diagram`, `test`, `security`, `piece`, `plan` |
| `graph_remember(id, text, collection, project)` | keeps a decision (`memory`) or a solved example (`examples`) for the local developer and later sessions |

### The compiler and the command line

| Tool | What it does |
|---|---|
| `nuclear_check(code \| path, dialect)` | `ok`, or one line per error with its fix, for a piece of Nuclear or a whole project |
| `nuclear_build(code, dialect)` | what a piece of Nuclear becomes in the dialect's language |
| `nuclear_fmt(code)` | the piece in the canonical layout |
| `nuclear_help(word, dialect)` | the documentation of a kind, a decorator, a topic or a command |
| `nuclear_brief(dialect)` | the short sheet of Nuclear and a platform: read before writing `.nuc` |
| `nuclear_defs(name, dialect)` | a declaration of the type definitions (a class or `Class.member` of a library) |
| `dragonfly(project, command, args)` | `new`, `build`, `install`, `test`, `check`, `analyze`, `make`, `gen`, `migrate`, `add`, `explain` in a project |

### The internet

| Tool | What it does |
|---|---|
| `web_search(query, pages)` | searches the internet and answers the first results with the text of their pages (your own SearXNG with `NUCLEAR_SEARXNG_URL`, Brave with `nuclear-graph web-key brave`, else Wikipedia) |
| `web_read(url)` | the text of a public page, without its code blocks. Local addresses are refused |

## 4. When a tool fails

| The tool answers | What to do |
|---|---|
| `the local developer is not running at ...` | `nuclear-loop setup --status` says what is missing; restart it in *Settings and integrations*, or with [the command of your system](setup.md#5-after-the-set-up) |
| the graph refuses the client | `nuclear-graph connect mcp --scopes read,write`, with the graph running |
| `... is not inside a Dragonfly project (no config.nuc)` | give the project's folder: `dragonfly new <name>` creates one |
| `'<command>' is not allowed here` | the `dragonfly` tool runs only the commands of its list |
| a generator refuses | its message says what is missing: [when a generator refuses](https://github.com/nuclearlang/dragonfly/blob/main/docs/dragonfly/generators.md#4-when-a-generator-refuses) |

The graph knows every tool of this page: ask it with `graph_card("mcp:<tool>")`, or ask the editor's chat
(*Nuclear agent*, `Ctrl+Alt+A`) in your own words.
