---
adr: 0071
title: "Assets by convention"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0071: Assets by convention

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`config.nuc` declares no assets: `assets/fonts/<Family>/` are fonts (weight and style from the end of the name: `Roboto-BoldItalic.ttf`), `assets/shaders/*.frag` are shaders and any other folder of `assets/` is bundled. `dragonfly build` rewrites `assets:`/`fonts:`/`shaders:` of the pubspec on every build. Two fonts with the same weight and style are an error. The project only adds: no `dependency_overrides`.
