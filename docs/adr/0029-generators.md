---
adr: 0029
title: "Generators"
status: "Accepted"
scope: nuclear
---

# ADR-0029: Generators

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`yield value` and `yield ...other()`; a function with `yield` is a generator (async when it uses `~`); the return type is the type of each value.
