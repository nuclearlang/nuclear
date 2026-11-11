---
layer: L1
component: system
read-when: you need to know what the platform is for, who uses it, or whether a feature fits the product
---

# Product requirements — NuclearLang platform

## 1. Vision

NuclearLang is a minimal base language built for AI agents and the people who work with them: the
fastest language to write and read, with the fewest tokens. It is not compiled to machine code: a
transpiler written in Rust turns `.nuc` into the code of another technology through **declarative
dialects** (TOML + templates loaded at run time). One way only: `.nuc` → `.ts` / `.dart` / ...; it never
reads target code.

**Dragonfly** is the framework built into each dialect. Writing an app on a platform usually means
choosing and wiring N packages (state, JSON, DI, http, routing, i18n, tests). With Dragonfly the dialect
already brings that architecture: in Nuclear you write only the business logic and the transpiler
generates the rest. The philosophy is common; each platform applies it its own way, because platforms
are incompatible (a Flutter `stateless` widget does not exist in a NestJS backend).

Open source platform, in the spirit of Rust, Deno, Gleam and Bun.

## 2. Users

| User | Needs |
|---|---|
| AI coding agents (Claude, Codex, OpenCode) | few tokens, one way to write each thing, clear diagnostics with fixes, a spec they can follow, type definitions to avoid hallucinated APIs |
| Developers of apps (Flutter first) | write business logic only; a project that builds, tests and runs with one CLI; editor support like the native one |
| Dialect authors | add a target or framework without touching Rust: data files, templates, a contract, tests |

## 3. Golden rule (governs every requirement)

**Performance, speed and readability of the syntax** (ADR-0038): generated code as fast as an expert's
in the target, instant transpilation (read only what is needed), minimal and readable Nuclear. A rule, kind
or shortcut that breaks one of the three is rejected.

## 4. Functional requirements

| ID | Requirement | Status |
|---|---|---|
| FR-01 | Base language: header + block, 2-space indentation, values everywhere (`if`/`match`), modules, inheritance, generics, `Result<E, T>` with automatic propagation, aliases, destructuring, generators, cascades | done (`spec/nuclear-base.md`) |
| FR-02 | Transpile a single file to TypeScript and to Dart | done |
| FR-03 | Dialects as data: inheritance, templates, kinds, decorators, options, keywords, contract (`nuclear spec`) | done (`spec/dialects.md`) |
| FR-04 | Type definitions per package and version (like `.d.ts`): auto-imports, overrides, inherited members, API for LSP and agents | done for Dart/Flutter; TypeScript pending |
| FR-05 | Multi-file projects: one build, cross-file imports, architecture rules (linter) | done (`spec/projects.md`) |
| FR-06 | Diagnostics with codes, positions, fixes; text and JSON | done (`spec/diagnostics.md`) |
| FR-07 | LSP: diagnostics, completion, hover, signature help, rename, metrics, docs from doc comments and `docs.toml` | done |
| FR-08 | Editor extensions: VS Code, JetBrains, Zed (base) + per-framework extensions (run, debug, devices) | base done; split per framework pending |
| FR-09 | Dragonfly for Flutter: kinds (model, repository, usecase, screen, statemanager, form...), runtime package, generated tests | done |
| FR-10 | `dragonfly` CLI: new, make, build, run, test, watch, repair, explain, devices, packages | done (Flutter only) |
| FR-11 | Debug `.nuc` through the target debugger (line map) | done in VS Code; JetBrains partial |
| FR-12 | Dialects created by agents (`dialect new/test`, skills, AGENTS.md) | planned (`specs/002-dialects-with-agents/plan.md`) |
| FR-13 | Agent support: brief spec, eval suite, hooks, defs validation | planned (`specs/001-agent-support/plan.md`) |
| FR-14 | More dialects: NestJS/Node (postponed by the user), React, Rust | planned |
| FR-15 | `nuclear fmt`, MCP server | planned |

## 5. Non-functional requirements

| ID | Requirement |
|---|---|
| NFR-01 | Generated code passes the target's analyzer without warnings (`flutter analyze` with `flutter_lints`, `tsc --strict`) |
| NFR-02 | A no-change `dragonfly build` takes ~40 ms; a project build ~20 ms in release |
| NFR-03 | The parser never changes per dialect; Rust decides *what* is emitted, the dialect *how* |
| NFR-04 | Everything in English (code, docs, diagnostics); Spanish only as website translation and localization data |
| NFR-05 | Every diagnostic code documented; every dialect word documented for the editor |
| NFR-06 | Errors over magic: no inferred parameters, no hoisting; explicit beats implicit when in doubt |

## 6. Non-goals

- Reading or converting target code into Nuclear.
- Records, operator overloading, Dart 3 pattern `switch` (not expressible yet; see `nuclear/limits.md`).
- Replacing the platform's own tools: Flutter's CLI still builds and runs; Dragonfly drives it.
