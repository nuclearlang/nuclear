---
adr: 0039
title: "The `flutter` dialect as a framework"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0039: The `flutter` dialect as a framework

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Minimizes Flutter: Nuclear holds the business logic and the transpiler generates the app with a light clean architecture (the layers exist in the generated Dart, not in Nuclear). It relies on official ecosystem libraries: `bloc`/`flutter_bloc` (state), `json_serializable`/`json_annotation` (JSON), `get_it` (dependency injection), `dio` (network). The SDK, packages, builds and devices are handled by the Flutter CLI.
