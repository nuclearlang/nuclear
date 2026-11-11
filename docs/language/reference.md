# Language reference

The whole base language on one page, for a quick lookup. The [guide](README.md) explains it step by step with
the output in TypeScript and Dart, [spec/nuclear-base.md](../../spec/nuclear-base.md) has the full rules and
[spec/grammar.ebnf](../../spec/grammar.ebnf) the formal grammar. What each dialect adds (kinds, modifiers,
decorators, `use` shortcuts) is in its spec: `spec/<dialect>.md`.

## Files

| Rule | Value |
|---|---|
| Extension and encoding | `.nuc`, UTF-8 |
| Indentation | 2 spaces per level; a tab is `N0002` |
| Blocks | a more indented line opens a block; returning to the previous level closes it |
| Line continuation | inside `( )` `[ ]` `{ }`; a line starting with `.` continues the chain; with `..` it is a cascade |
| Block inside parentheses | after `=>` + a line break, or after `(parameters)` + a more indented line (lambda) |
| End of statement | the line break; `;` is `N0009` |
| Comments | `//` to the end of the line; it continues on lines starting with ` * ` (the `*` below the second `/`); another shape is `N0021` |
| Doc comment | the comment right above a declaration; tags `@param name text`, `@return`, `@throws`, `@example`, `@see`, `@deprecated`, `@since` |

## Names

| Shape | It is | Example |
|---|---|---|
| `PascalCase` | module, interface, enum or type; calling it constructs an object | `User`, `User(name = 'Ana')` |
| `camelCase` | variable, function, parameter, attribute, method | `findUser`, `visits` |
| `_` | ignoring a value in a pattern or a `match` | `[_, b] = xs` |

## Reserved words

```
use from as
if else for in while match return break continue yield
try catch finally throw
and or not is
true false null this super
```

Using one as a name is `N0001`. There is no `import`, `export`, `var`, `const`, `let`, `new`, `class`,
`interface`, `enum`, `async`, `await`, `private` nor `public`. A dialect's words (`stateless`, `static`,
`extension`, `with`...) only mean something in a declaration's header and are not reserved.

## Literals

| Type | Examples |
|---|---|
| integer | `42`, `1_000_000`, `0xff`, `0b1010` |
| decimal | `3.14`, `1.5e-3` |
| raw text | `r"\d+"` (no escapes, no interpolation: regular expressions) |
| text | `'hello'`, `"hello"` (both quotes are the same) |
| interpolation | `'Hello ${name}'` |
| multi-line text | `"""` ... `"""` or `'''` ... `'''` (removes the common indentation) |
| escapes | `\n` `\t` `\r` `\0` `\\` `\"` `\'` `\$` `\u{1F600}` |
| boolean, null | `true`, `false`, `null` |
| list | `[1, 2, 3]`, `[...others, 4]`, `[a, if cond b]` |
| map | `{ role: 'user', content: text }`, `{ text }`, `{ "first-name": n }`, `{ ...base, a: 1 }`, `{}` |
| range | `0..10` (from 0 to 9) |

## Types

| Type | Form | Example |
|---|---|---|
| basic | `string` `int` `double` `number` `boolean` `void` | `age: int = 30` |
| list | `T[]` | `tags: string[] = []` |
| map | `<K, V>` | `scores: <string, int> = {}` |
| nullable | `T?` | `token: string?` |
| union and literals | `A \| B`, `'a' \| 'b'` | `role: 'user' \| 'assistant'` |
| generic | `Name<T, U>` | `Cache<string, User>` |
| function | `(params) => T` | `onTap: () => void` |
| result | `Result<E, T>` (error first) | `Result<string, User>` |

Types are optional where the target infers them (variables, lambda returns).

## Operators

From lowest to highest precedence:

| Level | Operators | Meaning |
|---|---|---|
| 1 | `c ? a : b` | conditional |
| 2 | `??` | default value if null |
| 3 | `or` | logical or |
| 4 | `and` | logical and |
| 5 | `not` | negation |
| 6 | `==` `!=` `<` `>` `<=` `>=` `is` `in` `as` | comparison (always strict), type, membership, conversion |
| 7 | `..` | range (on the same line) |
| 8 | `\|` | bitwise or |
| 9 | `^` | bitwise xor |
| 10 | `&` | bitwise and |
| 11 | `<<` `>>` `>>>` | shifts (`>>>` unsigned) |
| 12 | `+` `-` | addition, subtraction |
| 13 | `*` `/` `%` `~/` | product, division, remainder, integer division |
| 14 | `-x` `~x` `^x` | negative, await, bitwise complement |
| 15 | `a ** b` | power (right associative; `-2 ** 2` is -4) |
| 16 | `a.b` `a?.b` `a(...)` `a<T>(...)` `a[i]` `a!` | member, member if not null, call, generic call, index, not null |

Forms of other languages that are `N0009`, with their fix: `&&` → `and`, `||` → `or`, `!x` → `not x`, `===` →
`==`, `!==` → `!=`.

## Variables and assignment

| Form | Does |
|---|---|
| `x = value` | declares `x` if it does not exist in the block nor in those containing it; otherwise reassigns it |
| `x: Type = value` | declares with a type |
| `x += 1` `-=` `*=` `/=` `%=` `??=` `&=` `\|=` `^=` `<<=` `>>=` `>>>=` | compound assignment |
| `$field = value` | assigns an attribute of the module (when a parameter has the same name) |
| `[a, ...rest] = xs` | destructures a list (`...rest` always last) |
| `{ min, "k": b } = m` | destructures a map |

- Constant or mutable is decided by the transpiler: mutable only if reassigned in its block.
- A variable lives until the end of its block; reading it after is `N0105`. A name nothing defines is `N0106`.
- A pattern has only new names or only existing ones; mixing them is `N0107`.

## Declarations

Every declaration has the **header + block** shape:

```
[@decorators]
[dialect words] [~]Name[<Generics>][(parameters)] [: Bases] [> Interfaces] [Return | clause Type]
  indented body
```

Without a dialect word, the shape decides what it is:

| Shape | It is |
|---|---|
| `camelCase` name | function |
| `PascalCase` with only `name` or `name(value)` items | enum |
| `PascalCase` with only attributes without values and signatures without bodies | interface |
| any other `PascalCase` | module (a class in the target) |

## Functions

| Form | Example |
|---|---|
| with a block; returns the last line | `greet(n: string) string` + block |
| one line | `twice(n: int) int = n * 2` |
| no return | `log(m: string) void` + block |
| async without `~` in the body | `~load() string` + block |
| generator | uses `yield value` / `yield ...other()` in the body |
| alias of a method | `find() = Repo.find` |
| local function | `log = (m: string)` + block, inside a body |

| Parameter | Form |
|---|---|
| required | `name: Type` |
| optional | `name?: Type` |
| with a default | `name: Type = value` |
| named (dialects that have them) | `{name: Type, other: Type = value}` |

| Call | Form |
|---|---|
| by position | `create('Ana', true)` |
| by name (after the positional ones) | `create('Ana', admin = true)` |
| generic | `repo.findAll<User>()` (`<` right after the name) |
| await | `~load(id)`; a function that uses `~` is async by itself |
| spread | `f(...args)` |

- `return` exists to leave early; the last line is returned unless the return type is `void`.
- An assignment as the last line returns the created variable.
- Named functions are not declared inside a body (`N0016`): use a local function.

## Lambdas

| Form | Example |
|---|---|
| one line | `xs.map(n => n * 2)`, `xs.filter((u: User) => u.active)` |
| with a block as the last argument | `users.filter(user: User)` + block |
| with a block among other arguments | `onClick = ()` + block, inside the parentheses |
| assigned to a name | `twice = (n: int) => n * 2` |
| continuing the chain after the block | a `.join(', ')` line at the statement's level |

The last line of the block is the lambda's value. A one-line lambda does not assign: `(v) => x = v` is an error;
the assignment goes in a block.

## Modules

```nuclear
User(name: string, role: Role = Role.user) : Person > Named
  visits: int = 0
  token?: string

  isAdmin() boolean
    role == Role.admin

  visit()
    visits += 1
```

| Member | Form |
|---|---|
| attribute | `name: Type` or `name: Type = value` (`name = value` without a type is `N0017`) |
| constructor in the header | `User(name: string)`: each parameter is an attribute |
| constructor with a body | `(name: string)` on its own line inside the module + block |
| method | like a function, inside the module; without `(...)` it receives no parameters |
| own attribute | `name` (without `this.`); `$name` when a parameter hides it |
| construct | `User('Ana')`, `User(name = 'Ana')` (without `new`) |
| inheritance | `: Base` extends, `> Interface` implements |
| the base's constructor | `super(...)` as the first line of the constructor |
| a method of the base | `super.method()` |
| override | a method named like a member of the base |

A module is not declared inside another (`N0018`). Modifiers (`static`, `abstract`, `get`...) are dialect words.

## Interfaces and enums

| Declaration | Content |
|---|---|
| interface | only attributes without values and signatures without bodies; it may use `> OtherInterface` |
| simple enum | only names, one per line |
| key-value enum | `name("value")`; an item without a value is its own name |

An enum with anything else is `N0104`.

## Generics

| Form | Example |
|---|---|
| generic module | `Box<T>` + block |
| generic function | `first<T>(xs: T[]) T` |
| bound | `<T: Comparable>` (`T` extends `Comparable`) |
| use | `Box<int>(1)`, `items: Box<string>[]` |

## Control flow

| Form | Notes |
|---|---|
| `if c` / `else if c` / `else` + blocks | it is a value: `x = if c` + branches |
| `match value` + branches `pattern -> value` | `_` is any other; `a \| b ->` several patterns; `is Type ->` |
| `for x in list` | `for i in 0..n`, `for k, v in map` |
| `while c` | `break` and `continue` in loops |
| `try` / `catch e` / `finally` | `throw Error('...')`; as the last line, its value is returned |

- As a value, `if` needs `else` and `match` needs `_`; the value of each branch is its last line.
- Inside a larger expression, each branch is a single expression (`N0103`).

## Results

| Form | Does |
|---|---|
| `f() Result<E, T>` | the function can fail with an `E` or give a `T` |
| `Ok(value)`, `Ok()`, `Err(error)` | success, success without a value, failure |
| `x = f()` inside a function that returns `Result` | unwraps: if it is `Err`, the function returns that `Err` |
| `r: Result<E, T> = f()` | keeps the `Result` without unwrapping |
| `.ok(v => ...)` | transforms or observes the value |
| `.error(e => ...)` | recovers, changes or observes the error |
| `match f()` with `Ok(v) ->` and `Err(e) ->` | handles both branches |

Errors: `N0108` (unhandled), `N0109` (another error type), `N0110` (a position that may not run), `N0111`
(invalid use). Details in [chapter 13](13-results.md).

## Cascades

```nuclear
brush = Paint()
  ..color = Colors.blue
  ..strokeWidth = 4
```

Each `..` line assigns a member or calls a method of the same object; the value is the object.

## `use`

| Form | Imports |
|---|---|
| `use path{ a, b }` | the names `a` and `b` of `path` |
| `use * as name from path` | the whole module with a name |
| `use './file'` | a file (it only runs it) |
| `use path as x`, `use path show a` | depending on the dialect |

The names of the target's libraries are imported automatically (auto-imports). Everything declared in a file is
exported: there is no visibility.

## Decorators

`@Name`, `@Name(args)`, `@a.b(args)` before a declaration, a member or a parameter. They pass to the target
untouched, except those the dialect consumes (for example `@screen` in flutter).

## Symbols

| Symbol | Use |
|---|---|
| `:` | type (`x: int`), inheritance (`Admin: User`), bound (`<T: X>`), map entry |
| `>` | implements (`User > Named`) |
| `=` | assignment, named argument, default value, one-line function |
| `=>` | one-line lambda, function type |
| `->` | `match` branch |
| `~` | await (`~f()`) or async function (`~name()` in the declaration) |
| `$` | attribute of the module (`$name`) |
| `@` | decorator |
| `?` | optional (`x?: T`), nullable (`T?`), conditional (`c ? a : b`) |
| `?.` `??` `??=` | access, default value and assignment if null |
| `!` | not null, right after (`user!.name`) |
| `..` | range (`0..10`) or cascade (at the start of a line) |
| `...` | spread, rest in a pattern, `yield ...` |
| `\|` | union of types, several patterns in a `match` |

## Diagnostics

| Range | Area |
|---|---|
| `N00xx` | lexing and syntax |
| `N01xx` | names, scope and `Result` |
| `N02xx` | dialect |
| `N03xx` | projects and architecture |

`nuclear check file.nuc --dialect <d> --json` returns every error with its code, position and, when possible,
the exact fix. The full list is in [spec/diagnostics.md](../../spec/diagnostics.md).
