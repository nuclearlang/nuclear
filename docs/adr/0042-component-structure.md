---
adr: 0042
title: "Component structure"
status: "Accepted"
scope: dragonfly
---

# ADR-0042: Component structure

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`components/<c>/{config, data, domain/{models, usecases, aggregates, services}, presentation/<screen>/{managers, state_models}, presentation/widgets}` plus the facade `<c>_component.nuc` (kind `appcomponent`); `shared/`, `config/` (kind `appConfig`) and `main.nuc` at the root.
