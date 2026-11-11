---
adr: 0037
title: "Function aliases"
status: "Accepted"
scope: nuclear
---

# ADR-0037: Function aliases

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`name() = Module.method`: `name` is the same function as `method` (same parameters, return, async). A non-`static` method is called on a new `Module()`, whose constructor cannot have required parameters (N0112). Only when the body is `Module.method` of a module of the file; otherwise the one-line form keeps its meaning.
