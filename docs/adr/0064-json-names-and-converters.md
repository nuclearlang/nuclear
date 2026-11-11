---
adr: 0064
title: "JSON names and converters"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0064: JSON names and converters

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`@key("json_name")` on a model field is its name in the JSON (Dart: `@JsonKey(name:)`). `converter Name` + `fromJson(json: J) T` + `toJson(value: T) J` is a converter (Dart: `JsonConverter<T, J>`, types taken from the `fromJson` signature); a field uses it with `@Name`. In Dart, a PascalCase decorator without arguments is a call to its const constructor (`@Name` → `@Name()`).
