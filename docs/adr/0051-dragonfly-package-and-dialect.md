---
adr: 0051
title: "Dragonfly package and `flutter` dialect"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0051: Dragonfly package and `flutter` dialect

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`dragonfly` is an independent Dart package (publishable on pub.dev, also usable by hand; `spec/dragonfly.md`). The `flutter` dialect (kinds, project structure, architecture, tests and the `dragonfly` CLI; `spec/flutter.md`) generates code that uses it.
