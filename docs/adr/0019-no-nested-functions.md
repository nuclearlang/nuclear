---
adr: 0019
title: "No nested functions"
status: "Accepted"
scope: nuclear
---

# ADR-0019: No nested functions

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Functions are not declared inside a body; an anonymous function assigned to a name is used instead: `log = (message: string)` plus a block.
