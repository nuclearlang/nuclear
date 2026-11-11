---
adr: 0012
title: "Automatic async"
status: "Accepted"
scope: nuclear
---

# ADR-0012: Automatic async

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`async` is never written. A function that awaits another (`user = ~callUser()`) becomes async.
