---
adr: 0074
title: "Dragonfly device"
status: "Accepted"
scope: editors
---

# ADR-0074: Dragonfly device

- Status: Accepted
- Scope: `editors` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

The app runs inside VS Code, like Android Studio's *Running Devices*: a Flutter *custom device* (`dragonfly`, "Dragonfly device") shipped with the extension appears in the device picker; launching on it runs the app on Flutter's `chrome` device with a headless Chromium, and the panel shows its screen and sends input through the Chrome DevTools Protocol (breakpoints, hot reload, hot restart). A device bar like the browsers' (iPhone, Pixel, iPad, Galaxy, laptop, responsive; zoom and rotation) over Chrome's device mode. It is Flutter web: no native plugins nor `dart:io`.

## Alternatives rejected

`web-server` (no breakpoints).
