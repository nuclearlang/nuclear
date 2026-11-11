---
adr: 0097
title: "The web platform: the mobile platform in a browser, first dialect react"
status: "Accepted"
scope: dragonfly-web
---

# ADR-0097: The web platform: the mobile platform in a browser, first dialect `react`

- Status: Accepted (first version; the simplifications of §"Analysis" wait for the user)
- Scope: `dragonfly-web` (`dialects/web/`, `packages/dragonfly-web`, `spec/{web,react}.md`), `nuclear` (two generic
  emitter rules)
- Date: 2026-11-06 (asked by the user: start the web platform with `react`, based on mobile; analyse whether React,
  zustand and other libraries let it be simpler; implement for now what flutter has)

## Context

ADR-0093 names three platforms; `web` had no dialect, so "three platforms" was a promise (STATUS.md "Next" step 5).
The user asked for the first web dialect to be built on what the mobile platform already does, so the knowledge of
one platform carries to the other, and for an analysis of a simpler design on top of React's own ecosystem.

## Decision

1. **The web platform is the mobile platform in a browser.** The same kinds with the same words where a browser
   has a use for them: `stateless`/`stateful` (params = props, `build() Element`), `appConfig`/`config`,
   `repository` with the http decorators, `usecase`/`appcomponent`/`service`/`aggregate`, state models and
   `statemanager` with `@stateModel`/`@state`/`@before`/`@after`, `@screen(route, providers, loggedIn, acl,
   notFound)`, `event` + `component` with `@on`, `guard`, `tests` with `@test`/`@states`/`@widget`/`@mock`. The
   contract is `dialects/web/platform.toml`; the rules `spec/web.md`. Left out of this version: forms, converters,
   translations, the local database, plugins and AI (they come with the mobile words).
2. **First dialect `react`** (`dialects/web/react/`, extends `typescript`): React 19 **without JSX** (Nuclear has no
   JSX; elements are the runtime's HTML functions `div(className = ..., children = [...])` and views called like
   functions), React Router 7, Vite 7, Vitest 4 + jsdom + Testing Library. A view is a class (`StatelessView` /
   `StatefulView`) shown through `createElement`, so hooks work inside `build` and a `stateful` keeps its fields
   between renders (`setState`, `initState`, `dispose`), the way a Flutter `State` does. A state model generates a
   `<Model>Builder` component; `@screen(providers)` wraps the screen in `Providers` and `Manager.method()` inside
   its views is `this.manager(Manager).method()` (no hook in a callback).
3. **Runtime `packages/dragonfly-web`** (npm `dragonfly-web`, ESM): `Result`, the injector, configuration and the
   `Http` connection, `Repository` over fetch (cache, retries, `result`/`attempt`), `StateManager`, the view bases
   and hooks, the HTML functions, events, session/ACL/guards, the router (`buildRouter`, `goTo`, `goBack`), `Log`,
   and `dragonfly-web/testing` (`DragonflyTest` with canned fetch, `mock`/`when`/`verify`, `renderScreen`,
   `statesOf`). Type definitions `defs/typescript/{react,react_router,vitest,testing_library_react,dragonfly-web}`.
4. **Two generic emitter rules, no kind name in Rust:** `[kinds.<k>.syntax] construct` (how a declaration of that
   kind is built when called: a view is a function, not `new`), and named arguments to a kind with `named_params`
   stay one object in a dialect whose named arguments are an object (TypeScript), instead of being reordered by
   position.
5. **Checked** by `tools/react-check/check.sh` (the runtime's tsc, the example built, tsc --strict, vitest, vite
   build, the runtime's vitest) and the golden `examples/expected/characters`.

## Analysis: can it be simpler on React's ecosystem? (for the user to decide)

What the first version copies from Flutter, and what React's libraries would replace, one by one:

| Mobile piece (copied) | React-native alternative | Gain | Cost |
|---|---|---|---|
| `statemodel` + `statemanager` + `<Model>Builder` (a Cubit) | **zustand** store: `create((set) => ({ characters: [], load: async (page) => set({ ... }) }))` and `useStore((s) => s.characters)` in the view | less generated code (no builder, no status enum), selectors re-render only what changed, devtools | the state is a bag of fields, not a list of named states: `loading`/`full`/`error` become flags the view checks; the `@states` test (ordered states) loses its meaning; Nuclear would need a `store` kind, a new word for both platforms or a web-only one |
| `repository` with `cache`/`retry`/`purge` | **TanStack Query**: `useQuery({ queryKey, queryFn })` gives cache, retries, refetch, loading/error state in the view | removes the manager for read-only screens (query state is the screen state), background refetch | ties data fetching to the view (the platform keeps http in `data/` and state in `presentation/`); a use case between them has no place; mobile has nothing equivalent |
| `form` + `formmanager` (not in v1) | **react-hook-form** + **zod** | the standard of the ecosystem; the validation rules (`@required`, `@email`...) map to a zod schema one to one | none: this is the recommended path for v2 (the Nuclear stays the mobile `form` words; the dialect writes a zod schema and a `useForm`) |
| HTML functions without JSX | JSX in `.tsx` | what every React developer reads | Nuclear would need JSX syntax in the parser (rule 2: the parser never changes per dialect) or a template string; the functions keep Nuclear's call syntax, the same the Flutter widgets use |
| `Providers` + `this.manager(X)` (context) | zustand stores are global: no provider | less wiring | a store per screen instance (a detail opened twice) needs a factory anyway |
| `goTo(Screen, params)` | react-router's `<Link>`/`useNavigate` with paths | idiomatic | navigation by address breaks when a route changes; the mobile platform navigates by screen on purpose (ADR-0055) |
| `tests` with `DragonflyTest` + `@mock` | MSW for http, vitest mocks | the ecosystem's tools | `DragonflyTest.respond` is MSW's role with no package; `@mock` already uses `vi.fn` |

Recommendation: keep the platform words (they are what makes the mobile and web code read the same and what the
agents already know) and let the **dialect** use the ecosystem underneath where it costs no word: zod +
react-hook-form for `form` (v2), zustand as the engine of `StateManager` (same Nuclear, selectors for free) if the
builder's re-render cost shows up, TanStack Query only as an opt-in for a `repository` method
(`@get(query = true)` → a `useX` hook) once a real app asks for background refetch. JSX stays out.

## Alternatives rejected

- **A web platform designed from React up** (hooks, stores, JSX as the kinds): the Nuclear of a web app would share
  nothing with a mobile one; the user asked for the opposite.
- **react-native as the first web dialect**: it is a mobile dialect (ADR-0093 lists it under mobile).
- **Vue or Angular first**: React has the largest ecosystem and the type definitions came from `@types/react`
  with the existing generator; the platform contract is ready for them.
- **A `.tsx` output with JSX**: needs JSX in Nuclear (parser change per dialect) or a second syntax for views.

## Consequences

- Easier: a web app from the mobile example in an afternoon (the `characters` example is the mobile one minus
  Flutter's widgets); one platform spec more; the `dragonfly` CLI gets web projects through `[toolchain]` (cli/
  session: `dragonfly new -p web -d react`, `build` through `nuclear build .`).
- Limits of this version: no forms, converters, translations, local database, plugins or AI kinds; `@acl(guards)`
  only in a use case that awaits something (`Guards.verify` is async); a view's named arguments must be named (a
  positional call to a view is written as positional); no generated tests file; `useManager`/`useStateOf` are hooks
  (only in `build`); not run in a real browser (Untested).
