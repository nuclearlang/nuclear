# AGENTS.md — NuclearLang platform (L0)

NuclearLang is a minimal base language for AI agents and people. A Rust transpiler turns `.nuc` into other
technologies through **declarative dialects** (TOML + templates loaded at run time), one way only
(`.nuc` → `.ts` / `.dart` / `.py`). **Dragonfly** is the framework: three **platforms** (`web`, `mobile`,
`backend`), each written once in Nuclear and transpiled by the **dialects** grouped under it (`backend/fastapi`,
`backend/nestjs`, `mobile/flutter`; `web/react` later). Product: [docs/context/product/prd.md](docs/context/product/prd.md).

## Golden rule

**Performance, speed and readability of the syntax**, for the base language and EVERY dialect (ADR-0038):
generated code as fast as an expert's, instant transpilation, minimal readable Nuclear.

## Hard rules

1. **English everywhere** (ADR-0082): code, comments, diagnostics, CLI output, dialects, examples, docs,
   ADRs. Exceptions: localization data and unicode tests. **The chat with the user is always in Spanish.**
2. **The parser does not change per dialect.** Framework words, shortcuts and imports live in `dialects/`.
   Rust decides *what* is emitted; the dialect decides *how*.
3. `spec/nuclear-base.md` holds only final rules and only Nuclear. Each dialect's spec holds its rules and
   the real Nuclear → target equivalences (checked by `tools/spec-check`, 0 problems). No questions, history
   or "pending" in specs: doubts go to the chat; decisions become ADRs.
4. Every new diagnostic code is documented in `spec/diagnostics.md`; every dialect word in its `docs.toml`.
5. Done = tests + clippy green, docs updated in the same change, `STATUS.md` updated
   ([docs/context/workflow.md](docs/context/workflow.md)). Golden output changed → `UPDATE_GOLDEN=1` + review.
6. **`STATUS.md` is the only list of state and pending work** for the whole platform. Every task (also a
   report) ends with a Log line and current lists; half-done work is recorded the moment it is left. Hooks in
   `.claude/settings.json` check it.
7. The working tree is the `nuclear` repository with the other repositories as folders inside it (plan 006):
   `dragonfly/` (the CLI), `dragonfly-{mobile,backend,web}/`, `nuclear-{vscode,jetbrains,zed}/`, `nuclear-graph/`,
   `nuclear-developer/` (and `nuclear-developer/data/`, the knowledge base `nuclear-knowledge`); each has its own git
   and is ignored by the root's. If `ast.rs` or a public API of `crates/` changes, build `dragonfly/` too
   (`cargo build --manifest-path dragonfly/Cargo.toml`); the LSP is `crates/nuclear-lsp`. No commits unless the user
   asks; commits only in the user's name.
8. `CodeBuilder/` is teaching material only: never in specs, context or code.
9. Node.js/Express is postponed by the user: do not touch it until asked.
10. **Quality first: always the right solution, even when it costs more** (decided by the user, 2026-11-03). No
    shortcut that bends a rule, leaves a known gap or moves work to the user (e.g. asking the `.nuc` to break
    Nuclear's naming so a dialect is easier): build the generic mechanism instead. If the right way is unclear or
    too big, stop and ask; never ship the shortcut as a "decision".
11. **Three platforms; the dialects only change the language underneath** (decided by the user, 2026-11-04,
    ADR-0093). Dragonfly has exactly three platforms: `web`, `mobile` and `backend`. A platform is what the user
    and the agent write: its kinds, decorators, folders, `config.nuc`, plugins and tests, in Nuclear, identical
    for every dialect of that platform. A dialect (`backend/fastapi`, `backend/nestjs`, `mobile/flutter`...) is
    only the language and framework the code comes out in. **The Nuclear code of a project never changes when
    its dialect changes**; the one exception is code that uses a dialect's type definitions (`numpy` exists only
    in Python), which is allowed and ties that code to that dialect. Kinds and features may differ between
    platforms, never between the dialects of one platform. The standard library is Nuclear's own
    (ADR-0094, `spec/library.md`): every dialect translates it; a target's library name (`push`, `len`,
    `toUpperCase`) in platform code, specs, docs or examples is a bug. A platform's spec names no dialect. **Kinds are Nuclear's libraries** (ADR-0095): the native kinds
    of a platform live in its contract; extension kinds are packages a project or the community writes on top of a
    dialect with TOML, templates and a runtime only, never touching the compiler (a missing template capability is
    asked for as a generic one); a package belongs to one platform, and one implemented for some of its dialects ties
    the project to them, like type definitions. The editor shows where every kind comes from and lists only the
    kinds of the project's platform.

## Where things are (open only what the task needs)

| Area (future repository, ADR-0083) | Folders | Start here |
|---|---|---|
| System: layers, repositories, data flow | — | [docs/context/architecture.md](docs/context/architecture.md) |
| Commands, versions, SDK paths | — | [docs/context/tech.md](docs/context/tech.md) |
| `nuclear`: language, compiler, `nuclear` CLI, LSP, language dialects | `crates/` `languages/` `dialects/{typescript,dart,python,nodejs}` `defs/` `tools/` `examples/` `spec/` | [docs/context/nuclear/CONTEXT.md](docs/context/nuclear/CONTEXT.md) |
| `dragonfly`: platforms, philosophy, the `dragonfly` CLI | `dragonfly/docs/context/dragonfly/` (the library: `spec/library.md` here) | [docs/context/dragonfly/CONTEXT.md](https://github.com/nuclearlang/dragonfly/blob/main/docs/context/dragonfly/CONTEXT.md) |
| `dragonfly-mobile`: the mobile platform; dialect `flutter` + Dart runtime | `dragonfly-mobile/` (its own repository inside this one, plan 006): `dialects/mobile/flutter/` `packages/{dragonfly,dragonfly_genkit}/` `spec/{mobile,flutter,dragonfly}.md` | [dragonfly-mobile/docs/context/dragonfly-mobile/CONTEXT.md](https://github.com/nuclearlang/dragonfly-mobile/blob/main/docs/context/dragonfly-mobile/CONTEXT.md) |
| `dragonfly-backend`: the backend platform; dialects `fastapi` (Python) and `nestjs` (TypeScript), their runtimes | `dragonfly-backend/` (its own repository inside this one, plan 006): `dialects/backend/{fastapi,nestjs}/` `packages/{dragonfly-py,dragonfly-nest}/` `defs/{python,typescript}/` `spec/{backend,fastapi,nestjs}.md` | [dragonfly-backend/docs/context/dragonfly-backend/CONTEXT.md](https://github.com/nuclearlang/dragonfly-backend/blob/main/docs/context/dragonfly-backend/CONTEXT.md) |
| `dragonfly-web`: the web platform; dialect `react` + TypeScript runtime | `dragonfly-web/` (its own repository inside this one, plan 006): `dialects/web/react/` `packages/dragonfly-web/` `spec/{web,react}.md` | [dragonfly-web/docs/context/dragonfly-web/CONTEXT.md](https://github.com/nuclearlang/dragonfly-web/blob/main/docs/context/dragonfly-web/CONTEXT.md) |
| `dragonfly` CLI | `dragonfly/` (its own repository inside this one: `dragonfly/` crate, `docs/dragonfly/{README,kinds,tutorial}.md`, `docs/context/{dragonfly,cli}/`) | [docs/context/cli/CONTEXT.md](https://github.com/nuclearlang/dragonfly/blob/main/docs/context/cli/CONTEXT.md) |
| LSP and editor extensions | `crates/nuclear-lsp`, `nuclear-vscode/` (with `theme/`), `nuclear-jetbrains/`, `nuclear-zed/` (each its own repository inside this one) | [docs/context/editors/CONTEXT.md](docs/context/editors/CONTEXT.md) |
| AI: the private knowledge base (RAG) + the MCP server; the local 3B agents | `nuclear-graph/`, `nuclear-developer/`, `nuclear-developer/data/` (the knowledge base, generated: `nuclear-graph knowledge build`) (each its own repository inside this one; ADR-0102); the Loop tool is in `nuclear-vscode/` | [nuclear-graph/README.md](nuclear-graph/README.md), [nuclear-developer/README.md](nuclear-developer/README.md), [nuclear-developer/data/README.md](nuclear-developer/data/README.md) |
| Website | the `website` repository (Nuclear, Dragonfly web; today `~/Documents/projects/nuc/react/nuclearlang_website`) | [docs/context/website/CONTEXT.md](https://github.com/nuclearlang/website/blob/main/docs/context/website/CONTEXT.md) |
| Why a rule exists | — | [docs/adr/README.md](docs/adr/README.md) |
| What is done / pending | — | [STATUS.md](STATUS.md) |

Full map and loading protocol: [docs/context/README.md](docs/context/README.md). Read top-down: this file →
one `CONTEXT.md` → only the L3 file whose `read-when` matches.

## Most used commands

```sh
cargo test && cargo clippy --all-targets
UPDATE_GOLDEN=1 cargo test
cargo run -q -p nuclear-cli -- build <file.nuc> --dialect flutter
./install.sh                       # the user runs the installed binaries; re-run after changes
```
