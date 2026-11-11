---
adr: 0068
title: "Debugging `.nuc`"
status: "Accepted"
scope: editors
---

# ADR-0068: Debugging `.nuc`

- Status: Accepted
- Scope: `editors` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A line map (Dart line → Nuclear line of each statement, member, declaration and argument on its own line), not Dart aligned line by line with the `.nuc`: the generated code stays readable. `dragonfly build` writes it to `build/app/.dart_tool/nuclear_lines.json`; editors set a mirror breakpoint in the `.dart`, show the `.nuc` line of each pause and can mark a Dart error on its `.nuc` line with a side window that explains it (`dragonfly explain`).

## Alternatives rejected

Dart aligned line by line with the source.
