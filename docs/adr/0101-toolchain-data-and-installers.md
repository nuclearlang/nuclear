---
adr: 0101
title: Run, debug and install from data
status: Accepted
scope: dragonfly
---

# ADR-0101: Run, debug and install from data

- Status: Accepted
- Scope: `dragonfly`, `nuclear`, `editors`
- Date: 2026-11-10

## Context

The user asked to remove every hard-coded value (ports, urls, commands, dialect names, the website's menus and texts)
and to install NuclearLang with one command on Linux, macOS and Windows, as Claude Code or Ollama do. The `dragonfly`
CLI, the VS Code extension and the website each carried their own copy of how a dialect is run, debugged and served.

## Decision

1. **A dialect's `[toolchain]` is the only source** of how its generated project is installed, compiled, run, tested,
   served by an editor and debugged: commands, `url`, `ready`, `installed`, `[[toolchain.launch]]`,
   `[toolchain.settings]`, `extensions`, with `[toolchain.<os>]` overrides (spec/dialects.md). `dragonfly` writes the
   editor workspace from it; nothing in Rust or TypeScript names a port or a framework command.
2. **A platform names its default dialect** (`[platform] default_dialect`): `dragonfly new -p <platform>` and the legacy
   Flutter apps without `dragonfly(...)` use it.
3. **The project card carries `toolchain`** (`url`, `serve`, `installed`): the editor reads it instead of loading the
   dialect (web device server, HTTP tool base urls, "needs install").
4. **`nuclear build` removes the output of a removed `.nuc`**, from the previous line map.
5. **Installers**: `tools/release/package.sh` makes `nuclearlang-<version>-<os>-<arch>.tar.gz|zip` + `.sha256` with
   the layout of `tools/release/stage.sh` (also used by `./install.sh`); `install.sh` (`curl … | sh`) and `install.ps1`
   (`irm … | iex`) download the release of the system from GitHub, check the SHA-256, install into the user's folders
   (no sudo, no administrator), set the PATH, copy the agent skill and the VS Code extension, and uninstall.
   `.github/workflows/release.yml` builds Linux (musl, x86_64 and aarch64), macOS (x86_64, arm64) and Windows (x86_64,
   arm64) on a tag `v<version>`. The website serves both installers at its root and takes its texts, menus and links
   from `site.toml`.

## Consequences

A new dialect is runnable and debuggable from the editor with no code change. The mobile path of the CLI still calls
Flutter directly (its SDK lookup, devices and hot reload); moving it to `[toolchain]` is its own step.

## Alternatives rejected

- A package manager per system (Homebrew, winget, apt) first: three formats to maintain before there is a release;
  they can wrap the same archives later.
- `cargo install`: needs Rust on the user's machine and does not ship the dialects, runtimes or skill.
