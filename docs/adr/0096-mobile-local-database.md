---
adr: 0096
title: "Local database of the mobile platform: @primary models, @repository(saveLocal) and <method>AsLocal()"
status: "Accepted"
scope: dragonfly-mobile
---

# ADR-0096: Local database of the mobile platform: `@primary` models, `@repository(saveLocal)` and `<method>AsLocal()`

- Status: Accepted (amended by ADR-0100: `@generated` is now `@primary(generated = true)`; local queries have `paginate`)
- Scope: `dragonfly-mobile` (flutter dialect, `packages/dragonfly`), `nuclear` (two generic template variables)
- Date: 2026-11-05 (asked by the user)

## Context

The user asked (2026-11-05) for the mobile platform to have the ORM the backend platform has (ADR-0090, ADR-0091:
`entity` with `@primary`/`@generated`/`@created`/`@updated`, `repository` with CRUD and the query builder `where`,
`orderBy`, `limit`, `offset`, `getOne`, `getMany`, `count`, `exists`, `update`, `delete`), but **100 % local**: a
secure database on the device (hive, isar or another), automated through the `repository` kind. With
`@repository(saveLocal = true)` the transpiler must create the table from the model a GET answers (`Character`),
save every answer in it, demand only the minimum annotations on the model, and write for each method
(`getCharacter()`) a twin (`getCharacterAsLocal()`) that answers from the device instead of the network; a method
must also be able to turn it on by itself.

Constraints: the mobile `repository` is http calls, not tables, so the two kinds cannot be merged (ADR-0095 §2); the
Nuclear of a mobile project never names the storage engine (rule 11); the Dart package must work on web; the
compiler stays generic (ADR-0095 §4).

## Decision

1. **A model with `@primary` (or `@generated`) has a local table.** The same words as a backend entity: `@primary`
   on the key field, `@generated` for a key the table gives (an auto-increment: the field is `T?`), `@created` and
   `@updated` for stamps the table writes. `@entity("characters")` names the table as in the
   backend (default: the model's name in snake_case; the box is `dragonfly_<name>`). `@column` has no meaning in a
   document store (the JSON name, `@key`, is the column) and is not part of the mobile platform. The flutter dialect
   writes `static final LocalTable<M> table` in the model and registers it at startup with its component
   (`Local.register(M.table, component: 'characters')` in `registerDragonfly`).
2. **`@repository(saveLocal = true)`, or `@get(local = true)` on one method.** A GET that answers a model with a
   table (or a list of them; not `Page<Character>`, which has no key) passes `local: 'Repo.method'` to `send`: the
   decoded rows go to the model's table and a calls table remembers which rows that call (url + query) answered.
   The transpiler writes the twin **`<method>AsLocal()`**, same signature and return type, that reads those rows as
   they are now (`recall`) and never sends a request; without a previous answer it is `Err(E(message))` for a
   `Result` method and a thrown `NotFound` otherwise.
3. **The table is the ORM.** `local<Character>()` in a repository (and `Character.table` anywhere) is a
   `LocalTable`: `find`, `findAll`, `save`, `saveAll`, `remove`, `clear`, and the query builder of the backend
   (`where({ "age": gt(18) })` with `gt`, `gte`, `lt`, `lte`, `ne`, `like`, `ilike`, `oneOf`, `between`; a plain
   `null` is "is null" and `ne(null)` "is not null"; `orderBy(field, desc)`, `limit`, `offset`, `getOne`,
   `getMany`, `count`, `exists`, `update`, `delete`). A repository method with a body writes its queries on it, so
   a repository can be 100 % local (no http method at all).
4. **Engine: `hive_ce`, behind the runtime, one database for every platform.** Pure Dart: a file per box on
   Android, iOS, Linux, macOS and Windows, IndexedDB on web; no other engine per platform (asked by the user,
   2026-11-05). Rows as JSON text (the model's `toJson`/`fromJson`, nested objects intact); in memory and
   unencrypted in tests (`DragonflyTest.start`). **Encryption is a setting of the config files:** `local:
   LocalDatabase = LocalDatabase(encrypted = true)` in `appConfig` (the project) and in a component's `config` (the
   tables of its models); the default is encrypted (AES-256, key generated once and kept in the secure storage:
   Keychain, Keystore, libsecret, DPAPI, WebCrypto). Nothing generated names hive: the dialect writes `LocalTable`,
   `Local`, `LocalDatabase`, `send(local:)`, `recall`; the engine can change in the package without touching the
   transpiler or any `.nuc`.
5. **Two generic template variables, no kind name in Rust:** `ret_info.fields_decor.<decorator>` (the first field
   of the returned model with that decorator; here `primary`) and `field_decor` of each declaration in the project
   context; plus `{{#a or b}}` in templates (the first alternative that is true). Everything else is TOML, templates
   and the Dart package.

## Consequences

- Easier: offline reading with two words (`saveLocal = true`, `@primary`), the backend's query vocabulary on the
  device, a local-only repository without new kinds, generated tests that check each twin answers without a request.
- Harder / limits: a GET that answers a model without `@primary` under `saveLocal` fails at its first call with a
  `StateError` that names the fix (the generated tests catch it; no compile-time diagnostic yet); `local = false`
  on a method does not opt out of `saveLocal` (templates have no value comparison); `isNull` of the backend is not
  a mobile operator (the word is the `flutter_test` matcher already in the dialect's auto-imports, so a `[functions]`
  shortcut is refused by N0204); `hive_ce` keeps a box in memory, so a table is for the data a phone shows, not for
  millions of rows; `@created`/`@updated` are ISO-8601 texts; a backend entity's `@column` options (`length`,
  `unique`, `index`) have no equivalent in a document store; turning `encrypted` on or off after the first run needs
  the tables cleared (a box is read with the cipher it was written with); on web the key lives in the browser's
  storage, weaker than a phone's keystore.

## Alternatives rejected

- **Isar**: v3 is unmaintained against current Flutter, v4 unfinished. **Drift / sqflite / sqlite3**: real SQL and
  SQLCipher, but web needs a wasm bundle and a worker copied into every app, and drift adds its own code generation
  per table on top of json_serializable; the backend's query vocabulary is covered without SQL at mobile sizes.
  **Hive v2 (original)** is frozen; `hive_ce` is its maintained fork with the same API, pure Dart, web and encryption
  built in. **sembast**: queries built in, but no encryption without a third package.
- **A `entity` kind in mobile**: a second kind for the same model doubled every model (an http model and a table
  model); the user asked for the minimum annotations on the model the GET returns.
- **`<method>AsLocal()` reading the table by the method's parameters** (`getOne(id)` → `find(id)`): correct only when
  a parameter is the key; `search(name)` or `getPage(page)` would need a per-method mapping. Remembering which rows
  each call answered works for every signature and still returns fresh rows.
- **A plugin `local`** with the package added only when used: `hive_ce` is pure Dart and small; a plugin would add a
  line to `config.nuc` for a feature two annotations already declare.
