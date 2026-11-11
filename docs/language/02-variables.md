# 2. Variables and assignments

## Declaring is assigning

There is no `var`, `let` nor `const` in Nuclear. A variable is created the first time a value is assigned
to it:

```nuclear
name = 'Ana'
age: int = 30
```

```typescript
const name = 'Ana';
const age: number = 30;
```

- `x = value` **declares** `x` if the name does not exist yet in the block nor in the blocks that contain
  it. If it already exists, it is a reassignment.
- The type is optional (`age: int = 30`). If it is not written, the target language infers it.

## Constant or mutable: the transpiler decides

If the variable is assigned again later in its block, it is emitted as mutable (`let` in TS, `var` in Dart).
If it is never reassigned, as a constant (`const` in TS, `final` in Dart).

```nuclear
total(prices: double[]) double
  tax = 0.16
  sum = 0.0
  for p in prices
    sum += p
  sum * (1 + tax)
```

```typescript
export function total(prices: number[]): number {
  const tax = 0.16;
  let sum = 0.0;
  for (const p of prices) {
    sum += p;
  }
  return sum * (1 + tax);
}
```

```dart
double total(List<double> prices) {
  final tax = 0.16;
  var sum = 0.0;
  for (final p in prices) {
    sum += p;
  }
  return sum * (1 + tax);
}
```

Compound assignments exist: `+=`, `-=`, `*=`, `/=`, `%=`, `??=`.

## Block scope

A variable lives from its declaration to the end of the block where it was created: the body of a function or
the block of an `if`, `else`, `for`, `while`, `match` branch, `try` or `catch`. Reading it outside is error
`N0105`:

```nuclear error
label(age: int) string
  if age >= 18
    text = 'adult'
  else
    text = 'minor'
  text              // N0105: text only exists inside each branch
```

Using a name nothing defines (no variable, parameter, member, declaration nor import) is error `N0106`.

A variable created **before** the block can be reassigned inside it:

```nuclear
count(xs: int[]) int
  positives = 0
  for x in xs
    if x > 0
      positives += 1
  positives
```

```typescript
export function count(xs: number[]): number {
  let positives = 0;
  for (const x of xs) {
    if (x > 0) {
      positives += 1;
    }
  }
  return positives;
}
```

## `if` and `match` are values

As in Elixir, an `if` or a `match` produces a value: the **last line of each branch**. To get a value out of
an `if`, assign the whole `if`:

```nuclear
label(age: int) string
  text = if age >= 18
    prefix = 'adul'
    prefix + 't'
  else
    'minor'
  text
```

```typescript
export function label(age: number): string {
  let text;
  if (age >= 18) {
    const prefix = 'adul';
    text = prefix + 't';
  } else {
    text = 'minor';
  }
  return text;
}
```

If each branch is a single expression, a ternary comes out:

```nuclear
size(n: int) string
  name = match n
    0 -> 'zero'
    1 | 2 -> 'small'
    _ -> 'big'
  name
```

```typescript
export function size(n: number): string {
  const name = n === 0 ? 'zero' : n === 1 || n === 2 ? 'small' : 'big';
  return name;
}
```

Rules:

- As the value of an assignment, an `if` needs `else` and a `match` needs the `_` branch.
- As a value, each branch may have several lines. Inside a larger expression (an argument, a sum), each
  branch is a single expression.
- If the `if` or the `match` is the last line of a function with a return type, each branch returns its last
  line (see [Control flow](10-control.md)).

## Destructuring

A **pattern** on the left of `=` takes parts of a list or a map.

### From lists

```nuclear
parts(xs: int[]) int
  [first, second] = xs
  [head, ...tail] = xs
  [_, b, ...rest] = xs
  first + second + head + tail.length + b + rest.length
```

```typescript
export function parts(xs: number[]): number {
  const [first, second] = xs;
  const [head, ...tail] = xs;
  const [, b, ...rest] = xs;
  return first + second + head + tail.length + b + rest.length;
}
```

- `[a, b] = xs`: the list has exactly those elements.
- `...rest` (always last): the rest of the list.
- `_` ignores an element.

### From maps

```nuclear
limits(m: <string, int>) int
  { min, max } = m
  { "first-name": name } = m
  min + max + name
```

```typescript
export function limits(m: Record<string, number>): number {
  const { min, max } = m;
  const { 'first-name': name } = m;
  return min + max + name;
}
```

- `{ min, max }` takes the keys `"min"` and `"max"`. The map may have more keys.
- `{ "key": variable }` stores a key in a variable with another name.
- Patterns can be nested: `[{ name }, second] = users`.

### New or existing names, not mixed

The names of a pattern are **all new** (a declaration) or **all existing** (an assignment). This is how two
variables are swapped:

```nuclear
swap() void
  a = 1
  b = 2
  [a, b] = [b, a]
  print(a)
```

```typescript
export function swap(): void {
  let a = 1;
  let b = 2;
  ([a, b] = [b, a]);
  console.log(a);
}
```

Mixing new names with existing ones is error `N0107`:

```nuclear error
f(xs: int[]) void
  a = 1
  [a, b] = xs       // N0107: a exists, b is new
```

### Returning several values

There are no tuples nor records: a function returns a map or a list and the caller destructures it.

```nuclear
bounds(xs: int[]) <string, int>
  {"min": xs[0], "max": xs[1]}

width(xs: int[]) int
  { min, max } = bounds(xs)
  max - min
```

```typescript
export function bounds(xs: number[]): Record<string, number> {
  return { 'min': xs[0], 'max': xs[1] };
}

export function width(xs: number[]): number {
  const { min, max } = bounds(xs);
  return max - min;
}
```
