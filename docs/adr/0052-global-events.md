---
adr: 0052
title: "Global events"
status: "Accepted"
scope: dragonfly
---

# ADR-0052: Global events

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A global manager (`Events`) with two operations, `publish` and `subscribe`, like Guava's EventBus or Node's EventEmitter. Delivery is on demand: an event reaches only the visible screen and the global components at once; a hidden screen gets the last event of each type when it is shown again. Background isolates of the app also publish.
