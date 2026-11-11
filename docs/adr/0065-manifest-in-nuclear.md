---
adr: 0065
title: "Manifest in Nuclear (`config.nuc`)"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0065: Manifest in Nuclear (`config.nuc`)

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The pubspec of a Dragonfly project is written in Nuclear, at the root: `app(name, description, version, usesMaterialDesign)`, `environment(sdk, flutter)`, `dependencies(package = "version")`, `devDependencies(...)`, with the language's call syntax (no new grammar). It only lists the project's own packages; Dragonfly's are added automatically. It is not transpiled; it is checked with the project (N0303). Generic dialect mechanism: `[project.manifest] file, sections`.
