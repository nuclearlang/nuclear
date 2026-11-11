//! Editor features computed from a document's index.

use crate::dialects;
use crate::index::{BindKind, HeaderRole, Index, PRIMITIVE_TYPES, Target, Tok};
use crate::server::Doc;
use crate::typedefs::{self, Names, StaticItem};
use nuclear_diag::Span;
use nuclear_dialect::{Dialect, model::KindSource};
use nuclear_dialect::defs::{self, Decl, Member, Package, Param, TypeDefs};
use nuclear_syntax::DocComment;
use nuclear_syntax::ast::is_pascal;
use nuclear_syntax::token::{Keyword, TokenKind};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::Arc;

fn offset(doc: &Doc, p: &Value) -> usize {
    doc.lines.offset_of(&doc.text, &p["position"])
}

fn location(doc: &Doc, span: Span) -> Value {
    json!({ "uri": doc.uri, "range": doc.lines.range(&doc.text, span) })
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Start of the identifier that ends at `offset`.
fn word_start(text: &str, offset: usize) -> usize {
    text[..offset].char_indices().rev().take_while(|&(_, c)| is_word_char(c)).last().map_or(offset, |(i, _)| i)
}

// ---------------------------------------------------------------- external types (type definitions)

/// The type definitions of the dialect and their name index, when it has any.
fn type_defs(dialect: Option<&Dialect>) -> Option<(&TypeDefs, Arc<Names>)> {
    let d = dialect?;
    (!d.typedefs.packages.is_empty()).then(|| (d.typedefs.as_ref(), typedefs::names(&d.typedefs)))
}

/// What the word before a `.` is.
enum Object<'a> {
    /// A top-level declaration of another file of the project: that file's source, index and binding.
    Project(&'a str, &'a Index, usize),
    /// A module, kind or enum of this file.
    File(usize),
    /// An external class used by name (`Colors.`, `Text.`).
    Static(String),
    /// A value whose type is known (`dio: Dio`, `dio = Dio()`, `state` inherited from `Cubit<int>`).
    Instance(String),
}

/// Bases of the module or kind that contains `at`: the written ones and the classes its kinds inherit.
fn module_bases(doc: &Doc, dialect: Option<&Dialect>, at: usize) -> Vec<String> {
    let Some(m) = doc.index.enclosing_module(at) else { return Vec::new() };
    let bind = &doc.index.bindings[m];
    let mut out = bind.bases.clone();
    if let Some(d) = dialect {
        for w in &bind.words {
            if let Some(k) = d.kinds.get(w) {
                out.extend(k.value.inherits.iter().cloned());
            }
        }
    }
    out
}

/// Resolves the identifier at `start..end` (the object of `object.member`). Nothing when its type is unknown.
fn object_at<'a>(doc: &'a Doc, dialect: Option<&Dialect>, start: usize, end: usize) -> Option<Object<'a>> {
    let ix = &doc.index;
    let text = &doc.text;
    let name = text.get(start..end).filter(|n| !n.is_empty())?;
    let file_type = |ty: &str| {
        let base = typedefs::split(ty).0;
        ix.visible(start).into_iter().find(|&b| ix.bindings[b].name == base && ix.bindings[b].kind.is_type())
    };
    let project_type = |ty: &str| {
        let (p, path) = doc.project.as_ref()?;
        let (src, other, b) = p.decl(path, &typedefs::split(ty).0)?;
        other.bindings[b].kind.is_type().then_some(Object::Project(src, other, b))
    };
    let binding = ix
        .binding_at(end.saturating_sub(1).max(start))
        .filter(|&b| ix.bindings[b].name == name)
        .or_else(|| ix.visible(start).into_iter().find(|&b| ix.bindings[b].name == name));
    if let Some(b) = binding {
        let bind = &ix.bindings[b];
        if bind.kind.is_type() {
            return Some(Object::File(b));
        }
        if !matches!(bind.kind, BindKind::Local | BindKind::Param | BindKind::Field) {
            return None;
        }
        let ty = ix.type_text(text, b).map(str::to_string).or_else(|| bind.inferred.clone())?;
        return Some(file_type(&ty).map(Object::File).or_else(|| project_type(&ty)).unwrap_or(Object::Instance(ty)));
    }
    if let Some(o) = project_type(name) {
        return Some(o);
    }
    let (td, names) = type_defs(dialect)?;
    if is_pascal(name) {
        return td.get(name).map(|_| Object::Static(name.to_string()));
    }
    // a member inherited from an external base: `state` in `CounterCubit: Cubit<int>`
    module_bases(doc, dialect, start)
        .iter()
        .find_map(|base| typedefs::member_type(td, &names, base, name))
        .map(Object::Instance)
}

/// `object.` right before `start` (not `..`): the object's span.
fn object_before(text: &str, start: usize) -> Option<(usize, usize)> {
    let before = &text[..start];
    if !before.ends_with('.') || before.ends_with("..") {
        return None;
    }
    let end = start - 1;
    Some((word_start(text, end), end))
}

const TAG_DEPRECATED: u32 = 1;

fn tags(deprecated: bool) -> Value {
    if deprecated { json!([TAG_DEPRECATED]) } else { json!([]) }
}

fn docs(doc: Option<&String>) -> Value {
    doc.map_or(Value::Null, |d| json!({ "kind": "markdown", "value": d }))
}

fn member_item(m: &Member, sort: &str) -> Value {
    json!({
        "label": m.name,
        "kind": if m.kind == "method" { KIND_METHOD } else { KIND_PROPERTY },
        "detail": defs::render_member(m, ""),
        "documentation": docs(m.doc.as_ref()),
        "tags": tags(m.deprecated),
        "sortText": format!("{sort}{}", m.name),
    })
}

fn decl_kind(d: &Decl) -> u32 {
    match d.kind.as_str() {
        "enum" => KIND_ENUM,
        "function" => KIND_FUNCTION,
        "variable" => KIND_VARIABLE,
        _ => KIND_CLASS,
    }
}

/// A free name of the type definitions (`Scaffold`, `runApp`).
fn decl_item(p: &Package, d: &Decl) -> Value {
    let lib = d.library.as_deref().unwrap_or_default();
    json!({
        "label": d.name,
        "kind": decl_kind(d),
        "detail": format!("{} · {}", lib, p.id()),
        "labelDetails": { "description": lib },
        "documentation": docs(d.doc.as_ref()),
        "tags": tags(d.deprecated),
        "sortText": format!("7{}", d.name),
    })
}

/// Escapes a text for a snippet (`insertTextFormat` 2).
fn snippet_text(s: &str) -> String {
    s.replace('\\', "\\\\").replace('$', "\\$").replace('}', "\\}")
}

/// Something external that a call names: its label, parameters and doc.
struct Callable<'t> {
    /// `Text`, `Text.rich`, `Dio.get`.
    label: String,
    params: &'t [Param],
    doc: Option<&'t String>,
}

/// The external constructor, function or method called by the `(` at `paren`: `Text(`, `Text.rich(`,
/// `Scaffold<T>(`, `runApp(`, `dio.get<string>(`.
fn external_callable<'t>(doc: &Doc, dialect: Option<&'t Dialect>, paren: usize) -> Option<Callable<'t>> {
    let (td, names) = type_defs(dialect)?;
    let text = &doc.text;
    let end = generic_start(text, paren);
    let start = word_start(text, end);
    let name = text.get(start..end).filter(|n| !n.is_empty())?;
    match object_before(text, start) {
        None => {
            if doc.index.visible(start).into_iter().any(|b| doc.index.bindings[b].name == name) {
                return None; // declared in this file
            }
            let (_, d) = td.get(name)?;
            if d.kind == "function" {
                return Some(Callable { label: d.name.clone(), params: &d.params, doc: d.doc.as_ref() });
            }
            let c = d.constructors.iter().find(|c| c.name.is_empty())?;
            Some(Callable { label: d.name.clone(), params: &c.params, doc: c.doc.as_ref().or(d.doc.as_ref()) })
        }
        Some((os, oe)) => match object_at(doc, dialect, os, oe)? {
            Object::File(_) | Object::Project(..) => None,
            Object::Static(class) => {
                let (_, d) = td.get(&class)?;
                let label = format!("{class}.{name}");
                if let Some(c) = d.constructors.iter().find(|c| c.name == name) {
                    return Some(Callable { label, params: &c.params, doc: c.doc.as_ref() });
                }
                let m = d.members.iter().find(|m| m.is_static && m.name == name)?;
                Some(Callable { label, params: &m.params, doc: m.doc.as_ref() })
            }
            Object::Instance(ty) => {
                let found = typedefs::instance_members(td, &names, &ty).into_iter().find(|f| f.member.name == name)?;
                let m = found.member;
                Some(Callable { label: format!("{}.{name}", found.owner.name), params: &m.params, doc: m.doc.as_ref() })
            }
        },
    }
}

/// `Name<A, B>` before `paren` → the end of `Name`.
fn generic_start(text: &str, paren: usize) -> usize {
    if !text[..paren].ends_with('>') {
        return paren;
    }
    let mut depth = 0;
    for (i, c) in text[..paren].char_indices().rev() {
        match c {
            '>' => depth += 1,
            '<' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    paren
}

/// Hover of an external declaration: its API in Nuclear notation, its doc and its package.
fn decl_markdown(p: &Package, d: &Decl) -> String {
    // the first comment lines of `render` (library, package, doc) go below the code as text
    let rendered = defs::render(p, d);
    let mut lines: Vec<&str> = rendered.lines().skip_while(|l| l.starts_with("//")).collect();
    // shortcut: long classes (Colors, Icons) are cut at 40 lines; `nuclear defs show` prints them whole
    let total = lines.len();
    if total > 40 {
        lines.truncate(40);
    }
    let mut md = format!("```nuclear\n{}", lines.join("\n"));
    if total > 40 {
        md.push_str(&format!("\n  // ... {} more lines", total - 40));
    }
    md.push_str("\n```\n");
    md.push_str(&footer(p, d.library.as_deref(), d.deprecated, d.doc.as_ref()));
    md
}

fn footer(p: &Package, library: Option<&str>, deprecated: bool, doc: Option<&String>) -> String {
    let mut md = String::new();
    if deprecated {
        md.push_str("**Deprecated.** ");
    }
    if let Some(doc) = doc {
        md.push_str(doc);
        md.push_str("\n\n");
    }
    md.push_str(&format!("`{}`", p.id()));
    if let Some(lib) = library {
        md.push_str(&format!(" · `{lib}`"));
    }
    md
}

/// Hover of `object.name` when the object is external or has a known external type.
fn member_markdown(doc: &Doc, dialect: Option<&Dialect>, start: usize, name: &str) -> Option<String> {
    let (td, names) = type_defs(dialect)?;
    let (os, oe) = object_before(&doc.text, start)?;
    let (owner, code, deprecated, member_doc) = match object_at(doc, dialect, os, oe)? {
        Object::File(_) | Object::Project(..) => return None,
        Object::Static(class) => {
            let (_, d) = td.get(&class)?;
            if d.values.iter().any(|v| v == name) {
                (d, format!("{class}.{name}  // value of enum {class}"), false, None)
            } else if let Some(c) = d.constructors.iter().find(|c| c.name == name) {
                (d, defs::render_constructor(&class, c, ""), c.deprecated, c.doc.as_ref())
            } else {
                let m = d.members.iter().find(|m| m.is_static && m.name == name)?;
                (d, defs::render_member(m, ""), m.deprecated, m.doc.as_ref())
            }
        }
        Object::Instance(ty) => {
            let f = typedefs::instance_members(td, &names, &ty).into_iter().find(|f| f.member.name == name)?;
            (f.owner, defs::render_member(f.member, ""), f.member.deprecated, f.member.doc.as_ref())
        }
    };
    let (p, _) = td.get(&owner.name)?;
    let mut md = format!("```nuclear\n// {}\n{code}\n```\n", owner.name);
    md.push_str(&footer(p, owner.library.as_deref(), deprecated, member_doc));
    Some(md)
}

// ---------------------------------------------------------------- decorator arguments

/// Inside the arguments of `@name(...)`: the decorator and the path of the object being written
/// (`@screen(loggedIn = { for` → (`screen`, `loggedIn.`)). Brackets inside strings are not skipped.
fn decorator_context(text: &str, at: usize) -> Option<(&str, String)> {
    let mut depth = 0;
    let mut keys = Vec::new();
    for (i, c) in text[..at].char_indices().rev() {
        match c {
            ')' | ']' | '}' => depth += 1,
            '{' if depth == 0 => {
                let key = text[..i].trim_end().strip_suffix(['=', ':'])?.trim_end();
                let key = &key[word_start(key, key.len())..];
                if key.is_empty() {
                    return None;
                }
                keys.push(key);
            }
            '(' if depth == 0 => {
                let name_start = word_start(text, i);
                let name = &text[name_start..i];
                let path = keys.iter().rev().map(|k| format!("{k}.")).collect();
                return (!name.is_empty() && text[..name_start].ends_with('@')).then_some((name, path));
            }
            '[' if depth == 0 => return None,
            '(' | '[' | '{' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Hover of an argument name of a decorator: `route` in `@screen(route = "/")`, `forward` in
/// `loggedIn = { forward: LoginScreen }`.
fn decorator_arg_hover(text: &str, dialect: Option<&Dialect>, at: usize) -> Option<(Span, String)> {
    let d = dialect?;
    let start = word_start(text, at);
    let end = at + text[at..].find(|c: char| !is_word_char(c)).unwrap_or(text.len() - at);
    let rest = text[end..].trim_start();
    if start == end || !(rest.starts_with(':') || rest.starts_with('=') && !rest.starts_with("==")) {
        return None;
    }
    let (decorator, path) = decorator_context(text, start)?;
    let md = dialects::decorator_arg_doc(decorator, &format!("{path}{}", &text[start..end]), d)?;
    Some((Span::new(start, end), md))
}

// ---------------------------------------------------------------- semantic tokens

pub fn semantic_tokens(doc: &Doc) -> Value {
    let mut data = Vec::with_capacity(doc.index.occs.len() * 5);
    let (mut prev_line, mut prev_col) = (0u32, 0u32);
    for o in &doc.index.occs {
        let (line, col) = doc.lines.position(&doc.text, o.span.start);
        let (end_line, end_col) = doc.lines.position(&doc.text, o.span.end);
        if end_line != line {
            continue;
        }
        let delta_col = if line == prev_line { col - prev_col } else { col };
        data.extend([line - prev_line, delta_col, end_col - col, o.tok as u32, o.mods]);
        prev_line = line;
        prev_col = col;
    }
    json!({ "data": data })
}

// ---------------------------------------------------------------- hover

pub fn hover(doc: &Doc, dialect: Option<&Dialect>, p: &Value) -> Value {
    let at = offset(doc, p);
    let text = &doc.text;
    let ix = &doc.index;
    let (span, markdown) = if let Some((span, md)) = decorator_arg_hover(text, dialect, at) {
        (span, md)
    } else if let Some(o) = ix.occurrence_at(at) {
        let word = &text[o.span.start..o.span.end];
        let md = match &o.target {
            Target::Binding(b) => binding_markdown(doc, *b),
            Target::Library(module) => {
                let mut md = match type_defs(dialect).and_then(|(td, _)| td.get(word)) {
                    Some((p, d)) => format!("{}\n\nImported automatically from `{module}`.", decl_markdown(p, d)),
                    None => format!("```nuclear\n{word}\n```\nImported automatically from `{module}`."),
                };
                if let Some(url) = dialects::api_doc_url(module, word) {
                    md.push_str(&format!("\n\n[API reference]({url})"));
                }
                md
            }
            Target::KindMember(kind) => {
                let dialect_name = dialect.map_or("", |d| d.name.as_str());
                format!("```nuclear\n{word}\n```\nProvided by the `{kind}` kind of the `{dialect_name}` dialect.")
            }
            Target::DialectName => {
                format!("```nuclear\n{word}\n```\nDefined by the active dialect (`[functions]` / `[names]`).")
            }
            Target::HeaderWord(role) => header_word_markdown(word, *role, dialect),
            Target::None => {
                if let Some(n) = doc.project_name(word).filter(|_| object_before(text, o.span.start).is_none()) {
                    let mut md = format!("```nuclear\n{}\n```\nDeclared in `{}.nuc`.", n.signature, n.path);
                    let comment = doc
                        .project
                        .as_ref()
                        .and_then(|(p, from)| p.decl(from, word))
                        .and_then(|(src, ix, b)| ix.doc_comment(src, b));
                    if let Some(c) = comment {
                        md.push_str("\n\n---\n");
                        md.push_str(&DocComment::parse(&c).markdown());
                    }
                    md
                } else if o.tok == Tok::Decorator {
                    match dialect.and_then(|d| dialects::decorator_doc(word.trim_start_matches('@'), d)) {
                        Some(md) => md,
                        None => return Value::Null,
                    }
                } else if let Some(md) = dialects::builtin_doc(word).filter(|_| doc.index.binding_at(at).is_none()) {
                    format!("```nuclear\n{word}\n```\n{md}")
                } else if let Some(md) = member_markdown(doc, dialect, o.span.start, word) {
                    md
                } else if o.tok == Tok::Type || PRIMITIVE_TYPES.contains(&word) {
                    type_markdown(word, dialect)
                } else if let Some((p, d)) = type_defs(dialect).and_then(|(td, _)| td.get(word)) {
                    decl_markdown(p, d) // a name of an implicit library (`print`, `Duration`)
                } else {
                    return Value::Null;
                }
            }
        };
        (o.span, md)
    } else {
        // reserved words are not in the index: look at the token under the cursor
        let lexed = nuclear_syntax::lex(text);
        let Some(tok) = lexed
            .tokens
            .iter()
            .find(|t| t.span.start <= at && at <= t.span.end && matches!(t.kind, TokenKind::Keyword(_)))
        else {
            return Value::Null;
        };
        let TokenKind::Keyword(k) = tok.kind else { return Value::Null };
        let Some(doc_text) = dialects::keyword_doc(k.as_str()) else { return Value::Null };
        (tok.span, format!("**`{}`** — reserved word of the base language\n\n{doc_text}", k.as_str()))
    };
    json!({
        "contents": { "kind": "markdown", "value": markdown },
        "range": doc.lines.range(text, span),
    })
}

fn binding_markdown(doc: &Doc, b: usize) -> String {
    let ix = &doc.index;
    let bind = &ix.bindings[b];
    let mut md = format!("```nuclear\n{}\n```\n", ix.signature(&doc.text, b));
    let mut label = bind.kind.label().to_string();
    if bind.kind == BindKind::Kind
        && let Some(word) = bind.words.first()
    {
        label = format!("`{word}` declaration");
    }
    md.push_str(&format!("*{label}*"));
    if let Some(parent) = bind.parent {
        md.push_str(&format!(" in `{}`", ix.bindings[parent].name));
    }
    if matches!(bind.kind, BindKind::Local | BindKind::Param | BindKind::Field)
        && bind.ty.is_none()
        && !ix.signature(&doc.text, b).contains(':')
    {
        md.push_str(" · type inferred");
    }
    if let Some(comment) = ix.doc_comment(&doc.text, b) {
        md.push_str("\n\n---\n");
        md.push_str(&DocComment::parse(&comment).markdown());
    }
    md
}

/// A base type: its documentation and how the active dialect writes it.
fn type_markdown(word: &str, dialect: Option<&Dialect>) -> String {
    let text =
        nuclear_dialect::base_doc("types", word).unwrap_or("Base type. Each dialect writes it in the target language.");
    let mut md = format!("```nuclear\n{word}\n```\n{text}");
    if let Some(d) = dialect.filter(|d| d.types.contains_key(word)) {
        let target = d.types[word].value.render(&nuclear_dialect::template::Ctx::new().into_value());
        md.push_str(&format!("\n\nIn the `{}` dialect: `{target}`", d.name));
    }
    md
}

/// Where a kind comes from (ADR-0095): the language dialect, the platform's contract or a package.
pub fn kind_source(d: &Dialect, name: &str) -> Option<String> {
    Some(match d.kind_sources.get(name)? {
        KindSource::Language => format!("`{}` language", d.chain.last()?),
        KindSource::Platform(p) => format!("native of the `{p}` platform"),
        KindSource::Package { name, version, dialects } => {
            let version = if version.is_empty() { String::new() } else { format!(" {version}") };
            format!("package `{name}{version}` for {}", dialects.join(", "))
        }
    })
}

fn header_word_markdown(word: &str, role: HeaderRole, dialect: Option<&Dialect>) -> String {
    let dialect_name = dialect.map_or("base".to_string(), |d| d.name.clone());
    let (what, section) = match role {
        HeaderRole::Kind => ("kind", "kinds"),
        HeaderRole::Modifier => ("modifier", "words"),
        HeaderRole::Clause => ("clause", "words"),
    };
    let mut md = format!("**`{word}`** — {what} · `{dialect_name}` dialect");
    if let Some(source) = dialect.filter(|_| role == HeaderRole::Kind).and_then(|d| kind_source(d, word)) {
        md.push_str(&format!(" · {source}"));
    }
    let kind = dialect.and_then(|d| d.kinds.get(word)).filter(|_| role == HeaderRole::Kind).map(|k| &k.value);
    match dialect.and_then(|d| d.doc(section, word)) {
        Some(doc) => md.push_str(&format!("\n\n{doc}")),
        None => {
            if let Some(k) = kind.filter(|k| !k.description.is_empty()) {
                md.push_str(&format!("\n\n{}", k.description));
            }
        }
    }
    if let Some(k) = kind {
        let mut facts = Vec::new();
        if !k.requires.is_empty() {
            facts.push(format!("Required methods: `{}`", k.requires.join("`, `")));
        }
        if !k.members.is_empty() {
            facts.push(format!("Available without declaring them: `{}`", k.members.join("`, `")));
        }
        if !facts.is_empty() {
            md.push_str(&format!("\n\n---\n{}", facts.join("\n\n")));
        }
    }
    md
}

/// Markdown of a doc comment of the editor's documents, for completion items.
fn comment_docs(text: &str, ix: &Index, b: usize) -> (Value, bool) {
    match ix.doc_comment(text, b).map(|c| DocComment::parse(&c)) {
        Some(d) => (json!({ "kind": "markdown", "value": d.markdown() }), d.deprecated.is_some()),
        None => (Value::Null, false),
    }
}

// ---------------------------------------------------------------- completion

const KIND_METHOD: u32 = 2;
const KIND_FUNCTION: u32 = 3;
const KIND_CONSTRUCTOR: u32 = 4;
const KIND_FIELD: u32 = 5;
const KIND_VARIABLE: u32 = 6;
const KIND_CLASS: u32 = 7;
const KIND_PROPERTY: u32 = 10;
const KIND_ENUM: u32 = 13;
const KIND_KEYWORD: u32 = 14;
const KIND_SNIPPET: u32 = 15;
const KIND_ENUM_MEMBER: u32 = 20;
const KIND_TYPE_PARAMETER: u32 = 25;
/// Library names offered at once; the list is incomplete beyond that, so the editor asks again as the user types.
const MAX_LIBRARY_ITEMS: usize = 300;

fn completion_kind(kind: BindKind) -> u32 {
    match kind {
        BindKind::Module | BindKind::Kind => KIND_CLASS,
        BindKind::Enum => KIND_ENUM,
        BindKind::Function => KIND_FUNCTION,
        BindKind::Method => KIND_METHOD,
        BindKind::Getter => KIND_PROPERTY,
        BindKind::Field => KIND_FIELD,
        BindKind::Param | BindKind::Local => KIND_VARIABLE,
        BindKind::EnumItem => KIND_ENUM_MEMBER,
        BindKind::TypeParam => KIND_TYPE_PARAMETER,
    }
}

fn binding_item(doc: &Doc, b: usize, sort: &str) -> Value {
    binding_item_in(&doc.text, &doc.index, b, sort)
}

fn binding_item_in(text: &str, ix: &Index, b: usize, sort: &str) -> Value {
    let bind = &ix.bindings[b];
    let (documentation, deprecated) = comment_docs(text, ix, b);
    json!({
        "label": bind.name,
        "kind": completion_kind(bind.kind),
        "detail": ix.signature(text, b),
        "documentation": documentation,
        "tags": tags(deprecated),
        "sortText": format!("{sort}{}", bind.name),
    })
}

/// Code templates offered at the start of a line.
fn statement_snippets() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("if", "if ${1:condition}\n  $0", "if + block"),
        ("if else", "if ${1:condition}\n  $2\nelse\n  $0", "if / else"),
        ("for", "for ${1:item} in ${2:items}\n  $0", "for ... in"),
        ("while", "while ${1:condition}\n  $0", "while"),
        ("match", "match ${1:value}\n  ${2:pattern} -> $3\n  _ -> $0", "match with a default arm"),
        ("try", "try\n  $1\ncatch ${2:error}\n  $0", "try / catch"),
        ("use", "use ${0:module}", "import a module"),
    ]
}

fn kind_snippet(kind: &str) -> String {
    match kind {
        "stateless" => "stateless ${1:Name}${2:(${3:title}: ${4:string})}\n  build(context: BuildContext) Widget\n    ${0:Placeholder()}".into(),
        "stateful" => "stateful ${1:Name}\n  ${2:count}: ${3:int} = ${4:0}\n\n  build(context: BuildContext) Widget\n    ${0:Placeholder()}".into(),
        "data" => format!("{kind} ${{1:Name}}\n  ${{2:field}}: ${{3:string}}$0"),
        "extension" => format!("{kind} ${{1:Name}} on ${{2:Type}}\n  $0"),
        "mixin" => format!("{kind} ${{1:Name}}\n  $0"),
        _ => format!("{kind} ${{1:Name}}\n  $0"),
    }
}

pub fn completion(doc: &Doc, dialect: Option<&Dialect>, p: &Value) -> Value {
    let at = offset(doc, p);
    let text = &doc.text;
    let ix = &doc.index;
    let start = word_start(text, at);
    let prefix = &text[start..at];
    let before = text[..start].trim_end_matches([' ', '\t']);
    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let at_line_start = text[line_start..start].trim().is_empty();
    let mut items: Vec<Value> = Vec::new();

    // `object.` → members of a module, kind or enum of this file, static members of an external class, or
    // members of a value whose type is known
    if let Some((obj_start, obj_end)) = object_before(text, start) {
        match object_at(doc, dialect, obj_start, obj_end) {
            Some(Object::File(owner)) => {
                for &m in &ix.bindings[owner].members {
                    items.push(binding_item(doc, m, "0"));
                }
            }
            Some(Object::Project(src, other, owner)) => {
                for &m in &other.bindings[owner].members {
                    items.push(binding_item_in(src, other, m, "0"));
                }
            }
            Some(Object::Static(class)) => {
                if let Some((_, d)) = type_defs(dialect).and_then(|(td, _)| td.get(&class)) {
                    for item in typedefs::static_items(d) {
                        items.push(match item {
                            StaticItem::Value(v) => json!({
                                "label": v, "kind": KIND_ENUM_MEMBER, "detail": format!("{class}.{v}"), "sortText": "0",
                            }),
                            StaticItem::Constructor(c) => json!({
                                "label": c.name,
                                "kind": KIND_CONSTRUCTOR,
                                "detail": defs::render_constructor(&class, c, ""),
                                "documentation": docs(c.doc.as_ref()),
                                "tags": tags(c.deprecated),
                                "sortText": format!("1{}", c.name),
                            }),
                            StaticItem::Member(m) => member_item(m, "2"),
                        });
                    }
                }
            }
            Some(Object::Instance(ty)) => {
                if let Some((td, names)) = type_defs(dialect) {
                    for f in typedefs::instance_members(td, &names, &ty) {
                        items.push(member_item(f.member, "0"));
                    }
                }
            }
            None => {}
        }
        return json!({ "isIncomplete": false, "items": items });
    }

    // `@screen(rou` / `@screen(loggedIn = { for` → arguments and fields of the decorator (`[decorators.args]`)
    if let Some(d) = dialect
        && (at_line_start || before.ends_with(['(', ',', '{']))
        && let Some((decorator, path)) = decorator_context(text, start)
        && let Some(args) = d.decorator_args.get(decorator)
    {
        let assign = if path.is_empty() { " = " } else { ": " };
        for a in args {
            let required = a.ends_with('!');
            let Some(name) = a.trim_end_matches('!').strip_prefix(path.as_str()).filter(|n| !n.contains('.')) else {
                continue;
            };
            let full = format!("{path}{name}");
            items.push(json!({
                "label": name,
                "kind": KIND_FIELD,
                "detail": format!("@{decorator}{}", if required { " (required)" } else { "" }),
                "insertText": format!("{name}{assign}"),
                "sortText": format!("{}{name}", if required { "00" } else { "01" }),
                "documentation": dialects::decorator_arg_doc(decorator, &full, d)
                    .map(|v| json!({ "kind": "markdown", "value": v })),
            }));
        }
        return json!({ "isIncomplete": false, "items": items });
    }

    // `@` → decorators
    if text[..start].ends_with('@') {
        for name in ["override", "deprecated", "pragma"] {
            items.push(json!({ "label": name, "kind": KIND_KEYWORD }));
        }
        if let Some(d) = dialect {
            for name in dialects::decorator_names(d) {
                items.push(json!({
                    "label": name,
                    "kind": KIND_KEYWORD,
                    "detail": format!("{} dialect", d.name),
                    "documentation": dialects::decorator_doc(name, d).map(|v| json!({ "kind": "markdown", "value": v })),
                }));
            }
        }
        return json!({ "isIncomplete": false, "items": items });
    }

    // `$` → fields of the enclosing module
    if text[..start].ends_with('$') {
        if let Some(m) = ix.enclosing_module(at) {
            for &f in &ix.bindings[m].members {
                if matches!(ix.bindings[f].kind, BindKind::Field | BindKind::Getter) {
                    items.push(binding_item(doc, f, "0"));
                }
            }
        }
        return json!({ "isIncomplete": false, "items": items });
    }

    // named arguments of a call to something declared in this file: `Greeting(name = ...)`
    if (before.ends_with('(') || before.ends_with(',') || (before.ends_with('\n') || at_line_start))
        && let Some(callable) = enclosing_call(doc, at).and_then(|(_, b)| b)
    {
        named_param_items(text, ix, callable, &written_named_args(text, at), &mut items);
    }
    // named arguments of a call to a declaration of another file of the project: `CharacterTile(character = ...)`
    if (before.ends_with('(') || before.ends_with(',') || (before.ends_with('\n') || at_line_start))
        && let Some((paren, None)) = enclosing_call(doc, at)
        && let Some((src, other, callable)) = project_callable(doc, paren)
    {
        named_param_items(src, other, callable, &written_named_args(text, at), &mut items);
    }
    // named arguments of an external constructor or method: `Text('a', style = ...)`
    if (before.ends_with('(') || before.ends_with(',') || (before.ends_with('\n') || at_line_start))
        && let Some((paren, None)) = enclosing_call(doc, at)
        && let Some(callable) = external_callable(doc, dialect, paren)
    {
        let written = written_named_args(text, at);
        for p in callable.params.iter().filter(|p| p.named && !written.contains(&p.name)) {
            let required = !p.optional;
            items.push(json!({
                "label": format!("{} =", p.name),
                "filterText": p.name,
                "kind": KIND_FIELD,
                "detail": defs::render_param(p),
                "insertText": format!("{} = ", p.name),
                "sortText": format!("0{}{}", if required { 0 } else { 1 }, p.name),
                "preselect": required,
            }));
        }
    }

    // declarations and statements at the start of a line
    if at_line_start {
        let indent = start - line_start;
        if indent == 0
            && let Some(d) = dialect
        {
            for (name, kind) in &d.kinds {
                items.push(json!({
                    "label": name,
                    "labelDetails": { "description": kind_source(d, name) },
                    "kind": KIND_SNIPPET,
                    "detail": kind.value.description,
                    "documentation": d.doc("kinds", name).map(|v| json!({ "kind": "markdown", "value": v })),
                    "insertText": kind_snippet(name),
                    "insertTextFormat": 2,
                    "sortText": format!("1{name}"),
                }));
            }
        }
        // methods of an external base to override, at the member level of a module (`build`, `initState`)
        if indent == 2
            && let Some((td, names)) = type_defs(dialect)
            && let Some(m) = ix.enclosing_module(at)
        {
            let declared: HashSet<&str> =
                ix.bindings[m].members.iter().map(|&b| ix.bindings[b].name.as_str()).collect();
            let mut offered = HashSet::new();
            for base in module_bases(doc, dialect, at) {
                for f in typedefs::instance_members(td, &names, &base) {
                    let m = f.member;
                    if m.kind != "method" || declared.contains(m.name.as_str()) || !offered.insert(m.name.clone()) {
                        continue;
                    }
                    let signature = defs::render_member(m, "");
                    items.push(json!({
                        "label": m.name,
                        "kind": KIND_METHOD,
                        "detail": format!("override · {}", f.owner.name),
                        "labelDetails": { "detail": signature.trim_start_matches(m.name.as_str()) },
                        "documentation": docs(m.doc.as_ref()),
                        "tags": tags(m.deprecated),
                        "insertText": format!("{}\n  $0", snippet_text(&signature)),
                        "insertTextFormat": 2,
                        "sortText": format!("0{}", m.name),
                    }));
                }
            }
        }
        for (label, body, detail) in statement_snippets() {
            items.push(json!({
                "label": label,
                "kind": KIND_SNIPPET,
                "detail": detail,
                "insertText": body,
                "insertTextFormat": 2,
                "sortText": format!("3{label}"),
            }));
        }
        if let Some(d) = dialect {
            for name in d.modifiers.keys() {
                items.push(json!({
                    "label": name,
                    "kind": KIND_KEYWORD,
                    "detail": "modifier",
                    "documentation": d.doc("words", name).map(|v| json!({ "kind": "markdown", "value": v })),
                    "sortText": format!("4{name}"),
                }));
            }
        }
    }

    // everything visible here
    let mut seen: HashSet<String> = HashSet::new();
    for b in ix.visible(at) {
        let name = &ix.bindings[b].name;
        if name.as_str() == prefix && ix.bindings[b].span.end == at {
            continue; // the word being typed
        }
        if seen.insert(name.clone()) {
            items.push(binding_item(doc, b, "2"));
        }
    }
    if let (Some(m), Some(d)) = (ix.enclosing_module(at), dialect) {
        for word in &ix.bindings[m].words {
            if let Some(kind) = d.kinds.get(word) {
                for member in &kind.value.members {
                    if seen.insert(member.clone()) {
                        items.push(json!({
                            "label": member,
                            "kind": KIND_FUNCTION,
                            "detail": format!("from the `{word}` kind"),
                            "sortText": format!("2{member}"),
                        }));
                    }
                }
            }
        }
    }
    // top-level declarations of the other files of the project
    if let Some((p, path)) = &doc.project {
        for n in p.visible(path) {
            if seen.insert(n.name.clone()) {
                items.push(json!({
                    "label": n.name,
                    "kind": if is_pascal(&n.name) { KIND_CLASS } else { KIND_FUNCTION },
                    "detail": n.signature,
                    "labelDetails": { "description": format!("{}.nuc", n.path) },
                    "sortText": format!("2{}", n.name),
                }));
            }
        }
    }
    // members inherited from external bases (`emit`, `state`, `context`)
    if let Some((td, names)) = type_defs(dialect) {
        for base in module_bases(doc, dialect, at) {
            for f in typedefs::instance_members(td, &names, &base) {
                if seen.insert(f.member.name.clone()) {
                    items.push(member_item(f.member, "2"));
                }
            }
        }
    }
    for k in Keyword::ALL {
        items.push(json!({
            "label": k.as_str(),
            "kind": KIND_KEYWORD,
            "documentation": dialects::keyword_doc(k.as_str()).map(|v| json!({ "kind": "markdown", "value": v })),
            "sortText": format!("5{}", k.as_str()),
        }));
    }
    for t in PRIMITIVE_TYPES {
        if seen.insert((*t).to_string()) {
            items.push(json!({ "label": t, "kind": KIND_CLASS, "detail": "base type", "sortText": format!("6{t}") }));
        }
    }

    // names imported automatically by the dialect (thousands in Flutter): only those matching the prefix
    // public declarations of the type definitions, then the names of `[auto_imports]` that are not in them
    let mut incomplete = false;
    if let (Some(d), false) = (dialect, prefix.is_empty()) {
        let lower_prefix = prefix.to_lowercase();
        let mut count = 0;
        if let Some((td, names)) = type_defs(dialect) {
            for (p, decl) in names.with_prefix(td, prefix) {
                if !seen.insert(decl.name.clone()) {
                    continue;
                }
                count += 1;
                if count > MAX_LIBRARY_ITEMS {
                    incomplete = true;
                    break;
                }
                items.push(decl_item(p, decl));
            }
        }
        for (name, module) in &d.auto_imports {
            if incomplete || !name.to_lowercase().starts_with(&lower_prefix) || seen.contains(name) {
                continue;
            }
            count += 1;
            if count > MAX_LIBRARY_ITEMS {
                incomplete = true;
                break;
            }
            items.push(json!({
                "label": name,
                "kind": if is_pascal(name) { KIND_CLASS } else { KIND_FUNCTION },
                "detail": module.value,
                "labelDetails": { "description": module.value },
                "sortText": format!("7{name}"),
            }));
        }
        for name in d.functions.keys().chain(d.globals.iter()) {
            if seen.insert(name.clone()) {
                items.push(json!({ "label": name, "kind": KIND_FUNCTION, "detail": "dialect", "sortText": format!("7{name}") }));
            }
        }
    }
    json!({ "isIncomplete": incomplete, "items": items })
}

/// The declaration of another file of the project called at `paren` (`CharacterTile(`).
fn project_callable(doc: &Doc, paren: usize) -> Option<(&str, &Index, usize)> {
    let (p, path) = doc.project.as_ref()?;
    let end = generic_start(&doc.text, paren);
    let name = &doc.text[word_start(&doc.text, end)..end];
    if name.is_empty() || doc.index.visible(paren).into_iter().any(|b| doc.index.bindings[b].name == name) {
        return None;
    }
    p.decl(path, name).filter(|(_, ix, b)| ix.bindings[*b].kind.is_callable())
}

/// `name =` items for the parameters of `callable` (a binding of `ix`, whose source is `text`) not yet written.
fn named_param_items(text: &str, ix: &Index, callable: usize, written: &HashSet<String>, items: &mut Vec<Value>) {
    for &param in &ix.bindings[callable].params {
        let bind = &ix.bindings[param];
        if written.contains(&bind.name) {
            continue;
        }
        let ty = ix.type_text(text, param).unwrap_or("");
        items.push(json!({
            "label": format!("{} =", bind.name),
            "filterText": bind.name,
            "kind": KIND_FIELD,
            "detail": if bind.optional { format!("{ty} (optional)") } else { ty.to_string() },
            "insertText": format!("{} = ", bind.name),
            "sortText": format!("0{}{}", if bind.optional { 1 } else { 0 }, bind.name),
            "preselect": !bind.optional,
        }));
    }
}

/// The innermost unclosed `(` before `at`, and the declaration it calls (if declared in this file).
fn enclosing_call(doc: &Doc, at: usize) -> Option<(usize, Option<usize>)> {
    let text = &doc.text;
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut i = at;
    let limit = at.saturating_sub(8000);
    let mut in_string: Option<u8> = None;
    while i > limit {
        i -= 1;
        let c = bytes[i];
        if let Some(q) = in_string {
            if c == q {
                in_string = None;
            }
            continue;
        }
        match c {
            b'\'' | b'"' => in_string = Some(c),
            b')' | b']' | b'}' => depth += 1,
            b'[' | b'{' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
            }
            b'(' => {
                if depth == 0 {
                    return Some((i, callee_binding(doc, i)));
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Declaration called by the `(` at `paren`: `Greeting(`, `load(`, `FutureBuilder<T>(`.
fn callee_binding(doc: &Doc, paren: usize) -> Option<usize> {
    let text = &doc.text;
    let end = generic_start(text, paren);
    let start = word_start(text, end);
    if start == end {
        return None;
    }
    let ix = &doc.index;
    // while typing, the call may not parse yet: fall back to the declaration with that name
    let b = ix.binding_at(start).or_else(|| {
        let name = &text[start..end];
        ix.visible(start).into_iter().find(|&b| ix.bindings[b].name == name)
    })?;
    ix.bindings[b].kind.is_callable().then_some(b)
}

/// Names already written as `name =` in the call around `at`.
fn written_named_args(text: &str, at: usize) -> HashSet<String> {
    let mut out = HashSet::new();
    let open = text[..at].rfind('(').unwrap_or(0);
    let close = text[at..].find(')').map_or(text.len(), |i| at + i);
    for part in text[open..close].split([',', '\n', '(']) {
        if let Some((name, _)) = part.split_once('=') {
            let name = name.trim();
            if !name.is_empty() && name.chars().all(is_word_char) && !part.contains("==") {
                out.insert(name.to_string());
            }
        }
    }
    out
}

// ---------------------------------------------------------------- on type formatting

/// Enter on an empty ` *` line of a comment ends the comment: the editor already put the new line under the
/// `//` (`onEnterRules`); this removes the empty ` *` line it leaves above.
pub fn on_type_formatting(doc: &Doc, p: &Value) -> Value {
    let line = p["position"]["line"].as_u64().unwrap_or(0) as usize;
    let lines: Vec<&str> = doc.text.split('\n').collect();
    if p["ch"] != "\n" || line < 2 || line >= lines.len() {
        return json!([]);
    }
    let comment_line = |l: &str| {
        let t = l.trim_start();
        t.starts_with("//") || t == "*" || t.starts_with("* ")
    };
    if lines[line - 1].trim() != "*" || !comment_line(lines[line - 2]) {
        return json!([]);
    }
    let range = json!({ "start": { "line": line - 1, "character": 0 }, "end": { "line": line, "character": 0 } });
    json!([{ "range": range, "newText": "" }])
}

// ---------------------------------------------------------------- signature help

pub fn signature_help(doc: &Doc, dialect: Option<&Dialect>, p: &Value) -> Value {
    let at = offset(doc, p);
    let Some((paren, callable)) = enclosing_call(doc, at) else { return Value::Null };
    let Some(callable) = callable else {
        return external_callable(doc, dialect, paren).map_or(Value::Null, |c| external_signature(doc, &c, paren, at));
    };
    let ix = &doc.index;
    let text = &doc.text;
    let bind = &ix.bindings[callable];
    if bind.params.is_empty() {
        return Value::Null;
    }
    let comment = ix.doc_comment(text, callable).map(|c| DocComment::parse(&c));
    let mut label = format!("{}(", bind.name);
    let mut params = Vec::new();
    for (i, &param) in bind.params.iter().enumerate() {
        if i > 0 {
            label.push_str(", ");
        }
        let start = label.encode_utf16().count();
        let pb = &ix.bindings[param];
        label.push_str(&pb.name);
        if pb.optional {
            label.push('?');
        }
        if let Some(ty) = ix.type_text(text, param) {
            label.push_str(": ");
            label.push_str(ty);
        }
        let end = label.encode_utf16().count();
        let param_doc = comment.as_ref().and_then(|c| c.param(&pb.name));
        params.push(json!({ "label": [start, end], "documentation": param_doc }));
    }
    label.push(')');

    let (position, named) = current_argument(text, paren, at);
    let active = named
        .and_then(|n| bind.params.iter().position(|&p| ix.bindings[p].name == n))
        .unwrap_or(position.min(bind.params.len() - 1));
    json!({
        "signatures": [{
            "label": label,
            "parameters": params,
            "activeParameter": active,
            // the parameters are shown one by one, next to the argument being written
            "documentation": comment.map(|c| json!({ "kind": "markdown", "value": DocComment { params: Vec::new(), ..c }.markdown() })),
        }],
        "activeSignature": 0,
        "activeParameter": active,
    })
}

/// The argument being written in the call opened at `paren`: its position and its name (`name = ...`).
fn current_argument(text: &str, paren: usize, at: usize) -> (usize, Option<&str>) {
    let args = &text[paren + 1..at];
    let mut depth = 0;
    let mut current_start = 0;
    let mut position = 0;
    for (i, c) in args.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                position += 1;
                current_start = i + 1;
            }
            _ => {}
        }
    }
    let current = args[current_start..].trim_start();
    let named = current
        .split_once('=')
        .filter(|(_, rest)| !rest.starts_with('='))
        .map(|(n, _)| n.trim())
        .filter(|n| !n.is_empty() && n.chars().all(is_word_char));
    (position, named)
}

/// `Scaffold({key?: Key, appBar?: PreferredSizeWidget, ...})`: positional parameters first, then the named
/// ones between `{ }`, as in a Nuclear signature.
fn external_signature(doc: &Doc, c: &Callable, paren: usize, at: usize) -> Value {
    let positional: Vec<&Param> = c.params.iter().filter(|p| !p.named).collect();
    let named: Vec<&Param> = c.params.iter().filter(|p| p.named).collect();
    let mut label = format!("{}(", c.label);
    let mut params = Vec::new();
    for (i, p) in positional.iter().chain(&named).enumerate() {
        if i > 0 {
            label.push_str(", ");
        }
        if i == positional.len() {
            label.push('{');
        }
        let start = label.encode_utf16().count();
        label.push_str(&defs::render_param(p));
        params.push(json!({ "label": [start, label.encode_utf16().count()] }));
    }
    if !named.is_empty() {
        label.push('}');
    }
    label.push(')');
    let (position, name) = current_argument(&doc.text, paren, at);
    let active = match name {
        Some(n) => named.iter().position(|p| p.name == n).map(|i| positional.len() + i),
        None if position < positional.len() => Some(position),
        None => None,
    };
    let mut sig = json!({ "label": label, "parameters": params, "documentation": docs(c.doc) });
    if let Some(a) = active {
        sig["activeParameter"] = json!(a);
    }
    json!({ "signatures": [sig], "activeSignature": 0, "activeParameter": active.map_or(json!(null), |a| json!(a)) })
}

// ---------------------------------------------------------------- navigation

pub fn definition(doc: &Doc, p: &Value) -> Value {
    let at = offset(doc, p);
    if let Some(b) = doc.index.binding_at(at) {
        return location(doc, doc.index.bindings[b].span);
    }
    // a declaration of another file of the project
    let Some(o) = doc.index.occurrence_at(at) else { return Value::Null };
    if object_before(&doc.text, o.span.start).is_some() {
        return Value::Null;
    }
    match doc.project_name(&doc.text[o.span.start..o.span.end]) {
        Some(n) => json!({ "uri": n.uri, "range": n.range }),
        None => Value::Null,
    }
}

pub fn references(doc: &Doc, p: &Value) -> Value {
    let Some(b) = doc.index.binding_at(offset(doc, p)) else { return Value::Null };
    let include_decl = p["context"]["includeDeclaration"].as_bool().unwrap_or(true);
    let decl = doc.index.bindings[b].span;
    let locs: Vec<Value> =
        doc.index.references(b).filter(|o| include_decl || o.span != decl).map(|o| location(doc, o.span)).collect();
    json!(locs)
}

pub fn highlights(doc: &Doc, p: &Value) -> Value {
    let Some(b) = doc.index.binding_at(offset(doc, p)) else { return Value::Null };
    let decl = doc.index.bindings[b].span;
    let list: Vec<Value> = doc
        .index
        .references(b)
        .map(|o| json!({ "range": doc.lines.range(&doc.text, o.span), "kind": if o.span == decl { 3 } else { 2 } }))
        .collect();
    json!(list)
}

pub fn prepare_rename(doc: &Doc, p: &Value) -> Value {
    let at = offset(doc, p);
    let Some(o) = doc.index.occurrence_at(at) else { return Value::Null };
    if !matches!(o.target, Target::Binding(_)) {
        return Value::Null;
    }
    json!({ "range": doc.lines.range(&doc.text, o.span), "placeholder": &doc.text[o.span.start..o.span.end] })
}

pub fn rename(doc: &Doc, p: &Value) -> Result<Value, String> {
    let at = offset(doc, p);
    let new_name = p["newName"].as_str().unwrap_or_default();
    let Some(b) = doc.index.binding_at(at) else { return Ok(Value::Null) };
    let valid =
        new_name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') && new_name.chars().all(is_word_char);
    if !valid {
        return Err(format!("`{new_name}` is not a valid name"));
    }
    if Keyword::from_text(new_name).is_some() {
        return Err(format!("`{new_name}` is a reserved word"));
    }
    let old = &doc.index.bindings[b].name;
    if is_pascal(old) != is_pascal(new_name) {
        return Err(
            "PascalCase and camelCase mean different things in Nuclear (module vs. function); keep the same case"
                .into(),
        );
    }
    let edits: Vec<Value> = doc
        .index
        .references(b)
        .map(|o| json!({ "range": doc.lines.range(&doc.text, o.span), "newText": new_name }))
        .collect();
    Ok(json!({ "changes": { doc.uri.clone(): edits } }))
}

// ---------------------------------------------------------------- outline and folding

fn symbol_kind(kind: BindKind) -> u32 {
    match kind {
        BindKind::Module | BindKind::Kind => 5,
        BindKind::Method => 6,
        BindKind::Getter => 7,
        BindKind::Field | BindKind::Param => 8,
        BindKind::Enum => 10,
        BindKind::Function => 12,
        BindKind::Local => 13,
        BindKind::EnumItem => 22,
        BindKind::TypeParam => 26,
    }
}

fn document_symbol(doc: &Doc, b: usize) -> Value {
    let ix = &doc.index;
    let bind = &ix.bindings[b];
    let mut children: Vec<usize> = Vec::new();
    if bind.kind == BindKind::Kind {
        children.extend(bind.params.iter().copied().filter(|p| !bind.members.contains(p)));
    }
    if matches!(bind.kind, BindKind::Module | BindKind::Kind | BindKind::Enum) {
        children.extend(bind.members.iter().copied());
    }
    let mut detail = bind.words.join(" ");
    if let Some(ty) = ix.type_text(&doc.text, b) {
        if !detail.is_empty() {
            detail.push(' ');
        }
        detail.push_str(ty);
    }
    let decl = Span::new(bind.decl.start.min(bind.span.start), bind.decl.end.max(bind.span.end));
    json!({
        "name": bind.name,
        "detail": detail,
        "kind": symbol_kind(bind.kind),
        "range": doc.lines.range(&doc.text, decl),
        "selectionRange": doc.lines.range(&doc.text, bind.span),
        "children": children.into_iter().map(|c| document_symbol(doc, c)).collect::<Vec<_>>(),
    })
}

pub fn document_symbols(doc: &Doc) -> Value {
    json!(doc.index.toplevel.iter().map(|&b| document_symbol(doc, b)).collect::<Vec<_>>())
}

/// Folding by indentation (Nuclear blocks are indented) plus runs of `//` comment lines.
pub fn folding_ranges(doc: &Doc) -> Value {
    let lines: Vec<&str> = doc.text.split('\n').collect();
    let mut out = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (line, indent)
    let mut last_non_blank = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        while let Some(&(start, ind)) = stack.last() {
            if indent > ind {
                break;
            }
            stack.pop();
            if last_non_blank > start {
                out.push(json!({ "startLine": start, "endLine": last_non_blank }));
            }
        }
        stack.push((i, indent));
        last_non_blank = i;
    }
    while let Some((start, _)) = stack.pop() {
        if last_non_blank > start {
            out.push(json!({ "startLine": start, "endLine": last_non_blank }));
        }
    }
    // consecutive comment lines at the same indentation
    let mut run: Option<(usize, usize)> = None;
    for (i, line) in lines.iter().enumerate().chain(std::iter::once((lines.len(), &""))) {
        if line.trim_start().starts_with("//") {
            run = Some((run.map_or(i, |r| r.0), i));
        } else if let Some((s, e)) = run.take()
            && e > s
        {
            out.push(json!({ "startLine": s, "endLine": e, "kind": "comment" }));
        }
    }
    json!(out)
}

// ---------------------------------------------------------------- code actions

pub fn code_actions(doc: &Doc, p: &Value) -> Value {
    let mut actions = Vec::new();
    for d in p["context"]["diagnostics"].as_array().into_iter().flatten() {
        let fix = &d["data"]["fix"];
        if !fix.is_object() {
            continue;
        }
        let range = &fix["range"];
        let new_text = fix["newText"].as_str().unwrap_or_default();
        let old = {
            let span = doc.lines.span_of(&doc.text, range);
            doc.text.get(span.start..span.end).unwrap_or_default().to_string()
        };
        let title =
            if new_text.is_empty() { format!("Delete `{old}`") } else { format!("Replace `{old}` with `{new_text}`") };
        actions.push(json!({
            "title": title,
            "kind": "quickfix",
            "diagnostics": [d],
            "isPreferred": true,
            "edit": { "changes": { doc.uri.clone(): [{ "range": range, "newText": new_text }] } },
        }));
    }
    json!(actions)
}
