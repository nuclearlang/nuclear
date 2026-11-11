# 10. Control flow

No condition takes parentheses nor `{ }`: the block is the indentation.

## `if` / `else if` / `else`

```nuclear
sign(x: int) string
  if x > 0
    'positive'
  else if x < 0
    'negative'
  else
    'zero'
```

```typescript
export function sign(x: number): string {
  if (x > 0) {
    return 'positive';
  } else if (x < 0) {
    return 'negative';
  } else {
    return 'zero';
  }
}
```

Since the `if` is the last line of a function with a return type, each branch returns its last line. To assign
the result of an `if` to a variable, see [Variables](02-variables.md#if-and-match-are-values).

## `match`

Compares a value with several alternatives. `|` joins alternatives and `_` is "any other". A branch can be one
line (`value -> expression`) or have a block:

```nuclear
handle(kind: string) string
  match kind
    'message' -> 'got a message'
    'error' | 'fatal' ->
      print(kind)
      'failed'
    _ -> 'ignored'
```

```typescript
export function handle(kind: string): string {
  if (kind === 'message') {
    return 'got a message';
  } else if (kind === 'error' || kind === 'fatal') {
    console.log(kind);
    return 'failed';
  } else {
    return 'ignored';
  }
}
```

## `for`

```nuclear
loops(users: User[], config: <string, int>) void
  for user in users
    print(user.name)
  for i in 0..3
    print(i)
  for key, value in config
    print('${key}=${value}')
```

```typescript
export function loops(users: User[], config: Record<string, number>): void {
  for (const user of users) {
    console.log(user.name);
  }
  for (let i = 0; i < 3; i++) {
    console.log(i);
  }
  for (const [key, value] of Object.entries(config)) {
    console.log(`${key}=${value}`);
  }
}
```

- `for x in list`: each element.
- `for i in a..b`: from `a` to `b - 1`.
- `for key, value in map`: each entry of a map.
- `break` leaves the loop and `continue` goes to the next iteration.

## `while`

```nuclear
drain(pending: int) void
  left = pending
  while left > 0
    left -= 1
```

```typescript
export function drain(pending: number): void {
  let left = pending;
  while (left > 0) {
    left -= 1;
  }
}
```

## `try` / `catch` / `finally` y `throw`

```nuclear
load(url: string) string
  text = ''
  try
    res = ~fetch(url)
    text = ~res.text()
  catch error
    print(error)
    throw Error('load failed')
  finally
    print('done')
  text
```

```typescript
export async function load(url: string): Promise<string> {
  let text = '';
  try {
    const res = await fetch(url);
    text = await res.text();
  } catch (error) {
    console.log(error);
    throw new Error('load failed');
  } finally {
    console.log('done');
  }
  return text;
}
```

- `catch error` gives a name to the caught error.
- `throw` throws an object (`Error('...')`, without `new`).
- A `try` is not a value: to get a result out, create the variable before the `try` and reassign it inside, as
  in the example.
