---
adr: 0035
title: "External classes"
status: "Accepted"
scope: nuclear
---

# ADR-0035: External classes

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Versioned type definitions (like `.d.ts`), generated from the SDK (format and repository: ADR-0040). They mark overrides and give the inherited members of classes that are not in the file.
