---
adr: 0032
title: "Destructuring"
status: "Accepted"
scope: nuclear
---

# ADR-0032: Destructuring

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Lists (`[a, ...rest] = xs`, `_` ignores) and maps (`{ a, "k": b } = m`), Elixir style with TypeScript syntax: `...` instead of `|` (already union types and `match` alternatives). Names are all new or all existing (N0107).
