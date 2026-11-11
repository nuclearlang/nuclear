# NuclearLang brief (base language)

For someone who knows TypeScript or Dart: only what is different. Full rules: `nuclear help base`; any topic:
`nuclear help <topic>`. Check every file: `nuclear check <file|folder> --json` (each error has a code, a position
and usually the exact `fix`). Format before finishing: `nuclear fmt <file|folder>`.

## Rules that change

| You would write | In Nuclear | Note |
|---|---|---|
| `{ ... }` blocks, `;` | a 2-space indented block, no `;` | a tab is N0002 |
| `function f(a: number): string` | `f(a: int) string` | the return type goes after `)` |
| `return x` at the end | the last line is the value | `return` only to leave early; `void` returns nothing |
| `const x = 1`, `let`, `var`, `final` | `x = 1` | const or mutable is decided by the transpiler |
| `new User("Ana")` | `User("Ana")` | no `new` |
| `f(name: "Ana")`, `f({ name })` | `f(name = "Ana")` | named arguments use `=` |
| `await load()`, `async` | `~load()` | a function that uses `~` is async by itself |
| `&&` `\|\|` `!x` `===` | `and` `or` `not x` `==` | comparisons are always strict |
| `a.contains(x)` / `a.includes(x)` | `x in a` | also `key in map` |
| `class User extends Person implements Named` | `User(name: string) : Person > Named` + block | the header holds the constructor |
| `this.name` | `name`; `$name` when a parameter hides it | |
| `interface`, `enum` | the shape decides: only signatures = interface; only names (or `name("value")`) = enum | |
| `cond ? a : b` statements | `if`/`match` are values | `x = if c` + branches |
| `switch` | `match value` + `pattern -> value`, `_` = else | `is Type ->`, `a \| b ->` |
| `try/catch (e)` | `try` / `catch e` / `finally` | as the last line, its value is returned |
| `x!`, `x?.y`, `x ?? y` | the same | |
| `` `Hi ${name}` `` | `"Hi ${name}"` (both quotes are the same) | `r"\d+"` = raw text (regex) |
| `Math.pow(a, b)`, `a ~/ b` | `a ** b`, `a ~/ b` (integer division) | `-2 ** 2` is -4 |
| bitwise `& \| ^ ~ << >> >>>` | `& \| ^ ^x << >> >>>` | `^x` is the complement (`~` is await) |
| `import` | nothing: the target's library names are imported automatically; `use path{ a, b }` for others | |

## Shapes

```nuclear
// enum
Role
  user
  admin("administrator")

// interface
Named
  name: string
  describe() string

// module (a class): the header is the constructor; each parameter is an attribute
User(name: string, role: Role = Role.user) > Named
  visits: int = 0

  describe() string
    "${name} (${role})"

  visit()
    visits += 1

// functions: one line, a block, async, generator
twice(n: int) int = n * 2

label(n: int) string
  if n < 0
    "negative"
  else if n == 0
    "zero"
  else
    "positive"

squares(limit: int) int
  for i in 0..limit
    yield i * i

main() void
  users = [User("Ana"), User("Luis", role = Role.admin)]
  for u in users
    print(u.describe())
  [first, ...rest] = users
  kind = match first.role
    Role.admin -> "admin"
    _ -> "user"
  print(kind, rest.length)
```

- Types: `string int double number boolean void`, `T[]`, `<K, V>` (map), `T?`, `A | B`, `(a: int) => T`.
- Literals: `1_000`, `0xff`, `0b1010`, `1.5e-3`, `[a, ...b]`, `{ key: value }`, `0..10` (0 to 9), `"""` multi-line.
- Lambdas: `x => x * 2`, `(a, b) => a + b`; a block as the last argument: `items.forEach(item)` + indented block.
- `Result<E, T>` (error first): `Ok(v)`, `Err(e)`; `x = f()` inside a function returning `Result` unwraps it and
  returns the `Err` by itself (`nuclear help results`).
- Comments `//`; the comment right above a declaration is its doc comment.
- Names: `PascalCase` = types and constructors, `camelCase` = the rest. No visibility words: everything is exported.

## The library is the target's

Nuclear has no own standard library: a `.nuc` for TypeScript uses `Math.sqrt`, `xs.filter`, `s.includes`; for Dart
`sqrt`, `xs.where`, `s.contains`. Every area (math, text, regex, collections, types, bytes, dates, JSON, async)
has a working example per target in `examples/stdlib/`. Not sure an API exists: `nuclear defs show <Name>
--dialect <d>` (Dart targets) instead of guessing.

## The compiler corrects habits

Writing TypeScript, Dart or Python by reflex (`{ }` blocks, `await x`, `const x =`, `new`, `class A extends B`,
`int f(int x)`, `def`, `elif`, `None`, `#`, `i++`, `import`) is error `N0022` with the Nuclear form and an exact
`fix`: apply it. With type definitions, an invented class is `N0220` ("did you mean") and an invented named argument
is `N0219` (it lists the real ones). A dialect word: `nuclear help <word> --dialect <d>`; all of them:
`nuclear help words --dialect <d>`.
