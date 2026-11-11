---
adr: 0007
title: "No visibility modifiers"
status: "Accepted"
scope: nuclear
---

# ADR-0007: No visibility modifiers

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

No `private`, `public` or `protected`. Visibility between components is a project rule (see ADR-0046 and `[architecture]`).
