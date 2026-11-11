---
adr: 0075
title: "Code in English, documentation in Spanish"
status: "Superseded in part by ADR-0082"
scope: nuclear
---

# ADR-0075: Code in English, documentation in Spanish

- Status: Superseded in part by ADR-0082
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Everything that is not documentation is in English: code, comments, texts, test names, diagnostics and CLI output, dialects (comments, `note`, templates) and examples. Only the `.md` files (specs and docs) were in Spanish. Exceptions: localization data (`es` translations, the CLI motto with `es` locale) and unicode tests.
