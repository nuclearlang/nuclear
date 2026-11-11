---
adr: 0036
title: "Results"
status: "Accepted; order amended by ADR-0047"
scope: nuclear
---

# ADR-0036: Results

- Status: Accepted; order amended by ADR-0047
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`Result<E, T>` with `Ok(value)` / `Err(error)`, never declared. Inside a function that returns `Result`, errors propagate on their own (no `match`, `try` or `?`); outside, an unhandled `Result` is an error. `.ok(cb)` / `.error(cb)`: a callback returning `Ok`/`Err` transforms or recovers, otherwise it only observes. A `Result` is kept with a variable typed `Result`. An error of another type is a compile error (convert it with `.error(e => Err(...))`). `match` with `Ok(x) ->` / `Err(e) ->`. Each dialect generates its own runtime, without dependencies.

## Alternatives rejected

Exceptions; Rust-style `?`.
