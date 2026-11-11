//! Template engine for dialect files: a small subset of Mustache that agents already know.
//!
//! - `{{name}}`, `{{clause.on}}` insert a value. `{{.}}` is the current list item.
//! - `{{#name}}...{{/name}}` renders the inside when `name` is truthy; for a list, once per item
//!   (the item's fields become visible, plus `@index`, `@first`, `@last`).
//! - `{{^name}}...{{/name}}` renders the inside when `name` is falsy.
//! - Filters: `{{name|pascal}}`, `camel`, `snake`, `kebab`, `lower`, `upper`, `join:", "`.
//! - A run of braces opens the tag at its last two: `{{{#fields}}` is a literal `{` followed by a tag.
//!
//! Layout rules, so generated code is indented correctly:
//! - A line holding only a section tag (`{{#x}}`, `{{^x}}`, `{{/x}}`) disappears from the output.
//! - A line holding only `{{value}}` disappears when the value is empty.
//! - A multi-line value keeps the indentation of the line where it is inserted.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl Value {
    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(s.into())
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::List(l) => !l.is_empty(),
            Value::Map(_) => true,
        }
    }

    pub fn text(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    String::new()
                }
            }
            Value::List(l) => l.iter().map(Value::text).collect::<Vec<_>>().join(", "),
            Value::Map(m) => m.get(".").map(Value::text).unwrap_or_default(),
        }
    }
}

/// Builder for template contexts: `Ctx::new().set("name", "User").flag("async", true)`.
#[derive(Debug, Clone, Default)]
pub struct Ctx(pub BTreeMap<String, Value>);

impl Ctx {
    pub fn new() -> Self {
        Ctx(BTreeMap::new())
    }

    pub fn set(mut self, key: &str, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), Value::Str(value.into()));
        self
    }

    pub fn opt(mut self, key: &str, value: Option<String>) -> Self {
        self.0.insert(key.into(), Value::Str(value.unwrap_or_default()));
        self
    }

    pub fn flag(mut self, key: &str, value: bool) -> Self {
        self.0.insert(key.into(), Value::Bool(value));
        self
    }

    pub fn value(mut self, key: &str, value: Value) -> Self {
        self.0.insert(key.into(), value);
        self
    }

    pub fn into_value(self) -> Value {
        Value::Map(self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Filter {
    /// `'project'` → `project`: a string argument of a decorator written inside another name (`{{x|unquote}}Id`).
    Unquote,
    Pascal,
    Camel,
    Snake,
    Kebab,
    Lower,
    Upper,
    Join(String),
    StripSuffix(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Text(String),
    Var {
        path: String,
        filters: Vec<Filter>,
    },
    /// `{{value}}` alone on its line: removed with its line when empty.
    VarLine {
        indent: String,
        path: String,
        filters: Vec<Filter>,
        newline: bool,
    },
    Section {
        path: String,
        inverted: bool,
        body: Vec<Node>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateError {
    /// Byte offset inside the template text.
    pub offset: usize,
    pub message: String,
}

/// A variable used by a template, with its section depth (0 = not inside any section).
#[derive(Debug, Clone, PartialEq)]
pub struct UsedVar {
    pub name: String,
    pub depth: usize,
}

struct RawTag {
    start: usize,
    end: usize,
    content: String,
}

impl Template {
    pub fn compile(src: &str) -> Result<Template, TemplateError> {
        let tags = scan_tags(src)?;
        // Build a flat list of pieces, removing standalone lines.
        enum Piece {
            Text(String),
            Tag { content: String, offset: usize, line: Option<(String, bool)> },
        }
        let mut pieces = Vec::new();
        let mut cursor = 0;
        let mut i = 0;
        while i < tags.len() {
            let tag = &tags[i];
            let line_start = src[..tag.start].rfind('\n').map_or(0, |p| p + 1);
            let line_end = src[tag.end..].find('\n').map_or(src.len(), |p| tag.end + p);
            let before = &src[line_start..tag.start];
            let after = &src[tag.end..line_end];
            let alone = before.chars().all(|c| c == ' ' || c == '\t')
                && after.chars().all(|c| c == ' ' || c == '\t' || c == '\r')
                && line_start >= cursor
                && tags.get(i + 1).is_none_or(|n| n.start > line_end)
                && (i == 0 || tags[i - 1].end <= line_start);
            let is_section = tag.content.starts_with(['#', '^', '/']);
            if alone && (is_section || !tag.content.starts_with('!')) {
                pieces.push(Piece::Text(src[cursor..line_start].to_string()));
                let has_newline = line_end < src.len();
                let next = if has_newline { line_end + 1 } else { line_end };
                if is_section {
                    pieces.push(Piece::Tag { content: tag.content.clone(), offset: tag.start, line: None });
                } else {
                    pieces.push(Piece::Tag {
                        content: tag.content.clone(),
                        offset: tag.start,
                        line: Some((before.to_string(), has_newline)),
                    });
                }
                cursor = next;
            } else {
                pieces.push(Piece::Text(src[cursor..tag.start].to_string()));
                pieces.push(Piece::Tag { content: tag.content.clone(), offset: tag.start, line: None });
                cursor = tag.end;
            }
            i += 1;
        }
        pieces.push(Piece::Text(src[cursor..].to_string()));

        // Build the tree.
        let mut stack: Vec<(String, bool, Vec<Node>, usize)> = vec![(String::new(), false, Vec::new(), 0)];
        for piece in pieces {
            match piece {
                Piece::Text(t) => {
                    if !t.is_empty() {
                        stack.last_mut().unwrap().2.push(Node::Text(t));
                    }
                }
                Piece::Tag { content, offset, line } => {
                    if let Some(name) = content.strip_prefix('#').or_else(|| content.strip_prefix('^')) {
                        let inverted = content.starts_with('^');
                        stack.push((name.trim().to_string(), inverted, Vec::new(), offset));
                    } else if let Some(name) = content.strip_prefix('/') {
                        let name = name.trim();
                        if stack.len() == 1 {
                            return Err(TemplateError { offset, message: format!("'{{{{/{name}}}}}' closes nothing") });
                        }
                        let (open, inverted, body, _) = stack.pop().unwrap();
                        if open != name {
                            return Err(TemplateError {
                                offset,
                                message: format!("'{{{{/{name}}}}}' closes '{{{{#{open}}}}}'"),
                            });
                        }
                        stack.last_mut().unwrap().2.push(Node::Section { path: open, inverted, body });
                    } else if content.starts_with('!') {
                        // comment
                    } else {
                        let (path, filters) = parse_var(&content, offset)?;
                        let node = match line {
                            Some((indent, newline)) => Node::VarLine { indent, path, filters, newline },
                            None => Node::Var { path, filters },
                        };
                        stack.last_mut().unwrap().2.push(node);
                    }
                }
            }
        }
        if stack.len() > 1 {
            let (open, _, _, offset) = stack.pop().unwrap();
            return Err(TemplateError { offset, message: format!("'{{{{#{open}}}}}' is never closed") });
        }
        Ok(Template { nodes: stack.pop().unwrap().2 })
    }

    pub fn render(&self, ctx: &Value) -> String {
        let mut out = String::new();
        let mut stack = vec![ctx];
        render_nodes(&self.nodes, &mut stack, &mut out);
        out
    }

    /// Variables and sections the template reads, for validation.
    pub fn used_vars(&self) -> Vec<UsedVar> {
        fn walk(nodes: &[Node], depth: usize, out: &mut Vec<UsedVar>) {
            for n in nodes {
                match n {
                    Node::Text(_) => {}
                    Node::Var { path, .. } | Node::VarLine { path, .. } => {
                        out.push(UsedVar { name: root(path), depth });
                    }
                    Node::Section { path, body, .. } => {
                        out.push(UsedVar { name: root(path), depth });
                        walk(body, depth + 1, out);
                    }
                }
            }
        }
        fn root(path: &str) -> String {
            path.split('.').next().unwrap_or(path).to_string()
        }
        let mut out = Vec::new();
        walk(&self.nodes, 0, &mut out);
        out
    }

    /// Plain text with no tags (used for templates that are constant strings).
    pub fn is_constant(&self) -> bool {
        self.nodes.iter().all(|n| matches!(n, Node::Text(_)))
    }
}

fn scan_tags(src: &str) -> Result<Vec<RawTag>, TemplateError> {
    let b = src.as_bytes();
    let mut tags = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'{' && b[i + 1] == b'{' {
            // a run of braces opens the tag at its last two
            let mut j = i;
            while j < b.len() && b[j] == b'{' {
                j += 1;
            }
            let start = j - 2;
            let Some(close) = src[j..].find("}}") else {
                return Err(TemplateError { offset: start, message: "'{{' is never closed with '}}'".into() });
            };
            let end = j + close + 2;
            tags.push(RawTag { start, end, content: src[j..j + close].trim().to_string() });
            i = end;
        } else {
            i += 1;
        }
    }
    Ok(tags)
}

fn parse_var(content: &str, offset: usize) -> Result<(String, Vec<Filter>), TemplateError> {
    let mut parts = split_filters(content);
    let path = parts.remove(0).trim().to_string();
    if path.is_empty() {
        return Err(TemplateError { offset, message: "empty tag '{{}}'".into() });
    }
    let mut filters = Vec::new();
    for f in parts {
        let f = f.trim();
        let filter = match f {
            "pascal" => Filter::Pascal,
            "camel" => Filter::Camel,
            "snake" => Filter::Snake,
            "kebab" => Filter::Kebab,
            "lower" => Filter::Lower,
            "upper" => Filter::Upper,
            "unquote" => Filter::Unquote,
            _ => match (f.strip_prefix("join:"), f.strip_prefix("strip_suffix:")) {
                (Some(arg), _) => Filter::Join(unquote(arg.trim())),
                (_, Some(arg)) => Filter::StripSuffix(unquote(arg.trim())),
                _ => {
                    return Err(TemplateError {
                        offset,
                        message: format!(
                            "unknown filter '{f}' (filters: unquote, pascal, camel, snake, kebab, lower, upper, join:\", \", strip_suffix:\"X\")"
                        ),
                    });
                }
            },
        };
        filters.push(filter);
    }
    Ok((path, filters))
}

/// Splits `a|join:"|"|upper` on `|` outside quotes.
fn split_filters(s: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut quote = None;
    for c in s.chars() {
        match (c, quote) {
            ('"' | '\'', None) => {
                quote = Some(c);
                out.last_mut().unwrap().push(c);
            }
            (c2, Some(q)) if c2 == q => {
                quote = None;
                out.last_mut().unwrap().push(c);
            }
            ('|', None) => out.push(String::new()),
            _ => out.last_mut().unwrap().push(c),
        }
    }
    out
}

fn unquote(s: &str) -> String {
    let inner = if s.len() >= 2 && (s.starts_with('"') && s.ends_with('"') || s.starts_with('\'') && s.ends_with('\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    };
    inner.replace("\\n", "\n").replace("\\t", "\t")
}

fn lookup<'a>(stack: &[&'a Value], path: &str) -> Option<&'a Value> {
    // `a or b`: the first alternative that is true (`{{#deco_args.local or owner_decor.repository.saveLocal}}`)
    if path.contains(" or ") {
        return path.split(" or ").map(str::trim).find_map(|p| lookup(stack, p).filter(|v| v.truthy()));
    }
    if path == "." {
        let top = stack.last()?;
        return match top {
            Value::Map(m) => m.get(".").or(Some(top)),
            _ => Some(top),
        };
    }
    let mut segments = path.split('.');
    let first = segments.next()?;
    let mut cur = stack.iter().rev().find_map(|v| match v {
        Value::Map(m) => m.get(first),
        _ => None,
    })?;
    for seg in segments {
        cur = match cur {
            Value::Map(m) => m.get(seg)?,
            _ => return None,
        };
    }
    Some(cur)
}

fn apply_filters(v: Option<&Value>, filters: &[Filter]) -> String {
    let mut text = match (v, filters.iter().find_map(|f| if let Filter::Join(s) = f { Some(s) } else { None })) {
        (Some(Value::List(items)), Some(sep)) => items.iter().map(Value::text).collect::<Vec<_>>().join(sep),
        (Some(v), _) => v.text(),
        (None, _) => String::new(),
    };
    for f in filters {
        text = match f {
            Filter::Pascal => words(&text).iter().map(|w| capitalize(w)).collect(),
            Filter::Camel => {
                let w = words(&text);
                w.iter().enumerate().map(|(i, w)| if i == 0 { w.to_lowercase() } else { capitalize(w) }).collect()
            }
            Filter::Snake => words(&text).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("_"),
            Filter::Kebab => words(&text).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("-"),
            Filter::Unquote => unquote(&text),
            Filter::Lower => text.to_lowercase(),
            Filter::Upper => text.to_uppercase(),
            Filter::Join(_) => text,
            Filter::StripSuffix(x) => text.strip_suffix(x.as_str()).map_or(text.clone(), str::to_string),
        };
    }
    text
}

/// `UsersController` → [Users, Controller]; `user_name` → [user, name].
fn words(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let boundary = c.is_uppercase()
            && !cur.is_empty()
            && (chars[i - 1].is_lowercase() || chars.get(i + 1).is_some_and(|n| n.is_lowercase()));
        if boundary {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
        None => String::new(),
    }
}

/// Leading whitespace of the line being written.
fn current_indent(out: &str) -> String {
    let line = &out[out.rfind('\n').map_or(0, |p| p + 1)..];
    line.chars().take_while(|c| *c == ' ' || *c == '\t').collect()
}

/// Appends a value; its continuation lines get `indent` in front.
fn push_indented(out: &mut String, value: &str, indent: &str) {
    let lines: Vec<&str> = value.split('\n').collect();
    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
            // the last segment is followed by template text, so it gets the indentation too
            if !line.is_empty() || i + 1 == n {
                out.push_str(indent);
            }
        }
        out.push_str(line);
    }
}

fn render_nodes(nodes: &[Node], stack: &mut Vec<&Value>, out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) => out.push_str(t),
            Node::Var { path, filters } => {
                let text = apply_filters(lookup(stack, path), filters);
                let indent = current_indent(out);
                push_indented(out, &text, &indent);
            }
            Node::VarLine { indent, path, filters, newline } => {
                let text = apply_filters(lookup(stack, path), filters);
                if text.is_empty() {
                    continue;
                }
                let full_indent = current_indent(out) + indent;
                out.push_str(indent);
                push_indented(out, &text, &full_indent);
                if *newline {
                    out.push('\n');
                }
            }
            Node::Section { path, inverted, body } => {
                let v = lookup(stack, path);
                let truthy = v.is_some_and(Value::truthy);
                if *inverted {
                    if !truthy {
                        render_nodes(body, stack, out);
                    }
                    continue;
                }
                let Some(v) = v.filter(|v| v.truthy()) else {
                    continue;
                };
                match v {
                    Value::List(items) => {
                        let n = items.len();
                        for (i, item) in items.iter().enumerate() {
                            let mut frame = match item {
                                Value::Map(m) => m.clone(),
                                other => BTreeMap::from([(".".to_string(), other.clone())]),
                            };
                            frame.insert("@index".into(), Value::Str(i.to_string()));
                            frame.insert("@first".into(), Value::Bool(i == 0));
                            frame.insert("@last".into(), Value::Bool(i + 1 == n));
                            let frame = Value::Map(frame);
                            // the frame lives only during this iteration
                            let mut local: Vec<&Value> = stack.to_vec();
                            local.push(&frame);
                            let mut sub = String::new();
                            sub.push_str(&out[out.rfind('\n').map_or(0, |p| p + 1)..]);
                            let prefix_len = sub.len();
                            render_nodes(body, &mut local, &mut sub);
                            out.push_str(&sub[prefix_len..]);
                        }
                    }
                    Value::Map(_) => {
                        stack.push(v);
                        render_nodes(body, stack, out);
                        stack.pop();
                    }
                    _ => render_nodes(body, stack, out),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(t: &str, ctx: Value) -> String {
        Template::compile(t).unwrap().render(&ctx)
    }

    #[test]
    fn variables_and_sections() {
        let ctx = Ctx::new().set("name", "User").flag("async", true).set("ret", "").into_value();
        assert_eq!(render("{{#async}}async {{/async}}{{name}}{{#ret}}: {{ret}}{{/ret}}", ctx), "async User");
    }

    #[test]
    fn inverted_section() {
        let ctx = Ctx::new().set("type", "").into_value();
        assert_eq!(render("{{#type}}{{type}}{{/type}}{{^type}}var{{/type}} x", ctx), "var x");
    }

    #[test]
    fn multiline_values_keep_indentation() {
        let ctx = Ctx::new().set("body", "a();\nb();").into_value();
        assert_eq!(render("f() {\n  {{body}}\n}", ctx), "f() {\n  a();\n  b();\n}");
    }

    #[test]
    fn empty_value_line_disappears() {
        let ctx = Ctx::new().set("body", "").into_value();
        assert_eq!(render("f() {\n  {{body}}\n}", ctx), "f() {\n}");
    }

    #[test]
    fn standalone_section_lines_disappear() {
        let ctx = Ctx::new().flag("x", true).into_value();
        assert_eq!(render("a\n{{#x}}\nb\n{{/x}}\nc", ctx), "a\nb\nc");
    }

    #[test]
    fn lists_with_first_last() {
        let fields =
            Value::List(vec![Ctx::new().set("name", "a").into_value(), Ctx::new().set("name", "b").into_value()]);
        let ctx = Ctx::new().value("fields", fields).set("class_name", "P").into_value();
        assert_eq!(
            render("{{class_name}}({{{#fields}}this.{{name}}{{^@last}}, {{/@last}}{{/fields}}})", ctx),
            "P({this.a, this.b})"
        );
    }

    #[test]
    fn filters() {
        let ctx = Ctx::new().set("name", "UsersController").into_value();
        assert_eq!(render("{{name|strip_suffix:\"Controller\"}}", ctx.clone()), "Users");
        assert_eq!(
            render("{{name|kebab}} {{name|snake}} {{name|camel}}", ctx),
            "users-controller users_controller usersController"
        );
        let list = Ctx::new().value("names", Value::List(vec![Value::str("a"), Value::str("b")])).into_value();
        assert_eq!(render("{{names|join:\" | \"}}", list), "a | b");
        let arg = Ctx::new().set("inverse", "'project'").into_value();
        assert_eq!(render("{{inverse|unquote}}Id", arg), "projectId");
    }

    #[test]
    fn dotted_paths() {
        let clause = Ctx::new().set("on", "String").into_value();
        let ctx = Ctx::new().value("clause", clause).into_value();
        assert_eq!(render("on {{clause.on}}", ctx), "on String");
    }

    #[test]
    fn errors() {
        assert!(Template::compile("{{#a}}x").is_err());
        assert!(Template::compile("{{#a}}x{{/b}}").is_err());
        assert!(Template::compile("{{a|nope}}").is_err());
        assert!(Template::compile("{{a").is_err());
    }
}
