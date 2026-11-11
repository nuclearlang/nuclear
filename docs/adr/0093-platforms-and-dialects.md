---
adr: 0093
title: "Three platforms; dialects grouped under them, never visible in the Nuclear code"
status: "Accepted"
scope: dragonfly
---

# ADR-0093: Three platforms; dialects grouped under them, never visible in the Nuclear code

- Status: Accepted
- Scope: `dragonfly`, `dragonfly-mobile`, `dragonfly-backend`, `dragonfly-web`, `nuclear` (manifest, loader), `cli`
- Date: 2026-11-04 (decided by the user)

## Context

Dialects were created so that each language could use its own things (ADR-0025, ADR-0086: "the library is the
target's"). With a Python backend (ADR-0090) and a TypeScript backend (ADR-0091) written for the same framework,
the Nuclear code of the two examples came out different: `@get("/{id}")` against `@get(":id")`,
`SQLite(url = "sqlite+aiosqlite:///x.db")` against `SQLite(storage = "x.db")`, `len(xs)` against `xs.length`,
`tests/` against `test/`. The user's verdict (2026-11-04): that loses the real potential of Nuclear; anyone could
build projects that make Nuclear incompatible with itself. "Of what use is the state management of Flutter in an
HTTP API?" — the differences that matter are between a web app, a mobile app and a backend, not between the
languages that run them.

## Decision

1. **Dragonfly has exactly three platforms: `web`, `mobile` and `backend`.** A platform is a specification written
   in Nuclear only: its project layout, `config.nuc`, kinds, decorators, connections, plugins, tests, errors and the
   standard library (ADR-0094). It names no language. The user and the agent write the platform; nothing else.
2. **A dialect is a language + framework that implements a platform**, grouped under it:
   - `backend`: `fastapi` (Python: FastAPI, SQLAlchemy 2 async, pydantic, LangChain, LangGraph; today's `langgraph`
     dialect, renamed), `nestjs` (TypeScript: NestJS 12, Sequelize, class-validator, LangChain.js); later `express`
     and others, and Dragonfly's own backend in Rust.
   - `mobile`: `flutter` (Dart); later `react-native`, `kotlin-multiplatform`, native Android...
   - `web`: none yet; `react`, `vue`, `angular` planned.
   The language dialects `typescript`, `dart` and `python` stay as the layer the Dragonfly dialects extend (and
   as targets for a single `.nuc` without Dragonfly).
3. **The Nuclear code of a project never changes when its dialect changes.** The one exception is code that uses a
   dialect's type definitions (ADR-0040: `numpy` exists only in Python): it is allowed, and it ties that code to
   that dialect; the platform's own kinds and library never do. Kinds and features may differ between platforms,
   never between the dialects of one platform. A platform's spec, docs, examples and diagnostics name no dialect.
4. **A project declares both** in `config.nuc`, the only manifest of a Dragonfly project (ADR-0050, ADR-0070):

   ```nuclear
   dragonfly(platform = "backend", dialect = "fastapi")
   app(name = "tasks_api", description = "...", version = "0.1.0")
   plugins(sqlite = true, ollama = true)
   ```

   `nuclear.toml` stays for projects of a language dialect without Dragonfly. The CLI creates a project with
   `dragonfly new -p <platform> -d <dialect> <name>` and reads `dragonfly(...)` afterwards; it knows no language
   (ADR-0072: `[toolchain]`).
5. **Folders and contract.** `dialects/<platform>/platform.toml` is the platform's contract (its kinds, decorators,
   plugins, folders and editor docs); `dialects/<platform>/<dialect>/` implements it, naming its platform in
   `[dialect]`. The loader refuses a dialect that misses a kind, decorator or plugin of its platform, and the
   loader test guards it. Language dialects stay in `dialects/<name>/`. Kinds a dialect does not bring come as
   packages on top of it, never inside it (ADR-0095).
6. **One specification per platform** (`spec/backend.md`, `spec/mobile.md`, `spec/web.md` when it exists), with the
   rules and the Nuclear; one per dialect (`spec/fastapi.md`, `spec/nestjs.md`, `spec/flutter.md`) with only how
   that dialect writes it (the equivalences `tools/spec-check` verifies). `nuclear spec --dialect fastapi` prints
   the base, the language, the platform and the dialect. One brief per platform plus the dialect's notes.
7. **Repositories** (amends ADR-0083): one per platform — `dragonfly-mobile`, `dragonfly-backend`, `dragonfly-web` —
   each holding its spec, its dialects, their runtimes, definitions and examples. Context:
   `docs/context/dragonfly-<platform>/`.
8. **The backend platform, unified** (what the two dialects had decided differently):
   - routes: `@controller("tasks")`, `@get()`, `@get("{id}")`, `@post("plan")`: segments without a leading slash,
     `{name}` placeholders bound to the parameter of that name; the dialect writes `/tasks/:id` or `/tasks/{id}`;
   - connections: `SQLite(file = "tasks.sqlite")` (`":memory:"` in tests), `Postgres(url)`, `MySQL(url)`,
     `MariaDB(url)`, each with `createTables = true` for development; `Redis(url)`, `Kafka(servers, group)`,
     `Ollama(model, baseUrl)`, `OpenAI(model)`, `Chroma(collection)`, `env("VAR", "default")`;
   - plugins: `sqlite`, `postgres`, `mysql`, `mariadb`, `redis`, `kafka`, `events`, `ai`, `openai`, `ollama`,
     `chroma` in every backend dialect;
   - tests in `test/*_test.nuc` (as the mobile platform); an invalid body answers 400 (`Invalid`) everywhere;
   - a graph state is read as `state.field` in every dialect.
9. **A conformance test per platform:** one example project per platform is built with every dialect of the
   platform from the same `.nuc` files, and each output passes its dialect's checks (`tsc --strict`, `pyright
   --strict`, tests, smoke).

## Consequences

- `AGENTS.md` rule 11. `dialects/langgraph` becomes `dialects/backend/fastapi`, `dialects/nestjs` becomes
  `dialects/backend/nestjs`, `dialects/flutter` becomes `dialects/mobile/flutter`; `spec/langgraph.md` becomes
  `spec/fastapi.md`; examples, tools and docs follow. Phases and state: `STATUS.md`.
- `Manifest` gets `platform`; `dragonfly(...)` is parsed from `config.nuc`. `editors/` and `cli/` are told
  (rule 7).
- ADR-0086's "the library is the target's" is reversed for the base types by ADR-0094.
- Until phase B lands, `spec/langgraph.md` and `spec/nestjs.md` still show the pre-ADR Nuclear of their examples;
  `spec/backend.md` holds the rule.

## Alternatives rejected

- Keeping one spec per dialect and asking the two backends to "stay close": nothing would guard it, and the two
  examples already drifted in a day.
- A `platform` field in `nuclear.toml`: a Dragonfly project has only `.nuc` files (ADR-0050).
- Naming the dialects by language (`python`, `typescript`): tomorrow `express` is TypeScript too; the framework
  is what a dialect is.
