---
adr: 0076
title: "Mixed debugging"
status: "Accepted"
scope: editors
---

# ADR-0076: Mixed debugging

- Status: Accepted
- Scope: `editors` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

On top of the Dart extension's session (not an own debugger translating messages, which would lose devices, DevTools and the Inspector): steps by Nuclear line (repeat the Dart step until another `.nuc` line), active when the pause came from a `.nuc` breakpoint or the focus is on a `.nuc`; a `.dart` breakpoint gives Dart steps. Variable values in the `.nuc`, a Nuclear call stack and a command to go to the same line in Nuclear/Dart. VS Code's native stack still shows Dart.

## Alternatives rejected

An own debug adapter.
