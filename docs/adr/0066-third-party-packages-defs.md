---
adr: 0066
title: "Third-party packages with definitions"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0066: Third-party packages with definitions

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`dragonfly flutter add <package>` adds it to `config.nuc` and the app and generates its type definitions (Dart analyzer, `tools/defs`) in a global cache (`~/.cache/nuclearlang/defs`) shared by projects. `dragonfly flutter remove` removes it from the project and keeps the cached definitions. The project lists its own in `build/defs.lock`.
