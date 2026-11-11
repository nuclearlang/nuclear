---
adr: 0080
title: "Comment lines with ` * `"
status: "Accepted"
scope: nuclear
---

# ADR-0080: Comment lines with ` * `

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A multi-line comment continues on lines that start with ` * `, the `*` right below the second `/` of `//` (like `/** */`). A lone `*` is an empty line; `@example` code has more space after the `*`. A line indented another way is N0021, with its fix. In the editor, Enter inside a comment adds the ` * ` and Enter on an empty ` * ` line ends it.
