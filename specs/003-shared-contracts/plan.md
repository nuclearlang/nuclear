---
layer: L4
component: system
read-when: you start step 3 of the competitiveness plan (STATUS.md "Next"): one contract for the backend and its clients
---

# 003 — Shared contracts: the backend written once, its clients generated

Step 3 of the competitiveness plan (STATUS.md "Next", decided by the user 2026-11-06). **Delivered 2026-11-06**
(ADR-0098): F1–F4 as below with these differences: the word is `contracts(...)`; the generated folder is committed
and regenerated on every build (path) or on `nuclear contracts update` (url); entities travel when a route returns
them; the error model carries `message` only; F5 shows versions (`nuclear contracts list`) without a drift warning
yet. Spec: `spec/projects.md` §1.2.

## 1. Goal

A model, an entity and a route are written **once**, in the backend project. Every client project of the same
app (mobile, web) gets, without writing them, the same models, a typed repository per controller and the routes
with their parameters. When the backend changes, the clients fail to build until they follow, instead of failing
at run time with a 404 or a wrong field.

## 2. Diagnosis: what happens today

The backend example `examples/examples-backend/tasks_api` and a mobile app that uses it write the same thing
twice, by hand, with no check between them.

Backend (`components/tasks/`):

```nuclear
// data/entities/task.nuc
@entity("tasks")
entity Task()
  @generated()
  id: int
  @column(length = 120)
  title: string
  done: boolean = false

// domain/models/new_task.nuc
NewTask
  @required
  @minLength(3)
  title: string

// api/tasks_controller.nuc
@controller("tasks")
controller TasksController(tasks: TasksAppComponent)
  @get("{id}")
  one(id: int) Result<AppError, Task>
    tasks.find(id)

  @post
  @status(201)
  create(body: NewTask) Result<AppError, Task>
    tasks.create(body)

  @get
  search(done?: boolean, page: int = 1) Result<AppError, Task[]>
    tasks.search(done, page)
```

Mobile (`components/tasks/`), written again by a person or an agent, reading the backend's code:

```nuclear
// domain/models/task.nuc
Task
  @primary
  id: int
  title: string
  done: boolean

// domain/models/new_task.nuc
NewTask
  title: string

// data/task_repository.nuc
@repository(path = "tasks")
repository TaskRepository
  @get(path = ":id")
  one(id: int) Result<TaskError, Task>

  @post
  create(body: NewTask) Result<TaskError, Task>

  @get
  search(done?: boolean, page: int = 1) Result<TaskError, Task[]>
```

Three copies of `Task` (entity, backend model, mobile model), two of `NewTask`, the routes typed twice. A renamed
field (`done` → `completed`) breaks the app at run time. The validation rules (`@minLength(3)`) are lost in the
client, so a form sends what the API will refuse.

## 3. The design in one picture

```
backend project (tasks_api)                                client project (tasks_app, mobile or web)
  components/tasks/data/entities/task.nuc                   config.nuc: contracts(tasks_api = "../tasks_api")
  components/tasks/domain/models/new_task.nuc     ───►      contracts/tasks_api/           (generated, read-only)
  components/tasks/api/tasks_controller.nuc                   models/task.nuc
          │                                                   models/new_task.nuc
          ▼ nuclear build                                     tasks_repository.nuc
  .nuclear/contract.json  (models, routes)        ───►      then the normal build of the client dialect
```

The contract is **Nuclear generated into the client**, not target code: the client's dialect already knows how to
write a model and a repository (ADR-0096 made the mobile model a table too). The compiler only learns to write
the contract and to read it back as `.nuc`; no new kind, no new template variable.

## 4. Features

### F1 — The backend writes its contract

`nuclear build` of a backend project writes `.nuclear/contract.json` next to the project card (`spec/projects.md`
§1.1): every public model and entity (name, fields, types, optional, JSON name, validation rules, the key), every
controller (base path) and route (verb, path, path parameters, query parameters, body model, answer type, error
type, status).

```json
{
  "app": "tasks_api", "version": "1.0.0",
  "models": {
    "Task": { "key": "id", "fields": [{ "name": "id", "type": "int" }, { "name": "title", "type": "string", "rules": { "minLength": 3 } }, { "name": "done", "type": "boolean" }] },
    "NewTask": { "fields": [{ "name": "title", "type": "string", "rules": { "required": true, "minLength": 3 } }] }
  },
  "controllers": {
    "TasksController": { "path": "tasks", "routes": [
      { "name": "one", "verb": "get", "path": "{id}", "params": [{ "name": "id", "type": "int", "in": "path" }], "returns": "Task", "error": "AppError" },
      { "name": "create", "verb": "post", "status": 201, "body": "NewTask", "returns": "Task" },
      { "name": "search", "verb": "get", "params": [{ "name": "done", "type": "boolean", "optional": true, "in": "query" }, { "name": "page", "type": "int", "default": "1", "in": "query" }], "returns": "Task[]" }
    ] }
  }
}
```

Done when: `backend_example_builds_with_every_backend_dialect` checks the contract of `tasks_api` is the same with
`fastapi` and `nestjs` (it is the platform's, not the dialect's).

### F2 — A client declares the backends it talks to

In `config.nuc` of the client, next to `dragonfly(...)` and `app(...)`:

```nuclear
contracts(tasks_api = "../tasks_api")              // a path (monorepo) ...
contracts(tasks_api = "https://api.example.com")    // ... or the running API: GET /.nuclear/contract.json
```

The name (`tasks_api`) is the folder under `contracts/` and the `connectionName` of the connection the generated
repositories use:

```nuclear
appConfig ProjectConfig
  connections: Connection[] = [
    Http(connectionName = "tasks_api", baseUrl = "https://api.example.com/"),
  ]
```

Done when: `Manifest` reads `contracts(...)`; N0303 names a missing path or a name without a connection.

### F3 — The client's build writes the contract as Nuclear

Before building, `nuclear build` (and `dragonfly build`) writes `contracts/<name>/` from the contract:

```nuclear
// contracts/tasks_api/models/task.nuc — generated from tasks_api 1.0.0; do not edit
Task
  @primary
  id: int
  @minLength(3)
  title: string
  done: boolean

// contracts/tasks_api/models/new_task.nuc
NewTask
  @required
  @minLength(3)
  title: string

// contracts/tasks_api/tasks_repository.nuc
@repository(path = "tasks", connection = "tasks_api", saveLocal = true)
repository TasksRepository
  @get(path = ":id")
  one(id: int) Result<ApiError, Task>

  @post
  create(body: NewTask) Result<ApiError, Task>

  @get
  search(done?: boolean, page: int = 1) Result<ApiError, Task[]>
```

The validation rules travel with the model, so a mobile `form` built on `NewTask` refuses what the API refuses.
`ApiError` (`message`, `status`) is the client-side twin of the backend's `AppError`; `saveLocal` is on because
the models carry their key (ADR-0096), so every GET gets its `<method>AsLocal()` twin and the app works offline for
what it already fetched.

The generated folder is read-only for the architecture rules: a client component uses the repository as it uses
its own (`N0301`: only from a `usecase` or an `aggregate`), and may not declare a name the contract declares.

Done when: `examples/examples-flutter-dialect/tasks_app` (new, small) builds from `tasks_api` with no model of its
own; the golden includes the generated `.nuc`; `projects.rs` has a test that changes a field in the backend copy
and sees the client build fail with the field's name in the diagnostic.

### F4 — The client's use of the contract, end to end

```nuclear
// components/tasks/domain/usecases/create_task_usecase.nuc
usecase CreateTaskUseCase
  (tasks: TasksRepository)

  exec(title: string) Result<ApiError, Task>
    ~tasks.create(NewTask(title = title))

// components/tasks/presentation/list/managers/tasks_manager.nuc
@stateModel(TasksState)
statemanager TasksManager(tasks: TasksAppComponent)
  @state(TasksState.loaded)
  load() Result<ApiError, Task[]>
    ~tasks.search(done = false)
```

Nothing in the client names a field of `Task` that the backend did not declare: the editor completes them from
the generated `.nuc`, the LSP shows "from contract tasks_api 1.0.0" in the hover (the project card lists the
contract kinds as `contract:tasks_api`, next to `platform:` and `package:` of ADR-0095).

Done when: the tutorial has a chapter "one backend, one app" that builds both from scratch.

### F5 — Versions and drift

The contract carries the backend's `app(version)`. A client build prints `contract tasks_api 1.0.0` and, with a
path contract, regenerates when the backend's contract is newer; with a URL contract, `dragonfly contracts update`
fetches it (never at every build: a build must not need the network).

Done when: a stale contract is a warning (N03xx) with the two versions.

## 5. Work order

1. ADR-0097 (or the next free number): the decisions above, the alternatives rejected (OpenAPI as the contract:
   loses `Result` errors, validation rules and the key; target code generated directly: a second code path per
   dialect; a shared Nuclear library imported by both projects: makes the backend depend on the client's platform).
2. F1 (compiler: `project::write_contract` from the summaries the project context already has).
3. F2 + F3 (manifest, the Nuclear writer, N0303).
4. F4 (the example app, docs: `spec/projects.md` §1.2, `spec/backend.md`, `spec/mobile.md`, website guide).
5. F5.

## 6. Decisions pending for the user

- The word: `contracts(...)` in `config.nuc`, or `backends(...)`.
- Whether the generated folder is committed (visible to agents and reviewers, builds without the backend
  checked out) or ignored and regenerated (never stale). Recommended: committed, regenerated on build.
- Whether entities (tables) are part of the contract or only what controllers answer. Recommended: only what a
  route reads or answers; an entity never leaves the backend unless a route returns it.
