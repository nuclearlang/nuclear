---
layer: L4
component: dragonfly-backend
read-when: you start step 4 of the competitiveness plan (STATUS.md "Next"): what a real backend needs
---

# 004 — Backend parity: what a real API needs that the backend platform lacks

Step 4 of the competitiveness plan (STATUS.md "Next", decided by the user 2026-11-06). Status: F1, F3 and F4 DONE
2026-11-10 (ADR-0100: `@primary(generated)`, TypeORM relations with `relations(...)`, `@transactional`, `paginate` →
`Page<T>` also on the mobile local tables); F2 migrations DONE 2026-11-10 (ADR-0100 §5: Alembic, and Sequelize with the runtime's own diff); F5–F9 not
started.

Every feature is written **once in Nuclear** and both dialects (`fastapi`, `nestjs`) implement it: the platform
spec gets the Nuclear, each dialect spec its output, `tools/fastapi-check` and `tools/nestjs-check` run it.

## 1. Diagnosis

Today a backend is a CRUD over one table per repository with SQLite (Postgres, MySQL and MariaDB are connections
but were never run), events and AI agents. It has no relations, no migrations, no transactions, no pagination, no
authentication, no scheduled work, no uploads, no deploy file, no metrics; its CLI builds and tests but cannot
generate a kind, check a project or rebuild on save. A real API needs all of that in its first week.

## 2. Features

### F1 — Relations between entities

DONE 2026-11-10 as ADR-0100 §2; the example below is the first idea, the delivered form is in the ADR.

```nuclear
@entity("projects")
entity Project()
  @generated()
  id: int
  name: string
  @hasMany(Task)                     // Project.tasks: Task[]
  tasks: Task[]

@entity("tasks")
entity Task()
  @generated()
  id: int
  title: string
  @belongsTo(Project)                // the column project_id, the field Task.project
  project: Project
```

Repositories load a relation on demand, never by default:

```nuclear
repository ProjectRepository
  withTasks(id: int) Result<NotFound, Project>
    where({ "id": id })
    include("tasks")
    getOne()
```

| | fastapi (SQLAlchemy 2) | nestjs (sequelize-typescript) |
|---|---|---|
| `@hasMany(Task)` | `relationship("Task", back_populates="project")` | `@HasMany(() => Task)` |
| `@belongsTo(Project)` | `project_id: Mapped[int] = mapped_column(ForeignKey("projects.id"))` + `relationship` | `@ForeignKey(() => Project) projectId` + `@BelongsTo(() => Project)` |
| `include("tasks")` | `selectinload(Project.tasks)` | `include: [Task]` |

Also `@hasOne`, `@manyToMany(Tag, through = "task_tags")`. Done when: `tasks_api` has `Project` ↔ `Task`, a route
answers a project with its tasks, both checks pass, and the contract (plan 003) carries the nested model.

### F2 — Migrations

DONE 2026-11-10: the delivered form is ADR-0100 §5 (`migrations/` kept with the sources, not `build/`; nestjs
without umzug). The example below is the first idea.

Today `createTables = true` creates tables at start (development only). A real database changes by migrations:

```sh
dragonfly migrate make "add done to tasks"     # compares the entities with the migrations folder, writes one
dragonfly migrate                              # applies the pending ones
dragonfly migrate down                         # reverts the last one
```

The migration is written in the dialect's tool (Alembic for `fastapi`, umzug + Sequelize for `nestjs`), under
`build/migrations/`, from the diff between the entities and the last migration; the user never writes SQL for
the common cases (a column, a table, an index, a relation). `SQLite(createTables = true)` stays for tests.

Done when: the check scripts run `migrate make` + `migrate` on a fresh SQLite and the tests pass against it.

### F3 — Transactions

DONE 2026-11-10 as ADR-0100 §4; the example below is the first idea, the delivered form is in the ADR.

Several repositories in one unit of work:

```nuclear
usecase MoveTask(tasks: TaskRepository, projects: ProjectRepository)

  @transactional
  exec(taskId: int, projectId: int) Result<AppError, Task>
    project = ~projects.find(projectId)
    task = ~tasks.find(taskId)
    ~projects.touch(project.id)
    ~tasks.save(task.copyWith(project = project))
```

`@transactional` on `exec`: the use case runs inside one transaction of the default database; an `Err` or a
thrown error rolls it back. (`fastapi`: `async with session.begin()`; `nestjs`: `sequelize.transaction` + CLS
namespace so repositories pick it up.) Done when: a test asserts a failed second write leaves the first undone.

### F4 — Pagination

DONE 2026-11-10 as ADR-0100 §3; the example below is the first idea, the delivered form is in the ADR.

`Page<T>` exists in the mobile platform (`shared/models/page.nuc` of the example). The backend gets it as a
platform model and repositories a `paginate`:

```nuclear
repository TaskRepository
  page(page: int, size: int = 20) Page<Task>
    orderBy("id")
    paginate(page, size)             // Page<Task>: items, page, size, total, pages
```

A controller returns it as is; the contract (plan 003) carries `Page<Task>` so the mobile `Page<T>` is generated,
not written. Done when: `tasks_api` has a paged route in both dialects.

### F5 — Authentication, sessions and guards

Mobile has `Session`, ACL and `guard`. The backend gets the server side of the same words:

```nuclear
// config/project_config.nuc
appConfig ProjectConfig
  connections: Connection[] = [SQLite(file = "app.sqlite")]
  auth: Auth = Jwt(secret = env("JWT_SECRET"), expires = Duration(hours = 12))

// api/tasks_controller.nuc
@controller("tasks")
controller TasksController(tasks: TasksAppComponent)
  @post
  @auth                                 // 401 without a valid token; `session` is the caller
  create(body: NewTask, session: Session) Result<AppError, Task>
    tasks.create(body, owner = session.userId)

  @delete("{id}")
  @auth(roles = ["admin"])              // 403 without the role
  remove(id: int) Result<AppError, void>
    tasks.remove(id)

// shared/guards/owner_guard.nuc
guard OwnerGuard(tasks: TaskRepository)
  canActivate(session: Session, id: int) boolean
    task = ~tasks.find(id)
    task.ownerId == session.userId
```

`Jwt` issues and checks tokens (`POST /auth/login` is the app's own route that calls `auth.issue(userId, roles)`);
`Session(userId, roles, permissions)` is the platform model. `fastapi`: a dependency on the router; `nestjs`: a
guard + `@UseGuards`. Done when: the example has login, an authenticated route and a role-guarded route, tested in
both dialects.

### F6 — Scheduled and background work

```nuclear
// components/tasks/jobs/reminders.nuc
job Reminders(tasks: TaskRepository, mail: MailService)
  @every(minutes = 15)
  remind()
    due = ~tasks.dueSoon()
    for t in due
      ~mail.send(t.ownerEmail, "Due soon: ${t.title}")

  @cron("0 8 * * MON")
  weekly()
    ...
```

In process (APScheduler / `@nestjs/schedule`); a `queue` plugin later (Redis-backed) for work that must survive a
restart. Done when: a job runs in the tests with a fake clock.

### F7 — File uploads and static files

```nuclear
@controller("tasks")
controller TasksController(tasks: TasksAppComponent)
  @post("{id}/attachment")
  attach(id: int, file: Upload) Result<AppError, Attachment>
    tasks.attach(id, file)             // Upload: name, contentType, size, bytes(), save(path)
```

`Upload` is a platform type (`UploadFile` / multer behind). Storage targets (`Storage` connection: local folder,
S3-compatible) as a plugin. Done when: a test uploads a file through the test client in both dialects.

### F8 — Deploy and operations

- `dragonfly build` writes a `Dockerfile` and a `.dockerignore` for the dialect (multi-stage, non-root user, the
  `PORT` convention), and `compose.yaml` with the databases the config names (dev only).
- Health route `GET /health` (database ping) and readiness, written by the dialect.
- Metrics (`GET /metrics`, Prometheus) and request logs with a request id: `Log` of the platform with the call site
  as in mobile (ADR-0078).
- Config from the environment only (`env("X", default)` exists); a `.env.example` written from the `env(...)` the
  project uses.

Done when: `docker build` + `docker run` of `tasks_api` answers `/health` in both dialects (a manual test on the
user's machine, recorded in STATUS "Untested" if docker is not on CI).

### F9 — The backend CLI at parity with mobile

- `dragonfly make controller|entity|model|usecase|repository|listener|job <name>` for backend projects (the
  generators live in `cli/scaffold.rs`; today only mobile kinds).
- `dragonfly check` and `dragonfly watch` for backend projects (today `manifest()` refuses them).
- `[toolchain]` for mobile too, so `cli/` has no hard-coded `flutter` (STATUS "Pending: fastapi").

Owned by the `cli/` session (rule 7): this plan only lists it; the request goes by message.

## 3. Work order

1. ADR for F1–F3 (data model: relations, migrations, transactions) — the one that constrains the rest.
2. F4 pagination + F1 relations (they share the query builder).
3. F5 auth (unblocks every real app).
4. F2 migrations.
5. F3 transactions, F6 jobs, F7 uploads.
6. F8 deploy, F9 CLI (parallel, by the `cli/` session).

## 4. Decided by the user (2026-11-10)

Reviewing the first draft of ADR-0100 (which deduced keys and relations and gave kinds by folder), the user chose:

- **One key decorator, `@primary`, that takes what it needs as arguments**: `@primary(generated = true)` (the
  database gives it), `@primary(length = 8)` (the app gives a code). `@generated` is gone, also on mobile.
- **Relations the TypeORM way** (the user's preference): `@manyToOne` (+ its column `<name>Id`), `@oneToMany`,
  `@manyToMany` + `@joinTable`, `@oneToOne` + `@joinColumn`; loading explicit with `relations(...)`.
- Pagination as drafted (`paginate(page, size)` → `Page<T>`).
- **The kinds stay as they are**: no kind by folder for repositories and controllers, no default paths.
- **Transactions in the first step** (`@transactional` on `exec`); migrations are of the database schema, next.

## 5. Decisions still pending for the user

- Auth first provider: JWT only, or also sessions with cookies for web clients (plan 005's web platform).
