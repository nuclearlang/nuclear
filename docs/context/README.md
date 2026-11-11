---
layer: L1
component: system
read-when: you need the map of the context documentation or the loading protocol
---

# Context documentation map

Layered context for people and agents (ADR-0084). Read from the top and open only the link that answers
your question: a typical task needs L0 + one L2 + one or two L3 files.

```
AGENTS.md                              L0  router + hard rules (always loaded; CLAUDE.md imports it)
STATUS.md                              L0  state: done, half done, untested, pending, log (always loaded)
docs/context/
  README.md                            L1  this map
  product/prd.md                       L1  what the platform is, for whom, requirements (FR/NFR)
  architecture.md                      L1  layers, repositories, data flow, boundaries
  tech.md                              L1  stack, versions, machine paths, commands
  workflow.md                          L1  sessions, ownership, definition of done, the user
  nuclear/CONTEXT.md                   L2  language, compiler, `nuclear` CLI, LSP, language dialects, the standard library
    syntax.md                          L3  lexer and parser rules (indentation, comments, blocks)
    emitter.md                         L3  how the emitter turns the AST into target code
    result.md                          L3  Result<E, T>, propagation, aliases
    dialect-format.md                  L3  dialect files: kinds, options, templates, docs.toml
    type-definitions.md                L3  defs/ packages, priority, generator
    projects.md                        L3  multi-file projects, [project], [architecture]
    testing.md                         L3  golden, rules, loader, projects tests; target validation
    limits.md                          L3  known limitations and what Nuclear cannot express yet
  ../../dragonfly/docs/context/dragonfly/CONTEXT.md  L2  the three platforms (web, mobile, backend), dialects, the philosophy
  ../../dragonfly-mobile/docs/context/dragonfly-mobile/CONTEXT.md  L2  the mobile platform (in dragonfly-mobile/, plan 006)
    project.md                         L3  project layout, config.nuc, assets, architecture rules
    kinds.md                           L3  every kind and decorator of the platform, as Flutter writes them
    runtime.md                         L3  the `dragonfly` Dart package services (events, session, logs...)
    security.md                        L3  security and performance rules to warn about
  ../../dragonfly-backend/docs/context/dragonfly-backend/CONTEXT.md  L2  the backend platform (in dragonfly-backend/, plan 006)
  ../../dragonfly-web/docs/context/dragonfly-web/CONTEXT.md  L2  the web platform (in dragonfly-web/, plan 006)
  ../../dragonfly/docs/context/cli/CONTEXT.md  L2  the `dragonfly` CLI (in dragonfly/)
    build.md                           L3  build pipeline, markers, repair, line map
    devices.md                         L3  Dragonfly devices (window and terminal)
    packages.md                        L3  config.nuc sync, third-party definitions, assets
    landing.md                         L3  landing page, colors, help
  editors/CONTEXT.md                   L2  LSP server and the editor extensions
    lsp.md                             L3  server modules and decisions
    ../../nuclear-vscode/docs/context/editors/vscode.md     L3  VS Code extension: Dragonfly run/debug/device
    ../../nuclear-vscode/docs/context/editors/theme.md      L3  Chernobyl theme, icons, code metrics
    ../../nuclear-jetbrains/docs/context/editors/jetbrains.md  L3  Android Studio / IntelliJ plugin
    ../../nuclear-zed/docs/context/editors/zed.md           L3  Zed extension
  the website repository's docs/context/website/CONTEXT.md  L2  documentation website
docs/adr/                              --  decision records: open one only to learn WHY
spec/                                  L3  user-facing specs (also the website and `nuclear help`)
specs/NNN-feature/plan.md              L4  feature plans (spec → plan → tasks)
docs/language/, docs/dragonfly/        L3  user guide and tutorial
```

## Loading protocol

1. L0 is always in context. It says which L2 owns each folder.
2. Open the L2 `CONTEXT.md` of the area you touch. It lists its L3 files with a `read-when` line.
3. Open an L3 file or a spec only when its `read-when` matches the task.
4. Open an ADR only to learn why a rule exists or before proposing a change to it.
5. Never read a whole folder "to be safe".

## Writing rules

- Every L1–L3 file starts with front-matter: `layer`, `component`, `read-when` (one line: when the file is
  worth its tokens).
- Size caps: L0 ≤ 100 lines, L2 ≤ 150 lines, L3 as long as the topic needs. When an L2 grows past its cap,
  move a section into a new L3 and leave a one-line link.
- One fact lives in one file; other files link to it. Specs hold the language and dialect rules; context
  files hold how the code is built and why; `STATUS.md` holds state; ADRs hold decisions.
- English only (ADR-0082).
- After the repository split (ADR-0083) each `docs/context/<repo>/` moves into that repository and its
  `CONTEXT.md` becomes the core of that repository's `AGENTS.md`.
