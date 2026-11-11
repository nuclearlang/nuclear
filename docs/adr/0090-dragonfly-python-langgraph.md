---
adr: 0090
title: "Dragonfly for Python: the langgraph dialect (FastAPI, SQLAlchemy, pydantic, LangChain, LangGraph)"
status: "Accepted"
scope: dragonfly
---

# ADR-0090: Dragonfly for Python: the langgraph dialect

- Status: Accepted
- Scope: `dragonfly` (`dialects/langgraph`, `packages/dragonfly-py`)
- Date: 2026-11-03

## Context

The user asked (2026-11-03) for Dragonfly on Python with LangChain/LangGraph, based on the domain kinds of Flutter:
REST APIs with `@controller`, `@get`, `@post` like NestJS, repositories that reach tables through an ORM instead of an
HTTP API but read almost the same, CRUD like TypeORM with a model, connections to external systems (Kafka, Redis,
Postgres, MySQL) in the config, pydantic, and the most used library for each basic need, without two that do the same.
Decided with the user: SQLAlchemy 2 async as the ORM.

## Decision

- **Dialect `langgraph` extends `python`.** Same architecture as Flutter's Dragonfly: components, `usecase` (one
  `exec` returning `Result`), `appcomponent` (aliases through `inject`), `service`, `aggregate`, `config`/`appConfig`,
  architecture rules; kinds by folder for `entity`, `model`, `state`, `tests`.
- **Data:** `entity` = a SQLAlchemy 2 table whose field decorators are TypeORM's (`@entity`, `@generated`, `@primary`,
  `@column(...)`, `@created`, `@updated`); `repository` = `Repository[Entity]` with CRUD (`find`, `findAll`,
  `save`, `remove`) and methods whose lines build one query per call (`where`, `orderBy`, `limit`, `offset`, then
  `getOne` → `Result`, `getMany`, `count`, `exists`, `update`, `delete`): the kind writes `q = self.query()` and
  `self_member = "q.{{name}}"`, so the user's lines stay as in the request. Repository methods return an awaitable
  (callers write `~`).
- **API:** `controller` + runtime decorators over FastAPI; a `Result` answers its value or the `AppError`'s status
  (`NotFound` 404, `Invalid` 400, `Unauthorized` 401, `Forbidden` 403, `Conflict` 409); an entity in a body or an
  answer goes through a pydantic schema built from its columns (`schema_of`). `model` = pydantic.
- **AI:** `tools` (methods → LangChain StructuredTools, docstring = description), `agent` (LangChain `create_agent`
  with an in-memory checkpointer, `ask`/`stream`), `state` (TypedDict, `Messages`) and `graph` (StateGraph from
  methods, `@start`, `@next`, `Command` for run-time routes). Models through `init_chat_model`/`init_embeddings`
  (OpenAI and Ollama behind one API), Chroma through langchain-chroma.
- **Messaging and cache:** `listener` with `@consumer("topic")` (aiokafka), `publish`, `cacheGet`/`cacheSet`
  (redis.asyncio).
- **Runtime `packages/dragonfly-py`** (import name `dragonfly`): injector, connections opened by the app's lifespan,
  `create_app`, testing helpers (`running`, `client`, `FakeModel`, in-memory databases). Projects use `nuclear.toml`;
  the build writes `app/main.py`, `tests/conftest.py` and `pyproject.toml`.

## Consequences

- `examples/examples-langgraph/rick_api` (golden) builds to a project that passes `pyright --strict`, its Nuclear tests
  (pytest) and `tools/langgraph-check/check.sh` end to end (REST CRUD, 409, 422, 404, the agent over HTTP with a fake
  model, tools, the graph, OpenAPI).
- Not yet: the `dragonfly` CLI for this dialect (plan 002 D5), entity relations and migrations, a run against real
  Postgres, MySQL, Redis, Kafka, OpenAI or Ollama.
