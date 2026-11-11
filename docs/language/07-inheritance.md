# 7. Inheritance, interfaces and enums

## How Nuclear deduces what each declaration is

There are no `class`, `interface` nor `enum` words. The transpiler deduces it from the **shape** of the block:

| Shape | It is |
|---|---|
| camelCase name | function |
| PascalCase with only `name` or `name(value)` items | enum |
| PascalCase with only attributes without values and signatures without bodies | interface |
| any other PascalCase | module (class) |

## Interfaces

An interface has the same syntax as a module, but **nothing has a body**: attributes without values and methods
without a block.

```nuclear
Message
  role: 'user' | 'assistant' | 'system'
  content: string
  name?: string
  summary() string
```

```typescript
export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  name?: string;
  summary(): string;
}
```

```dart
abstract class Message {
  String get role;
  String get content;
  String? get name;
  String summary();
}
```

As soon as a method has a body or an attribute has a value, it is a module.

## Inheritance: `:` extends, `>` implements

```
Name: Base > Interface, OtherInterface
```

- `:` is **extends** (inheriting from a module).
- `>` is **implements** (fulfilling an interface).

```nuclear
Named
  name: string

Serializable > Named
  toJson() string

Animal
  (name: string)
  speak() string
    '...'

Dog: Animal > Serializable
  (name: string, breed: string)
    super(name)

  speak() string
    'woof'

  toJson() string
    '{"name": "${name}"}'
```

```typescript
export interface Named {
  name: string;
}

export interface Serializable extends Named {
  toJson(): string;
}

export class Animal {
  constructor(public name: string) {}

  speak(): string {
    return '...';
  }
}

export class Dog extends Animal implements Serializable {
  constructor(name: string, public breed: string) {
    super(name);
  }

  override speak(): string {
    return 'woof';
  }

  toJson(): string {
    return `{"name": "${this.name}"}`;
  }
}
```

```dart
abstract class Named {
  String get name;
}

abstract class Serializable implements Named {
  String toJson();
}

class Animal {
  String name;

  Animal(this.name);

  String speak() {
    return '...';
  }
}

class Dog extends Animal implements Serializable {
  String breed;

  Dog(super.name, this.breed);

  @override
  String speak() {
    return 'woof';
  }

  @override
  String toJson() {
    return '{"name": "$name"}';
  }
}
```

- An interface also uses `>` to inherit from other interfaces (`Serializable > Named`).
- If you write `:` with a base that is an interface, the transpiler moves it to `implements`.
- How many bases `:` accepts is up to the dialect. TypeScript and Dart accept one; more is error `N0208` (the
  others must go in `>` if they are interfaces).

### Calling the base's constructor: `super(...)`

A module that inherits calls its base's constructor with `super(...)` as the **first line** of its constructor's
body. Inside those parentheses, the parameter names are the received values. Without that line, the base's
constructor is called without arguments.

A parameter named like an attribute of the base (`name` in `Dog`) is that attribute, not a new one.

### Overriding

A method with the same name as a member of a base or interface **overrides it**, and the dialect marks it as
needed (`override` in TS for extended bases, `@override` in Dart). Nothing is written.

This works with the bases declared in the file and with the classes of the target's library the dialect knows
from its **type definitions** (for example `CustomPainter` or `State` in Flutter). A base neither of them knows
leaves the module "open": overrides are not marked and the names it uses are not checked.

## Enums

### Simple enum

Only names, one per line. Each target chooses how to represent it (in TypeScript, a numeric enum):

```nuclear
Role
  user
  admin
```

```typescript
export enum Role {
  user,
  admin
}
```

### Key-value enum

Each item carries its value in parentheses. An item without a value takes its own name:

```nuclear
Level
  low("Low")
  high("High")
  medium
```

```typescript
export enum Level {
  low = 'Low',
  high = 'High',
  medium = 'medium'
}
```

```dart
enum Level {
  low('Low'),
  high('High'),
  medium('medium');

  const Level(this.value);

  final String value;
}
```

An enum can only have items. If it mixes items with attributes or methods, it is error `N0104`:

```nuclear error
Role
  user
  admin() int       // N0104
```

Use: `Role.admin`, `user.role == Role.admin`.
