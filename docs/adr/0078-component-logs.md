---
adr: 0078
title: "Readable logs by component"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0078: Readable logs by component

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`Log.d/i/s/w/e` write a block with a breadcrumb (component › kind › Class.method), time, message, data as JSON and, at the foot, the `.nuc` file and line. The transpiler writes the breadcrumb in each call as a constant (`[call_site]` of the dialect): no runtime cost nor stack reading. Each component has a stable pastel color (xterm-256); its `config` changes it with `logColor`. A thick frame and a level badge use the colors of Bootstrap's dark-mode alerts; breadcrumb and data use code colors; Dragonfly's own logs are purple. Only the exact place of an error blinks. Flutter exceptions use their own format (double red frame, `✖ EXCEPTION` bar, stack in columns with the app highlighted). Readable without colors (`Log.colors = false`), for people and agents.

## Alternatives rejected

Reading the stack at runtime to find the caller.
