---
adr: 0043
title: "Dependency injection"
status: "Accepted"
scope: dragonfly
---

# ADR-0043: Dependency injection

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A typed parameter in a kind's header (`(repository: CharacterRepository)`) is injected automatically.
