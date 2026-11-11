---
adr: 0046
title: "Architecture rules (linter)"
status: "Accepted"
scope: dragonfly
---

# ADR-0046: Architecture rules (linter)

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A use case is only called from an appcomponent; a service only from a use case or an aggregate. The transpiler checks them (nuclear-flutter-lints): `[architecture]` of the dialect, N0301/N0302.
