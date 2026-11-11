# 4. Functions

## Declaring a function

There is no `function` nor `def` keyword. A function is a **camelCase** name with its parameters, the return
type (optional) and the indented body:

```
name(parameters) ReturnType
  body
```

```nuclear
greet(name: string) string
  'Hello ${name}'
```

```typescript
export function greet(name: string): string {
  return `Hello ${name}`;
}
```

The return type goes **without `:`**, right after the parameters. `:` is for the types of parameters and for
inheritance.

## Implicit return

**The last line of the body is returned.** `return` exists only to leave early.

```nuclear
discount(price: double, vip: boolean) double
  if price <= 0
    return 0.0
  rate = vip ? 0.2 : 0.1
  price * (1 - rate)
```

```typescript
export function discount(price: number, vip: boolean): number {
  if (price <= 0) {
    return 0.0;
  }
  const rate = vip ? 0.2 : 0.1;
  return price * (1 - rate);
}
```

- If the last line is an **assignment**, the created variable is returned.
- Without a return type, the last line is also returned and the target infers the type.
- With return type **`void`**, nothing is returned. Use it when the function produces no value.

```nuclear
sum(a: int, b: int)
  total = a + b

log(message: string) void
  print('[app] ${message}')
```

```typescript
export function sum(a: number, b: number) {
  const total = a + b;
  return total;
}

export function log(message: string): void {
  console.log(`[app] ${message}`);
}
```

## One-line form

For single-expression bodies: `name(parameters) Type = expression`.

```nuclear
double(n: int) int = n * 2
```

```typescript
export function double(n: number): number {
  return n * 2;
}
```

## Parameters

| Form | Meaning |
|---|---|
| `name: string` | required |
| `name?: string` | optional (may not arrive) |
| `name: string = 'Ana'` | with a default value |
| `{name: string, age: int = 0}` | named (only in dialects whose target has them, like Dart) |

Optional ones and those with a default go at the end.

```nuclear
connect(host: string, port: int = 80, secure?: boolean) string
  '${host}:${port}'
```

```typescript
export function connect(host: string, port: number = 80, secure?: boolean): string {
  return `${host}:${port}`;
}
```

```dart
String connect(String host, [int port = 80, bool? secure]) {
  return '$host:$port';
}
```

A function receives **only** the parameters written in its signature. Without parentheses (`refresh` + block)
it receives none.

### Named parameters `{ }`

Braces in the signature are optional. In TypeScript they change nothing; in Dart they give named parameters
(`required` when they have no default):

```nuclear
box({width: int, height: int = 1}) int
  width * height
```

```typescript
export function box(width: number, height: number = 1): number {
  return width * height;
}
```

```dart
int box({required int width, int height = 1}) {
  return width * height;
}
```

## Calling a function: by position and by name

Inside the parentheses, `:` and `=` have a fixed role:

| Where | Form | Meaning |
|---|---|---|
| signature | `get(user: User)` | `:` gives the parameter's type |
| call by position | `get(u)` | the argument goes by order |
| call by name | `get(user = u)` | `=` assigns the argument to the `user` parameter |

Positional arguments go first and then the named ones. If the function is written in Nuclear, the transpiler
reorders them:

```nuclear
create(name: string, admin: boolean = false, active: boolean = true) string
  name

run() void
  create('Ana', active = false)
  create(admin = true, name = 'Luis')
```

```typescript
export function create(name: string, admin: boolean = false, active: boolean = true): string {
  return name;
}

export function run(): void {
  create('Ana', false, false);
  create('Luis', true);
}
```

If the function is **external** (from the target's library), each dialect decides: TypeScript groups the named
arguments in an object and Dart emits them as named arguments.

```nuclear
send(url: string) void
  fetch(url, method = 'POST')
```

```typescript
export function send(url: string): void {
  fetch(url, { method: 'POST' });
}
```

## Async: `~` is await

`async` and `await` are not written. Write `~` before the call to wait for, and **the function that uses `~`
becomes async by itself**. The return type is written as the final value (`string`) and the dialect wraps it
(`Promise<string>`, `Future<String>`).

```nuclear
loadText(url: string) string
  res = ~fetch(url)
  ~res.text()
```

```typescript
export async function loadText(url: string): Promise<string> {
  const res = await fetch(url);
  return await res.text();
}
```

- `~` applies to the whole chain that follows: `~api.users.get()` waits for the result of `api.users.get()`.
- For a function to be async even if it does not use `~`, write `~` before its name:

```nuclear
~ping() string
  'pong'
```

```typescript
export async function ping(): Promise<string> {
  return 'pong';
}
```

## Generators: `yield`

A function that uses `yield` is a **generator**: it yields values one by one. The return type is the type of
**each value** and the function does not return its last line. `yield ...other()` yields every value of another
generator. If it also uses `~`, it is an asynchronous generator.

```nuclear
countTo(limit: int) int
  for i in 0..limit
    yield i

all(limit: int) int
  yield 0
  yield ...countTo(limit)
```

```typescript
export function* countTo(limit: number): Generator<number> {
  for (let i = 0; i < limit; i++) {
    yield i;
  }
}

export function* all(limit: number): Generator<number> {
  yield 0;
  yield* countTo(limit);
}
```

```dart
Iterable<int> countTo(int limit) sync* {
  for (var i = 0; i < limit; i++) {
    yield i;
  }
}

Iterable<int> all(int limit) sync* {
  yield 0;
  yield* countTo(limit);
}
```

## Aliases: a function that is another one

`name() = Module.method` makes `name` **the same function** as `method`, with the same parameters, the same
return and the same async. It lets a module expose what another one does (for example, a use case) without
writing its signature again:

```nuclear
Character(name: string)

GetOneCharacter
  exec(id: int) Character
    Character('${id}')

CharactersComponent()
  getOneCharacter() = GetOneCharacter.exec

  first() Character
    getOneCharacter(1)
```

```typescript
export class Character {
  constructor(public name: string) {}
}

export class GetOneCharacter {
  exec(id: number): Character {
    return new Character(`${id}`);
  }
}

export class CharactersComponent {
  getOneCharacter(id: number): Character {
    return new GetOneCharacter().exec(id);
  }

  first(): Character {
    return this.getOneCharacter(1);
  }
}
```

```dart
class Character {
  String name;

  Character(this.name);
}

class GetOneCharacter {
  Character exec(int id) {
    return Character('$id');
  }
}

class CharactersComponent {
  Character getOneCharacter(int id) {
    return GetOneCharacter().exec(id);
  }

  Character first() {
    return getOneCharacter(1);
  }
}
```

- `getOneCharacter()` is written with empty `()`, but it receives `id` because it copies the parameters of `exec`.
- If `method` is async, the alias is too. If it returns `Result`, its errors propagate the same way
  ([chapter 13](13-results.md)). If it is a generator, the alias yields its same values.
- A `static` method is called on the module (`Module.method(id)`). Any other is called on a new `Module()` on
  each call, so `Module`'s constructor cannot have required parameters (error `N0112`).
- It is an alias only if the right side is `Module.method`, not called, and `Module` is in the same file.
  `role() = Role.admin` is still a one-line function that returns `Role.admin`.

## Functions inside functions

Functions are not declared inside a body. A local function is a **lambda assigned to a name**
(see [Lambdas](05-lambdas.md)).
