---
adr: 0026
title: "Enum item without a value"
status: "Accepted"
scope: nuclear
---

# ADR-0026: Enum item without a value

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

In a key-value enum, an item without a value takes its own name: `medium` → `"medium"`.
