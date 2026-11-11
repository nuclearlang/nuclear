---
adr: 0041
title: "Dragonfly"
status: "Accepted; generalized by ADR-0072"
scope: dragonfly
---

# ADR-0041: Dragonfly

- Status: Accepted; generalized by ADR-0072
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

NuclearLang's framework for Flutter is called Dragonfly. It works with the `flutter` dialect (kinds and transpilation) and adds the classes and features Flutter lacks in its own Dart package (`packages/dragonfly`).
