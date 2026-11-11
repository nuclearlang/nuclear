---
adr: 0022
title: "Block scope and values"
status: "Accepted"
scope: nuclear
---

# ADR-0022: Block scope and values

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A variable created inside `if`, `for`, `while`, `match` or `try` does not exist outside (N0105). A name nothing defines is an error (N0106). `if` and `match` are values, as in Elixir: `x = if ...`.
