---
adr: 0025
title: "Dialect specs"
status: "Accepted"
scope: nuclear
---

# ADR-0025: Dialect specs

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`spec/nuclear-base.md` describes no dialect; every dialect has `spec/<dialect>.md`, which inherits from the base.
