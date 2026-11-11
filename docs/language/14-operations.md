# 14. Operations and the standard library

Nuclear's syntax covers every operation of the mainstream languages: arithmetic, power, integer division, bitwise
operators, shifts, raw text for regular expressions, type checks and conversions. The members of the base types
(`string`, lists, maps, sets, numbers, `Bytes`, `DateTime`, `Duration`, `Regex`) and the few objects (`Math`,
`Json`, `print`, `delay`, `awaitAll`) are **Nuclear's own standard library** ([library.md](../../spec/library.md),
`nuclear help library`): one name per operation, the same in every dialect. Each dialect writes it as an expert of
the target would (`xs.length` is `len(xs)` in Python and `xs.length` in TypeScript; `xs.add(x)` is `xs.push(x)`,
`xs.add(x)` and `xs.append(x)`), with no wrapper and no runtime layer. A target's name on a base type is an error
with the Nuclear name as the fix: `xs.push(x)` → "use `add(x)`", `len(xs)` → "use `xs.length`".

Each area has one program in [examples/stdlib/](../../examples/stdlib) that prints the same text in TypeScript,
Dart and Python; `tools/stdlib-check.sh` runs the three (`tsc --strict` + node, `dart run`, `pyright --strict` +
python).

## Operators

| Operation | Nuclear | Notes |
|---|---|---|
| power | `a ** b` | Dart writes `pow(a, b)` with the conversion the operand types ask for |
| integer division | `a ~/ b` | |
| remainder | `a % b` | follows the target: `-7 % 3` is `-1` in TypeScript and `2` in Dart and Python |
| and, or, xor | `a & b`, `a \| b`, `a ^ b` | 32 bits in TypeScript, 64 in Dart, unbounded in Python |
| complement | `^a` | |
| shifts | `a << n`, `a >> n`, `a >>> n` | |
| compound | `x %= 3`, `x &= m`, `x \|= m`, `x ^= m`, `x <<= 1`, `x >>= 1`, `x >>>= 1`, `x **= 2`, `x ??= y` | a target without the compound form gets `x = x op y` |
| literals | `0b1010`, `0xff`, `1_000`, `1.5e-3` | |
| type check | `x is string`, `x is int`, `x is int[]`, `x is User` | `x is int` never accepts a boolean |
| conversion | `x as int` | |
| membership | `x in list`, `"b" in text`, `key in map`, `x in set` | the only way to ask "contains" |

## The library, per area

| Area | Nuclear |
|---|---|
| roots, power, logs | `Math.sqrt(x)`, `Math.cbrt(x)`, `Math.hypot(a, b)`, `Math.exp(x)`, `Math.log(x)`, `Math.log10(x)`, `Math.log2(x)` |
| trigonometry | `Math.sin(x)`... `Math.atan2(y, x)`, `Math.pi`, `Math.e` |
| rounding | `x.floor()`, `x.ceil()`, `x.round()` (half away from zero), `x.truncate()`, `x.toFixed(2)` |
| min, max, clamp | `Math.min(a, b)`, `Math.max(a, b)`, `Math.clamp(x, low, high)`, `xs.min()`, `xs.max()` |
| parse, radix | `"42".toInt()` (`int?`), `"3.5".toDouble()`, `n.toRadix(16)`, `x.toInt()`, `x.toDouble()` |
| random | `Math.random()` |
| text | `length`, `isEmpty`, `upper()`, `lower()`, `trim()`, `trimStart()`, `trimEnd()`, `startsWith`, `endsWith`, `indexOf`, `substring`, `split`, `chars()`, `replace`, `replaceAll`, `repeat`, `padStart`, `padEnd`, `codeAt`, `compareTo`, `toBytes()` |
| regex | `Regex(r"\d+", ignoreCase = true)`, `re.test(s)`, `re.find(s)` (`Match?`), `re.findAll(s)`, `s.replaceAll(re, (m) => m.text)`, `s.split(re)`; a `Match` has `text`, `start`, `end`, `m[1]`, `m.group("name")` |
| lists | `map`, `filter`, `find`, `findIndex`, `any`, `all`, `count`, `reduce(initial, f)`, `sum`, `join`, `slice`, `take`, `skip`, `sorted`, `sortedBy`, `reversed`, `unique`, `flatten`, `groupBy`; in place: `add`, `addAll`, `insert`, `remove`, `removeAt`, `removeLast`, `clear`, `sort`, `sortBy`, `reverse` |
| maps, sets | `m[k]` (`V?`), `m[k] = v`, `m.remove(k)`, `m.keys`, `m.values`, `key in m`; `Set.of([...])`, `s.add(x)`, `s.remove(x)`, `x in s`, `union`, `intersection`, `difference`, `toList()` |
| bytes | `text.toBytes()`, `bytes.toText()`, `toHex()`, `toBase64()`, `Bytes.fromHex`, `Bytes.fromBase64`, `Bytes.of([...])`, `slice`, `toList()` |
| dates | `DateTime.utc(2026, 11, 3, 14, 30)`, `DateTime.now()`, `DateTime.parse(iso)` (`DateTime?`), `d.year`... `d.weekday`, `d.millis`, `d.toIso()`, `d.add(Duration(hours = 36))`, `d.difference(other).inDays` |
| JSON | `Json.encode(value)`, `Json.decode(text)` |
| async | `~delay(ms)`, `~awaitAll([a(), b()])`, `Future<T>` for a value not awaited yet |
| generators | `yield value` in a function whose type is the type of each value |
| errors | `Error("message")`, `e.message` in `catch e` |
| output | `print(a, b)`: values separated by spaces; booleans print `true`/`false` and null `null` in every dialect |

## Physics and formulas

Formulas read as in a book, because `**`, the decimal literals and `Math` are part of the language:

```nuclear
g = 9.80665

kineticEnergy(mass: double, speed: double) double = 0.5 * mass * speed ** 2

projectileRange(speed: double, degrees: double) double
  radians = degrees * Math.pi / 180
  speed ** 2 * Math.sin(2 * radians) / g

gravity(m1: double, m2: double, distance: double) double = 6.674e-11 * m1 * m2 / distance ** 2
```

## How the dialect knows the type

A member of the library is written for the type of the value before the dot, which Nuclear knows from a declared
type, a literal, another library member, a declared function or a type definition. When it is not known and the
member is written differently for several types in that dialect (`length` of a list and of a set in TypeScript),
the transpiler asks for a type on the value (`items: string[] = ...`). A class of the program or of a library
(`response.text()`, `repo.where(...)`) is never confused with the library: its members are its own.

## What differs between the targets

- `%` follows the target: `-7 % 3` is `-1` in TypeScript and `2` in Dart and Python.
- Bitwise operators work on 32-bit integers in TypeScript, on 64 bits in Dart and without limit in Python.
- `string.length` counts UTF-16 code units in TypeScript and Dart and characters in Python; `chars().length`
  counts characters everywhere.
- Dart's `pow` returns `num`: Nuclear adds `.toInt()` or `.toDouble()` when the operands' types are known; when
  they are not, convert it yourself.
- TypeScript's `Date` has no UTC flag: the parts of a `DateTime` are read in UTC.
