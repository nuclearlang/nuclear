# 13. Results: `Result`, `Ok` and `Err`

An operation that can fail (finding a user, reading a file, charging a card) does not throw an exception: it
returns a **result**. A `Result<E, T>` (first the error type, then the value type) is one of two things:

- `Ok(value)`: it went well, with a value of type `T`;
- `Err(error)`: it failed, with an error of type `E`.

`Result`, `Ok` and `Err` exist without declaring nor importing them. The formal rules are in
[spec/nuclear-base.md §3.2](../../spec/nuclear-base.md).

```nuclear
User(name: string)

findUser(id: string) Result<string, User>
  if id == ''
    return Err('empty id')
  Ok(User(id))
```

```typescript
// Result<E, T>: an operation that fails with an error E (Err) or succeeds with a value T (Ok)
export class Result<E, T> {
  private constructor(
    readonly isOk: boolean,
    private readonly okValue: T | undefined,
    private readonly errValue: E | undefined,
  ) {}

  static ok<T>(value: T): Result<never, T> {
    return new Result<never, T>(true, value, undefined);
  }

  static err<E>(error: E): Result<E, never> {
    return new Result<E, never>(false, undefined, error);
  }

  get isErr(): boolean {
    return !this.isOk;
  }

  get value(): T {
    if (!this.isOk) throw new Error('Err has no value');
    return this.okValue as T;
  }

  get error(): E {
    if (this.isOk) throw new Error('Ok has no error');
    return this.errValue as E;
  }

  andThen<F, U>(f: (value: T) => Result<F, U>): Result<E | F, U> {
    return this.isOk ? f(this.okValue as T) : Result.err(this.errValue as E);
  }

  orElse<F, U>(f: (error: E) => Result<F, U>): Result<F, T | U> {
    return this.isOk ? Result.ok(this.okValue as T) : f(this.errValue as E);
  }

  map<U>(f: (value: T) => U): Result<E, U> {
    return this.isOk ? Result.ok(f(this.okValue as T)) : Result.err(this.errValue as E);
  }

  getOrElse(f: (error: E) => T): T {
    return this.isOk ? (this.okValue as T) : f(this.errValue as E);
  }

  inspect(f: (value: T) => unknown): Result<E, T> {
    if (this.isOk) f(this.okValue as T);
    return this;
  }

  inspectErr(f: (error: E) => unknown): Result<E, T> {
    if (!this.isOk) f(this.errValue as E);
    return this;
  }
}

export function Ok<T = void>(value?: T): Result<never, T> {
  return Result.ok(value as T);
}

export function Err<E>(error: E): Result<E, never> {
  return Result.err(error);
}

export class User {
  constructor(public name: string) {}
}

export function findUser(id: string): Result<string, User> {
  if (id === '') {
    return Err('empty id');
  }
  return Ok(new User(id));
}
```

In TypeScript the transpiler adds once, at the top of the file, the code of `Result` (a class with `Ok` and
`Err`); in Dart it uses fpdart's `Either` (`Ok` → `Right`, `Err` → `Left`). Nothing has to be installed by hand.

`Ok(...)` and `Err(...)` are always written: a plain value on the last line does not become `Ok` by itself.

## Automatic propagation

This is the central idea. **Inside a function that returns `Result`, there is no need to check each call:**

- if the call gives `Ok(value)`, the variable gets `value` and the function continues;
- if it gives `Err(error)`, the function ends there and returns that same `Err`.

```nuclear
User(name: string)

findUser(id: string) Result<string, User>
  if id == ''
    return Err('empty id')
  Ok(User(id))

checkStock(id: string) Result<string, int>
  id.length > 3 ? Ok(5) : Err('no stock')

buy(id: string) Result<string, string>
  user = findUser(id)
  stock = checkStock(id)
  Ok('${user.name} x${stock}')
```

```typescript
export function buy(id: string): Result<string, string> {
  const userResult = findUser(id);
  if (userResult.isErr) return Err(userResult.error);
  const user = userResult.value;
  const stockResult = checkStock(id);
  if (stockResult.isErr) return Err(stockResult.error);
  const stock = stockResult.value;
  return Ok(`${user.name} x${stock}`);
}
```

Look at what the transpiler generated: for each call, it stores the result, returns the error if there is one
and continues with the value. That is what you do not have to write.

### What propagates

- Calls to functions and methods **written in Nuclear** with return type `Result<E, T>`. External libraries are
  not touched.
- An async function is called with `~`: `user = ~loadUser(id)`.
- A call whose value is not used also propagates its error: `save(user)` on its own line.
- A call inside another expression is resolved before the statement, in order:
  `Ok(Pair(findUser(a), findUser(b)))` propagates the error of `findUser(a)` first.

### Where it cannot propagate

In a part that may not run: the right side of `and`, `or` and `??`, the branches of `c ? a : b`, a condition of
`else if` or `while`, an `if` inside a list. There, assign it to a variable first:

```nuclear error
ok(admin: boolean, id: string) Result<string, boolean>
  allowed = admin or findUser(id).name == 'root'    // N0110
  Ok(allowed)
```

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

ok(admin: boolean, id: string) Result<string, boolean>
  user = findUser(id)
  allowed = admin or user.name == 'root'
  Ok(allowed)
```

```typescript
// Result<E, T>: an operation that fails with an error E (Err) or succeeds with a value T (Ok)
export class Result<E, T> {
  private constructor(
    readonly isOk: boolean,
    private readonly okValue: T | undefined,
    private readonly errValue: E | undefined,
  ) {}

  static ok<T>(value: T): Result<never, T> {
    return new Result<never, T>(true, value, undefined);
  }

  static err<E>(error: E): Result<E, never> {
    return new Result<E, never>(false, undefined, error);
  }

  get isErr(): boolean {
    return !this.isOk;
  }

  get value(): T {
    if (!this.isOk) throw new Error('Err has no value');
    return this.okValue as T;
  }

  get error(): E {
    if (this.isOk) throw new Error('Ok has no error');
    return this.errValue as E;
  }

  andThen<F, U>(f: (value: T) => Result<F, U>): Result<E | F, U> {
    return this.isOk ? f(this.okValue as T) : Result.err(this.errValue as E);
  }

  orElse<F, U>(f: (error: E) => Result<F, U>): Result<F, T | U> {
    return this.isOk ? Result.ok(this.okValue as T) : f(this.errValue as E);
  }

  map<U>(f: (value: T) => U): Result<E, U> {
    return this.isOk ? Result.ok(f(this.okValue as T)) : Result.err(this.errValue as E);
  }

  getOrElse(f: (error: E) => T): T {
    return this.isOk ? (this.okValue as T) : f(this.errValue as E);
  }

  inspect(f: (value: T) => unknown): Result<E, T> {
    if (this.isOk) f(this.okValue as T);
    return this;
  }

  inspectErr(f: (error: E) => unknown): Result<E, T> {
    if (!this.isOk) f(this.errValue as E);
    return this;
  }
}

export function Ok<T = void>(value?: T): Result<never, T> {
  return Result.ok(value as T);
}

export function Err<E>(error: E): Result<E, never> {
  return Result.err(error);
}

export class User {
  constructor(public name: string) {}
}

export function findUser(id: string): Result<string, User> {
  return Ok(new User(id));
}

export function ok(admin: boolean, id: string): Result<string, boolean> {
  const userResult = findUser(id);
  if (userResult.isErr) return Err(userResult.error);
  const user = userResult.value;
  const allowed = admin || user.name === 'root';
  return Ok(allowed);
}
```

### Outside a function that returns `Result`

If the function does not return `Result`, there is nowhere to send the error, and letting it pass silently would
be a bug. It is error `N0108`:

```nuclear error
greet(id: string) string
  user = findUser(id)     // N0108: greet does not return Result
  'hi ${user.name}'
```

There are four ways to fix it: a `match` (below), storing the `Result` in a typed variable, handling the error
with `.error(...)` on its own line, or making the function return `Result`.

### The error type must match

An error of type `string` cannot leave a function that returns `Result<AppError, Order>` (error `N0109`).
Convert it with `.error(...)`:

```nuclear
User(name: string)
AppError(message: string)

findUser(id: string) Result<string, User> = Ok(User(id))

load(id: string) Result<AppError, User>
  user = findUser(id).error(e => Err(AppError(e)))
  Ok(user)
```

```typescript
// Result<E, T>: an operation that fails with an error E (Err) or succeeds with a value T (Ok)
export class Result<E, T> {
  private constructor(
    readonly isOk: boolean,
    private readonly okValue: T | undefined,
    private readonly errValue: E | undefined,
  ) {}

  static ok<T>(value: T): Result<never, T> {
    return new Result<never, T>(true, value, undefined);
  }

  static err<E>(error: E): Result<E, never> {
    return new Result<E, never>(false, undefined, error);
  }

  get isErr(): boolean {
    return !this.isOk;
  }

  get value(): T {
    if (!this.isOk) throw new Error('Err has no value');
    return this.okValue as T;
  }

  get error(): E {
    if (this.isOk) throw new Error('Ok has no error');
    return this.errValue as E;
  }

  andThen<F, U>(f: (value: T) => Result<F, U>): Result<E | F, U> {
    return this.isOk ? f(this.okValue as T) : Result.err(this.errValue as E);
  }

  orElse<F, U>(f: (error: E) => Result<F, U>): Result<F, T | U> {
    return this.isOk ? Result.ok(this.okValue as T) : f(this.errValue as E);
  }

  map<U>(f: (value: T) => U): Result<E, U> {
    return this.isOk ? Result.ok(f(this.okValue as T)) : Result.err(this.errValue as E);
  }

  getOrElse(f: (error: E) => T): T {
    return this.isOk ? (this.okValue as T) : f(this.errValue as E);
  }

  inspect(f: (value: T) => unknown): Result<E, T> {
    if (this.isOk) f(this.okValue as T);
    return this;
  }

  inspectErr(f: (error: E) => unknown): Result<E, T> {
    if (!this.isOk) f(this.errValue as E);
    return this;
  }
}

export function Ok<T = void>(value?: T): Result<never, T> {
  return Result.ok(value as T);
}

export function Err<E>(error: E): Result<E, never> {
  return Result.err(error);
}

export class User {
  constructor(public name: string) {}
}

export class AppError {
  constructor(public message: string) {}
}

export function findUser(id: string): Result<string, User> {
  return Ok(new User(id));
}

export function load(id: string): Result<AppError, User> {
  const userResult = findUser(id).orElse((e) => Err(new AppError(e)));
  if (userResult.isErr) return Err(userResult.error);
  const user = userResult.value;
  return Ok(user);
}
```

## Keeping a `Result` without propagating it

Sometimes you want the `Result` itself, not its value. A `Result` is kept when:

- you store it in a variable **typed** `Result`: `r: Result<string, User> = findUser(id)`;
- it is what the function returns (the last line or a `return`);
- you pass it to a parameter typed `Result`;
- it is the subject of a `match` or the object of `.ok()` / `.error()`.

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

later(id: string) Result<string, User>
  r: Result<string, User> = findUser(id)
  findUser(id)
```

```typescript
// Result<E, T>: an operation that fails with an error E (Err) or succeeds with a value T (Ok)
export class Result<E, T> {
  private constructor(
    readonly isOk: boolean,
    private readonly okValue: T | undefined,
    private readonly errValue: E | undefined,
  ) {}

  static ok<T>(value: T): Result<never, T> {
    return new Result<never, T>(true, value, undefined);
  }

  static err<E>(error: E): Result<E, never> {
    return new Result<E, never>(false, undefined, error);
  }

  get isErr(): boolean {
    return !this.isOk;
  }

  get value(): T {
    if (!this.isOk) throw new Error('Err has no value');
    return this.okValue as T;
  }

  get error(): E {
    if (this.isOk) throw new Error('Ok has no error');
    return this.errValue as E;
  }

  andThen<F, U>(f: (value: T) => Result<F, U>): Result<E | F, U> {
    return this.isOk ? f(this.okValue as T) : Result.err(this.errValue as E);
  }

  orElse<F, U>(f: (error: E) => Result<F, U>): Result<F, T | U> {
    return this.isOk ? Result.ok(this.okValue as T) : f(this.errValue as E);
  }

  map<U>(f: (value: T) => U): Result<E, U> {
    return this.isOk ? Result.ok(f(this.okValue as T)) : Result.err(this.errValue as E);
  }

  getOrElse(f: (error: E) => T): T {
    return this.isOk ? (this.okValue as T) : f(this.errValue as E);
  }

  inspect(f: (value: T) => unknown): Result<E, T> {
    if (this.isOk) f(this.okValue as T);
    return this;
  }

  inspectErr(f: (error: E) => unknown): Result<E, T> {
    if (!this.isOk) f(this.errValue as E);
    return this;
  }
}

export function Ok<T = void>(value?: T): Result<never, T> {
  return Result.ok(value as T);
}

export function Err<E>(error: E): Result<E, never> {
  return Result.err(error);
}

export class User {
  constructor(public name: string) {}
}

export function findUser(id: string): Result<string, User> {
  return Ok(new User(id));
}

export function later(id: string): Result<string, User> {
  const r: Result<string, User> = findUser(id);
  return findUser(id);
}
```

## `.ok()` and `.error()`: handling one branch

`.ok(callback)` acts when the result is `Ok`; `.error(callback)` when it is `Err`. The other branch passes
unchanged. **What the callback's last line returns decides what happens:**

| The callback returns | `.ok(v => ...)` | `.error(e => ...)` |
|---|---|---|
| `Ok(...)` or `Err(...)` | **transforms**: that is the new result | **recovers** (`Ok`) or **changes the error** (`Err`) |
| a value (`User('guest')`, `u.name`, a function of yours that returns something) | **transforms** the value | **recovers**: the expression is that value (no longer a `Result`) |
| nothing (`log(e)`, `print(v)`, an external function) | **observes**: the `Ok` stays the same | **observes**: the `Err` stays the same and propagates |

Logging an error does not handle it: `.error(e => log(e))` lets it continue. To recover, return a value
(`.error(e => User('guest'))`) or an `Ok(...)`.

This separates *looking at* an error from *fixing* it: logging an error does not turn it into a success.

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

checkStock(id: string) Result<string, int> = Ok(id.length)

log(message: string) void
  print(message)

buy(id: string) Result<string, string>
  stock = checkStock(id)
    .error(e => log(e))
  guest = findUser('x')
    .error(e => Ok(User('guest')))
  name = findUser(id)
    .ok(u => Ok(u.name))
  Ok('${name} ${stock} ${guest.name}')
```

```typescript
export function buy(id: string): Result<string, string> {
  const stockResult = checkStock(id).inspectErr((e) => log(e));
  if (stockResult.isErr) return Err(stockResult.error);
  const stock = stockResult.value;
  const guestResult = findUser('x').orElse((e) => Ok(new User('guest')));
  if (guestResult.isErr) return Err(guestResult.error);
  const guest = guestResult.value;
  const nameResult = findUser(id).andThen((u) => Ok(u.name));
  if (nameResult.isErr) return Err(nameResult.error);
  const name = nameResult.value;
  return Ok(`${name} ${stock} ${guest.name}`);
}
```

- `stock`: if `checkStock` fails, it is logged **and** `buy` returns the error.
- `guest`: if `findUser` fails, the user `guest` is used and `buy` continues.
- `name`: the value is transformed (from `User` to its name).

A callback returns `Ok`/`Err` in **all** its branches or in **none**. Mixing them is error `N0111`. Inside a
callback that returns `Result`, errors also propagate by themselves.

## `match`: deciding with both branches

`Ok(name) ->` takes the value and `Err(name) ->` takes the error. `_` is any other case. It is the way to handle a
`Result` in a function that does not return `Result`:

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

greet(id: string) string
  match findUser(id)
    Ok(user) -> 'hi ${user.name}'
    Err(e) -> 'error: ${e}'
```

```typescript
export function greet(id: string): string {
  const findUserResult = findUser(id);
  if (findUserResult.isOk) {
    return `hi ${findUserResult.value.name}`;
  } else {
    return `error: ${findUserResult.error}`;
  }
}
```

As a value, with one expression per branch, a ternary comes out:

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

nameOf(id: string) string
  name = match findUser(id)
    Ok(user) -> user.name
    Err(e) -> 'unknown'
  name
```

```typescript
export function nameOf(id: string): string {
  const findUserResult = findUser(id);
  const name = findUserResult.isOk ? findUserResult.value.name : 'unknown';
  return name;
}
```

- As a value, the `match` needs the `Ok` and `Err` branches, or `_` (otherwise `N0103`).
- Each branch has one pattern: `Ok(x)`, `Ok(_)`, `Err(e)` or `_`. Another pattern is `N0111`.
- An async function gives its `Result` with `~`: `match ~loadUser(id)`. Without `~` it is `N0111`.

## Summary

| I want to... | I write |
|---|---|
| return success or error | `Ok(value)` / `Err(error)` in a `Result<E, T>` function |
| use the value and let the error go up by itself | `x = call()` inside a `Result` function |
| log the error and let it keep going up | `call().error(e => log(e))` |
| recover from the error | `call().error(e => defaultValue)` or `.error(e => Ok(defaultValue))` |
| change the error type | `call().error(e => Err(MyError(e)))` |
| transform the value | `call().ok(v => v.name)` or `.ok(v => Ok(somethingElse(v)))` |
| decide with both branches | `match call()` + `Ok(x) ->` / `Err(e) ->` |
| store the `Result` without propagating it | `r: Result<E, T> = call()` |
