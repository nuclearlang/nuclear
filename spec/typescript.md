# Dialect `typescript`

> Inherits every rule of [nuclear-base.md](nuclear-base.md). This document defines what the `typescript`
> dialect adds and how it translates each base rule. Files: `dialects/typescript/`. Inherited by
> [nodejs](nodejs.md) and [nestjs](https://github.com/nuclearlang/dragonfly-backend/blob/main/spec/nestjs.md).
>
> Section 4 is a list of Nuclear → TypeScript equivalences, in the order of the base. Each output is the one
> the transpiler produces.

## 1. Header words

- **Modifiers:** `abstract` `static` `readonly` `declare` `get` `set`.
- `get name() Type` + block → `get name(): Type { ... }`; `set name(value: T)` + block →
  `set name(value: T) { ... }` (a setter never returns).
- It defines no kinds nor clauses.

## 2. Translation of the base rules

| Base | TypeScript |
|---|---|
| Declaration | `export` before functions, classes, interfaces and enums |
| Variable that is not reassigned / that is reassigned | `const` / `let` |
| `x = if ...` with multi-line branches | `let x;` and an `if` that assigns in each branch |
| Members of the module | `this.` |
| Construct `User(...)` | `new User(...)` |
| Named arguments to a Nuclear function | reordered by position |
| Named arguments to an external function | an object: `fetch(url, { method: 'POST' })` |
| `{}` in the signature | no effect |
| Constructor parameters | `public` constructor parameters |
| Attribute without a value nor `?` | `name!: string` |
| `async` (with `~`) | `async` and the return `T` as `Promise<T>` |
| `name() = Module.method` (alias) | a method with `method`'s signature that returns `new Module().method(...)` (`Module.method(...)` if it is `static`) |
| Text | single quotes; with interpolation, backticks |
| `==` / `!=` | `===` / `!==` |
| `and` `or` `not` | `&&` `\|\|` `!` |
| `x is T` | `x instanceof T`; basic types: `x is string` → `typeof x === 'string'`, `x is int` → `Number.isInteger(x)`, `x is double` → `typeof x === 'number'`, `x is T[]` → `Array.isArray(x)` |
| `x as T` / `x!` | the same |
| `a in list` | `list.includes(a)`; a map literal or `<K, V>`: `key in map`; a `Map` object: `map.has(key)` |
| `for k, v in m` | `Object.entries(m)`; a `Map` object is iterated directly |
| `a ** b`, `a ~/ b` | `a ** b` (parenthesized under `-`: `-(a ** b)`), `Math.trunc(a / b)` |
| `& \| ^ ^x << >> >>>` | `& \| ^ ~x << >> >>>` (32-bit); a bitwise operand of a comparison gets parentheses |
| `(5).toString(2)` | the same (`int_member_parens`: `5.` would start a decimal) |
| Range `a..b` as a value | `Array.from({ length: b - a }, (_, i) => a + i)` |
| `for k, v in map` | `for (const [k, v] of Object.entries(map))` |
| `match` | an `if / else if / else` chain (or ternaries when it is the value of an expression) |
| `throw Error(...)` | `throw new Error(...)` |
| Constructor of an inheriting module | `super(...)` first; without it, `super();`. A parameter named like a member of the base is a plain parameter (`name: string`), not a new field |
| Method that overrides one of a base class of the file | `override` (not in interface methods) |
| `[a, ...rest] = xs` / `{ a, "k": b } = m` | `const [a, ...rest] = xs;` / `const { a, 'k': b } = m;` (`_` → hole) |
| Existing names in a pattern | `([a, b] = [b, a]);` |
| `Result<E, T>`, `Ok(x)`, `Err(e)` | class `Result<E, T>` and functions `Ok(x)` / `Err(e)` of the runtime (`templates/result.tpl`) |
| `x = call()` returning `Result`, in a `Result` function | `const xResult = call(); if (xResult.isErr) return Err(xResult.error); const x = xResult.value;` |
| `.ok(cb)` / `.error(cb)` that transform | `.andThen(cb)` / `.orElse(cb)` (`.map` / `.getOrElse` when the callback returns a plain value) |
| `.ok(cb)` / `.error(cb)` that observe | `.inspect(cb)` / `.inspectErr(cb)` |
| `match r` with `Ok(x)` / `Err(e)` | `if (r.isOk) { ... r.value ... } else { ... r.error ... }` |
| Function with `yield` | `function*` and return `Generator<T>`; with `~`, `async function*` and `AsyncGenerator<T>` |
| `yield ...other()` | `yield* other()` |
| `<T: Comparable>` | `<T extends Comparable>` |
| `[a, if cond b]` | `[a, ...(cond ? [b] : [])]` |
| Cascade `x = T()` + `..a = 1` | `const x = new T(); x.a = 1;`; inside an expression, `(($) => { $.a = 1; return $; })(new T())` |
| `print(x)` | `console.log(x)` |
| `now()` | `new Date()` |
| Types `int`, `double`, `number` | `number` |
| `<K, V>` | `Record<K, V>` |
| `T?` | `T \| null` |
| `any`, `dynamic` | `any` |
| Inheritance | a single base with `:`; interfaces go with `>` or are detected |

Global names that are not declared: `console`, `fetch`, `JSON`, `Math`, `Date`, `Promise`, `setTimeout`,
`parseInt`... (full list in `[names] globals` of `dialects/typescript/dialect.toml`).

## 3. Modules

| Nuclear | TypeScript |
|---|---|
| `use fs/promises{ readFile, writeFile }` | `import { readFile, writeFile } from 'fs/promises';` |
| `use * as path from path` | `import * as path from 'path';` |
| `use zod as z` | `import * as z from 'zod';` |
| `use './setup'` | `import './setup';` |

## 4. Equivalences

### Module with functions (base §1)

```nuclear
Users()
  addDoubles(a: double, b: double)
    a + b
  substractDoubles(a: double, b: double)
    a - b + addDoubles(a, b)
```

```typescript
export class Users {
  addDoubles(a: number, b: number) {
    return a + b;
  }

  substractDoubles(a: number, b: number) {
    return a - b + this.addDoubles(a, b);
  }
}
```

### Doc comments (base §2.3)

The comment above a declaration is written as JSDoc (`/** ... */`): tags pass as they are (`@return` → `@returns`) and a one-line description stays on one line.

```nuclear
// Price with tax, rounded to cents.
 * @param amount price without tax
 * @return the price with 16% tax
 * @example
 *   withTax(100.0)   // 116.0
withTax(amount: double) double
  amount * 1.16

Cart
  // Number of products.
  items: int = 0
```

```typescript
/**
 * Price with tax, rounded to cents.
 *
 * @param amount price without tax
 * @returns the price with 16% tax
 * @example
 * withTax(100.0)   // 116.0
 */
export function withTax(amount: number): number {
  return amount * 1.16;
}

export class Cart {
  /** Number of products. */
  items: number = 0;
}
```

### `if` inside a list (base §2.5)

```nuclear
children(loading: boolean, items: string[]) Widget[]
  [
    Text('Title'),
    if loading Spinner(),
    ...items.map(i => Text(i)),
  ]
```

```typescript
export function children(loading: boolean, items: string[]): Widget[] {
  return [
    new Text('Title'),
    ...(loading ? [new Spinner()] : []),
    ...items.map((i) => new Text(i)),
  ];
}
```

### Variables (base §3)

```nuclear
name = 'Ana'
age: number = 30
counter = 0
counter = counter + 1
counter += 1
```

```typescript
const name = 'Ana';
const age: number = 30;
let counter = 0;
counter = counter + 1;
counter += 1;
```

### `if` as a value (base §3)

```nuclear
label(age: int) string
  text = if age >= 18
    prefix = 'adu'
    prefix + 'lt'
  else
    'minor'
  text

shortLabel(age: int) string
  text = if age >= 18
    'adult'
  else
    'minor'
  text
```

```typescript
export function label(age: number): string {
  let text;
  if (age >= 18) {
    const prefix = 'adu';
    text = prefix + 'lt';
  } else {
    text = 'minor';
  }
  return text;
}

export function shortLabel(age: number): string {
  const text = age >= 18 ? 'adult' : 'minor';
  return text;
}
```

### Destructuring (base §3.1)

```nuclear
example(xs: int[], json: <string, any>, users: <string, string>[]) void
  [first, second] = xs
  [head, ...tail] = xs
  [_, other, ...rest] = xs
  { min, max } = bounds(xs)
  { "first-name": name, age } = json
  [{ email }, next] = users
  a = 1
  b = 2
  [a, b] = [b, a]
```

```typescript
export function example(
  xs: number[],
  json: Record<string, any>,
  users: Record<string, string>[],
): void {
  const [first, second] = xs;
  const [head, ...tail] = xs;
  const [, other, ...rest] = xs;
  const { min, max } = bounds(xs);
  const { 'first-name': name, age } = json;
  const [{ email }, next] = users;
  let a = 1;
  let b = 2;
  ([a, b] = [b, a]);
}
```

TypeScript does not check the number of elements of a list pattern.

### Results (base §3.2)

The runtime (`Result`, `Ok`, `Err`) is written once, after the imports, in each file that uses `Result`:
`dialects/typescript/templates/result.tpl`.

```nuclear
User(name: string)

findUser(id: string) Result<string, User>
  if id == ''
    return Err('empty id')
  Ok(User(id))

buy(id: string) Result<string, string>
  user = findUser(id)
  findUser('backup')
  Ok(user.name)
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

export function buy(id: string): Result<string, string> {
  const userResult = findUser(id);
  if (userResult.isErr) return Err(userResult.error);
  const user = userResult.value;
  const findUserResult = findUser('backup');
  if (findUserResult.isErr) return Err(findUserResult.error);
  return Ok(user.name);
}
```

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

export function checkStock(id: string): Result<string, number> {
  return Ok(id.length);
}

export function log(message: string): void {
  console.log(message);
}

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

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

show(id: string) string
  match findUser(id)
    Ok(user) -> 'hi ${user.name}'
    Err(e) -> 'error: ${e}'

short(id: string) string
  kept: Result<string, User> = findUser(id)
  label = match kept
    Ok(user) -> user.name
    Err(e) -> e
  label
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

export function show(id: string): string {
  const findUserResult = findUser(id);
  if (findUserResult.isOk) {
    return `hi ${findUserResult.value.name}`;
  } else {
    return `error: ${findUserResult.error}`;
  }
}

export function short(id: string): string {
  const kept: Result<string, User> = findUser(id);
  const label = kept.isOk ? kept.value.name : kept.error;
  return label;
}
```

### Bounded generics (base §4)

```nuclear
larger<T: Comparable>(a: T, b: T) T
  a.compareTo(b) > 0 ? a : b
```

```typescript
export function larger<T extends Comparable>(a: T, b: T): T {
  return a.compareTo(b) > 0 ? a : b;
}
```

### Functions and return (base §4.1)

```nuclear
greet(name: string) string
  'Hello ${name}'

add(a: number, b: number) number = a + b

total(a: number, b: number)
  sum = a + b

record(a: number, b: number) void
  sum = a + b
```

```typescript
export function greet(name: string): string {
  return `Hello ${name}`;
}

export function add(a: number, b: number): number {
  return a + b;
}

export function total(a: number, b: number) {
  const sum = a + b;
  return sum;
}

export function record(a: number, b: number): void {
  const sum = a + b;
}
```

### `async` and `~` (base §4.1)

```nuclear
greet(name: string) string
  "Hello ${name}"

welcome() string
  text = ~greet("world")
  text

~greetAsync(name: string) string
  "Hello ${name}"
```

```typescript
export function greet(name: string): string {
  return `Hello ${name}`;
}

export async function welcome(): Promise<string> {
  const text = await greet('world');
  return text;
}

export async function greetAsync(name: string): Promise<string> {
  return `Hello ${name}`;
}
```

### Generators (base §4.1)

```nuclear
count(limit: int) int
  for i in 0..limit
    yield i

all(limit: int) int
  yield 0
  yield ...count(limit)

ticks() int
  for i in 0..3
    ~wait(1)
    yield i
```

```typescript
export function* count(limit: number): Generator<number> {
  for (let i = 0; i < limit; i++) {
    yield i;
  }
}

export function* all(limit: number): Generator<number> {
  yield 0;
  yield* count(limit);
}

export async function* ticks(): AsyncGenerator<number> {
  for (let i = 0; i < 3; i++) {
    await wait(1);
    yield i;
  }
}
```

### Anonymous functions (base §4.1.1)

```nuclear
Network
  retry(attempt: int)
    log = (message: string)
      print(message)
    twice = (n: int) => n * 2
    connect()
```

```typescript
export class Network {
  retry(attempt: number) {
    const log = (message: string) => console.log(message);
    const twice = (n: number) => n * 2;
    return connect();
  }
}
```

### Function aliases (base §4.1.2)

```nuclear
Character(name: string)

GetOneCharacter
  exec(id: int) Character
    Character('${id}')

GetAllCharacters
  ~exec() Character[]
    [Character('all')]

CharactersComponent()
  getOneCharacter() = GetOneCharacter.exec
  getCharacters() = GetAllCharacters.exec

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

export class GetAllCharacters {
  async exec(): Promise<Character[]> {
    return [new Character('all')];
  }
}

export class CharactersComponent {
  getOneCharacter(id: number): Character {
    return new GetOneCharacter().exec(id);
  }

  async getCharacters(): Promise<Character[]> {
    return await new GetAllCharacters().exec();
  }

  first(): Character {
    return this.getOneCharacter(1);
  }
}
```

### Modules, attributes and constructor (base §4.2)

```nuclear
User()
  name: string
  email: string
  role: Role = Role.user
  token?: string

  ()
    name = "name assignment"

  isAdmin() boolean
    role == Role.admin

Admin
  permissions: string[] = []
```

```typescript
export class User {
  name!: string;
  email!: string;
  role: Role = Role.user;
  token?: string;

  constructor() {
    this.name = 'name assignment';
  }

  isAdmin(): boolean {
    return this.role === Role.admin;
  }
}

export class Admin {
  permissions: string[] = [];
}
```

Constructor parameters and own attributes:

```nuclear
User(name: string, email: string, role: string = "")
  localAttr: string = ""

  someMethod(myOwnParam: string)
    name = myOwnParam
    localAttr = myOwnParam
```

```typescript
export class User {
  localAttr: string = '';

  constructor(
    public name: string,
    public email: string,
    public role: string = '',
  ) {}

  someMethod(myOwnParam: string) {
    this.name = myOwnParam;
    this.localAttr = myOwnParam;
  }
}
```

Constructor with a body:

```nuclear
User
  (name: string, email: string, role: string = "", localAttr: string = "")
    localAttr = "other value"
```

```typescript
export class User {
  constructor(
    public name: string,
    public email: string,
    public role: string = '',
    public localAttr: string = '',
  ) {
    this.localAttr = 'other value';
  }
}
```

Parameters between `{}` (no effect in TypeScript):

```nuclear
User({name: string, email: string, role: string = ""})
  someMethod({myOwnParam: string})
    name = myOwnParam
```

```typescript
export class User {
  constructor(
    public name: string,
    public email: string,
    public role: string = '',
  ) {}

  someMethod(myOwnParam: string) {
    this.name = myOwnParam;
  }
}
```

`$` for the attribute:

```nuclear
User
  (
    name: string,
    email: string,
    role: string = ""
  )
    $localAttr = "other value"

  localAttr: string = ""

  someMethod(name: string)
    $name = name
```

```typescript
export class User {
  constructor(
    public name: string,
    public email: string,
    public role: string = '',
  ) {
    this.localAttr = 'other value';
  }

  localAttr: string = '';

  someMethod(name: string) {
    this.name = name;
  }
}
```

### Inheritance (base §4.2)

```nuclear
Serializable
  toJson() string

Admin: User > Serializable
  toJson() string
    "Say HI!"
```

```typescript
export interface Serializable {
  toJson(): string;
}

export class Admin extends User implements Serializable {
  toJson(): string {
    return 'Say HI!';
  }
}
```

Constructor of an inheriting module:

```nuclear
Animal
  (name: string)

Dog: Animal
  (name: string, breed: string)
    super(name)

  describe() string
    'dog'
```

```typescript
export class Animal {
  constructor(public name: string) {}
}

export class Dog extends Animal {
  constructor(name: string, public breed: string) {
    super(name);
  }

  describe(): string {
    return 'dog';
  }
}
```

### Interfaces and enums (base §4.3)

```nuclear
Message
  role: 'user' | 'assistant' | 'system'
  content: string
  name?: string
  some(name: string) void

Role
  user
  admin

Level
  low("Low")
  high("High")
  medium
```

```typescript
export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  name?: string;
  some(name: string): void;
}

export enum Role {
  user,
  admin
}

export enum Level {
  low = 'Low',
  high = 'High',
  medium = 'medium'
}
```

### Operators (base §6.1)

```nuclear
equal = a == b
both = active and not admin
isUser = x is User
data = read(file) as <string, int>
name = user!.name
```

```typescript
const equal = a === b;
const both = active && !admin;
const isUser = x instanceof User;
const data = read(file) as Record<string, number>;
const name = user!.name;
```

### Named arguments and construction (base §6.2, §6.3)

```nuclear
User(name: string, email: string)

createUser(name: string, admin: boolean = false) void
  print(name)

caller(url: string) void
  createUser(admin = true, name = 'Ana')
  fetch(url, method = 'POST')
  user = User(name = 'Ana', email = 'a@x.com')
```

```typescript
export class User {
  constructor(public name: string, public email: string) {}
}

export function createUser(name: string, admin: boolean = false): void {
  console.log(name);
}

export function caller(url: string): void {
  createUser('Ana', true);
  fetch(url, { method: 'POST' });
  const user = new User('Ana', 'a@x.com');
}
```

### Lambdas (base §6.4)

```nuclear
User(name: string, age: int = 0)

users = (1..10).map(iterator: int)
  User(name = "Julian")
active = usersRepo.get(active = true)
filtered = active.filter(user: User)
  user.age > 18
doubled = numbers.map(n => n * 2)
adults = users.filter((user: User) => user.age >= 18)
label = users.map(user: User)
  user.name.upper()
.join(', ')
```

```typescript
export class User {
  constructor(public name: string, public age: number = 0) {}
}

const users = Array.from({ length: 10 - 1 }, (_, i) => 1 + i).map((iterator: number) => new User('Julian'));
const active = usersRepo.get({ active: true });
const filtered = active.filter((user: User) => user.age > 18);
const doubled = numbers.map((n) => n * 2);
const adults = users.filter((user: User) => user.age >= 18);
const label = users.map((user: User) => user.name.toUpperCase()).join(', ');
```

Lambda with a block as an argument:

```nuclear
Form
  counter: int = 0

  render()
    Button(
      label = 'Save',
      onClick = ()
        counter += 1
        save(counter)
      disabled = false,
    )
```

```typescript
export class Form {
  counter: number = 0;

  render() {
    return new Button({
      label: 'Save',
      onClick: () => {
        this.counter += 1;
        return save(this.counter);
      },
      disabled: false,
    });
  }
}
```

### Cascades (base §6.5)

TypeScript has no cascades: in a declaration they are written as statements; inside an expression, as a
function that receives the object and returns it.

```nuclear
brushes() void
  brush = Paint()
    ..color = Colors.blue
    ..strokeWidth = 4
  draw(Paint()
    ..color = Colors.red)
```

```typescript
export function brushes(): void {
  const brush = new Paint();
  brush.color = Colors.blue;
  brush.strokeWidth = 4;
  draw((($) => {
    $.color = Colors.red;
    return $;
  })(new Paint()));
}
```

### Control flow (base §7)

```nuclear
if x > 0
  print('positive')
else if x < 0
  print('negative')
else
  print('zero')
for user in users
  print(user)
for i in 0..10
  print(i)
for key, value in config
  print(key)
while pending > 0
  pending -= 1
match event.kind
  'message' -> process(event)
  'error' ->
    print(event.error)
    process(event)
  _ -> print('other')
try
  process(event)
catch error
  print(error)
finally
  print('done')
throw Error('something went wrong')
```

```typescript
if (x > 0) {
  console.log('positive');
} else if (x < 0) {
  console.log('negative');
} else {
  console.log('zero');
}
for (const user of users) {
  console.log(user);
}
for (let i = 0; i < 10; i++) {
  console.log(i);
}
for (const [key, value] of Object.entries(config)) {
  console.log(key);
}
while (pending > 0) {
  pending -= 1;
}
if (event.kind === 'message') {
  process(event);
} else if (event.kind === 'error') {
  console.log(event.error);
  process(event);
} else {
  console.log('other');
}
try {
  process(event);
} catch (error) {
  console.log(error);
} finally {
  console.log('done');
}
throw new Error('something went wrong');
```

## 5. The standard library

[library.md](library.md) as `dialects/typescript/library.toml` writes it: lists are arrays, maps `<K, V>` are plain
objects (`Object.keys`), sets are `Set`, `Bytes` is `Uint8Array`, `DateTime` is `Date` (its parts are read in UTC:
`Date` has no UTC flag), `Duration` is a number of milliseconds, `Regex` is a `RegExp` with the `g` flag (`test`
uses `search`, `find` and `findAll` use `matchAll`, so no call depends on `lastIndex`), `Match` is a
`RegExpMatchArray`, `Error(message)` is `new Error(message)` and `e.message` is `(e as Error).message`. Members
without a one-line form are helpers written once per file (`_toInt`, `_remove`, `_min`, `_sortBy`, `_groupBy`,
`_round`: half away from zero, `_replaceWith`: a callback with a `Match`). `delay` is a `setTimeout` promise,
`awaitAll` is `Promise.all`.
