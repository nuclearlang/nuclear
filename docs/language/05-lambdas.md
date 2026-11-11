# 5. Lambdas

A lambda is a function without a name passed as a value (to `map`, `filter`, an `onClick`...). There are two
forms: one line and with a block.

## One line: `=>`

For a single-expression body:

```nuclear
run(numbers: int[], users: User[]) void
  doubles = numbers.map(n => n * 2)
  adults = users.filter((u: User) => u.age >= 18)
  print(adults.length + doubles.length)
```

```typescript
export function run(numbers: number[], users: User[]): void {
  const doubles = numbers.map((n) => n * 2);
  const adults = users.filter((u: User) => u.age >= 18);
  console.log(adults.length + doubles.length);
}
```

A one-line lambda **cannot assign**: `(v) => email = v` is error `N0013`. The assignment goes in a block.

## With a block

The parameters go between parentheses and the body in the indented block of the next line. No `=>`, `return`
nor `{ }` is written: the last line of the block is the value the lambda returns.

```
list.function(parameter: Type)
  body
  lastLine           // returned value
```

```nuclear
evens(xs: int[]) int[]
  xs.filter(x: int)
    rest = x % 2
    rest == 0
```

```typescript
export function evens(xs: number[]): number[] {
  return xs.filter((x: number) => {
    const rest = x % 2;
    return rest === 0;
  });
}
```

A range followed by a method goes between parentheses: `(1..10).map(...)`.

```nuclear
squares() int[]
  (1..5).map(i: int)
    i * i
```

```typescript
export function squares(): number[] {
  return Array.from({ length: 5 - 1 }, (_, i) => 1 + i).map((i: number) => i * i);
}
```

## Lambda with a block as an argument

Inside a call's parentheses, `(parameters)` followed by a more indented block is a lambda. The block ends when
the indentation returns to the line where it started, and the `,` before the next argument is optional.

```nuclear
Form
  count: int = 0

  render() void
    button(
      label = 'Save',
      onClick = ()
        count += 1
        print(count)
      disabled = false,
    )
```

```typescript
export class Form {
  count: number = 0;

  render(): void {
    button({
      label: 'Save',
      onClick: () => {
        this.count += 1;
        return console.log(this.count);
      },
      disabled: false,
    });
  }
}
```

## Continuing the chain after a block

A line that starts with `.` at the statement's level continues the chain after the block:

```nuclear
names(users: User[]) string
  users.map(u: User)
    u.name.upper()
  .join(', ')
```

```typescript
export function names(users: User[]): string {
  return users.map((u: User) => u.name.toUpperCase()).join(', ');
}
```

## Local functions

Inside a body, a local function is a lambda assigned to a name:

```nuclear
retry(attempts: int) void
  log = (message: string)
    print('[retry] ${message}')
  twice = (n: int) => n * 2
  log('max ${twice(attempts)}')
```

```typescript
export function retry(attempts: number): void {
  const log = (message: string) => console.log(`[retry] ${message}`);
  const twice = (n: number) => n * 2;
  log(`max ${twice(attempts)}`);
}
```

- With a return type: `format = (n: int) string` + block.
- Inside a body, `name(parameters)` followed by a block is always a **call** that receives a lambda, never a
  declaration.
