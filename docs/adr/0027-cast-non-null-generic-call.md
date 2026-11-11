---
adr: 0027
title: "`x as T`, `x!`, `X<T>(...)`"
status: "Accepted"
scope: nuclear
---

# ADR-0027: `x as T`, `x!`, `X<T>(...)`

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Kept as in TypeScript and Dart. The `<` of a generic call goes right after the name; with a space it is a comparison.
