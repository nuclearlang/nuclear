# Guide to the Nuclear base language

This guide explains, step by step and with examples, everything the Nuclear **base language** has today:
what works the same in every dialect. The formal rules are in
[spec/nuclear-base.md](../../spec/nuclear-base.md); here they are explained calmly and each example shows
what it generates.

The examples show the output in **TypeScript** (the `typescript` dialect) and, when it helps, also in
**Dart** (the `dart` dialect). Every output is the transpiler's real one: they are checked with
`tools/spec-check/check.py` (see [Checking this guide](#checking-this-guide)).

## Chapters

| # | Chapter | What it covers |
|---|---|---|
| 1 | [Files and syntax](01-syntax.md) | indentation, blocks, comments, names, literals, reserved words |
| 2 | [Variables and assignments](02-variables.md) | `x = value`, constant or mutable, block scope, `if`/`match` as a value, destructuring |
| 3 | [Types](03-types.md) | basic types, lists, maps, nullables, unions |
| 4 | [Functions](04-functions.md) | signature, implicit return, `void`, parameters, named arguments, async with `~`, generators, aliases (`f() = Module.method`) |
| 5 | [Lambdas](05-lambdas.md) | one-line lambdas, with a block, anonymous functions, chains after a block |
| 6 | [Modules and constructors](06-modules.md) | modules, attributes, methods, `$`, the constructor forms, constructing without `new`, modifiers, decorators |
| 7 | [Inheritance, interfaces and enums](07-inheritance.md) | `:` and `>`, `super(...)`, overriding, deduced interfaces, enums |
| 8 | [Generics](08-generics.md) | generic modules and functions, `<T: X>` bounds, `X<T>(...)` calls |
| 9 | [Expressions](09-expressions.md) | operators, null, `as`, `!`, `is`, `in`, ranges, lists and maps, cascades |
| 10 | [Control flow](10-control.md) | `if`, `match`, `for`, `while`, `try`, `throw` |
| 11 | [`use` and imports](11-use.md) | importing modules and files, auto-imports |
| 12 | [Frequent errors](12-errors.md) | the diagnostics that show up most and how to fix them |
| 13 | [Results](13-results.md) | `Result<E, T>`, `Ok`/`Err`, automatic error propagation, `.ok()`/`.error()`, `match` |
| 14 | [Operations](14-operations.md) | power, bits, raw text; math, text, regex, collections, types, bytes, dates, async in each target |

For a quick lookup, the [language reference](reference.md) sums everything up in tables
(`nuclear help reference`).

## Nuclear in one page

```nuclear
// An enum: only items
Role
  user
  admin("administrator")

// An interface: only attributes and signatures without a body
Named
  name: string

// A module (a class in the target) with the constructor in the header
User(name: string, role: Role = Role.user) > Named
  visits: int = 0

  isAdmin() boolean
    role == Role.admin

  visit() void
    visits += 1

// A function: the last line is returned
greet(user: User) string
  title = if user.isAdmin()
    'Admin'
  else
    'User'
  '${title} ${user.name}'

// automatic async: ~ is await
~loadName(url: string) string
  res = ~fetch(url)
  ~res.text()
```

```typescript
/** An enum: only items */
export enum Role {
  user = 'user',
  admin = 'administrator'
}

/** An interface: only attributes and signatures without a body */
export interface Named {
  name: string;
}

/** A module (a class in the target) with the constructor in the header */
export class User implements Named {
  visits: number = 0;

  constructor(public name: string, public role: Role = Role.user) {}

  isAdmin(): boolean {
    return this.role === Role.admin;
  }

  visit(): void {
    this.visits += 1;
  }
}

/** A function: the last line is returned */
export function greet(user: User): string {
  const title = user.isAdmin() ? 'Admin' : 'User';
  return `${title} ${user.name}`;
}

/** automatic async: ~ is await */
export async function loadName(url: string): Promise<string> {
  const res = await fetch(url);
  return await res.text();
}
```

What you do not write in Nuclear and the transpiler adds by itself: `;`, `{ }`, `return`, `new`, `this.`,
`async`, `const`/`let`/`final`/`var`, `export`, `class`, `interface`, `enum`.

## What does not exist yet

- Components (principle 6 of the spec): still to build. Multi-file projects do exist
  ([spec/projects.md](../../spec/projects.md)).
- Named or `factory` constructors (calling them works: `Character.fromJson(json)`).
- Operator overloading.
- Destructuring patterns in lambda parameters, in `for` or in `match` branches (only in assignments).
- Records: they do not exist by decision; use maps or lists.

## Checking this guide

Each `nuclear` block followed by a `typescript` block is transpiled and compared:

```sh
for f in docs/language/*.md; do tools/spec-check/check.py $f typescript typescript; done
```

The examples that show an error on purpose are marked `nuclear error` and are not transpiled.
