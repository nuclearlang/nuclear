---
layer: L3
component: editors
read-when: you change the LSP server (crates/nuclear-lsp) or need how completion, hover, projects or packaging work
---

# The LSP server (`crates/nuclear-lsp`, future `nuclear-lsp`)

## Modules

| File | What it does |
|---|---|
| `src/rpc.rs` | JSON-RPC over stdio (`Content-Length`) |
| `src/text.rs` | byte offsets ↔ LSP positions (line + UTF-16 column) |
| `src/index.rs` | walks the AST: declarations (`Binding`) and uses (`Occurrence`) with semantic token and target; block scope; deferred `Enum.item` and named arguments; `doc_comment` |
| `src/features.rs` | semantic tokens, hover, completion, signature, definition, references, rename, outline, folding, quick fixes (any `data.fix` of a diagnostic), `on_type_formatting` |
| `src/server.rs` | state (open documents, dialect), dispatch, diagnostics, `nuclear/transpile`, code lens and metrics commands |
| `src/metrics.rs` | code metrics (copy of CodeMetrics/tsmetrics-core) |
| `src/typedefs.rs` | `Dialect.typedefs`: prefix index of public names and extensions per type, `split`, `instance_members` (own, inherited, extensions, generics substituted), `static_items`, `static_call_type` |
| `src/project.rs` | multi-file projects: root (`nuclear.toml` or the dialect marker via `Manifest::find`), `Project::build`, `visible` (test/ isolated), `knows`, `default_kind` (copied), `with_project_defs` (`build/defs.lock`, roots `<project>/defs` + cache; warnings in config.nuc for missing packages), rebuild when `defs.lock` changes |
| `src/dialects.rs` | dialect search paths, keyword help, `builtin_doc` |
| `tests/server.rs` | end-to-end with `examples/` (including "no prefix of any example panics"; `project_packages_bring_their_definitions`) |

## Decisions

- Synchronous server, only `serde_json`; full re-analysis per change (`.nuc` files are small). It exits on stdin
  EOF; VS Code stops it 5 min after the last `.nuc` closes.
- Semantic tokens: LSP standard + modifier `kind` (dialect kinds) and `deprecated`. Named arguments colored as
  parameters (italic), not properties.
- Types of `x` in `x.` are known only from written text or construction: annotation, typed parameter,
  `x = Class()`, `Class.named()`/`Class.static()` (its return, `Binding.inferred`), module field, inherited member
  of an external base (`Binding.bases` + kinds' `inherits`). Unknown → nothing offered. Generics substituted
  along the chain (`Cubit<int>` → `BlocBase<int>` → `state: int`).
- Free names: sorted lowercase index (binary search, max 300, `isIncomplete`); first `api.json` read ~16 ms, each
  completion < 1 ms in release.
- External class hover: `render` without header comments (shown below as doc, package, library), cut at 40
  lines (`nuclear defs show` gives all).
- Projects: root is the closest folder with `nuclear.toml`, `config.nuc` naming `dragonfly(platform, dialect)` or
  `main.nuc` (same order as `Manifest::find`); the project's dialect is loaded with the kind packages of
  `<project>/kinds` and `<install>/kinds` (`dialects::load`), cached by name, or by `name@root` when the project
  has `kinds/`; `nuclear.dialect` only applies to files outside a project. `Project::build` writes the project card
  `.nuclear/project.json` (`project::write_metadata`, spec/projects.md §1.1) for the clients. Hover and completion
  of a kind say its source (`features::kind_source`: language, native of the platform, package). Built from disk on
  first open and on every save (~20 ms in release for the 26-file example), cached per root. A file equal to disk takes the project's diagnostics (N0301/N0302/N0303, imports, other files'
  names) and `nuclear/transpile` returns its project output; with unsaved changes it is checked alone (no N0106
  for names the project knows). Other files' names: completion, hover ("Declared in …"), go to definition; their
  members (`Object::Project`) and named params of `Class(`.
- Decorators: completion after `@` and hover; inside `@screen(...)` its arguments and object fields (required
  first) from `decorator_args`, detected by text (`decorator_context`).
- Docs (ADR-0079): hover of kinds/decorators/words from `Dialect::doc` + dialect data; keywords, `Result/Ok/Err`
  and base types from `base_doc`; doc comments in hover, completion (`documentation`, deprecated tag) and
  signature help (each `@param` next to its argument).
- Bundles: the `.vsix` and zip include `nuclear-lsp`, every dialect of the repository in `server/dialects/` and
  `../defs` in `server/defs/`; both scripts end with `nuclear-lsp --check-bundle`, which loads each bundled dialect
  (fails on any diagnostic, e.g. N0213). The server also searches `nuclear_dialect::install_dir()` (the
  `install.sh` copy, used by Zed's `~/.local/bin/nuclear-lsp`).

## Pending (in `STATUS.md`)

Types of chained expressions and calls (`a.b.`, `inject<X>().`, `Ok(c) -> c.`), generic substitution in shown
signatures, project rules with unsaved changes, `nuclear fmt`.
