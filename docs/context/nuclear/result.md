---
layer: L3
component: nuclear
read-when: you touch Result<E, T>, Ok/Err, propagation, .ok()/.error(), or function aliases
---

# Result and aliases

## Result<E, T> (ADR-0036, 0047, 0048; `spec/nuclear-base.md` §3.2)

Error first, like `Either`. No parser or AST changes (`Ok(x) ->` already parsed as a value pattern). All of it
lives in `emit.rs`, section "Result":

- `result_info` knows which calls give a Result (functions/methods of the file with that return, via
  `DeclInfo.sig`/`method_sigs`; variable types in `Frame.types`).
- `FnCtx` says whether the function returns Result (propagates) and whether its last line keeps it.
- `propagate` writes into `pending` the lines `const xResult = ...;` + `result_propagate`; `with_pending`
  puts them before each statement. `hoist_blocked` forbids propagating in lazy branches (N0110).
- `keep_next` keeps a Result (typed variable, return, match subject, Result parameter).
- `.ok/.error` → templates `result_then/tap/recover/tap_error` per `callback_returns_result`.
- Runtime: optional template `result_runtime` (`dialects/typescript/templates/result.tpl`), written after the
  imports if `uses_result`. Dart has none: fpdart (`[types] Result = "Either"`, `Right`/`Left` via
  `[names] written` + `[auto_imports]`; `result_value` = `(r as Right<E, T>).value` with `result_types`).
- A file that declares Result/Ok/Err sets `result_builtin = false`. Diagnostics N0108–N0111.

Limits: Result only propagates through functions/methods written in Nuclear with a Result return (not
external libraries nor methods of external bases); `~f().error(...)` on an async function does not chain
(store it first in a variable typed `Result`).

## Aliases (ADR-0037, base §4.1.2)

`f() = Module.method` with empty `()`, no return type, and `Module` of the file with that method.
`alias::expand` (called in `lib.rs::transpile`, before the emitter) rewrites it in the AST as
`f(params of method) ret = Module().method(params)` (`~` if async, `yield ...` if generator, `Module.method` if
`static`; `{}` params passed by name). So `collect`, Result and async know nothing about aliases. A
constructor with required params → N0112.
