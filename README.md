# NuclearLang

A minimal base language, made so agents (Claude, Codex, OpenCode...) and people write it fast, that
transpiles to other technologies through **dialects**.

# Vision

NuclearLang avoids writing unnecessary code or text, to spend as few tokens as possible, and it tries to be
extremely easy to use, write and read. NuclearLang avoids at all costs the text that has nothing to do with the
program's logic: it exists to write the program's logic in the easiest way and with the least text possible, and
everything else, which from the point of view of a program's logic is unnecessary, is left to the transpilers.
This helps agents not having to think about unnecessary context: only what matters, the logic of a product.

```
             ┌──────────── typescript dialect ──► .ts
 file.nuc ───┤──────────── nestjs dialect ──────► .ts (modules, controllers...)
  (base)     ├──────────── react dialect ───────► .ts (React)
             ├──────────── fastapi dialect ─────► .py (FastAPI)
             └──────────── flutter dialect ─────► .dart
```

As in an ORM (Sequelize with MySQL, Postgres or SQLite): the user writes one single form and each **adapter**
(dialect) knows how to translate it to its technology.

## Installation

One command, no Rust needed (the release of your system from GitHub, checked by SHA-256, into your user folders; ADR-0101):

```sh
curl -fsSL https://nuclearlang.github.io/install.sh | sh        # Linux and macOS (--version <v>, --uninstall)
irm https://nuclearlang.github.io/install.ps1 | iex             # Windows (PowerShell)
```

The installers are `tools/release/install.sh` and `install.ps1`; `tools/release/package.sh [target]` makes the
archives they download and `.github/workflows/release.yml` publishes them on a tag `v<version>`.

From the source tree:

```sh
./install.sh                 # builds and installs nuclear, dragonfly and nuclear-lsp in ~/.local/bin + the VS Code and Zed extensions
./install.sh --no-vscode     # without the VS Code extension (--no-zed: without the Zed one)
./install.sh --uninstall     # uninstalls
PREFIX=/usr/local ./install.sh
```

It needs Rust (`cargo`); for Flutter apps, Flutter (`FLUTTER` or `FLUTTER_ROOT` if it is not on the PATH). The
dialects, type definitions, specs and the dragonfly package go to `~/.local/share/nuclearlang` (`NUCLEAR_HOME`
chooses another folder).

## The repositories

NuclearLang is split into repositories of the [nuclearlang](https://github.com/nuclearlang) organization, each with
its own release cycle:

| Repository | What it is |
|---|---|
| [nuclear](https://github.com/nuclearlang/nuclear) (this one) | the language: compiler, `nuclear` CLI, language server, the language dialects (TypeScript, Dart, Python), specs |
| [dragonfly](https://github.com/nuclearlang/dragonfly) | Dragonfly, the framework: the `dragonfly` CLI and the platforms' philosophy |
| [dragonfly-mobile](https://github.com/nuclearlang/dragonfly-mobile) | the mobile platform: dialect `flutter`, the Dart runtime |
| [dragonfly-backend](https://github.com/nuclearlang/dragonfly-backend) | the backend platform: dialects `fastapi` and `nestjs`, their runtimes |
| [dragonfly-web](https://github.com/nuclearlang/dragonfly-web) | the web platform: dialect `react`, its runtime |
| [nuclear-vscode](https://github.com/nuclearlang/nuclear-vscode), [nuclear-jetbrains](https://github.com/nuclearlang/nuclear-jetbrains), [nuclear-zed](https://github.com/nuclearlang/nuclear-zed) | the editor extensions |
| [website](https://github.com/nuclearlang/website) | the documentation site, written in Nuclear |

To work on them, clone this repository and the others inside it (each folder is its own git, ignored by this one):

```sh
git clone https://github.com/nuclearlang/nuclear && cd nuclear
for r in dragonfly dragonfly-mobile dragonfly-backend dragonfly-web nuclear-vscode nuclear-jetbrains nuclear-zed; do
  git clone https://github.com/nuclearlang/$r
done
./install.sh
```

How to contribute: [CONTRIBUTING.md](CONTRIBUTING.md). License: [Apache-2.0](LICENSE).

## Documentation website

The documentation site is its own repository, `website`, written in Nuclear with Dragonfly web: it reads the `.md`
files of this repository and of the platform repositories at build time (`node tools/content.mjs`, then `dragonfly
build`). Its README tells how to run it.

## Status

The base transpiler works: lexer, parser, AST, resolver and emitter in Rust (`crates/`), driven by **declarative
dialects** (TOML files in `dialects/`). Shipped dialects: `typescript`, `nodejs`, `nestjs`, `dart`, `flutter`, `python` and `fastapi` (Dragonfly for Python:
FastAPI, SQLAlchemy, pydantic, LangChain, LangGraph).

```sh
cargo test                                                                  # tests
cargo run -q -p nuclear-cli -- build examples/examples-nodejs/chat.nuc --dialect nodejs    # transpile
cargo run -q -p nuclear-cli -- build examples/examples-flutter-dialect/flutter_app.nuc --dialect flutter
cargo run -q -p nuclear-cli -- check examples/examples-nodejs/chat.nuc --dialect nodejs --json   # errors for agents
cargo run -q -p nuclear-cli -- dialect list                                                # available dialects
cargo run -q -p nuclear-cli -- dialect check flutter                                       # validate a dialect
cargo run -q -p nuclear-cli -- spec --dialect flutter                                      # spec for agents: base + dart + flutter
FLUTTER=~/develop/flutter/bin/flutter tools/analyze-flutter.sh              # validate the output with Flutter
```

With the `flutter` dialect every widget and class of Flutter and of the Dart libraries (`dart:async`,
`dart:convert`, `dart:io`...) can be used, besides `bloc`/`flutter_bloc`, `dio`, `get_it` and `json_annotation`:
the imports are added automatically. Everything comes from the **type definitions** (`defs/`, like TypeScript's
`.d.ts`), generated from the SDK with `tools/defs`. `nuclear defs show Scaffold --dialect flutter` shows the API
of any class in Nuclear notation.

| Document | What it contains |
|---|---|
| [docs/dragonfly/tutorial.md](https://github.com/nuclearlang/dragonfly/blob/main/docs/dragonfly/tutorial.md) | **Start here:** a Flutter app from scratch with `dragonfly`, step by step (`nuclear help tutorial`) |
| [docs/language/](docs/language/README.md) | The language guide by chapters, with the real TS and Dart outputs (`nuclear help guide`) |
| [docs/language/reference.md](docs/language/reference.md) | Language reference: the whole base on one page, in tables (`nuclear help reference`) |
| [spec/nuclear-base.md](spec/nuclear-base.md) | Rules of the base language (every dialect inherits them) |
| [spec/typescript.md](spec/typescript.md), [nodejs](spec/nodejs.md), [nestjs](https://github.com/nuclearlang/dragonfly-backend/blob/main/spec/nestjs.md), [dart](spec/dart.md), [flutter](https://github.com/nuclearlang/dragonfly-mobile/blob/main/spec/flutter.md), [python](spec/python.md), [fastapi](https://github.com/nuclearlang/dragonfly-backend/blob/main/spec/fastapi.md) | Spec of each dialect: what it adds to the base, how it translates it and the equivalences of the examples |
| [spec/dialects.md](spec/dialects.md) | The standard dialect format: golden rule, templates, shortcuts, kinds, validation, how to create one with an agent |
| [spec/projects.md](spec/projects.md) | Multi-file projects: `nuclear.toml`, imports between files, `[project]`, architecture rules |
| [spec/flutter.md](https://github.com/nuclearlang/dragonfly-mobile/blob/main/spec/flutter.md) | The flutter dialect (Dragonfly): project structure, architecture, kinds (repository, usecase, appcomponent, statemanager, component, guard, tests...), screens and routes (`@screen`), global events, session, ACL, the `dragonfly` CLI and the editors |
| [dragonfly](https://github.com/nuclearlang/dragonfly) | The `dragonfly` CLI (its own repository): `new`, generators (`g c`, `g s`...), `code`, `watch`, `build`, `run`, `test` |
| [AGENTS.md](AGENTS.md), [docs/context/](docs/context/README.md), [STATUS.md](STATUS.md), [docs/adr/](docs/adr/README.md), [specs/](specs/README.md) | Context for agents and contributors (layered), project status, decision records, feature plans |
| [spec/dragonfly.md](https://github.com/nuclearlang/dragonfly-mobile/blob/main/spec/dragonfly.md) | The Dart package `dragonfly` the generated code uses (it can also be used by hand) |
| [packages/dragonfly/](https://github.com/nuclearlang/dragonfly-mobile/blob/main/packages/dragonfly/README.md) | Dragonfly's Dart runtime |
| [spec/definitions.md](spec/definitions.md) | Type definitions (like `.d.ts`): the `defs/` repository, format, import priority, `nuclear defs` |
| [spec/grammar.ebnf](spec/grammar.ebnf) | Formal grammar |
| [spec/decisions.md](spec/decisions.md) | Index of the decision records |
| [spec/diagnostics.md](spec/diagnostics.md) | Error codes |
| [spec/reserved-words.md](spec/reserved-words.md) | Reserved words per language and how Nuclear reaches each one |
| [dialects/](dialects) | The shipped dialects |
| [docs/architecture.md](docs/architecture.md) | How the transpiler works |
| [docs/aprende/desde-codebuilder.md](docs/aprende/desde-codebuilder.md) | Teaching material (in Spanish): compiler concepts explained from CodeBuilder (not a basis of the design) |
| [examples/](examples) | `.nuc` examples with their expected output (`.ts`, `.dart`); `widgets_tour.nuc` walks through Flutter patterns |
| [defs/](defs) | The type definitions repository: `defs/dart/{dart,flutter,dragonfly,flutter_bloc,bloc,dio,get_it,go_router,connectivity_plus,flutter_secure_storage,json_annotation}/<version>/` |
| [tools/defs/](tools/defs/README.md) | Generator of the type definitions of Dart, Flutter and pub.dev packages |

## Roadmap

1. **Base language** (spec + grammar) ✅
2. Lexer, parser, AST, resolver and emitter ✅
3. Declarative dialects in files, with shortcuts, kinds and automatic imports ✅
4. Dialects `typescript`, `nodejs`, `nestjs`, `dart`, `flutter`, `python`, `fastapi` ✅
5. CLI `nuclear build`, `check`, `dialect`, `spec` ✅ · `nuclear fmt` ← next
6. Language Server (LSP), VS Code extension, JetBrains plugin and Zed extension (`crates/nuclear-lsp`, repositories `nuclear-vscode`, `nuclear-jetbrains`, `nuclear-zed`) ✅ · missing: split a
   base extension for every dialect and one per framework (Dragonfly for Flutter), ADR-0077
7. Multi-file projects (`spec/projects.md`) ✅ · components without an explicit facade (spec §1, principle 6)
8. The Dragonfly framework for Flutter: dialect, `dragonfly` package and `dragonfly` CLI ✅ · Dragonfly devices
   (window and terminal) ✅ · missing: an own engine without a browser for machines with little memory
9. MCP server so an agent creates new dialects (ruby, go...) · `react` dialect
10. New dialects created by agents (Claude, Codex, OpenCode) with rules, process and restrictions
    (`spec/dialects.md` §8, tutorial `docs/dialects/create-with-agent.md`) ✅ · missing: `nuclear dialect
    new|test`, skill + `AGENTS.md`, Dragonfly independent of the language (`dragonfly crystal/amber new`) and
    dialects installable from a repository (`specs/002-dialects-with-agents/plan.md`)
11. Postponed: each project's own kinds and decorators
12. Documentation website (the `website` repository, in Nuclear) and language reference ✅ · missing: create the
    GitHub repository and turn on Pages
13. Layered context documentation (`AGENTS.md`, `docs/context/`, ADRs, `STATUS.md`) ✅ · all documentation in
    English ✅ · repository split (ADR-0083) ← planned

### To do

**CLI**
- ✅ Type definitions of a project's packages: `dragonfly flutter add <package>` adds it to `config.nuc` (the
  project's pubspec in Nuclear) and generates its definitions in `~/.cache/nuclearlang/defs` (ADR-0065–0067).
  Missing: the same command for the other dialects (`typescript`: npm).
- ✅ `config.nuc` without empty sections (ADR-0070) and assets by convention (ADR-0071): fonts in
  `assets/fonts/<Family>/` (weight and style from the name), shaders in `assets/shaders/`, the rest of `assets/`
  as it is; nothing to configure.
- A command to create the AI context documentation (templates) from the language's definitions, its purpose and
  its philosophy. It asks for the minimum: the *what for* and the *why* of the project, not the *how*.

**Extension / editor** (`nuclear-lsp` and the editor repositories)
- ✅ Documentation in the editor (ADR-0079): hover and completion with the definition, examples, what it
  generates, rules and diagrams of each reserved word, type, kind, decorator and modifier (`languages/docs.toml`,
  `dialects/<d>/docs.toml`), and doc comments with `@param`, `@return`, `@example`... that also come out in the
  generated code (JSDoc, `///`). Missing: a warning when a `@param` names a parameter that does not exist.
- Flutter: an assistant to create interfaces, with the Material palette.
- A UI to create and edit the AI context documentation of Claude, Codex, Gemini and opencode.
- A scaffold of that documentation by layers, taking Claude Code's structure as the reference:
  - `CLAUDE.md` / `CLAUDE.local.md`: project vision, stack, conventions and architecture.
  - `.mcp.json`: MCP integrations.
  - `.claude/settings.json` / `settings.local.json`: permissions, model and hooks.
  - `rules/`: one `.md` per topic (style, testing, API), which can apply to concrete paths.
  - `commands/`: own commands for repeatable flows.
  - `skills/<name>/SKILL.md`: loaded only when the task needs them.
  - `agents/`: subagents with a role, their own context and their tools.
  - `hooks/`: scripts before or after using a tool (validate, format, block).
- The UI's layers work as an assistant connected to the AI and as an editor of AI context markdown files (create
  and edit).
- A map of layers: the agent reads only the layer it needs, not the whole context, to save tokens.

### New projects
- **Nuclear and Dragonfly MCP:** an agent uses everything the `nuclear` and `dragonfly` CLIs do (build, check,
  spec, defs, generators, tests) and it can connect with the Flutter MCP and others. It extends point 9.
