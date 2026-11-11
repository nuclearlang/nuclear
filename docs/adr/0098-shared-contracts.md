---
adr: 0098
title: "Shared contracts: a backend publishes what it exposes, a client generates its models and repositories"
status: "Accepted"
scope: dragonfly
---

# ADR-0098: Shared contracts: a backend publishes what it exposes, a client generates its models and repositories

- Status: Accepted
- Scope: `dragonfly` (every platform), `nuclear` (`nuclear_transpile::contract`, `nuclear contracts`), the runtimes
  `dragonfly-py` and `dragonfly-nest`, `editors` (the contracts and HTTP views)
- Date: 2026-11-06 (asked by the user; the plan is `specs/003-shared-contracts/plan.md`)

## Context

A backend written in Nuclear defines its models and routes once; every app that calls it wrote the same models and
a repository per controller again, by hand, with nothing checking the copies. A renamed field or a changed path
compiled on both sides and failed at run time; the validation rules stayed in the backend; an agent spent its
turns transcribing. Backend and app stay two projects, deployed apart and talking over HTTP: what was missing is
the **description** of the API traveling from one to the other, the way OpenAPI, `.proto` files or tRPC do it.

## Decision

1. **Every project writes its contract.** `nuclear build`, `nuclear check`, `dragonfly build` and the editors (the
   same call that writes the project card, `project::write_metadata`) write `.nuclear/contract.json`: the models
   (fields, Nuclear types, optionality, defaults, JSON names, validation rules, the key field, doc comments) and,
   for a backend, the controllers with their routes (verb, path, parameters with where they go, body, answer,
   error type, status); for a mobile or web app, the repositories with the calls they make. The contract is in
   Nuclear's notation and is **the same whatever the dialect** (checked by a test with `nestjs` and `fastapi`).
2. **A backend also writes OpenAPI 3.1** (`.nuclear/openapi.json`, from the contract: paths, parameters, request
   bodies, responses, schemas with the rules, doc comments as summaries). Both files are copied into the output
   folder, and the runtimes serve them at `GET /.nuclear/contract.json` and `GET /.nuclear/openapi.json`, so a
   client that is not Nuclear (Swagger UI, a generator, an HTTP tool) reads the API as built.
3. **A client declares the backends it talks to** in its manifest: `contracts(tasks_api = "../tasks_api")` (a path:
   the backend built next to it, monorepo or a sibling checkout) or `contracts(tasks_api = "https://api.x.com")`
   (the running API: fetched by `nuclear contracts update`, kept in `contracts/<name>/contract.json`, committed so a
   build never needs the network). The name is the `connectionName` the generated repositories use.
4. **The build generates Nuclear, not target code.** Before reading the sources, `project::build` writes
   `contracts/<name>/`: one model per model the routes carry (transitively), the error model `<Name>Error`
   (`message`), one repository per controller with a method per route (`{id}` → `:id`, path/query/body placement,
   `Result<<Name>Error, T>`), with the backend's doc comments as comments. Only the decorators the client's dialect
   consumes are written (`@primary` and the rules for flutter, `@key` everywhere; a GET that answers a keyed model
   gets `local = true` where the local database exists). The files carry a "generated, do not edit" header and are
   regenerated on every build from a path contract; stale files are removed. The rest of the pipeline is untouched:
   the client's dialect writes the model and the repository as it writes its own.
5. **Drift is a build error.** A field the backend renamed is unknown in the client at its next build (`N0116`
   with the field's name); a contract that cannot be read is `N0307`. `nuclear contracts list` shows each contract's
   source and the version of the local copy.
6. **Two generic compiler fixes came with it** (no kind name in Rust): named arguments to a method of a value typed
   with a project declaration (`repo.all(done = false)`) are reordered by position like any Nuclear call (they were
   written as named, which Dart and TypeScript refuse); and a project declaration named like a library export
   (`Task` next to fpdart's) hides it from that import (`[exports]` of a dialect for libraries without type
   definitions; the `hide` clause of `import_default`).

## Consequences

- Easier: a mobile or web app over a Nuclear backend writes no model and no repository; the rules travel; a change
  in the API is caught at build time; the editors read every project's endpoints from the same file (ADR-0099).
- Limits: entities travel only when a route returns them; generics in a route's answer (`Page<Task>`) are
  generated but not stored locally (`local = true` only for a plain keyed model); the client's `<Name>Error` has the
  message only (the status is not in the mobile `Result` error today); a url contract needs `curl`; the generated
  repository names (`TasksRepository`) may clash with the app's own (a prefix is a one-line change in
  `contract::generate`); two contracts that export the same model name clash (each is a top-level declaration);
  versions are shown, not enforced (a stale copy is not a warning yet, plan 003 F5).

## Alternatives rejected

- **OpenAPI as the only contract**: it loses `Result` errors, Nuclear types, the key field and the rules' names;
  kept as an export for non-Nuclear clients (decision 2).
- **Target code generated directly in the client**: a second code path per dialect; generating Nuclear reuses the
  dialect's own templates and shows the user plain `.nuc` files.
- **A shared Nuclear library imported by both projects**: the backend would depend on the client's platform.
