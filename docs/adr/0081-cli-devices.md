---
adr: 0081
title: "Dragonfly devices from the CLI"
status: "Accepted"
scope: dragonfly
---

# ADR-0081: Dragonfly devices from the CLI

- Status: Accepted
- Scope: `dragonfly` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`dragonfly device [<device>]` runs the app in its own window, like an emulator, and `dragonfly terminal-device` shows it inside the terminal. Both share the backend (Flutter web in Chromium, device emulation over CDP, input, hot reload); only the frontend changes. Several at once are numbered. The window takes the device's ratio and at most 85 % of the screen height; on tiling window managers (Hyprland, Sway, i3) it floats. The terminal draws with the best it supports: images (kitty, sixel) or colored blocks with the app's texts written as letters in their exact place (given by the `dragonfly` package in debug). An own engine without a browser is postponed.

## Alternatives rejected

An own rendering engine (postponed).
