---
adr: 0053
title: "Global components"
status: "Accepted"
scope: dragonfly
---

# ADR-0053: Global components

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Kind `component` in `shared/components/`: cross-cutting use cases any declaration can use; their `@on(Event)` methods run on every event, wherever the user is.
