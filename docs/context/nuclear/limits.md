---
layer: L3
component: nuclear
read-when: before promising a feature, or when something "should work" and does not
---

# Known limits and what is planned

## Not expressible in Nuclear yet

- Dart 3 pattern `switch`; declaring named or `factory` constructors (`Character.fromJson`; calling them
  works); operator overloading (`Offset.zero & size`); `const` inference in Dart.
- Patterns only in assignments (not in lambda parameters, `for` or `match` branches).
- Records: decided they do NOT exist (ADR-0031).

## Behaviour limits

- External classes not in the type definitions (third-party packages like provider) leave the module open:
  no `@override`, no N0106. Fix: generate the package with `tools/defs` and list it in `defs`.
- Result only propagates through Nuclear functions with a Result return (see [result.md](result.md)).
- TypeScript/Node have no type definition packages yet.
- An enum field of a model gets `null` in generated tests, and so does a field with a converter whose type is
  neither basic nor `DateTime`.
- The integration test needs a device (here `-d linux`, with CMake).

## Planned (rough order)

`nuclear fmt` · components per principle 6 of the base (`users.getProfile()` without an explicit facade) ·
MCP server · `react` dialect · `const` inference in Dart. Node.js/Express is postponed by the user (including
`use express as "express"` of base §8, which does not parse): do not touch it until asked.
