---
layer: L3
component: nuclear
read-when: you add a test, a golden file changed, or you must validate generated code against the real target
---

# Testing

| Test | What it covers |
|---|---|
| `crates/nuclear-transpile/tests/golden.rs` | each `examples/**/x.nuc` against `x.ts` / `x.dart`: `examples-nodejs/chat`, `examples-nestjs/users`, `examples-flutter-dialect/{flutter_app,dart_extension,widgets_tour,framework_libs}`, `spec/base` (to ts and dart). `UPDATE_GOLDEN=1` regenerates |
| `crates/nuclear-transpile/tests/rules.rs` | one test per language rule |
| `crates/nuclear-transpile/tests/projects.rs` | each `examples/projects/<p>/` and the Dragonfly example against `examples/expected/<p>/`; architecture rules, kind by folder, `[project.files]`, duplicate names; `dragonfly_spec_shows_the_real_output` (blocks captioned `Output (`/`Generated (` in `spec/flutter.md`) |
| `crates/nuclear-dialect/tests/loader.rs` | dialect validation and type definitions (package order, import priority, full API, errors, project packages) |
| `every_code_is_documented`, `every_dialect_word_has_editor_docs` | docs of diagnostics and dialect words |

- `widgets_tour.nuc` (flutter): tour of Flutter patterns (routes, async, ListView.builder, form, GlobalKey,
  getter, mixin, JSON with `as`, `!`); its output passes `flutter analyze` without warnings.
- `framework_libs.nuc`: Cubit/Dio/GetIt/BlocBuilder imported only from the defs.

## Validating against the real target

- Dart: `dragonfly-mobile/tools/analyze-flutter.sh [file.nuc...]` (host `tools/defs/flutter_project`, `flutter_lints`, 0 warnings).
- TypeScript: `nuclear-vscode/node_modules/.bin/tsc --strict --noEmit --target es2022 --lib es2022,dom x.ts`.
- Base library in both targets: `TSC=<tsc> tools/stdlib-check.sh` (needs node, dart, TypeScript); the cargo test
  `stdlib_programs_transpile_in_both_targets` only checks they transpile. Agent sheets: `check.py spec/brief/<d>.md`.
- Agents: `tools/agent-eval/run.py --mode agent` (spends the user's Claude Code quota: ask first).
- Spec equivalences: `tools/spec-check/check.py spec/<d>.md <dialect> <language>` (0 problems), also on
  `docs/language/<x>.md typescript typescript --fix` when output changes.
