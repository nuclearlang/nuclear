# 3. Types

Types are written after `:` in variables, parameters and attributes, and **without `:`** as the return type
of a function (`name() string`). They are optional where the target can infer them (variables, lambdas).

| Type | Written | Example |
|---|---|---|
| text | `string` | `name: string` |
| integer | `int` | `age: int` |
| decimal | `double` | `price: double` |
| number (TS) | `number` | `n: number` |
| boolean | `boolean` | `active: boolean` |
| no value | `void` | `log(m: string) void` |
| list | `Type[]` | `names: string[]` |
| map | `<Key, Value>` | `ages: <string, int>` |
| nullable | `Type?` | `user: User?` |
| optional (attribute or parameter) | `name?: Type` | `token?: string` |
| union | `A \| B` | `role: 'user' \| 'admin'` |
| module, interface or enum | its name | `owner: User` |
| generic | `Name<T>` | `box: Box<string>` |
| result (success or error) | `Result<E, T>` | `find(id: string) Result<string, User>` ([chapter 13](13-results.md)) |

```nuclear
Profile
  name: string
  age: int
  score: double
  active: boolean
  tags: string[]
  ages: <string, int>
  friends: <string, int>[]
  nickname?: string
  manager: Profile?
  role: 'user' | 'admin'
```

```typescript
export interface Profile {
  name: string;
  age: number;
  score: number;
  active: boolean;
  tags: string[];
  ages: Record<string, number>;
  friends: Record<string, number>[];
  nickname?: string;
  manager: Profile | null;
  role: 'user' | 'admin';
}
```

```dart
abstract class Profile {
  String get name;
  int get age;
  double get score;
  bool get active;
  List<String> get tags;
  Map<String, int> get ages;
  List<Map<String, int>> get friends;
  String? get nickname;
  Profile? get manager;
  String get role;
}
```

## Maps

- The type is `<Key, Value>` in any position: variable, parameter, return, list (`<string, int>[]`) and
  nullable (`<string, int>?`).
- The value always goes between `{ }` with text keys: `{"name": "Ana", "age": 30}`. Empty map: `{}`.

```nuclear
user(name: string) <string, string>
  {"name": name, "role": "user"}
```

```typescript
export function user(name: string): Record<string, string> {
  return { 'name': name, 'role': 'user' };
}
```

## Null

- `Type?` accepts `null`.
- `a ?? b`: `a`, or `b` if `a` is null.
- `a?.b`: reads `b` only if `a` is not null.
- `a!`: asserts that `a` is not null (it goes right after the name, `)` or `]`).

More in [Expressions](09-expressions.md).
