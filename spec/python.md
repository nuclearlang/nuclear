# Dialect `python`

> Inherits every rule of [nuclear-base.md](nuclear-base.md). This document defines how the `python` dialect
> writes the base language in Python 3.12+ (PEP 8, PEP 695 generics, type hints that pass `pyright --strict`) and
> (section 3) the equivalence of the base examples. Files: `dialects/python/`. The `fastapi` dialect
> ([fastapi.md](https://github.com/nuclearlang/dragonfly-backend/blob/main/spec/fastapi.md)) extends it.

## 1. Rules

- **Target:** Python 3.12 or later. Every file starts with `from __future__ import annotations` (a method can name
  its own class). 4 spaces, lines up to 88 columns, two blank lines between top-level declarations.
- **Names:** the `.nuc` follows Nuclear's rule (`camelCase` for variables, functions, methods, fields and parameters;
  `PascalCase` for classes). The dialect writes them in Python's `snake_case` (options.snake_case_names):
  `getOne()` → `get_one()`, `addNode` → `add_node`, `minLength = 1` → `min_length=1`. A name that exists as it is in
  the type definitions (`getLogger` of logging) is kept. Strings and map keys are data and never change
  (`where({ "user_id": id })`).
- **The library is Python's:** `len(xs)`, `xs.append(x)`, `s.upper()`, `list(map(f, xs))`, `sorted(xs)`. There is
  no comprehension syntax: use `map`, `filter` and loops.
- **Named arguments** are Python keyword arguments: `f(x = 1)` → `f(x=1)`. `{a, b}` parameters are keyword-only
  (`*, a, b`). `name?: T` is `name: T | None = None`.
- **Type arguments of a call are arguments:** `inject<Repo>()` → `inject(Repo)` (options.type_args_as_args).
- **Types:** `string` `str`, `int` `int`, `double`/`number` `float`, `boolean` `bool`, `void` `None`, `any` `Any`,
  `T[]` `list[T]`, `<K, V>` `dict[K, V]`, `T?` `T | None`, `(a: A) => R` `Callable[[A], R]`, generators
  `Iterator[T]` / `AsyncIterator[T]`.
- **Operators:** `and` `or` `not`, `~/` `//`, `**`, bitwise as in Python; `>>>` is written on a 32-bit value
  (`(x % 0x100000000) >> n`), like TypeScript; `a in b` is Python's `in`.
- **Type definitions:** `defs = ["python@3.14"]`: builtins (implicit), `typing`, `dataclasses`, `enum`, `abc`,
  `collections`, `functools`, `itertools`, `datetime`, `time`, `json`, `re`, `math`, `random`, `decimal`, `uuid`,
  `pathlib`, `os`, `logging`, `asyncio`, `contextlib`. Generator: `tools/defs/python/` (any installed package).
- **Projects** (`nuclear.toml` with `dialect = "python"`): the files go to the package `app/` (or to the one
  `[project] package = "my_lib"` names: a library is imported by its own name) and import each other
  as `from app.<path> import Name`.

## 2. Emitter options this dialect uses

`hoist_block_lambdas` (a lambda with a block → a local `def`), `init_fields` (fields with a default and constructor
parameters are set in `__init__`), `doc_inside` (docstrings), `interp_prefix` + `interp_brace_escape`
(f-strings), `module_separator = "."`, `toplevel_gap = 2`, `type_args_as_args`, `file_prelude`. Meaning:
`nuclear spec` (§ options) and [dialects.md](dialects.md).

## 3. Equivalences

### 3.1 Variables and strings

A variable is assigned; a string with `${}` is an f-string; literal braces are doubled.

```nuclear
greet(name: string) string
  count = 3
  "Hi ${name}, you have ${count} messages {not interpolated}"
```

```python
from __future__ import annotations

def greet(name: str) -> str:
    count = 3
    return f"Hi {name}, you have {count} messages {{not interpolated}}"
```

### 3.2 Modules

A module with header parameters gets class annotations and `__init__`; fields with a default are set in `__init__` (no shared mutable defaults). Members are written with `self.`.

```nuclear
User(name: string, age: int = 0)
  tags: string[] = []

  isAdult() boolean
    age >= 18
```

```python
from __future__ import annotations


class User:
    name: str
    age: int
    tags: list[str]

    def __init__(self, name: str, age: int = 0) -> None:
        self.name = name
        self.age = age
        self.tags = []

    def is_adult(self) -> bool:
        return self.age >= 18
```

### 3.3 Inheritance and override

`: Base` is the base class; a method named like one of the base gets `@override` (typing). A subclass without its own constructor keeps its defaults in the class.

```nuclear
Animal(name: string)
  sound() string
    "..."

Dog: Animal
  sound() string
    "${name} barks"
```

```python
from __future__ import annotations

from typing import override

class Animal:
    name: str

    def __init__(self, name: str) -> None:
        self.name = name

    def sound(self) -> str:
        return "..."


class Dog(Animal):
    @override
    def sound(self) -> str:
        return f"{self.name} barks"
```

### 3.4 Interfaces

A module with only signatures is a `Protocol`.

```nuclear
Shape
  area() double

Square(side: double) > Shape
  area() double
    side * side
```

```python
from __future__ import annotations

from typing import Protocol

class Shape(Protocol):
    def area(self) -> float: ...


class Square(Shape):
    side: float

    def __init__(self, side: float) -> None:
        self.side = side

    def area(self) -> float:
        return self.side * self.side
```

### 3.5 Enums

```nuclear
Role
  admin
  user

Status
  active("active")
  banned("banned")
```

```python
from __future__ import annotations

from enum import Enum, auto

class Role(Enum):
    admin = auto()
    user = auto()


class Status(Enum):
    active = "active"
    banned = "banned"
```

### 3.6 Generics

PEP 695: `Box<T>` → `class Box[T]`, bounds with `:`.

```nuclear
Box<T>(value: T)
  get() T
    value

first<T>(items: T[]) T
  items[0]
```

```python
from __future__ import annotations

class Box[T]:
    value: T

    def __init__(self, value: T) -> None:
        self.value = value

    def get(self) -> T:
        return self.value


def first[T](items: list[T]) -> T:
    return items[0]
```

### 3.7 Async

`~call()` is `await`; the function becomes `async def`.

```nuclear
fetchName(id: int) string
  ~delay(100)
  "user${id}"
```

```python
from __future__ import annotations

import asyncio


async def fetch_name(id: int) -> str:
    await asyncio.sleep(100 / 1000)
    return f"user{id}"
```

### 3.8 Control flow

`if` and `match` are values: as the last line each branch returns; inside an expression with one expression per branch they are a conditional expression (`a if c else b`). `match` becomes `if`/`elif`.

```nuclear
label(n: int) string
  match n
    0 -> "zero"
    1 -> "one"
    _ -> "many"

sign(n: int) string
  if n >= 0
    "positive"
  else
    "negative"
```

```python
from __future__ import annotations

def label(n: int) -> str:
    if n == 0:
        return "zero"
    elif n == 1:
        return "one"
    else:
        return "many"


def sign(n: int) -> str:
    if n >= 0:
        return "positive"
    else:
        return "negative"
```

### 3.9 Loops

`for i in a..b` is `range`; `for k, v in map` uses `.items()`.

```nuclear
total(m: <string, int>) int
  sum = 0
  for k, v in m
    sum += v
  for i in 0..3
    sum += i
  sum
```

```python
from __future__ import annotations

def total(m: dict[str, int]) -> int:
    sum = 0
    for k, v in m.items():
        sum += v
    for i in range(0, 3):
        sum += i
    return sum
```

### 3.10 Lambdas

An expression lambda is a `lambda` without types; a lambda with a block becomes a local `def` written before its statement (options.hoist_block_lambdas).

```nuclear
names(users: User[]) string[]
  ages = list(map((u: User) => u.age, users))
  list(map((u: User)
    n = u.name.upper()
    n + "!"
  , users))
```

```python
from __future__ import annotations

def names(users: list[User]) -> list[str]:
    ages = list(map(lambda u: u.age, users))
    def _fn1(u: User):
        n = u.name.upper()
        return n + "!"
    return list(map(_fn1, users))
```

### 3.11 Null

`null` is `None`; `a ?? b` and `a?.b` test `is not None`.

```nuclear
display(name: string?) string
  name ?? "nobody"
```

```python
from __future__ import annotations

def display(name: str | None) -> str:
    return name if name is not None else "nobody"
```

A call or a map read on the left of `??` is evaluated once (pyright cannot narrow a call written twice); the right
side still runs only when the left is `None`:

```nuclear
lookup(names: <string, string>, key: string) string
  names[key] ?? "nobody"
```

```python
def _coalesce[T](value: T | None, fallback: Callable[[], T]) -> T:
    return value if value is not None else fallback()


def lookup(names: dict[str, str], key: str) -> str:
    return _coalesce(names.get(key), lambda: "nobody")
```

### 3.12 Errors

`throw` is `raise`; `catch` catches `Exception`. A `catch` whose block does not name the caught value binds none
(`except Exception:`): a name nothing reads is an error of pyright's strict mode.

```nuclear
safe() string
  try
    risky()
    "ok"
  catch err
    "failed: ${err}"
```

```python
from __future__ import annotations

def safe() -> str:
    try:
        risky()
        return "ok"
    except Exception as err:
        return f"failed: {err}"
```

```nuclear
ready() boolean
  try
    risky()
    true
  catch
    false
```

```python
from __future__ import annotations

def ready() -> bool:
    try:
        risky()
        return True
    except Exception:
        return False
```

### 3.13 Result

`Result<E, T>` is the runtime of the file (or `app/nuclear/result.py` in a project); an unwrapped `Result` returns its `Err` by itself.

```nuclear
divide(a: int, b: int) Result<string, int>
  if b == 0
    return Err("division by zero")
  Ok(a ~/ b)

half(n: int) Result<string, int>
  x = divide(n, 2)
  Ok(x)
```

```python
from __future__ import annotations


from collections.abc import Callable
from typing import Generic, Never, TypeVar, cast

_E = TypeVar("_E", covariant=True)
_T = TypeVar("_T", covariant=True)
_F = TypeVar("_F")
_U = TypeVar("_U")


class Result(Generic[_E, _T]):
    """An operation that fails with an error _E (Err) or succeeds with a value _T (Ok)."""

    __slots__ = ("is_ok", "_value", "_error")

    def __init__(self, is_ok: bool, value: _T | None = None, error: _E | None = None) -> None:
        self.is_ok = is_ok
        self._value = value
        self._error = error

    def __repr__(self) -> str:
        return f"Ok({self._value!r})" if self.is_ok else f"Err({self._error!r})"

    @property
    def is_err(self) -> bool:
        return not self.is_ok

    @property
    def value(self) -> _T:
        if not self.is_ok:
            raise ValueError("Err has no value")
        return cast(_T, self._value)

    @property
    def error(self) -> _E:
        if self.is_ok:
            raise ValueError("Ok has no error")
        return cast(_E, self._error)

    def and_then(self, f: Callable[[_T], Result[_F, _U]]) -> Result[_E | _F, _U]:
        return f(self.value) if self.is_ok else Result[_E | _F, _U](False, None, self._error)

    def or_else(self, f: Callable[[_E], Result[_F, _U]]) -> Result[_F, _T | _U]:
        return Result[_F, _T | _U](True, self._value) if self.is_ok else f(self.error)

    def map(self, f: Callable[[_T], _U]) -> Result[_E, _U]:
        return Result[_E, _U](True, f(self.value)) if self.is_ok else Result[_E, _U](False, None, self._error)

    def get_or_else(self, f: Callable[[_E], _U]) -> _T | _U:
        return self.value if self.is_ok else f(self.error)

    def inspect(self, f: Callable[[_T], object]) -> Result[_E, _T]:
        if self.is_ok:
            f(self.value)
        return self

    def inspect_err(self, f: Callable[[_E], object]) -> Result[_E, _T]:
        if not self.is_ok:
            f(self.error)
        return self


def Ok(value: _U) -> Result[Never, _U]:
    return Result[Never, _U](True, value)


def Err(error: _F) -> Result[_F, Never]:
    return Result[_F, Never](False, None, error)


def divide(a: int, b: int) -> Result[str, int]:
    if b == 0:
        return Err("division by zero")
    return Ok(a // b)


def half(n: int) -> Result[str, int]:
    x_result = divide(n, 2)
    if x_result.is_err:
        return Err(x_result.error)
    x = x_result.value
    return Ok(x)
```

### 3.14 Doc comments

A doc comment is the docstring of its function, method or class (options.doc_inside).

```nuclear
// Adds two numbers.
// @param a the first
// @return the sum
add(a: int, b: int) int
  a + b
```

```python
from __future__ import annotations

def add(a: int, b: int) -> int:
    """
    Adds two numbers.

    Args:
        a: the first

    Returns:
        the sum
    """
    return a + b
```

### 3.15 Imports

`use a/b{ x }` → `from a.b import x`; names of the standard library are imported by themselves (type definitions `python@3.14`).

```nuclear
use json

now() string
  json.dumps({ "at": 1 })
```

```python
from __future__ import annotations

import json


def now() -> str:
    return json.dumps({"at": 1})
```

## 4. The standard library

[library.md](library.md) as `dialects/python/library.toml` writes it: lists, dicts and sets are Python's; an
expression lambda becomes a comprehension (`xs.map((x) => x * 2)` is `[x * 2 for x in xs]`, `filter`, `find`,
`any`, `all`, `count` likewise; a function value goes through `map`/`filter`); `Bytes` is `bytes`; `DateTime` is
`datetime.datetime` (aware in UTC when built with `DateTime.utc`), `Duration` is `datetime.timedelta`; `Regex` is
`re.compile` through `_regex`, which also writes `(?<name>...)` as `(?P<name>...)`; `Match` is `re.Match`;
`Error(message)` is `Exception(message)` and `e.message` is `str(e)`; a map read `m[k]` is `m.get(k)` (`None`
when the key is absent, as [library.md](library.md) §3 says; `m[k] = v` stays an index). `x!` is `_sure(x)` (an assertion pyright
understands), `print` goes through `_print` (`true`, `false` and `null` as Nuclear prints them), `round` is half
away from zero; `toInt`, `toDouble`, `indexOf` on lists, `remove`, `groupBy`, `toRadix`, `toIso`, `Json.encode`
(compact, models through `model_dump` or `vars`) are helpers written once per file. `math`, `json`, `re`,
`base64`, `datetime`, `functools`, `random` and `asyncio` are imported when written, and those names are reserved
for Nuclear identifiers (`json_`). `string.length` counts characters, not UTF-16 code units; `string.split("")` is
not valid (use `chars()`).
