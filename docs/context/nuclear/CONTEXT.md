---
layer: L2
component: nuclear
read-when: you touch crates/, languages/, the target dialects, defs/ of Dart, tools/, examples/ or the base specs
---

# nuclear — language, compiler, CLI, LSP

Future repository `nuclear` (ADR-0083). The base language and everything that turns it into code.

## Code map

| Path | What it does |
|---|---|
| `crates/nuclear-diag/src/lib.rs` | `Diagnostic`, `SourceFile`, codes N00xx (syntax), N01xx (resolver), N02xx (dialect), N03xx (project); text and JSON |
| `crates/nuclear-syntax/src/lexer.rs`, `token.rs` | tokens + INDENT/DEDENT (none inside `()[]{}`) → [syntax.md](syntax.md) |
| `crates/nuclear-syntax/src/fmt.rs` | `nuclear fmt`: rewrites only the gaps between tokens (indentation, spaces, blank lines, comments), copies every token as written and refuses a result that does not lex to the same tokens and comments; `tests/fmt.rs` keeps every `.nuc` of the repository formatted |
| `crates/nuclear-syntax/src/parser.rs`, `ast.rs` | `parse(src) -> Parsed`. "Header + block": words before the name are kinds or modifiers (`Decl.words`), words after it are clauses |
| `crates/nuclear-syntax/src/doc.rs` | `DocComment::parse`: doc comments with `@param/@return/@throws/@example/@see/@deprecated/@since`; `markdown()`, `param(name)` |
| `crates/nuclear-dialect/src/template.rs` | Mustache subset: `{{x}}`, `{{#x}}`, `{{^x}}`, `{{.}}`, `@first/@last/@index`, filters `pascal camel snake kebab lower upper join:", "` |
| `crates/nuclear-dialect/src/contract.rs` | SYNTAX/TYPES/OPERATORS templates, variables of each kind, OPTIONS; `markdown()` = `nuclear spec` |
| `crates/nuclear-dialect/src/model.rs` | `Loader`, `Dialect`, `Report`: `extends`/`replaces`, several .toml merged, `{ file = "templates/x.tpl" }`, validations N0204/N0205/N0210/N0212; `Loader::load_defs`, `add_defs` → [dialect-format.md](dialect-format.md) |
| `crates/nuclear-dialect/src/defs.rs` | type definitions: `Package`, `Repository`, `TypeDefs`, `render*` → [type-definitions.md](type-definitions.md) |
| `crates/nuclear-transpile/src/lib.rs` | `transpile()` and `check()` (API also for the LSP) |
| `crates/nuclear-transpile/src/analysis.rs` | reassigned names per block (const/let), awaits → async, `yields_block`, `pattern_names` |
| `crates/nuclear-transpile/src/alias.rs` | `f() = Module.method` rewritten before emitting → [result.md](result.md) |
| `crates/nuclear-transpile/src/emit.rs` | the template-driven emitter (largest file) → [emitter.md](emitter.md); the standard library rewrite: `static_type` (the receiver's type), `library_access`, `library_render`, N0114/N0115 |
| `crates/nuclear-dialect/src/model.rs` (kind packages) | ADR-0095: `Loader.packages` (folders with `kinds/<name>/kind.toml`), `kind_packages`, `kind_package_at`, `apply_packages` (a package of the dialect's platform becomes a plugin; its `<dialect>/` folder is a layer named after the package), `Dialect.kind_sources`, `package_gaps` (→ N0306 in `project::check_config`), `extra_docs` |
| `crates/nuclear-dialect/src/library.rs` + `dialects/<lang>/library.toml` | Nuclear's standard library (ADR-0094, `spec/library.md`): the vocabulary in Rust (bases, members, result types, wrong target names), one template per member in each language dialect (`spec/dialects.md` §5.8; `helper` written once per file) |
| `crates/nuclear-transpile/src/project.rs` | multi-file projects → [projects.md](projects.md) |
| `crates/nuclear-transpile/src/contract.rs` | the contract of a project (`.nuclear/contract.json`: models, routes or calls, in Nuclear notation), its OpenAPI document, and the Nuclear a client generates from the contracts its manifest declares (`contracts/<name>/`; `sync` runs at the start of `project::build`, `write` with the card); `nuclear contracts list|update` in the CLI (ADR-0098, `spec/projects.md` §1.2) |
| `crates/nuclear-cli/src/main.rs` | the `nuclear` binary; `TOPICS` embeds the docs (`include_str!`; a new `.md` in `spec/` or `docs/language/` must be a topic, a test checks it) |
| `crates/nuclear-cli/src/agent.rs` | `nuclear agent init` (AGENTS.md section, CLAUDE.md, Cursor rule, Claude Code hook) and `nuclear agent hook` (checks the edited `.nuc`, one line per error, exit 2) (ADR-0087) |
| `agents/` | the `nuclear` skill (`skills/nuclear/SKILL.md`, installed by `install.sh` in `~/.claude/skills`, `~/.agents/skills`, `~/.codex/skills`) and the project templates of `agent init` |
| `spec/brief/` | the short sheets for agents: `nuclear spec --brief --dialect <d>` (base + chain, ≤ 4k tokens with flutter); `BRIEFS` in main.rs embeds them |
| `examples/stdlib/` | one program per area (math, physics, bits, strings, regex, collections, types, bytes, dates, async) per target, with the shared expected output `<area>.out` (ADR-0086) |
| `tools/stdlib-check.sh` | runs `examples/stdlib/` with `tsc --strict` + node and `dart run`; both must print `<area>.out` |
| `tools/agent-eval/` | agent evaluation: `run.py --mode baseline\|agent` runs `tasks.toml` with `claude -p` (the user's account) and stores `results/<mode>.json` |
| `languages/nuclear.toml`, `languages/docs.toml` | base reserved words; base editor docs (`nuclear_dialect::base_doc`) |
| `dialects/{typescript,dart,python,nodejs}/` | language dialects; every Dragonfly dialect extends one (Node/Express postponed by the user: do not touch until asked). Each translates the standard library (`spec/library.md`, ADR-0094) |
| `defs/dart/<package>/<version>/` | GENERATED type definitions (do not edit) |
| `tools/defs/` | defs generator (Dart, `package:analyzer`); `flutter_project/` is also the host of `analyze-flutter.sh` |
| `tools/spec-check/` | checks that spec equivalences are the real output |
| `tools/context/` | context hooks (`STATUS.md` freshness) |
| `crates/nuclear-lsp/` | the LSP server (moves into this repository, ADR-0083) → [../crates/nuclear-lsp.md](../crates/nuclear-lsp.md) |

## Dialect chain

```
typescript ─► nodejs ─► nestjs   (backend platform, see dragonfly-backend/)
python ─────► fastapi             (backend platform, see dragonfly-backend/)
dart ───────► flutter            (dart: kinds extension, mixin, data · flutter: mobile platform, see dragonfly-mobile/)
```

Each dialect is `dialects/<name>/`: `dialect.toml`, optional `keywords.toml` (target keywords only),
`docs.toml` (editor docs) and `templates/*.tpl`. `dart` and `flutter` have no auto-import lists: they come
from type definitions (`[dialect] defs`).

## Rules of this area

- The parser does not change per dialect; framework words, shortcuts and imports live in `dialects/`.
- Rust decides *what* is emitted; the dialect decides *how* it is written.
- No inferred parameters, no hoisting, no `type` (removed by the user).
- If `ast.rs` gains or changes variants, `cd editors && cargo build` breaks: tell the editors owner which ones.

## Specs (user-facing rules)

`spec/nuclear-base.md` (base, Nuclear only), `spec/library.md` (the standard library, Nuclear only), `spec/{typescript,nodejs,dart,python}.md`, `spec/grammar.ebnf`,
`spec/dialects.md` (dialect format, agents), `spec/definitions.md`, `spec/projects.md`,
`spec/diagnostics.md`, `spec/reserved-words.md`, `docs/language/` (guide, `referencia.md` = reference).

## L3 files

| File | read-when |
|---|---|
| [syntax.md](syntax.md) | lexer or parser work: indentation, comments, blocks inside parentheses |
| [emitter.md](emitter.md) | any change to generated code |
| [result.md](result.md) | `Result`, propagation, `.ok/.error`, aliases |
| [dialect-format.md](dialect-format.md) | kinds, options, templates, dialect validation |
| [type-definitions.md](type-definitions.md) | auto-imports, overrides, `defs/`, the generator |
| [projects.md](projects.md) | multi-file builds, `[project]`, `[architecture]`, project templates |
| [testing.md](testing.md) | which test to add or where a golden file lives |
| [limits.md](limits.md) | before promising a feature: what Nuclear cannot express yet, what is planned |
