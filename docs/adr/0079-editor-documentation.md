---
adr: 0079
title: "Editor documentation and doc comments"
status: "Accepted"
scope: nuclear
---

# ADR-0079: Editor documentation and doc comments

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The comment right above a declaration is its documentation, with JSDoc-like tags (`@param name text`, `@return`, `@throws`, `@example` with code on the following lines, `@see`, `@deprecated`, `@since`; the type comes from the signature). The editor shows it in hover, completion and signature help; the dialect writes it with the optional `doc_comment` template. Every word of the base (`languages/docs.toml`) and of each dialect (`docs.toml` next to `dialect.toml`) has documentation; a build never reads it.
