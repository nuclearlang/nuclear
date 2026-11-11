---
adr: 0016
title: "Not object oriented"
status: "Accepted"
scope: nuclear
---

# ADR-0016: Not object oriented

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Nuclear is like Elixir: components, modules and functions. It still has modules with constructors and inheritance so that it transpiles to object-oriented languages.
