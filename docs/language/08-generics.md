# 8. Generics

A generic is a type completed when it is used: a `Box<T>` can hold a `string`, a `User`... Type parameters go
between `< >` right after the name, as in TypeScript.

## Generic modules

```nuclear
Box<T>
  (value: T)

  get() T
    value

Pair<K, V>
  (key: K, value: V)
```

```typescript
export class Box<T> {
  constructor(public value: T) {}

  get(): T {
    return this.value;
  }
}

export class Pair<K, V> {
  constructor(public key: K, public value: V) {}
}
```

```dart
class Box<T> {
  T value;

  Box(this.value);

  T get() {
    return value;
  }
}

class Pair<K, V> {
  K key;
  V value;

  Pair(this.key, this.value);
}
```

## Generic functions

```nuclear
first<T>(xs: T[]) T
  xs[0]
```

```typescript
export function first<T>(xs: T[]): T {
  return xs[0];
}
```

## Bounds: `<T: Base>`

`<T: Comparable>` requires `T` to extend `Comparable`. `:` means "extends", as in inheritance.

```nuclear
Comparable
  compareTo(other: Comparable) int

larger<T: Comparable>(a: T, b: T) T
  a.compareTo(b) > 0 ? a : b

Sorted<T: Comparable, U>
  items: T[] = []
  extra?: U
```

```typescript
export interface Comparable {
  compareTo(other: Comparable): number;
}

export function larger<T extends Comparable>(a: T, b: T): T {
  return a.compareTo(b) > 0 ? a : b;
}

export class Sorted<T extends Comparable, U> {
  items: T[] = [];
  extra?: U;
}
```

## Using a generic: `X<T>(...)`

When constructing an object or calling a generic function, the types go between `< >` **right after the name**,
just before the parentheses. With a space before `<` it is a comparison (`a < b`).

```nuclear
Box<T>
  (value: T)

run(repo: Repo) void
  box = Box<string>('hi')
  users = repo.findAll<User>(active = true)
  print(box.value)
```

```typescript
export function run(repo: Repo): void {
  const box = new Box<string>('hi');
  const users = repo.findAll<User>({ active: true });
  console.log(box.value);
}
```

## Generics in types

A generic type is written the same way in variables, parameters, attributes and returns: `Box<string>`,
`Pair<string, int>[]`, `Box<User>?`.
