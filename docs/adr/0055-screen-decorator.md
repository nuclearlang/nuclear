---
adr: 0055
title: "`@screen`"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0055: `@screen`

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

One extensible decorator for screens: `@screen(route = "/", providers = [...], loggedIn = { forward: LoginScreen }, acl = {...}, shell = {...})`. Its objects use `:`, like every Nuclear object. `forward` is required in `loggedIn`.

## Alternatives rejected

`@route` + `@provider` (replaced).
