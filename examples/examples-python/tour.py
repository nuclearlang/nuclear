from __future__ import annotations

from enum import Enum, auto
from typing import Iterator, Protocol, override
import asyncio


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


def _show(v: object) -> str:
    if v is True:
        return "true"
    if v is False:
        return "false"
    if v is None:
        return "null"
    return str(v)


def _print(*args: object) -> None:
    print(*(_show(a) for a in args))


class Role(Enum):
    admin = auto()
    user = auto()


class Status(Enum):
    active = "active"
    banned = "banned"


class Shape(Protocol):
    def area(self) -> float: ...


class User:
    name: str
    age: int
    role: Role
    tags: list[str]

    def __init__(self, name: str, age: int = 0) -> None:
        self.name = name
        self.age = age
        self.role = Role.user
        self.tags = []

    def greet(self, other: User) -> str:
        return f"Hi {other.name}, I am {self.name} {{literal}}"

    def is_adult(self) -> bool:
        return self.age >= 18


class Admin(User):
    level: int = 1

    @override
    def greet(self, other: User) -> str:
        return f"Admin {self.name}"


class Square(Shape):
    side: float

    def __init__(self, side: float) -> None:
        self.side = side

    def area(self) -> float:
        return self.side * self.side


class Box[T]:
    value: T

    def __init__(self, value: T) -> None:
        self.value = value

    def get(self) -> T:
        return self.value


def divide(a: int, b: int) -> Result[str, int]:
    if b == 0:
        return Err("division by zero")
    return Ok(a // b)


def half(n: int) -> Result[str, int]:
    x_result = divide(n, 2)
    if x_result.is_err:
        return Err(x_result.error)
    x = x_result.value
    return Ok(x + 0)


def squares(limit: int) -> Iterator[int]:
    for i in range(0, limit):
        yield i * i


def label(n: int) -> str:
    if n == 0:
        return "zero"
    elif n == 1:
        return "one"
    else:
        return "many"


async def fetch_name(id: int) -> str:
    await asyncio.sleep(0)
    return f"user{id}"


def run() -> None:
    users = [User("Ana", 30), User("Luis")]
    for u in users:
        _print(u.greet(users[0]), u.is_adult())
    adults = list(users)
    adults.sort()
    total = 0
    for i in range(0, 5):
        total += i
    _print(total, 2 ** 10, 7 // 2, 7 % 3, 0xff & 0x0f, 1 << 4)
    m = {"a": 1, "b": 2}
    for k, v in m.items():
        _print(k, v)
    _print("a" in m, label(1), list(squares(4)))
    r: Result[str, int] = half(10)
    _print(r.is_ok, r.value)
    e: Result[str, int] = divide(1, 0)
    _print(e.is_err, e.error)
    s = Square(3.0)
    _print(s.area(), Box(5).get())
    name: str | None = None
    _print(name if name is not None else "nobody")
    big = "big" if total > 5 else "small"
    _print(big)
    try:
        raise_it()
    except Exception as err:
        _print("caught", err)
    doubled = list(map(lambda u: u.age * 2, users))
    _print(doubled)
    def _fn1(u: User):
        n = u.name.upper()
        return n + "!"
    names = list(map(_fn1, users))
    _print(names)
    _print(asyncio.run(fetch_name(7)))


def raise_it() -> None:
    raise ValueError("boom")


run()
