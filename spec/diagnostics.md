# Diagnostics

Every error has a stable code. `nuclear check --json` returns them with their position and, when possible,
the exact fix (`fix`), so an agent can correct the code without reading documentation (see nuclear-base.md §9).
The messages the developer sees are in English.

Ranges: `N00xx` lexing and syntax · `N01xx` resolver (semantics) · `N02xx` dialect · `N03xx` project and architecture.

## Lexer

| Code | Error | Example | Fix |
|---|---|---|---|
| N0002 | Tab in the indentation | `→x = 1` | 2 spaces per level |
| N0003 | *Retired:* single quotes are valid (ADR-0009) | — | — |
| N0004 | The indentation returns to no open block | a line at 2 when the blocks are at 0 and 4 | — |
| N0005 | Indentation that is not a multiple of 2 | 3 spaces | — |
| N0006 | Unexpected character | `#`, `&` | — |
| N0007 | Unclosed string | `"abc` | close it on the same line, or use `"""` |
| N0008 | Unmatched parenthesis, bracket or brace | `foo(1, 2` | — |
| N0009 | Syntax of another language | `&&` `\|\|` `!` `===` `!==` `;` | `and` `or` `not` `==` `!=` (and no `;`) |
| N0010 | Invalid interpolation | `"${}"`, `"${a"` | `"${expr}"` |
| N0011 | Invalid escape sequence | `"\q"` | `\n \t \r \0 \\ \" \' \$ \u{hex}` |
| N0012 | Invalid number | `10px`, `0x` | `42`, `1_000`, `3.14`, `0xff` |

## Parser

| Code | Error | Example | Fix |
|---|---|---|---|
| N0001 | Reserved word used as a name | `if = 3` | another name |
| N0013 | Unexpected token | `x = = 1` | the message says what was expected |
| N0014 | Range followed by a method without parentheses | `1..10.map(...)` | `(1..10).map(...)` |
| N0015 | Typed lambda parameter without a block | `users.filter(user: User)` without a block | an indented block, or `users.filter((user: User) => ...)` |
| N0016 | Function declared inside a body | `helper(x: int) int` + block inside a function | `helper = (x: int) int` + block |
| N0017 | Statement in the body of a module | `localAttr = "x"` inside `User` | field `localAttr: string = "x"` or assign it in the constructor |
| N0018 | Module declared inside another module | `Inner` inside `User` | declare it at the top level |
| N0019 | Invalid assignment target | `f() = 1`; `pieces = ~pieces(raw)` inside a module with a method `pieces` (a method is not a variable) | a name, `$field`, `a.b` or `a[i]`; another name for the variable |
| N0020 | One-line function returning void | `log(m: string) void = print(m)` | an indented block |
| N0021 | Comment line without ` * ` | `// title` + a more indented `  text` | ` * text`, with the `*` below the second `/` (the fix writes it) |
| N0022 | Habit of another language (TypeScript, Dart, Python, Java, Go); the other errors of that line are not shown | `f() int {`, `await g()`, `const x = 1`, `new User()`, `class A extends B`, `int f(int x)`, `def f(self):`, `elif`, `None`, `# note`, `i++`, `import { a } from "x"` | the NuclearLang form; the fix writes it when it is exact (`f() int`, `~g()`, `x = 1`, `User()`, `A: B`, `f(x: int) int`, `else if`, `null`, `// note`, `i += 1`, `use x{ a }`) |

## Resolver

| Code | Error | Example |
|---|---|---|
| N0102 | Duplicate declaration | two `User` in the same file; a member declared twice in one module (`load()` twice); a call to a method hidden by a local of the same name (`prompt(...)` inside `write(prompt: string)` of a module with a method `prompt`) |
| N0103 | `if`/`match` with multi-line branches inside an expression | `f(if a` + a two-line block `)` |
| N0104 | Invalid enum item | an enum that mixes `user` with `name: string` |
| N0105 | Name used outside the block that created it | `text` created inside an `if` and read after it |
| N0106 | Undefined name | `page` not declared, not a parameter nor an import |
| N0107 | Pattern mixing new and existing names | `[a, b] = xs` with `a` already declared and `b` new |
| N0108 | Unhandled `Result` outside a function that returns `Result` | `user = findUser(id)` in a function that returns `string` |
| N0109 | The propagated error has another type | `findUser` fails with `string` and the function returns `Result<AppError, Order>` (fix: `findUser(id).error(e => Err(AppError(e)))`) |
| N0110 | A `Result` that cannot be propagated from that position | `ok and findUser(id).active` (fix: assign it to a variable first) |
| N0111 | Invalid use of a `Result` | a branch `Ok(1) ->` in a `match`; an `.error()` callback that returns `Ok(...)` only in some branches; `match load(id)` with an async `load` (missing `~`) |
| N0112 | Alias of a method that cannot be reached | `find() = Repo.find` with `Repo(url: string)`: the alias creates a new `Repo()` and `url` has no default (fix: give it a default, or write `find` as a method) |
| N0113 | Top-level statements in a file with `main` (targets that run them in `main`, like Dart) | `print("hi")` at the top level and `main() void` in the same file (fix: move the statement into `main`; a top-level variable `limit = 10` is allowed) |
| N0114 | Not a member of Nuclear's standard library ([library.md](library.md)): a target's name on a base type, a function of a target (`len`, `str`, `parseInt`), a static of a target (`Math.floor`, `JSON.stringify`), a property called as a method or a method used as a property | `xs.push(x)` (fix: `add(x)`), `text.toUpperCase()` (fix: `upper()`), `len(xs)` (fix: `xs.length`), `xs.length()` (fix: `length` is a property); a kept `Result` read as its value (`found.length`, `found[0]` in a test: `found.value.length`) |
| N0116 | A member that a declaration of the project does not have: the value's type is a model, class or kind whose members are all known (its own, its project bases' and those of the runtime classes it or its kind inherits when the type definitions list them; the kind's `method_members`), and the message names them as the `.nuc` writes them | `saved.done` when `Task` has `completed` (a contract the backend changed); `~loans.get(id)` on a repository (the runtime's is `find`); the message names the closest member and all of them | rebuild (a contract) or fix the name |
| N0115 | The receiver's type is not known and the member is written differently for each base type in this dialect | `items.length` where `items` comes from a value without a type (fix: declare it, `items: string[] = ...`); only in the dialects where the bases differ (TypeScript's `length` against `size`) |

## Dialects

| Code | Error | Example |
|---|---|---|
| N0201 | Header word unknown to the dialect | `stateless MyApp` with the `typescript` dialect |
| N0202 | Clause unknown to the dialect | `Bird: Animal with Flyer` with the `typescript` dialect |
| N0203 | The dialect does not define a template | `[syntax] while` is missing |
| N0204 | Shortcut collision | `readFile` defined in two modules of `[auto_imports]` |
| N0205 | Shortcut very similar to another (warning) | `material` and `materal` |
| N0206 | Invalid dialect file | malformed TOML, a value of the wrong type, a template with an unclosed `{{#x}}` |
| N0207 | Dialect not found or circular inheritance | `--dialect rubi`, `a extends b` and `b extends a` |
| N0208 | The target allows only one base | `Admin: User, Person` in TypeScript or Dart |
| N0209 | The kind requires a member | `stateless MyApp` without `build` |
| N0210 | Unknown variable in a template (warning) | `{{nmae}}` in `[syntax] class` |
| N0211 | Unknown `use` clause | `use lib show foo` in TypeScript |
| N0212 | Shortcut named like a reserved word | `[imports] if = "..."` |
| N0213 | Type definitions package not found or invalid | `defs = ["flutter@9.9.9"]` without that version in the `defs/` repository |
| N0214 | Two versions of the same definitions package | `dart` asks for `dart@3.13.5` and a child dialect for `dart@3.12.0` |
| N0215 | Method not allowed in the kind | a second method in a `usecase` (only `exec` is allowed) |
| N0216 | The kind requires another return type | `exec() Character` in a `usecase` (it must return `Result<E, T>`) |
| N0221 | A method of the kind needs a body | `booksOf(author: string) Result<Error, Book[]>` alone in an `appcomponent` (it would be generated empty) | an alias `booksOf() = BooksOfAuthor.exec`, or a body |
| N0222 | An asynchronous call used without `~` | `expect(booksOfAuthor.exec("Ada").length > 0)` (the length of a promise); `found = books.all()` then `found.length` (the variable holds the call) | `(~booksOfAuthor.exec("Ada")).length`; `found = ~books.all()` |
| N0217 | Unknown or missing decorator argument (`[decorators.args]`) | `@screen(loggedIn = { })` (missing `forward`), `@screen(rute = "/")` |
| N0218 | The kind does not write the declaration's generics (`<T>`) or interfaces (`> I`): they would be lost | `Page<T>` with a kind whose template does not use `generics`; `@stateModel(S) statemanager M > Comparable` |
| N0219 | Named argument the constructor does not have (from the type definitions; files of a project) | `Text("a", colour = c)` | the closest parameter (`style`); the message lists all of them |
| N0220 | PascalCase call that nothing defines (files of a project, dialects with type definitions); also a call's type argument (`inject<Missing>()`), not a generic parameter | `Collumn(children = [])`; `inject<BooksAppComponent>()` with no such class | the closest known name (`Column`); `nuclear defs search` |

## Projects and architecture

| Code | Meaning | Example |
|---|---|---|
| N0301 | A declaration uses another one its kind does not allow (`[architecture] callers`), or a name its kind cannot use (`[architecture] names`) | a screen calls a `usecase` (only an `appcomponent` calls it); a `statemanager` receives a `BuildContext` |
| N0302 | A declaration uses something private of another component (`[architecture] public`) | `episodes` uses `CharacterRepository` of `characters` instead of `CharactersComponent` |
| N0303 | Invalid project | `nuclear.toml` is missing, two files declare the same name, or the manifest in Nuclear (`config.nuc`) has another section, an unnamed argument or a value that is not a literal |
| N0304 | A kind or a name of a plugin the project does not enable (`[project.plugins]` of the dialect) | `flow Chat` or `Genkit(...)` without `plugins(genkit = true)` in `config.nuc` | enable it in `config.nuc`: `plugins(genkit = true)` |
| N0309 | A project of a platform writes a decorator in lower case its platform does not declare (`decorators` of `platform.toml`): the dialect would drop it or write it as something else. The message names the closest one, or what replaced a removed one (`[decorators.replaced]`) | `@generated()` on a backend entity field | `@primary(generated = true)` |
| N0308 | A field decorator's rule is not met (`[decorators.fields]` of the platform, ADR-0100): a relation field is optional, its type is an entity (a list for `@oneToMany`/`@manyToMany`), `@manyToOne` needs its column `<name>Id`, `inverse` names a field of the related entity, one side of a `@manyToMany` has `@joinTable` and one side of a `@oneToOne` has `@joinColumn` | `@manyToOne project: Project` (not optional, no `projectId`) | `@manyToOne project?: Project` + `projectId: int` |
| N0307 | A contract the manifest declares (`contracts(name = "../path" \| "https://...")`, ADR-0098) cannot be read: the backend at that path was never built, a url contract was never fetched, or the JSON is invalid | `contracts(tasks_api = "https://api.x")` without `nuclear contracts update` | build the backend (`nuclear build ../tasks_api`) or run `nuclear contracts update` |
| N0306 | A kind package (`kinds/<name>/`, ADR-0095) enabled in `config.nuc` has no implementation for the project's dialect | `plugins(jobs = true)` with `kinds/jobs/fastapi/` only, built with `nestjs` (fix: build with fastapi, or add `kinds/jobs/nestjs/`) |
| N0305 | A dialect does not match its platform's contract (`dialects/<platform>/platform.toml`, ADR-0093): a kind missing or added, a consumed decorator or a plugin the platform does not declare (a plugin the dialect does not bring yet is a warning) | `[kinds.widget]` in a backend dialect | declare it in `platform.toml` and implement it in every dialect of the platform, or drop it |

The lexer and the parser never stop at the first error: they report it, recover and continue, so all the
errors come back at once.
