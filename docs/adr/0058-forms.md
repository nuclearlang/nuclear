---
adr: 0058
title: "Forms"
status: "Accepted"
scope: dragonfly
---

# ADR-0058: Forms

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Format validation belongs to presentation: a form model (kind `form`, `presentation/<p>/forms/`) with the rules as decorators on its fields (`@required`, `@email`, `@minLength(n)`...), a form manager (kind `formmanager`, `@form(F)`) that only validates, and widgets (`DragonflyForm`, `FormTextField`...) bound by field name. Business rules stay in the domain; `@validate("field")` asks the component.
