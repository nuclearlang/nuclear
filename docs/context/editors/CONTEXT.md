---
layer: L2
component: editors
read-when: you touch the editor tooling (crates/nuclear-lsp, nuclear-vscode, nuclear-jetbrains, nuclear-zed: LSP server, VS Code, JetBrains, Zed, theme) or editor behaviour of Dragonfly projects
---

# Editors — LSP server and extensions

The editor tooling, split by repository (plan 006): the server is `crates/nuclear-lsp` of the `nuclear` repository
(its workspace, its tests); the extensions are `nuclear-vscode/` (with the shared `theme/`, the grammar and the
icons the other two read), `nuclear-jetbrains/` and `nuclear-zed/`, each its own git inside the working tree; the
Dragonfly parts move later to `dragonfly-flutter-vscode` / `dragonfly-flutter-jetbrains` (ADR-0083). Extensions run
`nuclear-lsp` as an external binary and talk to it only through LSP (+ `nuclear/*` requests). Installed version:
**0.37.2**. No commits unless the user asks.

## Rules

1. The server uses the other crates of the workspace; a missing parser/transpiler feature is built there with the
   rest of the `nuclear` repository's rules.
2. APIs used: `nuclear_syntax::{parse, lex, ast, token, DocComment}`, `nuclear_transpile::{check, transpile,
   project}`, `nuclear_dialect::{Loader, Dialect, defs (TypeDefs, Decl, Member, Constructor, Param, render*),
   base_doc}`, `nuclear_diag`. If they change, `cargo build -p nuclear-lsp` catches it.
3. Done = `cargo test -p nuclear-lsp` (14 unit + 35 server) + `cargo clippy --all-targets`; `npm run theme` in
   `nuclear-vscode/` if the palette changed; `npm run compile` there if the client changed; `nuclear-jetbrains/build.sh`
   if the plugin changed.
4. **Always release after a change** of server, client, themes or bundled dialects/defs, without asking: bump
   the version (minor for features, patch for fixes) in `nuclear-vscode/package.json`,
   `nuclear-jetbrains/resources/META-INF/plugin.xml`, `nuclear-zed/extension.toml` and the `README.md` examples; then
   `cd nuclear-vscode && npm run package`, `nuclear-jetbrains/build.sh`, `nuclear-zed/build.sh` (Zed's version
   follows VS Code's), `./install.sh`. Tell the user to quit VS Code completely (*File → Exit*) and install the zip in Android
   Studio. `../install.sh` installs the newest `.vsix` (`sort -V`).
5. Themes and icons are generated for the three extensions by `nuclear-vscode/theme/` (a sibling not checked out is
   skipped): edit `palette.json`, `build.mjs` or `icons.mjs`, never the output JSON/XML/SVG.
6. Framework extensions must not collide: only the base registers the language, grammar and LSP; framework
   ones activate on their dialect's `[project] marker` and prefix everything with their dialect (ADR-0083).

## Notes from the platform session (2026-11-04, ADR-0093)

- `nuclear_transpile::project::Manifest` has `platform: Option<String>`; `Manifest::read(root)` now also reads
  `config.nuc` (`dragonfly(platform = ..., dialect = ...)`, `app(name = ...)`) and `Manifest::find` tries it before
  the `[project] marker` fallback. `lsp/src/project.rs` replicates the old `find`: add the `config.nuc` branch (or
  call `Manifest::find`) so backend projects (`fastapi`, `nestjs`) are found; flutter apps keep working through the
  marker. `project::output_dir(&manifest, &dialect)` gives the output folder (`Manifest.out` is empty for
  `config.nuc` manifests).
- Dialects moved to `dialects/mobile/flutter/`, `dialects/backend/{fastapi,nestjs}/` (`langgraph` renamed `fastapi`);
  `Loader::load("flutter")` finds them unchanged, `Loader::available()` lists them; `Dialect.platform` says the
  platform. `platform.toml` in a platform folder is the contract (N0305). Mention `fastapi` wherever the extensions
  listed `langgraph`.
- **Kinds have a source (ADR-0095 §7, asked by the user 2026-11-05).** Kinds are Nuclear's libraries: native kinds
  (the platform contract) and extension kinds in packages, one platform per package, enabled by `plugins(...)` in
  `config.nuc`. The LSP must tell them apart: list for a project only the kinds of its platform (native + enabled
  packages), show in completion and hover where a kind comes from (native of `backend`, or package `jobs 1.2.0`
  for `fastapi`) with the package's `docs.toml`, and report a kind of a package made for another platform or not
  implemented for the project's dialect, with the fix. Today `d.kinds` is flat and `Entry.origin.dialect` only names
  the dialect file that defined a kind (a project dialect's kinds look native); the loader will expose each kind's
  source in phase E of the platforms refactor (`STATUS.md`): until then nothing to do in the editors beyond the
  `config.nuc` branch above.
- 2026-11-05 (phase E, first part): the loader now gives `Dialect.kind_sources` (`KindSource::{Language,
  Platform(name), Package { name, version, dialects }}`), `Dialect.packages`, `package_gaps` and reads a package's
  `docs.toml` first (`extra_docs`); `Loader.packages` must be set to `<project>/kinds` and `<install>/kinds` (as
  `crates/nuclear-cli/src/main.rs` `loader()` does) for the LSP to see them. DONE 2026-11-05 (platform session,
  asked by the user): `project::find` reads `config.nuc`, `dialects::load` takes the project root (kind packages),
  hover/completion show the kind's source, every dialect is bundled, the project card `.nuclear/project.json`
  (`nuclear_transpile::project::{METADATA_FILE, metadata, write_metadata}`) is written by `Project::build` and read
  by the VS Code status bar. The JetBrains and Zed clients do not read the card yet (same server, so the
  diagnostics are right).
- 2026-11-05 (phase B2): `Manifest` gained `version` and `description` (from `app(...)`); `editors` builds unchanged.
- 2026-11-05 (phase C, ADR-0094): `Dialect` gained `library` (`[library.<base>]`), `function_helpers` and
  `syntax_helpers`; `nuclear_dialect::library` holds the vocabulary (`MEMBERS`, `BASES`, `OBJECTS`, `WRONG`): completion
  on a value of a base type can list its members from there (the emitter's receiver typing, `static_type`, is private
  to `nuclear-transpile`; an open request if the LSP wants it). New codes N0114, N0115. `editors` builds unchanged.

## L3 files

| File | read-when |
|---|---|
| [lsp.md](lsp.md) | server modules, completion/hover/type inference, projects in the LSP, docs, packaging |
| [vscode.md](https://github.com/nuclearlang/nuclear-vscode/blob/main/docs/context/nuclear-vscode.md) | VS Code client: Dragonfly run/debug/save, `.nuc` debugging, the Dragonfly device, processes |
| [jetbrains.md](https://github.com/nuclearlang/nuclear-jetbrains/blob/main/docs/context/nuclear-jetbrains.md) | Android Studio / IntelliJ plugin |
| [zed.md](https://github.com/nuclearlang/nuclear-zed/blob/main/docs/context/nuclear-zed.md) | Zed extension and what Zed cannot do |
| [theme.md](https://github.com/nuclearlang/nuclear-vscode/blob/main/docs/context/nuclear-vscode/theme.md) | Chernobyl theme, icons, code metrics, testing a theme |

User docs: `spec/flutter.md` §11.1 (VS Code), §11.2 (Android Studio), §11.3 (Zed), `nuclear-vscode/README.md`.
