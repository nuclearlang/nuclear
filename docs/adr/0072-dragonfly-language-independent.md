---
adr: 0072
title: "Dragonfly is language independent"
status: "Accepted"
scope: dragonfly
---

# ADR-0072: Dragonfly is language independent

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Dragonfly is a framework concept (components, layers, architecture kinds); each dialect applies it to its technology: `dragonfly flutter`, `dragonfly typescript`, `dragonfly crystal/amber`. The CLI knows no language: what it needs from each one is declared by the dialect.
