---
adr: 0061
title: "Splash"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0061: Splash

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`SplashScreen` comes in the package: `child` for at least `minDuration` (3 s by default) and until `onLoad` finishes (initial data); then it goes to `next` with the router. An `onLoad` error shows `onError` with `retry`, or without it is logged and continues to `next`.
