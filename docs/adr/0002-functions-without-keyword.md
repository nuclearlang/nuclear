---
adr: 0002
title: "Functions without a keyword"
status: "Accepted"
scope: nuclear
---

# ADR-0002: Functions without a keyword

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A function is a header plus a block: `greet(x: string) string` followed by an indented body. There is no `fn`/`def` keyword (the "header + block" rule).

## Alternatives rejected

`greet(...)` with a keyword.
