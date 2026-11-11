# Dialect `dart`

> Inherits every rule of [nuclear-base.md](nuclear-base.md). This document defines what the `dart`
> dialect adds, how it translates the base rules to Dart 3 and (section 4) the equivalence of the base
> examples. Files: `dialects/dart/`.

## 1. Header words

### 1.1 Kinds

| Kind | Nuclear | Dart |
|---|---|---|
| `extension` | `extension NumberParsing on String` + methods | `extension NumberParsing on String { ... }` |
| `mixin` | `mixin Flyer on Animal` + members | `mixin Flyer on Animal { ... }` |
| `data` | `data Point` + fields (+ methods) | immutable class with a `const` constructor of named parameters, `copyWith` and `toJson` |

```nuclear
extension NumberParsing on String
  parseInt() int
    this.toInt()!

mixin Flyer on Animal
  fly() string
    "${name} is flying"

data Point
  x: double
  y: double
```

### 1.2 Modifiers

`abstract` `static` `late` `final` `const` `sealed` `base` `external` `factory` `covariant` `get` `set`

- `get` declares a getter: `get color() Color` + block → `Color get color { ... }`.
- `set` declares a setter: `set color(value: Color)` + block → `set color(Color value) { ... }`.

### 1.3 Clauses

`with`: `Bird: Animal with Flyer` → `class Bird extends Animal with Flyer`.

## 2. Translation of the base rules

| Base | Dart |
|---|---|
| Variable that is not reassigned | `final x = ...` |
| Variable that is reassigned | `var x = ...` |
| `x = if ...` with multi-line branches | `var x;` (or `Type x;` if the assignment has a type) and an `if` that assigns in each branch |
| Members of the module | without `this.` |
| Construct `User(...)` | `User(...)` (no `new`) |
| Named arguments `f(x = 1)` | `f(x: 1)` |
| `{}` in the signature | named parameters: `{required String name}` |
| Optional parameter `name?: string` | `String? name` |
| Default value `= null` | omitted (`String? data;`) |
| List or map as a parameter default | `const []`, `const {}` |
| Constructor parameters | declared fields + `User(this.name)` |
| Text | single quotes; `'${name}'` is written `'$name'` when the name is not followed by letters |
| `x is T` / `x as T` / `x!` | the same |
| `and` `or` `not` | `&&` `\|\|` `!` |
| `a in list` | `list.contains(a)`; a map: `map.containsKey(key)` |
| `a ** b` | `pow(a, b)`, plus `.toInt()` when both operands are known integers, `.toDouble()` when one is a decimal |
| `a ~/ b`, `& \| ^ << >> >>>`, `^x` | the same; `^x` → `~x` |
| `0b1010` | `0xa` (Dart has no binary literals) |
| `print(a, b)` | `print([a, b].join(' '))` (Dart's `print` takes one value) |
| top-level statements | inside a generated `main()`; top-level variables stay top-level (`final g = 9.8;`) |
| Range `a..b` as a value | `List.generate(b - a, (i) => a + i)` |
| `for k, v in map` | `for (final MapEntry(key: k, value: v) in map.entries)` |
| Top-level statements | inside `void main() { ... }` |
| Inheritance | a single base (`: A`); interfaces go with `>` or are detected, also the SDK's `interface class`es: `Score: Comparable<Score>` → `class Score implements Comparable<Score>` |
| `super(...)` at the start of the constructor | initializer list: `Bird(this.wings) : super('bird')`; if it passes the inherited parameters as they are and in order, super parameters: `Dog(super.name, this.breed)` |
| Method that overrides one of a base or interface (of the file or of the SDK type definitions, including `Object`: `toString`) | `@override` |
| PascalCase decorator without arguments (`@DateConverter`) | the call to its const constructor: `@DateConverter()`; a camelCase one (`@override`) stays the same |
| `[a, ...rest] = xs` / `{ a, "k": b } = m` | `final [a, ...rest] = xs;` / `final {'a': a, 'k': b} = m;` |
| Existing names in a pattern | `[a, b] = [b, a];` |
| `Result<E, T>`, `Ok(x)`, `Err(e)` | `Either<E, T>`, `Right(x)`, `Left(e)` of [fpdart](https://pub.dev/packages/fpdart) (imported automatically; no own runtime); `Ok()` → `Right(null)` |
| `x = call()` returning `Result`, in a `Result` function | `final xResult = call(); if (xResult case Left(:final value)) return Left(value); final x = (xResult as Right<E, T>).value;` |
| `.ok(cb)` / `.error(cb)` that transform | `.flatMap(cb)` / `.orElse(cb)` (`.map` / `.getOrElse` when the callback returns a plain value) |
| `.ok(cb)` / `.error(cb)` that observe | a `map` / `mapLeft` that calls `cb` and returns the value unchanged |
| `match r` with `Ok(x)` / `Err(e)` | `if (r.isRight()) { ... (r as Right<E, T>).value ... } else { ... (r as Left<E, T>).value ... }` |
| Function with `yield` | `sync*` and return `Iterable<T>`; with `~`, `async*` and `Stream<T>` |
| `name() = Module.method` (alias) | a method with `method`'s signature that returns `Module().method(...)` (`Module.method(...)` if it is `static`) |
| `yield ...other()` | `yield* other()` |
| `<T: Comparable>` | `<T extends Comparable>` |
| `[a, if cond b]` | `[a, if (cond) b]` |
| Cascade `..a = 1` | the same: `Paint()..color = Colors.blue` |

## 3. Modules

- A `use` path is written `package:<path>.dart`: `use lib1/lib1` → `import 'package:lib1/lib1.dart';`.
- Words allowed after the path: `show`, `hide`, `deferred`.

```nuclear
use lib2/lib2 as lib2
use lib1/lib1 show foo
use lib2/lib2 hide foo
use greetings/hello deferred as hello
```

```dart
import 'package:lib2/lib2.dart' as lib2;
import 'package:lib1/lib1.dart' show foo;
import 'package:lib2/lib2.dart' hide foo;
import 'package:greetings/hello.dart' deferred as hello;
```

- `use` shortcuts: `async`, `collection`, `convert`, `io`, `math`, `typed_data` → `dart:<name>`.
- **Automatic imports:** every public name of `dart:async`, `dart:math`, `dart:convert`, `dart:collection`,
  `dart:io`, `dart:typed_data`, `dart:isolate` and `dart:developer`. Using `jsonDecode` adds
  `import 'dart:convert';`. The names of `dart:core` (`print`, `List`, `Future`...) are not imported.
- **Type definitions:** `defs = ["dart@3.13.5"]` ([definitions.md](definitions.md)). That package gives the
  automatic imports, the inherited members and the `@override`s of the SDK classes
  (`nuclear defs show Future --dialect dart`). The dialects that extend `dart` add their packages (Flutter,
  pub.dev libraries) from the same repository, `defs/dart/`.

## 4. Equivalences

The base examples in Dart, in the same order as the base. Each output is the one the transpiler produces.
Loose statements go inside `main()`.

### Module with functions (base §1)

```nuclear
Users()
  addDoubles(a: double, b: double)
    a + b
  substractDoubles(a: double, b: double)
    a - b + addDoubles(a, b)
```

```dart
class Users {
  addDoubles(double a, double b) {
    return a + b;
  }

  substractDoubles(double a, double b) {
    return a - b + addDoubles(a, b);
  }
}
```

### Doc comments (base §2.3)

The comment above a declaration is written with `///` (dartdoc): each parameter is a bullet
`* [name]: text` and `@return` is `* Returns ...`. An `@example` is a `nuclear` code block inside the
comment, and `@deprecated` also adds the `@Deprecated(...)` annotation, so the analyzer warns on every use.

```nuclear
// Price with tax, rounded to cents.
 * @param amount price without tax
 * @return the price with 16% tax
 * @deprecated use total()
withTax(amount: double) double
  amount * 1.16

Cart
  // Number of products.
  items: int = 0
```

```dart
/// Price with tax, rounded to cents.
///
/// * [amount]: price without tax
/// * Returns the price with 16% tax
@Deprecated('use total()')
double withTax(double amount) {
  return amount * 1.16;
}

class Cart {
  /// Number of products.
  int items = 0;
}
```

### `if` inside a list (base §2.5)

```nuclear
children(loading: boolean, items: string[]) Widget[]
  [
    Text('Title'),
    if loading Spinner(),
    ...items.map(i => Text(i)),
  ]
```

```dart
List<Widget> children(bool loading, List<String> items) {
  return [Text('Title'), if (loading) Spinner(), ...items.map((i) => Text(i)).toList()];
}
```

### Variables (base §3)

```nuclear
name = 'Ana'
age: number = 30
counter = 0
counter = counter + 1
counter += 1
```

```dart
void main() {
  final name = 'Ana';
  final num age = 30;
  var counter = 0;
  counter = counter + 1;
  counter += 1;
}
```

### `if` as a value (base §3)

```nuclear
label(age: int) string
  text = if age >= 18
    prefix = 'adu'
    prefix + 'lt'
  else
    'minor'
  text

shortLabel(age: int) string
  text = if age >= 18
    'adult'
  else
    'minor'
  text
```

```dart
String label(int age) {
  var text;
  if (age >= 18) {
    final prefix = 'adu';
    text = prefix + 'lt';
  } else {
    text = 'minor';
  }
  return text;
}

String shortLabel(int age) {
  final text = age >= 18 ? 'adult' : 'minor';
  return text;
}
```

### Destructuring (base §3.1)

```nuclear
example(xs: int[], json: <string, any>, users: <string, string>[]) void
  [first, second] = xs
  [head, ...tail] = xs
  [_, other, ...rest] = xs
  { min, max } = bounds(xs)
  { "first-name": name, age } = json
  [{ email }, next] = users
  a = 1
  b = 2
  [a, b] = [b, a]
```

```dart
void example(
  List<int> xs,
  Map<String, dynamic> json,
  List<Map<String, String>> users,
) {
  final [first, second] = xs;
  final [head, ...tail] = xs;
  final [_, other, ...rest] = xs;
  final {'min': min, 'max': max} = bounds(xs);
  final {'first-name': name, 'age': age} = json;
  final [{'email': email}, next] = users;
  var a = 1;
  var b = 2;
  [a, b] = [b, a];
}
```

A list pattern without `...rest` requires exactly that number of elements (a runtime error otherwise).

### Results (base §3.2)

`Result<E, T>` is fpdart's `Either<E, T>`: `Ok(x)` → `Right(x)`, `Err(e)` → `Left(e)`. The import of
`package:fpdart/fpdart.dart` is added automatically and there is no own runtime. The app declares `fpdart`
in its `pubspec.yaml`.

```nuclear
User(name: string)

findUser(id: string) Result<string, User>
  if id == ''
    return Err('empty id')
  Ok(User(id))

buy(id: string) Result<string, string>
  user = findUser(id)
  findUser('backup')
  Ok(user.name)
```

```dart
import 'package:fpdart/fpdart.dart';

class User {
  String name;

  User(this.name);
}

Either<String, User> findUser(String id) {
  if (id == '') {
    return Left('empty id');
  }
  return Right(User(id));
}

Either<String, String> buy(String id) {
  final userResult = findUser(id);
  if (userResult case Left(:final value)) return Left(value);
  final user = (userResult as Right<String, User>).value;
  final findUserResult = findUser('backup');
  if (findUserResult case Left(:final value)) return Left(value);
  return Right(user.name);
}
```

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

checkStock(id: string) Result<string, int> = Ok(id.length)

log(message: string) void
  print(message)

buy(id: string) Result<string, string>
  stock = checkStock(id)
    .error(e => log(e))
  guest = findUser('x')
    .error(e => Ok(User('guest')))
  name = findUser(id)
    .ok(u => Ok(u.name))
  Ok('${name} ${stock} ${guest.name}')
```

```dart
import 'package:fpdart/fpdart.dart';

class User {
  String name;

  User(this.name);
}

Either<String, User> findUser(String id) {
  return Right(User(id));
}

Either<String, int> checkStock(String id) {
  return Right(id.length);
}

void log(String message) {
  print(message);
}

Either<String, String> buy(String id) {
  final stockResult = checkStock(id).mapLeft((error) {
    ((e) => log(e))(error);
    return error;
  });
  if (stockResult case Left(:final value)) return Left(value);
  final stock = (stockResult as Right<String, int>).value;
  final guestResult = findUser('x').orElse((e) => Right(User('guest')));
  if (guestResult case Left(:final value)) return Left(value);
  final guest = (guestResult as Right<dynamic, User>).value;
  final nameResult = findUser(id).flatMap((u) => Right(u.name));
  if (nameResult case Left(:final value)) return Left(value);
  final name = (nameResult as Right<String, dynamic>).value;
  return Right('$name $stock ${guest.name}');
}
```

```nuclear
User(name: string)

findUser(id: string) Result<string, User> = Ok(User(id))

show(id: string) string
  match findUser(id)
    Ok(user) -> 'hi ${user.name}'
    Err(e) -> 'error: ${e}'

short(id: string) string
  kept: Result<string, User> = findUser(id)
  label = match kept
    Ok(user) -> user.name
    Err(e) -> e
  label
```

```dart
import 'package:fpdart/fpdart.dart';

class User {
  String name;

  User(this.name);
}

Either<String, User> findUser(String id) {
  return Right(User(id));
}

String show(String id) {
  final findUserResult = findUser(id);
  if (findUserResult.isRight()) {
    return 'hi ${(findUserResult as Right<String, User>).value.name}';
  } else {
    return 'error: ${(findUserResult as Left<String, User>).value}';
  }
}

String short(String id) {
  final Either<String, User> kept = findUser(id);
  final label = kept.isRight() ? (kept as Right<String, User>).value.name : (kept as Left<String, User>).value;
  return label;
}
```

### Bounded generics (base §4)

```nuclear
larger<T: Comparable>(a: T, b: T) T
  a.compareTo(b) > 0 ? a : b
```

```dart
T larger<T extends Comparable>(T a, T b) {
  return a.compareTo(b) > 0 ? a : b;
}
```

### Functions and return (base §4.1)

```nuclear
greet(name: string) string
  'Hello ${name}'

add(a: number, b: number) number = a + b

total(a: number, b: number)
  sum = a + b

record(a: number, b: number) void
  sum = a + b
```

```dart
String greet(String name) {
  return 'Hello $name';
}

num add(num a, num b) {
  return a + b;
}

total(num a, num b) {
  final sum = a + b;
  return sum;
}

void record(num a, num b) {
  final sum = a + b;
}
```

### `async` and `~` (base §4.1)

```nuclear
greet(name: string) string
  "Hello ${name}"

welcome() string
  text = ~greet("world")
  text

~greetAsync(name: string) string
  "Hello ${name}"
```

```dart
String greet(String name) {
  return 'Hello $name';
}

Future<String> welcome() async {
  final text = await greet('world');
  return text;
}

Future<String> greetAsync(String name) async {
  return 'Hello $name';
}
```

### Generators (base §4.1)

```nuclear
count(limit: int) int
  for i in 0..limit
    yield i

all(limit: int) int
  yield 0
  yield ...count(limit)

ticks() int
  for i in 0..3
    ~wait(1)
    yield i
```

```dart
Iterable<int> count(int limit) sync* {
  for (var i = 0; i < limit; i++) {
    yield i;
  }
}

Iterable<int> all(int limit) sync* {
  yield 0;
  yield* count(limit);
}

Stream<int> ticks() async* {
  for (var i = 0; i < 3; i++) {
    await wait(1);
    yield i;
  }
}
```

### Anonymous functions (base §4.1.1)

```nuclear
Network
  retry(attempt: int)
    log = (message: string)
      print(message)
    twice = (n: int) => n * 2
    connect()
```

```dart
class Network {
  retry(int attempt) {
    final log = (String message) => print(message);
    final twice = (int n) => n * 2;
    return connect();
  }
}
```

### Function aliases (base §4.1.2)

```nuclear
Character(name: string)

GetOneCharacter
  exec(id: int) Character
    Character('${id}')

GetAllCharacters
  ~exec() Character[]
    [Character('all')]

CharactersComponent()
  getOneCharacter() = GetOneCharacter.exec
  getCharacters() = GetAllCharacters.exec

  first() Character
    getOneCharacter(1)
```

```dart
class Character {
  String name;

  Character(this.name);
}

class GetOneCharacter {
  Character exec(int id) {
    return Character('$id');
  }
}

class GetAllCharacters {
  Future<List<Character>> exec() async {
    return [Character('all')];
  }
}

class CharactersComponent {
  Character getOneCharacter(int id) {
    return GetOneCharacter().exec(id);
  }

  Future<List<Character>> getCharacters() async {
    return await GetAllCharacters().exec();
  }

  Character first() {
    return getOneCharacter(1);
  }
}
```

### Modules, attributes and constructor (base §4.2)

```nuclear
User()
  name: string
  email: string
  role: Role = Role.user
  token?: string

  ()
    name = "name assignment"

  isAdmin() boolean
    role == Role.admin

Admin
  permissions: string[] = []
```

```dart
class User {
  late String name;
  late String email;
  Role role = Role.user;
  String? token;

  User() {
    name = 'name assignment';
  }

  bool isAdmin() {
    return role == Role.admin;
  }
}

class Admin {
  List<String> permissions = [];
}
```

Constructor parameters and own attributes:

```nuclear
User(name: string, email: string, role: string = "")
  localAttr: string = ""

  someMethod(myOwnParam: string)
    name = myOwnParam
    localAttr = myOwnParam
```

```dart
class User {
  String name;
  String email;
  String role;
  String localAttr = '';

  User(this.name, this.email, [this.role = '']);

  someMethod(String myOwnParam) {
    name = myOwnParam;
    localAttr = myOwnParam;
  }
}
```

Constructor with a body:

```nuclear
User
  (name: string, email: string, role: string = "", localAttr: string = "")
    localAttr = "other value"
```

```dart
class User {
  String name;
  String email;
  String role;
  String localAttr;

  User(this.name, this.email, [this.role = '', this.localAttr = '']) {
    localAttr = 'other value';
  }
}
```

Parameters between `{}` (named parameters):

```nuclear
User({name: string, email: string, role: string = ""})
  someMethod({myOwnParam: string})
    name = myOwnParam
```

```dart
class User {
  String name;
  String email;
  String role;

  User({required this.name, required this.email, this.role = ''});

  someMethod({required String myOwnParam}) {
    name = myOwnParam;
  }
}
```

`$` for the attribute:

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

```dart
class User {
  String name;
  String email;
  String role;

  User(this.name, this.email, [this.role = '']) {
    this.localAttr = 'other value';
  }

  String localAttr = '';

  someMethod(String name) {
    this.name = name;
  }
}
```

### Inheritance (base §4.2)

```nuclear
Serializable
  toJson() string

Admin: User > Serializable
  toJson() string
    "Say HI!"
```

```dart
abstract class Serializable {
  String toJson();
}

class Admin extends User implements Serializable {
  @override
  String toJson() {
    return 'Say HI!';
  }
}
```

Constructor of an inheriting module:

```nuclear
Animal
  (name: string)

Dog: Animal
  (name: string, breed: string)
    super(name)

  describe() string
    'dog'
```

```dart
class Animal {
  String name;

  Animal(this.name);
}

class Dog extends Animal {
  String breed;

  Dog(super.name, this.breed);

  String describe() {
    return 'dog';
  }
}
```

### Interfaces and enums (base §4.3)

```nuclear
Message
  role: 'user' | 'assistant' | 'system'
  content: string
  name?: string
  some(name: string) void

Role
  user
  admin

Level
  low("Low")
  high("High")
  medium
```

```dart
abstract class Message {
  String get role;
  String get content;
  String? get name;
  void some(String name);
}

enum Role {
  user,
  admin
}

enum Level {
  low('Low'),
  high('High'),
  medium('medium');

  const Level(this.value);

  final String value;
}
```

### Operators (base §6.1)

```nuclear
equal = a == b
both = active and not admin
isUser = x is User
data = read(file) as <string, int>
name = user!.name
```

```dart
void main() {
  final equal = a == b;
  final both = active && !admin;
  final isUser = x is User;
  final data = read(file) as Map<String, int>;
  final name = user!.name;
}
```

### Named arguments and construction (base §6.2, §6.3)

```nuclear
User(name: string, email: string)

createUser(name: string, admin: boolean = false) void
  print(name)

caller(url: string) void
  createUser(admin = true, name = 'Ana')
  fetch(url, method = 'POST')
  user = User(name = 'Ana', email = 'a@x.com')
```

```dart
class User {
  String name;
  String email;

  User(this.name, this.email);
}

void createUser(String name, [bool admin = false]) {
  print(name);
}

void caller(String url) {
  createUser('Ana', true);
  fetch(url, method: 'POST');
  final user = User('Ana', 'a@x.com');
}
```

### Lambdas (base §6.4)

```nuclear
User(name: string, age: int = 0)

users = (1..10).map(iterator: int)
  User(name = "Julian")
active = usersRepo.get(active = true)
filtered = active.filter(user: User)
  user.age > 18
doubled = numbers.map(n => n * 2)
adults = users.filter((user: User) => user.age >= 18)
label = users.map(user: User)
  user.name.upper()
.join(', ')
```

```dart
class User {
  String name;
  int age;

  User(this.name, [this.age = 0]);
}

void main() {
  final users = List.generate(10 - 1, (i) => 1 + i).map((int iterator) => User('Julian')).toList();
  final active = usersRepo.get(active: true);
  final filtered = active.where((User user) => user.age > 18).toList();
  final doubled = numbers.map((n) => n * 2).toList();
  final adults = users.where((User user) => user.age >= 18).toList();
  final label = users.map((User user) => user.name.toUpperCase()).toList().join(', ');
}
```

Lambda with a block as an argument:

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
```

```dart
class Form {
  int counter = 0;

  render() {
    return Button(
      label: 'Save',
      onClick: () {
        counter += 1;
        return save(counter);
      },
      disabled: false,
    );
  }
}
```

### Cascades (base §6.5)

```nuclear
brushes() void
  brush = Paint()
    ..color = Colors.blue
    ..strokeWidth = 4
  draw(Paint()
    ..color = Colors.red)
```

```dart
void brushes() {
  final brush = Paint()
    ..color = Colors.blue
    ..strokeWidth = 4;
  draw(
    Paint()
      ..color = Colors.red,
  );
}
```

### Control flow (base §7)

```nuclear
if x > 0
  print('positive')
else if x < 0
  print('negative')
else
  print('zero')
for user in users
  print(user)
for i in 0..10
  print(i)
for key, value in config
  print(key)
while pending > 0
  pending -= 1
match event.kind
  'message' -> process(event)
  'error' ->
    print(event.error)
    process(event)
  _ -> print('other')
try
  process(event)
catch error
  print(error)
finally
  print('done')
throw Error('something went wrong')
```

```dart
class NuclearError implements Exception {
  final String message;

  NuclearError(this.message);

  @override
  String toString() => message;
}

void main() {
  if (x > 0) {
    print('positive');
  } else if (x < 0) {
    print('negative');
  } else {
    print('zero');
  }
  for (final user in users) {
    print(user);
  }
  for (var i = 0; i < 10; i++) {
    print(i);
  }
  for (final MapEntry(key: key, value: value) in config.entries) {
    print(key);
  }
  while (pending > 0) {
    pending -= 1;
  }
  if (event.kind == 'message') {
    process(event);
  } else if (event.kind == 'error') {
    print(event.error);
    process(event);
  } else {
    print('other');
  }
  try {
    process(event);
  } catch (error) {
    print(error);
  } finally {
    print('done');
  }
  throw NuclearError('something went wrong');
}
```

## 5. The standard library

[library.md](library.md) as `dialects/dart/library.toml` writes it: lists, maps and sets are Dart's; `Bytes` is
`Uint8List`; `DateTime` and `Duration` are Dart's (`Duration(hours = 36)` is `Duration(hours: 36)`); `Regex` is
`RegExp` and `Match` is `RegExpMatch` (`m.text` is `m.group(0)!`, `m.group(name)` is `namedGroup`); `Error(message)`
is a `NuclearError` (an `Exception` with a `message`, written once per file) and `e.message` is `e.toString()`,
which also works for any other thrown object. `xs.map(f)` and `xs.filter(f)` end with `.toList()`; `find` is
`where(f).firstOrNull`; `reverse` in place, `min`, `max`, `sortBy`, `groupBy`, `cbrt`, `hypot` and `Bytes.fromHex`
are helpers written once per file. `dart:math` and `dart:convert` are imported when their names are written.
`string.length` counts UTF-16 code units; `chars()` gives the characters.
