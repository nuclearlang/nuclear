---
adr: 0017
title: "Generated context maps of the code"
status: "Accepted, not implemented"
scope: nuclear
---

# ADR-0017: Generated context maps of the code

- Status: Accepted, not implemented
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The transpiler will generate context documentation for each class and function, organized in maps and layers, so an agent goes to the concrete code instead of reading all of it.
