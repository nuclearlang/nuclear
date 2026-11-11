---
adr: 0057
title: "Routes"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0057: Routes

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

go_router, always from the views: never from managers or use cases (`BuildContext`, `GoRouter` and `appRouter` only in `stateless`/`stateful`, N0301).
