---
adr: 0018
title: "Map type"
status: "Accepted"
scope: nuclear
---

# ADR-0018: Map type

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`<Key, Value>` anywhere a type goes: `get(name: string) <string, string>`. A map value is always written between `{ }`.
