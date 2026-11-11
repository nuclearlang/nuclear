---
adr: 0091
title: "Dragonfly for NestJS: the nestjs dialect with Sequelize, Kafka, Redis and LangChain.js"
status: "Accepted"
scope: dragonfly
---

# ADR-0091: Dragonfly for NestJS: the nestjs dialect with Sequelize, Kafka, Redis and LangChain.js

- Status: Accepted
- Scope: `dragonfly` (`dialects/nestjs`, `packages/dragonfly-nest`), `nuclear` (emitter, `tools/defs/typescript`)
- Date: 2026-11-03

## Context

The user asked (2026-11-03) for the TypeScript dialect with NestJS using Sequelize as the ORM, and decided: the
official integration (`@nestjs/sequelize` + `sequelize-typescript`), turning the existing `nestjs` dialect (kinds
`controller` and `service` only) into Dragonfly, and the scope of langgraph: CRUD, Redis and Kafka, LangChain.js.

## Decision

- **Same Dragonfly as langgraph** (ADR-0090): entity, repository (CRUD + one query per method call, `self_member =
  "q.{{name}}"`), model, usecase, appcomponent, service, aggregate, controller, appConfig/config, listener, tools,
  agent, state, graph, tests; the same query names (`where`, `orderBy`, `getOne`...), errors (`AppError` with HTTP
  statuses) and decorators. Names stay as written (Nuclear's camelCase is TypeScript's).
- **NestJS 12, ES modules:** relative imports end in `.js`, `"type": "module"`, `tsconfig` nodenext. Tests run with
  `node:test` on the compiled code (decorator metadata, no test framework to configure); `expect(cond)` is
  `assert.ok`.
- **Generated per project:** `src/app.module.ts` (a global module per component, the app module with
  `SequelizeModule.forRootAsync` so tests use SQLite in memory), `src/main.ts`, `package.json` (drivers by the
  connections the code uses: `uses.<Name>`), `tsconfig.json`.
- **Nest idioms:** use cases injected in the appcomponent's constructor (`alias_via = "constructor"`), the
  repository's model with `@InjectModel`, route parameters, body and query read by themselves (`route` in
  `params_list`: `@Param` + `ParseIntPipe`, `@Body`, `@Query`), DTOs validated with class-validator from the type and
  the same rule decorators as Dragonfly's forms (`@minLength`, `@email`...). A `Result` answered by a controller goes
  through `ResultInterceptor`.
- **Messaging and AI:** kafkajs (`publish`, listeners with `@consumer`, missing topics created; without Kafka the
  listeners of the process receive it) and ioredis (`cacheGet`/`cacheSet`); LangChain.js `tool` + zod schemas from the
  parameters (`doc_text`/`doc_literal`), `createAgent`, LangGraph.js `StateGraph` over `Annotation.Root`, models
  through `initChatModel` (Ollama, OpenAI).
- **Type definitions of TypeScript** (`tools/defs/typescript/gen.js`, the TypeScript compiler over the `.d.ts`):
  the standard library (implicit) and NestJS, Sequelize, class-validator, class-transformer, RxJS and the runtime.
  The order of `defs` decides a name two packages export (class-validator's `IsInt`, Nest's `Query`).
- **Generic emitter additions:** `relative` in `project_import` (+ the `typescript` dialect's), `alias_via =
  "constructor"`, `route`, `field_decor`, `components` and `uses` in project templates, `tinfo` in `field`,
  `type_args_as_args` with names, `doc_text`/`doc_literal`, `left_infix`/`right_infix`/`null` in operators (TypeScript
  writes `== null` / `!= null`, which also covers undefined); N0219 only with native named arguments.

## Consequences

- `examples/examples-nestjs/tasks_api` (golden) builds to a project that passes `tsc --strict`, its Nuclear tests
  (`node:test`: CRUD, conflict, listener, tools) and `tools/nestjs-check/check.sh` (HTTP CRUD, 400/404/409); run by hand
  with local Ollama (the agent lists and adds tasks, the planner graph creates them) and against real Redis and Kafka
  (temporary containers).
- The single-file example `users.nuc` and the rule test changed: `controller Users` no longer becomes
  `UsersController` (names are written as they are).
- Not yet: entity relations and migrations, the `dragonfly` CLI for this dialect.
