---
adr: 0086
title: "Operators, literals and the target's library"
status: "Accepted"
scope: nuclear
---

# ADR-0086: Operators, literals and the target's library

- Status: Accepted
- Scope: `nuclear`
- Date: 2026-11-03

## Context

The user asked that Nuclear "have nothing to envy any language" in math, text, regular expressions, reflection,
bytes and physics, without breaking its rules, with tests. A survey of the operations of the mainstream languages
(C, Java, Python, JavaScript, Dart, Go, Rust, Kotlin) against Nuclear found that the library side was already
there (Nuclear calls the target's library and imports it automatically) but the syntax lacked power, integer
division, bitwise operators, shifts, binary and scientific literals and raw text; and that several emitter rules
were wrong in one target (`x is string` → `instanceof` in TypeScript, `(5).toString()` losing its parentheses,
`print(a, b)` in Dart, `for k, v in` a `Map` in TypeScript, `key in map`, a `try` as the last line, top-level
constants in Dart).

## Decision

1. **No own standard library.** Math, text, regex, collections, dates, JSON, bytes and async are the target's
   (golden rule: no wrapper between the code and the platform; agents already know those APIs). Every area has a
   program per target in `examples/stdlib/` printing the same output, run by `tools/stdlib-check.sh`.
2. **Operators** (base grammar, the same for every dialect; each dialect writes them in `[operators]`):
   `a ** b` (right associative, above unary minus: `-2 ** 2` is -4), `a ~/ b` (integer division, Dart's token),
   `&`, `|`, `^`, `<<`, `>>`, `>>>`, and `^x` for the complement (`~` is await, as in Go). Precedence as in Python
   and Dart: comparisons < `|` < `^` < `&` < shifts < `+`. Compound `%= &= |= ^= <<= >>= >>>=`.
3. `>>` and `>>>` are `>` tokens written together, joined by the parser, so `List<List<int>>` keeps working.
4. **Literals:** `0b1010`, `1.5e-3`; **raw text** `r"\d+"` (no escapes nor interpolation), for regular expressions.
5. **Semantics that stay the target's**, documented: `%` (TypeScript keeps the dividend's sign, Dart is never
   negative), the integer width of bitwise operators (32 bits in TypeScript, 64 in Dart), rounding of halves.
6. Emitter: `[operators]` templates get the operands' numeric kind (`int`/`double`, for Dart's `pow` returning
   `num`) and bare operands (`left_arg`); `in` knows maps; `is` gets the type's info; `try` as the last line is a
   value; a Dart top-level variable used by a declaration stays top-level; statements plus a `main` → `N0113`.

## Consequences

- Formulas read as written (`0.5 * mass * speed ** 2`), and bit manipulation needs no library calls.
- The same `.nuc` is not portable between TypeScript and Dart when it calls library functions; the operators
  and the language are.
- `editors/` must teach the TextMate and tree-sitter grammars the new operators and `r"..."`.

## Alternatives rejected

- A portable Nuclear standard library (`Math.sqrt` written once, mapped per dialect): a second API to learn and
  maintain over two that agents already know; a possible later decision if portability is needed.
- `div`/`xor`/`shl` words (Kotlin): more reserved words; C symbols are what every agent knows.
- `~x` for the complement: `~` is await.
- Defining `%` the same in every target: either Dart or TypeScript output would stop being the expert's.
