---
adr: 0069
title: "Only third-party packages"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0069: Only third-party packages

- Status: Accepted
- Scope: `dragonfly-flutter` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

A project only adds packages the framework does not use: neither the dialect's (`defs`) nor the ones Dragonfly adds (`[project.manifest] reserved`). `dragonfly flutter add` rejects them before changing anything and `config.nuc` marks them with N0303 on their line (check, build and editor). A project without `config.nuc` gets one on its first `check`/`build`. The LSP loads the project definitions (`build/defs.lock`).
