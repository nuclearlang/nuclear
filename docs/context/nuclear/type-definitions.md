---
layer: L3
component: nuclear
read-when: an import is missing or wrong, an override is not detected, or you regenerate defs/
---

# Type definitions (ADR-0035, 0040; `spec/definitions.md`)

Like `.d.ts` / `@types`: packages `name@version` in `defs/<language>/<package>/<version>/`, each with
`package.toml`, `index.json` (transpiler: classes, auto-imports, re-exports) and `api.json` (full API in
Nuclear notation, read lazily: LSP, `nuclear defs`, agents). GENERATED: never edit.

Packages today: `dart@3.13.5`, `flutter@3.47.7`, `dragonfly@0.1.0`, `flutter_bloc@9.1.1`, `bloc@9.2.1`,
`dio@5.11.1`, `get_it@9.3.0`, `go_router@18.1.1`, `connectivity_plus@7.3.2`, `flutter_secure_storage@11.2.0`,
`json_annotation@4.12.0`.

## Loading and priority

`[dialect] defs = ["flutter@3.47.7", ...]` (added to the parent's). Priority: parent first, order of `defs`,
each package before its dependencies, inside a package the most basic library (widgets before material).
`implicit` libraries (dart:core) are never imported. Hand-written `[auto_imports]`/`[functions]` win. A name
two packages export with different declarations goes to the first: `Flow` → `dart:developer`, not the widget.
Validations N0213/N0214. When migrating (2026-10-25) dart and flutter gave exactly the same auto-imports,
re-exports and classes as the old lists (3131 names, 2509 classes).

## Project packages (ADR-0066, 0069)

Third-party packages of a Dragonfly project get definitions generated once into
`~/.cache/nuclearlang/defs` and listed in `build/defs.lock` (`project::defs_lock`, `Loader::add_defs`). `nuclear
defs show` does not see them (they belong to the project). `Package::from_project` is false for the dialect's.

## Generator

`tools/defs/bin/defs.dart` (one package) and `generate.sh` (all, ~45 s); `flutter_project/` resolves Flutter
plus pub packages. Installed in `<install>/tools/defs` for the CLI. TypeScript has no generator yet (from
`.d.ts`; Node is postponed). Packages outside `defs/dart/` (provider, http) have no auto-imports until generated.
