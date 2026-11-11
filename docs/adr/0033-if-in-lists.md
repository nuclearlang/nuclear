---
adr: 0033
title: "`if` inside lists"
status: "Accepted"
scope: nuclear
---

# ADR-0033: `if` inside lists

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`[a, if cond b]`, without `else`; there is no `for` inside lists (`...xs.map(...)` is used).
