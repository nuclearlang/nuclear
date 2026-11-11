---
adr: 0092
title: "Plugins in config.nuc, and Genkit for Dragonfly Flutter"
status: "Accepted"
scope: dragonfly
---

# ADR-0092: Plugins in config.nuc, and Genkit for Dragonfly Flutter

- Status: Accepted
- Scope: `dragonfly` (flutter, langgraph, nestjs dialects, the runtimes, `cli/`), `nuclear` (`[project.plugins]`)
- Date: 2026-11-04

## Context

The user asked (2026-11-04) for full Genkit support in Dragonfly for Flutter, added only when a project asks for it:
"a section in config.nuc, like a pubspec or package.json but only to enable these packages, so each project does not
get heavier", for Flutter, Python and TypeScript. Decided with the user: Genkit in the app and remote flows; providers
Firebase AI, Google AI, Vertex AI, OpenAI, Anthropic; kinds flow + tools, chat, remote flow, embeddings/RAG; the CLI may
be changed for it.

## Decision

- **Plugins** (`spec/projects.md` §3.3): a dialect declares `[project.plugins.<name>]` (packages, dev packages, type
  definitions, kinds, names, requires); `config.nuc` enables them with `plugins(name = true)`. A kind or name of a plugin
  not enabled is **N0304** with the line to add; unknown plugins or values are N0303. Project templates get
  `plugins.<name>`, `plugins.packages`, `plugins.dev_packages`; the Python and TypeScript manifests list only the
  enabled packages; the `dragonfly` CLI adds and removes the Flutter packages of the plugins (`.dragonfly_plugins`).
  The plugins' type definitions are always loaded (index only), so editors know them.
- **Catalogs:** flutter `genkit` + one per provider; langgraph `sqlite`, `postgres`, `mysql`, `redis`, `kafka`, `ai`,
  `openai`, `ollama`, `chroma`; nestjs the same plus `mariadb` and `events` (in-process publish). The runtimes keep the
  optional parts in their own modules (`dragonfly.ai`, `dragonfly.events`; `dragonfly-nest/ai`,
  `dragonfly-nest/events`, kafkajs/ioredis loaded when opened), so a project without a plugin does not need its package.
- **Genkit Dart** (`packages/dragonfly_genkit`): providers are connections of the project config (descriptors without
  plugin imports); the generated `lib/dragonfly_genkit.g.dart` (only with the plugin) turns them into plugins and
  registers a schema per model of the app (`SchemanticType.from` with its fields and `fromJson`, no schemantic codegen).
  Kinds `tools` (doc comment = description, parameters = JSON schema), `chat` (history, `send`, `stream`,
  `ChatController`), `flow` (`GenkitFlow`, `generateObject` with a text fallback for servers without structured output),
  `remote` (`defineRemoteAction`, call and stream), `knowledge` (embed + cosine search, `context` for RAG). Tests:
  `startFakeGenkit`.
- **Generic emitter fixes found on the way:** `async_methods` also for methods that await nothing; `async` in the
  `methods` of kind templates.

## Consequences

- `examples/examples-flutter-dialect/genkit-assistant` (golden) builds with only its plugins' packages, `flutter analyze`
  0 issues, 4 tests; run against a local Ollama (chat with tools, a flow with structured output, knowledge with
  `embeddinggemma`); remote flows tested against a Genkit Dart HTTP server. The examples of langgraph and nestjs enable
  their plugins in `config.nuc`.
- Not run: Firebase AI, Google AI, Vertex AI, OpenAI's and Anthropic's real services (no keys), a device.
