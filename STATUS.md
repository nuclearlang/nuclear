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
from the CLI, an MCP server, a 30-minute validated tutorial and comparison tables.

## Half done

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
  deploy (GitHub Pages, SPA fallback) and a first release tag `v0.1.0` (the installers need it).
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
- An `appcomponent` alias with the camelCase name of a use case it aliases (`createProject() = CreateProject.exec`)
  collides with the injected field in TypeScript (found with tasks_api; the example renamed its aliases): a
  diagnostic or another field name is missing.
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

- ADR-0100 limits (2026-11-10): the join table
  name is always `<owner table>_<field>` (no custom name); `@transactional` uses the default database only (nestjs:
  the database of the last repository made, a `shortcut:` comment in `repository.ts`); `Page<T>` members are untyped
  in Nuclear (type definitions carry no member types: `items: Task[] = page.items`); the runtime's pyright run over
  `packages/dragonfly-py/src` shows 4 errors that predate this work (`events.py`, `repository.py` Select typing).
- The user's `~/Documents/projects/nuc/python/nuclear_langgraph_agent` still writes `@generated()` in
  `components/tasks/data/entities/task.nuc`: it is now N0309; the fix is `@primary(generated = true)` (not changed:
  the user's project).

- The `dragonfly` CLI for backend projects has `new`, `build`, `install`, `run`, `test` through the dialect's
  `[toolchain]` (2026-11-05, plan 002 D5 in its first form); missing: `make` generators for backend kinds, `check`
  and `watch` for backend projects, `[toolchain]` for mobile (today `"flutter"` is still hard-coded in `cli/`).
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

## Pending: nuclear (language and compiler)

- `Name<T>.member(...)` (`Set<int>.of`, `List<int>.filled`) does not parse: a generic type before a member; today
  `s: Set<int> = Set.of(...)`. Assessed 2026-11-08 (nuclearlang-4a): clean support needs an additive AST node for a
  generic receiver (e.g. `ExprKind::GenericName { name, type_args }`, since `Call`'s `type_args` go after the method
  for `obj.m<T>()`), which touches the shared `ast.rs` and every exhaustive `ExprKind` match in `nuclear-transpile`
  and `editors/` (rule 7). Deferred as disproportionate for a niche case with a working annotation workaround; do it
  with the owning sessions.

- `nuclear fmt` limits (2026-11-10): only reflow, it does not add trailing commas or wrap long lines; the two
  external projects (`rick_and_morty`, `nuclear_langgraph_agent`) are not formatted yet (only `--check` was run: 5+
  files would change in rick_and_morty, no errors) — run `nuclear fmt .` there when the user wants.
- Components per principle 6 of the base · MCP server · `const` inference in
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
