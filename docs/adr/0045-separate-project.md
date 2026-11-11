---
adr: 0045
title: "Separate project"
status: "Accepted"
scope: dragonfly
---

# ADR-0045: Separate project

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The `.nuc` files form a project separate from the Flutter app; the app is generated complete in another folder (`build/app`).
