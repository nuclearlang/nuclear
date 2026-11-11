---
adr: 0034
title: "Constructor of an inheriting module"
status: "Accepted"
scope: nuclear
---

# ADR-0034: Constructor of an inheriting module

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`super(...)` as the first line of the body (its arguments are the received parameters); without it the base is called without arguments. A parameter named like a member of the base is that member, not a new field.
