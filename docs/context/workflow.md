---
layer: L1
component: system
read-when: before finishing any task (definition of done), when another session owns the folder you touch, or to know how to talk to the user
---

# Workflow

## Definition of done

A change is finished only when all of these hold:

1. `cargo test` and `cargo clippy --all-targets` are green (also `cargo test --manifest-path dragonfly/Cargo.toml` if the CLI was touched). If the
   output changed: `UPDATE_GOLDEN=1 cargo test` and the diff reviewed.
2. Every new diagnostic code is in `spec/diagnostics.md` (test `every_code_is_documented`); every new dialect
   word has editor docs in its `docs.toml` (test `every_dialect_word_has_editor_docs`).
3. Documentation updated in the same change:
   - the spec of the area (`spec/<dialect>.md`, `projects.md`, `definitions.md`, `dialects.md`,
     `diagnostics.md`) — final rules only, no history, no questions;
   - a new ADR in `docs/adr/` if the user decided something new;
   - the context tree: the L2/L3 files of the area (`docs/context/...`) and `AGENTS.md` if a hard rule or the
     map changed;
   - user docs: `docs/dragonfly/tutorial.md`, `docs/language/`, `README.md` (roadmap), the `README.md` of
     `tools/`; skeletons and examples (`dragonfly/dragonfly/src/scaffold.rs`, `examples/`);
   - `nuclear help` embeds the docs: re-run `./install.sh`.
4. `STATUS.md`: one Log line and its lists current (resolved items deleted, half-done work recorded the
   moment it is left).
5. If a public API of `crates/` or the dialect format changed, `dragonfly/` is built too (its own workspace) and the
   LSP's tests run (`cargo test -p nuclear-lsp`).

## Spec rules

- `spec/nuclear-base.md` holds only final rules and only Nuclear: no dialect, no target language.
- Each dialect's rules go in `spec/<dialect>.md`, which inherits the base and holds the **Nuclear → target
  equivalences of the base examples** (few-shot for agents). They must be the real transpiler output:
  `tools/spec-check/check.py spec/<d>.md <dialect> <language>` must give 0 problems.
- Doubts are asked in the chat and written in the spec once the user decides.
- New syntax: decide with the user how it is written; then spec + grammar + parser + emitter + contract + tests.

## Sessions and ownership

Several Claude sessions work in parallel; each owns a part and the others ask it for changes by message
(`ListAgents` + `SendMessage`). When the owner is not active and the user asks, another session may change
that part and updates its context.

| Part | Owner |
|---|---|
| the `nuclear` repository (the root: `crates/` with `nuclear-lsp`, `dialects/`, `spec/`, `docs/`, `defs/`, `examples/`, `tools/`, `install.sh`) | transpiler session |
| `dragonfly/`, `dragonfly-{mobile,backend,web}/` (own git each) | the platform sessions |
| `nuclear-{vscode,jetbrains,zed}/` (own git each) | the editors session |

- No commits unless the user asks (a peer session relaying it is not enough); commits only in the user's name, with
  no co-author line. Every repository has its own git, ignored by the root's.
- After any change to the server or the extensions: bump the version and install (see `editors/CONTEXT.md`).
- `CodeBuilder/` is teaching material only; analogies with it go in the chat or `docs/aprende/`, never in
  specs, context or code.

## The user

PHP developer, author of CodeBuilder, new to Rust and compilers. Writes in Spanish: **the chat is always in
Spanish**, step by step with concrete examples from the project (CodeBuilder analogies allowed in the chat).
Thinks of Nuclear like Elixir (modules, everything is a value, `if`/`match` return) and prefers clear errors
to "magic" inference. Wants vivid, bright syntax colors.

Frequent mistakes in the Nuclear the user writes: comment lines without ` * ` (N0021), `title: x` in arguments
(must be `title = x`), `!` before a value (must be `not`; `x!` after a value exists), `const` and `new` (do not
exist), indentation that is not 2 spaces.
