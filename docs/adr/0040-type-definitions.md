---
adr: 0040
title: "Cross-dialect type definitions"
status: "Accepted"
scope: nuclear
---

# ADR-0040: Cross-dialect type definitions

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Like TypeScript `.d.ts` / `@types`: `name@version` packages in a repository per language (`defs/<language>/<package>/<version>/`: `package.toml`, `index.json` for the transpiler, `api.json` with the full API for the LSP and agents). Types in Nuclear notation. A dialect lists them in `[dialect] defs`; they give automatic imports (replacing `symbols.toml` lists), inherited members and overrides. One generator per language (Dart: `tools/defs`).

## Alternatives rejected

Hand-written symbol lists (`symbols.toml`, replaced).
