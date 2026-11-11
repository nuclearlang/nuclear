---
adr: 0060
title: "Translations"
status: "Accepted"
scope: dragonfly
---

# ADR-0060: Translations

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Native JSON dictionaries (no third-party libraries), one per component and language in `assets/i18n/<locale>/<component>.json`; key `<component>.<path>`, `"key".tr()`. Optional remote dictionaries, stored by version on every platform.
