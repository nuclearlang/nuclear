# Project status — NuclearLang platform

The only source of what is done, half done, untested and pending in the WHOLE platform (`crates/`,
dialects, `packages/dragonfly`, `cli/`, `editors/`, `website/`). Loaded in every session (`CLAUDE.md`
imports it). No other file keeps pending lists: the context tree (`docs/context/`) describes how things
work and links here.

## How it is kept (mandatory for every session)

1. **At start:** if the start hook (`tools/context/check.sh`) lists files newer than this one, another
   session finished work without recording it: look at them and record them here before reporting or planning.
2. **At the end of every task**, also a report or a review, before the final reply: one Log line
   (`date · session · what · where`) and the lists moved (Pending → done, Half done, Untested). A resolved
   item is deleted. An item lives in exactly one list.
3. **Before leaving something half done** (the user leaves, the context fills up, an unsolved error): record
   it in "Half done" with what is missing and where, at that moment, not at the end.
4. A status report is checked against the code before it is given.
5. The end-of-turn hook does not let a session finish if it changed code, dialects, specs or tools after
   the last edit of this file (once per turn).
6. How something works goes in its spec and its `docs/context/` file; here only state and a link.

## Next: the competitiveness plan (decided by the user, 2026-11-06)

What Nuclear and Dragonfly lack to compete, in the order the user chose after the 2026-11-06 analysis (nuclearlang-4a).
This is the work to pick up next; each step is done when its line here says so, and its details move to the lists
below while it is open.

1. **Exist publicly:** git repository and CI that runs everything done by hand today (`cargo test`, clippy, the spec
   checks, `flutter test`/`analyze`, pyright, tsc, the examples); public repository with license, CONTRIBUTING,
   changelog and a versioned 1.0 spec (semver); `dragonfly` on pub.dev, `dragonfly-py` on PyPI, `dragonfly-nest` on
   npm, the extensions in the marketplaces, binaries for macOS and Windows, a one-command installer. — installers and
   release workflow DONE 2026-11-10 (ADR-0101; they work once the repositories and a `v*` tag exist).
2. **A real reference app shipped:** one complete app on Android, iOS and web with a fastapi backend deployed, built
   from the tutorial on a clean machine; it burns down "Untested" (device, hot reload, debugging, the local database
   on disk, Postgres/Redis/Kafka/OpenAI for real) and the bugs it finds are fixed.
3. **Shared contracts across platforms:** a backend entity generates the mobile model, the typed repository and the
   routes (today a model is written twice and the http client by hand). Needs an ADR first: it touches the compiler.
   — DONE 2026-11-06 (ADR-0098: the contract, OpenAPI, `contracts(...)` by path or url, `nuclear contracts
   update|list`, the generated `contracts/<name>/`, N0307, N0116; the VS Code "Nuclear HTTP" view). Left: a drift
   warning by version, the CLI aliases (cli/ session).
4. **Backend parity:** entity relations, migrations, transactions, pagination, auth/guards and sessions in fastapi and
   nestjs, background jobs, file uploads, Dockerfile, observability; `dragonfly make`/`check`/`watch` for backend
   projects (details in "Pending: Dragonfly backend" and "Pending: cli/"). Needs an ADR for the data parts. — ADR-0100
   phase 1 DONE 2026-11-10 (`@primary(generated)`, TypeORM relations + `relations(...)`, `paginate` → `Page<T>` also on
   mobile local tables, `@transactional`); §5 migrations DONE 2026-11-10 (`dragonfly migrate`); next: auth/guards,
   jobs, uploads, Dockerfile, observability.
5. **The web platform:** the `react` dialect, so "three platforms" is true. — FIRST VERSION DONE 2026-11-06 (ADR-0097;
   what is left: forms, converters, translations, local database, plugins/AI, the CLI's web commands, a browser run).
6. **Language:** `nuclear fmt` (DONE 2026-11-10, only reflow), TypeScript type definitions, member types in the definitions, named/factory
   constructors and patterns in lambdas/`for`/`match`, `const` inference; then freeze the 1.0 spec.
7. **Prove the thesis with numbers:** the hard agent evaluation (plan 001 F7: an app from scratch, backend and mobile)
   and benchmarks of the generated code against hand-written Flutter/NestJS/FastAPI, published. — THE SYSTEM IS DONE
   2026-11-06 (`tools/thesis/`, README step by step); the bench ran (B holds, C does not on two CPU programs); the agent
   arms (thesis A) wait for the user's go (paid: ≈ USD 100 for 3 repeats).

Also listed in the analysis, after these: offline write sync and conflicts for the local database, push
notifications, OAuth/Firebase Auth, analytics and crash reporting, deep links, CI templates and store publishing
from the CLI, an MCP server (DONE 2026-11-10: `nuclear-graph mcp`, ADR-0102), a 30-minute validated tutorial and
comparison tables.

## Half done

- **The AI services built into the installer (ADR-0106; asked by the user 2026-11-13: "necesito que esto venga builtin
  dentro del instalador"; then: the extension reads the hardware and offers lite / medium / hard, the user decides,
  each says how far it works; the services start at login; Ollama detected or installed as a step, else remote agents
  only). DONE except the measurements below (nuclear-developer session, 2026-11-13).**
  - **DONE.** The engine `nuclear-vscode/src/aiSetup.ts` + its data `aiSetup.json` (requirements, profiles); the
    wizard `setupPanel.ts` (*Nuclear: Set Up the AI Services*, offered once, *Set-up…* in the settings page);
    `nuclear-loop setup [--profile] [--yes] | --status | --remove`; the services at login (systemd user units, launch
    agents, a Startup script); the machine's file `~/.config/nuclearlang/ai.env` as the default models of the Loop and
    the editor; the settings page's Start / Restart through the system's manager; `tools/release/stage.sh` puts `ai/`
    in every archive (9.9 MB the Linux one); `tools/release/install.sh` / `install.ps1` / `./install.sh --ai` call the
    set-up, bring a machine already set up to date without questions, and remove the services on `--uninstall`;
    `release.yml` clones the three AI repositories; `nuclear-graph`: the extra `mcp` (33 MB instead of 405), the
    remote-only MCP server (15 tools of 26, other instructions), `paths.machine()`. Ollama stays the runtime (the
    alternatives looked at are in the ADR). Docs: `docs/ai/setup.md` (new, in the website's AI menu), ADR-0106 + both
    indexes, loop.md, mcp.md, the READMEs of nuclear-graph, nuclear-developer and nuclear-vscode, eight CHANGELOGs,
    `docs/context/tech.md`, the editors' context. Knowledge base revision 37 (2 549 rows): `concept:ai-setup`,
    `playbook:set-up-the-ai-services`, `limit:remote-agents-only`, `failure:a-service-started-by-hand-holds-its-port`,
    ten rows that described the manual set-up updated, four questions (156 of 156 find their row). Editors 0.48.0
    (VS Code installed; JetBrains zip built; Zed installed).
  - **Checked.** nuclear-vscode `npm test` 28 + tsc; graph pytest 33; root cargo test 276; a local release archive
    installed into a test home: the `remote` profile end to end (the MCP server there offers no local tool); **on this
    machine for real:** `nuclear-loop setup --profile hard --yes` twice (the profile it already ran): the graph's
    tool, the Developer built and installed, the models found, both systemd user units written, enabled and started,
    the knowledge base loaded, `--status` all running, a strategy and a chat answer served by them; a hand-started
    graph on the port stops the set-up with the command that stops it; the wizard's pages rendered headless with this
    machine's state.
  - **This machine since 2026-11-13:** the graph (7878) and the Developer (7879) are the systemd user units
    `nuclear-graph` and `nuclear-developer` (profile `hard`, the models it had; the set-up suggests `medium` for its
    16 GB): `systemctl --user restart nuclear-developer` after a change to its code,
    `journalctl --user -u nuclear-developer -n 50`. They start at login; the processes started by hand are gone.
  - **LEFT.** (1) The measurements of the profiles: `lite` (`qwen2.5-coder:1.5b`) was never run; `medium`
    (`qwen2.5-coder:3b` for both roles) has its coder's numbers (the model of `hard`) and 6 of the first 8 chat
    questions, the ninth had not ended after five minutes when the run was stopped, and no `analyze` nor features
    round; `llama3.2:3b` as a lighter assistant was not run (its side Developer did not start); `qwen3.5:2b` took 2.5
    minutes per chat answer and was dropped. Each profile's card says what was measured; run `eval/run.py chat`,
    `analyze` and `local` on a side Developer (port 7880, `CODER_MODEL` / `ASSISTANT_MODEL`) per candidate, then write
    `aiSetup.json` (`test/setup.test.ts` holds `docs/ai/setup.md` to it). (2) A GPU other than NVIDIA's and Apple
    silicon is not recognised (this machine's Radeon 890M runs the models, Ollama says "100% GPU"; the set-up says no
    GPU with memory of its own). (3) Android Studio has no wizard (`nuclear-loop setup`). (4) The memory figures of
    the profiles (`ram`, `minRam`) come from the models' loaded size and this machine's behaviour, not from runs on 8
    and 32 GB machines.

- **The Smart Graph as a library of typed, linked, versioned rows, and the Nuclear Developer on top of it (asked by
  the user 2026-11-11; ADR-0104; nuclear-developer session, 2026-11-12).** The order: the RAG was "copied without
  sense"; design rows, collections and metadata for Chroma; every tool, error (and how they relate), capability and
  limit; versions, updated with every feature; retrieval tests with people's questions; then the local agent as a tool
  executor (spec → big tasks → medium → small, generators first, the internet for facts, knows its limits, roles and
  skills, a lite agent toolbox), later a frontier model as its manager.
  - **DONE.** `nuclear-graph`: `shelves.py` (the schema: 9 shelves), `assets.py`, `extract.py` (the generated rows),
    `knowledge.py` (`knowledge build | check | review | ask`), `store.py`, `retrieve.py` (meaning + exact names + the
    questions a row answers + links), `ingest.py`, routes `/v1/cards/{id}`, `/v1/applies`, `/v1/kinds/{platform}`,
    `workspace.py` (a project's files for an agent, read-only), MCP `graph_search` by shelf, `graph_card`,
    `developer_code(project)`. The knowledge base `nuclear-developer/data`: 2 549 rows, revision 37 since 2026-11-13
    (`gen usecase`, the generators' guide, the MCP page, the strategies; the running graph holds it) (2 325 generated, 186 written
    by hand: purpose, concepts, theory, questions, capabilities, limits, playbooks, roles, skills, failures, pitfalls,
    the diagnostics' relations), `sections.toml`, `questions.toml` (94 questions and 62 requests, English and Spanish: every one finds
    its row; before, "que es nuclear" answered the agent skill), `schema.json`, `manifest.json`, `revisions.json`; the
    byte-for-byte copies removed. The Developer: recall by kind (the kind's card, every pitfall of the kind, examples
    of the same kind only, each error's card at repair), the coder's guide without kind-specific rules, the faithful
    check, the project's files as context (`root`), the planner's spoken kinds, the plan's tree and limits, the
    assistant over every shelf in the developer's language. The Loop (editors 0.45.0): `root` in every call, the
    plan's limits in the step, *Tests first* skipped when everything was generated. The stop hook runs
    `knowledge check`; AGENTS.md rule 5 and workflow.md include the knowledge base. Measured: rounds 22 and 23: 43 / 45 (Sonnet 39), held out 16 / 18 (Sonnet 17), every task of one file in round 22 (42 of 42), features 1 then 2 of 3
    (round 17, before: 37 / 45, 14 / 18, 1 of 3; rounds 20–21, the guide without its general rules: 39, measured and
    discarded); the assistant answers 9 of 10 chat questions as required (`eval/run.py chat`).
  - **LIVE since 2026-11-12.** The old graph (7878) and Developer (7879) had stopped with the session that started
    them; the new ones were started on the real ports and the base ingested into the real database (revision 17, 96 s;
    the collections `knowledge` and `nuclear` removed; a copy of the old database is
    `~/.local/share/nuclear-graph/db.bak-2026-11-11`). Checked live: `knowledge ask` 84 of 84, `knowledge check` clean
    ("the graph holds it"), the Developer's 8 tests, a chat answer in Spanish. They ran as background tasks of that
    session and Claude Code stopped both the same night because the machine was critically low on memory.
    **Since 2026-11-13 they are systemd user units (ADR-0106, the item above); what follows is how they ran on
    2026-11-12.** Started again 2026-11-12 (asked by the user), detached from any session: `setsid nohup ~/.local/bin/nuclear-graph
    serve` and `setsid nohup ~/.local/bin/nuclear-developer` (logs in `~/.local/state/nuclearlang/{graph,developer}.log`;
    stop them with `pkill -f 'nuclear-graph serve'` and `pkill -f 'uvicorn app.main:app'`; the editor's *Settings and
    integrations* page starts them too). Both were restarted several times the same day to load the strategies' code (ADR-0105); the database holds the
    base's current revision; `knowledge ask` finds every question and request; the chat answered a question about
    `gen usecase` in Spanish in 40 s. They do not start by themselves after a reboot. With the
    assistant's model loaded the machine had under 1 GB of memory available (15 GB, the browser and the editor open). The second pair used for the measurements (ports 7888 and 7880) is stopped;
    its data folder `~/.local/share/nuclear-graph-next` can be deleted.
  - **Strategies (ADR-0105, asked by the user 2026-11-12: "formula cada estrategia para el agente dependiendo el tipo
    de feature"). DONE.** 31 `strategy:<name>` rows in five families (create, change, fix, verify and explain, beyond):
    how each kind of request is recognised, its plan with the tool of every step, and what it asks of a run
    (`nuclear-developer/data/assets/playbooks/strategies.json`). The selection asks no model (`retrieve.strategy`,
    `POST /v1/strategy`, `nuclear-graph strategy`, MCP `graph_strategy`); the analyst (`analyst.nuc`,
    `POST /developer/analyze`) reads the project, takes the strategy, turns a request in plain words into generator
    calls tried with `--dry-run` (its one model call), gives the pieces the tools find (the declaring files; a rename's
    files rewritten by `Workspace.renamed`) and asks for the facts the request lacks; the Loop (editors 0.46.0) runs the
    analysis as the last step of *Specification*, skips the stages the strategy has no use for, generates from the
    analysis, writes tool-made pieces as they are, wires each use case with `gen usecase` after *Code* and, for a bug,
    repairs the file the failure names and never the test. Measured: `knowledge ask` 62 of 62 requests to their
    strategy (English and Spanish, none among the rows' `asks`); `eval/run.py analyze` 10 of 11 (4 of the 5 that ask the
    model); end to end with `nuclear-loop run` and the local agents only, on a fresh fastapi project: a spec in plain
    words ("each supplier has a name and a phone; we add, list and change them") → `gen resource` from the analysis → 9
    files → build, install, test green; a rename → `Supplier` and the four names built from it, 9 files rewritten by the tool, no model writing → test
    green in 25 s; and a use case with its rule and its route → one piece planned in the component the project has,
    written by the coder, wired by `gen usecase --route` (alias and route), build, install and test green, then stopped
    at *Security* by the local reviewer (see LEFT 9); the tester's own test did not compile and was corrected by hand.
  - **LEFT (the agent as a tool executor).** (1) What the strategies still do by a model or by hand, each a missing
    tool: relations in `gen resource --source db`; a search of a stored resource (refused now: which column is
    searched is a decision); a field added to a resource that exists (the files are kept: `strategy:member` gives its
    declaring files to the coder); tests for a backend REST resource (needs `@mock` in the backend platform); on mobile
    and web what a screen shows of a use case; the Loop writes files and never deletes or moves one (`strategy:remove`
    is by hand; a renamed declaration's file keeps its path); a rename covers declarations, not fields or methods.
    (2) The planner is still asked when the generators wrote everything, twice (no pieces → `plan-again`): two long
    answers of the 2B model, 15 minutes on this machine short of memory; a strategy whose generator calls cover the
    request could skip it, but only a model can tell that nothing is left. (3) The Loop's form in the editor has no
    field for the strategy (the spec file's `- Strategy:` line and `nuclear-loop run` read it). (4) The internet tools
    (`web.py`) wired into the local planner (today only over MCP). (5) The frontier model as manager: `graph_strategy`
    gives it the plan of each kind of request; the Loop's Claude-led plan still pastes context. (6) The feature
    `new-reader` of the evaluation (a field declared twice in its entity, or a declaration written in two files) and
    `function-label` (passes in one round, not in the next). (7) The planner still names a component in the singular
    (the analyst corrects it in a generator call). (8) The evaluation has no watch on a stalled call. (9) The local security reviewer
    stops a backend run that changes data: it reports "missing authorization check" as a high finding on every use
    case that saves (also on generated ones, seen when a rename rewrote them: a rename now runs no security review) and
    proposes `@acl`, which the backend platform does not have yet (`limit:backend-not-yet`); what the platform cannot
    express should be told as a limit, not fail the run. (11) The local tester's test of a use case did not compile
    (it read `NewVendor.id` and never created the row): the run stops at *Code* until the test is corrected. (10) The analyst's model drops a field of a
    list now and then (the count check turns that into a question to the user).

- **Documentation and the RAG always up to date (decided by the user 2026-11-11; the RAG part DONE 2026-11-12 with
  ADR-0104: `knowledge check` in the stop hook, rule 5 of AGENTS.md, workflow.md §6; the watcher of layer 1 is no longer
  needed: the base is built from the tree in a second and the hook refuses a stale one. LEFT: layer 3, the docs; a first
  piece of it since 2026-11-12: a graph test holds every MCP tool to the README and to `docs/ai/mcp.md`).** Why: an ADR-0103 pass
  claimed "documented" and missed CHANGELOGs, READMEs, the skill and three platform CONTEXTs; the user had to ask.
  Scope of "documentation" (the user's words): context docs (`docs/context/`), `/docs`, every `.md` that describes the
  change, everything for the agents (skill `agents/skills/nuclear/SKILL.md`, `spec/brief/`), the website's docs
  (`~/Documents/projects/nuc/react/nuclearlang_website`, `tools/content.mjs`) and the RAG. Plan (approved, three layers):
  1. **Watcher in `nuclear-graph serve`:** polls the source roots (the tree + the website); a changed `.md`,
     `docs.toml`, example `.nuc` or prompt is copied byte for byte into `nuclear-developer/data` (manifest updated) and
     re-ingested by hash after a few seconds of quiet. Test: change a file, find it in a search. It is new Python:
     add it to the nuclear-graph → Nuclear migration item below.
  2. **Stop hook:** `tools/context/check.sh stop` fails while `nuclear-graph knowledge check` reports differences
     (for when the server is not running).
  3. **`tools/context/docs-check.sh`, called by the same hook:** for each repository with changed code, its
     `docs/context/`, its `/docs` or `.md`, its CHANGELOG, the skill/briefs when a command, MCP tool or Loop stage
     changed, the website content rebuilt when a page it publishes changed, the RAG clean. The list written in
     `docs/context/workflow.md`. Test: the hook refuses with a missing doc.
  The skill was copied by `./install.sh` on 2026-11-12; the running graph still holds the base of before (see the
  item above: not live yet).

- **Generators: the CLI and the MCP write the code, the agent only what is not standard (ADR-0103, asked by the user
  2026-11-11, for the three platforms, CLI and MCP complete).** The user had understood that this existed: only
  `dragonfly make` existed (one nearly empty file per kind). Steps, in order (each: tests, docs, this line updated):
  1. Backend HTTP repositories (fastapi + nestjs runtimes and templates, platform.toml, spec/backend.md, the example).
  2. `dragonfly gen component|resource|screen|inspect` (CLI, Rust) over `[generators]` templates in each platform.toml;
     fields from `--fields`, `--from-json`, `--from-spec`; a test that builds a generated resource with every dialect.
  3. MCP tools: `project_inspect`, `generate_*`, `api_inspect`, `web_search` (the graph's `web.py`).
  4. The Loop's *Generate* stage (the plan names resources; the agents write only what is left).
  5. Proved on `~/Documents/projects/nuc/backend/nuclear_rick_and_morty` (its characters spec); docs everywhere.
  State (2026-11-11): steps 1–4 DONE, step 5 proved on a copy. (1) Backend HTTP repositories in fastapi and nestjs.
  (2) `dragonfly gen inspect|component|resource` for every platform (`dragonfly/src/generate.rs`, `gen_command` in
  main.rs, `[[generators.*]]` + `[gen]` in the three platform.toml): fields from `--fields`, `--from-json <file|url>`,
  `--from-spec` (its example answer, its table, its `GET` url); `@key` for snake_case JSON (new in the backend platform:
  pydantic aliases, class-transformer `@Expose`); refuses undeclared types, a name declared elsewhere and a source the
  platform lacks before writing; test `a_generated_resource_compiles_with_every_dialect`; checked live on fresh projects
  with fastapi, nestjs, flutter and react (Rick and Morty characters and episodes: answers equal to the API's,
  pyright/tsc 0, `flutter analyze` clean, the generated tests pass, `vite build`). (3) MCP: `project_inspect`,
  `generate_component|resource|screen`, `api_inspect`, `web_search`, `web_read`, instructions "generate first". (4) The
  Loop's *Generate* stage (before Plan; resources from `## Resources` or `GET <url>` lines; files in the write
  ledger; the planner plans only the rest). (5) `nuclear-loop run` of the user's spec on a copy of
  nuclear_rick_and_morty: Generate wrote 11 files that compile, the planner left 0 pieces, the run failed at the local
  tester's own test (it read `characters.value` and asserted a 409 against the live API). Since editors 0.45.0
  (2026-11-12) the same kind of spec passes end to end on a fresh backend project (`nuclear-loop run`, 2 min 40 s: 11
  generated files, 0 pieces, *Tests first* skipped because every file was generated, build + install + test green). Left: `@mock` for the backend
  platform so `gen` writes a REST resource's tests (then the tester only adds what is specific); the nestjs HTTP
  repository decoding by the model (today the raw JSON: a renamed field reads `undefined` in a use case); `web_search`
  needs SearXNG or a Brave key to be useful (Wikipedia answers noise); the user's project still has the failed run's
  leftovers (`components/character/data/entities/character.nuc`, `test/implements_list_character_feature_test.nuc`):
  `gen` refuses `Character` until they are removed — not touched (the user's project).

- **The AI loop written in Nuclear/Dragonfly (the user, 2026-11-11: "todo se hacía con nuclear-lang y dragonfly, excepto
  las extensiones"):** today only `nuclear-developer` is Nuclear. Not Nuclear: `nuclear-graph` (≈1 500 lines of Python:
  server, store, encryption, keyring, access tokens, ingest, MCP server, CLI, `web.py`) and its tests, the Developer's
  `eval/run.py` (342 lines), `nuclear-loop` (TypeScript, inside nuclear-vscode). What the backend platform lacks to
  write them: an MCP server kind, a command-line kind, guards/`@acl` on routes, library files / hashing / crypto /
  processes, an outgoing HTTP connection. **Decided by the user 2026-11-11: only nuclear-graph** — the missing backend
  platform pieces first (an ADR), then nuclear-graph (server, store, access, ingest, MCP, CLI, the web tools) rewritten
  in Nuclear; `eval/run.py` and `nuclear-loop` stay as they are for now. NOT STARTED.
- **Internet tools for the local agents (asked by the user 2026-11-11):** `nuclear-graph/src/nuclear_graph/web.py`
  works (search: own SearXNG `NUCLEAR_SEARXNG_URL`, Brave with `nuclear-graph web-key brave`, else Wikipedia — DuckDuckGo
  refuses automated clients; `read` of public pages only, local addresses refused, code blocks removed) but it is Python
  and is not wired to the agents: to be written in Nuclear with the plan above.

- **The NuclearLang website rewritten in Nuclear (asked by the user 2026-11-10: Dragonfly web/react, new design, basic
  search, SPA + static text pages; moved to `~/Documents/projects/nuc/react/nuclearlang_website`):** DONE as a project
  (README there): `components/docs` (content repository, GetPage/GetNav/GetHome/Search use cases, SearchScore service,
  Page/Nav/Search/Home managers, Home/Doc/DocGroup/NotFound screens, layout, search box), `public/styles.css` (the
  Chernobyl palette, light theme by preference), `tools/content.mjs` (markdown → JSON with shiki highlighting from
  the 4 source roots, nav, search index, home sample transpiled by `nuclear`, `/static/` plain pages, llms.txt,
  llms-full.txt, sitemap, robots; since ADR-0101 every text, menu and link from `site.toml`, and `/install.sh`,
  `/install.ps1` served at the root). Checked: `nuclear check` 0, tsc 0, its 4 tests, `vite build`, screenshots of home
  (light, dark) and a doc page in headless Brave. The old VitePress `website/` is gone (in the desktop trash with its
  `docs.yml` workflow, 2026-11-10). Left: the deploy (repository
  `website` or `nuclearlang.github.io`, CI that checks out the repos and runs the content build, the SPA 404 fallback).

- **Public repositories (step 1 of "Next", asked by the user 2026-11-10; "sube los repos a github" the same day):**
  decided — organization `nuclearlang`, public from the first push, Apache-2.0, commits and pushes only in the user's
  name (no Co-Authored-By), a fresh history for `dragonfly` (the old `cli/` git had a co-author line and another
  author: kept in a bundle), `docs/aprende/` and `CodeBuilder/` not published, the e-mail out of the CLI credits.
  Plan: `specs/006-public-repositories/plan.md`. **Stage 1 DONE 2026-11-10:** the working tree is the `nuclear`
  repository with `dragonfly/` (was `cli/`), `dragonfly-{mobile,backend,web}/`, `nuclear-{vscode,jetbrains,zed}/`
  (were `editors/{vscode,jetbrains,zed}` + `theme`, `vendor`, `scripts` in nuclear-vscode) inside it; the LSP is
  `crates/nuclear-lsp` (root workspace); `source_roots` = root + `dragonfly` and `dragonfly-*` with dialects/, docs/
  or spec/ (Rust, stage.sh, bundle-server.mjs, the website); docs distributed (guides to `dragonfly/docs/dragonfly`,
  contracts to dragonfly-backend, context per repository) and 100 links rewritten (same repository relative, another
  one its GitHub url; the website maps those back to its pages); Zed's manifest points to the public repository
  (`path = "grammar"`, no nested git); `cargo run -p nuclear-cli` everywhere. Checked: root cargo test (21) + clippy 0,
  dragonfly 26 + clippy 0, nestjs/fastapi/react checks, stdlib-check, 7 spec-checks 0, dragonfly.py 27/0, tasks_app
  build + 27 tests, website (5 repositories, 4 tests, vite build), the three extensions built, `./install.sh`.
  **Stage 2 DONE 2026-11-10:** LICENSE (Apache-2.0, apache.org text) in every repository and runtime package, the
  `license`/`repository` fields (Cargo, package.json, pyproject, pubspec), README / AGENTS.md / CONTRIBUTING.md /
  CHANGELOG.md / .gitignore per repository, CI `.github/workflows/ci.yml` per repository (each job clones nuclear and
  places the repository inside it; none for nuclear-jetbrains: it needs an installed IDE), the organization profile
  (`github-profile/` → repository `.github`), the e-mail out of the CLI credits, commits signed with the account's
  noreply address (repository-local git config; the global one keeps the gmail). **Stage 3 IN PROGRESS:** pushed
  (public) all ten: nuclear, dragonfly, dragonfly-mobile, dragonfly-backend, dragonfly-web, nuclear-vscode,
  nuclear-jetbrains, nuclear-zed (2nd commit pins the grammar rev), .github, website (nuclear, dragonfly-backend
  and nuclear-vscode after nuclearlang-4e's change was green; its docs for `dependencies(...)` and `<project>/defs`
  are still to come, by that session). First CI fixes: a lone clone of nuclear failed 74 tests that read the platform
  dialects or examples — they now skip with a note (`without_platforms` in loader.rs, rules.rs, projects.rs, the
  LSP's server.rs; the flutter brief check); every CI clones the whole working tree; `tools/release/build-runtimes.sh`
  builds the JavaScript runtimes before `install.sh` (CI, release.yml); tools/node's lock had the B1 rename in a
  tarball url (`fastapi-checkpoint`), nuclear-vscode's lock was out of sync. Left: every CI green, the website
  deploy (GitHub Pages, SPA fallback) and a first release tag `v0.1.0` (the installers need it). The CI fixes were
  pushed. First CI round: nuclear, nuclear-vscode, dragonfly, nuclear-zed green; second round pushed (the Python
  tools' `fastapi-checkpoint` → `langgraph-checkpoint`, another B1 rename leftover; Node 26 in every workflow; the
  runtimes built before the platform checks; mobile builds the backend example whose contract tasks_app reads):
  **every CI green 2026-11-10** (nuclear, dragonfly, dragonfly-mobile, dragonfly-backend, dragonfly-web,
  nuclear-vscode, nuclear-zed, website; nuclear-jetbrains has none). `tools/context/check.sh` now skips `.git` folders (every
  repository of the tree has one).
- **Workspace, fourth round (asked by the user 2026-11-06; NOT STARTED, record of the requirements so a fresh session
  picks it up; the state of the code is in ADR-0099 and `docs/context/editors/vscode.md`):**
  1. **The platform actions leave the file tree** — DONE in 0.34.0: a horizontal row above the editor (the editor
     title bar's toolbar, `editor/title` by `nuclear.platform`), flutter first, same model for backend and web. Rule the
     user stated: separate, organize and optimize everything functional (UX/UI), order it for productivity. The run/debug
     actions were exercised live 2026-11-07 (Ctrl+Alt+R/G open the device pick); the toolbar glyphs themselves still want
     a visual check with an open editor.
  2. **Devices like Android Studio** — DONE in 0.35.0 (see the Log): several devices tiled, a run never replaces a
     running one, ▭ Device picks the device, devices only in mobile workspaces (parked elsewhere). VERIFIED in a live
     VS Code 2026-11-07 (nuclearlang-4a, test profile driven over Hyprland, see the Log): 3 devices at once tiled in the
     Devices workspace, independent processes, a new run keeps the running ones, park on tab close (app stays headless,
     listed), unpark on entering the Devices workspace. Left to check live: two devices with two signed-in users (needs
     real app auth), the park/unpark when switching to the python project tab and back (same park/unpark code, blocked in
     the test only by the native folder dialog landing off-screen).
  3. **HTTP showed nothing** because `HttpModel` only listed the projects of the VS Code workspace folders, not the ones
     opened as tabs — FIXED in 0.33.1 (`HttpModel.extraRoots`, `findProjects(extra)`); to check in a live VS Code
     with `nuclear_langgraph_agent` and `rick_and_morty` open as tabs.
  Then: ADR-0099 (decisions 3 and 4), `spec/flutter.md` §11.1, `editors/vscode/README.md`, the context, bump + release.

- **Platforms refactor (ADR-0093, ADR-0094, ADR-0095; asked by the user 2026-11-04):** three platforms (`web`,
  `mobile`, `backend`), dialects grouped under them, the Nuclear of a platform identical across its dialects,
  Nuclear's own standard library, kinds as libraries. Phases A, B1, B2, C and D are DONE (2026-11-04/05); E has its
  first part done (what remains is listed in it). Phase A (rules, ADRs, context, specs) — DONE 2026-11-04:
  `AGENTS.md` rule 11, ADR-0093/0094, `docs/context/{dragonfly,dragonfly-mobile,dragonfly-backend}/`,
  `spec/{backend,mobile,library}.md`, `spec/{dialects,projects,nuclear-base}.md`, headers of
  `spec/{langgraph,nestjs,flutter}.md`, skill, website. Code phases, in order (each one: tests + clippy, specs, STATUS):
  - **B1 — folders, names, manifest — DONE 2026-11-04** (see the Log). Left for B2: `app(version)` is not used by
    the generated manifests yet (they write 0.1.0).
  - **B2 — backend unified — DONE 2026-11-05** (see the Log); the conformance example (ADR-0093 §9) is
    `examples/examples-backend/tasks_api`, built with both dialects by `projects.rs`
    (`backend_example_builds_with_every_backend_dialect`) and checked by `tools/fastapi-check/check.sh` (pyright,
    pytest over its fastapi build) and `tools/nestjs-check/check.sh` — DONE 2026-11-05 with phase C. `dragonfly new
    -p backend` still writes an empty connections list (CLI step: `SQLite(file = ...)` + `plugins(sqlite = true)`).
  - **C — standard library (ADR-0094) — DONE 2026-11-05** (see the Log). Known limits: a property of a value
    without a type is written as it is (`x.length` stays `.length`, wrong in Python until the value is typed); type
    definitions carry no member types, so a value from a library class is "unknown" until typed; `string.length`
    counts characters in Python; `catch e` + `e.message` works in the three languages, but Dart's `NuclearError` is
    the base dialect's only (the platforms map `Error` to their `AppError`).
  - **D — mobile spec split — DONE 2026-11-05 for what names no Flutter class:** `spec/mobile.md` holds the rules
    (§3) and the architecture (§4); `spec/flutter.md` §1 and §4 keep only what Flutter adds. The project structure
    (§3), the kinds (§6) and the tests (§8) stay in `flutter.md` next to their Dart output (the 27 examples
    `tools/spec-check/dragonfly.py` checks) until a second mobile dialect exists (`mobile.md` §5).
  - **E — kind packages (ADR-0095) — FIRST PART DONE 2026-11-05** (see the Log): the package format
    (`kinds/<name>/kind.toml`, `docs.toml`, `<dialect>/kind.toml` with `[plugin]` + the dialect sections), the loader
    (a package of the platform is a plugin; `Dialect.kind_sources`, `package_gaps`, `extra_docs`), N0306, `nuclear
    kind new`, `dragonfly add <folder>`, tests in `loader.rs` and `projects.rs`, `spec/dialects.md` §7,
    `docs/dragonfly/kinds.md`. Pending: the LSP showing a kind's source and listing only the platform's kinds
    (`editors/` session, note in its CONTEXT), `nuclear dialect test` over a package and `dragonfly add <git url>`
    with a pinned version (plan 002 D1, D6), a platform-level list of library extensions in `platform.toml`
    (today a dialect may add `[library.<base>]` members freely).

- **Documentation refactor (2026-11-03, nuclearlang-02, decided by the user):** everything in English
  (ADR-0082); layered context tree (ADR-0084); ADRs in `docs/adr/`; repository split planned (ADR-0083). Order:
  1) context docs cut by future repository — DONE (`AGENTS.md`, `docs/context/`, `docs/adr/`, this file);
  2) translate the user docs to English — DONE (2026-11-03): every `.md` of `spec/`, `docs/` (except
  `docs/aprende/`, kept in Spanish as the user's teaching material), `specs/`, the READMEs, the website menu and
  home page; English file names.
  3) move folders into the repository layout — HELD 2026-11-05, needs the user's go: it moves `editors/lsp` into
  `crates/nuclear-lsp` and regroups `cli/`, `dialects/`, `packages/` and `defs/` by future repository (ADR-0083 amended
  by ADR-0093: `nuclear`, `dragonfly`, `dragonfly-mobile`, `dragonfly-backend`, `dragonfly-web`, the editor
  extensions, the website), which touches the `editors/` and `cli/` subprojects (their own git, other sessions:
  rule 7) and every path in `install.sh`, the check tools, the tests and the context tree. Plan when it starts:
  one move per future repository, `cargo test` + `install.sh` + the check scripts after each, paths updated in
  `docs/context/`.

## Untested (done; needs a manual test)

- The Loop's graph, the chat's exact answers and the Smart Graph in Nuclear (ADR-0102 §23, ADR-0107, editors 0.49.0,
  2026-11-13): the run's page inside a running VS Code (the graph, the drawer of a node, *Run again from this step*,
  *Run <stage> again*, *Run again from scratch* and its confirmation were rendered headless and the engine is tested;
  the retries were run for real only through `nuclear-loop`); the chat's panel with the exact rows under an answer
  (asked over HTTP only: the panel draws headings and lists, not tables); the JetBrains plugin
  (`nuclear-jetbrains/build/nuclearlang-0.49.0.zip`, built, not installed in an IDE); a release archive with the
  graph's `.nuc` sources (`tools/release/stage.sh`) and the set-up's `nuclear build` step on a machine without the
  working tree (here the installed graph is the working tree's, editable); the MCP tools `graph_answer` and
  `developer_retry(from_step, scratch)` called by a real client (their functions are tested).

- ADR-0106 (the set-up of the AI services, editors 0.48.0): the wizard inside a running VS Code (its pages were
  rendered headless; the offer at start-up, *Install it for me* in a terminal, the three-second check, *Try again*);
  macOS (launch agents, Homebrew for Ollama) and Windows (the Startup script, `winget`, `install.ps1 -AiProfile`,
  `nuclear-developer.cmd`): written, never run; a machine without uv or without Ollama (both were here: their
  installers were not run by the set-up); the public one-liner with a real release (only a local archive into a test
  home, and its local profiles only up to the services step, because systemd's user manager is the real user's); a
  login after a restart of the machine (the units are enabled and restart on failure; the keyring is not open when the
  graph starts: since 2026-11-13 the graph waits for it and makes no second secret, ADR-0106 §9, proved by a test
  with a keyring that closes and not by a real reboot); the `lite` profile's download (its model was already here); the
  settings page's Start / Restart against the units.

- ADR-0105 (strategies, editors 0.46.0): the Loop panel in a running VS Code (the *Analysis* step of the
  Specification stage, a stage skipped by a strategy, the questions that stop a run), the JetBrains tool window, and
  the MCP tool `graph_strategy` from an interactive Claude Code / Codex / Cursor (the selection ran over HTTP and the
  CLI only). End to end only on a backend (fastapi) project: no run on a mobile or a web project, none with nestjs,
  none led by Claude. Of the 31 strategies, three ran as whole Loop runs (a stored resource in plain words and a rename, both passed; a use
  case with its route, green up to the tests after its test was corrected by hand, stopped at Security); the bug strategies (`bug-compile`, `bug-failing-test`, `bug-behaviour`,
  `bug-runtime`) ran only as analyses and engine tests, never against a project with a real bug; `data-migration`,
  `configuration`, `document`, `tests`, `security-review` and `operate` only as classifications.
- `dragonfly gen resource` adding an action to a resource that exists: the compile test with the four dialects; not
  the generated code run (pyright, tsc, the routes over HTTP).

- `gen usecase` (2026-11-12): the MCP tool `generate_usecase` from a real MCP client (only its arguments are tested;
  the CLI underneath ran end to end on fresh fastapi and nestjs projects); on mobile and web only the compile test
  ran (no `flutter analyze`, no `tsc` of a react build); the fixed-segment-first route order of nestjs with a
  `@Controller` that Nest scans before `configureApp` (a module that reads its routes at construction).
- The services started detached on 2026-11-12: the editor's Start / Restart buttons against them (the launcher
  `~/.local/bin/nuclear-developer` was written as `install.sh --ai` writes it, `--ai` itself was not run), the Loop and
  the chat panels of the extension 0.46.0 (only `POST /developer/chat` and `nuclear-loop run` were called by hand).

- ADR-0104 in a running editor and from MCP clients: the chat panel and the Loop panel of the extension 0.45.0 (the
  `root` they send, the Plan step's tree and limits), the MCP tools `graph_card` and `developer_code(project)` from
  an interactive Claude Code / Codex / Cursor; `knowledge check` inside the stop hook was run by hand only.

- ADR-0103 in a running VS Code: the Loop panel's Generate stage (only `nuclear-loop` ran it) and the MCP generators
  from an interactive Claude Code / Codex / Cursor (only called from Python).
- The nestjs `ilike` is Sequelize's `Op.iLike` (PostgreSQL only): with MySQL/MariaDB/SQLite it is not case-insensitive
  everywhere yet (only SQLite ran in the checks, with `like`).
- ADR-0102 §§21–22 in a running VS Code: an interrupted run resumed after a real restart, the regular checks' notices,
  the OpenCode MCP entry (its config format `mcp.<name> = { type: local, command }` was not run against OpenCode).
- ADR-0102 §§15–19 in a running VS Code (the notifications, the badge, a run or a chat job going on after a tool switch,
  the step retry buttons, the context meter): the chat page, the settings page (Start/Restart, Pull, the MCP buttons
  against the real CLIs, Allow/Deny), the Retry button and the diagnosis box (only node checks of their scripts and the
  engine's tests ran).
- ADR-0102 §14 (MCP local first): only through `claude -p` with Sonnet; not run from an interactive Claude Code, Codex
  or Cursor, nor with Opus or Haiku.
- ADR-0102 decisions 9–10 (editors 0.39.0, 2026-11-11): the JetBrains *Nuclear Loop* tool window in a running IDE (only
  compiled against Android Studio 2026.2), the panel's *For the agent* tab and the Loop's new stages in a running VS
  Code (only `nuclear-loop` ran them), `install.sh --ai` on a clean machine
  (here the graph and the Developer were already installed), `nuclear-loop` from a release archive (`stage.sh` output
  ran here; `install.ps1` does not link it: Windows gets `nuclear-loop.cmd` in the installation's `bin/`).

- ADR-0102 (the AI loop, editors 0.38.0): the Loop panel inside a running VS Code (only a headless render with the real
  run data and stubbed theme was screenshot: form, levels 1–3), a Jira import against a real site (`jiraIssue`,
  `jiraMine`, ADF), a GitHub issue import (`gh`), an exported CI file run on GitHub/GitLab/Bitbucket, the deploy stage
  with real `seed`/`integration`/`deploy` commands, the MCP server added to an interactive Claude Code / Codex / Cursor
  (it ran over stdio from a Python MCP client and from `claude -p --mcp-config`), the keyring on macOS/Windows (only
  libsecret here). Ran end to end with the engine alone: a local-only run and a Claude-led run (USD 0.08) on fresh
  fastapi projects, every stage passed.

- ADR-0100 §5 migrations against PostgreSQL, MySQL and MariaDB (only SQLite ran: the runtimes' tests, the check
  scripts and both dialects end to end with `dragonfly migrate` on a copy of tasks_api, data kept through a column
  added and a length changed, and reverted); `dragonfly migrate` on Windows (`cmd /C` quoting).

- ADR-0101 installers: `install.sh` on macOS (Intel, Apple silicon, Rosetta detection), `install.ps1` on Windows (no
  PowerShell here), the `release.yml` workflow (never run: needs the public repositories; musl, arm runners, the
  Windows zip), `--version`, the latest-release lookup against GitHub, the VS Code `.vsix` step. Only the Linux x86_64
  archive from a local mirror was run.

- Editors 0.37 in a running VS Code (2026-11-10): the web device panel (only its engine `webSession.ts` was run, by a
  node script against the website: frames, address, console, network, DevTools url, screenshot, iPhone emulation in
  dark mode), ◉ Debug of a backend (debugpy needs the Python extension; node) and of a web app (chrome attach to the
  web device), the `.nuc` breakpoint mirrors under node/chrome/debugpy, build on save for backend/web. Checked by hand
  outside VS Code: `dragonfly run` of nestjs restarts on `dragonfly build` (node --watch), the CLI's workspaces for
  fastapi/nestjs/react, the source-map + line-map chain of nestjs.

- Debugging `.nuc` (ADR-0068, 0076), ▷ Run/Debug, hot reload on save and `dragonfly repair`: never with a real
  device or emulator (VS Code and Android Studio). Whether VS Code honours `task.allowAutomaticTasks: "on"`
  (if it asks, document it in `spec/flutter.md` §11.1). Hot reload in Run mode (`noDebug`).
- The JetBrains plugin loaded in a running IDE (compiled against Android Studio 2026.2).
- Editors 0.28.2 in a running VS Code / Android Studio / Zed (the project card status bar, hover of a kind with its
  source, the quick pick of dialects): only the installed server was driven by hand over JSON-RPC.
- A real phone; macOS.
- Dragonfly devices on macOS and Windows (they compile), GNOME/KDE (floating window over CDP), real sixel
  (foot, Konsole, Windows Terminal ≥ 1.22).
- `dragonfly setup`: real download (~1 GB), unpacking and PATH end to end; dnf/zypper package names are not
  in the official guide.
- The new CLI animation (WELCOME, shine every 5 s, page alive until Enter) and `credits` in the user's
  terminal (only checked with captures).
- The examples in the `docs.toml` files parse (checked once by hand; no test guards it).
- ADR-0087 in Codex, OpenCode and Cursor (only Claude Code was run): that each finds the skill in
  `~/.agents/skills` / `~/.codex/skills` and reads AGENTS.md / the Cursor rule.
- ADR-0086 in `editors/`: the grammars now know the new operators and `r"..."` (editors 0.36.4, all three; the Zed
  tree-sitter parse was checked), but the highlighting was not eyeballed in a running VS Code / Android Studio / Zed.
- ADR-0090 (langgraph) against real services: only SQLite in memory and a fake chat model were run. Not tried:
  Postgres (asyncpg), MySQL (aiomysql), Redis, Kafka (`listener`/`publish` need a broker), OpenAI, Ollama, Chroma,
  `uvicorn app.main:app` by hand, Python 3.12 and 3.13 (only 3.14 installed; the generated code targets 3.12+).
- ADR-0092 (Genkit): Firebase AI, Google AI, Vertex AI, OpenAI's and Anthropic's real services not run (no keys); the
  chat screen on a device; FirebaseAI needs `flutterfire configure` in the app (not automated by the CLI).
- ADR-0091 (nestjs): MySQL/MariaDB/Postgres drivers not run (only SQLite); OpenAI not run (only Ollama).
- ADR-0088 with agents: N0022/N0219/N0220 and `nuclear help words` not measured yet (F7 of plan 001).
- ADR-0085 on a device: an event published from a real workmanager task (another engine; tested only with
  `Isolate.run`), screen transitions as seen on screen, the back button of a nested route on Android.
- ADR-0099 (the Nuclear workspace, editors 0.33.0/0.36.0) in a running VS Code. VERIFIED live 2026-11-07
  (nuclearlang-4a, test profile over Hyprland, see the Log): the strip, the project tab and its platform (`Ctrl+1`),
  `Ctrl+Alt+V/O` and the Run/Debug device picks, the welcome page at startup, several Dragonfly devices running at once
  (up to 3, tiled, independent processes, named "Dragonfly device N" in the debug Call Stack / toolbar / picks and the
  window title), park on tab close and unpark on entering the Devices workspace, a device closed during the Flutter
  build left tracked (not an orphan), and the window-close reaper ending every device app (6 processes → 0 in ~3 s).
  Still to check live: the icon theme on the file tree, the editors swapped per workspace (and a dirty editor
  cancelling the switch), the terminals of build/test/serve and the web commands, the HTTP workspace (tree, component
  tabs, a request sent, restored after a switch, Update of a contract), hot reload of each device and the 30-minute idle
  auto-stop (the watchdog is wired and runs every minute; the 30-minute threshold was not wall-clock tested), the tool
  template pages, and `Ctrl+Alt+<letter>` chords a desktop environment may swallow. 0.36.1–0.36.3 (the device keeping
  its own group beside the code across a workspace round-trip, and the monitor-aware sharp streaming) not re-verified in
  a window. **Open with the user:** after 0.36.3 the user must re-run the device once (a device left running from the previous build is orphaned by
  the extension reload) and say whether it is sharp now; 0.36.3 renders the stream at the panel's exact on-screen
  pixels, so if it is still soft the next suspect is the webview's `devicePixelRatio` or VS Code zoom, not the capture.
- ADR-0098 (contracts): the VS Code "Nuclear HTTP" view in a running editor (tree, environments, a request sent, the
  Update action) and `nuclear contracts update` against a deployed API (`curl`, `/.nuclear/contract.json` served by the
  runtimes: only the files on disk were checked). Only compiled, packaged and installed.
- ADR-0097 (web platform, react): the generated app in a real browser (`npm run dev` in
  `examples/examples-web/characters/build/app`: routing, the detail page, the state builder re-rendering); only jsdom
  tests, tsc and `vite build` ran.
- ADR-0096 (local database) on a real device and on web: the AES key written to the secure storage, the hive files in
  the app's support folder, IndexedDB and WebCrypto on web, data surviving a restart, `LocalDatabase(encrypted =
  false)` writing plain boxes, Linux without libsecret. Only in-memory boxes ran (package tests and the example's
  generated tests).

## To decide with the user

- Quick fix of N0217 (`data.fix` in the diagnostic; the LSP would already apply it): offered, no answer.
- Suggested CLI commands: `g crud`, `g form`, `routes`, `graph`, `rename`, `remove`, `upgrade`, `g i18n`,
  `--dry-run`.
- Diagnostics of closed project files in the editor: only if the user asks.
- Dragonfly: see "Dragonfly: to design" below.

## Pending: Dragonfly mobile (platform, flutter dialect, packages)

### Dragonfly: to design with the user
- **Logs:** done (ADR-0078; `docs/context/dragonfly-mobile/runtime.md`). To decide: log navigation, events
  and use case duration automatically; async errors outside Flutter (no `PlatformDispatcher.onError` today:
  they do not go through `Log`); an own file output (today via `Log.sink`); `@log` on use cases?
- **TDD:** undefined; today there are tests in Nuclear and generated tests. Ideas: widget golden tests
  (`matchesGoldenFile`), accessibility (`meetsGuideline`), performance in integration.
- **Linter:** done (ADR-0046; `docs/context/dragonfly-mobile/project.md`). To decide: all rules are errors
  today — any as warning? disable a rule per file or declaration? new rules (e.g. domain without imports from
  `data/`)?
- What is mandatory in `main.nuc` and mandatory file names (`_usecase`, `_repository`).

### Dragonfly: to do
- **CI red since 2026-11-12: the new Flutter stable (after 3.47.0) deprecates `package:flutter/material.dart` and
  `GlobalMaterialLocalizations` for the `material_ui` package**; `flutter analyze` of `packages/dragonfly` reports 9
  infos (`deprecated_member_use`) and exits 1. Not caused by a change of ours. The right fix is moving the runtime, the
  flutter dialect's auto-imports, its templates and the Dart defs to `material_ui` (decide with the user; hiding it
  with `--no-fatal-infos` would be a shortcut). Local Flutter is older and does not show it.
- Genkit: interrupts (tool confirmations), middleware (retry), the Developer UI from `dragonfly run`, `dragonfly g chat|flow|tools` generators, a persistent Knowledge (today in memory), multimodal parts (images) in chats.
- Editors (ADR-0077, 0083): split the base extension from Dragonfly for Flutter (run/debug/device) in VS Code
  and JetBrains; framework extensions namespaced and activated by the project marker.
- Dragonfly language independent (ADR-0072): `[toolchain]` in the dialect, dialects installable from a repository
  (`specs/002-dialects-with-agents/plan.md`); the neutral spec is phase D of the platforms refactor (Half done).
- Package: `repository` URL in the pubspec and publish on pub.dev; then `dragonfly new` uses the published
  version. `WebSocket` uses `dart:io` (not on web). Splash `onLoad` has no timeout.
- Ideas: `@changed("field")` in forms, plurals in messages and translations, warning for a missing translation
  key, `dragonfly g form`.
- Contracts (ADR-0098) limits: the generated repository names (`TasksRepository`) and models are top-level names (a
  clash with the app's own or between two contracts is N0303; a prefix is a one-line change in `contract::generate`);
  a route answering a generic (`Page<Task>`) is generated but not stored locally; the client error has `message` only;
  a stale url copy is not a warning yet (plan 003 F5); a url contract needs `curl`; a web client gets no rules or key
  until the platform has forms and the local database; the HTTP view resolves `{{baseUrl}}` per project or connection
  only from the environment file (the app's `Http(baseUrl)` is not in the contract yet).
- Local database (ADR-0096) limits: a GET under `saveLocal` whose model has no `@primary` fails at run time (a
  compile-time diagnostic would need the emitter to check `fields_decor` against a dialect rule); `local = false` cannot
  opt a method out of `saveLocal` (templates compare no values); no `isNull` operator (the word is the `flutter_test`
  matcher in the dialect's auto-imports: a `[functions]` shortcut is refused by N0204; a plain `null` in `where`);
  `Page<T>` answers are not saved; `@column` options (`length`, `unique`, `index`) have no document-store meaning; a
  `LocalTable` keeps its box in memory (hive); switching `encrypted` after the first run needs the tables cleared.
- `hide Page` is written on `package:flutter/foundation.dart` (5 `flutter analyze` warnings in clean-flutter-project,
  output unchanged by ADR-0100): `hidden_from` uses every library of a definitions package (`def_libraries`), not the
  library that exports the name; fix it with the symbol's own library + `reexports`.
- Dart dialect: `match` over a typed `Result` writes `(result as Right<E, T>).value`, which `flutter analyze` reports as
  `unnecessary_cast` (6 warnings in rick_and_morty's `character_list_screen`); also 3 infos there
  (`use_build_context_synchronously`, `unintended_html_in_doc_comment`). Not caused by the move.

## Pending: Dragonfly web (platform, react dialect, packages/dragonfly-web)

- Found building the website (2026-11-10): navigation by address inside the app (a link of rendered content reloads
  the page; `goTo` is by screen), the type of a state builder's callback parameter is unknown to the emitter (so a
  library member such as `list.isEmpty` is not translated there: write `list.length == 0`), no way to give an
  attribute whose name is not an identifier (`aria-label`), a deploy story (base path for GitHub Pages, the SPA
  fallback `404.html`).

- Not in the first version (ADR-0097; the same words as mobile): `form`/`formmanager` (react-hook-form + zod
  underneath), `converter`, translations, the local database (`@primary`, `saveLocal`; IndexedDB), plugins and AI
  kinds, a generated tests file, `@splash`/offline.
- Limits: `@acl(guards)` only in a use case that awaits something; a view's arguments must be named; `useManager`
  and `useStateOf` are hooks (only in `build`; `this.manager(X)` elsewhere); `Guards.check(internet)` reads
  `navigator.onLine` only.
- To decide with the user (ADR-0097 analysis): zustand as the engine of `StateManager`, TanStack Query as an
  opt-in per repository method, JSX stays out.
- Thesis A runs (`tools/thesis/run.py --arm both --repeats 3`): paid, waits for the user's go; then `report.py` and
  the numbers published (step 7 of "Next"). Thesis C needs programs that use the framework kinds, not only CPU loops.

## Pending: Dragonfly backend — fastapi dialect (today `langgraph`, packages/dragonfly-py)

- `dragonfly analyze` in a fastapi project: "the fastapi dialect has no toolchain.analyze" (found 2026-11-13 in
  `nuclear-developer`); the `AGENTS.md` that `dragonfly agent init` writes tells every agent to run it after
  `dragonfly check`.

- A repository whose name implies an entity nobody declares compiles (found 2026-11-12 by a rename, ADR-0105): with
  `Supplier` renamed to `Vendor` and `SupplierRepository` left, `nuclear check .` reports nothing and the generated
  Python fails when imported (`NameError: name 'Supplier' is not defined`). The repository kind takes its entity from
  its own name: the compiler should report a `<Name>Repository` without a declared `<Name>` (a diagnostic of the
  platform's contract, both dialects). The rename tool now renames the names built from the old one, so it no longer
  leaves one.
- A map field of a graph's state gets no default (`state_field.tpl` gives one to lists, texts, numbers and booleans;
  the emitter's `tinfo` has no `map` flag): `strategy: <string, string>` in the Developer's `PieceState` is written
  with `= {}` by hand. Generic fix: `tinfo.map` in `emit.rs` and the template, for both backend dialects.

- ADR-0100 limits (2026-11-10): the join table
  name is always `<owner table>_<field>` (no custom name); `@transactional` uses the default database only (nestjs:
  the database of the last repository made, a `shortcut:` comment in `repository.ts`); `Page<T>` members are untyped
  in Nuclear (type definitions carry no member types: `items: Task[] = page.items`); the runtime's pyright run over
  `packages/dragonfly-py/src` shows 4 errors that predate this work (`events.py`, `repository.py` Select typing).
- The user's `~/Documents/projects/nuc/python/nuclear_langgraph_agent` still writes `@generated()` in
  `components/tasks/data/entities/task.nuc`: it is now N0309; the fix is `@primary(generated = true)` (not changed:
  the user's project).

- The `dragonfly` CLI for backend projects has `new`, `build`, `install`, `run`, `test`, `migrate`, `check` and `make`
  (every kind, from the platform's `[layout]` + `[skeletons]`, 2026-11-11); missing: `watch` for backend projects,
  `[toolchain]` for mobile (today `"flutter"` is still hard-coded in `cli/`).
- `@acl`/guards and sessions as in Flutter; logs with call sites (`[call_site]`) for Python.
- `examples/stdlib/` has no Python version of the 10 areas yet; `tools/stdlib-check.sh` runs TypeScript and Dart.
- Publish `dragonfly-py` (PyPI); the generated `pyproject.toml` asks for `dragonfly-py` by name.

## Pending: Dragonfly backend — nestjs dialect (packages/dragonfly-nest)

- Publish `dragonfly-nest` on npm (the generated
  `package.json` asks for it by name).
- Vector stores (Chroma) for LangChain.js, like langgraph's `Chroma` connection.

## Pending: cli/ (dragonfly CLI)

- The mobile path still calls Flutter directly (SDK lookup, `flutter` commands, devices, hot reload): move it to the
  flutter dialect's `[toolchain]` like backend/web (ADR-0101 consequences).
- Generators: `new`/`g c`/`g s` do not seed dictionaries, `Translations` or a locale in the skeleton (the only
  generator gap left for kinds; every kind now has a `make`: 2026-11-09 added `stateful`, `statemanager`/
  `statemodel` standalone, and `guard`/`global`/`page`/`integration` were already there).
- Removing `@splash` leaves the native splash (`flutter_native_splash:remove` needs the yaml); check there is a
  single `@splash`; dark variant.
- Ideas from the tutorial: `g s products product_list` gives route `/product-list` (home already has `/`): show
  `--route` more in the help; the `hello_<c>_usecase.nuc` that `g c` creates is almost always deleted.
- Devices: an own engine without a browser (flutter-pi as a guide) not started; user feedback 2026-10-28: three
  `pub get` calls, a crash on exit, 1.5–2 GB of memory.
- The workspace still sets `dart.previewHotReloadOnSaveWatcher` (may give a double reload; can be removed).

## Pending: editors/ (LSP, VS Code, JetBrains, Zed)

- Format on save: LSP `textDocument/formatting` over `nuclear_syntax::fmt::format` (asked by message to nuclearlang-dc,
  2026-11-10; leave the document unchanged when it returns Err).

- Server: types of chained expressions (`a.b.`, `~dio.get(url).`, `Theme.of(context).textTheme.`), of calls
  (`inject<X>().`, `Text("a").`) and of pattern variables (`Ok(c) -> c.`); substitute generics in shown
  signatures (`onChange(change: Change<State>)` in a `Cubit<int>`); with unsaved changes there are no
  architecture rules nor kind by folder.
- Open requests to `crates/` (optional): (a) `project::build` with the text of open files (`overrides:
  &HashMap<PathBuf, String>`); (b) public `default_kind` and `ProjectCtx::visible`; (c) inferred type of an
  expression; (d) `render_member` substituting generics.
- VS Code listing (`vscode/README.md`): the example goes in a ```` ```nuclear ```` block; a relative image breaks
  locally and `vsce` rejects it without `repository`.
- Binaries for other platforms (macOS, Windows) in the packages.
- Mixed debugging in Android Studio.

## Pending: AI loop (nuclear-graph, nuclear-developer, the Loop tool; ADR-0102)

- A resource that reads another API gets no test (found again 2026-11-13 with the user's run
  `nuclear_rick_and_morty/.nuclear/loop/runs/20261111-implements-list-character-feature`: it passes from scratch with
  10 generated files and **0 tests**; the acceptance step lists "create the unit test" as not shown met, and the
  endpoint's answer was never compared with the API's). The generators write no test for `--source rest`, and *Tests
  first* is skipped when every file was generated: the generator needs a test over a fake `Http` connection (the
  platform has none to give a test), or the tester must write one from the criteria.
- The old ledgers (runs of the Loop before `ENGINE = 2`): a from-scratch restart now removes a file the ledger says
  "was" what an earlier attempt wrote, only when that text is a step's `code` of the run; an older run whose steps
  kept no code still gets such a file back.

- The chat with exact facts (ADR-0107, measured 2026-11-13 with `gemma4:e2b`): the rows under an answer are always
  right and whole, the model's explanation above them is not always (asked for "a repository that calls an endpoint"
  it explained routes; given 27 tools it named six groups). The rows of `query:kind.create` say a generator's
  condition as the generator's own word (`when rest`, `when db`), not as the option that sets it (`--source rest`):
  the generated rows of the generators need the option of each condition. An answer with facts is long (the
  explanation, then every row): the chat's panel could fold the rows.
- The planner runs a query for 12 of the 111 open questions measured (2026-11-13), each about one named thing (a
  diagnostic's code, where a kind's file goes, the example files of a kind). One gets more than it asks: the bare
  command "dragonfly gen usecase" also gets the 9 rows of `query:generators` (its question is within `BESIDE` of
  `query:cli.command`'s). `query:examples` has no row for a repository or an entity of the backend: no example
  project on the shelf declares one (`rick_api` reads an API).
- The Loop's disk check holds the temporary folder to the same 500 MB as the project (2026-11-13: `/tmp`, a 7.7 GB
  tmpfs, was full of other sessions' scratch and of `flutter_tools.*` folders, and every run stopped at *Check the
  connections*): the message now names the folder and the diagnoser has the kind `disk`; nothing cleans the
  temporary folders the tools leave (`nuclear-lsp-backend-*`, `nuclear-web-device-*`).

- What the generators write in their comments says "a order" (`// Changes a order`, `@test("adds a order and finds
  it")`): the article is not chosen by the name's first letter (the three platforms' `[[generators.resource]]`
  templates).

- Weight training (LoRA): what it needs is `specs/007-lora-nuclear-coder/plan.md` (asked by the user 2026-11-11). To
  decide with the user: where the training answers come from (self-distillation validated by the compiler, or Claude's
  answers — check Anthropic's terms on training models with outputs first; the budget approved so far is USD 20), the
  base model's license (`qwen2.5-coder:3b`), and a GPU (none here: a rented one ≈ USD 5–15 per run).
- The local agents after round 17: 37/45 and 14/17 (round 15: 38/45 and 15/17) (Sonnet 39, 17), features 1/3
  (Sonnet 3/3). Left: the local tester writes tests that read an empty database or copy a close example, the planner
  maps a kind only from its own word ("application component" is not `appcomponent`), an agent's `model` field and a
  graph's `state` in the same file are still missed (`eval/results/REPORT.md`).
- The local planner on a spec in plain words (no Dragonfly words: "an author has a name; POST /authors adds one")
  planned two pieces in a component named `data` (`components/data/data/entities/author.nuc` and a test); Sonnet then
  wrote the rest (two MCP runs, 2026-11-11). Specs written with the platform's words plan well (eval rounds 10–11).
- The chat (*Nuclear agent*), the settings page, the notifications and the Retry buttons are in VS Code only; Android
  Studio has the Loop tool window without them.
- The local tester copies a solved graph example close to its task (a models-only spec got an appcomponent test): such
  a local-only run now fails at Tests first with the diagnosis; the tester's retrieval needs examples filtered by what
  the plan declares.

## Pending: nuclear (language and compiler)

- Found while writing `nuclear-graph` in Nuclear (the `python` dialect, 2026-11-13; each cost a wrong program that
  `nuclear check` accepted, or a detour). **Silent**: a top-level value of a file is not visible to the other files of
  a project (it is exported through a function), and an UPPER_CASE name nothing defines is not reported; a member of
  Nuclear's library on a receiver whose type is not known is written as it is (`x.length`) or rewritten as if it were
  a list (`collection.count()` → `sum(1 for ...)`, `replace`; `matrix.sum(axis = 1)` of an `any` → `sum(matrix)`, its
  named argument dropped: found again 2026-11-13), and an untyped map is indexed as it is instead of with `.get`; a constructor parameter is an attribute, so a local of the same name writes the instance's state;
  `xs.findIndex((i) => ...)` writes `for i, i in enumerate`; a spread of a parenthesised expression loses its
  parentheses (`[a, ...(c ? x : y)]`); a local named like a function of another file imports that function (a
  cycle); a typed top-level value (`PORT: int = 1`) is written as a class; a module with only typed attributes that
  extends a library class (`X: BaseModel` + `roots: string[]`) is written as a Protocol; `list` as a method's name
  captures the builtin inside its class. **Not parsed**: `any[] ? a : b` and a map literal inside a ternary; a
  back-tick or `#` inside a text nested in an interpolation (N0022). **Missing**: `assert` (tests use `expect()`),
  a regex that matches across lines (no `dotAll`). Reserved words that read as names: `match`, `use`, `final`.

- `~` before a call that is not asynchronous compiles and fails in Python (found 2026-11-12 with `gen usecase`): a
  use case whose `exec` has no `~` is synchronous, so is its facade alias, and a controller that writes
  `~app.tidyTitle(title)` is emitted `await self.app.tidy_title(title)`: pyright `"Result[AppError, str]" is not
  awaitable`, and at run time `TypeError: 'Result' object can't be awaited` (TypeScript and Dart only await a plain
  value). The compiler knows the callee is synchronous (`Sig.is_async`): the mirror of N0222 is missing, a diagnostic
  with the fix "remove `~`" (new code, `spec/diagnostics.md`, rules.rs). `gen usecase` already writes `~` only when
  the use case awaits; hand-written and agent-written code is not guarded.

- `Name<T>.member(...)` (`Set<int>.of`, `List<int>.filled`) does not parse: a generic type before a member; today
  `s: Set<int> = Set.of(...)`. Assessed 2026-11-08 (nuclearlang-4a): clean support needs an additive AST node for a
  generic receiver (e.g. `ExprKind::GenericName { name, type_args }`, since `Call`'s `type_args` go after the method
  for `obj.m<T>()`), which touches the shared `ast.rs` and every exhaustive `ExprKind` match in `nuclear-transpile`
  and `editors/` (rule 7). Deferred as disproportionate for a niche case with a working annotation workaround; do it
  with the owning sessions.

- Type definitions carry no member types: a value returned by a library method is untyped, so a library member on it
  (`path.length` of `~toolchain.pathOf(...)`) is written as it is (wrong in Python) until the value is typed
  (`path: string = ...`). Found again by the AI loop (2026-11-11).
- `nuclear fmt` limits (2026-11-10): only reflow, it does not add trailing commas or wrap long lines; the two
  external projects (`rick_and_morty`, `nuclear_langgraph_agent`) are not formatted yet (only `--check` was run: 5+
  files would change in rick_and_morty, no errors) — run `nuclear fmt .` there when the user wants.
- Components per principle 6 of the base · `const` inference in
  Dart · TypeScript type definitions (generator from `.d.ts`). Not expressible yet: `docs/context/nuclear/limits.md`.
- Postponed by the user: everything Node.js/Express. Do not touch it until asked.

## Plans not started

- `specs/001-agent-support/plan.md`: F1–F6 done (ADR-0087, ADR-0088). Measured 2026-11-03 (`tools/agent-eval/results/`):
  baseline 10/10 green, 9.8 turns, ~460k tokens, 0.8 red checks, USD 5.10; with skill + AGENTS.md + hook 10/10,
  10.6 turns, ~456k tokens, 0.7 red checks, USD 5.28: no clear gain (agents already copy the project's `.nuc`;
  the tasks are too easy to separate the modes). Not started: F7 (harder tasks — new project from scratch, no
  examples to copy — then tune the brief), now also for Python/langgraph.
- `specs/002-dialects-with-agents/plan.md` (D1–D7).
- Possible later decision (ADR-0086): a portable Nuclear standard library (one name per operation for every
  target), only if the user wants the same `.nuc` to build for TypeScript and Dart.

## Log (newest first)

- 2026-11-13 · nuclear-developer session · **The five findings of the entry below, fixed (asked by the user:
  "arreglalo").** (1) **The graph's search is exact** (ADR-0104 §3): `Store.near` and `Store.measure` compare the
  vector the store keeps of every item a filter matches (numpy over what Chroma reads back); the approximate index is
  not asked. Scan of the 2 618 rows: none unreachable (3, then 1, before); 173 questions in 42 s. (2) **A closed
  keyring is waited for** (ADR-0106 §9, `secrets_store.nuc`): the names the keyring keeps are written down
  (`.keyring`); a secret of a closed keyring is waited for 90 s and then refused (the unit starts again), never made
  again, and a file made while it was closed is not read; test with a keyring that closes
  (`tests/test_secrets.nuc`). This machine's stale `~/.local/share/nuclear-graph/.owner` is no longer read (not
  deleted). (3) **`nuclear agent hook`** builds with the project's own definitions (`agent.rs`, a test). (4) **The
  three open questions**: every MCP tool's row has the generated question "what does <tool> do"
  (`extract_tools.nuc`); `query:examples` answers "an example of a <kind>"; "que hace el mcp" moved from
  `query:mcp.tools` to `concept:mcp`; three more entries in `questions.toml` (173). (5) **The Developer's type
  check**: `nuclear-graph` ships `py.typed`; the `try` template has `catch_used` and the `python` dialect writes
  `except Exception:` when the block does not name the caught value (`emit.rs`, `contract.rs`, `spec/python.md`
  §3.12 with its equivalence): pyright strict over the Developer's `build/`, 0 errors (12 before). **Checked:** root
  `cargo test` 278 + clippy -D warnings, spec-check python 17/0 and fastapi 16/0; nuclear-graph 25 tests,
  basedpyright clean; nuclear-developer 16 tests; `knowledge check` clean, `knowledge ask` 173 of 173;
  `eval/run.py chat` 14 of 14; `./install.sh --no-vscode --no-zed` run (the hook on this project: no error); both
  services restarted. Not done: the editors were not rebuilt (their bundled `python` dialect lacks `catch_used`,
  which only the emitter reads); no `eval/run.py local` round; the keyring was not closed for real (a reboot).
  Found: a library member on an `any` receiver loses its named argument ("Pending: nuclear"). Docs: ADR-0104 §3,
  ADR-0106 §9, `spec/python.md`, the emitter's context, the graph's README, the CHANGELOGs of nuclear, nuclear-graph
  and nuclear-developer, `concept:smart-graph`, `failure:no-access-to-the-graph`. Then pushed (asked by the user:
  "sube los cambios en todos los proyectos que aun no se han actualizado"), with the work of 2026-11-13 the AI-loop
  session had left uncommitted: nuclear, dragonfly-backend, nuclear-graph, nuclear-knowledge, nuclear-developer,
  nuclear-vscode, nuclear-jetbrains, nuclear-zed (the other repositories had nothing pending). Before: the staged
  lines scanned for e-mails, home paths and keys (none), nuclear-vscode `npm test` 33 + tsc, nestjs-check. CI green
  in the four that have one (nuclear, dragonfly-backend, nuclear-vscode, nuclear-zed).
- 2026-11-13 · nuclear-developer session · **The chat answered "que es nuclear lang?" with every command of `nuclear`
  (reported by the user with a screenshot).** Cause: the graph's planner ran a query of the exact facts whenever the
  question wrote a name a query takes ("nuclear" is a tool and a component) and was within the loose `NAMED` distance
  of its exact question; measured, 57 of the 107 open questions of `questions.toml` ran one ("what is the smart graph"
  47 rows, "how does the mcp work" 27). Fixed in `nuclear-graph/planner.nuc`: a query runs only when its exact
  question is closer to what was asked than the closest card of the shelves (ADR-0107 §4); none of the 12 exact
  questions lost its query, 14 of 110 open ones still run one (they name one thing; "Pending: AI loop").
  `questions.toml`: `exact = false` (an open question the planner must leave to the shelves; `knowledge ask` checks
  it), 17 entries, three of them new; asks added to `concept:nuclearlang`, `concept:mcp`, `concept:loop` and
  `failure:a-command-is-not-found`; `concept:smart-graph` says the rule. **Found on the way and fixed:** (1) a card
  brought in by a name was looked up in Chroma's approximate index, which after the ingest no longer reached 3 cards
  of 2 617 (a question of the base stopped finding its failure): `Store.measure` measures it on its stored vector
  (`retrieve.nuc`); (2) a result's `distance` was that of the facet that ranked the card, not the closest by meaning,
  so a question of the card equal to the one asked did not count; (3) the chat's prompt kept the "Exact facts"
  heading and its instructions when there were no rows, and the 2B model ended its answer with those instructions:
  `Workbench.prompt` drops a section all of whose values are empty (nuclear-developer; the prompt files are
  unchanged: a first attempt that moved the instructions to the guide made the model retell the list and was
  reverted). **Checked:** nuclear-graph 24 tests (the planner's case fails without the rule), basedpyright clean;
  nuclear-developer 16 tests; the base at revision 48, `knowledge check` clean, `knowledge ask` 171 of 171;
  `eval/run.py chat` 14 of 14; the user's question in the live chat: the explanation alone, and "que comandos tiene
  nuclear-loop?" still ends with its 7 rows. Both services were restarted to load the code (the graph four times).
  **Found and not fixed** (recorded): the approximate index for a search by meaning ("Pending: AI loop", to decide
  with the user); the keyring at boot ("Half done", ADR-0106 LEFT 5); `nuclear agent hook` without the project's
  definitions ("Pending: nuclear"); the Developer's type check. Docs: ADR-0107 §§4–6, ADR-0102 §15, the graph's
  README, the CHANGELOGs of nuclear-graph and nuclear-developer, `prompts/agents/README.md`. No evaluation round of
  the coder (`eval/run.py local`): its prompts have no section that the new rule drops in a first attempt except
  `repair.md`'s and `faithful.md`'s "Your last answer", which a repair always fills. The graph's own `.venv` still
  held the 0.1.0 install of the deleted `src/` (the end-of-turn hook's `knowledge check` could not import the
  package): `uv sync --extra server --extra test` there (`uv.lock` now says 0.2.0). Nothing committed.
- 2026-11-13 · AI-loop session · **The Loop retries anything and shows its decisions as a graph; the chat answers
  from exact facts; the Smart Graph is written in Nuclear (asked by the user: "la herramienta de loop aun no consigue
  en lo absoluto su objetivo", "una base de datos relacional como sqlite para las consultas deterministas", "si el
  nuclear-smartgraph no esta hecho con nuclear debes migrarlo ya mismo", then "termina lo que falta").** ADR-0107
  (new), ADR-0102 §23, rule 12 of `AGENTS.md` (Nuclear and Dragonfly first). **Switched and installed**: `nuclear-graph`
  0.2.0 is its `.nuc` files (`src/` and the Python tests deleted; `build/nuclear_graph` generated and git-ignored;
  `pyproject.toml` lists it as an artifact, else the wheel is empty; `knowledge examples` replaces
  `check_examples.py`); the exact facts live in memory only (made from the sealed cards: nothing in clear on disk,
  which answers what was left to confirm with the user); the chat calls the planner and ends its answer with the
  rows, put there by code (the model alone gave the 27 MCP tools whole in one run and as six groups in the next);
  editors 0.49.0 packaged and installed (VS Code, Zed; the JetBrains zip built); `stage.sh` and the set-up carry and
  build the graph's sources; `[project] package` of `nuclear.toml`. **Checked**: nuclear-graph 24 tests, basedpyright
  clean; nuclear-developer 15; nuclear-vscode 33; `cargo test -p nuclear-transpile --test projects`; the base at its
  new revision, `knowledge check` and `knowledge examples` clean (30 blocks), `knowledge ask` 168 of 168 (12 with
  `exact = true`); `eval/run.py chat` 14 of 14 after the rows were appended (13 before). **The user's run**
  (`nuclear_rick_and_morty`, `nuclear-loop restart`): passed from scratch after three things it uncovered were fixed:
  `/tmp` full stopped it at the first check (the message now names the folder; the diagnoser has the kind `disk`);
  the undo of the old run brought back a test an earlier attempt had written (`LoopEngine.undo`, a test for it);
  `dragonfly test` ran the compiled test of a source that was gone (nestjs `package_json.tpl`: `dist/test` is removed
  before compiling). It still writes no test: "Pending: AI loop". Lists: "Half done" entry removed; "Untested",
  "Pending: AI loop", "Pending: nuclear" (the compiler's findings of the migration), "Pending: Dragonfly backend —
  fastapi" (`dragonfly analyze`). Nothing committed.
- 2026-11-13 · nuclear-developer session · **The AI services built into the installer (ADR-0106; asked by the user:
  "necesito que esto venga builtin dentro del instalador", then a wizard that reads the hardware and offers lite /
  medium / hard, services at login, Ollama as a detected or installed requirement, remote agents only as a choice).**
  Details, what was checked and what is left in "Half done" (first item) and "Untested". First answered "subiste los
  servicios?" (they were running). Looked for a better runtime than Ollama (web search: llama.cpp's server, LM
  Studio, MLX, vLLM): none for this case, the reasons are in the ADR. Measured candidate models on a side Developer
  (port 7880) and stopped the round when the user asked to push: partial numbers, recorded as LEFT. This machine's
  services are now systemd user units. Found on the way: another session packaged and installed the VS Code extension
  0.47.0 (the file tree's right-click menu) from the same working tree while this work was half written, so that
  package carried it; 0.48.0 replaces it. The machine's `/tmp` (tmpfs, 7.7 GB) was full: it made the graph's tests,
  the website's content build and one cargo test fail until this session's test folders were deleted; other
  sessions' scratch folders (3.9 GB) and Flutter's temporary folders (1.7 GB) are still there. Pushed to the
  repositories (asked by the user: "subelo al repo"): every repository of the tree with pending changes, the work of
  2026-11-11 to 2026-11-13. CI after the push: nuclear, dragonfly-backend, dragonfly-web, nuclear-vscode,
  nuclear-zed and the website green; dragonfly-mobile red as before (the Flutter deprecation, "Pending: Dragonfly
  mobile"); dragonfly failed first on the push order (it cloned nuclear before `map::shape` was there), then on a lint
  of the CI's newer clippy (Rust 1.100, `needless_range_loop` in a test of `device/terminal/render.rs`; the local
  1.99 does not report it): fixed and pushed.

- 2026-11-12 · nuclear-developer session (resumed) · **A strategy for each kind of request (ADR-0105; asked by the
  user: "formula cada estrategia para el agente dependiendo el tipo de feature… el apoyo máximo es en el generador, el
  rag y los tools del agente").** Presented to the user first (the 31 kinds in five families and the plan of each),
  then built. Details in "Half done" → Strategies. Knowledge base: the row type `strategy` (playbooks shelf; `meta`
  family / scope / triggers, `data` signals / target / mode / stages / pieces / generators / needs / lead), 31 rows
  (`assets/playbooks/strategies.json`), `concept:strategies`, two failure rows, six curated rows updated,
  `questions.toml` with 62 requests (`request = true`) and 2 questions. Graph: `retrieve.strategy` (exact signs, then
  meaning; no model), `POST /v1/strategy`, `GraphClient.strategy`, `nuclear-graph strategy`, MCP `graph_strategy` and
  its place first in the server's instructions, `knowledge ask` over requests, `Workspace.renames` / `renamed` (a
  rename as a tool, with the names built from the old one), the prompt `agents/analyze.md`; pytest 32. Developer: the
  analyst (`analyst.nuc`, `POST /developer/analyze`, `Analysis`), the planner knows the project (`PlanRequest.root`:
  what the project has and the spec only uses is not planned again, new pieces go to its component); 13 Nuclear
  tests, pyright strict 0; `eval/run.py analyze` (11 cases) 10 of 11; `eval/run.py chat` 10 of 10 (9 before); the 45 specs of the
  evaluation's tasks and features given to the analyst: every one keeps the whole write pipeline, none is asked a
  question and none gets a generator call (a request that introduces a declaration with its kind's word is already
  designed: the planner splits it). Loop (editors 0.46.0, the three packaged, VS Code
  installed): the analysis step, stages by strategy, generator calls from the analysis, tool-made pieces, `wire()`
  after Code, a bug's repair in the file the failure names and never in its test, `- Strategy:` in a spec; `npm
  test` 20. Generator (found on the way): `dragonfly gen resource` with a new action on a resource that exists left a
  use case calling a repository method nobody had (N0116): the repository and the controller are merged now
  (`merge = true` in the three platform.toml), and `--source db --actions search` is refused; dragonfly 38 tests +
  clippy. **Found by the real runs and fixed:** a model asked about a request that lists no field invents them (it
  is asked only when the request lists some, and held to that count); an entity renamed alone compiles and fails when
  it runs, because a repository is bound to its entity by its name (the rename takes the names built from the old
  one); a use case's request was read as a repository's (a use case names its repository); the planner planned again
  the entity and the repository the project had, in a component of its own. **Found and not fixed** (recorded):
  "Pending: Dragonfly backend" (a repository of an entity nobody declares compiles; a map field of a state has no
  default), "Pending: AI loop" ("a order" in generated comments), LEFT 2, 9, 10, 11. Docs: ADR-0105 + both indexes,
  docs/ai/loop.md (the strategy of a request), docs/ai/mcp.md, the generators' guide, spec/{backend,mobile,web}.md,
  the READMEs of nuclear-graph, nuclear-developer, nuclear-knowledge and nuclear-vscode, the editors' context, the
  skill (copied by install.sh), ten CHANGELOGs, the website's content (168 pages). Checked: root cargo test 276 +
  clippy -D warnings; the three real `nuclear-loop run` (Half done). The graph and the Developer run detached on
  7878 / 7879 with the base ingested; `knowledge check` clean. Not done: no `eval/run.py local` round (the coder's,
  the tester's and the planner's prompts and the shelves they recall from did not change; the planner's new
  post-processing only acts in a project that already has declarations, which the evaluation's fresh projects do
  not); fastapi-check and nestjs-check not run again (the runtimes did not change).

- 2026-11-12 · installer check · Website published at https://nuclearlang.github.io/ (repo renamed `nuclearlang.github.io`, `deploy` job in its ci.yml, SPA `404.html`); `/install.sh` and `/install.ps1` are served (200). Checked `tools/release/install.sh` with a local mirror of `package.sh` output into a fake HOME: installs, `dragonfly new -p backend` + `nuclear check` 0, `--uninstall` clean. Fixed: with no release yet, the latest-tag lookup took `releases` as the tag (404 on a nonsense url); now "could not find the latest release". The `install.sh` fix is committed and pushed (nuclear `main`) and the website redeployed (the served file has it). **The public one-liner still fails: no release exists in `nuclearlang/nuclear`; it needs the tag `v0.1.0` (release.yml, never run).**

- 2026-11-12 · nuclear-developer session (resumed) · The generators and the MCP documented as pages, and the services
  started (asked by the user: "documenta los nuevos generadores… hay documentación del mcp en el rag y en la página
  web? levanta los servicios"). Found: the knowledge base had a row per MCP tool (25 + `mcp:server`, `concept:mcp`,
  `playbook:connect-an-mcp-client`), the website only a paragraph in the Loop page and no list of the tools; the
  generators had sections in the specs and the guide, no page of their own. New: `dragonfly/docs/dragonfly/generators.md`
  (every option, what each platform gets, a use case written and wired with real output, the refusals, the MCP tool of
  each command, what is not generated yet) and `docs/ai/mcp.md` (set-up per client, the order a model works in, every
  tool by group, failures); linked from the Dragonfly guide, the Loop page and the graph's README (whose table lacked
  `developer_retry`); the website's AI menu (`site.toml`), content rebuilt (167 pages). A graph test holds every MCP
  tool to the README and to the page. Knowledge base revision 22 (2 450 rows: 22 rows of the two pages; `concept:mcp`
  and `concept:generators` have them as sources; 14 rows reviewed; 4 questions added, 88 of 88 find their row).
  Services: the graph (7878) and the Developer (7879) run detached, the base ingested, `knowledge check` clean ("the
  graph holds it"), one chat question answered (see "Half done" → LIVE for the commands and the memory left). In a
  shell with the Developer's venv first in the PATH `nuclear-graph` is that venv's client-only script (no Chroma):
  the server is `~/.local/bin/nuclear-graph`. Checked: graph pytest 30, the examples' blocks, the generators' examples
  run on a fresh fastapi project (`nuclear check` 16 files, 0). CHANGELOGs of nuclear, dragonfly, nuclear-graph and the
  website. Not done: the editors were not rebuilt (nothing of theirs changed), no evaluation round (no agent or
  prompt changed).

- 2026-11-12 · nuclear-developer session (resumed) · `dragonfly gen usecase` (asked by the user: "retoma la última
  sesión"; the first of what was left that needs no model running, the graph and the Developer being stopped).
  `dragonfly gen usecase <component> <Name> [--calls <Repository>.<method>] [--route "POST orders/{id}/cancel"]`:
  a use case the project declares is wired (its alias in the facade; backend: its route in the controller of the
  route's prefix, made when missing, called with `~` only when `exec` awaits), its signature read from the project
  (`nuclear_transpile::map::shape`, new); `--calls` writes a use case that only calls a repository method (an answer
  that is no `Result` inside an `Ok`: backend); refused: a use case neither declared nor given `--calls`, a `{name}`
  of the route the use case does not take, a use case of another component, `--route` on mobile and web.
  `[[generators.usecase]]` in the three platform.toml; a `merge` adds whole members (a route keeps its decorator and
  body: a bare `@get` the file already had no longer loses the new route's), and a name declared elsewhere stops a new
  `merge` file too. MCP `generate_usecase` + instructions. **Found and fixed on the way (it was there before): the
  order of a controller's routes decided which one answered** — fastapi added them by method name, so `GET
  <prefix>/search` of every generated REST resource answered 400 (taken by `{id}`); nestjs in the order written. Both
  runtimes now add a fixed segment before a parameter (`dragonfly-py` `router_of`, `dragonfly-nest` `configureApp` →
  `fixedRoutesFirst`; `spec/backend.md` §2.7; each runtime has a test that fails without it; `@nestjs/platform-express`
  is a devDependency of dragonfly-nest for that test). Knowledge base revision 20 (2 427 rows: `tool:dragonfly.gen.usecase`,
  `mcp:generate_usecase`, `generator:<platform>.usecase`; eleven curated rows updated, fifteen reviewed; `extract.py`
  gives every `gen` command of the help its row instead of a fixed list), `knowledge check` clean. Docs: ADR-0103,
  spec/dialects.md, spec/{backend,mobile,web}.md, the CLI's usage, dragonfly README + guide + tutorial + cli context,
  the skill (copied by install.sh), nuclear-graph README, six CHANGELOGs, the website's content rebuilt. Checked: root
  cargo test 276 + clippy -D warnings, dragonfly 38 + clippy, fastapi-check and nestjs-check, dragonfly-py 6,
  dragonfly-nest 6, graph pytest 29 + 38 example blocks, and on fresh fastapi and nestjs projects with the built CLI:
  pyright strict 0 / tsc strict 0, the generated tests, and the routes over HTTP (a fixed route after `{id}`, a
  synchronous use case, an awaiting one with a path parameter, a paged query). `./install.sh --no-vscode --no-zed`
  run. Not done: the editors were not rebuilt (their bundled dialects lack `[[generators.usecase]]`, which only the
  CLI reads); `nuclear-graph ingest` (the graph is stopped); no evaluation round (no agent or prompt changed). Left
  and found: "Half done" → LEFT, "Untested", "Pending: nuclear" (`~` before a synchronous call).

- 2026-11-12 · nuclear-developer session · The Smart Graph redesigned as a library and the local agent tuned on it
  (asked by the user; ADR-0104; details and what is left in "Half done"). Graph: shelves, rows with ids / keys / asks /
  links / sources, hybrid search, `knowledge build | check | review | ask`, `/v1/cards`, `/v1/applies`, `/v1/kinds`,
  `workspace.py`, 29 pytest tests + `tests/check_examples.py` (38 Nuclear blocks compile). Knowledge base: 2 414 rows
  on 9 shelves, revision 17, 84 of 84 questions find their row. Developer: recall by kind, every pitfall of the kind,
  the error's card at repair, the faithful check, the project's files as context, the plan's tree and limits, spoken
  kinds, the assistant over every shelf; 8 Nuclear tests, pyright strict 0. Loop: `root` in every call, limits in the
  Plan step, *Tests first* skipped when all was generated; `npm test` 16; editors 0.45.0 packaged and installed
  (quit VS Code completely to load it; Android Studio: install `nuclear-jetbrains/build/nuclearlang-0.45.0.zip`).
  Evaluation rounds 18–23: 43 / 45 against Sonnet's 39 (37 before), held out 16 / 18, features 2 of 3; chat 9 of 10.
  End to end: `nuclear-loop run` of a Rick and Morty spec (`GET <url>`) on a fresh backend project passed in 2 min 40 s.
  Docs: ADR-0104 + both indexes, AGENTS.md rule 5 and map, workflow.md §6, docs/ai/loop.md, the READMEs and
  CHANGELOGs of nuclear-graph, nuclear-developer, nuclear-knowledge, nuclear-vscode, tools/context. The stop hook runs
  `knowledge check`. Live on the real ports and the real database since the same day (revision 17).

- 2026-11-11 · nuclear-developer session · Report for the user: the steps of a Loop run for a feature, from
  `docs/ai/loop.md` checked against the `STAGES` of `nuclear-vscode/src/loopEngine.ts`, and what of it runs on the
  machine (the models and the embeddings on Ollama, the graph and the Developer on 127.0.0.1). Nothing changed.

- 2026-11-11 · nuclear-developer session · Report for the user: what the Nuclear Developer does, from its README,
  CHANGELOG and AGENTS.md checked against the controller and the config. Nothing changed. Stale in its docs (not
  fixed, not asked): README "Known limits" still lists the fixed 8192 context window (the config reads
  `CONTEXT_TOKENS`) and the framework names in the kinds' words (now under `[underneath]`); README round 3 says
  28 / 26 and 10 / 11 where `eval/results/REPORT.md` says 28 / 25 and 10 / 10; AGENTS.md and `config.nuc` name four
  agents of the seven; the CHANGELOG stops at round 15 (the README has round 17).

- 2026-11-11 · nuclearlang (AI loop session) · Pushed every repository and created the missing ones (asked by the user:
  "sube al repositorio y crea los repositorios que no existan"): nuclear-graph, nuclear-developer and nuclear-knowledge
  (`nuclear-developer/data`) created public under `nuclearlang` with LICENSE (Apache-2.0), CONTRIBUTING, CHANGELOG and
  the noreply identity (the two had the global gmail); the org profile lists them; `nuclear-graph/image.png` (a
  screenshot of the user's) kept out by `.git/info/exclude`. Before: root cargo test, dragonfly 36, graph pytest 20,
  nuclear-vscode 15 green; staged files scanned for e-mails, home paths and keys (none). Left: CI for the three new
  repositories (none yet: nuclear-developer and the graph's tests need the whole tree and a running graph).
  CI after the push: nuclear, dragonfly, dragonfly-backend failed only on push order (a CI cloned a sibling before it
  was pushed) and passed on re-run; dragonfly-web, nuclear-vscode, nuclear-zed, website green. dragonfly-mobile fails:
  see Pending: Dragonfly mobile (Flutter stable deprecated `package:flutter/material.dart`).

- 2026-11-11 · nuclearlang (AI loop session) · ADR-0103 docs the earlier pass missed (asked by the user): `dragonfly gen`
  in the skill (`agents/skills/nuclear/SKILL.md`, step 1), `dragonfly/README.md`, `dragonfly/docs/dragonfly/README.md`
  (Generators section), the Loop's Generate stage in `nuclear-vscode/README.md`, the `http` modules in the backend
  CONTEXT, `[[generators]]` in the mobile and web CONTEXT, a line in the CHANGELOGs of nuclear, dragonfly,
  dragonfly-{backend,mobile,web} and nuclear-vscode. Left (stopped by the user): rebuild + ingest the knowledge base
  (`nuclear-graph knowledge check` lists 4 changed files) and copy the skill to `~/.claude|.agents|.codex/skills`.

- 2026-11-11 · nuclearlang (AI loop session) · ADR-0103 steps 2–5 (asked by the user: the CLI and the MCP generate
  most of the code on the three platforms). `dragonfly gen` (see Half done for what it does and where), the backend
  `@key` decorator (fastapi, nestjs docs.toml), `alias.rs`: a facade alias named like its use case's field gets the
  field `<name>Instance`, MCP tools + instructions (`nuclear-graph/src/nuclear_graph/mcp_server.py`, tests in
  test_mcp_loop.py), the Loop's Generate stage (`loopEngine.ts`: `specResources`, `nucFiles`, `generateStage`,
  `generatedNote`, `withoutGenerated`; 2 node tests), editors 0.44.0 (the three packaged and installed), the knowledge
  base rebuilt and ingested. Docs: ADR-0103 (decisions refined, limits), spec/dialects.md (`[[generators]]`, `[gen]`),
  spec/{backend,mobile,web}.md (Generators), spec/flutter.md §11, the tutorial, the CLI's USAGE, cli CONTEXT,
  docs/ai/loop.md, nuclear-graph README, vscode context. Checked: root cargo test + clippy, dragonfly 36 + clippy,
  fastapi-check, nestjs-check, spec-checks fastapi/nestjs 16/0, nuclear-vscode `npm test` 15, graph pytest 20, live
  runs (above).

- 2026-11-11 · nuclearlang (AI loop session) · Answered the user (why the agents do not generate with the CLI; can the MCP
  generate most of a feature): no — `dragonfly make` writes one-kind skeletons without fields or wiring (backend and web
  from `[skeletons]`, mobile from scaffold.rs), the MCP only passes `dragonfly make` through, there is no field-aware,
  wired generator (Phoenix `phx.gen`, Laravel `make:model -mcr`, Symfony `make:crud`), and the backend has no REST
  repository. Proposed `dragonfly gen` (data-driven by each platform, exposed over MCP, a Generate stage in the Loop,
  the agent writing only custom use cases and UI); waiting for the user's choices. Nothing changed.

- 2026-11-11 · nuclearlang (AI loop session) · The user's resumed run (nuclear_rick_and_morty, "Implements list character
  feature") kept failing at Code with N0220 `CharactersAppComponent`. Causes: (1) the backend platform has no outgoing
  HTTP connection, so a backend that calls the Rick and Morty API cannot be written in Nuclear by anyone (already in
  "Half done": the platform pieces before migrating nuclear-graph); (2) the local planner split the plain-words spec into
  one piece (an entity) and the tester (written before the plan-names rule) injected a class the plan never had, so
  every repair went in circles. Fixed: *Tests first* stops the run when a test injects a class neither the plan nor the
  project declares, naming it and the plan; the diagnoser gets "X is not in the plan" for an N0220 of an unplanned class
  and classifies it as `spec` (before `code`), with the action "name every piece in the spec with the platform's
  words, or let Claude lead, then restart from Plan". Found on the way: the knowledge base inside nuclear-developer made
  its project fail (the copied `.nuc` examples were compiled with it): a folder that is a repository of its own is no
  longer part of a project (`project::sources`, test `a_nested_repository_is_not_part_of_the_project`). Checked: cargo
  test + clippy, Developer `dragonfly test` 5 + pyright 0, nuclear-vscode `npm test` 13. Editors 0.43.1.

- 2026-11-11 · nuclearlang (AI loop session) · Fail-safe runs, checked connections, every integration in the settings
  (asked by the user: VS Code was restarted and a run stayed "running" for ever with nothing working on it). `loopSafety.ts`
  + the engine: atomic saves, an append-only trace log per run, an owner heartbeat, a ledger of written files undone on
  recovery, `recoverRun` + resume from the failed step (on activation, `nuclear.ai.resumeInterrupted`; `nuclear-loop
  retry`; `nuclear-loop runs` marks interrupted runs), a lock (a run a live process owns is not started twice), a watch on
  every call (cut and retried on `callTimeoutMinutes`, or `stallSeconds` without the Developer or with Ollama holding no
  model), a time limit on commands (the process group killed), a preflight step before every run (the Developer, the
  graph, Ollama, the models installed, Claude when it leads, free disk). `connections.ts`: every connection (also
  Codex, OpenCode, GitHub, Jira) checked every `checkEveryMinutes` with a notification on a change, and before each chat
  question; a chat question left unanswered is marked interrupted with Retry. Settings with defaults for every
  integration (`nuclear.ai.claude.*`, `codex.*`, `opencode.*`, `cursor.mcpFile`, `github.command`, `jira.*`, `watch.*`,
  `resumeInterrupted`, `checkEveryMinutes`), the settings page shows the live checks and every field, OpenCode added to
  the MCP list, Jira's site and email are settings (only the token is a secret). Tests: nuclear-vscode `npm test` 12
  (recovery with the half-written file undone then resumed, a stalled call cut and retried, the preflight, a hanging
  command killed). Docs: ADR-0102 §§21–22, the vscode README and context, docs/ai/loop.md, the Developer's README.

- 2026-11-11 · nuclearlang (AI loop session) · The knowledge base as its own repository (asked by the user: the chat
  answered that it knew nothing of the MCP). Cause: the agents' collection admitted only the language and platform
  specs, examples and words, never the tools' docs (MCP, Loop, CLI, editors). `nuclear-developer/data` (`nuclear-knowledge`,
  git init, the root's noreply identity, no commit; ignored by nuclear-developer): `sections.toml` (17 sections in
  reading order, `agents = false` for the target languages, the dialects underneath, the runtimes and the ADRs; the
  website's checkout), 530 files copied byte for byte, `INDEX.md`, `manifest.json` (source, repository, section, size,
  SHA-256), README, LICENSE. `nuclear-graph knowledge build|check` (`knowledge.py`: the tree and the website, the help of
  `nuclear`, `dragonfly`, `nuclear-loop`, `nuclear-graph` and `nuclear help words|layout --platform`; `docs/aprende` and
  CodeBuilder never copied); `ingest` of a folder with a manifest replaces the whole knowledge and takes each file's
  agents flag from its section. The agents' collection went from 890 to 1917 chunks; the chat now explains the MCP
  (Model Context Protocol, said in nuclear-graph's README and docs/ai/loop.md: the model had invented an expansion).
  Tests: `test_knowledge.py` (a build copies everything and a changed comma is found; the sections decide what the
  agents read; this tree's base is complete). Docs: both READMEs, the base's README, docs/ai/loop.md, AGENTS.md,
  ADR-0102 §20, install.sh. Checked: graph pytest 18.

- 2026-11-11 · nuclearlang (AI loop session) · Answered the user: the Smart Graph is populated (from the tree's sources, the
  prompts and the distilled examples), there is no separate seed dataset; nothing changed.

- 2026-11-11 · nuclearlang (AI loop session) · Report for the user: where the Smart Graph's content comes from (sources,
  prompts, examples, data folder); found 7 `review-*` examples from a test MCP run in the `examples` collection.

- 2026-11-11 · nuclearlang (AI loop session) · The five compiler gaps fixed without changing a rule of the language
  (asked by the user). (1) A call to a function of the type definitions is typed by its `returns` (`def_function_ret`:
  `env("X").toInt()` resolves; the Developer's context window is `CONTEXT_TOKENS` again). (2) N0222 also when the
  un-awaited call is kept in a variable and then used (`found = books.all()`, `found.length`). (3) N0102 for a call to a
  method hidden by a local of the same name (`prompt(...)` inside `write(prompt: string)`); a parameter named like its
  own method that does not call it (`page(page: int)`) stays fine. (4) Python `??` with a call or a map read on the
  left: the optional operator template `coalesce_once` (contract) with an operator `helper` (now written for operators
  too), Python `_coalesce(value, lambda: fallback)` — evaluated once, the right side still lazy, pyright strict 0;
  `diagnoser.nuc` writes `??` again. (5) `docs.toml` entries are Nuclear only: what a dialect writes underneath a word
  moved to `[underneath.<section>]` (fastapi, nestjs, flutter, react: SQLAlchemy, TypeORM, Sequelize, pydantic,
  LangChain/LangGraph, pytest, node:test, Cubit, bloc_test, dio, flutter_native_splash...), shown by `Dialect::doc` as
  **Underneath (dialect):**, left out of the graph's agent collection (`words`, `nuclear_only`); "Full rules" point to
  the platform; spec/backend.md no longer names TypeORM. Found on the way and fixed: the nestjs runtime exported
  `iLike` (the platform's `ilike` did not compile with nestjs) and `orderBy(field, desc = true)` reached it as an object
  for a boolean (`SortOrder`); its docs and spec/nestjs.md said `iLike` / `orderBy(field, true)`; defs regenerated.
  Tests: projects `a_type_argument_must_name_a_class` kept, rules N0222 (kept), `a_local_does_not_hide_a_called_method`,
  graph `test_the_agents_never_read_what_a_dialect_writes_underneath`, the spec/python.md `??` equivalence (16/0).
  Docs: spec/{dialects,diagnostics,definitions,python}.md, dragonfly-backend/spec/{nestjs,backend}.md. Checked: cargo
  test + clippy -D warnings, dragonfly 31, spec-checks nestjs/fastapi 15/0, python 16/0, typescript 29/0, dart 30/0,
  nestjs-check and fastapi-check green, dragonfly-nest tests 5, graph pytest 15, Developer `dragonfly test` 5 + pyright
  strict 0, the graph re-ingested (no framework word left in the agents' answers sampled). Editors 0.42.1. Eval round 17
  (the reworded words): 37/45, held out 14/17 (one less: `entity-reader`, the 3B coder put `@column` after its field and
  did not repair it from N0013); round 16 void (stopped at 10 minutes).

- 2026-11-11 · nuclearlang (AI loop session) · Report of the pending work for the user (from this file); the AI loop's
  eval line updated to round 15.

- 2026-11-11 · nuclearlang (AI loop session) · The pipeline failed in the user's VS Code at Code (`nuclear: command not
  found`: an editor started from the desktop has no `~/.local/bin` in its PATH; the pipeline had only been run from a
  terminal). Fixed: every pipeline command runs with `toolEnv()` (the installers' bin folders added; also `claude`,
  `dragonfly explain`, the settings page's services, `loopCli`); reproduced and checked with `env -i PATH=/usr/bin:/bin`.
  The same run exposed: `inject<Undeclared>()` compiled (now N0220 for a call's type argument, generic parameters
  exempt; test `a_type_argument_must_name_a_class`); the diagnoser read pytest's raw log (now the `dragonfly explain`
  text, and its prompt forbids the generated code); a piece's errors carried the scratch file's path (now by line); the
  tester copied a solved example with `inject` for a models-only spec (it now gets only the plan's names, also when it
  repairs a test file, "nothing to inject" when there is nothing, a search without `inject`); *Tests first* that cannot
  write a compiling file now fails the run (it passed empty before). Asked by the user, all in VS Code (editors
  0.42.0): retry from the step that failed (`start(from, step)`, `kept()`; the run's and the failed step's *↻ Retry*,
  *Restart <stage>*, `nuclear-loop retry [--from]`, MCP `developer_retry`, the chat); Loop runs and chat jobs belong to
  the extension (module-level `engines`, `jobs`), so switching tools leaves them working; notifications
  (`notifications.ts`: toast with *Open* to the run/session, a badge on the Nuclear icon, counts on the tool tabs, the
  list in the tool's pane); the chat's context meter (`nuclear.ai.contextTokens`, the assistant's `context` trace), *↻
  Retry* of a failed question, ↑ / ↓ through the questions asked, what the agent did folded in grey with its time and
  tokens. Docs: the Developer's README (where it works, documented runs, failures, Nuclear only, known limits, rounds
  12–15) and CHANGELOG (new), ADR-0102 §19, docs/ai/loop.md, the vscode README and context. Checked: cargo test +
  clippy -D warnings (root and dragonfly/ 31), nuclear-vscode `npm test` (8 node:test tests) + tsc 0, the chat page
  driven by a node check (folded steps, meter, ↑/↓, retry link), Developer `dragonfly test` 5 + pyright strict 0, graph
  pytest 14, the pipeline end to end with a desktop PATH (now fails honestly at Tests first with a Nuclear diagnosis);
  eval round 15: 38/45, held out 15/17. `/tmp` filled up (my scratch projects; deleted with the user's go — 3.2 GB of
  another session and 1.8 GB of Flutter temp folders were left).

- 2026-11-11 · nuclearlang (AI loop session) · (1) `dragonfly new` (the user's `dragonfly new nuclear_rick_and_morty-p
  backend -d nestjs` answered "'nestjs' is a dialect of the backend platform, not of 'mobile'"): a dialect alone gives
  its platform, a stray word is refused with a hint (`did you mean -p backend?`), an option glued to the name is
  refused, names are checked; the backend "Next" lines use the dragonfly commands; the landing says "Mobile, web and
  backend" (it said "Flutter apps"); test `new_reads_the_platform_from_the_dialect_and_refuses_typos`; cli context and
  spec/flutter.md §11. (2) Failures (asked by the user): calls to the local agents retried with exponential backoff and
  full jitter (`backoff`, `transientStatus`, `retryAfter`, `config.retry`; also the chat), the local diagnoser
  (`diagnoser.nuc`, `POST /developer/diagnose`, rules + `nuclear check`/`dragonfly doctor` + prompt `agents/diagnose.md`,
  Nuclear test of the rules), `run.diagnosis` on the run, the journal and the chat, retry from the failed stage
  (`LoopEngine.start(from)`, `run.previous`, `restoreWritten`; panel *↻ Retry from*, chat, `nuclear-loop retry`, MCP
  `developer_retry`). (3) Settings (asked by the user): `nuclear.ai.*` settings and the *Settings and integrations* page
  (`settingsPanel.ts`, `aiSettings.ts`, `localModels.ts`: services start/restart with the models, Ollama models under
  the 3B rule with pull, MCP in Claude Code/Codex/Cursor, graph clients and requests, Jira, GitHub, Claude's model,
  retries); `readConfig(root, platform, machine)`. Editors 0.41.0 (all three packaged, VS Code installed); nuclear-vscode
  `npm test` (5 node:test tests: backoff, retry against a fake Developer, resume from a stage, chat helpers, the 3B
  rule). Docs: ADR-0102 §§17–18, docs/ai/loop.md, both READMEs, the vscode context. Checked: dragonfly cargo test 31 +
  clippy -D warnings, Developer `dragonfly test` 5 + pyright strict 0, graph pytest 14, tsc, `nuclear-loop retry` of a
  failed run (resumed at Code, diagnosed as `code` with the compiler's error), the settings page rendered from a real
  state by a node check.

- 2026-11-11 · nuclearlang (AI loop session) · (1) The local agents' prompts are files (asked by the user: "no se
  entiende nada de una sola línea"): `nuclear-graph/src/nuclear_graph/prompts/agents/` (piece, repair, test, plan,
  plan-again, security, document, chat + README), ingested as `agent.<step>`; `Workbench.prompt(name, values)` fills
  `{{name}}` and drops an empty section; the agents hold no prompt text and say which files they use; the workbench is
  grouped (graph, prompts, models, trace). (2) A chat with the local agents (asked by the user): VS Code tool *Nuclear
  agent* (`Ctrl+Alt+A`, `chatPanel.ts`): questions → the new local assistant (`assistant.nuc`, `POST /developer/chat`,
  `ChatRequest`, prompts `chat.md` + `agents/chat.md`), `/task` → a Loop run of the local agents with its journal,
  templates from the project's map, sessions in `.nuclear/chat/`. (3) Answered the user: nuclear-graph is Python (see
  Half done). Docs: ADR-0102 §§15–16, docs/ai/loop.md, the Developer's README, nuclear-vscode README + context.
  Checked: Developer `dragonfly test` 4 + pyright strict 0, `nuclear check` 0, tsc, the chat's templates/history and its
  page script and markdown by a node check. Eval round 12 (29/45) is void: `nuclear-graph ingest` run from inside
  `nuclear-graph/` took that folder as the only root and removed the rest of the graph (fixed: an ingest removes only
  sources of its own roots, test in test_graph.py; the graph re-ingested); round 13 with the prompt files: 38/45, held
  out 15/17 (Sonnet 39, 17), the best so far.

- 2026-11-11 · nuclearlang (AI loop session) · Over MCP the local developer works first (asked by the user: a cloud
  agent — Sonnet, Opus, Haiku — gives the task to the Nuclear Developer first, then only validates and corrects; the
  local agent documents everything it did). nuclear-graph MCP: new instructions (local first → validate → correct
  only what is wrong → review), `developer_task` (writes `specs/<id>/spec.md` led by the local agents, runs
  `nuclear-loop run --lead local`, answers the journal and asks for the review), `developer_review` (appends to
  `.nuclear/loop/runs/<id>.review.md`; corrected files, once the project's check is `ok`, go to the graph's `examples`),
  the `feature` prompt rewritten in that order (graph re-ingested); test `tests/test_mcp_loop.py`. Loop engine: every
  run writes its journal `.nuclear/loop/runs/<id>.md` (`journalMarkdown`: files written, what is left, each step with
  its agent, trace and what it read) and a check failure is summarised by its first error, not `]`. Developer: the
  local **documenter** (`documenter.nuc`, gemma4:e2b, `POST /developer/document`) writes "what I built / how I checked
  it / what is left" at the top of the journal. Editors 0.40.1 (all three packaged, VS Code installed). Docs: ADR-0102
  §14, docs/ai/loop.md, both READMEs. Checked: graph pytest 14, Developer `dragonfly test` 4 + pyright strict 0, tsc,
  `./install.sh`; `developer_task` run directly on a fresh project, and Sonnet through `claude -p --mcp-config` twice
  (USD 0.56): it called `developer_task` first, corrected the rest, and on the second run (after the tool's answer asked
  for it) recorded its review with `developer_review`; check ok, tests 3/3.

- 2026-11-11 · nuclearlang (AI loop session) · The local agents write only Nuclear, and the tools write the rest (asked
  by the user: drop Zed, check the alias, `dragonfly make`, LoRA, Nuclear only, tests first, logs for agents). Zed: the
  Loop tasks removed (the Loop is VS Code and Android Studio only). Compiler (generic): `--platform backend|mobile|web`
  for every command (the platform's default dialect underneath); `nuclear spec --brief --platform` = base + platform
  sheet (`spec/brief/{backend,mobile,web}.md`, new; the dialect sheets cut to what a dialect adds; the backend ones still
  said `nuclear.toml`); `.nuclear/map.md` (`map.rs`: every declaration by component with its kind, doc comment and
  members, written by check/build); `[layout]` + `[skeletons]` in platform.toml (`Dialect.skeletons`); `nuclear fix`
  (applies every exact fix); N0221 (an `appcomponent` method without a body: it was generated empty), N0222 (an async
  call used without `~`, with its fix), N0114 for a kept `Result` read as its value (`found.length`, `found[0]`, with
  the fix; a test's `x = ~call()` is now typed `Result`); N0102 also for a member declared twice in one module (a
  local answer had `load()` twice and it compiled). dragonfly CLI: `make <kind> <component> <Name>` for backend
  and web from the platform (test over every kind and dialect); `explain` reads Python and Node stacks and test
  summaries (backend/web line maps); `[toolchain] install_local` reinstalls folder dependencies (a stale
  `nuclear_graph` was found this way). Graph: collection `nuclear` (only what a local agent reads, target code blocks
  removed, `platform` on every chunk), `Toolchain` takes a platform, MCP `developer_*` take `platform`. Developer:
  `platform` everywhere instead of `dialect`, recall from `nuclear`; the planner takes the spec's sentence as each
  piece's task (the model's summary had turned the alias into a method). Prompts: few-shots that compile and repeat no
  held-out task (appcomponent aliases, entity columns, `replaceAll`, state model + manager; a whole test file).
  Loop engine: spec → plan → tests first → code (`nuclear fix .` before any agent repair; a test file goes to the
  tester) → verification (tests run; a failure told by `dragonfly explain` goes back to the coder, 3 rounds) → security
  → artifacts → deploy; a step's `agent` field / the panel's *For the agent* tab. Measured and discarded: the
  `dragonfly make` skeleton given to the local coder (29/42 against 37). Eval: rounds 10–11 (the first feature
  passed by the local agents alone in round 10), two acceptance texts accepted the entity by folder. LoRA:
  `specs/007-lora-nuclear-coder/plan.md`. Editors 0.40.0 (all three packaged, VS Code installed). Docs: ADR-0102 §§11–13,
  spec/{projects,dialects,diagnostics}.md, briefs, docs/ai/loop.md, the graph's and the Developer's READMEs, cli context.
  Checked: cargo test + clippy -D warnings (root and dragonfly/), fastapi/nestjs/react checks, spec-checks 0, graph
  pytest 12, Developer tests 4 + pyright strict 0, tsc, the three extensions built, `./install.sh`, five
  `nuclear-loop run` end to end (every stage runs; the last stops at the tester's empty-database test).

- 2026-11-11 · nuclearlang (AI loop session) · Report for the user: what the AI loop (ADR-0102) has and lacks; nothing changed.

- 2026-11-11 · nuclearlang (AI loop session) · Summary of the pending work for the user (from this file); nothing changed.

- 2026-11-11 · nuclearlang (AI loop session) · Fixed what the AI loop found and did the pending AI-loop work (asked by
  the user: "arregla los errores que encontraste, después continúa con lo que falta"). Compiler (generic):
  messages of snake_case dialects name what the `.nuc` writes (`naming::apply` returns the renames, `naming::restore`,
  `naming::camel`); N0116 closes a kind through the runtime classes it inherits when the type definitions list their
  members (`closed_members`; `[kinds.x] method_members` for what a method template adds: flutter/react
  `purge{{name|pascal}}`, `{{name}}AsLocal`); N0013 says where a decorator left after the last member goes; N0019 for a
  variable named like a method of its module (Python wrote `self.pieces = ...`); Python's `??` binds like a conditional
  (`conditional_form`, `Template::literals`) and a conditional in a ternary's first branch keeps its parentheses (Python
  grouped them wrong); `[layout]` in every `platform.toml` (`Dialect.layout`, `nuclear help layout --dialect d`);
  `spec/brief/nuclear-base.md` no longer says the library is the target's. dragonfly CLI: the mobile skeleton of
  `dragonfly new` did not compile (`toList()`); `every_new_project_compiles_with_its_dialect` builds every skeleton.
  dragonfly-web: `spec/brief/react.md` (the web platform had no agent sheet; Sonnet went from 0/4 to 4/4 web tasks).
  Loop: `nuclear-loop` (`loopCli.ts`: run / ci / export / runs; `parseSpecMarkdown`, `ciCommands`), installed by
  `install.sh`, `stage.sh` (+ `.cmd`), `build-runtimes.sh` and the release installer; JetBrains *Nuclear Loop* tool
  window (`NuclearLoop.java`), Zed tasks; `install.sh --ai` (graph + Developer + `nuclear-developer` launcher). Developer:
  the planner builds paths from the layout (`Toolchain.layout`/`path_of`, `layout_path`, `snake` in nuclear-graph, defs
  regenerated), reads moved columns, keeps only what the spec names, takes kinds and forgotten pieces from the spec's
  words, appends the sentence that introduces each piece; a Nuclear test of it; a feature is distilled as one graph
  example per file. Eval: 10 mobile/web tasks and 3 `[[feature]]`
  tasks (planned, written piece by piece, checked as a fresh `dragonfly new` project); rounds 4–9 (single files local 37/42 vs Sonnet 36; features: 2 of 3 compile, 0 pass; Sonnet 3/3; Claude USD 0.34 more, total 0.89). Prompts:
  `piece.md` no longer lists `contains` (membership is `in`), rules to keep every value the task gives and to write
  only what it declares. Editors 0.39.0
  (all three packaged; VS Code installed). Docs: ADR-0102 decisions 9–10, spec/{dialects,diagnostics}.md, briefs, skill,
  docs/ai/loop.md, the editors' context files and READMEs, nuclear-graph and nuclear-developer READMEs. `nuclear-loop run` with the local agents only took a
  fresh fastapi project through every stage (5 pieces, each ok in 1 attempt; the tester's file did not compile and was
  skipped). Checked: cargo
  test + clippy -D warnings (root and dragonfly/), fastapi/nestjs/react checks, nuclear-graph pytest 10, Developer
  `dragonfly test` 4 + pyright strict 0, tsc, `npm run package`, `jetbrains/build.sh`, `zed/build.sh`, `./install.sh`.

- 2026-11-10 · nuclearlang-60 · nuclearlang-4e's changes verified green here (cargo test 21 + clippy 0, fastapi/nestjs
  checks, spec-checks, nuclear-vscode tsc) and NOT pushed: waiting for the user (nuclear docs + ADR-0102,
  dragonfly-backend `e.message` on a caught exception, nuclear-vscode Loop tool 0.38.0, website "AI" menu; they name
  the unpublished nuclear-graph / nuclear-developer). `tools/context/check.sh` also skips `dist`, `.pytest_cache`,
  `__pycache__` (the checks' build output).

- 2026-11-10 · nuclearlang (AI loop session) · The AI loop (asked by the user; ADR-0102; decided with the user: retrieval +
  distillation instead of weight training, up to USD 20 of Claude, phases without stopping). New repositories in the
  tree: `nuclear-graph/` (Nuclear Smart Graph: Chroma + EmbeddingGemma, 4 collections, documents AES-GCM encrypted,
  key and owner token in the keyring, server on 127.0.0.1 with per-client tokens the user allows, no `Origin`, only
  ingest writes `knowledge`; ingest of the whole tree + website by hash, 2 783 chunks; 9 validated use-case prompts;
  MCP server with 13 tools; 7 pytest tests) and `nuclear-developer/` (a fastapi Dragonfly backend in Nuclear: coder,
  tester, security, planner graphs on qwen2.5-coder:3b / gemma4:e2b; recall → write → check → repair; 3 Nuclear tests;
  pyright strict 0; `eval/` with 32 tasks: after 3 rounds local 28/32 vs Sonnet 26/32, held out 10/12 vs 11/12, Claude
  cost USD 0.55). Loop tool in `nuclear-vscode` 0.38.0 (`loopEngine.ts`, `loopPanel.ts`, `Ctrl+Alt+P`): spec form
  (use cases, Jira/GitHub import, criteria, suggested context, lead), runs as a C4-like zoomable pipeline,
  `.nuclear/loop.json`, CI export (GitHub, GitLab, Bitbucket); ran end to end local-only and Claude-led. Platform
  (generic): `nuclear build|check` load `<project>/defs/` (`project::{project_defs, with_project_defs}`, LSP too), backend
  `dependencies(...)` in `config.nuc` (fastapi `pyproject.toml`, nestjs `package.json`, folder versions), `catch e` +
  `e.message` fixed in both backend dialects (`[library.error]`). Docs: ADR-0102 + indexes, spec/{projects,definitions}.md,
  dragonfly-backend/spec/backend.md, nuclear-vscode README + context, AGENTS.md map, `docs/ai/loop.md` + website menu
  "AI", both new READMEs and AGENTS.md. Checked: cargo test + clippy -D warnings (root and dragonfly/), fastapi-check,
  nestjs-check, nuclear-graph pytest, Developer `dragonfly test`, tsc, `npm run package`, extension installed. Found and
  recorded: three compiler issues (Pending: nuclear).
- 2026-11-10 · nuclearlang-60 · Checked (stop hook): the dragonfly-mobile goldens it listed were rewritten with the
  same content (nuclearlang-4e's `UPDATE_GOLDEN` run; `git status` clean). nuclearlang-4e is now editing
  nuclear-vscode (a Loop tool) and the fastapi/nestjs `[library.error] message`: held from commits until it says
  green. Open with the user: rename `website` → `nuclearlang.github.io` for Pages, and the first release `v0.1.0`.

- 2026-11-10 · nuclearlang-60 · Public repositories, stages 1–3 (asked by the user: "sube los repos a github"; their
  choices: finish the split first, a fresh history for dragonfly, docs/aprende private, the e-mail out of the
  credits). See "Half done" → Public repositories for the layout, the publishing files, the ten pushed repositories
  and the CI fixes. Checked before each push: the whole tree (root 21 test binaries + clippy -D warnings 0,
  dragonfly 26, the platform checks, spec-checks, a Flutter app, the website, the three extensions, install.sh) and
  a lone clone of nuclear (21 green); staged content scanned for keys, e-mails and home paths (none).

- 2026-11-10 · nuclearlang-60 · Documentation pass (asked by the user: record the latest changes everywhere): the
  start hook lists no unrecorded file; README and the tutorial's install step now give the one-command installers
  (ADR-0101) before `./install.sh`; the website's README describes `site.toml`, GetSite and the installers it serves;
  two memory notes updated (the website guide's menu is `site.toml`, the website is its own repository); website
  content rebuilt.
- 2026-11-10 · nuclearlang-60 · (1) The old VitePress `website/` removed (asked by the user): moved to the desktop
  trash with `.github/workflows/docs.yml` (the auto-mode guard refused `rm -rf`); README, AGENTS.md,
  docs/context/{architecture,tech,workflow,website/CONTEXT}.md and plan 006 name the `website` repository. (2) ADR-0100
  §5 migrations (asked by the user: "empieza"): `dragonfly migrate make "<msg>" | [up] | down` (cli `migrate_command`,
  `shell_quote`, `backend_command` vars) over the dialects' `[toolchain] migrate_make/migrate/migrate_down`
  (`{{message}}`, `{{migrations}}`); fastapi: `dragonfly.migrate` (Alembic, `make` over a scratch SQLite at head, batch
  mode, drop warnings, `alembic` a runtime dependency, `create_app` keeps the config in `app.state`); nestjs:
  `dragonfly-nest/migrate` (bin `dragonfly-nest-migrate`: Sequelize attributes → schema, diff with the last migration's
  exported `schema`, `queryInterface` up/down, `SequelizeMeta`), the app module exports `entities` and `config`
  (golden). Found on the way: a Python `column(default=...)` was only a client default, so adding a non-null column to
  a table with rows failed — it is now also the SQL default (`_sql_literal`), and `schema_of` keeps such a column in
  the POST body. Tests: `packages/dragonfly-py/tests/test_migrate.py`, `packages/dragonfly-nest/test/migrate.test.ts`,
  cli `a_placeholder_is_one_shell_word`; both check scripts make + apply + "no changes" on a fresh SQLite. Docs:
  ADR-0100 §5, spec/backend.md §2.8, spec/{fastapi,nestjs}.md (+ their stale CLI line), spec/dialects.md, the Python
  and NestJS guides, cli and backend context, plan 004 F2. Checked: cargo test + clippy 0, cli 26 + clippy 0, editors
  tests, nestjs-check (runtime 5), fastapi-check, react-check, spec-checks 0, dragonfly.py 27/0, pyright of the
  runtime (the 4 older errors only), both dialects end to end with the installed CLI; editors 0.37.2 released (bundled
  dialects changed), `./install.sh`.
- 2026-11-10 · nuclearlang-60 · No hard-coding + installers (asked by the user; ADR-0101). Dialect data: `[toolchain]`
  keys `url`/`ready`/`installed`/`serve`/`compile`, `[toolchain.<os>]`, `[[toolchain.launch]]`, `[toolchain.settings]`,
  `extensions` (fastapi, nestjs, react, flutter; react's install also clears Vite's cache), `[platform]
  default_dialect` (model.rs `Dialect::tool`, `Loader::default_dialect`). dragonfly CLI: one generic `vscode::write`,
  `backend_command` via `dialect.tool` + `shell()` (cmd on Windows), `any_manifest`, default dialects from the
  platforms, the web scaffold takes the dialect and the backend's url. nuclear CLI: `agent::init` by dialect, `kind
  new` from `[syntax] class`, `platform_topics()` read from the doc roots, `nuclear build` removes the output of a
  removed `.nuc` (`tests/build.rs`). Card `toolchain` (spec/projects.md). Extension 0.37.1: `card.ts`, the web device
  serves with the card's `serve`, the HTTP tool's base urls from the backends' cards. Installers: `tools/release/
  {stage,package,install}.sh`, `install.ps1`, `.github/workflows/release.yml`; `./install.sh` uses `stage.sh`. Website:
  `site.toml` holds every text/menu/link, `content.mjs` reads it (site.json, home.json, llms.txt from the menu, shiki
  languages from the pages) and serves the installers; the app draws TopBar/Sidebar/Footer/Home from the data (Site
  model, SiteManager), 4 tests. Checked: cargo test + clippy 0, editors tests + clippy 0, cli 25 + clippy 0, extension
  tsc + package, nestjs/fastapi/react checks, spec-checks (7) 0 problems, dragonfly.py 27/0, website tsc/tests/vite
  build, `./install.sh`, `package.sh` (linux x86_64) + `install.sh` from a local mirror into a fake HOME (install,
  PATH lines, skill, `dragonfly new -p backend|web` + build with the installed binaries, uninstall clean). See
  "Untested".
- 2026-11-10 · nuclearlang-60 · Runners and debuggers of every platform + the web device (asked by the user): editors
  0.37.0. Compiler: every project build writes `<out>/.nuclear/lines.json` (`project::{LINE_MAP_FILE, line_map}`).
  nestjs dialect: `sourceMap`, `dev` with `node --enable-source-maps --watch`, `[toolchain] compile` (run by `dragonfly
  build` once installed), `.npmrc` install-links (the same symlink bug as react). dragonfly CLI: `vscode::write_platform`
  (`build/<name>.code-workspace` for backend and web: tasks build/install/run/test/check, launch debugpy for fastapi,
  node with source maps for nestjs, chrome for react). Extension: `debug.ts` generic (`.nuclear/lines.json`, trackers
  for dart/node/chrome/debugpy, mirrors per file's project), `runPlatformApp`, build on save for backend/web
  (`nuclear.buildOnSave`), toolbar actions (backend ◉ Debug; web ▭ Device, ◉ Debug, ↻ Reload; `dragonfly` commands,
  install first when needed), the web device (`webSession.ts` engine without VS Code, `webDevice.ts` panel, `cdp.ts`
  shared with the mobile device: presets, address bar, DevTools, color scheme, throttling, no cache, Console, Network,
  screenshots, debugger attach). The web device's console found a runtime bug: children lists made React ask for keys
  (`el()` now spreads them, dragonfly-web). Website: top bar fits a phone. Docs: ADR-0099 decisions 9–10,
  `docs/context/editors/vscode.md`, editors README, the React/NestJS/Python guides. Checked: cargo test (goldens:
  nestjs `.npmrc`, `sourceMap`, `dev`), clippy 0, editors cargo test, cli tests + clippy, nestjs-check, react-check,
  spec-check nestjs, extension tsc + package (check-bundle), `./install.sh` (0.37.0). See "Untested".
- 2026-11-10 · nuclearlang-60 · The NuclearLang website in Nuclear (asked by the user; see "Half done"). Platform
  fixes found on the way (generic): the web router builds a new screen instance per address (`useLocation` key);
  views read the managers their screen provides (`[kinds.stateless|stateful] provide`); HTML functions type their
  props (`Tag<T>` overloads: `onChange: (e) => e.target.value`); the generated app's `.npmrc` (`install-links=true`)
  — the runtime was a symlink after the toolchain's second `npm install`, so React's types were `any` and `vite
  build` failed: the 2026-11-08 known limit is fixed; index.html takes `app(title)`, `app(description)` and the
  project's `public/styles.css` / `favicon.svg` (new `app(title)` in the manifest, `asset.<path>` for project
  templates). Docs: spec/projects.md, dragonfly-web/spec/{web,react}.md, dragonfly-web/docs/dragonfly/react.md.
  Checked: cargo test (golden: characters' description meta + `.npmrc`), clippy 0, react-check (tsc 0, 4 + 7 tests,
  vite build), spec-check react 0, `./install.sh`; editors 0.36.6 released (bundled react dialect changed).
- 2026-11-10 · nuclearlang-60 · Public repositories, stage 1: `dragonfly-backend/` and `dragonfly-mobile/` moved
  (see "Half done" for the list and the checks); golden.rs `root_of`, LSP tests `in_tree`, spec-check's loader, the
  defs generators' platform outputs, the backend/mobile check scripts (`here`/`root`, `NUCLEAR_ROOT`), cli
  `runtime_package` + stale pubspec path repair, the python venv's editable runtime reinstalled from its new path.
  The user confirmed: commits only in their name (no Co-Authored-By); gh logged in as the user (admin of nuclearlang),
  git over HTTPS.
- 2026-11-10 · nuclearlang-60 · Public repositories, stage 1 started (asked by the user): the source-roots mechanism
  (crates/nuclear-dialect `source_roots`/`dialect_dirs`; nuclear CLI `PLATFORM_TOPICS`/`read_doc`; tests `tree`/
  `expected_of`; install.sh; editors `bundle-server.mjs`, `jetbrains/build.sh`, `lsp/src/dialects.rs`;
  website `scripts/sync.mjs`) and the pilot move of `dragonfly-web/` (dialects/web, packages/dragonfly-web, its defs,
  spec/{web,react}.md, examples/examples-web + expected/characters, tools/react-check, docs/dragonfly/react.md,
  docs/context/dragonfly-web). Checked: cargo test, clippy 0, editors tests + clippy 0, cli build, react-check,
  .vsix check-bundle, `./install.sh`, website build. AGENTS.md and docs/context paths updated.
- 2026-11-10 · nuclearlang-60 · Public repositories planned (asked by the user): decisions taken by the user (org
  `nuclearlang`, public, Apache-2.0, staged split), `specs/006-public-repositories/plan.md`, `specs/README.md`; checked
  the org exists (an empty `nuclear` repo), no secrets in the tree (personal data: the CLI credits by design, the Zed
  manifest's `file://` paths to replace). No code changed.
- 2026-11-10 · nuclearlang-60 · Editors 0.36.5 (asked by the user: the editor did not flag `@generated` in
  nuclear_langgraph_agent, the bundled server predated ADR-0100): bump in `vscode/package.json`, `plugin.xml`,
  `zed/extension.toml`, README, context; `npm run package` (check-bundle ok), `jetbrains/build.sh`, `zed/build.sh`,
  `./install.sh` (0.36.5 installed); the bundled server driven over JSON-RPC shows N0309 on that file. Not eyeballed in
  a running VS Code (it needs a full restart).
- 2026-11-10 · nuclearlang-60 · ADR-0100 phase 1 (the user's review of the draft: one `@primary(generated, length)`,
  TypeORM relation decorators, kinds untouched, transactions now; ADR renamed `0100-backend-data.md`, Accepted). Compiler
  (generic): `[decorators.fields]` rules + `[decorators.args]` from `platform.toml` (N0308, Nuclear-name comparison for
  snake_case dialects), N0309 (a decorator the platform does not declare; `[decorators.replaced]`), `tinfo.decor`,
  `tinfo.runtime`, `tinfo.model` for runtime classes with `fromJson`, `owner`/`owner_decor` in field templates, filter
  `unquote`, `[kinds.x] type_imports` + `[syntax] type_imports` (Python `if TYPE_CHECKING:`), `[types]` renames import
  by the target name and a project declaration wins over them, repository kinds inherit `Query` (so `update`/`delete`
  resolve). Runtimes: dragonfly-py (`primary(generated, uuid_key)`, `many_to_one`/`one_to_many`/`many_to_many`/
  `one_to_one`, `relations`, `paginate` → `Page`, `transactional`, unloaded relations left out of answers, SQLite FKs,
  `entities=` in `create_app`/`running`), dragonfly-nest (`Entity`, `Related<T>`, `joinTable`, `Page`, `relations`,
  `paginate`, `transactional` over AsyncLocalStorage, `save` writes relations), dragonfly Dart (`PageOf<T>`,
  `LocalQuery.paginate`); defs of the three regenerated. Templates of fastapi, nestjs, flutter; `@generated` removed
  (backend + mobile). Example tasks_api: `Project` ↔ `Task`, `Tag` many-to-many, paged route, `@transactional`
  `CloseProject`, 2 new tests; tasks_app contract regenerated. Checked: cargo test + new `relation_decorators_have_rules`,
  clippy 0, editors and cli build, nestjs-check (tsc 0, 6 tests, smoke, runtime 4 incl. `data.test.ts`), fastapi-check
  (pyright 0, rick_api + tasks_api 6 tests, runtime 3 incl. `test_data.py`), tasks_app analyze 0 + 27 tests,
  clean-flutter-project 34 tests, package local tests, spec-check fastapi/nestjs 15/0, flutter 3/0, dragonfly.py 27/0
  (5 blocks resynced after the earlier `nuclear fmt` pass), website build, `./install.sh --no-vscode --no-zed`. The check
  scripts now fail on a failing pyright/pytest/node:test and reinstall a fresh nest runtime. Docs: ADR-0100 + indexes,
  ADR-0096 note, spec/{backend,fastapi,nestjs,mobile,flutter,dialects,projects,diagnostics}.md, briefs, docs.toml (3),
  docs/dragonfly/{python,nestjs,kinds}.md, docs/context/{dragonfly-backend/CONTEXT,dragonfly-mobile/kinds}.md, plan 004.
  Editors rebuilt by this session afterwards (0.36.5, see the entry above).
- 2026-11-10 · nuclearlang-4a · Verified (stop hook flagged files newer than my last STATUS edit): the flagged
  `crates/nuclear-syntax/src/{fmt,lib}.rs`, `crates/nuclear-syntax/tests/fmt.rs`, `crates/nuclear-cli/src/main.rs`,
  `spec/{nuclear-base,brief/nuclear-base}.md` and `spec/decisions.md` are nuclearlang-60's `nuclear fmt` work (plan
  005 F1), already fully recorded by it (step 6, Pending: editors/ format-on-save, the fmt limits and its own Log
  entry). Not my change; nothing to add.

- 2026-11-10 · nuclearlang-60 · Steps 4 and 6 started (asked by the user: both, the ADR shown for confirmation; code as
  alike as possible across platforms, cost/benefit middle point). (1) ADR-0100 drafted (`docs/adr/0100-data-by-
  deduction.md`, Proposed, both indexes). (2) `nuclear fmt <path>... [--check]` (`crates/nuclear-syntax/src/fmt.rs`):
  rewrites only the gaps between tokens, copies tokens as written, skips the lexer's synthetic tokens, keeps the space
  before `:` and around `<`/`>`/`?`/`*`… as written, refuses a result whose tokens/comments differ; 6 unit tests +
  `tests/fmt.rs` (every `.nuc` of examples/dialects/packages/kinds is a fixed point); 10 example files formatted (map
  braces `{ "a": 1 }`, the docs' majority style). CLI usage + `nuclear help fmt`; docs: `spec/nuclear-base.md` §2.2,
  `spec/brief/nuclear-base.md`, skill step 4, `docs/language/01-syntax.md`, `docs/context/{tech,nuclear/CONTEXT}.md`,
  plan 005 (F1 done). cargo test green (goldens unchanged), clippy 0, `./install.sh --no-vscode --no-zed`, website build.
- 2026-11-10 · nuclearlang-60 · Design talk with the user on steps 4 and 6 (no code): decided "write only what cannot
  be deduced" for the backend data ADR — relations from field types, `id: int` as the key, kind by folder for
  repository/controller and the path from the name; recorded in `specs/004-backend-parity/plan.md` §4. The F1 example
  above that section still shows the decorators (the ADR will rewrite it). Open: fmt reflow only?, order (fmt first?),
  the user's next feature.
- 2026-11-10 · nuclearlang-60 · Re-checked for the user (start hook clean, no file newer than this one, only
  nuclearlang-dc's documentation pass below is new); recap given, lists unchanged.
- 2026-11-10 · nuclearlang-4a · Documentation pass (asked by the user: write everything done into the docs):
  verified every recent change is recorded (editors 0.36.1–0.36.4 device layout + sharp streaming + ADR-0086
  grammars in STATUS Log/Untested, spec/flutter.md §11.1, docs/context/editors/vscode.md, ADR-0099; CLI web projects
  and the new CLI generators in STATUS Log, spec/flutter.md §11, docs/dragonfly/tutorial.md,
  docs/context/cli/CONTEXT.md; web also covered by nuclearlang-60 in docs/dragonfly/react.md and
  docs/context/dragonfly-web/CONTEXT.md). One gap filled: the `dragonfly new` line in spec/flutter.md §11 listed
  mobile and backend but not `-p web -d react` — now it names all three platforms/dialects and the `nuclear build .`
  + `[toolchain]` + `check` delegation. No code changed.

- 2026-11-10 · nuclearlang-60 · Second recap of the pending work for the user after the sync; nothing changed.
- 2026-11-10 · nuclearlang-60 · Synced with nuclearlang-dc (asked by the user): its work since 2026-11-07 was already
  recorded; the theme/icon files dated 2026-11-09 are the 0.36.4 build's regenerated output. Docs updated:
  `docs/dragonfly/react.md` §1 (create/build/run with `dragonfly new -p web -d react` and the CLI) and §5 (the `vite
  build` limit), `docs/context/dragonfly-web/CONTEXT.md` (the CLI section), `docs/context/editors/CONTEXT.md`
  (installed 0.36.4). Website build green.
- 2026-11-10 · nuclearlang-60 · Recap of the pending work for the user (from this file; the start hook lists no
  unrecorded files); nothing changed.

- 2026-11-09 · nuclearlang-4a · CLI generators for the remaining kinds (Pending: cli/, point 4): `scaffold::presentation`
  (`g sm`/`statemanager <c> <s> <name>` → a Cubit in `presentation/<s>/managers/` + its state model; `g smo`/
  `statemodel <c> <s> <name>` → a state model) and `stateful` (`g sf <name>` → a `stateful` widget in
  `shared/widgets/`, sibling of `g w`); reused the STATE/MANAGER/SHARED templates. Checked first that `guard`,
  `global` (the `shared/components` `@on(Event)`), `page` (shared screens) and `integration` already existed and
  were wired — the old STATUS bullet was stale; only dictionaries/`Translations`/locale in the skeleton are left.
  `main.rs` aliases + `make` arms + USAGE (+ the help slice `take(21)`); `every_skeleton_parses` covers the three;
  spec/flutter.md §11 generator table completed (it was missing statemanager/statemodel/stateful and also
  guard/global/page/integration), docs/dragonfly/tutorial.md, docs/context/cli/CONTEXT.md. cli tests 25 + clippy 0;
  smoke `g sf`/`g sm`/`g smo` in a scratch project (correct paths, HomeComponent manager); `./install.sh --no-vscode
  --no-zed` (installed help shows them).


- 2026-11-08 · nuclearlang-4a · `dragonfly new -p web -d react` and the web build commands (Pending: cli/, ADR-0097):
  `scaffold::web_project` (config.nuc `dragonfly(platform="web", dialect="react")` + `app`, `main.nuc` with the App that
  renders `RouterProvider` over `appRouter()`, `config/project_config.nuc` with an `Http` connection); `new_project`
  branches to it; `backend_manifest` → `non_mobile_manifest` (platform backend **or** web) so `build`/`install`/`run`/
  `test` route through the dialect's `[toolchain]` like backend; `check` delegates to `nuclear check .` for backend and
  web (the Flutter check wants a pubspec/fonts). USAGE + usage string updated; `every_skeleton_parses` covers the web
  skeleton. Verified end to end: `dragonfly new -p web` → `dragonfly check` (3 files, 0) → `dragonfly build` (9 files →
  build/app) → `dragonfly install` (npm) → `tsc --noEmit` 0. `vite build` fails on the react runtime's bundling (react
  not resolved from the `--install-links`ed `dragonfly-web`): recorded under Pending: Dragonfly web, it is the
  runtime's packaging, not the CLI. cli tests 25 + clippy 0; `./install.sh --no-vscode`. docs/context/cli/CONTEXT.md.
  Also assessed and deferred `Name<T>.member(...)` (needs a shared-AST variant; see Pending: nuclear).

- 2026-11-08 · nuclearlang-4a · Editors 0.36.4 (Pending: editors/, ADR-0086): the highlight grammars learned the
  operators and raw strings the lexer already had. TextMate (`vscode/syntaxes/nuclear.tmLanguage.json`, shared with
  JetBrains): binary `0b…` and float exponents in `number`, raw strings `r"…"`/`r'…'`/triples in `string`, and `**`,
  `~/`, `<<`, `>>`, `>>>`, `&`, `^`, `~` plus their compound assignments in `operator`. Zed tree-sitter
  (`zed/grammar/grammar.js`, `languages/nuclear/highlights.scm`): same tokens; `tree-sitter generate` clean and a
  sample with every new token parsed with no ERROR (raw `r"…"` → `string`, `1.5e-3` → `number`). Bump + package +
  jetbrains/zed builds + `./install.sh` (0.36.4). Not eyeballed in a running editor.

- 2026-11-08 · nuclearlang-4a · Editors 0.36.3, device still soft after 0.36.2 (user screenshot): dropped the
  `screenDpr` heuristic for an exact measure — the webview reports `shot` = the screen's physical pixels right now
  (CSS width × zoom × `devicePixelRatio`) in the `size` message, and `sized` renders+streams the app at
  `density = max(dpr, shot / width)` (capped 4, only ever raised, re-emulating live), so a frame matches the panel's
  on-screen pixels 1:1 and is never upscaled. `editors/vscode/src/device.ts` (`Running.density`, `Size.shot`);
  `spec/flutter.md` §11.1, `docs/context/editors/vscode.md`. compile clean; bump + package + builds + `./install.sh`
  (0.36.3). It now adapts live as the panel resizes, but a device already running from before this build is orphaned
  by the extension reload, so re-run it once after installing. Not re-verified live.

- 2026-11-08 · nuclearlang-4a · Editors 0.36.2, device still looked pixelated after 0.36.1 (user screenshot): the
  screencast now renders and streams at `captureScale` = `max(device dpr, 1.5 × monitor dpr)` capped at 4 (the webview
  reports the monitor's `devicePixelRatio` in the `size` message → `screenDpr`; `emulate` re-renders if it changes),
  and jpeg quality 90→100 to drop the compression artefacts that read as pixelation. `editors/vscode/src/device.ts`
  (`captureScale`, `Size.dpr`); `spec/flutter.md` §11.1, `docs/context/editors/vscode.md`. compile clean; bump +
  package + builds + `./install.sh` (0.36.2). Note for the user: the stream resolution is fixed when the app starts,
  so a running device must be re-run to pick this up. Not re-verified live.

- 2026-11-08 · nuclearlang-4a · Editors 0.36.1, two device bugs the user hit live (screenshots): (1) the device panel
  lost its layout across a workspace round-trip — after visiting the Devices tool and returning to a mobile project the
  device collapsed into the first editor column and the reopened code merged into its tab group (device as a tab left of
  the code, filling the screen when clicked). `Workspace.switchTabs` now parks the devices on every switch and re-adopts
  them *after* the code is reopened (was: only parked when leaving to a non-mobile workspace), so each device always
  lands in its own locked group beside the code (the right, like Android Studio) and is never merged into the code's
  tabs; the per-screen saved width (`nuclear.device.widths`) is still restored, so the user's distribution is kept.
  (2) the device looked low-resolution: `screencast` captured at `min(dpr, 2)`, so a 3x phone (iPhone 14 Pro) was
  streamed at 2x and upscaled blurry on a HiDPI monitor; now it captures at the device's full `dpr` (the page's real
  backing-store size) and jpeg quality 80→90. `editors/vscode/src/{workspace,device}.ts`; ADR-0099, `spec/flutter.md`
  §11.1, `docs/context/editors/vscode.md`. `npm run compile` clean; bump + package + jetbrains/zed builds +
  `./install.sh` (0.36.1 installed). Not re-verified in a running VS Code window (Untested below).

- 2026-11-07 · nuclearlang-4a · Live verification of the 0.36.0 device work in a running VS Code (asked by the user:
  "ejecuta los puntos 1,2,3,4" of the device Untested list). Method: an isolated test VS Code (its own `--user-data-dir`,
  the installed 0.36.0 extension via `--extensions-dir ~/.vscode/extensions`, Copilot/claude-code disabled), driven over
  Hyprland `send_shortcut`/`window.close` with `grim` screenshots and `pgrep` on the `__dragonfly_device_<pid>_` markers;
  the user's real VS Code (pid 848862) never touched. Results on `rick_and_morty`: (3) session names are "Dragonfly
  device 1/2/3" in the debug Call Stack, the debug toolbar, the Run/Debug device picks and the window title — not the old
  "rick_and_morty (debug) (Chrome) N". (4) three devices ran at once, tiled in the Devices workspace, each its own dartvm
  + brave; a new Run kept the running ones; closing a device tab parked the app (stayed headless, still in the Devices
  pane and the Call Stack); entering the Devices workspace unparked it. (2) a device whose tab was closed during the
  Flutter build (dartvm up, browser not yet connected) stayed tracked/parked and finished headless — no untracked orphan
  (the old bug). (1) closing the window reaped every device app (6 processes → 0 in ~3 s); the watchdog is wired
  (`activateDevice` subscribes `watchdog()`, every 60 s). Not driven: two devices with two signed-in app users (needs
  real auth), the park/unpark on switching to the python project tab (same code; the native folder dialog landed
  off-screen), the 30-minute idle auto-stop (can't wall-clock), and the non-device UI (icon theme, HTTP workspace, web
  commands, tool pages). No code changed; STATUS.md "Untested"/"Half done" updated above.

- 2026-11-07 · nuclearlang-4a · Editors 0.36.0 (the user: device numbers jumping to 6, sessions named "rick_and_morty
  (debug) (Chrome) 2", 13.5 GB of RAM with devices created and closed while starting, ▷ Run without the devices,
  closing a device tab must not end it, idle devices should end by themselves). Checked first: no orphan process was
  alive on the machine at that moment (the activation reaper had run), so the memory was live apps: a closed panel kept
  its app 5 more minutes and one closed during Flutter's build had no owner at all. `device.ts`: `Running` lives on
  its own (`panel` optional, `title`, `lastUsed`, `ended`), closing a tab parks the app (hidden; listed in the Devices
  pane, shown again by click or ▷ run), `endRunning` ends session + cdp + browser + Flutter at once (the pane's ✕
  `nuclear.closeDevice`, the panel's ■ also while building, the session's end), a session that starts with no owner
  is stopped and reaped, `watchdog()` every minute (a device not looked at for 30 min ends; `pgrep -af` by the window's
  marker reaps keys no app owns; Unix only), `freeNumber()` gives the lowest free name, the launch config is named after
  the device, `nuclear.runOnDevice(panelId, root, noDebug)`. `workspace.ts`: ▷ Run / ◉ Debug / ▭ Device share
  `pickDevice(p, mode)` (devices shown or hidden, *New Dragonfly device…*, *Other Flutter device…* through
  `flutter.selectDevice`, or the Devices workspace); the Devices pane shows hidden devices and a ✕. ADR-0099 decision
  4, `spec/flutter.md` §11.1, `editors/vscode/README.md`, `docs/context/editors/{vscode,CONTEXT}.md`. Bump, package,
  jetbrains/zed builds, `./install.sh`. Not run in a VS Code window (Untested: the watchdog's `pgrep`, the idle end, a
  device closed while Flutter builds, the session names in the debug toolbar).

- 2026-11-07 · nuclearlang-4a · Editors 0.35.1 (the user: switching workspaces asked "Do you want to save the changes"):
  `switchTabs` closes only the editors without unsaved changes; a dirty one stays open until saved (no prompt, no
  cancelled switch). ADR-0099 limits, `spec/flutter.md` §11.1. Bump, package, jetbrains/zed builds, `./install.sh`.

- 2026-11-07 · nuclearlang-4a · Editors 0.35.0 (the user: devices shared with the python workspace, the ▭ Device button
  replacing the running device and separating the panel, the Devices tool overwriting instead of adding, tiling, logos
  for HTTP and Devices). `device.ts`: a run without a chosen device takes `DevicePanel.idle()` or a new panel, never a
  running one; `parked` + `DevicePanel.park()`/`unpark()` (`nuclear.parkDevices`/`unparkDevices`): leaving to a
  non-mobile workspace closes the panels and keeps the apps running (screencast stopped, idle stop skipped), entering
  a mobile workspace or the Devices tool opens them again side by side (`show(context, preset, title)`, `endRunning`
  for a parked app whose session ends, `post` ignores a disposed panel); `nuclear.openDevice` returns `{ id }`.
  `workspace.ts`: ▭ Device = `pickDevice` (open devices with their state, *New device…*, *Devices workspace*), the
  Devices pane runs `mobileRoot()` (active mobile project, the only one, or a pick) through `inWorkspace` (the Dart
  "Dart-only workspace" join), `switchTabs` parks/unparks, the strip draws the tools as glyphs (`GLYPHS.http/devices/
  designer/docs/agent/settings`) colored with `nuclearHttp.*`. ADR-0099 decisions 2 and 4, `spec/flutter.md` §11.1,
  `editors/vscode/README.md`, `docs/context/editors/{vscode,CONTEXT}.md`. Bump, package, jetbrains/zed builds,
  `./install.sh`. Not run in a VS Code window (Untested).

- 2026-11-06 · nuclearlang-4a · Editors 0.34.1 (the user: Flutter stopped running — "Unable to launch Flutter project
  in a Dart-only workspace"): the Dart extension runs Flutter only for a project inside the workspace folders, and a
  project opened as a tab is outside them. Run/Debug/Device of such a mobile project now save `PendingAction`
  (`nuclear.workspace.pendingAction`, global state), add the folder (`addToVsCode`), reload the window once and
  `activateWorkspace` resumes the action (`workspace.ts`). ADR-0099 limits, `spec/flutter.md` §11.1, README, context.
  Bump, package, jetbrains/zed builds, `./install.sh`. Not run in a VS Code window.

- 2026-11-06 · nuclearlang-4a · Editors 0.34.0 (the user: the actions must not sit inside the file tree but in a
  horizontal row above the editor tabs): the platform actions are now the editor title bar's toolbar, commands
  `nuclear.action.{run,serve,debug,reload,device,build,test,check,api}` with codicons, shown by the context key
  `nuclear.platform` that `Workspace.refresh()` sets (mobile: run/debug/reload/device/build/test/check; backend:
  serve/test/build/check/api; web: serve/build/test/check; none while a tool is active); keybindings moved to them
  (`serve` on Ctrl+Alt+R for backend/web); the per-file `editor/title/run` and `runOnDevice` entries removed (they
  duplicated the toolbar); the button bar of the pane deleted. ADR-0099 decision 3 + limits, `spec/flutter.md` §11.1,
  `editors/vscode/README.md`, `docs/context/editors/{vscode,CONTEXT}.md`. Bump, package, jetbrains/zed builds,
  `./install.sh`. Not run in a VS Code window.

- 2026-11-06 · nuclearlang-4a · Editors 0.33.1 (the user: the HTTP tool showed nothing): `HttpModel` listed only the
  projects of the VS Code workspace folders, so projects opened as tabs (outside the folders) had no endpoints;
  `HttpModel.extraRoots` (set to the workspace's saved tabs) + `findProjects(extra)`, refreshed when a tab is opened or
  removed. Bump, package, jetbrains/zed builds, `./install.sh` (0.33.1 installed). Not run in a VS Code window. The
  user's other requests of this round (actions in their own column, devices like Android Studio, mobile only) are
  recorded in "Half done" for a fresh session.

- 2026-11-06 · nuclearlang-4a · **Workspace, third round (asked by the user): editors 0.33.0.** Every tab of the strip
  is a workspace of its own, tools included (`switchTabs` keeps only the device webviews). The HTTP tool is back to the
  tree the user liked, drawn in the workspace pane (platform → project → contracts [↓] / components → endpoints,
  OpenAPI documents [+], the env bar; `HttpModel.tree()`, inline glyphs + `nuclearHttp.*` colors) and works like Bruno:
  a component is an editor tab, each endpoint a tab inside it (`ComponentPanel`, state kept in `HttpModel.memory` and
  restored when the workspace is entered again); environments back to the window's first folder. A project's pane shows
  the actions of its platform above the files (`PLATFORM_ACTIONS`: mobile run/debug/reload/device/build/test/check,
  backend serve/test/build/check/API, web serve/build/test/check; `Ctrl+Alt+R/G/L/V/B/T/C`; `nuclear.run`/`debug`/
  `runOnDevice` take the project's root, `dragonflyOf(file)` for hot reload on save). Devices tool (`Ctrl+Alt+V`):
  `device.ts` refactored for several devices at once (`panels`, `Running.panel`, `dragonflyPanel` in the launch config,
  `DevicePanel.show(context, preset)` always one more, `devices()`/`devicesChanged`), the pane lists them with ▷ run and
  *+ New device* (preset pick). ADR-0099 decisions 2–4 and limits, `spec/flutter.md` §11.1 (+ the *Devices* bullet),
  `editors/vscode/README.md`, `docs/context/editors/{vscode,CONTEXT}.md`. Bump of the three extensions, package,
  jetbrains/zed builds, `./install.sh` (0.33.0 installed), dragonfly.py 27/0. Not run in a VS Code window (Untested).

- 2026-11-06 · nuclearlang-4a · **Workspace refined (asked by the user): editors 0.32.0.** Projects share nothing:
  switching a tab remembers and closes the editors of the project left and reopens those of the one entered
  (`switchTabs`, `workspaceState` `nuclear.workspace.tabs`); the HTTP tool is one full-screen page for the active
  project (`HttpModel` + `HttpPanel`: environment select, contracts with Update, endpoints by component, OpenAPI
  documents on the left, request/response on the right; environments per project in `<project>/.nuclear/http.json`;
  the sidebar `Endpoints` view and its menus are gone); the file tree wears the Chernobyl icon theme (the user's
  variant, VS Code's lookup) and honours plain `files.exclude` names; project tabs show the platform glyph
  (`icons/platform-{mobile,web,backend}.svg`, `theme/icons.mjs`) with the dialect's Atom Material icon as a badge.
  ADR-0099 decisions 1, 3, 4, 5 and limits; `spec/flutter.md` §11.1; `editors/vscode/README.md`;
  `docs/context/editors/{vscode,CONTEXT}.md`. Bump of the three extensions, package, jetbrains/zed builds,
  `./install.sh` (0.32.0 installed), dragonfly.py 27/0. Not run in a VS Code window (Untested).

- 2026-11-06 · nuclearlang-4a · **The Nuclear workspace in the editor (ADR-0099, asked by the user with a sketch): editors
  0.31.0.** `editors/vscode/src/workspace.ts`: one activity bar container `Nuclear` with a webview view (a vertical
  strip of tabs: the projects at the top with `Ctrl+1`…`Ctrl+9`, `+` to open a project as a tab without touching the VS
  Code workspace, the tools at the bottom with `Ctrl+Alt+H/D/M/A/S`) beside a file tree of the active project; the
  `Endpoints` tree of Nuclear HTTP moved into the container (collapsed); tools open full screen in the editor area
  (template pages for designer, documenter, agent, settings; HTTP expands its tree); a toast with the shortcut when a
  tab is clicked; the welcome page (`Ctrl+Alt+W`, at startup when no editor is open, `nuclear.workspace.welcome`);
  commands `nuclear.workspace.*`, `onStartupFinished`, `icons/http.svg` → `icons/nuclear.svg` (`theme/icons.mjs`).
  Docs: ADR-0099 (+ indexes; it was already referenced by ADR-0098), `spec/flutter.md` §11.1, `editors/vscode/README.md`,
  `docs/context/editors/{vscode,CONTEXT}.md`. Bump of the three extensions, `npm run package`, `jetbrains/build.sh`,
  `zed/build.sh`, `./install.sh` (0.31.0 installed), dragonfly.py 27/0. Not run in a VS Code window (Untested).

- 2026-11-06 · nuclearlang-4a · Editors 0.30.0 (asked by the user: the Nuclear HTTP view in Android Studio):
  `editors/jetbrains/src/.../NuclearHttp.java` (tool window *Nuclear HTTP*: projects by platform → components →
  endpoints from the contracts, contracts with Update, OpenAPI documents, environments `.nuclear/http.json`, request
  panel with `java.net.http`, pretty JSON, copy as curl; action with Alt+Shift+H in `plugin.xml`), description and
  tool window registered; `docs/context/editors/jetbrains.md`, `spec/flutter.md` §11.2. Bump of the three
  extensions, `jetbrains/build.sh` (javac ok), `npm run package`, `zed/build.sh`, `./install.sh` (0.30.0). Not run in
  Android Studio (Untested, as the whole plugin).

- 2026-11-06 · nuclearlang-4a · Editors 0.29.3 (asked by the user: the Nuclear HTTP tree in the Chernobyl palette):
  `theme/build.mjs` sets `nuclearHttp.*` from the palette of each variant (themes regenerated); `spec/flutter.md`
  §11.1 documents the view, `docs/context/editors/theme.md` the colors. Bump, package, jetbrains/zed builds,
  `./install.sh` (0.29.3 installed).

- 2026-11-06 · nuclearlang-4a · Editors 0.29.2 (asked by the user: colors in the Nuclear HTTP tree): theme colors
  contributed by the extension (`nuclearHttp.get/post/put/patch/delete/platform/project/component`, dark and light
  defaults) on the tree icons; bump, package, jetbrains/zed builds, `./install.sh` (0.29.2 installed); context updated.

- 2026-11-06 · nuclearlang-4a · Editors 0.29.1 (asked by the user: the Nuclear HTTP view uses the Nuclear logo): the
  trefoil glyph as a `currentColor` mask generated by `theme/icons.mjs` (`vscode/icons/http.svg`); the activity bar
  paints icons in the theme's color, so the logo's yellow cannot show there. Bump, package, jetbrains/zed builds,
  `./install.sh` (0.29.1 installed); context updated.

- 2026-11-06 · nuclearlang-4a · **Shared contracts (ADR-0098, asked by the user: step 3 of "Next", the OpenAPI of the
  backend, the option without a monorepo, and in the extension a contracts view and an HTTP client like Bruno).**
  Compiler: `crates/nuclear-transpile/src/contract.rs` (the contract of a project from the AST, in Nuclear notation and
  the same for every dialect: models with rules/key/doc comments, controllers + routes with param placement, client
  repositories as calls; `openapi()` 3.1; `write` with the card and into the output folder; `declared`/`sync`/`install`
  /`generate`/`fetch` for a client), `project::build` runs `contract::sync` first (`contracts/<name>/models/*.nuc`,
  `<name>_error.nuc`, one repository per controller; only decorators the dialect consumes; `local = true` for keyed
  GETs), `contracts` manifest section + `contracts/*/models` kind folder in the four Dragonfly dialects, N0307
  (contract unreadable), N0116 (unknown member of a closed project declaration: the drift error; `check_project_member`),
  `nuclear contracts list|update` (url via curl); two generic emitter fixes found by the example: named arguments to a
  method of a typed receiver are reordered (`callee_params` through `static_type`), and a project name a library also
  exports gets a `hide` clause (`[exports]` section + the type definitions; `dialects/dart` lists fpdart's names);
  `fields_required` declared in the contract (it broke the bundle check). Runtimes: `dragonfly-py` and `dragonfly-nest`
  serve `/.nuclear/contract.json` and `/.nuclear/openapi.json`. Example `examples/examples-flutter-dialect/tasks_app`
  over `tasks_api` (contracts committed; `dragonfly build` + 19 tests + analyze 0); `tasks_api`'s listener writes
  `event["title"]!` (a map read is `V?`: pyright caught it). Test `a_client_builds_from_the_backend_contract_and_sees_drift`
  (same contract with nestjs and fastapi; drift → N0116). Editors 0.29.0 (asked by the user; the editors session agreed):
  `editors/vscode/src/{http,httpPanel}.ts` (Nuclear HTTP view: projects by platform → components → endpoints, contracts
  with Update, OpenAPI documents, environments `.nuclear/http.json`, request panel, copy as curl), `react` in the
  dialect lists, bump + package + jetbrains/zed builds + install. Docs: `spec/projects.md` §1.2, `spec/{backend,mobile,
  web,diagnostics,dialects}.md`, `docs/dragonfly/contracts.md` (+ menu, llms, index, README), ADR-0098 + indexes, plan
  003 marked delivered, context (nuclear, editors/vscode, dragonfly-backend), editors README. Checked: cargo test 203,
  clippy 0, editors 49 + clippy 0, cli 25 + clippy 0, every spec-check 0, dragonfly.py 27/0, stdlib 30/30, react-check,
  nestjs-check, fastapi-check (the check scripts now relink their runtime), every example and external project
  `nuclear check` clean, website build, `./install.sh` (0.29.0 installed). Not run: the HTTP view inside a running VS
  Code, a url contract against a deployed API (Untested).

- 2026-11-06 · nuclearlang-4a · Three tasks asked by the user. (1) **Plans with examples** for steps 3, 4 and 6 of
  "Next": `specs/003-shared-contracts/plan.md`, `specs/004-backend-parity/plan.md`, `specs/005-language-1-0/plan.md`
  (+ `specs/README.md`). (2) **Web platform, dialect `react`** (ADR-0097: the mobile platform in a browser; the
  analysis of zustand / TanStack Query / react-hook-form is in the ADR for the user to decide): `dialects/web/platform.toml`
  (16 kinds, 21 decorators, no plugins), `dialects/web/react` (extends typescript; React 19 without JSX, React Router 7,
  Vite 7, Vitest 4 + jsdom + Testing Library; `[toolchain]`, `[project] output = "build/app"`; `docs.toml`), runtime
  `packages/dragonfly-web` (npm `dragonfly-web`, ESM, 7 vitest tests; `install.sh` copies it), defs
  `defs/typescript/{react,react_router,vitest,testing_library_react,dragonfly-web}`, example
  `examples/examples-web/characters` (golden `examples/expected/characters`; `tools/react-check/check.sh`: tsc --strict
  0, 4 tests, vite build), `spec/web.md`, `spec/react.md` (3 equivalences, check.py 0), `docs/context/dragonfly-web/`,
  `docs/dragonfly/react.md` + website menu/llms/index, `nuclear help web|react`, ADR indexes, `AGENTS.md` row. Generic
  emitter rules (no kind name in Rust): `[kinds.<k>.syntax] construct`, named args of `named_params` kinds kept as one
  object with `named_args = "object"`, template flag `fields_required` (`spec/dialects.md`). (3) **The thesis test
  system** `tools/thesis/` (README step by step; `tasks.toml` 4 tasks × arms nuclear/native; `run.py` with `claude -p`,
  not run: paid; `bench.py` 2 programs × 3 languages vs hand-written; `report.py` charts with the data-viz palette
  validated + `results/REPORT.md`). The bench found and fixed a python dialect bug: a map read `m[k]` raised KeyError
  (now `m.get(k)`; emitter flag `map` on `index`, `assign_target`; `spec/python.md` §4; goldens). Bench run: identical
  output in 6/6, worst generated/hand ratio 1.11 (thesis B holds); Nuclear lines / hand-written 1.03 and 0.84 (thesis C
  does not hold on two tiny CPU programs: the base language is as long as the target there; the gain is in the
  framework kinds, which the agent tasks measure). Checked: cargo test 202, clippy 0, stdlib-check 30/30, spec-check
  python/fastapi/react 0, react-check, `./install.sh --no-vscode --no-zed`, website build. Not run: the web app in a
  real browser, the agent arms of the thesis (Untested / Next step 7).

- 2026-11-06 · nuclearlang-4a · The competitiveness plan recorded as "Next" (asked by the user: the list of the
  analysis is the next work, in its order); no code changed.

- 2026-11-06 · nuclearlang-4a · Competitiveness analysis for the user (what Nuclear and Dragonfly lack; checked
  against this file, the PRD, `docs/context/nuclear/limits.md` and the code): the assessment is in the chat; the
  lists did not change.

- 2026-11-05 · nuclearlang-02 · Editors 0.28.2 released (rule 4 of `editors/`: flutter dialect `@entity("table")`,
  `LocalDatabase` in config and `defs/dart/dragonfly` regenerated by nuclearlang-4a, ADR-0096 second step): bump,
  editors tests 14 + 35, `npm run package` (check-bundle ok), `jetbrains/build.sh`, `zed/build.sh`, `./install.sh`
  (0.28.2 installed, `code --list-extensions` checked). The "Pending: editors/" note is resolved. Not run in an editor.

- 2026-11-05 · nuclearlang-4a · Local database, second step (asked by the user: the backend's equivalences the engine
  allows, one database for every Flutter platform, encryption in the config files). `@entity("table")` on a model names
  its local table (mobile contract + consumed, `model.tpl` `decor.entity.arg`, docs.toml); `local: LocalDatabase =
  LocalDatabase(encrypted = true)` in `appConfig` and in a component's `config` (`AppConfig.local`, `LocalDatabase`,
  `Local.configure/settings`, `Local.register(table, component:)` from `dragonfly_g.tpl`, `Local.open(name,
  component:)`; the AES key is made at the first encrypted box); `isNull()` tried as a dialect function and refused
  (N0204: the word is the `flutter_test` matcher), kept as a limit. Example `clean-flutter-project` (`@entity
  ("characters")`, `local` in `project_config.nuc`), goldens, `spec/flutter.md` §6.1–6.3 (+ `dragonfly.py --fix`),
  `spec/dragonfly.md` §3.2, `spec/mobile.md` rule 6, ADR-0096 (decisions 1 and 4, limits), context, website README,
  defs of `dragonfly@0.1.0` regenerated (100 symbols). Checked: package 64 tests + analyze 0, cargo test 202, clippy 0,
  spec-check flutter 3/0, dragonfly.py 27/0, `./install.sh --no-vscode --no-zed`, clean-flutter-project build + 34
  tests + analyze (2 infos as before), genkit-assistant build + 4 tests, cli 25, website build. Not run: a device or
  web (Untested). The editors bundle the flutter dialect and the dragonfly defs: they need a rebuild + bump again
  (Pending: editors/).

- 2026-11-05 · nuclearlang-02 · Editors 0.28.1 released (rule 4 of `editors/`: the flutter dialect and `defs/dart/dragonfly`
  changed with ADR-0096, nuclearlang-4a): bump in `vscode/package.json`, `plugin.xml`, `zed/extension.toml`, READMEs,
  context; editors tests 14 + 35, `npm run package` (check-bundle 7 dialects ok), `jetbrains/build.sh`, `zed/build.sh`,
  `./install.sh` (0.28.1 installed). The "Pending: editors/" ADR-0096 note is resolved. Not run: in a running editor.

- 2026-11-05 · nuclearlang-4a · Local database of the mobile platform (ADR-0096, asked by the user: the backend's ORM
  on the device, automated by `@repository(saveLocal = true)`, with `<method>AsLocal()` twins). Runtime
  `packages/dragonfly/lib/src/local/local.dart` (`Local`, `LocalTable<T>` with CRUD + the backend's query builder over
  JSON rows, `LocalQuery`, operators `gt`…`between`, `NotFound`; hive_ce boxes encrypted with a key kept in the secure
  storage, in memory in tests), `Repository.send(local:)`, `recall`, `target`, `local<T>()`, `saveLocal`,
  `RepositoryError` (`HttpError` and `NotFound`; `attempt` catches both), `Dragonfly.reset` closes it,
  `DragonflyTest.start` sets `Local.memory`; package tests 63 (`test/local_test.dart`), analyze 0. Compiler (generic):
  `DeclInfo.field_decor` → `ret_info.fields_decor.<decorator>` (`emit.rs`), `field_decor` of every declaration in the
  project context (`project.rs`), `{{#a or b}}` in templates (`template.rs`, `spec/dialects.md`). Dialect: decorators
  `primary`, `generated`, `created`, `updated` (platform contract + consumed), `model.tpl` writes `static final
  LocalTable<M> table`, `dragonfly_g.tpl` registers it, `repository_method.tpl` adds `local:` and the AsLocal twin for a
  GET whose model has a key (`{{#deco_args.local or owner_decor.repository.saveLocal}}`), `generated_tests.tpl` tests
  only http methods and calls each twin; `docs.toml` (kind, `@repository`, `@get`, the four decorators). Example
  `clean-flutter-project`: `Character` `@primary`, `CharacterRepository(saveLocal = true)` + `recent(count)` local
  query; goldens, `spec/flutter.md` §6.2–6.3 (blocks by `dragonfly.py --fix`), `spec/mobile.md` (rule 6),
  `spec/dragonfly.md` §3.2, `docs/context/dragonfly-mobile/*`, `docs/dragonfly/README.md` (website), ADR indexes.
  Defs of `dragonfly@0.1.0` regenerated (only that package, pinned to the existing dart 3.13.5 / flutter 3.47.7 defs;
  the installed SDK is 3.13.7 / 3.47.8). Checked: cargo test 202, clippy 0, cli 25 + clippy 0, editors build,
  spec-check flutter 3/0, dragonfly.py 27/0, website build, `./install.sh --no-vscode --no-zed`, clean-flutter-project
  `dragonfly build` + 34 tests + analyze (2 infos as before), genkit-assistant build + tests. Not run: a real device or
  web (Untested); the editor extensions were not rebuilt (Pending: editors/).

- 2026-11-05 · nuclearlang-02 · Editors follow the project's platform and dialect (asked by the user: a fastapi project
  was checked as flutter, N0201 `controller`; this repository showed "dialect 'flutter' could not be loaded" because
  the installed server 0.27.1 predated ADR-0086/phase C and read the workspace's newer `dialects/`). `editors/lsp`:
  `project::find` reads `config.nuc` (`Manifest::read_dragonfly`), `dialects::load` takes the project root and sets
  `Loader.packages` (`<project>/kinds`, `<install>/kinds`) and searches `install_dir()`, dialect cache by `name@root`
  when the project has `kinds/`, `features::kind_source` in hover (`· native of the `backend` platform`) and
  completion (`labelDetails`), `--check-bundle` loads every bundled dialect; the `.vsix` and the JetBrains zip bundle
  every dialect and `defs/`. Project card `.nuclear/project.json` (name, platform, dialect, language, output, plugins,
  kinds with their source): `nuclear_transpile::project::{METADATA_FILE, metadata, write_metadata}`, written by
  `nuclear build|check`, `dragonfly build` (both branches) and the LSP's `Project::build`; the VS Code status bar
  shows `backend · fastapi` from it (watcher), `nuclear.dialect` and the quick pick list every dialect and apply only
  outside a project. Tests: `backend_example_builds_with_every_backend_dialect` checks the card, editors
  `a_dragonfly_project_uses_the_dialect_of_its_config`. Docs: `spec/projects.md` §1.1, `editors/vscode/README.md`,
  `docs/context/editors/{CONTEXT,lsp,vscode}.md`, `docs/context/cli/CONTEXT.md`. Checked: cargo test (workspace) green,
  editors 14 + 35, cli 25, clippy 0 in the three, `npm run package` (check-bundle 7 dialects ok), `jetbrains/build.sh`,
  `zed/build.sh`, `./install.sh` (0.28.0 installed), the installed server driven over JSON-RPC on the external python
  project (fastapi, 0 diagnostics) and on this repository (flutter and nestjs examples, 0 diagnostics, no message).
  Not run: the extensions inside a running editor (Untested).

- 2026-11-05 · nuclearlang-02 · Analysis of the pending work for the user (checked against the lists here and the
  code after phases A–E and the CLI step): the recommendation is in the chat; the lists did not change.

- 2026-11-05 · nuclearlang-02 · CLI update after the refactor (asked by the user): `[toolchain]` section in dialects
  (`Dialect.toolchain`; fastapi and nestjs declare `install`, `run`, `test` with `{{packages}}`), `dragonfly build |
  install | run | test` for backend projects (`backend_manifest`, `backend_command`: `nuclear build .` + the toolchain
  command with `sh -c` in `build/`), `dragonfly new -p backend` writes `SQLite(file = ..., createTables = true)` and
  `plugins(sqlite = true)` and a first test (`test/app_test.nuc`, so `dragonfly test` has something to run);
  `spec/dialects.md` §3, `docs/context/cli/CONTEXT.md`, the Python and NestJS guides.
  Found on the way: `dragonfly install` got a stale runtime because setuptools reused `packages/dragonfly-py/build/lib`
  (a wheel built before B2); `install.sh` now drops that folder from the copied runtime. Checked: cli 25 tests +
  clippy 0, cargo test 202, `./install.sh --no-vscode --no-zed` run, `dragonfly new -p backend -d fastapi` +
  `build` + `install` + `test` end to end with the installed binaries.

- 2026-11-05 · nuclearlang-02 · Platforms refactor, phases D (bounded) and E (first part): `spec/mobile.md` §3–§4 hold
  the platform rules and architecture, `spec/flutter.md` §1/§4 keep what Flutter adds; kind packages (ADR-0095):
  `Loader.packages` + `kind_packages`/`kind_package_at`/`apply_packages` in `model.rs` (a package of the dialect's
  platform is a plugin; its `<dialect>/` folder is a layer named after it; `[plugin]` gives packages/defs/names),
  `Dialect.kind_sources` (language / platform / package), `package_gaps` → N0306 in `check_config`, a package's
  `docs.toml` read first, `nuclear kind new <name> -p -d` (skeleton), `dragonfly add <folder>` (loads with every
  dialect it implements, copies to `<install>/kinds/`), both CLIs add `kinds/` and `<install>/kinds` to the loader;
  tests `kind_packages_extend_a_platform_dialect` (loader) and `kind_package_builds_with_its_dialects_only`
  (projects); `spec/dialects.md` §7 rewritten, `spec/projects.md` §3.3, `spec/diagnostics.md` N0306, the kinds guide
  §3 (package form), ADR-0095 "Delivered", context files. Checked: cargo test 202, clippy 0 (workspace and cli),
  smoke `nuclear kind new` + `nuclear build .` (fastapi) + the nestjs gap + `dragonfly add` with the installed
  binaries, website build.

- 2026-11-05 · nuclearlang-02 · Platforms refactor, phase C (standard library, ADR-0094) and the B2 conformance example:
  vocabulary in `crates/nuclear-dialect/src/library.rs` (bases, members, result types, wrong target names, printed by
  `nuclear spec`), `[library.<base>]` section (`dialects/{typescript,dart,python}/library.toml`; `helper` written once
  per file, also for `[functions]` and `[syntax]` entries), emitter receiver typing (`static_type`: literals, locals,
  fields, declared and alias signatures, `inject<T>()`, `Result.value/error`, library results, callback parameters,
  `this` in extensions, catch variables as `Error`, destructuring), library rewrite with N0114/N0115, a dialect's own
  base members (`"key".tr()`), `Error(message)`/`e.message` portable (`NuclearError` in Dart, `Exception` in Python,
  `AppError` in the backends), Python `print`/`x!`/named groups/`is int` fixed for conformance, compound operators
  without a target form (`>>>=`, `**=`, `??=`) expanded, bitwise operands of comparisons parenthesized in TypeScript,
  the module-named auto-import (`import math`), naming kept out of library lookups (`src_name`). `examples/stdlib`
  unified: 10 programs × 3 languages, `tools/stdlib-check.sh` (TypeScript, Dart, Python with pyright; `UPDATE=1`).
  Specs (`spec/{dialects,typescript,dart,python,fastapi,nestjs,flutter,nuclear-base}.md`, briefs, `docs.toml`), guide
  chapter 14, website guides, every example and the external projects rewritten with the library's names; `check.py
  --fix` used. Backend example moved to `examples/examples-backend/tasks_api`; Python runtime `@status(code)`.
  Checked: cargo test 200, clippy 0, stdlib-check 30/30, every spec-check 0 (dragonfly.py 27/0), fastapi-check (+ the
  conformance build: pyright 0, pytest 4), nestjs-check (tsc 0, node:test 4, smoke, runtime 3), analyze-flutter 0,
  clean-flutter-project analyze (2 infos as before) + 34 tests, genkit-assistant analyze 0 + 4 tests, external
  rick_and_morty check 41 files, external nuclear_langgraph_agent pyright 0 + pytest 3, editors and cli build, website
  build; `./install.sh --no-vscode --no-zed` run. Not run: the stdlib examples are the only live run of the library in
  Dart/TypeScript/Python outside the examples' tests.

- 2026-11-05 · nuclearlang-02 · Platforms refactor, phase B2 (backend unified, ADR-0093 §8): emitter `decor.<d>.path` +
  `[options] route_param` (nestjs `":name"`: `@get("{id}")` → `@Get(':id')`); Python runtime: route segments without a
  leading slash (`_segment`), `SQLite(file)`, `MariaDB`, plain `postgres://`/`mysql://` URLs get their async driver,
  `Chroma(collection, embeddings, url)` on a server, 400 for an invalid body, in-process `publish` without Kafka
  (`Listener.__new__` registry, `running(listeners=...)`, conftest creates them), graph states as pydantic models
  (`state.field`, `state_field.tpl` defaults), tests `test/*_test.nuc` → `tests/`; Nest runtime: `SQLite({ file,
  createTables })`, `Kafka({ servers })`, `Embeddings('provider:model')`, `Chroma` + `ChromaStore` over chromadb
  (`@langchain/community` rejected: peer conflicts with zod 4 / ioredis 6; ioredis pinned to 5.x), `vectorStore`;
  plugins `mariadb` + `events` in fastapi, `chroma` in nestjs (no N0305 warnings; loader test); `app(version,
  description)` reach pyproject.toml / package.json (`Manifest.version/description`, template var `app.*`); both
  examples, goldens, `spec/{fastapi,nestjs,backend,projects}.md`, docs.toml, briefs, website guides, the external
  `nuclear_langgraph_agent` rewritten. Found and fixed: the B1 rename had renamed
  `tools/node/node_modules/@langchain/langgraph-checkpoint` to `fastapi-checkpoint` (and the lock file); `tools/node`
  now installs the runtime with `--install-links` (a symlink gave two `@nestjs/common` copies to tsc). Checked: cargo
  test 199 + goldens, clippy 0, fastapi-check (pyright 0, pytest 2, smoke, runtime 2), nestjs-check (tsc 0, node:test
  4, smoke, runtime 3), spec-check fastapi 12/0 and nestjs 12/0, editors and cli build, external project pyright 0 +
  pytest 3, website build; `./install.sh --no-vscode --no-zed` run. Not run: Chroma/Embeddings against a real server
  (both runtimes), MariaDB, Kafka with the new `servers` option.

- 2026-11-05 · nuclearlang-02 · Status summary for the user (checked first: context hook lists no unrecorded files,
  cargo test 199 green, dialect folders and examples as recorded here); nothing changed.

- 2026-11-05 · nuclearlang-02 · Kinds as libraries (asked by the user): review of how a kind is made today (data only:
  `[kinds.*]` + template, no kind name in Rust; a kind added to a fresh `dragonfly new -p backend` project with a
  project dialect of two files built with `nuclear build .`), ADR-0095 (native kinds in the platform contract,
  extension kinds as packages on top of a dialect, portability rule, never the compiler), `AGENTS.md` rule 11,
  ADR-0093 §5, `spec/dialects.md` §1/§7 (what is true today for a project's kinds), `docs/context/dragonfly/CONTEXT.md`
  rule 12, website guide `docs/dragonfly/kinds.md` (menu, `llms.txt`, README link), "To decide" item on project
  dialects closed, phase E listed in Half done. Then (the user): a package belongs to one platform and the editor
  must tell kinds apart (ADR-0095 §2, §7; note for the `editors/` session in its CONTEXT). Checked: website build.

- 2026-11-04 · nuclearlang-02 · Platforms refactor, phase B1 (ADR-0093): `dialects/mobile/flutter`,
  `dialects/backend/{fastapi,nestjs}` (`langgraph` renamed `fastapi` everywhere: spec, brief, examples
  `examples-fastapi`, `tools/fastapi-check`, skill, website, the external project), `platform.toml` contracts, loader
  lookup `<platform>/<name>` + `Dialect.platform` + N0305 contract check (kinds exact, consumed decorators and plugins
  within the platform; missing plugins = warnings; loader test), `config.nuc` manifest `dragonfly(platform, dialect)`
  + `app(name)` (`Manifest::read_dragonfly`, `Manifest.platform`, `project::output_dir`, N0303 checks in
  `check_config`; the backend examples lost their `nuclear.toml`; flutter examples and the external apps got the
  line), `nuclear help library|backend|mobile`, `cli/`: `dragonfly new -p -d` (backend skeleton), `manifest()` reads
  `config.nuc` and refuses non-flutter projects; context notes for `cli/` and `editors/`. Checked: cargo test 199,
  clippy 0, cli 25 + clippy 0, editors build, spec-check fastapi/nestjs/flutter 0, dragonfly.py 27/0, fastapi-check,
  nestjs-check, website build, `dragonfly new -p backend -d fastapi` + `nuclear build .` by hand, the external
  rick_and_morty `dragonfly check` 41 files. Not run: `dragonfly new` (mobile) end to end with Flutter.

- 2026-11-04 · nuclearlang-02 · Platforms refactor, phase A (asked by the user; ADR-0093, ADR-0094): rule 11 in
  `AGENTS.md`; context regrouped by platform (`docs/context/dragonfly-mobile/`, `dragonfly-backend/`, map and
  architecture updated); `spec/backend.md` (the platform in Nuclear only, unified routes/connections/plugins/tests),
  `spec/mobile.md` (index over flutter.md), `spec/library.md` (Nuclear's standard library), `spec/dialects.md`
  (platform contract, lookup), `spec/projects.md` (`config.nuc` with `dragonfly(platform, dialect)`); dialect spec
  headers, skill, website guide and menu. Code phases B1–D listed in Half done. Checked: website build.

- 2026-11-04 · nuclearlang-02 · Website docs of the new dialects (asked by the user): `docs/dragonfly/README.md`
  (the three targets, plugins), `python.md` (langgraph), `nestjs.md`, `genkit.md`; menu, home page and `llms.txt`
  updated. Following the guides' steps outside the repository found and fixed three bugs of `dragonfly-nest`: it
  could not build alone (no devDependencies), `index` re-exported `testing` (a production install without
  `@nestjs/testing` crashed) → new entry point `dragonfly-nest/testing` (dialect auto_imports, defs regenerated), and
  npm 12 blocked `sqlite3`'s install script → the generated `package.json` approves it (`allowScripts`). Checked: both
  guides run as written in a scratch folder (Python: pytest 2, POST/404 over HTTP; NestJS: npm test 4, POST/GET,
  `--omit=dev` import), cargo test 198, nestjs-check, spec-check nestjs 0, website build.

- 2026-11-04 · nuclearlang-02 · Plugins + Genkit (ADR-0092, asked by the user): `[project.plugins]` in dialects +
  `plugins(...)` in `config.nuc` (N0303 checks, N0304 for a kind/name of a disabled plugin; `project::enabled_plugins`;
  template vars `plugins.*`); catalogs for flutter (genkit + 5 providers), langgraph (9) and nestjs (10); Python and
  TypeScript manifests list only enabled packages; runtimes split into optional modules (`dragonfly.ai`/`.events`,
  `dragonfly-nest/ai`/`/events`, lazy kafkajs/ioredis); `cli/`: `dragonfly build` adds/removes plugin packages
  (`.dragonfly_plugins`, `runtime_package`). Genkit: runtime `packages/dragonfly_genkit` (providers, tools, chat +
  ChatController, GenkitFlow + generateObject, RemoteFlows, Knowledge, Schemas, testing), flutter kinds tools/chat/flow/
  remote/knowledge, `lib/dragonfly_genkit.g.dart`, Dart defs genkit/schemantic/dragonfly_genkit, example
  `examples/examples-flutter-dialect/genkit-assistant` (golden). Emitter: `async_methods` for methods without await,
  `async` in `methods`. Checked: cargo test 198, clippy 0, all spec-checks 0, langgraph + nestjs checks, cli 25 + clippy
  0, editors build, website build, genkit-assistant analyze 0 + 4 tests, dragonfly_genkit 3 tests, live against local
  Ollama (chat with tools, structured flow, embeddings), remote flows against a Genkit HTTP server; install run.

- 2026-11-03 · nuclearlang-02 · Dragonfly for NestJS (ADR-0091, asked by the user; decided: @nestjs/sequelize +
  sequelize-typescript, `nestjs` becomes Dragonfly, scope CRUD + Redis/Kafka + LangChain.js): `dialects/nestjs` (16
  kinds; ESM for NestJS 12; tests with node:test), runtime `packages/dragonfly-nest`, TypeScript defs generator
  `tools/defs/typescript/` + `defs/typescript/` (11 packages), `tools/node/`, example `examples/examples-backend/tasks_api`
  + new `users.nuc`, `tools/nestjs-check/check.sh`; generic emitter: `relative`, `alias_via = "constructor"`, `route`,
  `field_decor`, `components`, `tinfo`/`decorated` in `field`, `type_args_as_args` names, `doc_text`/`doc_literal`;
  TypeScript writes `== null`/`!= null`. Also: Kafka consumers create their topics (TS and Python); langgraph models
  take the same rule decorators (`@minLength`...; `dragonfly.validators`). Checked: cargo test 198, clippy 0, every
  spec-check 0 problems, nestjs-check (tsc strict 0, node:test 4, HTTP smoke, runtime 3), langgraph-check, by hand with
  local Ollama (agent + planner) and real Redis + Kafka in temporary containers (both runtimes), editors build, cli 25,
  website build; install run.

- 2026-11-03 · nuclearlang-02 · External examples (asked by the user): `~/Documents/projects/flutter/rick_and_morty` moved
  to `~/Documents/projects/nuc/flutter/rick_and_morty` (rebuilt, `dragonfly check` [], analyze 0 errors; paths updated in
  `docs/context/tech.md`, `tools/agent-eval/run.py`); new `~/Documents/projects/nuc/python/nuclear_langgraph_agent`
  (tasks CRUD + agent with tools + planner graph, FastAPI, SQLite, local Ollama; README, AGENTS.md): pyright strict 0,
  its Nuclear tests 3, and run by hand with uvicorn + qwen3.5:4b (CRUD, 404/409, agent lists/adds/completes tasks, the
  graph created 5 tasks). Fixes found on the way: a Result method of a base in the type definitions was not unwrapped
  (`ClassDef.results`, kind `inherits` as bases, error subtypes, `parse_type`); string annotations lost their generics
  in `tools/defs/python/gen.py`; auto-imports only for names the code writes; Python `is None`/`is not None`
  (`null`, `left_infix`/`right_infix`); snake_case temporaries; `uses.<Name>` in project templates (pyproject extras by
  connection, `core` extra); a `tools` method without a doc comment is a helper, not a tool (a real model failed on its
  schema; `FakeModel` now validates tool schemas). Checked: cargo test 198, clippy 0, spec-check 0 problems, langgraph
  check, editors build, cli 25; install run.

- 2026-11-03 · nuclearlang-02 · Rule 10 in `AGENTS.md` (asked by the user): quality first, the right solution even
  when it costs more; no shortcuts presented as decisions.

- 2026-11-03 · nuclearlang-02 · Correction asked by the user: the `.nuc` of python/langgraph had been told to use
  `snake_case`, breaking Nuclear's naming rule. Now the `.nuc` keeps camelCase and option `snake_case_names`
  (`crates/nuclear-transpile/src/naming.rs`) writes PEP 8 names (`getOne` → `get_one`; names that exist in the defs
  stay; strings and map keys untouched). rick_api, tour.nuc, spec/python.md, spec/langgraph.md, briefs, docs.toml,
  ADR-0089/0090 rewritten; runtime `in_` → `one_of` (Nuclear `oneOf`). Checked: cargo test 198, clippy 0, spec-check
  python 15/0, langgraph 12/0, langgraph-check (pyright strict 0, pytest, smoke), editors build; install run.

- 2026-11-03 · nuclearlang-02 · Agents "as if trained" (ADR-0088): N0022 habits of TS/Dart/Python/Java with fixes
  (`crates/nuclear-syntax/src/habits.rs`), N0219/N0220 from the type definitions in projects (`params` in index.json,
  `tools/defs/index_params.py`, Dart generator), `nuclear help <word>|words --dialect`, `llms.txt`/`llms-full.txt` on
  the website, skill/brief/AGENTS template. Python (ADR-0089): `dialects/python` (pyright --strict), generic emitter
  options (hoist_block_lambdas, init_fields, doc_inside, f-strings, module_separator, toplevel_gap, type_args_as_args,
  builtin_result, file_prelude, super_ctor, filter strip_suffix), `tools/defs/python/` + `defs/python/` (18 packages),
  `tools/python/` venv. Dragonfly for Python (ADR-0090): `dialects/langgraph` (16 kinds), `packages/dragonfly-py`,
  example `examples/examples-langgraph/rick_api`, `tools/langgraph-check/check.sh`; spec/python.md, spec/langgraph.md,
  briefs, docs.toml, context `dragonfly-langgraph`. Checked: cargo test 197, clippy 0, cli 25, editors build, spec-check
  typescript 29/0, dart 30/0, flutter 3/0, python 15/0, langgraph 12/0, dragonfly 27/0, rick_api pyright strict 0 +
  pytest 2 + smoke, runtime tests 2, website build; `./install.sh --no-vscode --no-zed` run.

- 2026-11-03 · nuclearlang-02 · Agent evaluation `agent` mode completed (3 empty-project tasks rerun after the usage
  limit): 10/10 green, no clear gain over the baseline; recorded in the plan line and ADR-0087.
- 2026-11-03 · nuclearlang-02 · Agents (ADR-0087): `spec/brief/` (base, typescript, dart, flutter; spec-check 0) and
  `nuclear spec --brief`; skill `agents/skills/nuclear` installed by `install.sh` (`--no-agent`) in
  `~/.claude/skills`, `~/.agents/skills`, `~/.codex/skills`; `nuclear agent init|hook` (`crates/nuclear-cli/src/agent.rs`),
  `dragonfly agent init` (also in `dragonfly new`); `tools/agent-eval/` (10 tasks, `claude -p` with the user's
  account): baseline 10/10 green, 9.8 turns, ~460k tokens/task, USD 5.1. Operations (ADR-0086): `**`, `~/`, bitwise,
  shifts, compound ops, `0b`, exponents, `r"..."`; fixes: TS `is` basic types, `(5).toString()`, Dart `print(a, b)`,
  `for k, v` / `in` on maps, `try` as last line, Dart top-level variables, N0113; `examples/stdlib/` (10 areas × 2
  targets, `tools/stdlib-check.sh`: 20/20 same output, TS with `tsc --strict`); guide chapter 14 (`nuclear help
  operations`); spec base, grammar, reference, typescript.md, dart.md, diagnostics.md, context. Checked: cargo test
  194, clippy 0, cli 25 + build, editors build, spec-check typescript 29/0, dart 30/0, flutter 3/0, dragonfly 27/0.
  Spanish leftovers of the tree in spec/flutter.md §3 translated.
- 2026-11-03 · nuclearlang-02 · The 11 Dragonfly ⟨proposals⟩ accepted (ADR-0085): 6 were already done; implemented
  async guards (`FutureOr<bool>`, `Acl.check`, `Guards.verify`, `canOpenAsync`, async `AclBuilder`), `ScreenTransition`
  per screen, routes nested by address, events with `eventName` + JSON between engines (`Events.decoder`; event kind
  uses json_serializable), generated tests with the enum's first value and converter `sample()`. Package, router,
  templates `event`/`usecase_method`/`dragonfly_g`/`generated_tests`, `emit.rs` (`tinfo.enum`, `converter_sample`),
  defs of dragonfly regenerated, example (`Plan`, `UriConverter`, slide transition), spec/flutter.md, spec/dragonfly.md,
  docs.toml, context. Checked: cargo test 191, package 58 tests + analyze 0, example app analyze (2 infos as before)
  + 34 tests, spec-check flutter 3/0 and dragonfly 27/0. `./install.sh --no-vscode --no-zed` run.
- 2026-11-03 · nuclearlang-02 · Listed the not-started work for the user (from this file; nothing changed).
- 2026-11-03 · nuclearlang-02 · Documentation refactor step 2: all user docs translated to English and renamed
  (`spec/{dialects,definitions,projects,diagnostics,reserved-words,decisions}.md`, `docs/language/`,
  `docs/dialects/create-with-agent.md`, `docs/architecture.md`, plans → `specs/00N-*/plan.md` + `specs/README.md`);
  shared example vocabulary (saludar→greet, edad→age...); stale facts fixed on the way (dart.md Result table,
  guide "projects do not exist", tutorial assets note, base README links). Checked: tests 191 + cli 25 + editors
  48, clippy 0, spec-check typescript 29/0, dart 30/0, dragonfly 27/0, guide 0 problems, website build green;
  `./install.sh --no-vscode --no-zed` run (help in English). Captions `Output (`/`Generated (` in
  `tools/spec-check/dragonfly.py` and `tests/projects.rs`.
- 2026-11-03 · nuclearlang-02 · Documentation refactor step 1: `AGENTS.md` (L0), `docs/context/` (L1–L3 per
  future repository), 84 ADRs in `docs/adr/` (`spec/decisions.md` = index), this file replaces `ESTADO.md`;
  `CLAUDE.md`, `cli/CLAUDE.md`, `editors/CLAUDE.md` reduced to imports; `docs/dragonfly/contexto.md` removed
  (moved to `docs/context/dragonfly*`); hooks point to `STATUS.md`. Decisions: ADR-0082 (English), 0083
  (repositories), 0084 (layered context). Verified: links of the tree resolve, every L1–L3 file has
  front-matter, `cargo test` 191 green, website build green. Memory files updated.
- 2026-11-03 · nuclearlang-02 · Consultation on layered documentation methods (progressive disclosure,
  AGENTS.md, Memory Bank, C4/arc42, ADR, PRD, Diátaxis); the user's model is
  `~/Documents/projects/store-checkout` (its ADR-0010).
- 2026-11-03 · nuclearlang-02 · Context strategy: single status file, hooks `tools/context/check.sh` in
  `.claude/settings.json` (README in `tools/context/`). Script tested by hand.
- 2026-11-03 · nuclearlang-02 · Pending review: logs (ADR-0078) and the architecture linter (ADR-0046) were done
  but listed as "to design"; gap: async errors outside Flutter bypass `Log`. Tests 191 + 25 (cli), clippy green.
- 2026-10-28 · several · Dragonfly devices from the CLI (ADR-0081); Zed extension (tree-sitter, same LSP, tasks);
  orphan processes fixed (editors 0.27: marker + reaper, panel idle 5 min, LSP idle 5 min, `watch` exits with
  its parent).
- 2026-10-27 · several · Logs (ADR-0078, `[call_site]`); editor docs and doc comments (ADR-0079, editors
  0.25); ` * ` comment lines (ADR-0080, N0021); extensions per framework (ADR-0077: `[docs]`, `[project]
  marker/output`, `Manifest::find`); mixed debugging (ADR-0076, editors 0.22, VS Code only); Dragonfly device
  (ADR-0074, editors 0.20–0.21; the package works on web); `config.nuc` without empty sections and assets by
  convention (ADR-0070, 0071); LSP loads project definitions (editors 0.19); `.nuc` debugging with a line map
  (ADR-0068, editors 0.18); `dragonfly repair` (editors 0.17); dialects with agents (ADR-0072, 0073, plan only).
- 2026-10-26 · several · Events on demand, session, ACL, routes, `@screen` (ADR-0052–0057); `nuclear help`;
  `dragonfly` CLI as subproject with aliases and VS Code workspace; tutorial; `Result<E, T>` error first with
  fpdart (ADR-0047, 0048); tests in Nuclear (ADR-0049); `.nuc`-only projects (ADR-0050); publishable package
  (ADR-0051); forms, dialogs, translations, splash (ADR-0058–0063); `dragonfly build` ~40 ms with markers;
  editors 0.13–0.16 (`@screen` completion, Dragonfly run/debug/save in VS Code and Android Studio).
- 2026-10-25 · several · Golden rule (ADR-0038), flutter dialect as framework (ADR-0039), type definitions
  (ADR-0040) replacing `symbols.toml`.
- 2026-10-22 · several · Function aliases (ADR-0037).
- 2026-10-21 · several · `Result` with automatic propagation (ADR-0036); destructuring, `yield`, bounded
  generics, `if` in lists, cascades (ADR-0027–0033); `@override`/`super` (ADR-0034); versioned definitions
  (ADR-0035); validated with `tsc --strict` and `flutter analyze` (0 warnings).
- 2026-10-20 · several · Lexer, parser, AST, emitter, dialect loader, CLI, five dialects; any Flutter/Dart class
  usable (auto-imports from the SDK); first editor support.
