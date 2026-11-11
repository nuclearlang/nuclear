---
layer: L3
component: nuclear
read-when: you add or change a kind, decorator, option or template of a dialect, or a dialect validation
---

# Dialect format (summary; full rules in `spec/dialects.md`)

## Files

`dialects/<name>/dialect.toml` (several `.toml` are merged; `docs.toml` is skipped by the loader),
`keywords.toml`, `docs.toml`, `templates/*.tpl` (referenced as `{ file = "templates/x.tpl" }`). Inheritance
with `extends`/`replaces`. `nuclear spec` prints the contract: which templates exist and the variables of each.

## Kinds (`[kinds.<name>]`)

Fields: `description`, `template`, `overrides` (methods with `@override`), `requires` (required methods,
N0209), `members` (names the kind brings, like `setState`), `param_access`, `imports`, `rename` (e.g.
`{{name}}Controller`), `[kinds.X.syntax]` (templates overridden inside the kind), `named_params`, `inherits`.
Framework fields: `async_methods`, `methods`, `returns`, `file_header`, `alias_via`, `declares`, `provide`,
`results`, `void_methods`. Template variables: `name`, `members`, `body`, `methods_code`, `fields`,
`params_list`, `ctor`, `decorators`, `decor`, `deco_args`, `param_groups`, `ret_info`, `owner_decor`,
`call_args`, `generic_list`, `tinfo` (full table in `spec/projects.md` §5).

- `fields` carry `decor` (decorators of the field), `decorators` (text of the non-consumed ones) and
  `class_decorator` (first `@PascalCase` without parentheses); `methods` carry `is.<name>`.
- `decor`: an argument that is a list or object also gives its parts (`decor.screen.loggedIn.forward`,
  `decor.screen.providers.items`).
- A kind that would not write generics or interfaces is N0218 (`check_kind_writes`).

## Decorators and names

`[decorators] consumed/void`, `[decorators.args]` (argument schema, N0217), `[decorators.provide]
"deco.arg"`, `[names] written/globals`, `public = ["@screen"]`. The Dart `decorator` template writes
`@Name()` for a PascalCase decorator without arguments.

## Editor docs (ADR-0079, `spec/dialects.md` §5.7)

`languages/docs.toml` = base `[keywords] [builtins] [types] [tags]`; `dialects/<d>/docs.toml` =
`[kinds] [decorators] [words]` (`Dialect::doc`, read on demand). Markdown in English. Test
`every_dialect_word_has_editor_docs`: every kind/decorator/modifier/clause has one. The examples in the
`docs.toml` files parse (checked once with `nuclear check`; no test guards it).

## Other sections

`[docs]` (`Dialect::docs_url`: API reference links for the editor), `[project] marker`/`output`
(`Manifest::find`, ADR-0077), `[call_site.<Module>]`, `[project.manifest]`, `[architecture]`
(see [projects.md](projects.md)).

## Dialects with agents (ADR-0072, 0073)

Rules, process and restrictions in `spec/dialects.md` §8; tutorial `docs/dialects/create-with-agent.md`;
plan (not started) `specs/002-dialects-with-agents/plan.md`: `dialect new/test`, skill + AGENTS.md, neutral
Dragonfly spec, `[toolchain]`, install from a repository. Per-project kinds: postponed.
