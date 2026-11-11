---
adr: 0084
title: "Layered AI context"
status: "Accepted"
scope: nuclear
---

# ADR-0084: Layered AI context

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: 2026-11-03

## Decision

Context documentation is a tree read top-down (progressive disclosure): L0 `AGENTS.md` (router and hard rules, always loaded; `CLAUDE.md` only imports it) → L1 system (PRD, architecture, technology) → L2 one `CONTEXT.md` per repository/component → L3 topic deep dives and specs → L4 feature specs (spec → plan → tasks). Every L1–L3 file starts with `layer`, `component` and `read-when`. ADRs record why and are not on the default reading path. `STATUS.md` is the only list of state and pending work, kept current by hooks. Model: the user's `store-checkout` project (its ADR-0010).

## Alternatives rejected

One large `CLAUDE.md` (stale, token heavy).
