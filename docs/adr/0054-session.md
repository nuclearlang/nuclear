---
adr: 0054
title: "Session"
status: "Accepted"
scope: dragonfly
---

# ADR-0054: Session

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`session.start(data, ...)` and `session.destroy()` (which publishes the end to the global events), kept in secure storage. It combines with the ACL when the developer wants.
