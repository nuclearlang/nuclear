---
adr: 0083
title: "Repository split"
status: "Accepted"
scope: nuclear
---

# ADR-0083: Repository split

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: 2026-11-03

## Decision

The platform is split by release cycle and audience, like Rust, Deno and Gleam: `nuclear` (language spec, compiler crates, `nuclear` CLI, the LSP as its own crate and binary, language-target dialects `typescript` and `dart` with their type definitions and generator), `nuclear-vscode`, `nuclear-jetbrains`, `nuclear-zed` (base extensions; they run the LSP binary as an external component and talk to it only through the LSP protocol), `dragonfly` (the neutral framework philosophy and the `dragonfly` CLI), one repository per framework dialect (`dragonfly-flutter`: dialect + Dart runtime + Flutter definitions + example; later `dragonfly-nestjs`, `dragonfly-rust`), their editor extensions each in its own repository (`dragonfly-flutter-vscode`, `dragonfly-flutter-jetbrains`), and `nuclear-website` (the site only; content stays with the code). Framework extensions never register the language or the LSP, activate only on their dialect's project marker, and prefix every command, setting and debug type with their dialect.

## Alternatives rejected

One monorepo; the LSP in its own repository (it walks the AST and would break on every parser change).
