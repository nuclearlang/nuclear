# The standard library of Nuclear

> Part of the base language ([nuclear-base.md](nuclear-base.md)). The members of Nuclear's base types and its few
> objects, written the same in every dialect (ADR-0094). Each dialect translates them into the target's expert
> code (`xs.length` → `len(xs)` in Python, `xs.add(x)` → `xs.push(x)` in TypeScript). Type definitions of
> libraries ([definitions.md](definitions.md)) are not part of this: `numpy`, `dio` or `Text(...)` are written as
> the library names them, and tie the code to that dialect.

## Rules

- **One name per operation.** Properties (no parentheses) for cheap queries: `length`, `isEmpty`, `first`, `last`,
  `keys`, `values`, `year`. Methods for everything else.
- **The receiver's type decides the member.** It is known from a declared type, a literal, a library member, a
  declared function or a type definition. When it is not known and the name belongs to two types that are not
  written the same way, the transpiler asks for a type on the receiver (a fix in the diagnostic).
- **A target's name on a base type is an error**, with the Nuclear name as the fix: `xs.push(x)` → "use `add`",
  `len(xs)` → "use `xs.length`", `s.toUpperCase()` → "use `upper`".
- **Membership is the `in` operator** (`x in xs`, `"b" in text`, `key in map`, `x in set`): there is no `contains`
  or `has`.
- Generated code pays nothing: a property is a property or a call of the target, a method is the target's method
  or a one-line helper of the dialect, never a runtime layer.
- `examples/stdlib/*.nuc` run in every language dialect and print the same text (`tools/stdlib-check.sh`).

## 1. `string`

| Member | Gives | Meaning |
|---|---|---|
| `length` | `int` | number of characters (code units) |
| `isEmpty` | `boolean` | `length == 0` |
| `s[i]` | `string` | the character at `i` |
| `codeAt(i)` | `int` | the code unit at `i` |
| `upper()` `lower()` | `string` | case |
| `trim()` `trimStart()` `trimEnd()` | `string` | without surrounding whitespace |
| `startsWith(text)` `endsWith(text)` | `boolean` | |
| `indexOf(text)` `lastIndexOf(text)` | `int` | position, `-1` when absent |
| `substring(start, end?)` | `string` | `end` exclusive, to the end when omitted |
| `split(separator)` | `string[]` | `separator`: a `string` or a `Regex` |
| `chars()` | `string[]` | one element per character |
| `replace(from, to)` | `string` | the first occurrence |
| `replaceAll(from, to)` | `string` | every occurrence; `from`: `string` or `Regex`; `to`: `string` or `(m: Match) => string` |
| `repeat(n)` | `string` | |
| `padStart(width, fill = " ")` `padEnd(width, fill = " ")` | `string` | |
| `toInt()` `toDouble()` | `int?` `double?` | `null` when the text is not a number |
| `compareTo(other)` | `int` | `< 0`, `0`, `> 0` |
| `String.fromCodes(codes: int[])` | `string` | |

`+` joins strings; `'${a} ${b}'` interpolates; `"b" in text` searches.

## 2. Lists: `T[]`

| Member | Gives | Meaning |
|---|---|---|
| `length` `isEmpty` | `int` `boolean` | |
| `first` `last` | `T` | an error on an empty list |
| `xs[i]` · `xs[i] = v` | `T` | read, write |
| `add(x)` `addAll(ys)` `insert(i, x)` | `void` | in place |
| `remove(x)` | `boolean` | removes the first equal element; whether it was there |
| `removeAt(i)` `removeLast()` | `T` | in place |
| `clear()` | `void` | |
| `indexOf(x)` `lastIndexOf(x)` | `int` | `-1` when absent |
| `map(f)` | `R[]` | `f: (x: T) => R` |
| `filter(f)` | `T[]` | the elements where `f` is true |
| `find(f)` | `T?` | the first element where `f` is true |
| `findIndex(f)` | `int` | `-1` when none |
| `any(f)` `all(f)` | `boolean` | |
| `count(f)` | `int` | how many satisfy `f` |
| `reduce(initial, f)` | `R` | `f: (acc: R, x: T) => R` |
| `sum()` | `int` or `double` | numbers only |
| `min()` `max()` | `T?` | `null` on an empty list |
| `join(separator = "")` | `string` | |
| `slice(start, end?)` `take(n)` `skip(n)` | `T[]` | new lists |
| `reversed()` `sorted()` `sortedBy(key)` | `T[]` | new lists; `key: (x: T) => K` |
| `reverse()` `sort()` `sortBy(key)` | `void` | in place |
| `unique()` | `T[]` | first occurrences, in order |
| `flatten()` | `T[]` | of a `T[][]` |
| `copy()` | `T[]` | a shallow copy |
| `groupBy(key)` | `<K, T[]>` | in order of first appearance |

`[...xs, ...ys]` concatenates, `[first, ...rest] = xs` destructures, `x in xs` searches, `for x in xs` iterates,
`0..n` is a range (a list of `int`).

## 3. Maps: `<K, V>`

| Member | Gives | Meaning |
|---|---|---|
| `length` `isEmpty` | `int` `boolean` | |
| `m[k]` | `V?` | `null` when the key is absent |
| `m[k] = v` | `void` | adds or replaces |
| `remove(k)` | `V?` | the value removed |
| `keys` `values` | `K[]` `V[]` | in insertion order |

`{ "a": 1 }` is a literal (an empty map needs its type: `ages: <string, int> = {}`), `k in m` asks for a key,
`for k, v in m` iterates. There is no `get`: `m[k] ?? fallback`.

## 4. Sets: `Set<T>`

`Set<T>()` and `Set.of(xs)` build one. `length`, `isEmpty`, `add(x)` (`boolean`: whether it was new), `remove(x)`
(`boolean`), `x in s`, `toList()`, `union(other)`, `intersection(other)`, `difference(other)` (new sets).

## 5. Numbers: `int` and `double`

| Member | Gives | Meaning |
|---|---|---|
| `abs()` | same type | |
| `sign()` | `int` | `-1`, `0`, `1` |
| `round()` `floor()` `ceil()` `truncate()` | `int` | `round` is half away from zero |
| `toFixed(digits)` | `string` | fixed decimals |
| `toInt()` `toDouble()` `toString()` | | conversion |
| `toRadix(base)` | `string` | `255.toRadix(16)` is `"ff"` |
| `isNaN` `isFinite` | `boolean` | `double` |

`Math`: `Math.pi`, `Math.e`, `Math.sqrt(x)`, `Math.cbrt(x)`, `Math.hypot(a, b)`, `Math.sin(x)`, `Math.cos(x)`,
`Math.tan(x)`, `Math.asin(x)`, `Math.acos(x)`, `Math.atan(x)`, `Math.atan2(y, x)`, `Math.exp(x)`, `Math.log(x)`,
`Math.log10(x)`, `Math.log2(x)`, `Math.min(a, b)`, `Math.max(a, b)`, `Math.clamp(x, low, high)`, `Math.random()`
(a `double` in `[0, 1)`). Powers are `**`, integer division `~/`, the rest as in [nuclear-base.md](nuclear-base.md)
§6.1. Parsing is `"42".toInt()`.

## 6. `boolean`

`toString()` gives `"true"` or `"false"`; `not`, `and`, `or` are the operators.

## 7. `Json`

`Json.encode(value)` gives the JSON text of a value (models through their JSON form); `Json.decode(text)` gives
`any` (maps, lists, strings, numbers, booleans, `null`).

## 8. `Regex` and `Match`

`Regex(pattern, ignoreCase = false, multiline = false)`; patterns are written as raw strings (`r"\d+"`).
`test(text)` (`boolean`), `find(text)` (`Match?`), `findAll(text)` (`Match[]`). A `Match` has `text`, `start`,
`end`, `m[i]` (group `i`, `string?`) and `group(name)` (`string?`). The string members `split`, `replace`,
`replaceAll` take a `Regex`.

## 9. `DateTime` and `Duration`

`DateTime.now()`, `DateTime.utc(year, month, day, hour = 0, minute = 0, second = 0, millisecond = 0)`,
`DateTime(year, month, ...)` (local), `DateTime.parse(iso)` (`DateTime?`), `DateTime.fromMillis(ms)`.
Properties: `year`, `month` (1–12), `day`, `hour`, `minute`, `second`, `millisecond`, `weekday` (1 Monday … 7
Sunday), `millis` (since the epoch, UTC), `isUtc`. Methods: `add(d)`, `subtract(d)` (a `Duration`),
`difference(other)` (`Duration`), `isBefore(other)`, `isAfter(other)`, `toUtc()`, `toLocal()`, `toIso()`.

`Duration(days = 0, hours = 0, minutes = 0, seconds = 0, milliseconds = 0)`; `inDays`, `inHours`, `inMinutes`,
`inSeconds`, `inMillis` (whole units); `+`, `-`, comparisons.

## 10. Asynchrony

A value not awaited yet has the type `Future<T>`. `~delay(ms)` waits; `~awaitAll([a(), b()])` waits for all and
gives their results in order. `~` and generators: [nuclear-base.md](nuclear-base.md) §4.1.

## 11. `Bytes`

`text.toBytes()` (UTF-8) and `bytes.toText()`; `Bytes.of(ints)`, `Bytes.fromBase64(text)`, `Bytes.fromHex(text)`;
`length`, `bytes[i]` (`int`), `slice(start, end?)`, `toList()` (`int[]`), `toBase64()`, `toHex()`.

## 12. Output and errors

`print(a, b, ...)` writes the values separated by spaces and a newline. `x.toString()` exists on every value;
only numbers, booleans, strings, `DateTime` (`toIso()`) and `Duration` print the same text in every dialect: a
list or a map is printed through `join` or `Json.encode`.

`Error(message)` is the base error (`e.message`); `throw`, `try`/`catch` and `Result` are in
[nuclear-base.md](nuclear-base.md) §3.2 and §7.
