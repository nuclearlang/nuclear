# 1. Files and syntax

## Files

- Extension `.nuc`, UTF-8 encoding.
- Indentation of **2 spaces**. A tab or an indentation that is not a multiple of 2 is an error (`N0005`).

## Blocks by indentation

There are no `{ }` nor `;`. A block starts when the next line is more indented and ends when the previous
level returns:

```nuclear
check(age: int) void
  if age >= 18
    print('adult')
  else
    print('minor')
```

```typescript
export function check(age: number): void {
  if (age >= 18) {
    console.log('adult');
  } else {
    console.log('minor');
  }
}
```

### When indentation does not open a block

1. **Inside open `( )`, `[ ]` or `{ }`** indentation is ignored: a call or a list can be split over several
   lines.
2. **A line that starts with `.`** continues the previous one (a method chain).
3. **A line that starts with `..`** is a cascade section (see [Expressions](09-expressions.md)).

```nuclear
names(users: User[]) string
  users
    .filter(u => u.active)
    .map(u => u.name)
    .join(', ')
```

```typescript
export function names(users: User[]): string {
  return users.filter((u) => u.active).map((u) => u.name).join(', ');
}
```

The exception to rule 1 is the block of a lambda inside parentheses (see [Lambdas](05-lambdas.md)).

## Comments

`//` starts a comment. To continue it on another line, the line starts with ` * `: the `*` goes right below the
second `/` of the `//`, as in a JavaScript or Java `/** */` comment. That way it stands apart from the code at a
glance. In VS Code, pressing Enter inside a comment is enough: the new line already has its ` * `; a second Enter
on an empty ` * ` line ends the comment. Comments stay in place in the generated code.

A line more indented than the `//` without ` * ` is error `N0021`, with its fix.

```nuclear
// Tax calculations:
 * every function uses
 * the 16% VAT

tax(x: double) double
  x * 0.16
```

```typescript
// Tax calculations:
// every function uses
// the 16% VAT

export function tax(x: number): number {
  return x * 0.16;
}
```

### Doc comments

A comment **attached** to a declaration (on the line right above, without a blank line) is its documentation.
The editor shows it when hovering the name, in the completion list and, while you write a call, next to each
argument. In the generated code it comes out as documentation of the target: JSDoc in TypeScript, `///` in Dart.

As in JSDoc or JavaDoc, a line that starts with `@` opens a tag:

| Tag | For |
|---|---|
| `@param name text` | explaining a parameter (the type is already in the signature) |
| `@return text` | what it returns |
| `@throws text` | an error it can throw |
| `@example` | an example: the lines below, with more space after the `*`, are Nuclear code |
| `@see text` | something related |
| `@deprecated reason` | warning that it should not be used anymore (the editor strikes it through) |
| `@since version` | since which version it exists |

```nuclear
// Price with tax, rounded to cents.
 * @param amount price without tax
 * @return the price with 16% tax
 * @example
 *   withTax(100.0)   // 116.0
withTax(amount: double) double
  amount * 1.16
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
```

When hovering `withTax`, the editor shows:

```text
withTax(amount: double) double
function
──────────────────────────────────────
Price with tax, rounded to cents.

Parameters
 • amount — price without tax

Returns the price with 16% tax

Example
  withTax(100.0)   // 116.0
```

The words of the language and of each dialect (`if`, `match`, `stateless`, `@screen`, `static`...) also have
their documentation in the editor, with examples: just hover them.

## Names

The shape of a name says what it is. It is mandatory:

| Shape | What for | Example |
|---|---|---|
| `PascalCase` | modules (classes), interfaces, enums, types | `User`, `HttpClient` |
| `camelCase` | variables, functions, methods, attributes | `userName`, `getProfile` |

That is why **calling a PascalCase name constructs an object** (`User('Ana')`), without `new`.

## Literals

| Type | Example |
|---|---|
| integer | `42`, `1_000_000`, `0xff` |
| decimal | `3.14` |
| text | `'hello'` or `"hello"` (they are the same) |
| interpolation | `'Hello ${name}'` |
| multi-line text | `"""` ... `"""` or `'''` ... `'''` (removes the common indentation) |
| boolean | `true`, `false` |
| null | `null` |
| list | `[1, 2, 3]` |
| map | `{"name": "Ana", "age": 30}` |

```nuclear
prompt(name: string) string
  """
  You are an assistant.
  The user's name is ${name}.
  """
```

```typescript
export function prompt(name: string): string {
  return `You are an assistant.\nThe user's name is ${name}.`;
}
```

## Reserved words

```
use from as
if else for in while match return break continue yield
try catch finally throw
and or not is
true false null this super
```

There is no `import`, `export`, `var`, `const`, `let`, `new`, `class`, `interface`, `enum`, `async`, `await`,
`private`, `public`, `protected` nor `type`. Dialects add no reserved words: their words (like `abstract`,
`static`, `stateless`) only mean something before the name of a declaration.

If one of your names is a reserved word of the target language (`default`, `function`...), the transpiler
renames it by itself. The list is in [spec/reserved-words.md](../../spec/reserved-words.md).

## Formatting

`nuclear fmt <file|folder>` writes every file in the same layout: 2 spaces per block, one space after `,` and `:`,
spaces around operators, `{ "id": id }`, at most one blank line. It only moves spaces and line breaks: it never
joins, splits or reorders lines, and it refuses to touch a file if the result would not mean exactly the same.
`nuclear fmt --check .` lists the files that are not formatted (for CI).
