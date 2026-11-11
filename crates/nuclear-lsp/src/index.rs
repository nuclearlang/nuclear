//! Symbol index of one `.nuc` file.
//!
//! Walks the AST once and records:
//! - every declaration (`Binding`): modules, kinds, enums, functions, methods, fields, parameters, locals...
//! - every name written in the source (`Occurrence`), with its semantic class and what it refers to.
//!
//! Name resolution follows spec/nuclear-base.md: variables are block scoped and the first assignment of a
//! name declares it; members of a module are visible inside all its methods; top-level declarations are
//! visible everywhere in the file. Everything the editor features need (highlighting, hover, go to
//! definition, references, rename, completion) is answered from this index.

use nuclear_diag::Span;
use nuclear_dialect::Dialect;
use nuclear_syntax::ast::*;
use nuclear_syntax::token::Comment;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindKind {
    /// PascalCase declaration with members (a class in the target language).
    Module,
    Enum,
    /// Declaration whose header has a dialect kind (`stateless Greeting`).
    Kind,
    Function,
    Method,
    Getter,
    Field,
    Param,
    Local,
    EnumItem,
    TypeParam,
}

impl BindKind {
    pub fn label(self) -> &'static str {
        match self {
            BindKind::Module => "module",
            BindKind::Enum => "enum",
            BindKind::Kind => "kind",
            BindKind::Function => "function",
            BindKind::Method => "method",
            BindKind::Getter => "getter",
            BindKind::Field => "field",
            BindKind::Param => "parameter",
            BindKind::Local => "variable",
            BindKind::EnumItem => "enum item",
            BindKind::TypeParam => "type parameter",
        }
    }

    pub fn is_callable(self) -> bool {
        matches!(self, BindKind::Function | BindKind::Method | BindKind::Module | BindKind::Kind)
    }

    pub fn is_type(self) -> bool {
        matches!(self, BindKind::Module | BindKind::Enum | BindKind::Kind | BindKind::TypeParam)
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub kind: BindKind,
    /// The name in its declaration.
    pub span: Span,
    /// The whole declaration (first line is used as its signature).
    pub decl: Span,
    /// Where the name is visible.
    pub scope: Span,
    /// Enclosing module or function.
    pub parent: Option<usize>,
    /// Header words (`stateless`, `static`, `get`).
    pub words: Vec<String>,
    /// Parameters (functions, methods, kinds; a module's constructor parameters).
    pub params: Vec<usize>,
    /// Fields, methods and enum items of a module, kind or enum.
    pub members: Vec<usize>,
    /// Written type (fields, parameters, typed locals) or return type (functions).
    pub ty: Option<Span>,
    /// Parameter written with `?` or with a default value.
    pub optional: bool,
    /// Bases of a module or kind as written (`Cubit<int>`, `> Comparable`, `with Flyer`).
    pub bases: Vec<String>,
    /// Type of a local or field without a written type, from its value: `dio = Dio()` → `Dio`,
    /// `theme = Theme.of(context)` → `ThemeData` (from the type definitions).
    pub inferred: Option<String>,
}

/// Semantic token types. The order is the LSP legend (`TOKEN_TYPES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tok {
    Namespace,
    Type,
    Class,
    Enum,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Function,
    Method,
    Keyword,
    Modifier,
    Decorator,
}

pub const TOKEN_TYPES: &[&str] = &[
    "namespace",
    "type",
    "class",
    "enum",
    "typeParameter",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "function",
    "method",
    "keyword",
    "modifier",
    "decorator",
];

pub const TOKEN_MODIFIERS: &[&str] =
    &["declaration", "readonly", "static", "async", "defaultLibrary", "kind", "deprecated"];
pub const MOD_DECLARATION: u32 = 1;
pub const MOD_READONLY: u32 = 1 << 1;
pub const MOD_STATIC: u32 = 1 << 2;
pub const MOD_ASYNC: u32 = 1 << 3;
pub const MOD_DEFAULT_LIBRARY: u32 = 1 << 4;
/// A dialect kind in a declaration header (`stateless`, `stateful`, `extension`).
pub const MOD_KIND: u32 = 1 << 5;
/// Deprecated in the type definitions of the dialect.
pub const MOD_DEPRECATED: u32 = 1 << 6;

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    None,
    Binding(usize),
    /// A name imported automatically by the dialect, with its module (`package:flutter/widgets.dart`).
    Library(String),
    /// A dialect `[functions]` shortcut or `[names] globals` entry.
    DialectName,
    /// A name provided by the kind of the enclosing declaration (`setState` in `stateful`).
    KindMember(String),
    /// A word in a declaration header: a kind, a modifier or a clause.
    HeaderWord(HeaderRole),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderRole {
    Kind,
    Modifier,
    Clause,
}

#[derive(Debug, Clone)]
pub struct Occurrence {
    pub span: Span,
    pub tok: Tok,
    pub mods: u32,
    pub target: Target,
}

#[derive(Debug, Default)]
pub struct Index {
    pub bindings: Vec<Binding>,
    /// Sorted by position, without overlaps.
    pub occs: Vec<Occurrence>,
    pub comments: Vec<Comment>,
    /// Top-level declarations, in source order.
    pub toplevel: Vec<usize>,
}

pub const PRIMITIVE_TYPES: &[&str] = &[
    "string", "int", "double", "num", "bool", "void", "dynamic", "float", "number", "any", "object", "never", "Object",
];

impl Index {
    /// `default_kind`: the kind of a project folder (`[project.kinds]`), for declarations without a kind word.
    pub fn build(src: &str, file: &File, dialect: Option<&Dialect>, default_kind: Option<&str>) -> Index {
        let mut ix = Indexer {
            src,
            dialect,
            default_kind: default_kind.filter(|k| dialect.is_some_and(|d| d.kinds.contains_key(*k))).map(String::from),
            bindings: Vec::new(),
            occs: Vec::new(),
            scopes: vec![Scope { span: Span::new(0, src.len()), names: HashMap::new() }],
            module: None,
            kind: None,
            parent: None,
            deferred: Vec::new(),
        };
        let toplevel = ix.file(file);
        ix.resolve_deferred();
        let mut occs = ix.occs;
        occs.sort_by_key(|o| (o.span.start, std::cmp::Reverse(o.span.end)));
        let mut clean: Vec<Occurrence> = Vec::with_capacity(occs.len());
        for o in occs {
            if o.span.end <= o.span.start || o.span.end > src.len() {
                continue;
            }
            if clean.last().is_some_and(|prev| o.span.start < prev.span.end) {
                continue;
            }
            clean.push(o);
        }
        Index { bindings: ix.bindings, occs: clean, comments: file.comments.clone(), toplevel }
    }

    /// The occurrence under the cursor (the cursor may be right after the last character).
    pub fn occurrence_at(&self, offset: usize) -> Option<&Occurrence> {
        let i = self.occs.partition_point(|o| o.span.end < offset);
        self.occs[i..].iter().take(2).find(|o| o.span.start <= offset && offset <= o.span.end)
    }

    pub fn binding_at(&self, offset: usize) -> Option<usize> {
        match self.occurrence_at(offset)?.target {
            Target::Binding(b) => Some(b),
            _ => None,
        }
    }

    /// Every occurrence of binding `b`, its declaration included.
    pub fn references(&self, b: usize) -> impl Iterator<Item = &Occurrence> {
        self.occs.iter().filter(move |o| o.target == Target::Binding(b))
    }

    /// Bindings visible at `offset`, innermost first.
    pub fn visible(&self, offset: usize) -> Vec<usize> {
        let mut out: Vec<usize> = (0..self.bindings.len())
            .filter(|&i| {
                let b = &self.bindings[i];
                let inside = b.scope.start <= offset && offset <= b.scope.end;
                let declared = !matches!(b.kind, BindKind::Local) || b.span.end <= offset;
                inside && declared
            })
            .collect();
        out.sort_by_key(|&i| std::cmp::Reverse(self.bindings[i].scope.start));
        let mut seen = std::collections::HashSet::new();
        out.retain(|&i| seen.insert(self.bindings[i].name.clone()));
        out
    }

    /// Innermost module, kind or enum whose declaration contains `offset`.
    pub fn enclosing_module(&self, offset: usize) -> Option<usize> {
        self.toplevel.iter().copied().find(|&i| {
            let b = &self.bindings[i];
            matches!(b.kind, BindKind::Module | BindKind::Kind | BindKind::Enum)
                && b.decl.start <= offset
                && offset <= b.decl.end
        })
    }

    /// `//` comments on their own lines right above the declaration.
    pub fn doc_comment(&self, src: &str, b: usize) -> Option<String> {
        let decl_start = self.bindings[b].decl.start;
        let line_start = src[..decl_start].rfind('\n').map_or(0, |i| i + 1);
        let mut lines = Vec::new();
        let mut expected_end = line_start;
        for c in self.comments.iter().rev().filter(|c| c.own_line && c.span.end <= line_start) {
            // the comment must end on the line right before `expected_end`
            let between = &src[c.span.end..expected_end];
            if between.chars().filter(|&ch| ch == '\n').count() > 1 || !between.trim().is_empty() {
                break;
            }
            lines.push(c.text.trim_end().to_string());
            expected_end = src[..c.span.start].rfind('\n').map_or(0, |i| i + 1);
        }
        if lines.is_empty() {
            return None;
        }
        lines.reverse();
        Some(lines.join("\n"))
    }

    /// First line of the declaration, as written.
    pub fn signature(&self, src: &str, b: usize) -> String {
        let bind = &self.bindings[b];
        let start = match bind.kind {
            // a declaration may start with decorators; the signature starts at its header line
            BindKind::Module
            | BindKind::Kind
            | BindKind::Enum
            | BindKind::Function
            | BindKind::Method
            | BindKind::Getter => src[..bind.span.start].rfind('\n').map_or(0, |i| i + 1),
            _ => bind.decl.start,
        };
        let end = src[start..].find('\n').map_or(src.len(), |i| start + i).min(src.len());
        let end = if matches!(bind.kind, BindKind::Param) { bind.decl.end.min(end).max(bind.span.end) } else { end };
        src[start..end].trim().to_string()
    }

    pub fn type_text<'s>(&self, src: &'s str, b: usize) -> Option<&'s str> {
        self.bindings[b].ty.map(|t| &src[t.start..t.end])
    }
}

struct Scope {
    span: Span,
    names: HashMap<String, usize>,
}

struct Indexer<'a> {
    src: &'a str,
    dialect: Option<&'a Dialect>,
    default_kind: Option<String>,
    bindings: Vec<Binding>,
    occs: Vec<Occurrence>,
    scopes: Vec<Scope>,
    /// Module / kind / enum being walked.
    module: Option<usize>,
    /// Dialect kind of the declaration being walked (`stateful`).
    kind: Option<String>,
    parent: Option<usize>,
    /// References to members, resolved once every declaration of the file is known (a call may name a
    /// widget or an enum declared further down).
    deferred: Vec<Deferred>,
}

/// What a deferred reference points into.
#[derive(Clone)]
enum Owner {
    /// A declaration (`Greeting(...)`, `Status.alive`).
    Binding(usize),
    /// A member of a declaration (`Character.fromJson(...)`).
    Member(usize, String),
}

enum Deferred {
    /// `name = value` in a call.
    NamedArg { span: Span, callee: Owner, name: String },
    /// `Owner.name`.
    Member { span: Span, owner: usize, name: String, called: bool },
}

impl<'a> Indexer<'a> {
    // ------------------------------------------------------------ helpers

    fn occ(&mut self, span: Span, tok: Tok, mods: u32, target: Target) {
        self.occs.push(Occurrence { span, tok, mods, target });
    }

    fn push_scope(&mut self, span: Span) {
        self.scopes.push(Scope { span, names: HashMap::new() });
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn lookup(&self, name: &str) -> Option<usize> {
        self.scopes.iter().rev().find_map(|s| s.names.get(name).copied())
    }

    fn bind(&mut self, name: &Ident, kind: BindKind, decl: Span, mods: u32) -> usize {
        let id = self.bindings.len();
        let scope = self.scopes.last().map_or(Span::default(), |s| s.span);
        self.bindings.push(Binding {
            name: name.name.clone(),
            kind,
            span: name.span,
            decl,
            scope,
            parent: self.parent,
            words: Vec::new(),
            params: Vec::new(),
            members: Vec::new(),
            ty: None,
            optional: false,
            bases: Vec::new(),
            inferred: None,
        });
        if let Some(s) = self.scopes.last_mut() {
            s.names.insert(name.name.clone(), id);
        }
        self.occ(name.span, tok_of(kind), mods | MOD_DECLARATION, Target::Binding(id));
        id
    }

    fn is_kind(&self, word: &str) -> bool {
        self.dialect.is_some_and(|d| d.kinds.contains_key(word))
    }

    fn is_modifier(&self, word: &str) -> bool {
        self.dialect.is_some_and(|d| d.modifiers.contains_key(word))
    }

    fn kind_member(&self, name: &str) -> Option<String> {
        let kind = self.kind.as_ref()?;
        let k = self.dialect?.kinds.get(kind)?;
        k.value.members.iter().any(|m| m == name).then(|| kind.clone())
    }

    // ------------------------------------------------------------ file

    fn file(&mut self, file: &File) -> Vec<usize> {
        // first pass: every top-level declaration is visible in the whole file
        let mut ids = Vec::new();
        for item in &file.items {
            if let Item::Decl(d) = item {
                let mut kind = self.decl_kind(d, true);
                let folder_kind = self.default_kind.clone().filter(|_| kind == BindKind::Module && d.words.is_empty());
                if folder_kind.is_some() {
                    kind = BindKind::Kind;
                }
                let id = self.bind(&d.name, kind, d.span, decl_mods(d));
                self.bindings[id].words = d.words.iter().map(|w| w.name.clone()).chain(folder_kind).collect();
                ids.push(id);
            }
        }
        let mut next = ids.iter();
        for item in &file.items {
            match item {
                Item::Use(u) => self.use_item(u),
                Item::Decl(d) => {
                    if let Some(&id) = next.next() {
                        self.decl(d, id);
                    }
                }
                Item::Stmt(s) => self.stmt(s),
            }
        }
        ids
    }

    fn use_item(&mut self, u: &Use) {
        for n in &u.names {
            self.occ(n.span, if n.is_pascal() { Tok::Class } else { Tok::Function }, 0, Target::None);
        }
        if let Some(a) = &u.alias {
            self.occ(a.span, Tok::Namespace, MOD_DECLARATION, Target::None);
        }
        for c in &u.clauses {
            self.occ(c.word.span, Tok::Keyword, 0, Target::None);
            for n in &c.names {
                self.occ(n.span, if n.is_pascal() { Tok::Class } else { Tok::Function }, 0, Target::None);
            }
        }
    }

    // ------------------------------------------------------------ declarations

    fn decl_kind(&self, d: &Decl, toplevel: bool) -> BindKind {
        if d.words.iter().any(|w| self.is_kind(&w.name)) {
            return BindKind::Kind;
        }
        if d.name.is_pascal() {
            return match &d.body {
                Body::Members(m) if m.iter().any(|m| matches!(m, Member::EnumItem(_))) => BindKind::Enum,
                _ if toplevel && !d.words.is_empty() && self.dialect.is_none() => BindKind::Kind,
                _ => BindKind::Module,
            };
        }
        if d.words.iter().any(|w| w.name == "get") {
            BindKind::Getter
        } else if toplevel {
            BindKind::Function
        } else {
            BindKind::Method
        }
    }

    fn decorators(&mut self, decorators: &[Decorator]) {
        for dec in decorators {
            let name_end = dec.span.start + 1 + dec.name.len();
            self.occ(Span::new(dec.span.start, name_end.min(dec.span.end)), Tok::Decorator, 0, Target::None);
            if let Some(args) = &dec.args {
                for a in args {
                    self.expr(&a.value, false);
                }
            }
        }
    }

    fn header_words(&mut self, words: &[Ident], pascal: bool) {
        for (i, w) in words.iter().enumerate() {
            let (tok, mods, role) = if self.is_kind(&w.name) || (self.dialect.is_none() && pascal && i == 0) {
                (Tok::Keyword, MOD_KIND, HeaderRole::Kind)
            } else if self.is_modifier(&w.name) || !pascal {
                (Tok::Modifier, 0, HeaderRole::Modifier)
            } else {
                (Tok::Keyword, MOD_KIND, HeaderRole::Kind)
            };
            self.occ(w.span, tok, mods, Target::HeaderWord(role));
        }
    }

    fn decl(&mut self, d: &Decl, id: usize) {
        self.decorators(&d.decorators);
        self.header_words(&d.words, d.name.is_pascal());

        let saved_kind = self.kind.clone();
        if let Some(k) = self.bindings[id].words.iter().find(|w| self.is_kind(w)) {
            self.kind = Some(k.clone());
        }
        let saved_parent = self.parent.replace(id);
        self.push_scope(d.span);

        for g in &d.generics {
            self.bind(g, BindKind::TypeParam, g.span, 0);
        }
        // `<T: Comparable>`: the bound is a type in use
        for bound in d.generic_bounds.iter().flatten() {
            self.ty(bound);
        }
        for t in d.bases.iter().chain(&d.implements) {
            self.ty(t);
        }
        for c in &d.clauses {
            self.occ(c.word.span, Tok::Keyword, 0, Target::HeaderWord(HeaderRole::Clause));
            for t in &c.types {
                self.ty(t);
            }
        }
        let with = d.clauses.iter().filter(|c| c.word.name == "with").flat_map(|c| &c.types);
        let bases = d.bases.iter().chain(&d.implements).chain(with);
        self.bindings[id].bases =
            bases.filter_map(|t| self.src.get(t.span.start..t.span.end)).map(String::from).collect();
        if let Some(r) = &d.ret {
            self.ty(r);
            self.bindings[id].ty = Some(r.span);
        }
        if let Some(p) = &d.params {
            let params = self.params(&p.list, BindKind::Param);
            self.bindings[id].params = params;
        }

        match &d.body {
            Body::None => {}
            Body::Members(members) => {
                let saved_module = self.module.replace(id);
                self.members(id, members);
                self.module = saved_module;
            }
            Body::Block(b) => self.block(b),
            Body::Expr(e) => self.expr(e, false),
        }

        self.pop_scope();
        self.parent = saved_parent;
        self.kind = saved_kind;
    }

    fn params(&mut self, list: &[Param], kind: BindKind) -> Vec<usize> {
        let mut ids = Vec::new();
        for p in list {
            self.decorators(&p.decorators);
            if let Some(t) = &p.ty {
                self.ty(t);
            }
            if let Some(def) = &p.default {
                self.expr(def, false);
            }
            let id = self.bind(&p.name, kind, p.span, 0);
            self.bindings[id].ty = p.ty.as_ref().map(|t| t.span);
            self.bindings[id].optional = p.optional || p.default.is_some();
            ids.push(id);
        }
        ids
    }

    fn members(&mut self, owner: usize, members: &[Member]) {
        // first pass: members are visible in every method of the module
        let mut method_ids = Vec::new();
        for m in members {
            match m {
                Member::Field(f) => {
                    let mods = if f.words.iter().any(|w| w.name == "static") { MOD_STATIC } else { 0 };
                    let id = self.bind(&f.name, BindKind::Field, f.span, mods);
                    self.bindings[id].ty = f.ty.as_ref().map(|t| t.span);
                    self.bindings[id].words = f.words.iter().map(|w| w.name.clone()).collect();
                    self.bindings[id].optional = f.optional || f.default.is_some();
                    self.bindings[id].inferred = f.default.as_ref().and_then(|v| self.value_type(v));
                    self.bindings[owner].members.push(id);
                }
                Member::Method(d) => {
                    let kind = self.decl_kind(d, false);
                    let id = self.bind(&d.name, kind, d.span, decl_mods(d));
                    self.bindings[id].words = d.words.iter().map(|w| w.name.clone()).collect();
                    self.bindings[owner].members.push(id);
                    method_ids.push(id);
                }
                Member::EnumItem(e) => {
                    let id = self.bind(&e.name, BindKind::EnumItem, e.span, MOD_READONLY | MOD_STATIC);
                    self.bindings[owner].members.push(id);
                }
                Member::Ctor(c) => {
                    // constructor parameters declare the module's fields (spec §4.3)
                    for p in &c.params.list {
                        let id = self.bind(&p.name, BindKind::Field, p.span, 0);
                        self.bindings[id].ty = p.ty.as_ref().map(|t| t.span);
                        self.bindings[id].optional = p.optional || p.default.is_some();
                        self.bindings[owner].members.push(id);
                        self.bindings[owner].params.push(id);
                    }
                }
            }
        }
        let mut next = method_ids.iter();
        for m in members {
            match m {
                Member::Field(f) => {
                    self.decorators(&f.decorators);
                    self.header_words(&f.words, false);
                    if let Some(t) = &f.ty {
                        self.ty(t);
                    }
                    if let Some(d) = &f.default {
                        self.expr(d, false);
                    }
                }
                Member::Method(d) => {
                    if let Some(&id) = next.next() {
                        self.decl(d, id);
                    }
                }
                Member::EnumItem(e) => {
                    for a in &e.args {
                        self.expr(a, false);
                    }
                }
                Member::Ctor(c) => {
                    for p in &c.params.list {
                        self.decorators(&p.decorators);
                        if let Some(t) = &p.ty {
                            self.ty(t);
                        }
                        if let Some(d) = &p.default {
                            self.expr(d, false);
                        }
                    }
                    if let Some(b) = &c.body {
                        self.block(b);
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ statements

    fn block(&mut self, b: &Block) {
        self.push_scope(b.span);
        for s in &b.stmts {
            self.stmt(s);
        }
        self.pop_scope();
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Expr(e) => self.expr(e, false),
            Stmt::Assign(a) => self.assign(a),
            Stmt::If(i) => self.if_(i),
            Stmt::For(f) => {
                self.expr(&f.iter, false);
                self.push_scope(f.span);
                for v in &f.vars {
                    self.bind(v, BindKind::Local, f.span, MOD_READONLY);
                }
                self.block(&f.body);
                self.pop_scope();
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond, false);
                self.block(body);
            }
            Stmt::Match(m) => self.match_(m),
            Stmt::Try(t) => {
                self.block(&t.body);
                if let Some((var, body)) = &t.catch {
                    self.push_scope(body.span);
                    if let Some(v) = var {
                        self.bind(v, BindKind::Local, v.span, MOD_READONLY);
                    }
                    self.block(body);
                    self.pop_scope();
                }
                if let Some(f) = &t.finally {
                    self.block(f);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.expr(v, false);
                }
            }
            Stmt::Throw { value, .. } => self.expr(value, false),
            Stmt::Yield { value, .. } => self.expr(value, false),
            Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn assign(&mut self, a: &Assign) {
        self.expr(&a.value, false);
        if let Some(t) = &a.ty {
            self.ty(t);
        }
        if let ExprKind::Name(name) = &a.target.kind
            && self.lookup(name).is_none()
            && a.op == AssignOp::Set
            && self.kind_member(name).is_none()
        {
            let ident = Ident { name: name.clone(), span: a.target.span };
            let id = self.bind(&ident, BindKind::Local, a.span, 0);
            self.bindings[id].ty = a.ty.as_ref().map(|t| t.span);
            self.bindings[id].inferred = self.value_type(&a.value);
            return;
        }
        if a.op == AssignOp::Set && matches!(a.target.kind, ExprKind::List(_) | ExprKind::Object(_)) {
            self.pattern(&a.target, a.span);
            return;
        }
        self.expr(&a.target, false);
    }

    /// Destructuring: `[a, ...rest] = xs`, `{ name, "k": v } = m`. Each name of the pattern is declared,
    /// or assigned when it already exists; `_` ignores an element.
    fn pattern(&mut self, target: &Expr, decl: Span) {
        match &target.kind {
            ExprKind::Name(n) if n == "_" => {}
            ExprKind::Name(n) => self.pattern_name(&Ident { name: n.clone(), span: target.span }, decl),
            ExprKind::Spread(x) => self.pattern(x, decl),
            ExprKind::List(items) => {
                for x in items {
                    self.pattern(x, decl);
                }
            }
            ExprKind::Object(entries) => {
                for entry in entries {
                    match entry {
                        ObjEntry::Shorthand(id) => self.pattern_name(id, decl),
                        ObjEntry::Pair { key, value } => {
                            if let ObjKey::Ident(k) = key {
                                self.occ(k.span, Tok::Property, 0, Target::None);
                            }
                            self.pattern(value, decl);
                        }
                        ObjEntry::Spread(x) => self.pattern(x, decl),
                    }
                }
            }
            _ => self.expr(target, false),
        }
    }

    fn pattern_name(&mut self, name: &Ident, decl: Span) {
        if self.lookup(&name.name).is_some() {
            self.name_ref(&name.name, name.span, false);
        } else {
            self.bind(name, BindKind::Local, decl, 0);
        }
    }

    fn if_(&mut self, i: &If) {
        for (cond, body) in &i.branches {
            self.expr(cond, false);
            self.block(body);
        }
        if let Some(b) = &i.otherwise {
            self.block(b);
        }
    }

    fn match_(&mut self, m: &Match) {
        self.expr(&m.subject, false);
        for arm in &m.arms {
            for p in &arm.patterns {
                match p {
                    Pattern::Value(e) => self.expr(e, false),
                    Pattern::Is(t) => self.ty(t),
                    Pattern::Wildcard(_) => {}
                }
            }
            match &arm.body {
                ArmBody::Expr(e) => self.expr(e, false),
                ArmBody::Block(b) => self.block(b),
            }
        }
    }

    // ------------------------------------------------------------ expressions

    fn expr(&mut self, e: &Expr, called: bool) {
        match &e.kind {
            ExprKind::Name(n) => self.name_ref(n, e.span, called),
            ExprKind::SelfField(n) => {
                let span = Span::new(e.span.start + 1, e.span.end);
                let target = self.module.and_then(|m| self.member_of(m, n));
                let tok = target.map_or(Tok::Property, |b| tok_of(self.bindings[b].kind));
                self.occ(span, tok, 0, target.map_or(Target::None, Target::Binding));
            }
            ExprKind::This | ExprKind::Super | ExprKind::Number(_) | ExprKind::Bool(_) | ExprKind::Null => {}
            ExprKind::Str(pieces) => {
                for p in pieces {
                    if let StrPiece::Interp(x) = p {
                        self.expr(x, false);
                    }
                }
            }
            ExprKind::List(items) => {
                for x in items {
                    self.expr(x, false);
                }
            }
            ExprKind::Object(entries) => {
                for entry in entries {
                    match entry {
                        ObjEntry::Pair { key, value } => {
                            if let ObjKey::Ident(k) = key {
                                self.occ(k.span, Tok::Property, 0, Target::None);
                            }
                            self.expr(value, false);
                        }
                        ObjEntry::Shorthand(id) => self.name_ref(&id.name, id.span, false),
                        ObjEntry::Spread(x) => self.expr(x, false),
                    }
                }
            }
            ExprKind::Unary { expr, .. } => self.expr(expr, called),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left, false);
                self.expr(right, false);
            }
            ExprKind::Ternary { cond, then, otherwise } => {
                self.expr(cond, false);
                self.expr(then, false);
                self.expr(otherwise, false);
            }
            ExprKind::Is { expr, ty } | ExprKind::Cast { expr, ty } => {
                self.expr(expr, false);
                self.ty(ty);
            }
            ExprKind::Member { object, name, .. } => {
                self.expr(object, false);
                self.member_ref(object, name, called);
            }
            ExprKind::Index { object, index } => {
                self.expr(object, false);
                self.expr(index, false);
            }
            ExprKind::Call { callee, args, type_args } => {
                self.expr(callee, true);
                for t in type_args {
                    self.ty(t);
                }
                let target = self.callee_owner(callee);
                for a in args {
                    if let Some(n) = &a.name {
                        match &target {
                            Some(owner) => self.deferred.push(Deferred::NamedArg {
                                span: n.span,
                                callee: owner.clone(),
                                name: n.name.clone(),
                            }),
                            None => self.occ(n.span, Tok::Parameter, 0, Target::None),
                        }
                    }
                    if let Some(t) = &a.ty {
                        self.ty(t);
                    }
                    self.expr(&a.value, false);
                }
            }
            ExprKind::Spread(x) | ExprKind::NonNull(x) => self.expr(x, called),
            ExprKind::ListIf { cond, value } => {
                self.expr(cond, false);
                self.expr(value, false);
            }
            ExprKind::Cascade { target, sections } => {
                self.expr(target, false);
                for section in sections {
                    self.expr(&section.access, false);
                    if let Some((_, value)) = &section.assign {
                        self.expr(value, false);
                    }
                }
            }
            ExprKind::CascadeSelf => {}
            ExprKind::Lambda(l) => {
                self.push_scope(l.span);
                self.params(&l.params, BindKind::Param);
                if let Some(r) = &l.ret {
                    self.ty(r);
                }
                match &l.body {
                    LambdaBody::Expr(x) => self.expr(x, false),
                    LambdaBody::Block(b) => self.block(b),
                }
                self.pop_scope();
            }
            ExprKind::If(i) => self.if_(i),
            ExprKind::Match(m) => self.match_(m),
        }
    }

    fn name_ref(&mut self, name: &str, span: Span, called: bool) {
        if let Some(b) = self.lookup(name) {
            let kind = self.bindings[b].kind;
            let mut tok = tok_of(kind);
            if called && matches!(kind, BindKind::Local | BindKind::Param | BindKind::Field) {
                tok = Tok::Function;
            }
            self.occ(span, tok, 0, Target::Binding(b));
        } else if let Some(kind) = self.kind_member(name) {
            let tok = if called { Tok::Function } else { Tok::Variable };
            self.occ(span, tok, MOD_DEFAULT_LIBRARY, Target::KindMember(kind));
        } else if let Some(module) = self.dialect.and_then(|d| d.auto_imports.get(name)) {
            let tok = if is_pascal(name) {
                Tok::Class
            } else if called {
                Tok::Function
            } else {
                Tok::Variable
            };
            let mods = MOD_DEFAULT_LIBRARY | self.deprecated(name);
            self.occ(span, tok, mods, Target::Library(module.value.clone()));
        } else if self.dialect.is_some_and(|d| d.functions.contains_key(name) || d.globals.contains(name)) {
            let tok = if called { Tok::Function } else { Tok::Variable };
            self.occ(span, tok, MOD_DEFAULT_LIBRARY, Target::DialectName);
        } else if is_pascal(name) {
            let mods = if PRIMITIVE_TYPES.contains(&name) { MOD_DEFAULT_LIBRARY } else { 0 };
            self.occ(span, Tok::Class, mods, Target::None);
        } else if PRIMITIVE_TYPES.contains(&name) {
            // `int.parse(...)`: a base type used as a value
            self.occ(span, Tok::Type, MOD_DEFAULT_LIBRARY, Target::None);
        } else {
            self.occ(span, if called { Tok::Function } else { Tok::Variable }, 0, Target::None);
        }
    }

    fn member_of(&self, owner: usize, name: &str) -> Option<usize> {
        let o = &self.bindings[owner];
        o.members.iter().chain(o.params.iter()).copied().find(|&m| self.bindings[m].name == name)
    }

    fn param_of(&self, callable: usize, name: &str) -> Option<usize> {
        self.bindings[callable].params.iter().copied().find(|&p| self.bindings[p].name == name)
    }

    /// The declaration a call refers to: `Greeting(...)`, `load()`, `Status.values`, `this.save()`.
    fn callee_owner(&self, callee: &Expr) -> Option<Owner> {
        match &callee.kind {
            ExprKind::Name(n) => self.lookup(n).map(Owner::Binding),
            ExprKind::Member { object, name, .. } => {
                self.member_owner(object).map(|o| Owner::Member(o, name.name.clone()))
            }
            _ => None,
        }
    }

    /// The declaration whose members `object.x` names: a module, kind or enum, or `this`.
    fn member_owner(&self, object: &Expr) -> Option<usize> {
        match &object.kind {
            ExprKind::Name(n) => self.lookup(n).filter(|&o| self.bindings[o].kind.is_type()),
            ExprKind::This => self.module,
            _ => None,
        }
    }

    fn member_ref(&mut self, object: &Expr, name: &Ident, called: bool) {
        if let Some(owner) = self.member_owner(object) {
            self.deferred.push(Deferred::Member { span: name.span, owner, name: name.name.clone(), called });
            return;
        }
        // `Colors.red`, `Text.rich`: static members of an external class
        let mut mods = 0;
        if let ExprKind::Name(class) = &object.kind
            && is_pascal(class)
            && self.lookup(class).is_none()
            && let Some((_, d)) = self.dialect.and_then(|d| d.typedefs.get(class))
        {
            let deprecated = d.members.iter().any(|m| m.is_static && m.name == name.name && m.deprecated)
                || d.constructors.iter().any(|c| c.name == name.name && c.deprecated);
            if deprecated {
                mods = MOD_DEPRECATED;
            }
            if d.values.contains(&name.name) {
                self.occ(name.span, Tok::EnumMember, mods | MOD_READONLY, Target::None);
                return;
            }
        }
        self.occ(name.span, member_tok(None, &name.name, called), mods, Target::None);
    }

    /// `MOD_DEPRECATED` when the type definitions mark `name` as deprecated.
    fn deprecated(&self, name: &str) -> u32 {
        match self.dialect.and_then(|d| d.typedefs.get(name)) {
            Some((_, d)) if d.deprecated => MOD_DEPRECATED,
            _ => 0,
        }
    }

    /// The type of a value that builds an object: `Dio()`, `Cubit<int>()`, `Text.rich(...)`, `Theme.of(c)`.
    fn value_type(&self, e: &Expr) -> Option<String> {
        let ExprKind::Call { callee, type_args, .. } = &e.kind else { return None };
        match &callee.kind {
            ExprKind::Name(n) if is_pascal(n) => {
                let args: Vec<&str> = type_args.iter().filter_map(|t| self.src.get(t.span.start..t.span.end)).collect();
                Some(if args.is_empty() { n.clone() } else { format!("{n}<{}>", args.join(", ")) })
            }
            ExprKind::Member { object, name, .. } => {
                let ExprKind::Name(class) = &object.kind else { return None };
                if !is_pascal(class) || self.lookup(class).is_some() {
                    return None;
                }
                let (_, d) = self.dialect?.typedefs.get(class)?;
                crate::typedefs::static_call_type(d, &name.name)
            }
            _ => None,
        }
    }

    fn resolve_deferred(&mut self) {
        for d in std::mem::take(&mut self.deferred) {
            match d {
                Deferred::Member { span, owner, name, called } => {
                    let target = self.member_of(owner, &name);
                    let tok = member_tok(target.map(|b| self.bindings[b].kind), &name, called);
                    self.occ(span, tok, 0, target.map_or(Target::None, Target::Binding));
                }
                Deferred::NamedArg { span, callee, name } => {
                    let callable = match callee {
                        Owner::Binding(b) => Some(b),
                        Owner::Member(o, m) => self.member_of(o, &m),
                    };
                    let param = callable.and_then(|c| self.param_of(c, &name));
                    self.occ(span, Tok::Parameter, 0, param.map_or(Target::None, Target::Binding));
                }
            }
        }
    }

    // ------------------------------------------------------------ types

    fn ty(&mut self, t: &Type) {
        match &t.kind {
            TypeKind::Named { name, args } => {
                let end = t.span.start + name.len();
                if !name.contains('.') && self.src.get(t.span.start..end) == Some(name.as_str()) {
                    let span = Span::new(t.span.start, end);
                    if let Some(b) = self.lookup(name).filter(|&b| self.bindings[b].kind.is_type()) {
                        self.occ(span, tok_of(self.bindings[b].kind), 0, Target::Binding(b));
                    } else if PRIMITIVE_TYPES.contains(&name.as_str()) {
                        self.occ(span, Tok::Type, MOD_DEFAULT_LIBRARY, Target::None);
                    } else if let Some(module) = self.dialect.and_then(|d| d.auto_imports.get(name)) {
                        let mods = MOD_DEFAULT_LIBRARY | self.deprecated(name);
                        self.occ(span, Tok::Class, mods, Target::Library(module.value.clone()));
                    } else {
                        self.occ(span, Tok::Class, 0, Target::None);
                    }
                }
                for a in args {
                    self.ty(a);
                }
            }
            TypeKind::List(inner) | TypeKind::Nullable(inner) => self.ty(inner),
            TypeKind::Map(k, v) => {
                self.ty(k);
                self.ty(v);
            }
            TypeKind::Union(items) => {
                for i in items {
                    self.ty(i);
                }
            }
            TypeKind::Func { params, ret } => {
                for p in params {
                    if let Some(t) = &p.ty {
                        self.ty(t);
                    }
                }
                self.ty(ret);
            }
            TypeKind::StrLit(_) | TypeKind::NumLit(_) => {}
        }
    }
}

fn member_tok(kind: Option<BindKind>, name: &str, called: bool) -> Tok {
    match kind {
        Some(k) => tok_of(k),
        None if is_pascal(name) => Tok::Class,
        None if called => Tok::Method,
        None => Tok::Property,
    }
}

fn decl_mods(d: &Decl) -> u32 {
    let mut m = 0;
    if d.is_async {
        m |= MOD_ASYNC;
    }
    if d.words.iter().any(|w| w.name == "static") {
        m |= MOD_STATIC;
    }
    m
}

pub fn tok_of(kind: BindKind) -> Tok {
    match kind {
        BindKind::Module | BindKind::Kind => Tok::Class,
        BindKind::Enum => Tok::Enum,
        BindKind::Function => Tok::Function,
        BindKind::Method => Tok::Method,
        BindKind::Getter | BindKind::Field => Tok::Property,
        BindKind::Param => Tok::Parameter,
        BindKind::Local => Tok::Variable,
        BindKind::EnumItem => Tok::EnumMember,
        BindKind::TypeParam => Tok::TypeParameter,
    }
}
