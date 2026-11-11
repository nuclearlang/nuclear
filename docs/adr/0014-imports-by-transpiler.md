---
adr: 0014
title: "Imports written by the transpiler"
status: "Accepted"
scope: nuclear
---

# ADR-0014: Imports written by the transpiler

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The transpiler writes the imports: it knows the libraries of the dialect (type definitions, ADR-0040) and organizes them per component, module or function. `use` exists for shortcuts (`use flutter.ui`).
