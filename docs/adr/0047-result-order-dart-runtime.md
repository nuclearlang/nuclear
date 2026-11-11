---
adr: 0047
title: "`Result` order and Dart runtime"
status: "Accepted"
scope: nuclear
---

# ADR-0047: `Result` order and Dart runtime

- Status: Accepted
- Scope: `nuclear` (the repository it belongs to after the split, ADR-0083)
- Date: taken before 2026-11-03; migrated from `spec/decisions.md` on 2026-11-03

## Decision

`Result<E, T>`: the error type goes first, like `Either<L, R>` in fpdart and Arrow. Names (`Result`, `Ok`, `Err`), automatic propagation and `.ok()`/`.error()` do not change. In Dart, `Result<E, T>` is fpdart's `Either<E, T>` (`Ok` → `Right`, `Err` → `Left`); async is `Future<Either<E, T>>` with `~`.

## Alternatives rejected

`Result<T, E>` (the first order, ADR-0036).
