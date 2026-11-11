# Dialect `dart` brief

The library is Dart's: `dart:core`, `dart:math`, `dart:convert`, `dart:async`, `dart:io`, `dart:typed_data`...
are imported by themselves when a name is used (`sqrt`, `jsonEncode`, `File`). `nuclear defs show <Name> --dialect
dart` prints any class with its members.

| Nuclear | Dart |
|---|---|
| `x = 1` (not reassigned / reassigned) | `final x = 1;` / `var x = 1;` |
| `f(x = 1)` | `f(x: 1)` |
| `{name: string}` in the parameters | named parameters `{required String name}` |
| `name?: string` | `String? name` |
| `x in list`, `key in map` | `list.contains(x)`, `map.containsKey(key)` |
| `a ** b` | `pow(a, b)` (`.toInt()` / `.toDouble()` when the types are known) |
| `0b1010` | `0xa` |
| `print(a, b)` | `print([a, b].join(' '))` |
| `Result<E, T>`, `Ok`, `Err` | fpdart `Either<E, T>`, `Right`, `Left` |

Header words: `extension Name on Type`, `mixin Name on Type`, `data Name` (immutable, `copyWith`, `toJson`), modifiers
`static abstract late final const sealed base factory get set`, clause `with`. Top-level statements run in a
generated `main()`; top-level variables stay top-level.

```nuclear
Point(x: double, y: double)
  distance(other: Point) double
    sqrt((x - other.x) ** 2 + (y - other.y) ** 2)

lineCount(path: string) int
  text = ~File(path).readAsString()
  text.split("\n").length
```

```dart
import 'dart:io';
import 'dart:math';

class Point {
  double x;
  double y;

  Point(this.x, this.y);

  double distance(Point other) {
    return sqrt(pow(x - other.x, 2).toDouble() + pow(y - other.y, 2).toDouble());
  }
}

Future<int> lineCount(String path) async {
  final text = await File(path).readAsString();
  return text.split('\n').length;
}
```

```nuclear
extension Words on String
  words() string[]
    split(Regex(r"\s+")).filter((w) => not w.isEmpty)

data Size
  width: double
  height: double
```

```dart
extension Words on String {
  List<String> words() {
    return split(RegExp('\\s+')).where((w) => !w.isEmpty).toList();
  }
}

class Size {
  const Size({required this.width, required this.height});

  final double width;
  final double height;

  Size copyWith({double? width, double? height}) {
    return Size(width: width ?? this.width, height: height ?? this.height);
  }

  Map<String, dynamic> toJson() => {'width': width, 'height': height};
}
```

Common Dart names (not TypeScript's): `where` (filter), `fold` (reduce with a start), `contains`, `padLeft`,
`toStringAsFixed`, `firstWhere`, `sublist`, `isEmpty`, `runtimeType`. `%` is Dart's modulo (never negative);
integers are 64-bit.
