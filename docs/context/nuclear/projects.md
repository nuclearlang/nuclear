---
layer: L3
component: nuclear
read-when: you change project builds, [project]/[architecture] of a dialect, project files templates or config.nuc checks
---

# Projects (`spec/projects.md`)

`crates/nuclear-transpile/src/project.rs`:

- `Manifest` (`nuclear.toml`, or inferred: `Manifest::infer(root, dialect, out)`, `Manifest::find(path,
  &dialects)` by the dialect's `[project] marker`).
- `build()`: pass 1 = declarations of every file (`Emitter::declarations`, `ProjectCtx`); pass 2 = emit each
  file with that table. A name of another file is used without `use`: the import is written.
- `[architecture]` rules (the linter, ADR-0046): N0301 (callers, names), N0302 (component privacy), checked by
  tokens inside each declaration. `scope`, `public`, `exempt`, `callers`, `names`. All are errors today.
- Result runtime in `nuclear/result`; `[project.files]` templates (`project_context`: `decls`, `kinds.<k>`,
  `decorated.<d>`, `files`, `has.<kind>`, `has.@<decorator>`); a project file whose template renders empty is not
  written; it imports the files whose names its code writes.
- `[project] sources/kinds/files/outputs/assets/marker/output`. Kind by folder (`[project.kinds]`). Assets: text
  files go to `generated`, others to `ProjectOutput.copies` (path → source).
- `config.nuc` (ADR-0065): `[project.manifest] file, sections, packages, reserved`; `project::read_config`
  (section → arguments as text, N0303), `check_config` (reserved/dialect packages → N0303),
  `reserved_package`. `build` returns it in `out.files` with an empty `out_path` (writers skip empty paths).
- Output: `Output::lines` / `BuiltFile::lines` (line map for debuggers).

Open requests from the LSP (optional, not done): `project::build` with the text of open files
(`overrides: &HashMap<PathBuf, String>`); public `default_kind(dialect, path)` and `ProjectCtx::visible` (the
LSP copied them); the inferred type of an expression; `render_member` substituting generics.
