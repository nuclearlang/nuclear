# NuclearLang — Base language v0.1 (draft)

> This document defines the **base language**: the syntax and rules every dialect shares. Each dialect
> has its own spec (`spec/<dialect>.md`) with what it adds and the equivalence, in its target language, of
> each example of this document.

## 1. Principles

**Golden rule:** performance, speed and readability of the syntax. It holds for the base language and for
every dialect: a new rule, shortcut or kind is accepted only if it respects the three. The generated code
must run as fast as what an expert in the target would write, transpiling must be immediate and Nuclear
code must read effortlessly.

1. **The easiest to write.** No `;`, no `{ }` for blocks, no mandatory `return`, no `new`, no mandatory
   `this.`, no manual `async`.
2. **One single way to do each thing.** An agent must never hesitate between two equivalent syntaxes.
   `nuclear fmt` always produces the same result (canonical format).
3. **Familiar before novel.** If TypeScript already has a good syntax (types, `import`, `=>`, decorators,
   generics), we keep it. Agents already know it.
4. **The parser never changes for a dialect.** Dialects add *meaning*, not *grammar*. That is why a
   declaration with a dialect word (`<kind> Name`) can be parsed without knowing anything about the dialect
   (see §4).
5. **Made for agents.** Errors in JSON with a code, a position and a suggested fix; the whole spec fits in
   an LLM's context; deterministic output.
6. **Not object oriented.** It has components, modules and methods. A component groups business modules
   (similar to Elixir modules), for example: a `users` component with the modules profile, auth, sessions,
   etc., business related to users, and the component has an application component as its architecture:
   the other modules or components cannot reach `users` except through the application component of users.
   If another component such as `sales` wants a feature of users such as profile, to get a user's data, it
   must go, if and only if, through the application component: `users.profile.getProfile()` or simply
   `users.getProfile()`. The transpiler must be able to check whether `getProfile` exists in one of the
   modules of users to get it; it works a lot with inference.

7. Functions are inferred, like modules: there is no need to write `module users...` or `func getUsers`;
   we know that if the first level is a module, the next level at that position is a function.

ex:

```nuclear
Users()
  addDoubles(a: double, b: double)
    a + b
  substracDoubles(a, b)
    a - b + addDoubles(a, b)
```

## 2. Lexicon

### 2.1 Files
- Extension `.nuc`. UTF-8 encoding.
- Indentation: **2 spaces**. Tabs are an error.

### 2.2 Blocks by indentation
A block starts when a line ends and the next one is more indented. It ends when the previous level returns.

```nuclear
if age >= 18
  print('adult')
else
  print('minor')
```

**Line continuation** (indentation is ignored) in two cases:
- Inside open `( )`, `[ ]`, `{ }`.
- A line that starts with `.` continues the previous one (method chains), and one that starts with `..`
  is a cascade section (§6.5).

```nuclear
app = express()
  .use(cors())
  .listen(3000)
```

Exception: after `=>` followed by a line break, a block opens even inside parentheses (multi-line lambdas,
see §6.4).

**Canonical layout** (what `nuclear fmt` writes): 2 spaces per block; one space after `,` and `:` and around
binary operators and `=`; none inside `( )` and `[ ]`, one inside `{ }` (`{ "id": id }`); at most one blank line,
and one before a top-level declaration that follows a block; no trailing spaces. Lines are never joined, split or
reordered.

### 2.3 Comments

`//` starts a comment. A comment continues on the lines that start with ` * `: the `*` goes **right below
the second `/`** (one column after the `//`) and the text starts after `* `. A lone `*` is an empty line of
the comment. A `//` after code runs to the end of the line.

```nuclear
// a multi-line comment:
 * this line continues the comment
 *
 * and so does this one, after an empty line
total = 0   // comment at the end of a line

User
  // also inside a block:
   * the * stays below the second /
  name: string
```

```
  // title
   * text         ← the * in the column of the second /
   *   code       ← the indentation after "* " is kept
```

A line more indented than the `//` that does not have that shape (no `*`, or the `*` in another column) is
error `N0021`; the fix rewrites it with ` * `.

#### Doc comments

The comment that ends on the line **right above** a declaration (function, module, method, attribute,
constructor or enum item) is its **documentation**: the editor shows it on hover and in completion, and the
dialect writes it as documentation of the target. With a blank line between them it is a normal comment.
Several consecutive `//` lines form a single comment.

A line that starts with `@tag` (after `* `) opens a tag, as in JSDoc or JavaDoc; the following lines
continue it:

| Tag | Documents |
|---|---|
| `@param name text` | a parameter |
| `@return text` (or `@returns`) | what it returns |
| `@throws text` | an error it can throw |
| `@example [title]` | an example: the lines below with more space after the `*` than `@example` (` *   code`) are Nuclear code and keep their shape |
| `@see text` | a related declaration or link |
| `@deprecated [reason]` | the declaration should not be used anymore |
| `@since version` | the version that added it |

What comes before the first tag is the description. A tag that is not in the table is kept as it is. A
parameter's type is not written in `@param`: it comes from the signature.

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

### 2.4 Identifiers and mandatory convention
- `PascalCase` → types, classes, interfaces, enums. **Calling a PascalCase name constructs an object**
  (`User(name = 'Ana')`, without `new`).
- `camelCase` → variables, functions, fields.
- The compiler gives an error if it is not respected. This removes ambiguity for the transpiler and for the
  agent.

### 2.5 Literals
| Type | Example |
|---|---|
| integer | `42`, `1_000_000`, `0xff`, `0b1010` |
| decimal | `3.14`, `1.5e-3`, `2e10` |
| text | `'hello'` or `"hello"` (they work the same) |
| raw text | `r"\d+\s"`, `r'C:\temp'`: no escapes and no interpolation (regular expressions, paths) |
| interpolation | `'Hello ${name}'` or `"Hello ${name}"` |
| multi-line text | `"""` ... `"""` or `'''` ... `'''` (removes the common indentation; ideal for prompts) |
| boolean | `true`, `false` |
| null | `null` |
| list | `[1, 2, 3]`, spread `[...others, 4]`, conditional `[a, if cond b]` |
| object or map | `{ role: 'user', content: text }`, shorthand `{ text }`, spread `{ ...base, a: 1 }` |

**`if` inside a list:** `if cond value` includes `value` only when `cond` holds. It has no `else` (to choose
between two values use `cond ? a : b`). To walk another list use a spread: `...items.map(i => Text(i))`.

```nuclear
children = [
  Text('Title'),
  if loading Spinner(),
  ...items.map(i => Text(i)),
]
```

### 2.6 Reserved words of the base
```
use from as
if else for in while match return break continue yield
try catch finally throw
and or not is
true false null this super
```
There is no `import`, `export`, `var`, `private`, `class`, `interface`, `enum` nor `await`
(see [decisions.md](decisions.md)).
The full list is in [languages/nuclear.toml](../languages/nuclear.toml).

Dialects add no reserved words to the parser. Their words (kinds, modifiers and clauses) only mean
something in the header of a declaration (§4) and can never be named like a reserved word of the base. Each
target language has its own dictionary of reserved words with the path from Nuclear to each one: see
[reserved-words.md](reserved-words.md). The dialect format is in [dialects.md](dialects.md).

## 3. Variables

```nuclear
name = 'Ana'              // not reassigned → a constant in the target
age: number = 30          // with a type
counter = 0
counter = counter + 1     // reassignment → counter is mutable
counter += 1
```

Rule: `x = value` **declares** `x` if it does not exist in the block nor in the blocks that contain it. If
`x` is assigned again later in its block, the transpiler emits it as a mutable variable in the target; if it
is never reassigned, it emits it as a constant. There is no `var` in Nuclear.

Rule: the transpiler optimizes as much as possible for the target language.

**Block scope.** A variable exists from its declaration to the end of the block where it was declared
(function body, `if`, `else`, `for`, `while`, `match` branch, `try`, `catch`). Using it outside that block is
error `N0105`. Using a name nothing defines (variable, parameter, member, declaration or import) is error
`N0106`.

**`if` and `match` are values**, as in Elixir: the value of each branch is its last line. To get a value
out of an `if` or a `match`, assign the whole `if`:

```nuclear
// error N0105: text only exists inside each branch
if age >= 18
  text = 'adult'
else
  text = 'minor'
print(text)

// correct
text = if age >= 18
  prefix = 'adu'
  prefix + 'lt'
else
  'minor'
print(text)
```

- As the value of an assignment or a `return`, each branch may have several lines; an `if` needs `else` and
  a `match` needs the `_` branch.
- Inside a larger expression (an argument, an operation), each branch is a single expression.

### 3.1 Destructuring

A pattern on the left of `=` takes parts of a list or a map:

```nuclear
[first, second] = xs                 // exactly two elements
[head, ...tail] = xs                 // the first one and the rest
[_, second, ...rest] = xs            // _ ignores an element
{ min, max } = bounds(xs)            // the keys "min" and "max" of the map
{ "first-name": name, age } = json   // a text key into another variable
[{ email }, second] = users          // nested patterns
```

- A list pattern has exactly those elements; with `...rest` (always last), at least those.
- A map pattern takes the named keys; the map may have others.
- The names of a pattern are all new (a declaration) or all existing (an assignment: `[a, b] = [b, a]`).
  Mixing them is error `N0107`.
- To return several values, a function returns a map (`<string, int>`) or a list, and the caller
  destructures it.

### 3.2 Results: `Result`, `Ok` and `Err`

An operation that can fail returns `Result<E, T>`: the error type goes first and the value type after.
`Ok(value)` when it succeeds (a value of type `T`) or `Err(error)` when it fails (an error of type `E`).
`Result`, `Ok` and `Err` exist without declaring them; a file that declares a module or function with one of
those names uses its own.

```nuclear
findUser(id: string) Result<string, User>
  if id == ''
    return Err('empty id')
  Ok(User(id))
```

`Ok(...)` and `Err(...)` are always written: a plain value on the last line does not become `Ok`. `Ok()`
without a value is the success of a `Result<E, void>`.

#### Automatic propagation

Inside a function that returns `Result`, a call that returns `Result` **unwraps itself**: if it is
`Ok(value)`, the expression is `value` and execution continues; if it is `Err(error)`, the function ends at
that point and returns that `Err`. No `match`, `try` nor any operator is written.

```nuclear
buy(id: string) Result<string, Order>
  user = findUser(id)          // user is a User; if it fails, buy returns the Err
  stock = checkStock(id)       // does not run if findUser failed
  save(user)                   // unused value: only its error propagates
  Ok(Order(user, stock))
```

- Calls to functions and methods written in Nuclear with return type `Result<E, T>` propagate (with `~` if
  they are async). Calls to external libraries do not propagate.
- The call runs before the statement that contains it, in writing order: `Order(findUser(x), findUser(y))`
  propagates the error of `findUser(x)` first.
- It cannot propagate from a part that may not run: the right side of `and`, `or` and `??`, the branches of
  `c ? a : b` and of an `if`/`match` used as a value, an `if` inside a list, the condition of an `else if`
  and of a `while`. There the `Result` is assigned to a variable first (error `N0110`).
- The propagated error must have the function's error type: `findUser` fails with `string` and does not
  propagate into a function that returns `Result<AppError, Order>` (error `N0109`). Convert it with
  `.error(e => Err(AppError(e)))`.
- Outside a function that returns `Result` there is nowhere to propagate: an unhandled `Result` is error
  `N0108`. Handle it with `match`, by storing it in a variable typed `Result`, with `.error(...)` in a
  statement, or by declaring the function's return as `Result<E, T>`.

#### Keeping a `Result`

A `Result` is not unwrapped when:

- it is stored in a variable typed `Result`: `r: Result<string, User> = findUser(id)`;
- it is the return value (the last line or a `return`) of a function that returns `Result` or that does not
  write its return type;
- it is the argument of a parameter typed `Result`;
- it is the subject of a `match` or the object of `.ok()` / `.error()`.

#### `.ok()` and `.error()`

Automatic propagation is the default behaviour. `.ok(callback)` and `.error(callback)` are optional and let
you handle one branch by hand: `.ok` acts only on the `Ok` branch and `.error` only on the `Err` branch; the
other branch stays the same. For one result a single callback runs. What the last line of the callback
returns decides what it does:

| The callback returns | `.ok(v => ...)` | `.error(e => ...)` |
|---|---|---|
| `Ok(...)` or `Err(...)` (or a call that returns `Result`) | transforms: that is the new result | recovers (`Ok`) or changes the error (`Err`) |
| a value: a construction (`User('guest')`), a literal, a variable, an operation or a call to a program function with a return type | transforms the value: the result is `Ok(new value)` | **recovers**: the expression is that value and is no longer a `Result` |
| nothing: a function returning `void` or an external function (`print`, a logger) | observes: the `Ok` stays the same | observes: the `Err` stays the same and **keeps propagating** |

Logging an error does not handle it: only a value or an `Ok(...)` recovers it.

```nuclear
buy(id: string) Result<string, string>
  stock = checkStock(id)
    .error(e => log(e))                    // observes: the error is logged and propagates
  guest = findUser('x')
    .error(e => User('guest'))             // recovers: guest is a User
  name = findUser(id)
    .ok(u => u.name)                       // transforms: name is a string
  Ok('${name} ${stock} ${guest.name}')
```

- The result of `.ok()` and of an `.error()` that does not recover with a value is still a `Result`: it
  propagates or is kept like any other.
- A callback returns `Ok`/`Err` in all its branches or in none; mixing them is error `N0111`.
- A callback that returns `Result` is like a function that returns `Result`: errors propagate inside it.
- To transform or recover with the value of an external function, write `Ok(f(v))`.

#### `match` on a `Result`

The branches `Ok(name) ->` and `Err(name) ->` take the value or the error; `_` is any other case and `Ok(_)`
ignores the value. Each branch has a single pattern.

```nuclear
show(id: string) string
  match buy(id)
    Ok(order) -> 'purchase ${order.id}'
    Err(e) -> 'failed: ${e}'
```

- As a value (`x = match ...`), the `match` needs the `Ok` and `Err` branches, or `_`.
- A pattern other than `Ok(name)`, `Err(name)` or `_` is error `N0111`.
- An async function gives its `Result` with `~`: `match ~load(id)`. Without `~` it is error `N0111`.

## 4. Declarations: the "header + block" rule

**Every** declaration has the same shape:

```
[decorators]
[leading-words...] [~]Name[<Generics>][(parameters)] [: Base, Other] [> Interface, Other] [ReturnType | clause Type...]
  indented body
```

- **Leading words:** all the words before the name. They can be the **kind of declaration** (*kind*) or
  **modifiers** (`static`, `abstract`).
- **Name:** the first PascalCase name of the header; if all are camelCase, the last one (`static count()` →
  modifier `static`, name `count`).
- **Generics:** `<T, U>` after the name. `<T: Comparable>` requires `T` to extend `Comparable` (`:` means
  "extends", as in inheritance): `Box<T: Comparable>`, `larger<T: Comparable>(a: T, b: T) T`.
- **Clauses:** after the name, a word followed by a type (`<word> Type`). A header has a return type **or**
  clauses, not both (so the parser tells them apart without ambiguity).
- The parser accepts any word in those positions. Then the **active dialect** decides whether it knows it: a
  word the dialect does not define is error `N0201`, with the closest one as a suggestion.

The base language has no kinds nor modifiers: they all come from the active dialect. Without a kind, the
transpiler deduces the declaration from its shape:

| Shape | It is |
|---|---|
| camelCase name | function |
| PascalCase name with only `name` or `name(value)` items | enum (§4.3) |
| PascalCase name with only attributes without values and signatures without bodies | interface (§4.3) |
| Any other PascalCase name | module (a class in the target) |

Header words are defined in the dialect's files, without touching the parser (see
[dialects.md](dialects.md)). Each dialect documents its own in its spec, which inherits from this document:
`spec/<dialect>.md`.

### 4.1 Functions

```nuclear
greet(name: string) string
  'Hello ${name}'

add(a: number, b: number) number = a + b         // one-line form.

log(message: string)                             // no return type = void
  print('[app] ${message}')
```

RULE: the one-line code `add(a: number, b: number) number = a + b` returns number automatically, but if the
developer wants to return void, they must add it as the return type in the signature and the transpiler
then leaves out the return, ex:

```
add(a: number, b: number) void
  number = a + b
```

The return type is written **without `:`**, right after the parameters. `:` is used only for inheritance
(see §4.2).

The previous function returns void.

- **Implicit return:** if the function declares a return type (other than `void`), the last expression of
  the block is returned. `return` exists to leave early.
- Without a return type → the last line is returned, as in Elixir, and its type is inferred, unless the
  developer writes the explicit `void` type in the signature.

```
// with an inferred return, this returns number
add(a: number, b: number)
  number = a + b

// with an explicit return it also returns number, but it must be multi-line
add(a: number, b: number) number
  number = a + b

// with a return type, inline code cannot be used: this is an error.
add(a: number, b: number) void number = a + b

// to return void, use an indented multi-line body. void is special because, without it, the transpiler
// infers the return from the last line.
add(a: number, b: number) void
  number = a + b
```

- Parameters: `name: Type`, optional `name?: Type`, with a default `name: Type = value`.
- **await:** write `~` before the call. `~` applies to the whole chain that follows: `~getUsers.get()` waits
  for the result of `getUsers.get()`.
- **Automatic async:** `async` is not written. A function that uses `~` is async; the dialect emits its
  return type as the target's asynchronous type.

```nuclear
greet(name: string) string
  "Hello ${name}"

welcome() string
  text = ~greet("world")
  text
```

- For a function to be async even if it does not use `~` in its body, write `~` before the name in the
  declaration:

```nuclear
~greet(name: string) string
  "Hello ${name}"
```

- `@` is used only for decorators (§4.4).
- **Generators:** a function that uses `yield` is a generator; if it also uses `~`, it is an asynchronous
  generator. As with `async`, the return type is the type of each value, and the function does not return
  its last line. `yield ...other()` yields every value of another generator.

```nuclear
count(limit: int) int
  for i in 0..limit
    yield i

all(limit: int) int
  yield 0
  yield ...count(limit)
```

### 4.1.1 Anonymous functions

Functions are not declared inside a body. A local function is written as an **anonymous function assigned
to a name**: parameters in parentheses, an optional return type and the body in the indented block.

```nuclear
Network
  retry(attempt: int)
    log = (message: string)
      print(message)
    connect()
```

- One-line form: `twice = (n: int) => n * 2`.
- With a return type: `format = (n: int) string` + block.
- Inside a body, `name(parameters)` followed by a block is always a **call** that receives a lambda (§6.4),
  never a declaration.

### 4.1.2 Function aliases

`name() = Module.method` declares `name` as **the same function** as `method`: it receives the same
parameters, returns the same type and is async or a generator if `method` is. It exposes, from one module,
the functionality of another (a use case) without repeating its signature.

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

- It is an alias when the parentheses are empty, there is no return type and the body is only
  `Module.method` (not called), with `Module` declared in the file and `method` one of its methods. In any
  other case it is a normal one-line function: `role() = Role.admin` returns `Role.admin`.
- It is declared as a top-level function or as a method of a module.
- A `static` method is called on the module. Any other is called on a new `Module()` on each call, so every
  parameter of `Module`'s constructor needs a default value (error N0112).
- Calls to the alias are treated as calls to `method`: if it returns `Result`, its errors propagate (§3.2).

### 4.2 Classes

Classes do not exist in Nuclear, but we need an equivalent for other languages and technologies. For that
Nuclear has the module, a concept similar to Elixir's, which in Nuclear only gives the equivalence of
classes and constructors.

```nuclear
User()
  // attributes
  name: string
  email: string
  role: Role = Role.user
  token?: string

  // constructor
  ()
    name = "name assignment"

  isAdmin() boolean
    role == Role.admin

Admin
  permissions: string[] = []
```

- **Constructor**
  The constructor is omitted if it does not exist; when it is needed, write empty parentheses or
  parentheses with parameters, for example:

  - Parser RULE: constructors and attributes follow the same assignment and mandatory initialization rules
    as in TypeScript.

  1. Only parameters, with an optional parameter at the end.

  ```nuclear
  User(name: string, email: string, role: string = "")
    someMethod(myOwnParam: string)
      name = myOwnParam
  ```

  2. Received and local parameters

  ```nuclear
    User(name: string, email: string, role: string = "")
      localAttr: string = ""

      someMethod(myOwnParam: string)
        name = myOwnParam
        localAttr = myOwnParam
    ```

    3. A module with a body for the constructor.

    RULE: a constructor with a body is written **on its own line**, inside the module's block: the module's
    name goes alone, below it the parameters in parentheses and, more indented, the constructor's body. It
    is the only way to write a constructor with a body.

    ```nuclear
    User
      (name: string, email: string, role: string = "", localAttr: string = "")
        localAttr = "other value"
    ```

    If the parameters do not fit on one line, they are split inside the same parentheses:

    ```nuclear
    User
      (
        name: string,
        email: string,
        role: string = "",
        localAttr: string = ""
      )
        localAttr = "other value"
    ```

    4. Named parameters: `{}` in the signature is optional and only has an effect in the dialects whose
    target has named parameters; each one documents it in its spec. In the other dialects `{}` changes
    nothing: the parameters are emitted as without `{}`. It also holds for functions.

    ```nuclear
    User({name: string, email: string, role: string = ""})
      someMethod({myOwnParam: string})
        name = myOwnParam
    ```

    5. Use of this, replaced by the `$` wildcard

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

- **Inheritance:**

Officially Nuclear wants to look like, and be, a language more like Elixir. But, since it has to transpile to
object-oriented languages, Nuclear supports transpiling object-oriented programming.

```nuclear

Admin: User, Person > Serializable, JsonSerializable
  someMethod()
    "Say HI!"

```

`:` is the equivalent of `extends`; note that it supports multiple inheritance.

`>` is the equivalent of `implements`.

A module that inherits calls its base's constructor with `super(...)` as the first line of its constructor's
body; inside those parentheses, the parameter names are the received values. Without that line, the base's
constructor is called without arguments.

```nuclear
Animal
  (name: string)

Dog: Animal
  (name: string, breed: string)
    super(name)
```

A method with the same name as a member of a base or interface declared in the file overrides it; the
dialect marks it as its target requires.

### 4.3 Interfaces and enums

Interfaces have the same syntax as modules; the only way to tell them apart is that their methods and
attributes have no body. If that holds, the transpiler considers it an interface automatically.

There is no word nor symbol to declare an interface. The transpiler deduces it: if the block has only
attributes and signatures, it is an interface.

Body means that a function has indented code on the next line, for example:

With a body: NOT AN INTERFACE, A MODULE OR STRUCT:

```nuclear
Message
  some(name: string) void
    print(name)
```

Without a body: IT IS AN INTERFACE:

```nuclear
Message
  role: 'user' | 'assistant' | 'system'
  some(name: string) void
```

Full example of an interface:

```nuclear
Message
  role: 'user' | 'assistant' | 'system'
  content: string
  name?: string
  some(name: string) void
```

Interfaces can implement other interfaces, with the `>` wildcard as in modules.

Enums 1: it is an enum if it has no parameters, parentheses or functions and only has the enum's items,
indented.

```nuclear
Role
  user
  admin
```

Enums 2: key-value

```nuclear
Role
  user("person")
  admin("administrator")
```

In a key-value enum, an item without a value takes its own name as its value: in the next one, `medium` is
`"medium"`.

```nuclear
Level
  low("Low")
  high("High")
  medium
```

IMPORTANT, PARSER RULE!

 - If the enum has any other syntax besides the examples enum 1 and enum 2, the parser throws an error.

### 4.4 Decorators

They are written as in TypeScript and pass to the dialect untouched:

```nuclear
@Controller('users')
UsersController
  @Get(':id')
  find(@Param('id') id: string) User
    users.findOne(id)
```

### 4.5 Visibility between files
There is no private, public nor protected in NuclearLang.

### 4.6 Parameters

A function or method receives only the parameters written in its signature. Without parentheses
(`getSales` + block) it receives no parameters.

## 5. Types

| Description | Types | Examples |
|---|---|---|
| native basics | `string`, `double`, `int`, `boolean` | num: double |
| lists | `User[]` | num[] = [1, 2, 3] |
| maps | `<Key, Value>` | users: <string, User> = {"julian": User(name = "julian"), "other": User(name = "other")} |
| nullable | `?` | user: string? |
| result | `Result<E, T>` | `findUser(id: string) Result<string, User>` (§3.2) |

Types are optional where the target can infer them (variables, lambda returns).

The members of these types (`length`, `add`, `map`, `split`, `upper`...), `Math`, `Json`, `Regex`, `DateTime` and
the other objects of the base are Nuclear's own and the same in every dialect: [library.md](library.md).

Maps:
- The type is written `<Key, Value>` in any position: variable (`users: <string, User>`), parameter
  (`f(scores: <string, int>)`), return (`get(name: string) <string, string>`), list (`<string, int>[]`) and
  nullable (`<string, int>?`).
- The value always goes between `{ }`: `{"name": "julian", "last": "molina"}`. Empty map: `{}`.

## 6. Expressions

### 6.1 Operators (lowest to highest precedence)

| Level | Operators |
|---|---|
| 1 | `c ? a : b` |
| 2 | `??` |
| 3 | `or` |
| 4 | `and` |
| 5 | `not` |
| 6 | `==` `!=` `<` `>` `<=` `>=` `is` `in` `as` |
| 7 | `..` (range) |
| 8 | `\|` (bitwise or) |
| 9 | `^` (bitwise xor) |
| 10 | `&` (bitwise and) |
| 11 | `<<` `>>` `>>>` (shifts; `>>>` fills with zeros) |
| 12 | `+` `-` |
| 13 | `*` `/` `%` `~/` (integer division) |
| 14 | `-x` (unary), `~x` (await), `^x` (bitwise complement) |
| 15 | `a ** b` (power, right associative: `2 ** 3 ** 2` is `2 ** 9`, `-2 ** 2` is `-4`) |
| 16 | `a.b` `a?.b` `a(...)` `a<T>(...)` `a[i]` `a!` |

- `==` is always a strict comparison. There is no `===`.
- `and`/`or`/`not` instead of `&&`/`||`/`!`.
- `x is User` checks whether `x` is of type `User`.
- `x as Type` converts the type: `data = read(file) as <string, int>`.
- `value!` asserts that the value is not null: `user.address!.city`. The `!` goes right after a name, `)` or
  `]`. Before a value, `!` is an error: write `not`.
- `a / b` is always a decimal division; `a ~/ b` drops the decimals (toward zero). `%`, the bitwise operators and
  the integer width are the target's: TypeScript's `%` keeps the dividend's sign and its bitwise operators work on
  32 bits; Dart's `%` is never negative and its integers have 64 bits.
- The bitwise operators bind tighter than the comparisons (unlike C): `flags & READ != 0` is `(flags & READ) != 0`.
  `~` is await, so the complement is `^x`. Compound forms: `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `>>>=`.
- `x in xs` asks whether a list contains `x`; with a map, whether it has the key `x`.
- Nuclear has no standard library of its own: math, text, regular expressions, collections, dates, JSON, bytes
  and async come from the target (`Math.sqrt` in TypeScript, `sqrt` of `dart:math` in Dart), imported
  automatically. `examples/stdlib/` has one program per area and target that prints the same result in both
  (`tools/stdlib-check.sh` runs them); `nuclear help operations` lists them.

### 6.2 Calls with named arguments

`:` and `=` have a fixed role inside parentheses:

| Where | Form | Meaning |
|---|---|---|
| Declaration (signature) | `get(user: User)` | `:` defines the parameter's type |
| Call, by position | `get(User(name = 'Ana'))` | the argument is assigned by order |
| Call, by name | `get(user = User(name = 'Ana'))` | `=` assigns the argument to the `user` parameter |

A call can mix both forms: first the positional arguments and then the named ones.

```nuclear
createUser('Ana', admin = true)
```
- If the function is written in Nuclear, the transpiler reorders the arguments by position.
- If it is external, each dialect decides: group them in an object or emit them as the target's named
  arguments.

### 6.3 Construction without `new`
```nuclear
user = User(name = 'Ana', email = 'a@x.com')
```

Type arguments go between `< >` right after the name, just before the parentheses. With a space before `<`
it is a comparison (`a < b`):

```nuclear
cache = Cache<string, User>()
users = repo.findAll<User>(active = true)
```

### 6.4 Lambdas

Functions that receive a lambda (`map`, `filter`, `reduce`...) are written with the **lambda's parameters in
parentheses** and the **body in the indented block** of the next line. No `=>`, `return` nor `{ }` is
written: the last line of the block is the value the lambda returns.

```
list.function(parameter: Type)
  body
  lastLine           // returned value
```

**`map` over a range:**

```nuclear
User(name: string)

users = (1..10).map(iterator: int)
  User(name = "Julian")
```

**`filter` over a list that is already full** (for example, from a repository):

```nuclear
users = usersRepo.get(active = true)
usersFiltered = users.filter(user: User)
  user.birthday < now()
```

**One-line lambda:** with `=>`, for single-expression bodies.

```nuclear
doubled = numbers.map(n => n * 2)
adults = users.filter((user: User) => user.age >= 18)
```

**Lambda with a block as an argument:** inside a call's parentheses, `(parameters)` followed by a more
indented block is a lambda. The block ends when the indentation returns to the line where it started, and
the `,` before the next argument is optional.

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

server.on(
  'message',
  (socket, data)
    text = data.toString()
    socket.send(text)
)
```

**Chain after a block:** a line that starts with `.` at the statement's level continues the chain after the
lambda's block.

```nuclear
label = users.map(user: User)
  user.name.toUpperCase()
.join(', ')
```

Rules:
- Inside a call's parentheses, `name: Type` declares a lambda parameter; named arguments use `=` (see §6.2).
- If the body takes more than one line, use the block form.
- A one-line lambda does not assign: `(v) => email = v` is an error; the assignment goes in a block.
- A range followed by a method goes between parentheses: `(1..10).map(...)`. Writing `1..10.map(...)` is an
  error.

### 6.5 Cascades

The lines that start with `..` below an expression operate on the same object, and the value is that object:

```nuclear
brush = Paint()
  ..color = Colors.blue
  ..strokeWidth = 4

text = StringBuffer()
  ..write('a')
  ..write('b')
```

- Each `..` line assigns a member or calls a method of the object.
- A `.` line below a section continues that section (`..items.add(x)`); to use the object after the cascade,
  assign it to a variable.
- `..` with something on its left on the same line is a range (`1..10`); at the start of a line it is a
  cascade.

## 7. Control flow

```nuclear
if x > 0
  ...
else if x < 0
  ...
else
  ...

for user in users
  print(user.name)

for i in 0..10          // 0 to 9; to call a method: (0..10).map(...)
for key, value in config

while pending > 0
  pending -= 1

match event.kind
  'message' -> process(event)
  'error' ->
    log(event.error)
    retry()
  _ -> ignore()

try
  data = ~fetchJson(url)
catch error
  log(error.message)
finally
  close()

throw Error('something went wrong')
```

`if` and `match` are also values: `label = match n` ... The value of each branch is its last line (see §3). A
`try` as the last line of a function returns the last line of `try` (or of `catch` when it fails).

At the top level of a file, statements other than declarations run in order; in a target that runs them inside
`main` (Dart), a top-level variable stays a top-level variable and a file cannot have both other statements and
a `main` function (`N0113`).

## 8. Modules

`use` imports modules and files. The path and the import it generates depend on each dialect, which knows
its native technology:

```nuclear
use express as "express"
use fs/promises{ readFile, writeFile }
use * as path from path
use './setup'
```

Dialects can **auto-import**: a name of a native library of the technology is imported when the code uses
it, so the code does not fill up with imports. The words after a `use` path (`show`, `hide`...) and the
shape of paths are defined by each dialect in its spec.

## 9. Diagnostics for agents

Each error has a stable code, a position and, when possible, the exact fix:

```json
{
  "code": "N0009",
  "message": "'&&' does not exist in NuclearLang",
  "file": "src/cart.nuc", "line": 12, "col": 3,
  "fix": { "line": 12, "replace": "active && admin", "with": "active and admin" },
  "explanation-context": "NuclearLang uses the words and, or, not instead of &&, ||, !."
}
```

`nuclear check --json` returns this list. An agent can fix the code without reading documentation.

## 10. Complete example

Each dialect makes the code easier: it knows its technology, so the imports of its native libraries are
shortened or added automatically.

```nuclear

// not needed, because the transpiler infers it.
use fs/promises{ readFile, writeFile}

// header of a comment, it works like a title for comments
 * multi-line comment with indentation

// only attributes and signatures → the transpiler treats it as an interface
Message
  role: 'user' | 'assistant'
  content: string

Text()
  describeText(text: string)
    text

Chat: Text > Message
  model: string
  history: Message[] = []

  send(text: string) string
    history.add({ role: 'user', content: text })
    reply = ~callModel(model, history)
    history.add({ role: 'assistant', content: reply })
    // from extends
    describeText(reply)

loadPrompt(path: string) string
  ~readFile(path, 'utf8')

callModel(model: string, messages: Message[]) string
  res = ~fetch('https://api.example.com/chat', method = 'POST', body = JSON.stringify({ model, messages }))
  data = ~res.json()
  data.text
```
