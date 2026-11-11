---
adr: 0015
title: "Versioned technologies and optional agents"
status: "Accepted"
scope: nuclear
---

# ADR-0015: Versioned technologies and optional agents

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The transpiler keeps versions of each technology with their dictionaries (type definitions). Optionally it can connect to a coding agent (Claude, Codex, OpenCode) to optimize and personalize the generated code.
