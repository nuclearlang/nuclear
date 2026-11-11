# 9. Expressions

## Operators

From lowest to highest precedence:

| Level | Operators |
|---|---|
| 1 | `c ? a : b` |
| 2 | `??` |
| 3 | `or` |
| 4 | `and` |
| 5 | `not` |
| 6 | `==` `!=` `<` `>` `<=` `>=` `is` `in` `as` |
| 7 | `..` (range) |
| 8 | `+` `-` |
| 9 | `*` `/` `%` |
| 10 | `-x` (unary), `~x` (await) |
| 11 | `a.b` `a?.b` `a(...)` `a<T>(...)` `a[i]` `a!` |

- `==` is always a strict comparison. There is no `===`.
- Logic with words: `and`, `or`, `not`. `&&`, `||` and `!` before a value are error `N0009`.

```nuclear
canEdit(user: User, roles: string[]) boolean
  not user.banned and (user.admin or 'editor' in roles)
```

```typescript
export function canEdit(user: User, roles: string[]): boolean {
  return !user.banned && (user.admin || roles.includes('editor'));
}
```

## Types and null

| Expression | Meaning |
|---|---|
| `x is User` | is `x` of type `User`? |
| `x as Type` | treats `x` as `Type` |
| `a ?? b` | `a`, or `b` if `a` is null |
| `a?.b` | `b` only if `a` is not null |
| `a!` | asserts that `a` is not null (right after a name, `)` or `]`) |

```nuclear
info(u: User?, data: Object) string
  name = u?.name ?? 'nobody'
  city = u!.address!.city
  scores = data as <string, int>
  isUser = u is User
  '${name} ${city}'
```

```typescript
export function info(u: User | null, data: Object): string {
  const name = u?.name ?? 'nobody';
  const city = u!.address!.city;
  const scores = data as Record<string, number>;
  const isUser = u instanceof User;
  return `${name} ${city}`;
}
```

## Ranges

`a..b` goes from `a` to `b - 1`. It is used in `for` and, between parentheses, with methods: `(0..10).map(...)`.
Writing `0..10.map(...)` is an error.

## Lists and maps

```nuclear
build(loading: boolean, xs: string[]) string[]
  base = {"a": 1}
  full = {...base, "b": 2}
  ['start', if loading 'loading', ...xs, 'end']
```

```typescript
export function build(loading: boolean, xs: string[]): string[] {
  const base = { 'a': 1 };
  const full = { ...base, 'b': 2 };
  return ['start', ...(loading ? ['loading'] : []), ...xs, 'end'];
}
```

- `...xs` (spread) puts in every element of another list or map.
- `if cond value` inside a list includes `value` only if `cond` holds. It has no `else` (to choose between two
  values: `cond ? a : b`).
- There is no `for` inside lists: use `...items.map(i => ...)`.
- Shorthand in objects: `{ model, messages }` is `{ model: model, messages: messages }`.

## Cascades: `..`

The lines that start with `..` below an expression operate on the **same object**, and the value of the whole
expression is that object:

```nuclear
make() void
  sb = StringBuffer()
    ..write('a')
    ..write('b')
  print(sb)
```

```typescript
export function make(): void {
  const sb = new StringBuffer();
  sb.write('a');
  sb.write('b');
  console.log(sb);
}
```

```dart
void make() {
  final sb = StringBuffer()
    ..write('a')
    ..write('b');
  print(sb);
}
```

- Each `..` line assigns a member (`..color = Colors.blue`) or calls a method.
- A `.` line below a section continues that section (`..items.add(x)`).
- `..` with something on its left on the same line is a range (`1..10`); at the start of a line it is a cascade.
