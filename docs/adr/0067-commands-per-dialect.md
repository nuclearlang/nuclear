---
adr: 0067
title: "Commands per dialect"
status: "Accepted"
scope: dragonfly
---

# ADR-0067: Commands per dialect

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A dialect's own commands go under its name: `dragonfly <dialect> <command>` (`dragonfly flutter add`). Common commands (`build`, `check`, `make`...) stay at the first level because the project already says its dialect. Subcommands the dialect does not define go to its CLI (`dragonfly flutter doctor` → `flutter doctor`).
