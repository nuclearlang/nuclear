---
adr: 0059
title: "Dialogs"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0059: Dialogs

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`DragonflyDialog` like a Bootstrap modal (header, body, footer), types info/success/warning/danger with the color and icon of the app theme, sizes small/medium/large, localized Accept/Close; `Popups.toast`/`snackbar` for messages that go away on their own.
