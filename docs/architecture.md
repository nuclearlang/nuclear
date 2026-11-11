# Transpiler architecture

> The context documentation for contributors and agents (layers, repositories, every component) is in
> [docs/context/](context/README.md); this page is the overview for readers of the website.

## 1. Stages

```
 .nuc text                                    dialects/<name>/*.toml
    │  1. LEXER      text   → tokens                 │
    │  2. PARSER     tokens → AST                    │  loaded at run time
    │  3. RESOLVER   block scopes, undefined         │  (inheritance, collisions, validation)
    │                names, implicit this,           ▼
    │                const/let, async, returns ◄── Dialect: kinds, shortcuts, auto-imports
    │  4. EMITTER    AST → dialect templates   ◄── [syntax] [types] [operators]
    ▼
 .ts / .dart / ... text
```

Each stage knows only the previous one. Diagnostics accumulate in all of them and come out as text or JSON.

## 2. Design rules

1. **The lexer produces indentation as tokens.** `INDENT`/`DEDENT` let the parser work as if there were braces.
   Inside `( ) [ ] { }` they are not emitted (line continuation).
2. **The AST has no format.** No `;`, no spaces, no braces. Each node keeps its `Span` (position in the file)
   for errors and for the LSP.
3. **The parser is generic.** It knows nothing about Flutter or NestJS. A dialect's words (`stateless`,
   `extension`, `static`...) arrive as data in the declaration's header and the dialect validates them.
4. **The transpiler knows no target language.** TypeScript and Dart are dialects like any other: files with
   templates. The Rust code decides *what* to emit (a constant, a `this`, an `await`); the dialect decides *how*
   it is written.
5. **Data before code.** Dialects, shortcuts and automatic imports live in TOML files read at run time. The same
   files feed the CLI, the LSP and the future MCP server.

## 3. Dialects

A dialect is a folder `dialects/<name>/` with TOML files (full format in [spec/dialects.md](../spec/dialects.md)).
Dialects inherit:

```
typescript ──► nodejs ──► nestjs
dart ───────► flutter
```

- **Target** (`typescript`, `dart`): templates for each construct of the base language, types, operators,
  options and reserved words.
- **Framework** (`nodejs`, `nestjs`, `flutter`): kinds (`stateless`, `controller`), `use` shortcuts, automatic
  imports and function shortcuts.
- **Project**: any developer extends a framework with their own shortcuts and kinds.

The loader merges the files, applies inheritance and validates: shortcut collisions, similar names, unknown
template variables and missing templates. `nuclear spec` publishes the whole contract (which templates exist and
which variables each one receives) so an agent can write a new dialect.

## 4. Organization in Rust

```
nuclearlang/
├── Cargo.toml                 # workspace
├── languages/nuclear.toml     # reserved words of the base language
├── dialects/                  # the language dialects: typescript, nodejs, dart, python
├── defs/                      # type definitions (like .d.ts): defs/<language>/<package>/<version>/
├── crates/
│   ├── nuclear-diag/          # diagnostics: codes, positions, text and JSON
│   ├── nuclear-syntax/        # lexer, AST, parser and formatter
│   ├── nuclear-dialect/       # template engine, contract, dialect loading and validation
│   ├── nuclear-transpile/     # resolver + dialect-driven emitter; projects (project.rs); API for CLI and LSP
│   ├── nuclear-cli/           # the `nuclear` binary (build, check, fmt, spec, defs, help)
│   └── nuclear-lsp/           # the language server
├── spec/                      # base language, grammar, library, dialects, diagnostics
├── examples/                  # each .nuc with its expected output; examples/projects/, stdlib/
├── tools/                     # spec checks, type-definition generators, release (installers, archives)
├── install.sh                 # installs nuclear, dragonfly and nuclear-lsp in ~/.local (files in share/nuclearlang)
│
│  the other repositories, each a folder with its own git (plan 006):
├── dragonfly/                 # the `dragonfly` CLI and the platforms' philosophy
├── dragonfly-mobile/          # the mobile platform: dialect flutter, the Dart runtime
├── dragonfly-backend/         # the backend platform: dialects fastapi and nestjs, their runtimes
├── dragonfly-web/             # the web platform: dialect react, its runtime
├── nuclear-vscode/            # the VS Code extension (and the shared theme)
├── nuclear-jetbrains/         # the Android Studio / IntelliJ plugin
└── nuclear-zed/               # the Zed extension
```

| Crate | Input → output |
|---|---|
| `nuclear-syntax` | text → `Lexed` (tokens) → `Parsed` (AST + diagnostics) |
| `nuclear-dialect` | `dialects/<name>/*.toml` + `defs/` packages → `Dialect` + `Report` (diagnostics per file). `Dialect.typedefs` gives the full API of the libraries (lazy read of `api.json`) |
| `nuclear-transpile` | text + `Dialect` → `Output` (code + diagnostics + Dart → Nuclear line map for debuggers). `check()` for the LSP. `project::build(folder, manifest, dialect)` → one file per `.nuc` + project files (spec/projects.md) |

**Imports:** the emitter first writes the code and then decides the imports from the words of that code
(`emit::code_words`: without comments nor the text of strings, with their interpolations). So a name a template
writes is imported, and one that is only in a comment or in a decorator that is not written is not. In a project
the same holds across files and for the files of `[project.files]`.

**Tests:** each `examples/**/x.nuc` has its expected output (`x.ts`, `x.dart`). A test transpiles the `.nuc`
with its dialect and compares with that file (*golden tests*); `UPDATE_GOLDEN=1` regenerates them.
`tools/analyze-flutter.sh` also compiles the Flutter examples with the real SDK (`flutter analyze`). There is
also one test per language rule (`crates/nuclear-transpile/tests/rules.rs`) and per dialect validation rule
(`crates/nuclear-dialect/tests/loader.rs`).
