//! The standard library of Nuclear (spec/library.md, ADR-0094): the members of the base types and of its few
//! objects. Rust decides *what* exists (names, properties against methods, what each member gives); each language
//! dialect decides *how* it is written (`[library.<base>]` in its `library.toml`).

/// What a member gives, in terms of the receiver's type: the emitter follows it to type chained calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ret {
    Str,
    StrOpt,
    Int,
    IntOpt,
    Double,
    DoubleOpt,
    Bool,
    Void,
    /// The receiver's own type (`trim` on a string, `abs` on an int).
    Same,
    /// The element type: of a list, the value of a map, the character of a string.
    Elem,
    ElemOpt,
    /// A list of the receiver's elements.
    ListElem,
    ListStr,
    ListInt,
    /// A list whose element type is not known (`map`, `flatten`).
    ListAny,
    /// `keys` / `values` of a map.
    Keys,
    Values,
    /// `groupBy`: a map from the key to lists of the elements.
    GroupMap,
    /// A number of the receiver's kind (`sum` of ints is an int).
    Num,
    MatchOpt,
    ListMatch,
    DateTime,
    DateTimeOpt,
    Duration,
    Bytes,
    Regex,
    /// A `Set` whose element type is not known.
    SetAny,
    /// The base error (`Error(message)`).
    Error,
    Any,
}

/// A member of a base type (`base` is a key of `BASES`) or of a static object (`base` is `Math`, `Json`...).
#[derive(Debug, Clone, Copy)]
pub struct Member {
    pub base: &'static str,
    pub name: &'static str,
    /// Written without parentheses (`xs.length`); a method is called.
    pub property: bool,
    pub returns: Ret,
}

const fn p(base: &'static str, name: &'static str, returns: Ret) -> Member {
    Member { base, name, property: true, returns }
}

const fn m(base: &'static str, name: &'static str, returns: Ret) -> Member {
    Member { base, name, property: false, returns }
}

/// The base types, as the `[library.<base>]` sections name them. `number` holds what `int` and `double` share;
/// `any` holds what every value has (`toString`).
pub const BASES: &[&str] = &[
    "string", "list", "map", "set", "int", "double", "number", "boolean", "bytes", "datetime", "duration", "regex",
    "match", "error", "any",
];

/// The static objects: `Math.sqrt(x)`, `Json.encode(v)`, `Regex(...)` (`new`), `DateTime.now()`.
pub const OBJECTS: &[&str] = &["Math", "Json", "String", "DateTime", "Duration", "Regex", "Set", "Bytes", "Error"];

use Ret::*;

/// Every member of the standard library. A language dialect renders each one (N0203 when one is missing).
pub const MEMBERS: &[Member] = &[
    // string
    p("string", "length", Int),
    p("string", "isEmpty", Bool),
    m("string", "codeAt", Int),
    m("string", "upper", Str),
    m("string", "lower", Str),
    m("string", "trim", Str),
    m("string", "trimStart", Str),
    m("string", "trimEnd", Str),
    m("string", "startsWith", Bool),
    m("string", "endsWith", Bool),
    m("string", "indexOf", Int),
    m("string", "lastIndexOf", Int),
    m("string", "substring", Str),
    m("string", "split", ListStr),
    m("string", "chars", ListStr),
    m("string", "replace", Str),
    m("string", "replaceAll", Str),
    m("string", "repeat", Str),
    m("string", "padStart", Str),
    m("string", "padEnd", Str),
    m("string", "toInt", IntOpt),
    m("string", "toDouble", DoubleOpt),
    m("string", "compareTo", Int),
    m("string", "toBytes", Bytes),
    // list
    p("list", "length", Int),
    p("list", "isEmpty", Bool),
    p("list", "first", Elem),
    p("list", "last", Elem),
    m("list", "add", Void),
    m("list", "addAll", Void),
    m("list", "insert", Void),
    m("list", "remove", Bool),
    m("list", "removeAt", Elem),
    m("list", "removeLast", Elem),
    m("list", "clear", Void),
    m("list", "indexOf", Int),
    m("list", "lastIndexOf", Int),
    m("list", "map", ListAny),
    m("list", "filter", ListElem),
    m("list", "find", ElemOpt),
    m("list", "findIndex", Int),
    m("list", "any", Bool),
    m("list", "all", Bool),
    m("list", "count", Int),
    m("list", "reduce", Any),
    m("list", "sum", Num),
    m("list", "min", ElemOpt),
    m("list", "max", ElemOpt),
    m("list", "join", Str),
    m("list", "slice", ListElem),
    m("list", "take", ListElem),
    m("list", "skip", ListElem),
    m("list", "reversed", ListElem),
    m("list", "sorted", ListElem),
    m("list", "sortedBy", ListElem),
    m("list", "reverse", Void),
    m("list", "sort", Void),
    m("list", "sortBy", Void),
    m("list", "unique", ListElem),
    m("list", "flatten", ListAny),
    m("list", "copy", ListElem),
    m("list", "groupBy", GroupMap),
    // map
    p("map", "length", Int),
    p("map", "isEmpty", Bool),
    m("map", "remove", ElemOpt),
    p("map", "keys", Keys),
    p("map", "values", Values),
    // set
    p("set", "length", Int),
    p("set", "isEmpty", Bool),
    m("set", "add", Bool),
    m("set", "remove", Bool),
    m("set", "toList", ListElem),
    m("set", "union", Same),
    m("set", "intersection", Same),
    m("set", "difference", Same),
    // numbers (int and double)
    m("number", "abs", Same),
    m("number", "sign", Int),
    m("number", "round", Int),
    m("number", "floor", Int),
    m("number", "ceil", Int),
    m("number", "truncate", Int),
    m("number", "toFixed", Str),
    m("number", "toInt", Int),
    m("number", "toDouble", Double),
    m("number", "toRadix", Str),
    p("double", "isNaN", Bool),
    p("double", "isFinite", Bool),
    // booleans print "true" / "false" in every dialect
    m("boolean", "toString", Str),
    // every value
    m("any", "toString", Str),
    // bytes
    p("bytes", "length", Int),
    m("bytes", "toText", Str),
    m("bytes", "slice", Bytes),
    m("bytes", "toList", ListInt),
    m("bytes", "toBase64", Str),
    m("bytes", "toHex", Str),
    // datetime
    p("datetime", "year", Int),
    p("datetime", "month", Int),
    p("datetime", "day", Int),
    p("datetime", "hour", Int),
    p("datetime", "minute", Int),
    p("datetime", "second", Int),
    p("datetime", "millisecond", Int),
    p("datetime", "weekday", Int),
    p("datetime", "millis", Int),
    p("datetime", "isUtc", Bool),
    m("datetime", "add", DateTime),
    m("datetime", "subtract", DateTime),
    m("datetime", "difference", Duration),
    m("datetime", "isBefore", Bool),
    m("datetime", "isAfter", Bool),
    m("datetime", "toUtc", DateTime),
    m("datetime", "toLocal", DateTime),
    m("datetime", "toIso", Str),
    // duration
    p("duration", "inDays", Int),
    p("duration", "inHours", Int),
    p("duration", "inMinutes", Int),
    p("duration", "inSeconds", Int),
    p("duration", "inMillis", Int),
    // regex and match
    m("regex", "test", Bool),
    m("regex", "find", MatchOpt),
    m("regex", "findAll", ListMatch),
    p("match", "text", Str),
    p("match", "start", Int),
    p("match", "end", Int),
    m("match", "group", StrOpt),
    // the base error: `Error(message)`, `e.message`; a dialect's own errors add members (not checked)
    p("error", "message", Str),
    // static objects
    p("Math", "pi", Double),
    p("Math", "e", Double),
    m("Math", "sqrt", Double),
    m("Math", "cbrt", Double),
    m("Math", "hypot", Double),
    m("Math", "sin", Double),
    m("Math", "cos", Double),
    m("Math", "tan", Double),
    m("Math", "asin", Double),
    m("Math", "acos", Double),
    m("Math", "atan", Double),
    m("Math", "atan2", Double),
    m("Math", "exp", Double),
    m("Math", "log", Double),
    m("Math", "log10", Double),
    m("Math", "log2", Double),
    m("Math", "min", Num),
    m("Math", "max", Num),
    m("Math", "clamp", Num),
    m("Math", "random", Double),
    m("Json", "encode", Str),
    m("Json", "decode", Any),
    m("String", "fromCodes", Str),
    m("DateTime", "now", DateTime),
    m("DateTime", "utc", DateTime),
    m("DateTime", "new", DateTime),
    m("DateTime", "parse", DateTimeOpt),
    m("DateTime", "fromMillis", DateTime),
    m("Duration", "new", Duration),
    m("Regex", "new", Regex),
    m("Set", "new", SetAny),
    m("Set", "of", SetAny),
    m("Bytes", "of", Bytes),
    m("Bytes", "fromBase64", Bytes),
    m("Bytes", "fromHex", Bytes),
    m("Error", "new", Error),
];

/// The member exactly as a `[library.<base>]` section declares it (`number`, `any` and the objects included).
pub fn member_of(base: &str, name: &str) -> Option<&'static Member> {
    MEMBERS.iter().find(|m| m.base == base && m.name == name)
}

/// The member `name` of `base`, looking in `number` for `int` and `double` and in `any` for every base.
pub fn member(base: &str, name: &str) -> Option<&'static Member> {
    let find = |b: &str| MEMBERS.iter().find(|m| m.base == b && m.name == name);
    find(base)
        .or_else(|| if base == "int" || base == "double" { find("number") } else { None })
        .or_else(|| if BASES.contains(&base) && base != "any" { find("any") } else { None })
}

/// The bases (not the static objects) that have a member called `name`: `length` → string, list, map, set, bytes.
pub fn bases_with(name: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for m in MEMBERS.iter().filter(|m| m.name == name && BASES.contains(&m.base)) {
        let bases: &[&str] = match m.base {
            "number" => &["int", "double"],
            "any" => &[],
            b => std::slice::from_ref(bases_entry(b)),
        };
        for b in bases {
            if !out.contains(b) {
                out.push(b);
            }
        }
    }
    out
}

fn bases_entry(b: &str) -> &'static &'static str {
    BASES.iter().find(|x| **x == b).expect("a base of the library")
}

/// A target's name on a base type, with the Nuclear name that replaces it (N0114). `*` applies to every base.
pub const WRONG: &[(&str, &str, &str)] = &[
    ("list", "push", "add(x)"),
    ("list", "append", "add(x)"),
    ("list", "extend", "addAll(ys)"),
    ("list", "concat", "addAll(ys) or [...xs, ...ys]"),
    ("list", "pop", "removeLast()"),
    ("list", "shift", "removeAt(0)"),
    ("list", "unshift", "insert(0, x)"),
    ("list", "includes", "the `in` operator (x in xs)"),
    ("list", "contains", "the `in` operator (x in xs)"),
    ("list", "some", "any(f)"),
    ("list", "every", "all(f)"),
    ("list", "where", "filter(f)"),
    ("list", "firstWhere", "find(f)"),
    ("list", "indexWhere", "findIndex(f)"),
    ("list", "fold", "reduce(initial, f)"),
    ("list", "forEach", "for x in xs"),
    ("list", "size", "length"),
    ("list", "isNotEmpty", "not xs.isEmpty"),
    ("list", "toList", "nothing: a list is already a list"),
    ("list", "sublist", "slice(start, end)"),
    ("list", "index", "indexOf(x)"),
    ("list", "removeWhere", "filter(f)"),
    ("list", "expand", "map(f).flatten()"),
    ("list", "at", "xs[i]"),
    ("list", "splice", "removeAt(i) or insert(i, x)"),
    ("list", "elementAt", "xs[i]"),
    ("string", "toUpperCase", "upper()"),
    ("string", "toLowerCase", "lower()"),
    ("string", "trimLeft", "trimStart()"),
    ("string", "trimRight", "trimEnd()"),
    ("string", "lstrip", "trimStart()"),
    ("string", "rstrip", "trimEnd()"),
    ("string", "strip", "trim()"),
    ("string", "includes", "the `in` operator (\"b\" in text)"),
    ("string", "contains", "the `in` operator (\"b\" in text)"),
    ("string", "charAt", "text[i]"),
    ("string", "charCodeAt", "codeAt(i)"),
    ("string", "codeUnitAt", "codeAt(i)"),
    ("string", "replaceFirst", "replace(from, to)"),
    ("string", "startswith", "startsWith(text)"),
    ("string", "endswith", "endsWith(text)"),
    ("string", "find", "indexOf(text)"),
    ("string", "rfind", "lastIndexOf(text)"),
    ("string", "substr", "substring(start, end)"),
    ("string", "slice", "substring(start, end)"),
    ("string", "padLeft", "padStart(width, fill)"),
    ("string", "padRight", "padEnd(width, fill)"),
    ("string", "ljust", "padEnd(width, fill)"),
    ("string", "rjust", "padStart(width, fill)"),
    ("string", "zfill", "padStart(width, \"0\")"),
    ("string", "splitlines", "split(\"\\n\")"),
    ("string", "isNotEmpty", "not text.isEmpty"),
    ("string", "size", "length"),
    ("string", "runes", "chars()"),
    ("string", "toStringAsFixed", "toFixed(digits)"),
    ("map", "containsKey", "the `in` operator (key in map)"),
    ("map", "has", "the `in` operator (key in map)"),
    ("map", "get", "map[key]"),
    ("map", "set", "map[key] = value"),
    ("map", "delete", "remove(key)"),
    ("map", "size", "length"),
    ("map", "entries", "for key, value in map"),
    ("map", "items", "for key, value in map"),
    ("map", "isNotEmpty", "not map.isEmpty"),
    ("set", "contains", "the `in` operator (x in set)"),
    ("set", "has", "the `in` operator (x in set)"),
    ("set", "size", "length"),
    ("set", "isNotEmpty", "not set.isEmpty"),
    ("int", "toStringAsFixed", "toFixed(digits)"),
    ("double", "toStringAsFixed", "toFixed(digits)"),
    ("int", "toPrecision", "toFixed(digits)"),
    ("double", "toPrecision", "toFixed(digits)"),
    ("int", "trunc", "truncate()"),
    ("double", "trunc", "truncate()"),
    ("*", "toStr", "toString()"),
];

/// The fix for a target's name on a base type.
pub fn wrong(base: &str, name: &str) -> Option<&'static str> {
    WRONG.iter().find(|(b, n, _)| (*b == base || *b == "*") && *n == name).map(|(_, _, fix)| *fix)
}

/// A target's function on a value of a base type, with the Nuclear member that replaces it (N0114).
pub const WRONG_FUNCTIONS: &[(&str, &str)] = &[
    ("len", "x.length"),
    ("str", "x.toString()"),
    ("String", "x.toString()"),
    ("int", "x.toInt()"),
    ("float", "x.toDouble()"),
    ("parseInt", "x.toInt()"),
    ("parseFloat", "x.toDouble()"),
    ("Number", "x.toDouble()"),
    ("sorted", "xs.sorted()"),
    ("reversed", "xs.reversed()"),
    ("sum", "xs.sum()"),
    ("min", "xs.min() or Math.min(a, b)"),
    ("max", "xs.max() or Math.max(a, b)"),
    ("abs", "x.abs()"),
    ("round", "x.round()"),
    ("jsonEncode", "Json.encode(value)"),
    ("jsonDecode", "Json.decode(text)"),
];

/// A target's static member on one of the objects (`Math.floor(x)` → `x.floor()`).
pub const WRONG_STATIC: &[(&str, &str, &str)] = &[
    ("console", "log", "print(a, b)"),
    ("Math", "floor", "x.floor()"),
    ("Math", "ceil", "x.ceil()"),
    ("Math", "round", "x.round()"),
    ("Math", "trunc", "x.truncate()"),
    ("Math", "abs", "x.abs()"),
    ("Math", "pow", "the `**` operator"),
    ("Math", "sign", "x.sign()"),
    ("Math", "PI", "Math.pi"),
    ("Math", "E", "Math.e"),
    ("JSON", "stringify", "Json.encode(value)"),
    ("JSON", "parse", "Json.decode(text)"),
    ("Date", "now", "DateTime.now()"),
    ("Number", "parseInt", "text.toInt()"),
    ("Number", "parseFloat", "text.toDouble()"),
    ("String", "fromCharCode", "String.fromCodes(codes)"),
    ("String", "fromCharCodes", "String.fromCodes(codes)"),
    ("RegExp", "new", "Regex(pattern)"),
    ("int", "parse", "text.toInt()"),
    ("double", "parse", "text.toDouble()"),
    ("DateTime", "fromMillisecondsSinceEpoch", "DateTime.fromMillis(ms)"),
];

/// The variables a `[library.<base>]` template receives.
pub const VARS: &[&str] = &[
    "recv", "args", "has_args", "arg0", "arg1", "arg2", "arg3", "arg4", "arg5", "arg6", "n", "fn", "fn.param",
    "fn.params", "fn.p0", "fn.p1", "fn.body", "named", "type_args", "elem", "arg0_is", "arg1_is", "arg2_is", "call_args",
];
