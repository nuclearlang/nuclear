---
adr: 0050
title: "A Dragonfly project is only `.nuc`"
status: "Accepted"
scope: dragonfly
---

# ADR-0050: A Dragonfly project is only `.nuc`

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

No `nuclear.toml` nor files of other types (except the dictionaries in `assets/`, ADR-0060): everything generated goes to `build/`.
