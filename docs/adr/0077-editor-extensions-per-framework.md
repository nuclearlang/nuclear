---
adr: 0077
title: "Editor extensions per framework"
status: "Accepted"
scope: editors
---

# ADR-0077: Editor extensions per framework

- Status: Accepted
- Scope: `editors` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

One Language Server and one base "NuclearLang" extension per editor (VS Code, JetBrains): syntax, highlighting, theme, metrics, diagnostics, completion and the `.nuc` ↔ target line map, for any dialect. Each file's dialect comes from its project (`nuclear.toml` or the dialect's `[project] marker`). What depends on a framework's tools (run, debug, devices, hot reload) goes in one extension per framework that depends on the base. What the editor needs from a language is declared by the dialect: `[docs]` and `[project] marker`/`output`.
