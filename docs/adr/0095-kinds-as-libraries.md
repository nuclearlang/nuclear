---
adr: 0095
title: "Kinds are Nuclear's libraries: native kinds of the platform, extension kinds as packages"
status: "Accepted"
scope: dragonfly
---

# ADR-0095: Kinds are Nuclear's libraries: native kinds of the platform, extension kinds as packages

- Status: Accepted
- Scope: `dragonfly`, `nuclear` (dialect loader, manifest), `cli`, `website`
- Date: 2026-11-05 (decided by the user)

## Context

The user's rule (2026-11-05): kinds are to Dragonfly what libraries are to a language. There will be **native
kinds** and **kinds uploaded by the community**, written as extensions of the dialect a developer uses. What matters
is that a developer never has to touch the parser, the lexer or the emitter: they get a framework that makes a kind
easy to write.

What exists today (checked against the code, 2026-11-05):

- A kind is data, not code: `[kinds.<word>]` in a TOML file plus a Mustache template (`spec/dialects.md` §5.4,
  `spec/projects.md` §5). No kind name is written in Rust: the parser sees "header + block", the loader reads the
  TOML, the emitter renders the template with the declaration's data (`decor`, `params_list`, `tinfo`, `route`,
  `methods_code`, `components`...) and the kind options (`requires`, `methods`, `returns`, `async_methods`,
  `inherits`, `provide`, `alias_via`, `declares`, `file_header`, `syntax`). `nuclear dialect check` validates it.
- A project can already add a kind on top of a dialect with two files (`dialects/<name>/dialect.toml` with
  `extends = "fastapi"` and a `.tpl`), and `nuclear build .` emits it: tried on a fresh `dragonfly new -p backend`
  project. The platform is inherited; the generated code is the template's.
- What a developer cannot do yet: share or install a kind (there is no unit smaller than a dialect, and no
  repository: plan 002 D6); know which dialects a kind exists for; keep `dialect = "fastapi"` in `config.nuc`, since
  the project must name the child dialect instead, which hides the real dialect from the CLI and the editors; test a
  kind outside the repository (`nuclear dialect test`, plan 002 D1); and every framework written so far needed new
  generic emitter capabilities (`STATUS.md` Log: NestJS, Python, Genkit, ADR-0085), which only the `nuclear` team
  can add: templates have no expressions, only sections and seven filters.
- ADR-0093 says a dialect may not add a kind its platform does not declare, and the loader refuses it (N0305); a
  project dialect was left unchecked ("To decide" in `STATUS.md`).

## Decision

1. **Two sources of kinds.** *Native kinds* are the platform's: listed in `dialects/<platform>/platform.toml`,
   implemented identically by every dialect of the platform (ADR-0093). *Extension kinds* come in **packages**
   written by a project or by the community on top of one or more dialects of a platform. A dialect's own TOML
   never adds kinds (N0305 stays): what a dialect would add goes in a package, so any other dialect can implement it.
2. **A kind package is a plugin that is not shipped with the dialect.** It has the format of `[project.plugins.<name>]`
   (`spec/projects.md` §3.3: `kinds`, `names`, `defs`, `packages`) plus its `[kinds.*]`, `[decorators]`, templates
   and `docs.toml`, in its own folder, with one implementation per dialect it supports. **A package belongs to
   exactly one platform** (`platform = "backend"`; the user, 2026-11-05: extensions come for `web`, `mobile` and
   `backend`): its kinds never appear in a project of another platform, even when the word exists there with
   another meaning (`repository` is table queries in backend and HTTP calls in mobile). A project enables it in
   `config.nuc` with `plugins(<name> = true)`, exactly like a shipped plugin; the project keeps
   `dragonfly(platform = "backend", dialect = "fastapi")`: the dialect is always the platform's dialect, never a child
   dialect that wraps it. The exact file layout is fixed in phase E (`STATUS.md`).
3. **Portability rule.** A package declares the dialects it implements. The Nuclear of the kind is the same in all of
   them. A project that enables a package implemented for only some dialects of its platform is tied to those
   dialects, the same exception rule 11 gives to type definitions; the compiler says which dialects the project can
   still build with and what is missing. A package may gain implementations later without changing its Nuclear.
4. **A kind never touches the compiler.** A package holds TOML, templates, a runtime package in the target language,
   type definitions and docs; nothing else. A kind adds no syntax: it is a header + block of the base grammar, and
   its words are kinds, decorators, modifiers and clauses (`spec/dialects.md` §8.4). When a template cannot express
   something, the author asks for a **generic** capability (an `[options]` entry or a template variable), and the
   `nuclear` team adds it for every kind (plan 002 D7); never a case for one kind or one language.
5. **Project dialects (`spec/dialects.md` §7)** keep their shortcuts. The kinds they add are extension kinds of
   that project under rule 3: allowed, tied to that dialect, not checked by the platform contract. Phase E turns
   them into a local package with the same format as a published one. This closes the "To decide" item.
6. **Owed to kind authors** (phase E; plan 002 D1, D2, D6 in the same order): `nuclear kind new <name>` (skeleton,
   spec with the Nuclear side written), `nuclear dialect test` over a package's spec, `dragonfly add <source>`
   (installs a package after `check` and `test`, pins its version in the project), one `docs.toml` per package read
   by the editors, the missing-implementation diagnostic of rule 3, and the website guide `docs/dragonfly/kinds.md`.
7. **The editor tells kinds apart.** The loader gives every kind its source: the language's (`dart`'s `data`,
   `mixin`), native of the platform (the contract), or an extension package (name, version, platform, the dialects
   it implements); today `Entry.origin.dialect` only says which dialect file defined it. The Language Server lists,
   for a project, only the kinds of its platform (the native ones plus the packages its `config.nuc` enables), shows
   each kind's source and docs in completion and hover (the package's own `docs.toml`), and reports with the fix a
   kind of a package made for another platform or not implemented for the project's dialect. The same word may mean
   different things in two platforms; the editor always shows the meaning of the project's platform.

## Consequences

- `AGENTS.md` rule 11 names the two sources of kinds and the portability rule; `spec/dialects.md` §1 and §7 state
  what is true today for a project's kinds; the website gets the guide; `STATUS.md` records phase E and drops the
  "To decide" item.
- Until phase E, a project's kind lives in a project dialect named in `config.nuc`; the tutorial says so.
- Reviewed and kept: the template engine (Mustache subset with filters) and the fixed data it receives. The cost of
  that choice is that new data comes from Rust; the benefit is that a kind can only produce code an expert reads
  (golden rule, ADR-0038). It is revisited only if phase E and the first community packages show that templates are
  not enough.

## Delivered

- 2026-11-05 (phase E, first part): the package format (a `kind.toml` with `[package]`, a `docs.toml`, one folder
  per dialect such as `fastapi/kind.toml` with `[plugin]`, the dialect sections and templates), the loader
  (`Loader.packages`, a package of the platform is a plugin; `Dialect.kind_sources`, `package_gaps`, `extra_docs`),
  N0306, `nuclear kind new`, `dragonfly add` with the package's folder. Pending: the editor showing a kind's
  source, `nuclear dialect test` over a package, installing from a repository with a pinned version (plan 002 D1, D6).

## Alternatives rejected

- **Kinds inside a child dialect named in `config.nuc`** (today's §7 used for sharing): the real dialect disappears
  from the manifest, nothing says which dialects the kind exists for, and there is nothing to install.
- **A kind API in Rust or WebAssembly** (a plugin computes the output): it would let a kind fake syntax and emit
  code no template reviewer reads; it moves the parser problem to the author, the opposite of the goal.
- **Letting each dialect declare extra kinds** (dropping N0305): the Nuclear of a platform would drift again
  between its dialects, which ADR-0093 forbids.
