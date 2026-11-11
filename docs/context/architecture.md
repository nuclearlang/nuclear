---
layer: L1
component: system
read-when: you need the shape of the system, which repository owns what, or how data flows from .nuc to target code
---

# Architecture

## 1. Layers of the platform

```
              ┌──────────────────────────────────────────────────────────────┐
  editors     │ nuclear-vscode · nuclear-jetbrains · nuclear-zed            │  run the LSP binary
              │ dragonfly-flutter-vscode · dragonfly-flutter-jetbrains      │  (framework extras)
              └───────────────▲──────────────────────────────▲──────────────┘
                              │ LSP protocol                  │ runs
              ┌───────────────┴───────────┐   ┌──────────────┴──────────────┐
  tools       │ nuclear-lsp (server)      │   │ dragonfly CLI               │
              │ nuclear CLI               │   │ (new, build, run, devices)  │
              └───────────────▲───────────┘   └──────────────▲──────────────┘
                              │ Rust API (crates)              │
              ┌───────────────┴────────────────────────────────┴─────────────┐
  compiler    │ nuclear-syntax → nuclear-transpile (resolver, emitter,       │
              │ projects) ← nuclear-dialect (loader, templates, defs)        │
              │ nuclear-diag (diagnostics everywhere)                        │
              └───────────────▲──────────────────────────────────────────────┘
                              │ data (TOML + templates + defs JSON)
              ┌───────────────┴──────────────────────────────────────────────┐
  dialects    │ language: typescript, dart, python (+ nodejs)                │
              │ platforms (Dragonfly, ADR-0093): mobile/flutter · backend/   │
              │ {fastapi, nestjs} · web (to do)   runtimes: packages/        │
              └──────────────────────────────────────────────────────────────┘
```

## 2. Compiler pipeline

```
 .nuc text                                   dialects/<name>/*.toml (+ defs/)
   │ 1 LEXER     text → tokens (INDENT/DEDENT)        │ loaded at run time:
   │ 2 PARSER    tokens → AST (no format)             │ inheritance, merge, validation
   │ 3 RESOLVER  block scope, undefined names,        ▼
   │             implicit this, const/let, async ◄── Dialect: kinds, shortcuts, auto-imports
   │ 4 EMITTER   AST → dialect templates        ◄── [syntax] [types] [operators] [options]
   ▼
 .ts / .dart text  (+ line map target → .nuc for debuggers)
```

Each stage knows only the previous one. Diagnostics accumulate in all of them (text or JSON).

## 3. Design rules

1. The lexer turns indentation into tokens; inside `( ) [ ] { }` it emits none (line continuation).
2. The AST has no format; every node keeps its `Span` for errors and the LSP.
3. The parser is generic: dialect words (`stateless`, `static`) arrive as data in a declaration header.
4. The transpiler knows no target language: TypeScript and Dart are dialects like any other.
5. Data before code: dialects, shortcuts and imports are files read at run time; the same files feed the
   CLI, the LSP and the future MCP server.
6. Imports are decided after emitting, from the words of the generated code (`emit::code_words`).

## 4. Repositories (ADR-0083) and current folders

The working tree is the `nuclear` repository with every other repository as a folder inside it, each with its own
git (plan 006):

| Future repository | Current folders | L2 context |
|---|---|---|
| `nuclear` | `crates/`, `languages/`, `dialects/{typescript,dart,python,nodejs}`, `defs/{dart/dart@*,python/python@*,typescript/typescript@*}`, `tools/{defs,spec-check,stdlib-check.sh,context}`, `spec/` (base, library, dialects, grammar, diagnostics), `docs/language/`, `examples/` (non-Dragonfly, `stdlib/`), `install.sh`, `crates/nuclear-lsp/` | [nuclear](nuclear/CONTEXT.md) |
| `nuclear-vscode`, `nuclear-jetbrains`, `nuclear-zed` | `nuclear-vscode/` (with `theme/`, `vendor/`, `scripts/`), `nuclear-jetbrains/`, `nuclear-zed/` | [editors](editors/CONTEXT.md) |
| `dragonfly` | `dragonfly/`: the CLI crate, `docs/dragonfly/{README,kinds,tutorial}.md`, `docs/context/{dragonfly,cli}/` | [dragonfly](https://github.com/nuclearlang/dragonfly/blob/main/docs/context/dragonfly/CONTEXT.md), [cli](https://github.com/nuclearlang/dragonfly/blob/main/docs/context/cli/CONTEXT.md) |
| `dragonfly-mobile` | in `dragonfly-mobile/` (moved, plan 006): `dialects/mobile/flutter/` (→ `dialects/mobile/flutter/`), `packages/{dragonfly,dragonfly_genkit}/`, `defs/dart/{flutter,bloc,dio,genkit,...}`, `spec/{mobile,flutter,dragonfly}.md`, `docs/dragonfly/{tutorial,genkit}.md`, `examples/examples-flutter-dialect/`, `tools/analyze-flutter.sh` | [dragonfly-mobile](https://github.com/nuclearlang/dragonfly-mobile/blob/main/docs/context/dragonfly-mobile/CONTEXT.md) |
| `dragonfly-backend` | in `dragonfly-backend/` (moved, plan 006): `dialects/{langgraph,nestjs}/` (→ `dialects/backend/{fastapi,nestjs}/`), `packages/{dragonfly-py,dragonfly-nest}/`, `defs/{python,typescript}/` (packages), `tools/defs/{python,typescript}/`, `tools/{python,node}/`, `tools/{langgraph,nestjs}-check/`, `spec/{backend,langgraph,nestjs}.md`, `docs/dragonfly/{python,nestjs}.md`, `examples/examples-{langgraph,nestjs}/` | [dragonfly-backend](https://github.com/nuclearlang/dragonfly-backend/blob/main/docs/context/dragonfly-backend/CONTEXT.md) |
| `dragonfly-web` | in `dragonfly-web/` (moved, plan 006): `dialects/web/react/`, `packages/dragonfly-web/`, `defs/typescript/{react,react_router,vitest,testing_library_react,dragonfly-web}`, `tools/react-check/`, `spec/{web,react}.md`, `docs/dragonfly/react.md`, `examples/examples-web/` | [dragonfly-web](https://github.com/nuclearlang/dragonfly-web/blob/main/docs/context/dragonfly-web/CONTEXT.md) |
| `dragonfly-flutter-vscode`, `dragonfly-flutter-jetbrains` | Dragonfly parts of `nuclear-vscode/src/{extension,debug,device}.ts`, `nuclear-jetbrains/.../NuclearDragonfly.java`, `NuclearDebug.java` | [editors](editors/CONTEXT.md) |
| `website` | its own repository (`~/Documents/projects/nuc/react/nuclearlang_website`) | [website](https://github.com/nuclearlang/website/blob/main/docs/context/website/CONTEXT.md) |

Boundaries that hold after the split:
- Extensions talk to `nuclear-lsp` only through the LSP protocol (plus the documented `nuclear/*` requests).
- Framework extensions never register the language or the LSP; they activate on their dialect's project
  marker and prefix commands, settings and debug types with their dialect.
- The `dragonfly` CLI uses `nuclear-transpile::project` and `nuclear-dialect::Loader` as a library; what it
  needs from a platform comes from the dialect (`[toolchain]`, planned; today `"flutter"` is hard-coded).
- Website content stays with the code it documents; the site copies it at build time.

## 5. Crates

| Crate | Input → output |
|---|---|
| `nuclear-diag` | `Diagnostic`, `SourceFile`, codes N00xx (syntax), N01xx (resolver), N02xx (dialect), N03xx (project); text and JSON |
| `nuclear-syntax` | text → `Lexed` (tokens) → `Parsed` (AST + diagnostics); `DocComment` |
| `nuclear-dialect` | `dialects/<name>/*.toml` + `defs/` packages → `Dialect` + `Report`; templates, contract, `Dialect.typedefs` (lazy `api.json`) |
| `nuclear-transpile` | text + `Dialect` → `Output` (code + diagnostics + line map); `check()` for the LSP; `project::build` |
| `nuclear-cli` | the `nuclear` binary: build, check, tokens, ast, spec, dialect, defs, help (embedded docs) |
| `nuclear-lsp` | the language server binary |
