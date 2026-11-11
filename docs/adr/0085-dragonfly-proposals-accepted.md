---
adr: 0085
title: "Dragonfly proposals accepted"
status: "Accepted"
scope: dragonfly-flutter
---

# ADR-0085: Dragonfly proposals accepted

- Status: Accepted
- Scope: `dragonfly-flutter` (the parts about events, ACL and routes also shape the general `dragonfly`)
- Date: 2026-11-03

## Context

Eleven decisions Claude made while building Dragonfly were marked ⟨proposal⟩ in `STATUS.md`, waiting for the
user's review. The user asked for all of them to be implemented (2026-11-03). Six were already in the code;
five were not.

## Decision

Already in the code, now rules:

1. `GlobalEvents` uses `sessionStarted` / `sessionEnded` (and `onSessionEnded`).
2. `forward` is required in `acl`, as in `loggedIn` (`N0217`).
3. Translations use `.tr()`.
4. `@validate("field")` takes the field as text; a bare name is `N0106`.
5. A dialog has an Accept button only with `onAccept`.
6. Each state model generates its own `<Model>Builder<Manager>`, not a generic `StateBuilder<Manager>`.

Implemented now:

7. **Async guards.** `Guard.canActivate()` returns `FutureOr<bool>`; with `~` in its body it is `Future<bool>`.
   The router waits for it (its `redirect` stays sync when no guard is async), `AclBuilder` shows `fallback`
   until it answers, an async use case with `@acl` uses `Guards.verify`, and `Acl.check` is the version of
   `Acl.can` that waits. The sync `Acl.can`, `Acl.canOpen` and `Guards.check` throw a clear error with an async
   guard.
8. **Transitions per screen:** `@screen(transition = ScreenTransition.fade)`: `fade`, `slide`, `up`, `scale` or
   `none`; without it, the platform's. It is an enum, not the text `"fade"` the proposal wrote: the editor
   completes it and Dart checks it, and `Transition` would clash with bloc's class.
9. **Nested routes with their own back:** a route is nested in the route with the longest address that starts
   its own (never `/`). Going to it stacks the parent below, also from a link, so back returns to the parent.
   Inside a `shell` tab it stays in that tab. No new argument: the address already says it.
10. **Events from another engine** (workmanager): the `event` kind writes `fromJson`/`toJson`
    (json_serializable) and `eventName`. From a background isolate, an event with a name travels as its name
    and its JSON text, and `Events.decoder` (registered in `registerModels`) reads it back. So event fields are
    JSON types.
11. **Generated tests:** an enum field of a model gets its first value (`Plan.values.first.name`). A field with
    a converter of a type the tests cannot invent uses the converter's optional `sample() T`; without it, the
    field gets `null`.

## Consequences

- An `event` now has a `part` file and needs `build_runner`, like a model; an event field that is not a JSON
  type is a build_runner error.
- A screen whose route is nested is built from the address when a child is opened: its parameters must be in
  its route.
- `canActivate` of a hand-written `Guard` in Dart may return `bool` or `Future<bool>`.

## Alternatives rejected

- `transition = "fade"` (text): no completion and no check until run time.
- An explicit `parent` argument for nested routes: a second way to say what the address already says.
- Sending event objects between engines: Flutter engines share no objects; only JSON-like values travel.
- Making every guard async: it would force every view and use case to wait, also for the sync guards.
