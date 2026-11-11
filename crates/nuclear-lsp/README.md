# nuclear-lsp — the language server of NuclearLang

## Supported dialects

`flutter` (default) and `base` (only the base language, no dialect). The server loads any dialect it finds, but the
packages only include `dart` and `flutter`.

## What the server does

| Feature | Detail |
|---|---|
| Diagnostics | The same as `nuclear check` (codes N00xx/N01xx/N02xx) while you type. |
| Quick fixes | Each diagnostic with an exact fix (`&&` → `and`) offers a *quick fix*. |
| Semantic highlighting | Tells apart dialect kinds (`stateless`), modifiers (`get`), modules, enums, functions, fields, parameters, named arguments, Flutter widgets (`defaultLibrary`), deprecated names struck through (`deprecated`)... |
| Hover | Signature and comment of each declaration; for an external class, function, constructor or member (Flutter, Dart, bloc, dio...), its API in Nuclear notation, its documentation, the package (`flutter@3.47.7`) and the link to api.flutter.dev; for a kind, its description; for reserved words, their use. |
| Completion | Visible variables (block scope), members after `Enum.`/`Module.`, statics, named constructors and enum values of external classes (`Colors.`, `Text.rich`, `TextAlign.center`), members of a value of a known type (`dio: Dio`, `dio = Dio()`, `state` inherited from `Cubit<int>`), named arguments of the file's and external widgets (`Text('a', style = …)`, required ones first), methods of the external base to override (`initState`, `build`...), kinds and templates at the start of a line, and the public declarations of the type definitions filtered by prefix. |
| Signature help | Parameters of the file's functions and widgets and of external constructors, functions and methods (`Scaffold({key?: Key, appBar?: ...})`, positional ones first); highlights the current argument. |
| Navigation | Go to definition, find references, highlight uses, rename (it does not allow switching PascalCase ↔ camelCase). |
| Structure | Outline (modules, kinds, fields, methods) and folding by indentation. |
| Projects | A file inside a folder with `nuclear.toml` or `main.nuc` (Dragonfly) sees the names of the other files (completion, hover, go to definition), takes the kind of its folder (`domain/models` → `model`) and, saved, the project's diagnostics (N0301/N0302/N0303). What is in `test/` is only seen by `test/`. |
| Decorators | Completion after `@` (the dialect's: `@screen`, `@on`, `@test`...) and, inside `@screen(...)`, its arguments (`route = `) and the fields of an object argument (`loggedIn = { forward: `) from `[decorators.args]`, required ones first; hover with the list of arguments. Guide: `nuclear help tutorial` and `spec/flutter.md` §6.9–6.13. |
| Generated code | The `nuclear/transpile` request: VS Code shows it with **Nuclear: Show Generated Code**. |
| Code metrics | Complexity (an approximation of the cyclomatic one) on each function, method, module...: see below. |

Dialects reload without restarting when a `.toml` changes (VS Code watches `**/dialects/**`).

Where it looks for dialects, in order: the `nuclear.dialectPaths` setting, `$NUCLEAR_DIALECT_PATH`,
`<workspace>/dialects`, `dialects/` next to the binary (so it finds them in the packages) and the repository's
`dialects/`. Type definitions (`../../spec/definitions.md`) are looked for in the `defs/` folder next to each dialects
folder; the full `api.json` is read once per loaded dialect.

## Code metrics

A copy of the **CodeMetrics** extension (kisstkondoros, MIT), with the same settings, costs and messages. Above
each function, method, constructor, lambda, getter/setter, enum and module a *code lens* appears:

```
Complexity is 3 Everything is cool!
label(n: int) string
```

| Complexity | Message (setting) |
|---|---|
| ≤ 5 (`ComplexityLevelNormal`) | `Everything is cool!` (`ComplexityLevelLowDescription`) |
| 6–10 (`ComplexityLevelHigh`) | `It's time to do something...` |
| 11–25 (`ComplexityLevelExtreme`) | `You must be kidding` |
| > 25 | `Bloody hell...` |

It shows only from 3 (`CodeLensHiddenUnder`). A click opens the list of what adds up, in the format
`1 - Ln 2 Col 3 If with else statement` with the code on the right; walking it, the cursor jumps to each entry. A
square of the level's color is also drawn at the end of the line, with a mark in the right side bar.

How it is computed (like `tsmetrics-core`): each node adds a configurable cost. A function, method, constructor or
lambda adds its own cost plus what is inside; a module shows the **maximum** of its members. Default costs (the
names are the original's TypeScript nodes):

| Nuclear | Node | Cost |
|---|---|---|
| function, method, constructor, lambda | `FunctionDeclaration`, `MethodDeclaration`, `Constructor`, `ArrowFunction` | 1 |
| enum | `EnumDeclaration` | 1 |
| `if` / `else if` (with another branch after it) | `IfWithElseStatement` | 1 |
| `if` / `else if` (last branch), `if` inside a list | `IfStatement` | 1 |
| `match` / each branch / the `_` branch | `SwitchStatement` / `CaseClause` / `DefaultClause` | 1 / 1 / 1 |
| `for`, `while`, `try`, `catch`, `throw`, `return`, `break`, `continue` | `ForStatement`... | 1 |
| `and`, `or` | `BinaryExpression` | 1 |
| `cond ? a : b` | `ConditionalExpression` | 1 |
| module, getter, setter, fields, calls, literals... | `ClassDeclaration`... | 0 |

Settings: in VS Code, `nuclear.codemetrics.basics.*` (thresholds, messages, template, colors, which declarations
get a *code lens*, `Exclude`, `IgnoredFunctionNames`, `DiagnosticsEnabled`...) and
`nuclear.codemetrics.nodeconfiguration.*` (cost and description of each node). In Android Studio: *Settings →
Tools → NuclearLang → Code Metrics*. Commands: **Toggle code metrics** and **Toggle code lenses for arrow
functions** (VS Code's palette; *Tools → NuclearLang* in JetBrains).

The server computes it (`lsp/src/metrics.rs`), so both editors show the same. `nuclear-lsp --codemetrics-defaults`
prints every setting with its default value; `node scripts/codemetrics-settings.mjs` uses it to generate the
settings of `package.json` and `jetbrains/resources/codemetrics-defaults.json` (run by `npm run package` and
`build.sh`).
