# 12. Frequent errors

Every error has a stable code, its position and, when possible, the exact fix. `nuclear check file.nuc
--dialect <d> --json` returns the list as JSON so an agent can fix the code without reading documentation.
The full list is in [spec/diagnostics.md](../../spec/diagnostics.md).

## The most common ones

| You write | Error | It is written |
|---|---|---|
| `a && b`, `a \|\| b` | `N0009` | `a and b`, `a or b` |
| `!active` | `N0009` | `not active` (`x!` after a value does exist: not null) |
| `title: x` in a call | `N0015` | `title = x` (`:` is for types in the signature) |
| 3-space indentation or tabs | `N0005` | multiples of 2 spaces |
| `new User()` | `N0013` | `User()` |
| `const x = 1`, `var x = 1` | `N0013` | `x = 1` |
| `(v) => email = v` | `N0013` | a lambda with a block: `(v)` + a block with `email = v` |
| `1..10.map(...)` | `N0014` | `(1..10).map(...)` |
| reading a variable outside the block where it was created | `N0105` | `x = if ...` or create it before the block |
| using a name that does not exist | `N0106` | declare it, import it or pass it as a parameter |
| `[a, b] = xs` with `a` existing and `b` new | `N0107` | all names new or all existing |
| an enum with items and methods | `N0104` | separate the enum from the module |
| `C: A, B` in a dialect with a single base | `N0208` | `C: A > B` if `B` is an interface |
| a header word the dialect does not know | `N0201` | the one suggested in the message |
| `x = call()` returning `Result` in a function that does not return `Result` | `N0108` | `match`, a variable typed `Result`, or return `Result` ([chapter 13](13-results.md)) |
| propagating an error of another type | `N0109` | `call().error(e => Err(MyError(e)))` |
| `Result` on the right of `and`/`or`/`??`, in a ternary or in a `while` | `N0110` | assign it to a variable first |
| `match asyncCall()` without `~`, a branch `Ok(1) ->` | `N0111` | `match ~asyncCall()`, `Ok(x) ->` |
| `f() = Repo.find` with `Repo(url: string)` | `N0112` | give `url` a default value, or write `f` as a method |

## What an error looks like

```
error[N0105]: 'text' was created inside a block (if, for, while, match, try) and does not exist
here; to take a value out of an if or match, use it as a value: text = if ...
 --> e.nuc:4:9
  |
4 |   print(text)
  |         ^^^^
```

With `--json`:

```json
[
  {
    "code": "N0009",
    "col": 5,
    "end_col": 7,
    "end_line": 2,
    "explanation-context": "Line 2, column 5: NuclearLang uses `and`, `or`, `not`, `==`, `!=` and has no `;`.",
    "file": "j.nuc",
    "fix": { "col": 5, "line": 2, "replace": "&&", "with": "and" },
    "line": 2,
    "message": "NuclearLang does not use '&&'; use 'and'",
    "severity": "error"
  }
]
```

`fix` says exactly which text to replace and with what: an agent applies it without thinking.
