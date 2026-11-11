---
adr: 0003
title: "Variables"
status: "Accepted"
scope: nuclear
---

# ADR-0003: Variables

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`x = 1` declares a variable. The transpiler decides whether it is a constant or mutable (it is mutable when it is reassigned in its block); there is no `const`/`let`/`var` in Nuclear.
