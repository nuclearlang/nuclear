---
adr: 0063
title: "Native splash"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0063: Native splash

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`@splash(color, image)` on the splash screen: the project generates `flutter_native_splash.yaml` and the CLI runs `flutter_native_splash:create` (Android, also 12+, iOS and web; on Linux and Windows the window waits for the first frame). `startDragonfly()` removes the native splash on the first frame; the `@splash` route is the initial one. Images in `assets/` are copied to the app.
