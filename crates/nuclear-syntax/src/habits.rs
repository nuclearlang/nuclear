//! Habits of other languages (TypeScript, Dart, Python, Java, Go...): what an agent or a person writes by reflex.
//! Each one is N0022 with the Nuclear form and, when it is exact, a fix. `parse` drops the other errors of those
//! lines: they would only repeat the same mistake in worse words.

use nuclear_diag::{Diagnostic, Span, codes::FOREIGN_HABIT};

/// Words that start a block statement or a declaration in other languages (a `:` or `{` after them is a habit).
const BLOCK_WORDS: &[&str] =
    &["if", "else", "elif", "for", "while", "try", "except", "catch", "finally", "def", "class", "with", "switch"];
/// Words that come before a return type written first (`int f(x)`, `void main()`).
const TYPE_WORDS: &[&str] =
    &["int", "void", "string", "bool", "boolean", "double", "num", "number", "dynamic", "any", "float", "long"];

pub fn scan(src: &str) -> Vec<Diagnostic> {
    let (code, mut out) = mask(src);
    let bytes = code.as_bytes();
    let mut braces: Vec<bool> = Vec::new(); // true = a `{` opened as a block (a habit)
    let mut start = 0;
    while start < bytes.len() {
        let end = bytes[start..].iter().position(|&b| b == b'\n').map_or(bytes.len(), |p| start + p);
        line(src, &code[start..end], start, end, &mut braces, &mut out);
        start = end + 1;
    }
    out.sort_by_key(|d| d.span.start);
    out
}

fn habit(out: &mut Vec<Diagnostic>, at: usize, len: usize, msg: &str, fix: Option<&str>) {
    let span = Span::new(at, at + len);
    let d = Diagnostic::error(FOREIGN_HABIT, span, msg);
    out.push(match fix {
        Some(f) => d.with_fix(span, f),
        None => d,
    });
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}

/// The words of a masked line: (offset in the line, word).
fn words(l: &str) -> Vec<(usize, &str)> {
    let b = l.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if is_ident(b[i]) && (i == 0 || !is_ident(b[i - 1])) {
            let s = i;
            while i < b.len() && is_ident(b[i]) {
                i += 1;
            }
            out.push((s, &l[s..i]));
        } else {
            i += 1;
        }
    }
    out
}

fn line(src: &str, l: &str, at: usize, end: usize, braces: &mut Vec<bool>, out: &mut Vec<Diagnostic>) {
    let indent = l.len() - l.trim_start().len();
    let t = l.trim();
    if t.is_empty() {
        return;
    }
    let ws = words(l);
    let first = ws.first().filter(|(o, _)| *o == indent).map(|(_, w)| *w).unwrap_or("");
    let b = l.as_bytes();
    let prev = |i: usize| l[..i].trim_end().as_bytes().last().copied();

    // braces: a `{` after a header or `=>` opens a block; its `}` closes it
    let last_open = l.rfind('{').filter(|&i| l[i + 1..].trim().is_empty());
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'{' => {
                let block = Some(i) == last_open
                    && prev(i).is_some_and(|p| is_ident(p) || matches!(p, b')' | b'>'))
                    && !l[..i].trim_end().ends_with("->")
                    && !words(&l[..i]).last().is_some_and(|(o, w)| {
                        o + w.len() == l[..i].trim_end().len() && matches!(*w, "return" | "yield" | "in" | "and" | "or" | "not")
                    });
                braces.push(block);
                if block {
                    let s = l[..i].trim_end().len();
                    habit(out, at + s, i + 1 - s, "blocks use indentation, not braces: end the line here", Some(""));
                }
            }
            b'}' if braces.pop() == Some(true) => {
                if t == "}" {
                    let to = (end + 1).min(src.len());
                    habit(out, at, to - at, "blocks end when the indentation returns: delete this `}`", Some(""));
                } else if i == indent {
                    let n = 1 + l[i + 1..].len() - l[i + 1..].trim_start().len();
                    habit(out, at + i, n, "blocks end when the indentation returns: delete this `}`", Some(""));
                }
            }
            _ => {}
        }
    }

    // a `:` ending a block header (Python)
    if t.ends_with(':') && BLOCK_WORDS.contains(&first) {
        habit(out, at + l.trim_end().len() - 1, 1, "blocks start on the next indented line: delete the `:`", Some(""));
    }

    for (k, &(o, w)) in ws.iter().enumerate() {
        let after = &l[o + w.len()..];
        let next = ws.get(k + 1);
        let dotted = o > 0 && b[o - 1] == b'.';
        let starts = o == indent;
        match w {
            "await" if next.is_some() || after.trim_start().starts_with('(') => {
                let n = w.len() + after.len() - after.trim_start().len();
                habit(out, at + o, n, "async is automatic: write `~call()` instead of `await call()`", Some("~"));
            }
            "let" | "var" | "val" | "const" | "final" | "auto"
                if starts && next.is_some_and(|(no, _)| {
                    let rest = l[*no..].trim_start_matches(|c: char| is_ident(c as u8));
                    rest.trim_start().starts_with('=') && !rest.trim_start().starts_with("==")
                        || matches!(w, "let" | "var" | "val" | "auto")
                }) =>
            {
                habit(out, at + o, w.len() + 1, "declare by assigning: `x = 1` (const or not is inferred)", Some(""));
            }
            "new" if next.is_some_and(|(no, nw)| *no == o + 4 && nw.starts_with(|c: char| c.is_ascii_uppercase())) => {
                habit(out, at + o, 4, "NuclearLang has no `new`: call the constructor, `User(...)`", Some(""));
            }
            "def" | "function" | "func" | "fn" | "fun"
                if starts && next.is_some_and(|(no, nw)| *no == o + w.len() + 1 && l[no + nw.len()..].starts_with('(')) =>
            {
                habit(out, at + o, w.len() + 1, "a function is `name(params) ReturnType` + an indented block", Some(""));
            }
            "elif" => habit(out, at + o, 4, "`elif` is `else if`", Some("else if")),
            "except" if starts => {
                let tail = after.trim_end().trim_end_matches(':');
                let var = tail.rsplit_once(" as ").map(|(_, v)| v.trim());
                let fix = var.map_or("catch".to_string(), |v| format!("catch {v}"));
                habit(out, at + o, w.len() + tail.len(), "`except` is `catch` (`catch error`)", Some(&fix));
            }
            "None" | "nil" | "undefined" if !dotted => habit(out, at + o, w.len(), "`null`", Some("null")),
            "True" if !dotted => habit(out, at + o, 4, "`true`", Some("true")),
            "False" if !dotted => habit(out, at + o, 5, "`false`", Some("false")),
            "self" if !dotted => {
                let rest = &l[o + 4..];
                if o > 0 && b[o - 1] == b'(' && (rest.starts_with(", ") || rest.starts_with(')')) {
                    let n = if rest.starts_with(", ") { 6 } else { 4 };
                    habit(out, at + o, n, "methods do not declare `self`", Some(""));
                } else {
                    habit(out, at + o, 4, "the instance is implicit; write `this` only when it is needed", Some("this"));
                }
            }
            "pass" if starts && t == "pass" => {
                let to = (end + 1).min(src.len());
                habit(out, at, to - at, "a block cannot be empty in NuclearLang and has no `pass`: delete the line", Some(""));
            }
            "lambda" => {
                if let Some(c) = after.find(':') {
                    let fix = format!("{} =>", after[..c].trim());
                    habit(out, at + o, w.len() + c + 1, "a lambda is `x => expr`", Some(&fix));
                }
            }
            "switch" if starts => habit(out, at + o, 6, "use `match value` with `pattern -> result` arms", None),
            "for" if starts && after.trim_start().starts_with('(') => {
                habit(out, at + o, 3, "loops are `for x in items` or `for i in 0..n`", None);
            }
            "class" | "struct" | "interface" if starts && next.is_some() => {
                class_header(l, at, o, w, out);
            }
            "import" if starts => import(&src[at..end], at, o, out),
            "from" if starts && l.contains(" import ") => import(&src[at..end], at, o, out),
            _ => {}
        }
    }

    // `x++` / `x--`
    for op in ["++", "--"] {
        if let Some(i) = l.find(op).filter(|&i| i > 0 && is_ident(b[i - 1]) && l[i + 2..].trim().is_empty()) {
            let fix = if op == "++" { " += 1" } else { " -= 1" };
            habit(out, at + i, 2, "no `++`/`--`: `x += 1`", Some(fix));
        }
    }

    // return type first: `int f(x)`, `Widget build(BuildContext context)`, `Future<void> load()`
    if !first.is_empty() && first != "class" && let Some(fix) = type_first(l, indent) {
        let n = l[indent..].trim_end().trim_end_matches(['{', ':']).trim_end().len();
        habit(out, at + indent, n, "the return type goes after the parameters: `name(param: Type) ReturnType`", Some(&fix));
    }
}

/// `class User extends Base implements I` → `User: Base > I`; `class User(Base)` → `User: Base`.
fn class_header(l: &str, at: usize, o: usize, w: &str, out: &mut Vec<Diagnostic>) {
    let rest = l[o + w.len()..].trim().trim_end_matches(['{', ':']).trim_end();
    let (name, bases) = match rest.find('(') {
        Some(p) if rest.ends_with(')') => (rest[..p].trim(), format!(": {}", &rest[p + 1..rest.len() - 1])),
        _ => {
            let mut parts = rest.splitn(2, ' ');
            let name = parts.next().unwrap_or("");
            let tail = parts.next().unwrap_or("").replace("extends ", ": ").replace("implements ", "> ");
            (name, if tail.is_empty() { String::new() } else { format!(" {tail}") })
        }
    };
    let bases = bases.replace(": : ", ": ").replace(" : ", ": ");
    let fix = format!("{name}{}", if bases.starts_with(':') { bases } else { bases.trim_end().to_string() });
    let len = l.trim_end().trim_end_matches(['{', ':']).trim_end().len() - o;
    habit(out, at + o, len, "a module is its name: `User`, `User: Base`, `User > Interface` (or a kind: `data User`)", Some(&fix));
}

/// `from a.b import c, d` → `use a/b{ c, d }`; `import { a } from "x"` → `use x{ a }`; `import x` → `use x`.
fn import(l: &str, at: usize, o: usize, out: &mut Vec<Diagnostic>) {
    let t = l[o..].trim_end().trim_end_matches(';');
    let strip = |s: &str| s.trim().trim_matches(['"', '\'']).trim_start_matches("package:").trim_end_matches(".dart").to_string();
    let fix = if let Some(r) = t.strip_prefix("from ") {
        r.split_once(" import ").map(|(m, names)| format!("use {}{{ {} }}", m.trim().replace('.', "/"), names.trim()))
    } else if let Some((names, m)) = t.strip_prefix("import ").and_then(|r| r.split_once(" from ")) {
        let names = names.trim().trim_start_matches('{').trim_end_matches('}').trim();
        Some(format!("use {}{{ {names} }}", strip(m)))
    } else {
        t.strip_prefix("import ").filter(|r| !r.contains(' ')).map(|r| format!("use {}", strip(r)))
    };
    // the masked line has no string text: the fix is taken from the source columns, which are the same
    let msg = "imports are `use module{ Name }`, and most names are imported by themselves";
    habit(out, at + o, t.len(), msg, fix.as_deref());
}

/// The header rewritten when the return type comes first; None if it is not that form.
fn type_first(l: &str, indent: usize) -> Option<String> {
    let t = l[indent..].trim_end().trim_end_matches(['{', ':']).trim_end();
    let open = t.find('(')?;
    let close = t.rfind(')')?;
    let head = &t[..open];
    let (ty, name) = head.trim_end().rsplit_once(' ')?;
    let ty = ty.trim();
    let generic_ok = ty.chars().all(|c| is_ident(c as u8) || "<>, ?[]".contains(c));
    let pascal = ty.starts_with(|c: char| c.is_ascii_uppercase());
    let base = ty.split(['<', '[', '?']).next().unwrap_or("");
    if !generic_ok || !(pascal || TYPE_WORDS.contains(&base)) || ty.contains(' ') && !ty.contains('<') {
        return None;
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') || !name.chars().all(|c| is_ident(c as u8)) {
        return None;
    }
    if !t[close + 1..].trim().is_empty() {
        return None;
    }
    let params: Vec<String> = t[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| match p.rsplit_once(' ') {
            Some((pt, pn)) if !p.contains(':') && !p.contains('=') => format!("{pn}: {}", pt.trim()),
            _ => p.to_string(),
        })
        .collect();
    Some(format!("{name}({}) {ty}", params.join(", ")))
}

/// The source with the text of strings and comments replaced by spaces (same offsets), plus the habits found
/// while masking: `#` comments, f-strings and backticks.
fn mask(src: &str) -> (String, Vec<Diagnostic>) {
    let b = src.as_bytes();
    let mut m = b.to_vec();
    let mut out = Vec::new();
    let mut i = 0;
    let blank = |m: &mut Vec<u8>, from: usize, to: usize| {
        for c in &mut m[from..to] {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    };
    while i < b.len() {
        let c = b[i];
        if c == b'/' && b.get(i + 1) == Some(&b'/') || c == b'#' {
            let e = b[i..].iter().position(|&x| x == b'\n').map_or(b.len(), |p| i + p);
            if c == b'#' {
                habit(&mut out, i, 1, "comments start with `//`", Some("//"));
            }
            blank(&mut m, i, e);
            i = e;
        } else if c == b'"' || c == b'\'' || c == b'`' {
            let raw = i > 0 && b[i - 1] == b'r' && (i < 2 || !is_ident(b[i - 2]));
            if i > 0 && b[i - 1] == b'f' && (i < 2 || !is_ident(b[i - 2])) {
                habit(&mut out, i - 1, 1, "every string interpolates: \"Hi ${name}\" (no `f` prefix)", Some(""));
            }
            if c == b'`' {
                habit(&mut out, i, 1, "strings use \" or ': \"Hi ${name}\"", Some("\""));
            }
            let triple = b.get(i + 1) == Some(&c) && b.get(i + 2) == Some(&c);
            let q = if triple { 3 } else { 1 };
            let mut j = i + q;
            loop {
                if j >= b.len() || (!triple && b[j] == b'\n') {
                    break;
                }
                if b[j] == b'\\' && !raw {
                    j += 2;
                    continue;
                }
                if b[j] == c && (!triple || (b.get(j + 1) == Some(&c) && b.get(j + 2) == Some(&c))) {
                    break;
                }
                j += 1;
            }
            let j = j.min(b.len());
            blank(&mut m, i + q, j);
            if c == b'`' && j < b.len() && b[j] == b'`' {
                habit(&mut out, j, 1, "strings use \" or ': \"Hi ${name}\"", Some("\""));
            }
            i = (j + q).min(b.len());
        } else {
            i += 1;
        }
    }
    (String::from_utf8(m).unwrap_or_default(), out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(src: &str) -> String {
        let mut d = scan(src);
        d.sort_by_key(|d| std::cmp::Reverse(d.span.start));
        let mut s = src.to_string();
        for d in d {
            let f = d.fix.expect("fix");
            s.replace_range(f.span.start..f.span.end, &f.replacement);
        }
        s
    }

    #[test]
    fn typescript_and_dart_habits_are_fixed() {
        assert_eq!(fixed("f(x: int) int {\n  return x\n}\n"), "f(x: int) int\n  return x\n");
        assert_eq!(fixed("f() int\n  const y = await g()\n  y\n"), "f() int\n  y = ~g()\n  y\n");
        assert_eq!(fixed("f()\n  u = new User(1)\n  i++\n"), "f()\n  u = User(1)\n  i += 1\n");
        assert_eq!(fixed("class Admin extends User {\n  name: string\n}\n"), "Admin: User\n  name: string\n");
        assert_eq!(fixed("Widget build(BuildContext context)\n  x\n"), "build(context: BuildContext) Widget\n  x\n");
        assert_eq!(fixed("int add(int a, int b) {\n  a + b\n}\n"), "add(a: int, b: int) int\n  a + b\n");
        assert_eq!(fixed("import { readFile } from \"fs\"\n"), "use fs{ readFile }\n");
        assert_eq!(fixed("x = `a ${b}`\n"), "x = \"a ${b}\"\n");
    }

    #[test]
    fn python_habits_are_fixed() {
        let py = "def f(self, x: int) int:\n  if x is None:\n    pass\n  elif x:\n    return True\n  # done\n";
        let nuc = "f(x: int) int\n  if x is null\n  else if x\n    return true\n  // done\n";
        assert_eq!(fixed(py), nuc);
        assert_eq!(fixed("from langchain_core.tools import tool\n"), "use langchain_core/tools{ tool }\n");
        assert_eq!(fixed("try\n  a()\nexcept ValueError as e:\n  b(e)\n"), "try\n  a()\ncatch e\n  b(e)\n");
        assert_eq!(fixed("g = lambda x: x + 1\n"), "g = x => x + 1\n");
        assert_eq!(fixed("print(f\"hi {name}\")\n"), "print(\"hi {name}\")\n");
    }

    #[test]
    fn nuclear_code_has_no_habits() {
        let ok = "use flutter/material\n\n// a comment with a # and an await word\nUser\n  name: string\n\n  greet(other: User) string\n    m = { \"a\": 1 }\n    where({\n      \"id\": 1,\n    })\n    s = \"x # { self\"\n    match m\n      Ok(c) -> c\n    \"Hi ${other.name}\"\n\nstatic helper(x: int) int = x\n";
        assert!(scan(ok).is_empty(), "{:?}", scan(ok));
    }
}
