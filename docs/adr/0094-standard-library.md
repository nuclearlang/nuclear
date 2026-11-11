---
adr: 0094
title: "Nuclear has its own standard library, translated by every dialect"
status: "Accepted"
scope: nuclear
---

# ADR-0094: Nuclear has its own standard library, translated by every dialect

- Status: Accepted
- Scope: `nuclear` (base spec, emitter, language dialects), every Dragonfly platform
- Date: 2026-11-04 (decided by the user)

## Context

ADR-0086 chose the target's library (`xs.push(x)` in TypeScript, `xs.add(x)` in Dart, `xs.append(x)` in Python)
and rejected a portable library as "a second API to learn". With three platforms whose Nuclear must not change
between dialects (ADR-0093), a `.nuc` that calls `xs.length` cannot be the same file that calls `len(xs)`. The user
chose Nuclear's own vocabulary over TypeScript's or Python's.

## Decision

- **Nuclear defines the members of its base types** (`string`, `int`, `double`, `boolean`, `T[]`, `<K, V>`,
  `Set<T>`, `Bytes`) and a small set of objects (`Math`, `Json`, `Regex`, `DateTime`, `Duration`, `delay`,
  `awaitAll`, `print`): `spec/library.md`. One name per operation, camelCase, properties for cheap queries
  (`length`, `isEmpty`, `first`, `keys`), methods for everything else; the most widespread name wins when the
  targets disagree (`contains`, `add`, `upper`, `trim`, `join`, `sorted`).
- **Every language dialect translates it** (`[library.<type>]` in `dialect.toml`, a template per member with the
  receiver and the arguments, including the lambda's parameters and body so Python can write a comprehension).
  The emitter rewrites a member of a base type when the receiver's type is known (declared, a literal, or the
  result of a library member or a declared function); when it is not known and the name belongs to more than
  one type with different translations, it reports the receiver needs a type.
- **A target's library name on a base type is an error** in every dialect (with the Nuclear name as the fix):
  `xs.push(x)` → "use add". Type definitions of libraries (ADR-0040) are untouched: `numpy`, `dio`, `Text(...)`
  are written as they are; only the base types are Nuclear's.
- **One `examples/stdlib/` set for every target**, checked by `tools/stdlib-check.sh` in TypeScript, Dart and
  Python with the same expected output.

## Consequences

- The language guide, the specs, the briefs and the examples that used `push`, `includes`, `toUpperCase`,
  `console.log`, `len`... are rewritten; `tools/spec-check` keeps them honest.
- The generated code stays the expert's: the templates write `len(xs)`, `[f(x) for x in xs]`, `xs.where(f).toList()`.
- Dart's `fpdart`-style `Result` and Flutter's widget API are not library members: they are type definitions.
- ADR-0086 stays for operators; its rejected alternative is now this decision.

## Alternatives rejected

- TypeScript's vocabulary as the standard (agents know it best): it would make the Python output read as a
  translation and tie the language to one target's naming.
- Python's vocabulary: the same, the other way; and `len(xs)` as a function reads worse than a property.
