---
adr: 0100
title: "Backend data: one key decorator, TypeORM relations, pagination, transactions, migrations"
status: "Accepted"
scope: dragonfly
---

# ADR-0100: Backend data: one key decorator, TypeORM relations, pagination, transactions, migrations

- Status: Accepted (2026-11-10, the user, after reviewing the first draft; §5 waits for the user's go)
- Scope: `dragonfly` (the backend platform; the key and `paginate` also on the mobile local tables), `nuclear` (two
  generic template variables, the generic `[decorators.fields]` rules, one diagnostic)
- Date: 2026-11-10 (asked by the user: step 4 of the competitiveness plan, `specs/004-backend-parity/plan.md`)

## Context

A real backend needs relations between tables, pagination, transactions and migrations (plan 004, F1–F4). The user's
rule for them (2026-11-10): **the code must look as much alike as possible across the platforms**, at the middle point
of cost and benefit, polished step by step. The first draft deduced the key from `id` and the relations from the field
types; the user chose instead (review of the draft, 2026-11-10):

1. **One key decorator, `@primary`, that takes what it needs as arguments** (no `@generated`, no key by name).
2. **Relations the TypeORM way**: a decorator per relation, as in TypeORM and sequelize, which the user prefers.
3. Pagination as drafted.
4. **The kinds stay as they are** (no kind by folder, no default paths: the first draft's §4 is dropped).
5. Transactions now, with the relations; migrations (of the database schema) after them.

## Decision

### 1. The key: `@primary(...)`

```nuclear
@entity("tasks")
entity Task()
  @primary(generated = true)     // the database gives it: auto increment (int) or a uuid (string)
  id: int

@entity("codes")
entity Code()
  @primary(length = 8)           // the app gives it
  code: string
```

`@primary` arguments: `generated` (the database or the runtime fills it: auto increment for an `int`, a uuid for a
`string`), `length` (a `string` key). `@generated` is removed on every platform (backend entities and the mobile local
tables of ADR-0096 alike): `@generated()` → `@primary(generated = true)`.

### 2. Relations: TypeORM's decorators

```nuclear
// data/entities/project.nuc
@entity("projects")
entity Project()
  @primary(generated = true)
  id: int
  name: string
  @oneToMany(inverse = "project")
  tasks?: Task[]

// data/entities/task.nuc
@entity("tasks")
entity Task()
  @primary(generated = true)
  id: int
  title: string
  @manyToOne(inverse = "tasks", onDelete = "cascade")
  project?: Project
  projectId: int                 // the join column of `project` (projectId?: int → a task may have no project)
  @manyToMany(inverse = "tasks")
  @joinTable()
  tags?: Tag[]
```

| Decorator | Arguments | fastapi (SQLAlchemy 2) | nestjs (sequelize-typescript) |
|---|---|---|---|
| `@manyToOne` on `x?: T` + a field `xId` | `inverse`, `onDelete` (`"cascade"`, `"set null"`, `"restrict"`) | `relationship` + `ForeignKey` on `x_id` | `@BelongsTo` + `@ForeignKey` on `xId` |
| `@oneToMany` on `xs?: T[]` | `inverse` (required: the `@manyToOne` field of `T`) | `relationship(back_populates=...)` | `@HasMany` |
| `@manyToMany` on `xs?: T[]` | `inverse`; `@joinTable()` on the side that owns the join table | `relationship(secondary=...)` | `@BelongsToMany(..., through)` |
| `@oneToOne` on `x?: T` | `inverse`, `onDelete`; `@joinColumn()` + a field `xId` on the side that keeps the column | `relationship(uselist=False)` | `@BelongsTo` / `@HasOne` |

The join table defaults to `<owner table>_<field>` (`tasks_tags`), with the columns `<entity>_id` of both sides. The
related type comes from the field's type, so it is not repeated in the decorator (TypeORM writes `() => Project`
because TypeScript erases types; Nuclear has them). The join column is a field of its own (`projectId`), as TypeORM
allows: it is set without loading the other row (`Task(title = t, projectId = id)`) and it is in the JSON answer.

**Rules, checked at compile time (N0308):** a relation field is optional (`?`: it is there only when loaded); its type
is an entity (a list for `@oneToMany`/`@manyToMany`, one for `@manyToOne`/`@oneToOne`); `@manyToOne` (and `@oneToOne`
with `@joinColumn`) needs the field `<name>Id`; `inverse` names a field of the related entity.

**Loading is explicit**, TypeORM's `relations`, one more step of the query builder; an unloaded relation is absent
from the JSON answer (never `null`, never a lazy query):

```nuclear
repository ProjectRepository
  withTasks(id: int) Result<NotFound, Project>
    where({ "id": id })
    relations("tasks")
    getOne()
```

Saving writes what the entity holds: the join column of a `@manyToOne`, and the rows of the join table when a
`@manyToMany` list is set (`task.tags = [a, b]` then `save(task)`), the same in both dialects.

The mechanism is generic (no decorator name in Rust): the dialect lists the rules of a field decorator in
`[decorators.fields]` (`optional`, `list`, `target` = a decorator the field's type must have, `sibling` = a field that
must exist, `inverse` = an argument that names a field of the type); templates get `tinfo.decor` (the decorators of the
field's type: its table) and `owner` / `owner_decor` (the declaration the field is in).

### 3. Pagination: `paginate(page, size)` → `Page<T>`

```nuclear
repository TaskRepository
  page(page: int, size: int = 20) Page<Task>
    orderBy("id")
    paginate(page, size)
```

`Page<T>` (`items: T[]`, `page` from 1, `size`, `total`, `pages`) is a class of the runtimes; `paginate` is one more
step of the query builder in `dragonfly-py`, `dragonfly-nest` and the mobile `LocalTable` (one `count` + one
`limit/offset`), so the line is the same in a backend repository and in a mobile local query.

### 4. Transactions: `@transactional`

```nuclear
usecase MoveTasks(tasks: TaskRepository, projects: ProjectRepository)
  @transactional
  exec(from: int, to: int) Result<Error, int>
    ~projects.find(to)
    moved = ~tasks.where({ "projectId": from }).update({ "projectId": to })
    Ok(moved)
```

Every repository call inside the method joins one transaction of the default database; an `Err` result or a thrown
error rolls it back, an `Ok` commits it. fastapi: an `AsyncSession` in a `ContextVar`; nestjs: a sequelize
transaction in an `AsyncLocalStorage`; the repositories of both runtimes read it, so nothing else changes.

### 5. Migrations of the database schema

`dragonfly migrate make "<message>"` writes one migration from the difference between the entities and **the last
migration** (never a live database), in the dialect's tool, under `migrations/` of the project (kept in the repository,
not in `build/`); `dragonfly migrate` applies the pending ones to the default database, `dragonfly migrate down`
reverts the last. They are the dialect's `[toolchain]` keys `migrate_make`, `migrate`, `migrate_down` (placeholders
`{{message}}`, `{{migrations}}`), so the CLI names no tool. `SQLite(createTables = true)` stays for tests and development.

- fastapi: Alembic. `make` brings an empty SQLite in memory to the last migration and lets autogenerate write the
  difference, in batch mode (SQLite cannot alter a column; the other databases run the same file);
  `dragonfly.migrate` writes the folder's `env.py` once; `alembic_version` lists what is applied.
- nestjs: Sequelize has no autogenerate, so `dragonfly-nest-migrate` (the runtime) reads the entities' attributes,
  diffs them with the `schema` the last migration exports, and writes `up`/`down` over `queryInterface`; applied
  migrations are listed in `SequelizeMeta` (sequelize-cli's table). umzug was not needed for that.
- A field default (`done: boolean = false`) is also the column's SQL default in both (Python's `column(default=...)`
  sets `server_default`), so adding a non-null column to a table with rows works.
- Renames show as a drop and an add; `make` warns of every drop; the file is the project's to edit.

### 6. Order

1. Now: §1, §2, §3, §4 in fastapi and nestjs (§1 and §3 also on the mobile local tables); `tasks_api` gets `Project`
   ↔ `Task`, a paged route and a transactional use case; `tasks_app` regenerates its contract.
2. Then: §5 — DONE 2026-11-10.

## Consequences

- An entity reads like a TypeORM entity in both backend dialects; the key is one word on backend and mobile.
- The rules of the relation decorators are data of the dialect, so a new backend dialect gets them by writing templates.
- The column name of a relation (`project_id`) and the join table name (`<owner table>_<field>`) are conventions, the
  same in both dialects; another name would be a later argument.
- Implementation (2026-11-10): the rules are `[decorators.fields]` of `dialects/backend/platform.toml`; templates read
  `tinfo.decor`, `owner`, `owner_decor` and the `unquote` filter; fastapi imports related entities only for the type
  checker (`[kinds.entity] type_imports`); nestjs types a single relation `Related<T>` (decorator metadata and ESM
  circular imports) and its runtime `Entity` keeps relations given to the constructor; in Dart `Page<T>` is the
  runtime's `PageOf<T>` (`[types] Page = "PageOf"`: Flutter has its own `Page`; a project's own `Page` keeps its name).

## Alternatives rejected

- **Relations and the key deduced from types and names** (the first draft): less to write, but the user prefers the
  explicit TypeORM form, and it needs no deduction rules to learn.
- **Kind by folder for repositories and controllers, paths from the name** (the first draft): the user keeps the kinds
  as they are.
- **Lazy or eager loading by default**: hidden queries and N+1, against the golden rule (ADR-0038).
- **Cursor pagination first**: offset pages are what most APIs and the mobile `Page<T>` use; cursors can come later.
