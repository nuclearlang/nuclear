# Dialect `typescript` brief

The library is the platform's: `Math`, `JSON`, `Date`, `Promise`, `Map`, `Set`, `RegExp`, `fetch`... A
`PascalCase(...)` call of a class is written with `new`. Output passes `tsc --strict`.

| Nuclear | TypeScript |
|---|---|
| `x = 1` (not reassigned / reassigned) | `const x = 1;` / `let x = 1;` |
| `f(a, x = 1)` to an external function | `f(a, { x: 1 })` |
| `int`, `double`, `number` | `number` |
| `x is string`, `x is int`, `x is int[]`, `x is User` | `typeof x === 'string'`, `Number.isInteger(x)`, `Array.isArray(x)`, `x instanceof User` |
| `x in list`, `key in map`, `key in m` (a `Map`) | `list.includes(x)`, `key in map`, `m.has(key)` |
| `a ~/ b` | `Math.trunc(a / b)` |
| `print(a, b)` | `console.log(a, b)` |
| top-level declarations | exported (`export function`, `export class`) |

```nuclear
Point(x: number, y: number)
  distance(other: Point) number
    Math.hypot(x - other.x, y - other.y)

loadUser(id: int) string
  response = ~fetch("/users/${id}")
  json = ~response.json()
  json.name ?? "unknown"
```

```typescript
export class Point {
  constructor(public x: number, public y: number) {}

  distance(other: Point): number {
    return Math.hypot(this.x - other.x, this.y - other.y);
  }
}

export async function loadUser(id: number): Promise<string> {
  const response = await fetch(`/users/${id}`);
  const json = await response.json();
  return json.name ?? 'unknown';
}
```

Bitwise operators work on 32-bit integers (JavaScript); use `BigInt(...)` for 64 bits. `%` keeps the dividend's sign.
