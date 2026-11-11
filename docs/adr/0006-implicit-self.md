---
adr: 0006
title: "Implicit `this`"
status: "Accepted"
scope: nuclear
---

# ADR-0006: Implicit `this`

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Inside a module, members are used by name; the resolver knows with certainty when a name is a member and the dialect adds `this`/`self` when the target needs it. Less noise.

## Alternatives rejected

An explicit `$` prefix.
