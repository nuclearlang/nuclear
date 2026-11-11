---
adr: 0005
title: "Implicit return"
status: "Accepted"
scope: nuclear
---

# ADR-0005: Implicit return

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The last line is returned only when the function declares a return type. It avoids returning something by accident from `void` functions.

## Alternatives rejected

Always / never.
