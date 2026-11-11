---
layer: L4
component: nuclear
read-when: you start step 6 of the competitiveness plan (STATUS.md "Next"): the language gaps and the 1.0 freeze
---

# 005 — The language to 1.0: formatter, definitions, constructors, patterns, freeze

Step 6 of the competitiveness plan (STATUS.md "Next", decided by the user 2026-11-06). Status: F1 (`nuclear fmt`)
done 2026-11-10 (only reflow, decided by the user; the editor's format on save is the `editors/` session's); F2–F6
not started. Each feature changes `crates/` and `spec/nuclear-base.md`; the dialect side is small.

## 1. Diagnosis

`docs/context/nuclear/limits.md` lists what Nuclear cannot say yet; the agent evaluation and the external
projects show which of those an agent hits first: a named constructor (`Character.fromJson` can be called, not
declared), a pattern in a `for` or a `match` branch, a value of a library class whose members are unknown
(`text.split(",")` is "unknown" until typed, so no `N0106` and no completion). There is no formatter, so two
agents write the same code two ways and diffs are noise. TypeScript has no type definitions, so the NestJS and
future web dialects check less than Flutter. And nothing says what is stable: specs change daily.

## 2. Features

### F1 — `nuclear fmt`

One canonical layout, the one the specs already use: 2 spaces, one blank line between declarations, members
without blank lines unless the author left one, (trailing commas in multi-line calls: left as written, only reflow), decorators on their own
line, doc comments kept. `nuclear fmt <file|dir>` rewrites; `nuclear fmt --check` for CI and the editor (format on
save through the LSP).

```nuclear
// before
usecase GetUser(repo:UserRepository)
  exec(id:int)   Result<UserError,User>
    ~repo.find( id )
// after
usecase GetUser(repo: UserRepository)
  exec(id: int) Result<UserError, User>
    ~repo.find(id)
```

Built on the parser's AST plus the comments and blank lines the lexer already keeps (`DocComment`, the `*`
comment lines of ADR-0080). Done when: every `.nuc` of the repository is a fixed point of `fmt` and a test
guards it.

### F2 — TypeScript type definitions for the base dialect, and member types everywhere

`defs/typescript/typescript@5.9.3` exists for the nestjs dialect; the `typescript` and `nodejs` dialects get
`defs = ["typescript@5.9.3", "node@26"]` so `.nuc` files targeted at TypeScript get `N0219`/`N0220` (unknown
member, wrong arguments) like Dart.

Member types: today `index.json` lists members by name only, so a value returned by a library method is
"unknown". The generators (`tools/defs/dart`, `tools/defs/typescript`, `tools/defs/python`) write the return type
of every member in Nuclear notation (`api.json` already has them in TypeScript; Dart's generator loses them), and
the emitter's `static_type` follows a chain:

```nuclear
parts = "a,b".split(",")        // list<string>: known, so
first = parts.first              // string → `first` is typed, N0106 and completion work
```

Done when: `examples/stdlib` programs type every intermediate value (a test prints the inferred types), and the
LSP's open request (c) "inferred type of an expression" is served by the same function.

### F3 — Named and factory constructors

```nuclear
Point
  x: double
  y: double

  // a named constructor: Point.origin()
  new origin() this(x = 0, y = 0)

  // a factory: computes before building
  new fromJson(json: <string, any>) Point
    Point(x = json["x"] as double, y = json["y"] as double)
```

| Dart | TypeScript | Python |
|---|---|---|
| `Point.origin() : this(x: 0, y: 0);` | `static origin(): Point { return new Point(0, 0); }` | `@classmethod def origin(cls) -> "Point": return cls(x=0, y=0)` |
| `factory Point.fromJson(...)` | `static fromJson(json): Point` | `@classmethod def from_json(cls, json)` |

`new name(params)` with `this(...)` as the body is a redirecting constructor; with a return type it is a factory.
Done when: the flutter dialect's `model` keeps generating `fromJson` and a hand-written `new` on a model is
allowed next to it (`N0xxx` when they clash).

### F4 — Patterns everywhere

Today patterns exist only in assignments (`(a, b) = pair`). They become legal in `for`, in `match` branches and
in lambda parameters:

```nuclear
for (name, score) in scores            // a list of pairs, or map entries
  print("${name}: ${score}")

match point
  Point(x = 0, y = 0) -> "origin"
  Point(x = 0) -> "on y"
  _ -> "elsewhere"

users.map(({ name, age }) => "${name} (${age})")
```

Dart 3 patterns, TypeScript destructuring, Python 3.10 `match`. Done when: `examples/stdlib` has a program per
form that prints the same in the three languages.

### F5 — `const` inference in Dart and a `switch` on patterns

Dart-only output quality: a widget tree whose arguments are all constants gets `const` (today only literals do),
so `flutter analyze` with `prefer_const_constructors` stays at 0 infos. `match` over a sealed value writes a
pattern `switch` (today `if` chains with casts, the `unnecessary_cast` warnings in STATUS).

### F6 — Freeze 1.0

- `spec/nuclear-base.md`, `spec/dialects.md`, `spec/library.md`, `spec/projects.md`, `spec/definitions.md` and
  `spec/diagnostics.md` get a version line and a changelog file (`CHANGELOG.md`, keep-a-changelog).
- Semantic versioning: a change that makes a valid `.nuc` invalid or changes its meaning is a major; a new
  construct is a minor; a diagnostic text or a generated-code improvement is a patch.
- A compatibility test: every `.nuc` of `examples/` and the external projects builds with 1.0 and keeps building
  with every 1.x.
- The platforms (`backend.md`, `mobile.md`, `web.md`) version separately from the language.

Done when: `nuclear --version` prints `1.0.0`, the website has a "What is stable" page, and the first external
project builds from the published release.

## 3. Work order

1. F1 `fmt` (independent, high value for agents and reviews).
2. F2 definitions (unblocks the web dialect's checks and the LSP).
3. F3 constructors, F4 patterns, F5 Dart quality.
4. F6 freeze, after steps 1–5 of the competitiveness plan ship.

## 4. Decisions pending for the user

- The constructor word: `new origin()` (above) or `Point.origin()` as a method header (reads like the call, but
  `Name.` in a header is new syntax). Recommended: `new`.
