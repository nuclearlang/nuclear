# 6. Modules and constructors

Nuclear is not object oriented: it thinks in **modules** (as in Elixir) and functions. But since it transpiles
to languages with classes, a module is emitted as a **class** in the target.

A module is a **PascalCase** name with a block that has attributes with values, methods with bodies or a
constructor.

```nuclear
Counter
  count: int = 0

  increment() void
    count += 1

  current() int
    count
```

```typescript
export class Counter {
  count: number = 0;

  increment(): void {
    this.count += 1;
  }

  current(): number {
    return this.count;
  }
}
```

```dart
class Counter {
  int count = 0;

  void increment() {
    count += 1;
  }

  int current() {
    return count;
  }
}
```

There is no `private`, `public` nor `protected`.

## Attributes

```
name: Type = value       // with an initial value
name?: Type              // optional (may stay without a value)
name: Type               // no value: it must come through the constructor
```

## Methods and `this`

A method is a function inside the module's block. It follows the same rules: implicit return, `void`,
parameters, `~`...

**`this` is not written.** Inside a method, an attribute is used by its name and the transpiler adds `this.`
where the target needs it.

When a parameter is named like an attribute, the bare name is the **parameter** and `$name` is the
**attribute** (`$` is the explicit `this`):

```nuclear
Account
  owner: string = ''

  rename(owner: string) void
    $owner = owner
```

```typescript
export class Account {
  owner: string = '';

  rename(owner: string): void {
    this.owner = owner;
  }
}
```

## Constructors

The constructor is not called `constructor`: it is the **module's parameters**. Each constructor parameter is
also an attribute. There are three forms.

### 1. Parameters in the header

When the constructor does nothing but store the values, the parameters go next to the name:

```nuclear
User(name: string, email: string, role: string = 'user')
  token?: string

  describe() string
    '${name} <${email}>'
```

```typescript
export class User {
  token?: string;

  constructor(
    public name: string,
    public email: string,
    public role: string = 'user',
  ) {}

  describe(): string {
    return `${this.name} <${this.email}>`;
  }
}
```

```dart
class User {
  String name;
  String email;
  String role;
  String? token;

  User(this.name, this.email, [this.role = 'user']);

  String describe() {
    return '$name <$email>';
  }
}
```

### 2. Constructor with a body

If the constructor has to **do something** when the object is created, it is written on its own line inside
the block: the module's name goes alone, below it the parameters in parentheses and, more indented, the body.
It is the only way to write a constructor with a body.

```nuclear
Account
  (owner: string, balance: double = 0)
    print('account for ${owner}')

  deposit(amount: double) void
    balance += amount
```

```typescript
export class Account {
  constructor(public owner: string, public balance: number = 0) {
    console.log(`account for ${this.owner}`);
  }

  deposit(amount: number): void {
    this.balance += amount;
  }
}
```

```dart
class Account {
  String owner;
  double balance;

  Account(this.owner, [this.balance = 0]) {
    print('account for $owner');
  }

  void deposit(double amount) {
    balance += amount;
  }
}
```

If the parameters do not fit on one line, they are split inside the same parentheses:

```nuclear
Address
  (
    street: string,
    city: string,
    zip: string = '',
    label: string = ''
  )
    label = '${street}, ${city}'
```

```typescript
export class Address {
  constructor(
    public street: string,
    public city: string,
    public zip: string = '',
    public label: string = '',
  ) {
    this.label = `${this.street}, ${this.city}`;
  }
}
```

### 3. Named parameters `{ }`

As in functions: in TypeScript nothing changes; in Dart the constructor gets named parameters.

```nuclear
Person({name: string, age: int = 0})
  greet() string
    'Hi ${name}'
```

```typescript
export class Person {
  constructor(public name: string, public age: number = 0) {}

  greet(): string {
    return `Hi ${this.name}`;
  }
}
```

```dart
class Person {
  String name;
  int age;

  Person({required this.name, this.age = 0});

  String greet() {
    return 'Hi $name';
  }
}
```

## Constructing an object: without `new`

Calling a PascalCase name constructs an object. The arguments go by position or by name (`=`), as in a
function:

```nuclear
Person({name: string, age: int = 0})
  greet() string
    'Hi ${name}'

run() void
  ana = Person(name = 'Ana', age = 30)
  print(ana.greet())
```

```typescript
export function run(): void {
  const ana = new Person('Ana', 30);
  console.log(ana.greet());
}
```

```dart
void run() {
  final ana = Person(name: 'Ana', age: 30);
  print(ana.greet());
}
```

## Dialect modifiers

The words before the name (`abstract`, `static`, `get`, `set`...) **are not part of the base language**: each
dialect defines them. TypeScript's and Dart's include:

```nuclear
abstract Shape
  (name: string)
  area() double
    0.0

Circle: Shape
  (name: string, radius: double)
    super(name)

  static unit() Circle
    Circle('unit', 1.0)

  get diameter() double
    radius * 2

  area() double
    3.14 * radius * radius
```

```typescript
export abstract class Shape {
  constructor(public name: string) {}

  area(): number {
    return 0.0;
  }
}

export class Circle extends Shape {
  constructor(name: string, public radius: number) {
    super(name);
  }

  static unit(): Circle {
    return new Circle('unit', 1.0);
  }

  get diameter(): number {
    return this.radius * 2;
  }

  override area(): number {
    return 3.14 * this.radius * this.radius;
  }
}
```

A word the dialect does not know is error `N0201`, with the closest one as a suggestion. Each dialect
documents its own in `spec/<dialect>.md`.

## Decorators

They are written as in TypeScript (`@Name(...)` on the line before) and pass to the target untouched:

```nuclear
@Injectable()
UsersService
  @Log('find')
  find(id: string) string
    id
```

```typescript
@Injectable()
export class UsersService {
  @Log('find')
  find(id: string): string {
    return id;
  }
}
```

`@` is used only for decorators.
