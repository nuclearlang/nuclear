# Dialect `python` brief

Python 3.12+, typed (passes `pyright --strict`). The library is Nuclear's (`xs.length`, `xs.add(x)`, `text.upper()`: `nuclear help library`), written as Python by the dialect (`len(xs)`, `xs.append(x)`),
`s.upper()`, a comprehension for `xs.map((x) => ...)`, `sorted(xs)`). Names follow Nuclear (`camelCase`, `PascalCase`
for classes) and are written in Python's `snake_case`: `getOne()` → `get_one()`, `addNode` → `add_node`. Standard library names import by themselves (`datetime`, `Path`, `dataclass`);
`nuclear defs show <Name> --dialect python` prints a class.

| Nuclear | Python |
|---|---|
| `userName = 1`, `"Hi ${name}"` | `user_name = 1`, `f"Hi {name}"` |
| `f(x = 1)`, `inject<Repo>()` | `f(x=1)`, `inject(Repo)` |
| `name?: string` | `name: str \| None = None` |
| `User(name: string)` + fields | class annotations + `__init__` that sets them (`self.`) |
| `Admin: User`, `Square > Shape` | `class Admin(User)`, `Shape` is a `Protocol` |
| `Box<T>` | `class Box[T]` |
| `~call()` | `await call()` (the function is `async def`) |
| `x => x * 2`; a lambda with a block | `lambda x: x * 2`; a local `def _fn1(...)` before the statement |
| `a ?? b`, `a ~/ b`, `a ** b` | `a if a is not None else b`, `a // b`, `a ** b` |
| `throw ValueError("x")`, `catch e` | `raise ValueError("x")`, `except Exception as e` |
| `// doc` above a function | its docstring |
| `Result<E, T>`, `Ok`, `Err` | the file's `Result` runtime (`.isOk`, `.value`, `.error`) |

```nuclear
// Counts the words of a text.
wordCount(text: string) <string, int>
  counts: <string, int> = {}
  for word in text.lower().split(" ")
    counts[word] = (counts[word] ?? 0) + 1
  counts

fetch(id: int) string
  ~delay(100)
  "item${id}"
```
