---
adr: 0049
title: "Tests in Nuclear"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0049: Tests in Nuclear

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

Kind `tests` (folder `test/`) and `integration` (`integration_test/`) with `@mock`, `@test`, `@states` (bloc_test), `@widget`; mocks with mocktail; Dragonfly generates tests for models (JSON), repositories (http contract), routes (session/ACL) and screens.
