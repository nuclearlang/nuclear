---
adr: 0013
title: "Await with `~`"
status: "Accepted"
scope: nuclear
---

# ADR-0013: Await with `~`

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`~` before a call awaits it: `users = ~getUsers.get()`. `@` is only for decorators.

## Alternatives rejected

`await`, `@`.
