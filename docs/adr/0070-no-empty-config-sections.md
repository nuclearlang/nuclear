---
adr: 0070
title: "No empty sections in `config.nuc`"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0070: No empty sections in `config.nuc`

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

What is not used is not written: a new project has only `app(...)`. `dragonfly flutter add` creates `dependencies`/`devDependencies` with its first package and `remove` deletes the section with its last one.
