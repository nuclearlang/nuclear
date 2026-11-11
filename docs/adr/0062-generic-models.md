---
adr: 0062
title: "Generic models"
status: "Accepted"
scope: dragonfly
---

# ADR-0062: Generic models

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A model can have type parameters (`Page<T>` with `results: T[]`) and interfaces (`ApiFailure > Exception`); the dialect writes what is needed (Dart: `genericArgumentFactories` of json_serializable, the `fromJson` of each argument in repositories). A kind that would not write generics or interfaces is an error (N0218) instead of losing them.
