---
adr: 0048
title: "`.ok()` / `.error()` by what the callback returns"
status: "Accepted"
scope: nuclear
---

# ADR-0048: `.ok()` / `.error()` by what the callback returns

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`Ok`/`Err` transforms or recovers; a plain value transforms (`.ok`) or recovers (`.error`: the expression is that value); nothing (`void`, an external function) only observes and the error keeps propagating.
