//! Emitter: walks the AST, resolves names (locals, implicit `this`, constructors, shortcuts,
//! auto-imports) and renders every construct with the active dialect's templates.
//!
//! Nothing here knows a target language: TypeScript, Dart or any other target is described by its
//! dialect files. This file only decides *what* to render and passes the values to the templates.

use crate::analysis;
use nuclear_diag::{Code, Diagnostic, Span, codes};
use nuclear_dialect::Dialect;
use nuclear_dialect::library;
use nuclear_dialect::model::{FunctionShortcut, ImportShortcut, Kind, closest};
use nuclear_dialect::template::{Ctx, Template, Value};
use nuclear_syntax::DocComment;
use nuclear_syntax::ast::*;
use nuclear_syntax::token::{Comment, NumberKind};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Path of the project file that holds the dialect's Result runtime (project mode).
pub(crate) const RESULT_PATH: &str = "nuclear/result";

// precedence of rendered expressions (higher binds tighter)
const P_LAMBDA: u8 = 0;
const P_TERNARY: u8 = 1;
const P_UNARY: u8 = 14;
const P_POSTFIX: u8 = 16;
const P_PRIMARY: u8 = 17;

/// What a top-level declaration turned out to be.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DeclKind {
    Function,
    Class,
    Interface,
    Enum,
    Kind(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumKind {
    Int,
    Double,
}

fn num_kind_of(t: &Type) -> Option<NumKind> {
    match &t.kind {
        TypeKind::Named { name, .. } if name == "int" => Some(NumKind::Int),
        TypeKind::Named { name, .. } if name == "double" => Some(NumKind::Double),
        _ => None,
    }
}

/// What other code needs to know about a top-level declaration.
#[derive(Debug, Clone)]
pub(crate) struct DeclInfo {
    pub(crate) kind: DeclKind,
    /// Parameters when called (function) or constructed (module).
    params: Vec<Param>,
    methods: HashMap<String, Vec<Param>>,
    /// Fields, methods and constructor parameters (modules).
    members: HashSet<String>,
    /// Names of the bases and clause types (`: Animal`, `with Flyer`, `on String`).
    bases: Vec<String>,
    /// Return type and async flag (functions).
    sig: Sig,
    /// Return type and async flag of each method (modules).
    method_sigs: HashMap<String, Sig>,
    /// Types of the fields and constructor parameters (modules).
    field_types: HashMap<String, Type>,
    /// The first field with each decorator (`primary` → `id`), for `fields_decor` of a type.
    field_decor: HashMap<String, String>,
    /// The declaration's decorators as templates read them (`decor.entity.arg`), for `tinfo.decor` of a type.
    decor: Value,
    /// Alias methods (`list() = ListTasks.exec`) → (module, method): their signature is the target's.
    aliases: HashMap<String, (String, String)>,
}

/// What a call returns: its declared return type and whether it is async.
#[derive(Debug, Clone, Default)]
pub(crate) struct Sig {
    ret: Option<Type>,
    is_async: bool,
}

/// A value of type Result<E, T>: what is known about the error E and the value T.
#[derive(Debug, Clone, Default)]
struct ResInfo {
    ok: Option<Type>,
    err: Option<Type>,
}

/// The function (or lambda) whose body is being emitted, as Result propagation sees it.
#[derive(Debug, Clone, Default)]
struct FnCtx {
    /// It returns Result<_, E>: errors propagate to it. The inner option is E, when it is written.
    result: Option<Option<Type>>,
    /// Its last line returns a Result as it is (it returns Result, or its type is not written).
    keep_tail: bool,
}

/// One arm of a match on a Result: `Ok(user) ->`, `Err(e) ->` or `_ ->`.
struct ResultArm<'m> {
    /// Some(true) = Ok, Some(false) = Err, None = `_`
    ok: Option<bool>,
    bind: Option<String>,
    body: &'m ArmBody,
    span: Span,
}

/// How the last statement of a block is used.
enum Tail {
    None,
    /// `return <last>`
    Return,
    /// `<target> = <last>` (an if/match used as the value of an assignment)
    Assign(String),
}

/// One function body being emitted.
#[derive(Default)]
struct Frame {
    /// source name → emitted name
    vars: HashMap<String, String>,
    mutable: HashSet<String>,
    /// Variables of blocks that already ended: using them is error N0105.
    gone: HashSet<String>,
    /// Known types of the variables (parameters, typed or constructed values): source name → type.
    types: HashMap<String, Type>,
}

/// The module whose members are being emitted.
#[derive(Clone)]
struct ModCtx {
    /// Source name of the module.
    name: String,
    members: HashSet<String>,
    ctor_params: HashSet<String>,
    param_access: Option<Template>,
    /// It extends something declared outside this file, whose members are unknown.
    open: bool,
}

/// What a file of a project knows about the other files (`project::build`).
#[derive(Default)]
pub(crate) struct ProjectCtx {
    pub name: String,
    /// `app(version, description)` of the manifest, for the generated manifest's template (`app.version`).
    pub version: String,
    pub description: String,
    /// `app(title)`: the app's name as people read it (empty: the name).
    pub title: String,
    /// `dependencies(...)` of the manifest: name, version and, for a folder on disk, its absolute path.
    pub dependencies: Vec<(String, String, Option<String>)>,
    /// Source name → declaration of a file of the project.
    pub decls: HashMap<String, ExtDecl>,
    /// Names declared by the dialect's project files (`[project.files]`) → their path.
    pub generated: HashMap<String, String>,
    /// Folders of `[project.outputs]` (test, integration_test): what they declare is only seen inside them.
    pub isolated: Vec<String>,
    /// The plugins `config.nuc` enables (with the ones they require). `None`: the project has no manifest of
    /// plugins, so nothing is checked.
    pub plugins: Option<HashSet<String>>,
    /// `[project] sources` and `[project.outputs]`: where each file goes (for relative imports).
    pub sources: String,
    pub outputs: Vec<(String, String)>,
}

impl ProjectCtx {
    /// The output path of a source path, without extension: `components/x` → `src/components/x`.
    pub fn out_stem(&self, path: &str) -> String {
        for (from, to) in &self.outputs {
            if let Some(rest) = path.strip_prefix(&format!("{from}/")) {
                return format!("{to}/{rest}");
            }
        }
        if self.sources.is_empty() { path.to_string() } else { format!("{}/{path}", self.sources) }
    }

    /// The import path from the output file `from_out` (with extension or not) to the source path `to`:
    /// `./data/task`, `../src/components/tasks/task` (TypeScript's relative imports).
    pub fn relative(&self, from_out: &str, to: &str) -> String {
        relative_path(from_out, &self.out_stem(to))
    }

    /// Can the file `from` use what the file `path` declares? Not when `path` is in an isolated folder
    /// (a test) and `from` is not in the same one.
    pub fn visible(&self, path: &str, from: &str) -> bool {
        match self.isolated.iter().find(|f| path.starts_with(&format!("{f}/"))) {
            Some(f) => from.starts_with(&format!("{f}/")),
            None => true,
        }
    }
}

/// `src/a/b/x` → `src/a/c/y` gives `../c/y`; a path in the same folder or below starts with `./`.
pub(crate) fn relative_path(from_file: &str, to: &str) -> String {
    let from: Vec<&str> = from_file.split('/').collect();
    let from_dir = &from[..from.len().saturating_sub(1)];
    let to: Vec<&str> = to.split('/').collect();
    let common = from_dir.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let ups = from_dir.len() - common;
    let rest = to[common..].join("/");
    if ups == 0 { format!("./{rest}") } else { format!("{}{rest}", "../".repeat(ups)) }
}

#[derive(Debug, Clone)]
pub(crate) struct ExtDecl {
    pub out_name: String,
    /// Path of its file in the project, without extension (`components/users/data/user_repository`).
    pub path: String,
    pub info: DeclInfo,
}

/// A top-level declaration as the dialect's project files see it.
#[derive(Debug, Clone)]
pub(crate) struct DeclSummary {
    pub name: String,
    pub kind: Option<String>,
    pub decor: Value,
    /// Constructor parameters: name, type.
    pub params: Value,
    /// Fields: name, type, tinfo, optional, default.
    pub fields: Value,
    /// Methods: name, decor, deco_args, params_list, param_groups, ret_info.
    pub methods: Value,
    /// Type parameters of the declaration: name.
    pub generics: Value,
}

pub struct Emitter<'a> {
    d: &'a Dialect,
    project: Option<&'a ProjectCtx>,
    /// This file's path in the project, without extension.
    file_path: String,
    /// Names written in this file (project mode: the files to import).
    used_names: HashSet<String>,
    /// Kind of the declarations without a kind word (`[project] kinds` of this file's folder).
    default_kind: Option<String>,
    /// Kinds declared in this file (their `file_header` is written once).
    kinds_used: BTreeSet<String>,
    /// Objects provided to the declaration being written (`@provider([...])`, a kind's `provide`):
    /// name → how it is read.
    provided: HashMap<String, Template>,
    /// Decorators already checked against `[decorators.args]` (by position), so each is reported once.
    checked_decorators: HashSet<usize>,
    /// The kind being written.
    cur_kind: Option<&'a Kind>,
    /// Name of the module being written, as emitted.
    owner: String,
    /// The code names Result, Ok or Err (project mode: import the runtime file).
    result_import: bool,
    /// Decorators of the module being written, for the templates of its members (`owner_decor`).
    owner_decor: Value,
    /// Names written by kinds next to their declaration (`[kinds.x] declares`).
    extra_decls: Vec<String>,
    /// Data of the last method written (`methods` of kinds and project files).
    last_method: Option<Value>,
    /// Type parameters of the declaration being written (`T` of `Page<T>`): `tinfo.generic`.
    type_params: Vec<String>,
    /// Every generic parameter named so far in the file (`T` of `pick<T>()`): never an unknown class (N0220).
    generic_names: HashSet<String>,
    pub(crate) summaries: Vec<DeclSummary>,
    pub diags: Vec<Diagnostic>,
    /// Line map of the last `file`: (output line, source offset), both from 1 and 0.
    pub marks: Vec<(u32, usize)>,
    decls: HashMap<String, DeclInfo>,
    /// Decorators already checked against the platform's (N0309), so each is reported once.
    checked_platform: HashSet<usize>,
    /// Classes of the type definitions known to read themselves from JSON (`reads_json`).
    json_classes: HashMap<String, bool>,
    /// source name → emitted name of declarations (kinds may rename: `controller Users` → `UsersController`)
    decl_names: HashMap<String, String>,
    imported: HashSet<String>,
    /// A `use` brings a whole module or file: its names are unknown, so N0106 is not reported.
    open_imports: bool,
    /// Constructor parameters of the module being written that are members of its base.
    inherited_params: HashSet<String>,
    /// Inherited constructor parameters written as super parameters (`super.name`).
    super_params: HashSet<String>,
    /// How the object of the cascade being written is rendered (`ExprKind::CascadeSelf`).
    cascade_self: Vec<String>,
    frames: Vec<Frame>,
    module: Option<ModCtx>,
    kind_syntax: Option<&'a BTreeMap<String, Template>>,
    /// names referenced but not declared: candidates for auto-imports
    refs: BTreeSet<String>,
    missing_reported: HashSet<String>,
    comments: Vec<Comment>,
    next_comment: usize,
    /// Result, Ok and Err of the base language (false when this file declares its own).
    result_builtin: bool,
    /// The file uses Result: the dialect's `result_runtime` is written after the imports.
    uses_result: bool,
    /// The source text, to read a member's name as written (naming may have renamed it: `trim_start`).
    source: String,
    /// The next Member rendered is the callee of a call that is not a library call: no property rewrite.
    callee_member: bool,
    /// The expression being rendered is the target of an assignment (`m[k] = v` stays an index).
    assign_target: bool,
    /// Helpers of `[library.*]` members the file uses (`base.member` → text), written once after the imports.
    helpers: BTreeMap<String, String>,
    /// Types for the untyped parameters of the next lambda (the element type of a library receiver).
    lambda_hints: Vec<Type>,
    /// Functions and lambdas being emitted (innermost last).
    fn_ctx: Vec<FnCtx>,
    /// Lines that run before the current statement: the Results it unwraps.
    pending: Vec<String>,
    /// > 0 inside a branch that may not run: a Result there cannot be unwrapped before the statement.
    hoist_blocked: u32,
    /// Local functions that lambdas with a block became (options.hoist_block_lambdas), for the statement.
    hoisted: Vec<String>,
    lambda_count: usize,
    /// The parameter names of the lambda being written, without types (`names` of `lambda`/`lambda_expr`).
    lambda_names: String,
    /// options.doc_inside: the doc comment of the declaration being written, for its template's `doc`.
    inner_doc: Option<String>,
    /// The summary of the last doc comment read (`doc_text` of the next method in `methods`).
    doc_summary: Option<String>,
    /// The words of the generated declarations, once written (imports only what the code writes).
    written_words: Option<HashSet<String>>,
    /// Name → the disabled plugin it belongs to (built on first use); names already reported.
    plugin_names: Option<HashMap<String, String>>,
    plugin_reported: HashSet<String>,
    /// Fields the next constructor sets (options.init_fields): (name, value).
    ctor_inits: Vec<(String, String)>,
    /// The next expression keeps its Result instead of unwrapping it.
    keep_next: bool,
    /// Name for the next temporary Result (`user = find()` → `userResult`).
    temp_hint: Option<String>,
    /// The next lambda is a callback of .ok()/.error() that returns a Result.
    next_lambda_result: bool,
    /// Emitted name of a Result variable → its value and error types, rendered (templates such as
    /// `result_value` may need them: `(r as Right<E, T>).value`).
    result_types: HashMap<String, (String, String)>,
    /// Offset where each source line starts (`[call_site]` writes the line of a call).
    line_starts: Vec<usize>,
    /// Kind of the declaration being written (`[call_site]`).
    kind_name: String,
    /// Functions and methods being written, innermost last (`[call_site]`; lambdas keep their method).
    fn_names: Vec<String>,
}

/// Declared return type of a function or method, and whether it is async (`~name` or a `~call` inside).
fn sig_of(d: &Decl) -> Sig {
    let awaits = match &d.body {
        Body::Block(b) => analysis::awaits_block(&b.stmts),
        Body::Expr(e) => analysis::awaits_expr(e),
        _ => false,
    };
    Sig { ret: d.ret.clone(), is_async: d.is_async || awaits }
}

fn is_void(t: &Type) -> bool {
    matches!(&t.kind, TypeKind::Named { name, args } if name == "void" && args.is_empty())
}

/// `@nestjs/common` → `common`, `fs/promises` → `promises`, `material_ui` → `materialUi`.
fn default_name(path: &str) -> String {
    let last = path.rsplit(['/', ':']).next().unwrap_or(path);
    let mut out = String::new();
    let mut upper = false;
    for c in last.chars() {
        if c.is_alphanumeric() {
            if upper {
                out.extend(c.to_uppercase());
                upper = false;
            } else {
                out.push(c);
            }
        } else if !out.is_empty() {
            upper = true;
        }
    }
    out
}

/// Text with no spaces outside brackets and quotes behaves like a primary expression.
fn is_atomic(text: &str) -> bool {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for c in text.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                '\'' | '"' | '`' => quote = Some(c),
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                c if c.is_whitespace() && depth == 0 => return false,
                _ => {}
            },
        }
    }
    true
}

/// Emitted name of a declaration.
fn out_name_for_summary(src: &str, names: &HashMap<String, String>) -> String {
    names.get(src).cloned().unwrap_or_else(|| src.to_string())
}

/// `word` appears in `text` as a whole identifier.
fn contains_word(text: &str, word: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(ident) && !after.is_some_and(ident)
    })
}

fn indent_lines(text: &str, indent: &str) -> String {
    text.split('\n')
        .map(|l| if l.is_empty() { String::new() } else { format!("{indent}{l}") })
        .collect::<Vec<_>>()
        .join("\n")
}

impl<'a> Emitter<'a> {
    pub fn new(d: &'a Dialect, comments: Vec<Comment>) -> Self {
        Emitter {
            d,
            project: None,
            file_path: String::new(),
            used_names: HashSet::new(),
            default_kind: None,
            kinds_used: BTreeSet::new(),
            provided: HashMap::new(),
            checked_decorators: HashSet::new(),
            cur_kind: None,
            owner: String::new(),
            result_import: false,
            owner_decor: Value::Map(BTreeMap::new()),
            extra_decls: Vec::new(),
            last_method: None,
            type_params: Vec::new(),
            generic_names: HashSet::new(),
            summaries: Vec::new(),
            diags: Vec::new(),
            marks: Vec::new(),
            decls: HashMap::new(),
            json_classes: HashMap::new(),
            checked_platform: HashSet::new(),
            decl_names: HashMap::new(),
            imported: HashSet::new(),
            open_imports: false,
            inherited_params: HashSet::new(),
            super_params: HashSet::new(),
            cascade_self: Vec::new(),
            frames: Vec::new(),
            module: None,
            kind_syntax: None,
            refs: BTreeSet::new(),
            missing_reported: HashSet::new(),
            comments: comments.into_iter().filter(|c| c.own_line).collect(),
            next_comment: 0,
            result_builtin: true,
            uses_result: false,
            source: String::new(),
            callee_member: false,
            assign_target: false,
            helpers: BTreeMap::new(),
            lambda_hints: Vec::new(),
            fn_ctx: Vec::new(),
            line_starts: vec![0],
            kind_name: String::new(),
            fn_names: Vec::new(),
            pending: Vec::new(),
            hoist_blocked: 0,
            hoisted: Vec::new(),
            lambda_count: 0,
            lambda_names: String::new(),
            inner_doc: None,
            doc_summary: None,
            written_words: None,
            plugin_names: None,
            plugin_reported: HashSet::new(),
            ctor_inits: Vec::new(),
            keep_next: false,
            temp_hint: None,
            next_lambda_result: false,
            result_types: HashMap::new(),
        }
    }

    /// Project mode: the other files of the project, this file's path, the names it uses and the kind of
    /// its declarations without a kind word.
    pub(crate) fn in_project(
        mut self,
        project: &'a ProjectCtx,
        file_path: &str,
        used_names: HashSet<String>,
        default_kind: Option<String>,
    ) -> Self {
        self.project = Some(project);
        self.file_path = file_path.to_string();
        self.used_names = used_names;
        self.default_kind = default_kind;
        self
    }

    /// The source text: `[call_site]` writes the line of each call.
    pub(crate) fn with_source(mut self, src: &str) -> Self {
        self.line_starts = std::iter::once(0).chain(src.match_indices('\n').map(|(i, _)| i + 1)).collect();
        self.source = src.to_string();
        self
    }

    /// Only the first pass (`collect`): what other files of a project need to know about this one.
    pub(crate) fn declarations(&mut self, f: &File) -> Vec<(String, String, DeclInfo)> {
        self.collect(f);
        let mut out = Vec::new();
        for item in &f.items {
            if let Item::Decl(d) = item
                && let Some(o) = self.decl_names.get(&d.name.name)
                && let Some(info) = self.decls.get(o)
            {
                out.push((d.name.name.clone(), o.clone(), info.clone()));
            }
        }
        for extra in &self.extra_decls {
            if let Some(info) = self.decls.get(extra) {
                out.push((extra.clone(), extra.clone(), info.clone()));
            }
        }
        out
    }

    /// Component of this file: the folder under `[architecture] scope` (`components/users/...` → `users`).
    fn component(&self) -> String {
        let Some(scope) = &self.d.architecture.scope else { return String::new() };
        let prefix = format!("{scope}/");
        self.file_path.strip_prefix(&prefix).and_then(|r| r.split('/').next()).unwrap_or_default().to_string()
    }

    pub(crate) fn uses_result(&self) -> bool {
        self.uses_result
    }

    fn error(&mut self, code: Code, span: Span, msg: impl Into<String>) {
        self.diags.push(Diagnostic::error(code, span, msg));
    }

    // ---------- templates ----------

    fn find_template(&self, key: &str) -> Option<&'a Template> {
        if let Some(t) = self.kind_syntax.and_then(|m| m.get(key)) {
            return Some(t);
        }
        self.d.syntax(key)
    }

    fn has(&self, key: &str) -> bool {
        self.find_template(key).is_some()
    }

    fn tpl(&mut self, key: &str, ctx: Ctx, span: Span) -> String {
        // `[syntax] key = { template, helper }`: the helper is written once per file that uses the template
        if let Some(h) = self.d.syntax_helpers.get(key) {
            self.helpers.entry(format!("syntax.{key}")).or_insert_with(|| h.render(&Value::Map(BTreeMap::new())));
        }
        match self.find_template(key) {
            Some(t) => t.render(&ctx.into_value()),
            None => {
                if self.missing_reported.insert(key.to_string()) {
                    let chain = self.d.chain.join(" → ");
                    self.error(
                        codes::MISSING_TEMPLATE,
                        span,
                        format!("dialect '{}' ({chain}) has no [syntax] template '{key}'", self.d.name),
                    );
                }
                String::new()
            }
        }
    }

    fn plain(&mut self, key: &str, span: Span) -> String {
        self.tpl(key, Ctx::new(), span)
    }

    fn type_tpl(&mut self, key: &str, ctx: Ctx, span: Span) -> Option<String> {
        match self.d.types.get(key) {
            Some(e) => Some(e.value.render(&ctx.into_value())),
            None => {
                if self.missing_reported.insert(format!("types.{key}")) {
                    self.error(
                        codes::MISSING_TEMPLATE,
                        span,
                        format!("dialect '{}' has no [types] template '{key}'", self.d.name),
                    );
                }
                None
            }
        }
    }

    fn operator(&self, key: &str) -> Option<&'a Template> {
        self.d.operators.get(key).map(|e| &e.value)
    }

    // ---------- comments ----------

    fn comments_before(&mut self, pos: usize) -> String {
        let mut lines = Vec::new();
        while self.next_comment < self.comments.len() && self.comments[self.next_comment].span.end <= pos {
            let c = self.comments[self.next_comment].clone();
            self.next_comment += 1;
            for line in c.text.split('\n') {
                let l = self.tpl("comment", Ctx::new().set("text", line), c.span);
                lines.push(l);
            }
        }
        lines.join("\n")
    }

    /// Comments above a declaration. The run of own-line comments that ends on the line right above it is
    /// its doc comment (base §2.3): with a `doc_comment` template it is written as a doc comment of the target.
    fn doc_before(&mut self, pos: usize) -> String {
        let (plain, code) = self.doc_parts(pos);
        attach(plain, code.unwrap_or_default())
    }

    /// Like doc_before, but with options.doc_inside the doc comment goes to the template (`doc`, Python's
    /// docstring) of the function, method or class that follows instead of above it.
    fn doc_into(&mut self, pos: usize) -> String {
        let (plain, code) = self.doc_parts(pos);
        if self.d.options.doc_inside {
            self.inner_doc = code.filter(|c| !c.is_empty());
            return plain;
        }
        attach(plain, code.unwrap_or_default())
    }

    /// (comments before the doc comment, the doc comment rendered with `doc_comment`).
    fn doc_parts(&mut self, pos: usize) -> (String, Option<String>) {
        self.doc_summary = None;
        let Some(tpl) = self.find_template("doc_comment") else { return (self.comments_before(pos), None) };
        let mut end = self.next_comment;
        while end < self.comments.len() && self.comments[end].span.end <= pos {
            end += 1;
        }
        let line = |p: usize| self.line_starts.partition_point(|&s| s <= p);
        let (mut first, mut want) = (end, line(pos));
        while first > self.next_comment {
            let c = &self.comments[first - 1];
            if !c.own_line || line(c.span.end) + 1 != want {
                break;
            }
            want = line(c.span.start);
            first -= 1;
        }
        if first == end {
            let gap = end > self.next_comment && line(self.comments[end - 1].span.end) + 1 < want;
            let mut plain = self.comments_before(pos);
            if gap {
                plain.push('\n'); // the blank line that says "not a doc comment" stays
            }
            return (plain, None);
        }
        let doc_line = line(self.comments[first].span.start);
        let gap = self.next_comment < first && line(self.comments[first - 1].span.end) + 1 < doc_line;
        let mut plain = self.comments_before(self.comments[first].span.start);
        if gap {
            plain.push('\n'); // keeps the blank line between a loose comment and the doc comment
        }
        let text: Vec<&str> = self.comments[first..end].iter().map(|c| c.text.as_str()).collect();
        let doc = DocComment::parse(&text.join("\n"));
        self.doc_summary = Some(doc.summary.join(" ").trim().to_string()).filter(|s| !s.is_empty());
        self.next_comment = end;
        let strs = |v: &[String]| Value::List(v.iter().map(|s| Value::str(s.as_str())).collect());
        let pairs = |v: &[(String, String)], a: &str, b: &str| {
            Value::List(v.iter().map(|(x, y)| Ctx::new().set(a, x.as_str()).set(b, y.as_str()).into_value()).collect())
        };
        let examples = doc
            .examples
            .iter()
            .map(|(caption, code)| {
                let lines: Vec<String> = code.lines().map(str::to_string).collect();
                Ctx::new().set("caption", caption.as_str()).value("lines", strs(&lines)).into_value()
            })
            .collect();
        let ctx = Ctx::new()
            .value("summary", strs(&doc.summary))
            .value("params", pairs(&doc.params, "name", "text"))
            .opt("returns", doc.returns.clone())
            .value("throws", strs(&doc.throws))
            .value("examples", Value::List(examples))
            .value("see", strs(&doc.see))
            .flag("deprecated", doc.deprecated.is_some())
            .opt("deprecated_text", doc.deprecated.clone())
            .opt("deprecated_literal", doc.deprecated.clone().map(|t| self.string(&[StrPiece::Text(t)])))
            .flag("one_line", doc.summary.len() == 1 && !doc.has_tags())
            .opt("since", doc.since.clone())
            .value("other", pairs(&doc.other, "tag", "text"))
            .flag("has_tags", doc.has_tags() && !doc.summary.is_empty());
        let code = tpl.render(&ctx.into_value()).trim_end().to_string();
        (plain, Some(code))
    }

    fn with_doc(&mut self, pos: usize, code: String) -> String {
        let c = self.doc_before(pos);
        attach(c, code)
    }

    // ---------- names ----------

    fn escape(&self, name: &str) -> String {
        if self.d.is_reserved(name) {
            if self.d.options.reserved_suffix { format!("{name}_") } else { format!("_{name}") }
        } else {
            name.to_string()
        }
    }

    fn lookup_local(&self, name: &str) -> Option<String> {
        self.frames.iter().rev().find_map(|f| f.vars.get(name).cloned())
    }

    fn declare_local(&mut self, name: &str) -> String {
        let out = self.escape(name);
        if let Some(f) = self.frames.last_mut() {
            f.vars.insert(name.to_string(), out.clone());
        }
        out
    }

    fn is_member(&self, name: &str) -> bool {
        self.module.as_ref().is_some_and(|m| m.members.contains(name))
    }

    /// Access to a module member from inside the module.
    fn member_access(&mut self, name: &str, explicit: bool, span: Span) -> String {
        if let Some(m) = &self.module
            && m.ctor_params.contains(name)
            && let Some(t) = &m.param_access
        {
            return t.render(&Ctx::new().set("name", name).into_value());
        }
        if self.d.options.implicit_self && !explicit {
            return name.to_string();
        }
        self.tpl("self_member", Ctx::new().set("name", name), span)
    }

    fn known_symbol(&self, name: &str) -> bool {
        self.lookup_local(name).is_some()
            || self.is_member(name)
            || self.decl_names.contains_key(name)
            || self.decls.contains_key(name)
            || self.imported.contains(name)
            || self.d.globals.contains(name)
            || self.d.auto_imports.contains_key(name)
            || self.d.functions.contains_key(name)
            || self.project.is_some_and(|p| p.generated.contains_key(name))
    }

    // ---------- file ----------

    pub fn file(&mut self, f: &File) -> String {
        self.collect(f);
        let mut top = Frame::default();
        let top_stmts: Vec<Stmt> =
            f.items.iter().filter_map(|i| if let Item::Stmt(s) = i { Some(s.clone()) } else { None }).collect();
        top.mutable = analysis::reassigned_after(&top_stmts);
        self.frames.push(top);

        let mut uses = Vec::new();
        let mut decl_out: Vec<(bool, String)> = Vec::new(); // (is_statement, code)
        // top-level statements where the target runs them in main: (position in decl_out, code, span, the
        // variable it declares)
        let mut pending: Vec<(usize, String, Span, Option<String>)> = Vec::new();
        for item in &f.items {
            match item {
                Item::Use(u) => uses.push(u),
                Item::Decl(d) => {
                    // comments above a declaration are taken before its body can claim them
                    let pre = self.doc_into(d.span.start);
                    let code = self.top_decl(d);
                    self.inner_doc = None;
                    let code = attach(pre, mark(d.span.start, code));
                    decl_out.push((false, code));
                }
                Item::Stmt(s) => {
                    let declared = match s {
                        Stmt::Assign(a) if a.op == AssignOp::Set => match &a.target.kind {
                            ExprKind::Name(n) if self.frames.last().is_some_and(|f| !f.vars.contains_key(n)) => {
                                Some(n.clone())
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    let pre = self.comments_before(s.span().start);
                    let code = self.with_pending(|e| e.stmt(s));
                    let code = attach(pre, mark(s.span().start, code));
                    if self.d.options.toplevel_main {
                        pending.push((decl_out.len(), code, s.span(), declared));
                    } else {
                        decl_out.push((true, code));
                    }
                }
            }
        }
        // a variable that a declaration of the file uses stays top-level (`g = 9.8` used by a function →
        // `final g = 9.8;`); every other statement runs in main, in order
        let used: HashSet<String> = decl_out.iter().flat_map(|(_, c)| code_words(c)).collect();
        let mut main_stmts: Vec<String> = Vec::new();
        let mut first_main_stmt = None;
        for (at, code, span, declared) in pending.into_iter().rev() {
            if declared.is_some_and(|n| used.contains(&n)) && !code.contains("await ") {
                decl_out.insert(at, (true, code));
            } else {
                first_main_stmt = Some(span);
                main_stmts.insert(0, code);
            }
        }
        if !main_stmts.is_empty() {
            let has_main = f.items.iter().any(|i| matches!(i, Item::Decl(d) if d.name.name == "main"));
            if has_main {
                self.error(
                    codes::STATEMENTS_AND_MAIN,
                    first_main_stmt.unwrap_or_default(),
                    "this statement would run in a second main(): move it into main()",
                );
            }
            decl_out.push((false, self.tpl("main", Ctx::new().set("body", main_stmts.join("\n")), Span::default())));
        }
        let trailing = self.comments_before(usize::MAX);
        // names the generated code contains: in a project, the files to import (aliases and templates
        // write names the source does not) and the Result runtime; in any file, the names of
        // `[names] written` that templates write (`Right`, `Left`)
        let mut written = HashSet::new();
        for (_, c) in &decl_out {
            written.extend(code_words(c));
        }
        // the helpers of the library (`Callable`, `math`) need their imports too
        for h in self.helpers.values() {
            written.extend(code_words(h));
        }
        for w in &self.d.written {
            if written.contains(w) && !self.decl_names.contains_key(w) {
                self.refs.insert(w.clone());
            }
        }
        if self.project.is_some() {
            self.result_import = self.uses_result && ["Result", "Ok", "Err"].iter().any(|w| written.contains(*w));
            self.used_names.extend(written.iter().cloned());
        }
        self.written_words = Some(written.clone());
        let imports = self.imports(&uses);

        let mut out = String::new();
        if self.has("file_prelude") {
            out.push_str(self.plain("file_prelude", Span::default()).trim_end());
            let gap = if imports.is_empty() { self.d.options.toplevel_gap + 1 } else { 2 };
            out.push_str(&"\n".repeat(gap));
        }
        if !imports.is_empty() {
            out.push_str(&imports.join("\n"));
            out.push_str(&"\n".repeat(self.d.options.toplevel_gap + 1));
        }
        let file_stem = self.file_path.rsplit('/').next().unwrap_or_default().to_string();
        let project_name = self.project.map(|p| p.name.clone()).unwrap_or_default();
        for k in self.kinds_used.clone() {
            if let Some(h) = self.d.kinds.get(&k).and_then(|e| e.value.file_header.as_ref()) {
                let text = h.render(&Ctx::new().set("file", &file_stem).set("project", &project_name).into_value());
                if !text.trim().is_empty() {
                    out.push_str(text.trim_end());
                    out.push_str("\n\n");
                }
            }
        }
        // the dialect's own Result runtime; a target that uses a library (fpdart) has none
        if self.uses_result && self.project.is_none() && self.has("result_runtime") {
            let runtime = self.plain("result_runtime", Span::default());
            if !runtime.is_empty() {
                out.push_str(runtime.trim_end());
                out.push_str(&"\n".repeat(self.d.options.toplevel_gap + 1));
            }
        }
        // the helpers of the library members the file uses (`[library.*] helper`), once each
        for text in std::mem::take(&mut self.helpers).into_values() {
            if !text.trim().is_empty() {
                out.push_str(text.trim_end());
                out.push_str(&"\n".repeat(self.d.options.toplevel_gap + 1));
            }
        }
        let mut prev_stmt = None;
        for (is_stmt, code) in decl_out {
            if let Some(prev) = prev_stmt {
                let gap = "\n".repeat(self.d.options.toplevel_gap + 1);
                out.push_str(if prev && is_stmt { "\n" } else { &gap });
            }
            out.push_str(&code);
            prev_stmt = Some(is_stmt);
        }
        if !trailing.is_empty() {
            out.push_str("\n\n");
            out.push_str(&trailing);
        }
        let (code, marks) = unmark(&tidy(&out, self.d.options.toplevel_gap));
        self.marks = marks;
        code
    }

    /// First pass: every top-level name, so code can refer to declarations below it.
    fn collect(&mut self, f: &File) {
        for item in &f.items {
            match item {
                Item::Use(u) => {
                    self.imported.extend(u.names.iter().map(|n| n.name.clone()));
                    if let Some(a) = &u.alias {
                        self.imported.insert(a.name.clone());
                    }
                    if u.names.is_empty() && u.alias.is_none() {
                        self.open_imports = true;
                        if let UsePath::Module(p) = &u.path {
                            self.imported.insert(default_name(p));
                        }
                    }
                }
                Item::Decl(d) => {
                    let kind = self.classify(d);
                    let out_name = match &kind {
                        DeclKind::Kind(k) => match self.d.kinds.get(k).and_then(|e| e.value.rename.as_ref()) {
                            Some(r) => r.render(&Ctx::new().set("name", &d.name.name).into_value()),
                            None => d.name.name.clone(),
                        },
                        DeclKind::Function => self.escape(&d.name.name),
                        _ => d.name.name.clone(),
                    };
                    let mut methods = HashMap::new();
                    let mut method_sigs = HashMap::new();
                    let mut aliases = HashMap::new();
                    let mut field_types = HashMap::new();
                    let mut field_decor = HashMap::new();
                    let mut params = d.params.as_ref().map(|p| p.list.clone()).unwrap_or_default();
                    let mut member_names = HashSet::new();
                    if let Body::Members(members) = &d.body {
                        for m in members {
                            match m {
                                Member::Method(md) => {
                                    methods.insert(
                                        md.name.name.clone(),
                                        md.params.as_ref().map(|p| p.list.clone()).unwrap_or_default(),
                                    );
                                    let mut sig = sig_of(md);
                                    if let DeclKind::Kind(k) = &kind
                                        && self.d.kinds.get(k).is_some_and(|e| e.value.async_methods)
                                    {
                                        sig.is_async = true;
                                    }
                                    if md.decorators.iter().any(|x| self.d.void_decorators.contains(&x.name)) {
                                        sig.ret = None;
                                    }
                                    method_sigs.insert(md.name.name.clone(), sig);
                                    // a `get`/`set` pair shares its name
                                    let accessor = md.words.iter().any(|w| w.name == "get" || w.name == "set");
                                    if !member_names.insert(md.name.name.clone()) && !accessor {
                                        self.error(
                                            codes::DUPLICATE_DECL,
                                            md.name.span,
                                            format!("'{}' is declared twice in '{}'; each member is declared once", md.name.name, d.name.name),
                                        );
                                    }
                                    if let DeclKind::Kind(k) = &kind
                                        && let Some(kd) = self.d.kinds.get(k)
                                    {
                                        let ctx = Ctx::new().set("name", &md.name.name).into_value();
                                        member_names.extend(kd.value.method_members.iter().map(|t| t.render(&ctx)));
                                    }
                                    // `list() = ListTasks.exec`: resolved when its type is needed
                                    if md.ret.is_none()
                                        && md.params.as_ref().is_none_or(|p| p.list.is_empty())
                                        && let Body::Expr(Expr { kind: ExprKind::Member { object, name, optional: false }, .. }) = &md.body
                                        && let ExprKind::Name(m) = &object.kind
                                        && is_pascal(m)
                                    {
                                        aliases.insert(md.name.name.clone(), (m.clone(), name.name.clone()));
                                    }
                                }
                                Member::Field(f) => {
                                    if !member_names.insert(f.name.name.clone()) {
                                        self.error(
                                            codes::DUPLICATE_DECL,
                                            f.name.span,
                                            format!("'{}' is declared twice in '{}'; each member is declared once", f.name.name, d.name.name),
                                        );
                                    }
                                    if let Some(t) = &f.ty {
                                        field_types.insert(f.name.name.clone(), t.clone());
                                    }
                                    for x in &f.decorators {
                                        field_decor.entry(x.name.clone()).or_insert_with(|| f.name.name.clone());
                                    }
                                }
                                Member::Ctor(c) => params = c.params.list.clone(),
                                _ => {}
                            }
                        }
                    }
                    member_names.extend(params.iter().map(|p| p.name.name.clone()));
                    for p in &params {
                        if let Some(t) = &p.ty {
                            field_types.entry(p.name.name.clone()).or_insert_with(|| t.clone());
                        }
                    }
                    let mut bases: Vec<String> =
                        d.bases
                            .iter()
                            .chain(d.clauses.iter().flat_map(|c| c.types.iter()))
                            .filter_map(|t| {
                                if let TypeKind::Named { name, .. } = &t.kind { Some(name.clone()) } else { None }
                            })
                            .collect();
                    // the classes a kind's template extends (`Repository` of a repository) are bases too
                    if let DeclKind::Kind(k) = &kind
                        && let Some(kd) = self.d.kinds.get(k)
                    {
                        bases.extend(kd.value.inherits.iter().cloned());
                    }
                    if let DeclKind::Kind(k) = &kind
                        && self.d.kinds.get(k).is_some_and(|e| e.value.named_params)
                    {
                        for p in &mut params {
                            p.named = true;
                        }
                    }
                    if self.decls.contains_key(&out_name) {
                        self.error(
                            codes::DUPLICATE_DECL,
                            d.name.span,
                            format!("'{out_name}' is declared twice in this file"),
                        );
                    }
                    // classes the kind's template writes next to it (`CounterStateBuilder`)
                    if let DeclKind::Kind(k) = &kind
                        && let Some(kd) = self.d.kinds.get(k)
                    {
                        for t in &kd.value.declares {
                            let extra = t.render(&Ctx::new().set("name", &out_name).into_value());
                            self.decl_names.insert(extra.clone(), extra.clone());
                            self.extra_decls.push(extra.clone());
                            self.decls.insert(
                                extra,
                                DeclInfo {
                                    kind: DeclKind::Class,
                                    params: Vec::new(),
                                    methods: HashMap::new(),
                                    members: HashSet::new(),
                                    bases: Vec::new(),
                                    sig: Sig::default(),
                                    method_sigs: HashMap::new(),
                                    field_types: HashMap::new(),
                                    field_decor: HashMap::new(),
                                    decor: Value::Map(BTreeMap::new()),
                                    aliases: HashMap::new(),
                                },
                            );
                        }
                    }
                    self.decl_names.insert(d.name.name.clone(), out_name.clone());
                    let sig = sig_of(d);
                    let (decl_decor, _) = self.decor(&d.decorators);
                    self.decls.insert(
                        out_name,
                        DeclInfo {
                            kind,
                            params,
                            methods,
                            members: member_names,
                            bases,
                            sig,
                            method_sigs,
                            field_types,
                            field_decor,
                            decor: decl_decor,
                            aliases,
                        },
                    );
                }
                Item::Stmt(_) => {}
            }
        }
        self.result_builtin = !["Result", "Ok", "Err"].iter().any(|n| self.decl_names.contains_key(*n));
        // declarations of the other files of the project
        if let Some(p) = self.project {
            for (name, ext) in &p.decls {
                if !self.decl_names.contains_key(name) {
                    self.decl_names.insert(name.clone(), ext.out_name.clone());
                    self.decls.entry(ext.out_name.clone()).or_insert_with(|| ext.info.clone());
                }
            }
        }
    }

    /// Kind, interface, enum, class or function, from the header words and the shape of the body.
    fn classify(&self, d: &Decl) -> DeclKind {
        if let Some(w) = d.words.iter().rev().find(|w| self.d.kinds.contains_key(&w.name)) {
            return DeclKind::Kind(w.name.clone());
        }
        let shape = self.classify_shape(d);
        match (&shape, &self.default_kind) {
            (DeclKind::Class | DeclKind::Interface, Some(k)) if d.words.is_empty() && self.d.kinds.contains_key(k) => {
                DeclKind::Kind(k.clone())
            }
            _ => shape,
        }
    }

    /// What the declaration is by its shape alone (no kind word).
    fn classify_shape(&self, d: &Decl) -> DeclKind {
        if !d.name.is_pascal() {
            return DeclKind::Function;
        }
        let Body::Members(members) = &d.body else {
            return DeclKind::Class;
        };
        if !members.is_empty() && members.iter().all(|m| matches!(m, Member::EnumItem(_))) {
            return DeclKind::Enum;
        }
        let signatures_only = d.params.is_none()
            && !members.is_empty()
            && members.iter().all(|m| match m {
                Member::Field(f) => f.default.is_none(),
                Member::Method(md) => matches!(md.body, Body::None),
                _ => false,
            });
        let abstract_word = d.words.iter().any(|w| w.name == "abstract");
        if signatures_only && !abstract_word { DeclKind::Interface } else { DeclKind::Class }
    }

    /// An interface of this file, or an external `interface class` of the type definitions.
    fn is_interface(&self, name: &str) -> bool {
        self.decls.get(name).map_or_else(
            || self.d.defs.get(name).is_some_and(|c| c.kind.as_deref() == Some("interface")),
            |i| i.kind == DeclKind::Interface,
        )
    }

    // ---------- imports ----------

    fn imports(&mut self, uses: &[&Use]) -> Vec<String> {
        // module → names, in first-seen order
        let mut named: Vec<(String, BTreeSet<String>)> = Vec::new();
        let add_named = |named: &mut Vec<(String, BTreeSet<String>)>, module: &str, names: &[String]| match named
            .iter_mut()
            .find(|(m, _)| m == module)
        {
            Some((_, set)) => set.extend(names.iter().cloned()),
            None => named.push((module.to_string(), names.iter().cloned().collect())),
        };
        let mut lines: Vec<String> = Vec::new();
        let mut seen_modules: HashSet<String> = HashSet::new();
        for u in uses {
            for c in &u.clauses {
                if !self.d.use_clauses.contains(&c.word.name) {
                    let hint = closest(&c.word.name, self.d.use_clauses.iter().map(String::as_str))
                        .map(|s| format!("; did you mean '{s}'?"))
                        .unwrap_or_default();
                    self.error(
                        codes::UNKNOWN_USE_CLAUSE,
                        c.word.span,
                        format!("'{}' is not a use clause of dialect '{}'{hint}", c.word.name, self.d.name),
                    );
                }
            }
            let clauses = Value::Map(
                u.clauses
                    .iter()
                    .map(|c| {
                        let v = if c.names.is_empty() {
                            Value::Bool(true)
                        } else {
                            Value::str(c.names.iter().map(|n| n.name.clone()).collect::<Vec<_>>().join(", "))
                        };
                        (c.word.name.clone(), v)
                    })
                    .collect(),
            );
            let module = match &u.path {
                UsePath::File(p) => {
                    let path = self.tpl("file_path", Ctx::new().set("path", self.sep(p)), u.span);
                    lines.push(self.tpl("import_file", Ctx::new().set("path", path).value("clauses", clauses), u.span));
                    continue;
                }
                UsePath::Module(p) => match self.d.imports.get(p).map(|e| e.value.clone()) {
                    Some(ImportShortcut::Line(l)) => {
                        lines.push(l.trim_end().to_string());
                        continue;
                    }
                    Some(ImportShortcut::Module(m)) => m,
                    None => self.tpl("module_path", Ctx::new().set("path", self.sep(p)), u.span),
                },
            };
            seen_modules.insert(module.clone());
            let names: Vec<String> = u.names.iter().map(|n| n.name.clone()).collect();
            if let Some(alias) = &u.alias {
                let key = if self.has("import_alias") { "import_alias" } else { "import_default" };
                let ctx = Ctx::new()
                    .set("module", &module)
                    .set("alias", &alias.name)
                    .set("default_name", &alias.name)
                    .value("clauses", clauses);
                lines.push(self.tpl(key, ctx, u.span));
            } else if !names.is_empty() && u.clauses.is_empty() && self.has("import_names") {
                add_named(&mut named, &module, &names);
            } else {
                let ctx = Ctx::new()
                    .set("module", &module)
                    .set("default_name", default_name(&module))
                    .set("names", names.join(", "))
                    .value("clauses", clauses);
                lines.push(self.tpl("import_default", ctx, u.span));
            }
        }
        // auto-imports: names used but not declared anywhere
        let mut auto: BTreeMap<String, Vec<String>> = BTreeMap::new();
        // imports for the type checker only (`[kinds.x] type_imports`), written after the others
        let mut type_lines: Vec<String> = Vec::new();
        let mut type_block: Option<String> = None;
        for r in &self.refs {
            if self.decl_names.contains_key(r) || self.decls.contains_key(r) || self.imported.contains(r) {
                continue;
            }
            // a name the generated code never writes (a type rendered only to compare) needs no import
            if self.written_words.as_ref().is_some_and(|w| !w.contains(r)) {
                continue;
            }
            if let Some(e) = self.d.auto_imports.get(r) {
                auto.entry(e.value.clone()).or_default().push(r.clone());
            }
        }
        // other files of the project, and the project's Result runtime
        if let Some(p) = self.project {
            let mut paths: BTreeSet<String> = BTreeSet::new();
            let mut type_paths: BTreeSet<String> = BTreeSet::new();
            let mut names_by_path: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let declared_here: HashSet<&str> = self
                .summaries
                .iter()
                .map(|s| s.name.as_str())
                .chain(self.extra_decls.iter().map(String::as_str))
                .collect();
            for n in &self.used_names {
                if declared_here.contains(n.as_str()) {
                    continue;
                }
                let path = match p.decls.get(n) {
                    Some(ext) if ext.path != self.file_path && p.visible(&ext.path, &self.file_path) => {
                        Some(ext.path.clone())
                    }
                    Some(_) => None,
                    None => p.generated.get(n).cloned(),
                };
                // a declaration of a kind with `type_imports` used by one of the same kind: a type-only import
                let type_only = p.decls.get(n).is_some_and(|ext| match &ext.info.kind {
                    DeclKind::Kind(k) => {
                        self.kinds_used.contains(k)
                            && self.d.kinds.get(k).is_some_and(|e| e.value.type_imports)
                            && self.has("type_imports")
                    }
                    _ => false,
                });
                if let Some(path) = path {
                    if type_only {
                        type_paths.insert(path.clone());
                    }
                    paths.insert(path.clone());
                    names_by_path.entry(path).or_default().push(n.clone());
                }
            }
            if self.result_import && self.result_builtin && self.has("result_runtime") {
                paths.insert(RESULT_PATH.to_string());
                names_by_path
                    .entry(RESULT_PATH.to_string())
                    .or_default()
                    .extend(["Result", "Ok", "Err"].map(String::from));
            }
            for path in paths {
                let relative = p.relative(&p.out_stem(&self.file_path), &path);
                let ctx = Ctx::new().set("project", &p.name).set("path", self.sep(&path)).set("relative", relative);
                let module = self.tpl("project_import", ctx, Span::default());
                let mut names = names_by_path.remove(&path).unwrap_or_default();
                names.sort();
                if type_paths.contains(&path) {
                    let ctx = Ctx::new()
                        .set("module", &module)
                        .set("names", names.join(", "))
                        .value("clauses", Value::Map(BTreeMap::new()));
                    type_lines.push(self.tpl("import_names", ctx, Span::default()));
                } else {
                    auto.entry(module).or_default().extend(names);
                }
            }
            if !type_lines.is_empty() {
                let indent = self.d.indent.clone();
                let lines = type_lines.iter().map(|l| format!("{indent}{l}")).collect::<Vec<_>>().join("\n");
                let block = self.tpl("type_imports", Ctx::new().set("imports", lines), Span::default());
                // the names the block itself writes (`TYPE_CHECKING`) are imported like any other
                let own = block.lines().next().unwrap_or_default().to_string();
                for w in code_words(&own) {
                    if let Some(e) = self.d.auto_imports.get(&w) {
                        auto.entry(e.value.clone()).or_default().push(w.clone());
                    }
                }
                type_block = Some(block);
            }
        }
        // a module that another needed module re-exports is not imported (material.dart includes widgets.dart)
        let needed: Vec<String> = auto.keys().chain(seen_modules.iter()).cloned().collect();
        let mut dropped: BTreeSet<String> = BTreeSet::new();
        for m in auto.keys() {
            let covered = needed
                .iter()
                .any(|n| n != m && !dropped.contains(n) && self.d.reexports.get(n).is_some_and(|r| r.contains(m)));
            if covered {
                dropped.insert(m.clone());
            }
        }
        auto.retain(|m, _| !dropped.contains(m));
        let use_names = self.has("import_names");
        let mut auto_lines = Vec::new();
        for (module, names) in auto {
            // a module imported as itself (`"math" = ["math"]`: Python's `import math`)
            let as_module = names.len() == 1 && names.first().is_some_and(|n| *n == module);
            if use_names && !as_module {
                add_named(&mut named, &module, &names);
            } else if seen_modules.insert(module.clone()) {
                // a name the project declares that the library also exports (`Task` of fpdart) is hidden
                let hide = self.hidden_from(&module);
                let mut clauses = BTreeMap::new();
                if !hide.is_empty() {
                    clauses.insert("hide".to_string(), Value::str(hide.join(", ")));
                }
                let ctx = Ctx::new()
                    .set("module", &module)
                    .set("default_name", default_name(&module))
                    .value("clauses", Value::Map(clauses));
                auto_lines.push(self.tpl("import_default", ctx, Span::default()));
            }
        }
        let mut out = Vec::new();
        for (module, names) in named {
            let list: Vec<String> = names.into_iter().collect();
            let ctx = Ctx::new()
                .set("module", &module)
                .set("names", list.join(", "))
                .value("clauses", Value::Map(BTreeMap::new()));
            out.push(self.tpl("import_names", ctx, Span::default()));
        }
        out.extend(lines);
        out.extend(auto_lines);
        out.extend(type_block);
        out
    }

    /// N0116: a member of a value typed with a closed declaration of the project must exist. Closed: every member is
    /// known — its own, those of its project bases, and those of the runtime classes it or its kind inherits when the
    /// type definitions list them (`repository` → `Repository`, `Query`); nothing the kind's template declares. A model
    /// generated from a contract that the backend changed fails here, with the field's name.
    fn check_project_member(&mut self, object: &Expr, name: &Ident) {
        let Some(t) = self.static_type(object) else { return };
        let TypeKind::Named { name: tn, .. } = &strip_nullable(t).kind else { return };
        let Some(info) = self.decl_info(tn) else { return };
        if !matches!(info.kind, DeclKind::Class | DeclKind::Kind(_)) || info.members.contains(&name.name) {
            return;
        }
        let Some(all) = self.closed_members(tn) else { return };
        if all.contains(&name.name) {
            return;
        }
        // a dialect that renames to snake_case shows the runtime's members as the `.nuc` writes them
        let shown = |m: &str| if self.d.options.snake_case_names { crate::naming::camel(m) } else { m.to_string() };
        let mut members: Vec<String> = all.iter().map(|m| shown(m)).collect();
        members.sort();
        members.dedup();
        let wanted = shown(&name.name);
        let hint = closest(&wanted, members.iter().map(|m| m.as_str())).map(|c| format!("did you mean '{c}'? ")).unwrap_or_default();
        let msg = format!("'{tn}' has no member '{wanted}'; {hint}its members: {}", members.join(", "));
        self.error(codes::UNKNOWN_MEMBER, name.span, msg);
    }

    /// Every member of `name` and its bases, when all of them are known (project declarations, or classes of the type
    /// definitions with their supers); `None` as soon as one is not, or a kind's template declares members.
    fn closed_members(&self, name: &str) -> Option<HashSet<String>> {
        let mut out = HashSet::new();
        let mut pending = vec![name.to_string()];
        let mut seen = HashSet::new();
        while let Some(n) = pending.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(info) = self.decl_info(&n) {
                if let DeclKind::Kind(k) = &info.kind
                    && self.d.kinds.get(k).is_some_and(|e| !e.value.declares.is_empty())
                {
                    return None;
                }
                out.extend(info.members.iter().cloned());
                pending.extend(info.bases.iter().cloned());
            } else {
                let c = self.d.defs.get(&n)?;
                out.extend(c.members.iter().cloned());
                pending.extend(c.supers.iter().cloned());
            }
        }
        Some(out)
    }

    /// The names the project declares that `module` exports too: a dialect with a `hide` clause keeps the project's.
    fn hidden_from(&self, module: &str) -> Vec<String> {
        let has_hide = self
            .find_template("import_default")
            .is_some_and(|t| t.used_vars().into_iter().any(|v| v.name == "clauses"));
        if !has_hide {
            return Vec::new();
        }
        let mut names: BTreeSet<String> = self.decls.keys().cloned().collect();
        if let Some(p) = self.project {
            names.extend(p.decls.keys().cloned());
        }
        let exported = self.d.exports.get(module);
        names
            .into_iter()
            .filter(|n| {
                exported.is_some_and(|e| e.contains(n))
                    || self.d.def_libraries.get(n).is_some_and(|libs| libs.iter().any(|l| l == module))
            })
            .collect()
    }

    // ---------- declarations ----------

    fn export(&mut self) -> String {
        if self.has("export") { self.plain("export", Span::default()) } else { String::new() }
    }

    /// `<T, U: Comparable>`: each generic rendered with `generic_param` (name and bound).
    fn generics(&mut self, g: &[Ident], bounds: &[Option<Type>], span: Span) -> String {
        if g.is_empty() {
            return String::new();
        }
        self.generic_names.extend(g.iter().map(|n| n.name.clone()));
        let mut names = Vec::new();
        for (i, name) in g.iter().enumerate() {
            let bound = bounds.get(i).cloned().flatten().map(|t| self.ty(&t));
            names.push(self.tpl("generic_param", Ctx::new().set("name", &name.name).opt("bound", bound), span));
        }
        self.tpl("generics", Ctx::new().set("names", names.join(", ")), span)
    }

    fn decorators(&mut self, list: &[Decorator], inline: bool) -> String {
        let mut out = String::new();
        for d in list {
            if self.d.consumed_decorators.contains(&d.name) {
                continue;
            }
            let root = d.name.split('.').next().unwrap_or(&d.name).to_string();
            if self.lookup_local(&root).is_none()
                && !self.decl_names.contains_key(&root)
                && !self.imported.contains(&root)
            {
                self.check_plugin_name(&root, d.span);
                self.refs.insert(root);
            }
            let args = match &d.args {
                Some(a) => self.args(a, None, d.span, d.name.len() + 1),
                None => String::new(),
            };
            let has_args = d.args.is_some();
            let pascal = d.name.starts_with(|c: char| c.is_ascii_uppercase());
            let text = self.tpl(
                "decorator",
                Ctx::new().set("name", &d.name).set("args", args).flag("has_args", has_args).flag("pascal", pascal),
                d.span,
            );
            out.push_str(&text);
            out.push(if inline { ' ' } else { '\n' });
        }
        out
    }

    /// Decorators as data for templates: `decor` maps each name to its arguments (`arg` = first positional,
    /// `args` = all, `items` = elements of a list argument, plus each named argument) and `deco_args`
    /// merges the named arguments of all of them.
    fn decor(&mut self, list: &[Decorator]) -> (Value, Value) {
        if list.is_empty() {
            return (Value::Map(BTreeMap::new()), Value::Map(BTreeMap::new()));
        }
        // names in decorator arguments are written as they are, not as provided objects
        let provided = std::mem::take(&mut self.provided);
        let out = self.decor_inner(list);
        self.provided = provided;
        out
    }

    /// The value of a decorator argument for templates: its code, and for a list (`items`) or an object
    /// (each field) also its parts, so `{{decor.screen.loggedIn.forward}}` and
    /// `{{#decor.screen.providers.items}}` work; `{{decor.screen.providers}}` is still the code.
    fn decor_value(&mut self, e: &Expr) -> (String, Value) {
        let text = self.expr(e).0;
        let parts = match &e.kind {
            ExprKind::List(items) => {
                let items: Vec<Value> = items.iter().map(|i| Value::str(self.expr(i).0)).collect();
                BTreeMap::from([("items".to_string(), Value::List(items))])
            }
            ExprKind::Object(entries) => {
                let mut m = BTreeMap::new();
                for entry in entries {
                    if let ObjEntry::Pair { key, value } = entry {
                        let key = match key {
                            ObjKey::Ident(i) => i.name.clone(),
                            ObjKey::Str(s) => s.clone(),
                        };
                        m.insert(key, self.decor_value(value).1);
                    }
                }
                m
            }
            _ => return (text.clone(), Value::str(text)),
        };
        let mut parts = parts;
        parts.insert(".".into(), Value::str(&text));
        (text, Value::Map(parts))
    }

    /// `[decorators.args]`: every named argument (and field of an object argument) must be listed, and
    /// the required ones (`!`) must be there (N0217).
    /// N0308: the rules of a field's decorators (`[decorators.fields]`, the relations of the backend platform).
    fn check_field_rules(&mut self, f: &Field, fields: &HashSet<String>) {
        for dec in &f.decorators {
            let Some(rule) = self.d.decorator_fields.get(&dec.name).cloned() else { continue };
            let at = format!("@{} {}", dec.name, f.name.name);
            let mut errs = Vec::new();
            let mut ty = f.ty.clone();
            if let Some(Type { kind: TypeKind::Nullable(x), .. }) = &ty {
                ty = Some((**x).clone());
            }
            let is_list = matches!(&ty, Some(Type { kind: TypeKind::List(_), .. }));
            if let Some(Type { kind: TypeKind::List(x), .. }) = &ty {
                ty = Some((**x).clone());
            }
            let item = match &ty {
                Some(Type { kind: TypeKind::Named { name, .. }, .. }) => name.clone(),
                _ => String::new(),
            };
            let info = self.decl_info(&item).cloned();
            if rule.optional && !f.optional {
                errs.push(format!("{at}: the field is optional (it is there only when loaded): `{}?: ...`", f.name.name));
            }
            match rule.list {
                Some(true) if !is_list => errs.push(format!("{at}: the field is a list: `{}?: {item}[]`", f.name.name)),
                Some(false) if is_list => errs.push(format!("{at}: the field is one value, not a list")),
                _ => {}
            }
            if let Some(t) = &rule.target {
                let ok = info.as_ref().is_some_and(|i| matches!(&i.decor, Value::Map(m) if m.contains_key(t)));
                if !ok {
                    errs.push(format!("{at}: the field's type is a declaration with @{t} (`{item}` is not)"));
                }
            }
            if let Some(sib) = &rule.sibling {
                let want = sib.replace("{name}", &f.name.name);
                if !fields.iter().any(|x| same_name(x, &want)) {
                    errs.push(format!("{at}: add the field `{want}` (its column: `{want}: int`, or `{want}?: int`)"));
                }
            }
            // the value of the argument that names a field of the related declaration (`inverse = "tasks"`)
            let inverse = rule.inverse.as_ref().and_then(|arg| {
                let a = dec.args.as_ref()?.iter().find(|a| a.name.as_ref().is_some_and(|n| same_name(&n.name, arg)))?;
                match &a.value.kind {
                    ExprKind::Str(p) => match p.as_slice() {
                        [StrPiece::Text(t)] => Some((arg.clone(), t.clone())),
                        _ => None,
                    },
                    _ => None,
                }
            });
            if let (Some((arg, v)), Some(i)) = (&inverse, &info)
                && !i.field_types.keys().any(|k| same_name(k, v))
            {
                errs.push(format!("{at}: {arg} = \"{v}\" names no field of {item}"));
            }
            if let Some(w) = &rule.with {
                let here = f.decorators.iter().any(|d| &d.name == w);
                let there = inverse
                    .as_ref()
                    .zip(info.as_ref())
                    .is_some_and(|((_, v), i)| i.field_decor.get(w).is_some_and(|x| same_name(x, v)));
                if !here && !there {
                    let other = inverse.as_ref().map_or(format!("the field of {item}"), |(_, v)| format!("`{item}.{v}`"));
                    errs.push(format!("{at}: one side of the relation has @{w}: this field or {other}"));
                }
            }
            for e in errs {
                self.error(codes::FIELD_RULE, dec.span, e);
            }
        }
    }

    fn check_decorator_args(&mut self, d: &Decorator) {
        let Some(allowed) = self.d.decorator_args.get(&d.name).cloned() else { return };
        if !self.checked_decorators.insert(d.span.start) {
            return;
        }
        let names: Vec<&str> = allowed.iter().map(|a| a.trim_end_matches('!')).collect();
        let args = d.args.clone().unwrap_or_default();
        let given: Vec<&str> = args.iter().filter_map(|a| a.name.as_ref().map(|n| n.name.as_str())).collect();
        let has = |list: &[&str], n: &str| list.iter().any(|x| same_name(x, n));
        for a in &args {
            let Some(n) = &a.name else {
                let msg = format!("@{} takes only named arguments ({})", d.name, names.join(", "));
                self.error(codes::DECORATOR_ARGS, a.value.span, msg);
                continue;
            };
            if !has(&names, &n.name) {
                let top: Vec<&str> = names.iter().filter(|x| !x.contains('.')).copied().collect();
                let msg = format!("@{} has no argument '{}' (it has {})", d.name, n.name, top.join(", "));
                self.error(codes::DECORATOR_ARGS, n.span, msg);
                continue;
            }
            let declared = names.iter().find(|x| same_name(x, &n.name)).copied().unwrap_or(n.name.as_str());
            let prefix = format!("{declared}.");
            let fields: Vec<&str> = names.iter().filter_map(|x| x.strip_prefix(prefix.as_str())).collect();
            if fields.is_empty() {
                continue;
            }
            let ExprKind::Object(entries) = &a.value.kind else {
                let msg = format!("{} of @{} is an object: {{ {}: ... }}", n.name, d.name, fields.join(": ..., "));
                self.error(codes::DECORATOR_ARGS, a.value.span, msg);
                continue;
            };
            let mut present = Vec::new();
            for e in entries {
                if let ObjEntry::Pair { key: ObjKey::Ident(k), .. } = e {
                    present.push(k.name.clone());
                    if !has(&fields, &k.name) {
                        let msg = format!(
                            "{} of @{} has no field '{}' (it has {})",
                            n.name,
                            d.name,
                            k.name,
                            fields.join(", ")
                        );
                        self.error(codes::DECORATOR_ARGS, k.span, msg);
                    }
                }
            }
            for req in allowed.iter().filter_map(|x| x.strip_prefix(prefix.as_str())?.strip_suffix('!')) {
                if !present.iter().any(|p| same_name(p, req)) {
                    let msg = format!("{} of @{} needs the field '{req}'", n.name, d.name);
                    self.error(codes::DECORATOR_ARGS, a.value.span, msg);
                }
            }
        }
        for req in allowed.iter().filter_map(|x| x.strip_suffix('!')).filter(|x| !x.contains('.')) {
            if !has(&given, req) {
                self.error(codes::DECORATOR_ARGS, d.span, format!("@{} needs the argument '{req}'", d.name));
            }
        }
    }

    /// N0309: in a platform's dialect, a decorator in lower case is one the platform declares.
    fn check_platform_decorator(&mut self, d: &Decorator) {
        let known = &self.d.platform_decorators;
        if known.is_empty()
            || known.contains(&d.name)
            || self.d.consumed_decorators.contains(&d.name)
            || d.name == "override"
            || !d.name.starts_with(|c: char| c.is_ascii_lowercase())
            || !self.checked_platform.insert(d.span.start)
        {
            return;
        }
        let platform = self.d.platform.clone().unwrap_or_default();
        let msg = match self.d.replaced_decorators.get(&d.name) {
            Some(by) => format!("@{} is no longer a decorator of the {platform} platform: write {by}", d.name),
            None => {
                let hint = closest(&d.name, known.iter().map(String::as_str))
                    .map(|c| format!("did you mean @{c}? "))
                    .unwrap_or_default();
                format!("@{} is not a decorator of the {platform} platform; {hint}(nuclear help {platform})", d.name)
            }
        };
        self.error(codes::PLATFORM_DECORATOR, d.span, msg);
    }

    fn decor_inner(&mut self, list: &[Decorator]) -> (Value, Value) {
        let mut decor = BTreeMap::new();
        let mut merged = BTreeMap::new();
        for d in list {
            self.check_decorator_args(d);
            self.check_platform_decorator(d);
            let mut m = BTreeMap::new();
            let mut path = None;
            if let Some(args) = &d.args {
                let mut all = Vec::new();
                for a in args {
                    match &a.name {
                        Some(n) => {
                            let (text, value) = self.decor_value(&a.value);
                            // as the target writes a named argument (Dart `a: 1`, Python `a=1`)
                            let arg = if self.d.options.named_args_native {
                                self.tpl("named_arg", Ctx::new().set("name", &n.name).set("value", &text), a.span)
                            } else {
                                format!("{}: {text}", n.name)
                            };
                            all.push(arg);
                            m.insert(n.name.clone(), value.clone());
                            merged.insert(n.name.clone(), value);
                        }
                        None => {
                            let text = self.expr(&a.value).0;
                            if !m.contains_key("arg") {
                                m.insert("arg".into(), Value::str(&text));
                                if let ExprKind::List(items) = &a.value.kind {
                                    let items: Vec<Value> = items.iter().map(|i| Value::str(self.expr(i).0)).collect();
                                    m.insert("items".into(), Value::List(items));
                                }
                                // `path`: a route (`"tasks/{id}"`) with its parameters as the target writes them
                                if let ExprKind::Str(pieces) = &a.value.kind
                                    && let Some(p) = route_path(pieces, &self.d.options.route_param)
                                {
                                    path = Some(p);
                                }
                            }
                            all.push(text);
                        }
                    }
                }
                m.insert("args".into(), Value::str(all.join(", ")));
                if let Some(p) = path.take() {
                    m.entry("path".into()).or_insert(Value::str(p));
                }
            }
            decor.insert(d.name.clone(), Value::Map(m));
        }
        (Value::Map(decor), Value::Map(merged))
    }

    /// What templates need to know about a type to generate code for it: `type` (rendered), `list`,
    /// `item` (element type of a list, or the type), `model` (the item is a class of the program),
    /// `enum` (an enum of the program), `primitive`, `void`, `is.<name>` (source name of the item: `is.boolean`),
    /// `fields_decor.<decorator>` (a model's first field with that decorator: `fields_decor.primary`), and for
    /// `Result<E, T>`: `result`, `err`, with the rest describing T.
    fn type_info(&mut self, t: Option<&Type>) -> Value {
        let mut m = BTreeMap::new();
        let Some(t) = t else {
            m.insert("void".into(), Value::Bool(true));
            return Value::Map(m);
        };
        let mut t = t.clone();
        if let Some(r) = self.result_type(&t) {
            m.insert("result".into(), Value::Bool(true));
            m.insert("full".into(), Value::str(self.ty(&t)));
            if let Some(e) = &r.err {
                m.insert("err".into(), Value::str(self.ty(e)));
                if let TypeKind::Named { name, .. } = &e.kind {
                    m.insert("err_is".into(), Value::Map(BTreeMap::from([(name.clone(), Value::Bool(true))])));
                }
            }
            match r.ok {
                Some(ok) => t = ok,
                None => return Value::Map(m),
            }
        }
        if is_void(&t) {
            m.insert("void".into(), Value::Bool(true));
        }
        m.insert("type".into(), Value::str(self.ty(&t)));
        let mut inner = t.clone();
        if let TypeKind::Nullable(x) = &inner.kind {
            m.insert("nullable".into(), Value::Bool(true));
            inner = (**x).clone();
        }
        if let TypeKind::List(x) = &inner.kind {
            m.insert("list".into(), Value::Bool(true));
            inner = (**x).clone();
        }
        m.insert("item".into(), Value::str(self.ty(&inner)));
        if let TypeKind::Named { name, args } = &inner.kind {
            m.insert("name".into(), Value::str(name));
            if self.type_params.contains(name) {
                m.insert("generic".into(), Value::Bool(true));
            }
            if !args.is_empty() {
                let args = args.iter().map(|a| self.type_info(Some(a))).collect();
                m.insert("args".into(), Value::List(args));
            }
            const PRIMITIVES: &[&str] =
                &["string", "int", "double", "number", "boolean", "bool", "dynamic", "any", "void"];
            let primitive = PRIMITIVES.contains(&name.as_str());
            m.insert("primitive".into(), Value::Bool(primitive));
            // a declaration of the project, or a class of the runtime that reads itself from JSON (`Page`)
            let target = (!self.decl_names.contains_key(name.as_str()))
                .then(|| self.d.types.get(name.as_str()).map(|e| e.value.render(&Value::Map(BTreeMap::new()))))
                .flatten();
            let reads_json = !primitive && self.reads_json(target.as_deref().unwrap_or(name));
            let declared = self.decl_info(name).is_some_and(|i| matches!(i.kind, DeclKind::Class | DeclKind::Kind(_)));
            let model = !primitive && (reads_json || declared);
            // a class of the runtime, not of the project (no sample JSON of it in the generated tests)
            m.insert("runtime".into(), Value::Bool(model && !declared));
            m.insert("model".into(), Value::Bool(model));
            if model && let Some(info) = self.decl_info(name) {
                let fields_decor = info.field_decor.iter().map(|(k, v)| (k.clone(), Value::str(v))).collect();
                m.insert("fields_decor".into(), Value::Map(fields_decor));
                m.insert("decor".into(), info.decor.clone());
            }
            let is_enum = !primitive && self.decl_info(name).is_some_and(|i| matches!(i.kind, DeclKind::Enum));
            m.insert("enum".into(), Value::Bool(is_enum));
            m.insert("is".into(), Value::Map(BTreeMap::from([(name.clone(), Value::Bool(true))])));
        }
        Value::Map(m)
    }

    /// A class of the type definitions with a `fromJson` constructor or member (the runtime's `PageOf`), once per name.
    fn reads_json(&mut self, class: &str) -> bool {
        if !self.d.defs.contains_key(class) {
            return false;
        }
        if let Some(known) = self.json_classes.get(class) {
            return *known;
        }
        let found = self.d.typedefs.declarations().any(|(_, d)| {
            d.name == class
                && (d.constructors.iter().any(|c| c.name == "fromJson") || d.members.iter().any(|m| m.name == "fromJson"))
        });
        self.json_classes.insert(class.to_string(), found);
        found
    }

    /// N0218: a kind whose template never writes the declaration's generics or interfaces would drop them.
    fn check_kind_writes(&mut self, d: &Decl, kind_name: &str, k: &Kind, has_interfaces: bool) {
        let used: HashSet<String> = k.template.used_vars().into_iter().map(|v| v.name).collect();
        if let Some(g) = d.generics.first()
            && !used.contains("generics")
            && !used.contains("generic_list")
        {
            let msg = format!("the kind '{kind_name}' does not write generics: <{}> would be lost", g.name);
            self.error(codes::KIND_IGNORES, g.span, msg);
        }
        if has_interfaces && !used.contains("implements") && !used.contains("clauses") && !used.contains("clause") {
            let msg = format!("the kind '{kind_name}' does not write interfaces: '> ...' would be lost");
            self.error(codes::KIND_IGNORES, d.name.span, msg);
        }
    }

    /// Parameters as data for templates: name, type, tinfo, optional, named, decor, decorated.
    fn params_data(&mut self, params: &[Param]) -> Value {
        let mut out = Vec::new();
        for p in params {
            let ty = p.ty.as_ref().map(|t| self.ty(t)).unwrap_or_default();
            let tinfo = self.type_info(p.ty.as_ref());
            let (decor, _) = self.decor(&p.decorators);
            let default = p.default.as_ref().map(|e| self.expr(e).0).unwrap_or_default();
            out.push(
                Ctx::new()
                    .set("name", &p.name.name)
                    .set("type", ty)
                    .set("default", default)
                    .value("tinfo", tinfo)
                    .flag("optional", p.optional)
                    .flag("named", p.named)
                    .flag("decorated", !p.decorators.is_empty())
                    .value("decor", decor)
                    .into_value(),
            );
        }
        Value::List(out)
    }

    /// Parameters grouped for templates: `plain` (no decorator) and one group per decorator
    /// (`path`, `query`); each group is `{ list: [...] }`, present only when it has parameters.
    fn param_groups(&self, params_list: &Value) -> Value {
        let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        if let Value::List(items) = params_list {
            for p in items {
                let Value::Map(m) = p else { continue };
                match m.get("decor") {
                    Some(Value::Map(d)) if !d.is_empty() => {
                        for k in d.keys() {
                            groups.entry(k.clone()).or_default().push(p.clone());
                        }
                    }
                    _ => groups.entry("plain".into()).or_default().push(p.clone()),
                }
            }
        }
        Value::Map(
            groups
                .into_iter()
                .map(|(k, v)| (k, Value::Map(BTreeMap::from([("list".to_string(), Value::List(v))]))))
                .collect(),
        )
    }

    /// Modifier words (`static`, `abstract`) → their prefixes. Reports unknown words.
    fn modifiers(&mut self, words: &[Ident], allow_kind: bool) -> String {
        let mut out = String::new();
        for w in words {
            if let Some(m) = self.d.modifiers.get(&w.name) {
                out.push_str(&m.value.prefix);
            } else if allow_kind && self.d.kinds.contains_key(&w.name) {
                // the kind itself
            } else {
                self.unknown_header_word(w);
            }
        }
        out
    }

    fn unknown_header_word(&mut self, w: &Ident) {
        let candidates: Vec<&str> = self.d.kinds.keys().chain(self.d.modifiers.keys()).map(String::as_str).collect();
        let hint =
            closest(&w.name, candidates.iter().copied()).map(|s| format!("; did you mean '{s}'?")).unwrap_or_default();
        let avail =
            if candidates.is_empty() { String::new() } else { format!(" (available: {})", candidates.join(", ")) };
        let msg = if hint.is_empty() {
            format!("'{}' is not a kind or modifier of dialect '{}'{avail}", w.name, self.d.name)
        } else {
            format!("'{}' is not a kind or modifier of dialect '{}'{hint}", w.name, self.d.name)
        };
        self.error(codes::UNKNOWN_HEADER_WORD, w.span, msg);
    }

    fn top_decl(&mut self, d: &Decl) -> String {
        let kind = self.classify(d);
        // what the dialect's project files see of it
        let (decor, _) = self.decor(&d.decorators);
        let ctor_params: Vec<Param> = match &d.body {
            Body::Members(ms) => {
                ms.iter().find_map(|m| if let Member::Ctor(c) = m { Some(c.params.list.clone()) } else { None })
            }
            _ => None,
        }
        .or_else(|| d.params.as_ref().map(|p| p.list.clone()))
        .unwrap_or_default();
        let params =
            if matches!(kind, DeclKind::Function) { Value::List(Vec::new()) } else { self.params_data(&ctor_params) };
        let kind_name = if let DeclKind::Kind(k) = &kind { Some(k.clone()) } else { None };
        if let Some(k) = &kind_name {
            self.kinds_used.insert(k.clone());
        }
        let name = self.decl_names.get(&d.name.name).cloned().unwrap_or_else(|| d.name.name.clone());
        self.summaries.push(DeclSummary {
            name,
            kind: kind_name,
            decor,
            params,
            fields: Value::List(Vec::new()),
            methods: Value::List(Vec::new()),
            generics: generic_list(&d.generics),
        });
        match kind {
            DeclKind::Function => self.function(d),
            DeclKind::Enum => self.enum_decl(d),
            DeclKind::Interface => self.interface(d),
            DeclKind::Class => self.class(d, None),
            DeclKind::Kind(k) => {
                let kind = self.d.kinds.get(&k).map(|e| &e.value);
                match kind {
                    Some(kind) => self.class(d, Some((k.clone(), kind))),
                    None => String::new(),
                }
            }
        }
    }

    fn function(&mut self, d: &Decl) -> String {
        let doc = self.inner_doc.take();
        self.doc_summary = None;
        self.modifiers(&d.words, false);
        let decorators = self.decorators(&d.decorators, false);
        let generics = self.generics(&d.generics, &d.generic_bounds, d.span);
        let export = self.export();
        let name = self.decl_names.get(&d.name.name).cloned().unwrap_or_else(|| d.name.name.clone());
        let (decor, deco_args) = self.decor(&d.decorators);
        let params_list = self.params_data(&d.params.as_ref().map(|p| p.list.clone()).unwrap_or_default());
        let ret_info = self.type_info(d.ret.as_ref());
        let (params, ret, is_async, generator, body) = self.callable(d);
        self.tpl(
            "function",
            Ctx::new()
                .value("decor", decor)
                .value("deco_args", deco_args)
                .value("params_list", params_list)
                .value("ret_info", ret_info)
                .set("name", name)
                .set("generics", generics)
                .set("params", params)
                .opt("ret", ret)
                .flag("async", is_async)
                .flag("generator", generator)
                .set("body", body)
                .set("export", export)
                .opt("doc", doc)
                .set("decorators", decorators),
            d.span,
        )
    }

    /// Parameters, return type, async and generator flags and body of a function or method.
    fn callable(&mut self, d: &Decl) -> (String, Option<String>, bool, bool, String) {
        let (stmts, expr): (Vec<Stmt>, Option<&Expr>) = match &d.body {
            Body::Block(b) => (b.stmts.clone(), None),
            Body::Expr(e) => (Vec::new(), Some(e)),
            _ => (Vec::new(), None),
        };
        let params: Vec<Param> = d.params.as_ref().map(|p| p.list.clone()).unwrap_or_default();
        let is_async = d.is_async || analysis::awaits_block(&stmts) || expr.is_some_and(analysis::awaits_expr);
        let void_word = d.words.iter().any(|w| self.d.modifiers.get(&w.name).is_some_and(|m| m.value.void));
        // a body with `yield` is a generator: its return type is the type of each value
        let generator = analysis::yields_block(&stmts);
        let returns = !void_word && !generator && d.ret.as_ref().is_none_or(|t| !is_void(t));
        let ctx = if generator || void_word { FnCtx::default() } else { self.fn_ctx_for(d.ret.as_ref(), false) };
        self.fn_ctx.push(ctx);
        self.fn_names.push(d.name.name.clone());
        self.frames.push(Frame::default());
        let rendered_params = self.params(&params, false, d.span);
        let ret = if generator {
            d.ret.as_ref().and_then(|t| {
                let inner = self.ty(t);
                let key = if is_async { "async_generator" } else { "generator" };
                self.type_tpl(key, Ctx::new().set("type", inner), d.span)
            })
        } else {
            self.ret_type(d.ret.as_ref(), is_async, d.span)
        };
        let body = match expr {
            Some(e) => self.with_pending(|s| {
                let v = s.tail_expr(e, &Tail::Return);
                s.tpl("return", Ctx::new().set("value", v), e.span)
            }),
            None => self.body(&stmts, returns),
        };
        self.frames.pop();
        self.fn_names.pop();
        self.fn_ctx.pop();
        (rendered_params, ret, is_async, generator, body)
    }

    fn ret_type(&mut self, ret: Option<&Type>, is_async: bool, span: Span) -> Option<String> {
        match (ret, is_async) {
            (Some(t), false) => Some(self.ty(t)),
            (Some(t), true) => {
                let inner = self.ty(t);
                self.type_tpl("async", Ctx::new().set("type", inner), span)
            }
            (None, _) => None,
        }
    }

    /// Parameters of a callee declared in this file: `fn`, `Module`, `Module.method`, `method`.
    fn callee_params(&self, callee: &Expr) -> Option<Vec<Param>> {
        match &callee.kind {
            ExprKind::Name(n) => {
                let out = self.decl_names.get(n).cloned().unwrap_or_else(|| n.clone());
                self.decls.get(&out).map(|i| i.params.clone())
            }
            ExprKind::Member { object, name, .. } => {
                // `Module.method(...)` or `Module(...).method(...)`
                let module = match &object.kind {
                    ExprKind::Name(m) => Some(m),
                    ExprKind::Call { callee, .. } => match &callee.kind {
                        ExprKind::Name(m) => Some(m),
                        _ => None,
                    },
                    _ => None,
                }?;
                let out = self.decl_names.get(module).cloned().unwrap_or_else(|| module.clone());
                if let Some(ps) = self.decls.get(&out).and_then(|i| i.methods.get(&name.name).cloned()) {
                    return Some(ps);
                }
                // `repo.all(...)` where `repo` is a field or variable typed with a declaration of the project
                let t = self.static_type(object)?;
                let TypeKind::Named { name: tn, .. } = &strip_nullable(t).kind else { return None };
                self.decl_info(tn).and_then(|i| i.methods.get(&name.name).cloned())
            }
            _ => None,
        }
    }

    fn params(&mut self, params: &[Param], ctor: bool, span: Span) -> String {
        let base_key = if ctor { "ctor_param" } else { "param" };
        let group_optional = self.has("params_optional_group");
        let group_named = self.has("params_named_group");
        let mut positional = Vec::new();
        let mut optional = Vec::new();
        let mut named = Vec::new();
        for p in params {
            let name = if ctor { p.name.name.clone() } else { self.declare_local(&p.name.name) };
            if !ctor
                && let Some(t) = &p.ty
                && let Some(f) = self.frames.last_mut()
            {
                f.types.insert(p.name.name.clone(), t.clone());
            }
            let mut ty = p.ty.as_ref().map(|t| self.ty(t));
            if p.optional
                && self.d.options.optional_params_nullable
                && let Some(t) = &ty
            {
                ty = self.type_tpl("nullable", Ctx::new().set("type", t.clone()), p.span);
            }
            let default = p.default.as_ref().filter(|e| !self.omitted_default(e)).map(|e| {
                let text = self.expr(e).0;
                if self.d.options.const_collection_defaults && matches!(e.kind, ExprKind::List(_) | ExprKind::Object(_))
                {
                    format!("const {text}")
                } else {
                    text
                }
            });
            let decorators = self.decorators(&p.decorators, true);
            let required = p.named && !p.optional && p.default.is_none();
            let key = if ctor && self.super_params.contains(&p.name.name) {
                "super_param"
            } else if ctor && self.inherited_params.contains(&p.name.name) {
                "param"
            } else {
                base_key
            };
            let text = self.tpl(
                key,
                Ctx::new()
                    .set("name", name)
                    .opt("type", ty)
                    .opt("default", default.clone())
                    .flag("optional", p.optional)
                    .flag("named", p.named)
                    .flag("required", required)
                    .set("decorators", decorators),
                p.span,
            );
            if p.named && group_named {
                named.push(text);
            } else if (p.optional || p.default.is_some()) && group_optional {
                optional.push(text);
            } else if !optional.is_empty() && group_optional {
                // a required parameter after optional ones: no grouping possible, keep the order
                positional.append(&mut optional);
                positional.push(text);
            } else {
                positional.push(text);
            }
        }
        let mut parts = positional;
        if !optional.is_empty() {
            let inner = optional.join(", ");
            parts.push(self.tpl("params_optional_group", Ctx::new().set("params", inner), span));
        }
        if !named.is_empty() {
            let inner = named.join(", ");
            parts.push(self.tpl("params_named_group", Ctx::new().set("params", inner), span));
        }
        self.join_items(&parts, 16)
    }

    fn enum_decl(&mut self, d: &Decl) -> String {
        let Body::Members(members) = &d.body else {
            return String::new();
        };
        let mut items = Vec::new();
        let mut value_type = None;
        // in a string enum, an item without a value takes its own name: `user` → user('user')
        let string_values = members.iter().any(
            |m| matches!(m, Member::EnumItem(e) if matches!(e.args.first().map(|a| &a.kind), Some(ExprKind::Str(_)))),
        );
        for m in members {
            let Member::EnumItem(e) = m else { continue };
            let own_name = Expr { span: e.span, kind: ExprKind::Str(vec![StrPiece::Text(e.name.name.clone())]) };
            let arg = e.args.first().or(if string_values { Some(&own_name) } else { None });
            let value = arg.map(|a| {
                if value_type.is_none() {
                    value_type = Some(match &a.kind {
                        ExprKind::Str(_) => "string",
                        ExprKind::Number(n) if n.kind == NumberKind::Float => "double",
                        ExprKind::Number(_) => "int",
                        ExprKind::Bool(_) => "boolean",
                        _ => "dynamic",
                    });
                }
                self.expr(a).0
            });
            let item = self.tpl("enum_item", Ctx::new().set("name", &e.name.name).opt("value", value), e.span);
            items.push(self.with_doc(e.span.start, item));
        }
        let value_type = value_type.map(|t| {
            let t = Type { span: d.span, kind: TypeKind::Named { name: t.into(), args: Vec::new() } };
            self.ty(&t)
        });
        let export = self.export();
        self.tpl(
            "enum",
            Ctx::new()
                .set("name", &d.name.name)
                .set("items", items.join(",\n"))
                .value("items_list", Value::List(items.iter().map(|i| Value::str(i.clone())).collect()))
                .set("export", export)
                .flag("values", value_type.is_some())
                .opt("value_type", value_type),
            d.span,
        )
    }

    fn interface(&mut self, d: &Decl) -> String {
        self.modifiers(&d.words, false);
        let Body::Members(members) = &d.body else {
            return String::new();
        };
        let mut lines = Vec::new();
        for m in members {
            let text = match m {
                Member::Field(f) => {
                    let ty = f.ty.as_ref().map(|t| self.ty(t)).unwrap_or_default();
                    self.tpl(
                        "interface_field",
                        Ctx::new().set("name", &f.name.name).set("type", ty).flag("optional", f.optional),
                        f.span,
                    )
                }
                Member::Method(md) => {
                    self.frames.push(Frame::default());
                    let params = md.params.as_ref().map(|p| self.params(&p.list, false, md.span)).unwrap_or_default();
                    self.frames.pop();
                    let ret = self.ret_type(md.ret.as_ref(), md.is_async, md.span);
                    let generics = self.generics(&md.generics, &md.generic_bounds, md.span);
                    self.tpl(
                        "interface_method",
                        Ctx::new()
                            .set("name", &md.name.name)
                            .set("generics", generics)
                            .set("params", params)
                            .opt("ret", ret)
                            .flag("async", md.is_async),
                        md.span,
                    )
                }
                _ => continue,
            };
            lines.push(self.with_doc(m.span().start, mark(m.span().start, text)));
        }
        let mut extends: Vec<String> = Vec::new();
        for t in d.bases.iter().chain(d.implements.iter()) {
            extends.push(self.ty(t));
        }
        let generics = self.generics(&d.generics, &d.generic_bounds, d.span);
        let decorators = self.decorators(&d.decorators, false);
        let export = self.export();
        self.tpl(
            "interface",
            Ctx::new()
                .set("name", &d.name.name)
                .set("generics", generics)
                .set("extends", extends.join(", "))
                .set("members", lines.join("\n"))
                .set("decorators", decorators)
                .set("export", export),
            d.span,
        )
    }

    /// A module with code: a class, or a dialect kind (`stateless`, `controller`...).
    fn class(&mut self, d: &Decl, kind: Option<(String, &'a Kind)>) -> String {
        let class_doc = self.inner_doc.take();
        let modifiers = self.modifiers(&d.words, kind.is_some());
        let is_abstract = d.words.iter().any(|w| w.name == "abstract");
        let members: Vec<Member> = match &d.body {
            Body::Members(m) => m.clone(),
            _ => Vec::new(),
        };
        if members.iter().any(|m| matches!(m, Member::EnumItem(_))) {
            for m in &members {
                if let Member::EnumItem(e) = m {
                    self.error(
                        codes::ENUM_SYNTAX,
                        e.span,
                        format!(
                            "'{}' looks like an enum item, but '{}' also has fields or methods",
                            e.name.name, d.name.name
                        ),
                    );
                }
            }
        }
        let prev_kind_syntax = self.kind_syntax;
        let prev_kind = self.cur_kind;
        let prev_type_params =
            std::mem::replace(&mut self.type_params, d.generics.iter().map(|g| g.name.clone()).collect());
        let prev_owner = std::mem::replace(&mut self.owner, d.name.name.clone());
        let prev_provided = std::mem::take(&mut self.provided);
        let (owner_decor, _) = self.decor(&d.decorators);
        let prev_owner_decor = std::mem::replace(&mut self.owner_decor, owner_decor);
        let prev_kind_name =
            std::mem::replace(&mut self.kind_name, kind.as_ref().map(|(n, _)| n.clone()).unwrap_or_default());
        self.cur_kind = kind.as_ref().map(|(_, k)| *k);
        // the kind reads the program's declarations of some kinds with a template (tests: inject<X>())
        if let Some((_, k)) = &kind
            && let Some((tpl, kinds)) = &k.provide
        {
            let names: Vec<String> = self
                .decl_names
                .iter()
                .filter(|(_, out)| {
                    matches!(self.decls.get(*out).map(|i| &i.kind), Some(DeclKind::Kind(dk)) if kinds.contains(dk))
                })
                .map(|(src, _)| src.clone())
                .collect();
            for n in names {
                self.provided.insert(n, tpl.clone());
            }
        }
        // decorators that provide objects to the declaration: @provider([CounterManager])
        for dec in &d.decorators {
            // `provider` = every argument of @provider; `screen.providers` = the argument providers of @screen
            for (key, tpl) in &self.d.provide.clone() {
                let (name, arg) = key.split_once('.').map_or((key.as_str(), None), |(n, a)| (n, Some(a)));
                if name != dec.name {
                    continue;
                }
                for a in dec.args.iter().flatten() {
                    if arg.is_some_and(|arg| a.name.as_ref().is_none_or(|n| n.name != arg)) {
                        continue;
                    }
                    let items = match &a.value.kind {
                        ExprKind::List(items) => items.clone(),
                        _ => vec![a.value.clone()],
                    };
                    for i in items {
                        if let ExprKind::Name(n) = &i.kind {
                            self.provided.insert(n.clone(), tpl.clone());
                        }
                    }
                }
            }
        }
        if let Some((kname, k)) = &kind {
            for m in &members {
                let Member::Method(md) = m else { continue };
                if !k.methods.is_empty() && !k.methods.contains(&md.name.name) {
                    self.error(
                        codes::KIND_METHOD_NOT_ALLOWED,
                        md.name.span,
                        format!(
                            "'{kname} {}' can only have the method(s) {}; move '{}' to another declaration",
                            d.name.name,
                            k.methods.join(", "),
                            md.name.name
                        ),
                    );
                }
                if k.needs_body && matches!(md.body, Body::None) {
                    self.error(
                        codes::KIND_METHOD_BODY,
                        md.name.span,
                        format!(
                            "'{}' of '{kname} {}' has no body: write it as an alias of a use case (`{}() = UseCase.exec`) or give it one",
                            md.name.name, d.name.name, md.name.name
                        ),
                    );
                }
                if let Some(want) = &k.returns {
                    let ok = matches!(&md.ret, Some(Type { kind: TypeKind::Named { name, .. }, .. }) if name == want);
                    if !ok {
                        self.error(
                            codes::KIND_RETURN_TYPE,
                            md.name.span,
                            format!(
                                "every method of '{kname} {}' returns {want}<...>: '{}' does not",
                                d.name.name, md.name.name
                            ),
                        );
                    }
                }
            }
        }
        if let Some((_, k)) = &kind {
            self.kind_syntax = Some(&k.syntax);
            for req in &k.requires {
                let present = members.iter().any(|m| matches!(m, Member::Method(md) if &md.name.name == req));
                if !present {
                    let (name, _) = kind.as_ref().unwrap();
                    self.error(
                        codes::KIND_REQUIRES_MEMBER,
                        d.name.span,
                        format!("'{name} {}' needs a '{req}' method", d.name.name),
                    );
                }
            }
        }
        // constructor: a `(params)` member, or the header's parameters
        let ctor_member = members.iter().find_map(|m| if let Member::Ctor(c) = m { Some(c.clone()) } else { None });
        let ctor_params: Vec<Param> = match (&ctor_member, &d.params) {
            (Some(c), _) => c.params.list.clone(),
            (None, Some(p)) => p.list.clone(),
            _ => Vec::new(),
        };
        let field_names: HashSet<String> = members
            .iter()
            .filter_map(|m| if let Member::Field(f) = m { Some(f.name.name.clone()) } else { None })
            .collect();
        let mut member_names: HashSet<String> = field_names.clone();
        member_names.extend(
            members.iter().filter_map(|m| if let Member::Method(md) = m { Some(md.name.name.clone()) } else { None }),
        );
        member_names.extend(ctor_params.iter().map(|p| p.name.name.clone()));
        if let Some((_, k)) = &kind {
            member_names.extend(k.members.iter().cloned());
        }
        // members inherited from bases declared in this file or in the dialect's type definitions; a base
        // known by neither makes the module open (its members are unknown)
        let pending: Vec<String> = d
            .bases
            .iter()
            .chain(d.clauses.iter().flat_map(|c| c.types.iter()))
            .filter_map(|t| if let TypeKind::Named { name, .. } = &t.kind { Some(name.clone()) } else { None })
            .collect();
        let (inherited, all_known) = self.inherited_members_known(pending);
        member_names.extend(inherited);
        let open = !all_known;
        // the classes a kind extends (`inherits`) and the class every class extends (`implicit_base`)
        let mut implicit: Vec<String> = kind.as_ref().map(|(_, k)| k.inherits.clone()).unwrap_or_default();
        if !self.d.options.implicit_base.is_empty() {
            implicit.push(self.d.options.implicit_base.clone());
        }
        let implicit_members = self.inherited_members(implicit);
        member_names.extend(implicit_members.iter().cloned());
        let prev_module = self.module.replace(ModCtx {
            name: d.name.name.clone(),
            members: member_names,
            ctor_params: ctor_params.iter().map(|p| p.name.name.clone()).collect(),
            param_access: kind.as_ref().and_then(|(_, k)| k.param_access.clone()),
            open,
        });
        let mut overrides: Vec<String> = kind.as_ref().map(|(_, k)| k.overrides.clone()).unwrap_or_default();
        overrides.extend(implicit_members);
        // members of bases declared in this file: extended classes (override) and interfaces
        let named = |t: &Type| {
            if let TypeKind::Named { name, .. } = &t.kind { Some(name.clone()) } else { None }
        };
        let (iface_bases, class_bases): (Vec<&Type>, Vec<&Type>) =
            d.bases.iter().partition(|b| self.base_is_interface(b));
        let extends_class = !class_bases.is_empty();
        let base_members = self.inherited_members(class_bases.into_iter().filter_map(named).collect());
        overrides.extend(base_members.iter().cloned());
        // a constructor parameter named like a member of the base is that member: a plain parameter
        // (passed with super(...)), not a new field
        self.inherited_params =
            ctor_params.iter().map(|p| p.name.name.clone()).filter(|n| base_members.contains(n)).collect();
        let interface_members =
            self.inherited_members(iface_bases.into_iter().chain(d.implements.iter()).filter_map(named).collect());

        // fields declared for constructor parameters (targets that need them)
        let ctor_param_names: HashSet<String> = ctor_params.iter().map(|p| p.name.name.clone()).collect();
        let mut parts: Vec<(bool, String)> = Vec::new(); // (is_field, code)
        if self.d.options.declare_ctor_fields && kind.is_none() {
            for p in &ctor_params {
                if field_names.contains(&p.name.name) || self.inherited_params.contains(&p.name.name) {
                    continue;
                }
                let mut ty = p.ty.as_ref().map(|t| self.ty(t));
                if p.optional
                    && let Some(t) = &ty
                {
                    ty = self.type_tpl("nullable", Ctx::new().set("type", t.clone()), p.span);
                }
                let text = self.tpl(
                    "field",
                    Ctx::new()
                        .set("name", &p.name.name)
                        .opt("type", ty)
                        .set("default", "")
                        .flag("optional", p.optional)
                        .set("modifiers", "")
                        .set("decorators", "")
                        .flag("uninitialized", false),
                    p.span,
                );
                parts.push((true, text));
            }
        }

        let out_name = match &kind {
            Some((_, k)) => match &k.rename {
                Some(r) => r.render(&Ctx::new().set("name", &d.name.name).into_value()),
                None => d.name.name.clone(),
            },
            None => d.name.name.clone(),
        };
        let mut ctor_pos: Option<usize> = None;
        let mut ctor_text: Option<String> = None;
        let mut fields_list = Vec::new();
        let mut fields_required = false;
        let mut field_inits: Vec<(String, String)> = Vec::new();
        let mut deferred_ctor = None;
        let mut methods_list = Vec::new();
        let mut method_parts = Vec::new();
        for m in &members {
            match m {
                Member::Field(f) => {
                    if ctor_param_names.contains(&f.name.name) && !self.d.options.declare_ctor_fields {
                        continue; // the constructor parameter declares it
                    }
                    let mods = self.modifiers(&f.words, false);
                    let decorators = self.decorators(&f.decorators, false);
                    let ty = f.ty.as_ref().map(|t| self.ty(t));
                    let type_nullable =
                        ty.clone().and_then(|t| self.type_tpl("nullable", Ctx::new().set("type", t), f.span));
                    let default = f.default.as_ref().filter(|e| !self.omitted_default(e)).map(|e| self.expr(e).0);
                    // a subclass without its own constructor keeps its defaults in the class: a generated
                    // constructor would hide the inherited one
                    // a kind with its own field template writes its fields itself
                    let in_init = self.d.options.init_fields
                        && self.kind_syntax.is_none_or(|m| !m.contains_key("field"))
                        && f.default.is_some()
                        && (!extends_class || !ctor_params.is_empty() || ctor_member.is_some());
                    if in_init {
                        let value = default.clone().unwrap_or_else(|| self.plain("null", f.span));
                        field_inits.push((f.name.name.clone(), value));
                    }
                    let uninitialized = f.default.is_none() && !f.optional && !ctor_param_names.contains(&f.name.name);
                    let tinfo = self.type_info(f.ty.as_ref());
                    let (decor, _) = self.decor(&f.decorators);
                    self.check_field_rules(f, &field_names);
                    // `@Name` without parentheses and PascalCase names a class (Dart: a JSON converter)
                    let class_decorator = f
                        .decorators
                        .iter()
                        .find(|d| d.args.is_none() && d.name.starts_with(|c: char| c.is_ascii_uppercase()))
                        .map(|d| d.name.clone());
                    // the converter gives its own sample value (`sample() T`) for generated tests
                    let converter_sample = class_decorator
                        .as_ref()
                        .is_some_and(|c| self.decl_info(c).is_some_and(|i| i.methods.contains_key("sample")));
                    if !f.optional && default.is_none() {
                        fields_required = true;
                    }
                    fields_list.push(
                        Ctx::new()
                            .value("tinfo", tinfo)
                            .value("decor", decor.clone())
                            .set("decorators", decorators.clone())
                            .opt("class_decorator", class_decorator)
                            .flag("converter_sample", converter_sample)
                            .set("name", &f.name.name)
                            .opt("type", ty.clone())
                            .opt("type_nullable", type_nullable)
                            .opt("default", default.clone())
                            .flag("optional", f.optional)
                            .into_value(),
                    );
                    let field_tinfo = self.type_info(f.ty.as_ref());
                    let text = self.tpl(
                        "field",
                        Ctx::new()
                            .set("name", &f.name.name)
                            .opt("type", ty)
                            .opt("default", default)
                            .flag("optional", f.optional)
                            .set("modifiers", mods)
                            .set("decorators", decorators)
                            .value("decor", decor)
                            .flag("override", overrides.contains(&f.name.name))
                            .flag("uninitialized", uninitialized)
                            .value("tinfo", field_tinfo)
                            .flag("decorated", !f.decorators.is_empty())
                            .flag("in_init", in_init)
                            .set("owner", &out_name)
                            .value("owner_decor", self.owner_decor.clone()),
                        f.span,
                    );
                    let text = self.with_doc(f.span.start, mark(f.span.start, text));
                    parts.push((true, text));
                }
                Member::Ctor(c) => {
                    if ctor_pos.is_none() && self.d.options.init_fields {
                        // after every field: the constructor sets the ones with a default
                        ctor_pos = Some(parts.len());
                        deferred_ctor = Some(c.clone());
                    } else if ctor_pos.is_none() {
                        ctor_pos = Some(parts.len());
                        // rendered here, in source order, so its comments stay with it
                        let pre = self.doc_before(c.span.start);
                        let text = self.constructor(&out_name, &ctor_params, c.body.as_ref(), extends_class, c.span);
                        ctor_text = Some(attach(pre, text));
                    }
                }
                Member::Method(md) => {
                    let pre = self.doc_into(md.span.start);
                    let text = self.method(
                        md,
                        overrides.contains(&md.name.name),
                        interface_members.contains(&md.name.name),
                        is_abstract,
                    );
                    let data =
                        self.last_method.take().unwrap_or_else(|| Ctx::new().set("name", &md.name.name).into_value());
                    methods_list.push(data);
                    let text = attach(pre, text);
                    method_parts.push(text.clone());
                    parts.push((false, text));
                }
                Member::EnumItem(_) => {}
            }
        }
        if let Some(sum) = self.summaries.last_mut()
            && sum.name == out_name_for_summary(&d.name.name, &self.decl_names)
        {
            sum.fields = Value::List(fields_list.clone());
            sum.methods = Value::List(methods_list.clone());
        }
        if let Some(c) = deferred_ctor {
            let pre = self.doc_before(c.span.start);
            self.ctor_inits = field_inits.clone();
            let text = self.constructor(&out_name, &ctor_params, c.body.as_ref(), extends_class, c.span);
            ctor_text = Some(attach(pre, text));
        }
        let has_ctor = !ctor_params.is_empty()
            || ctor_member.as_ref().is_some_and(|c| c.body.is_some())
            || !field_inits.is_empty();
        if !has_ctor {
            ctor_text = None;
        } else if ctor_text.is_none() {
            // parameters written in the header: `User(name: string)`
            self.ctor_inits = field_inits;
            ctor_text = Some(self.constructor(&out_name, &ctor_params, None, extends_class, d.span));
        }
        let join = |parts: &[(bool, String)]| {
            let mut text = String::new();
            let mut prev_field = None;
            for (is_field, code) in parts {
                if let Some(pf) = prev_field {
                    text.push_str(if pf && *is_field { "\n" } else { "\n\n" });
                }
                text.push_str(code);
                prev_field = Some(*is_field);
            }
            text
        };
        let members_text_no_ctor = join(&parts);
        let mut with_ctor = parts.clone();
        if let Some(c) = ctor_text.as_ref().filter(|c| !c.trim().is_empty()) {
            // where it was written, or after the fields (a kind whose constructor template is empty has none)
            let pos = ctor_pos.unwrap_or_else(|| with_ctor.iter().position(|(f, _)| !f).unwrap_or(with_ctor.len()));
            with_ctor.insert(pos, (false, c.clone()));
        }
        let members_text = join(&with_ctor);
        let methods_code = method_parts.join("\n\n");

        // header
        let (extends, implements) = self.bases(d);
        let (clauses_text, clause_map) = self.clauses(d, kind.is_some());
        let generics = self.generics(&d.generics, &d.generic_bounds, d.span);
        let decorators = self.decorators(&d.decorators, false);
        let export = self.export();

        let result = match &kind {
            None => self.tpl(
                "class",
                Ctx::new()
                    .set("name", &out_name)
                    .set("generics", generics)
                    .set("extends", extends)
                    .set("implements", implements)
                    .set("clauses", clauses_text)
                    .value("clause", clause_map)
                    .set("modifiers", modifiers)
                    .set("decorators", decorators)
                    .set("export", export)
                    .set("members", members_text)
                    .opt("doc", class_doc.clone())
                    .flag("abstract", is_abstract),
                d.span,
            ),
            Some((kind_name, k)) => {
                if let Some(plugin) = self.disabled_plugin_of_kind(kind_name) {
                    let span = d.words.iter().find(|w| &w.name == kind_name).map_or(d.name.span, |w| w.span);
                    let msg = format!("the kind '{kind_name}' is not available: {}", self.plugin_hint(&plugin));
                    self.error(codes::PLUGIN_DISABLED, span, msg);
                }
                self.check_kind_writes(d, kind_name, k, !implements.is_empty());
                // constructor parameters for templates that write the constructor themselves
                let mut params_list = Vec::new();
                for p in &ctor_params {
                    let tinfo = self.type_info(p.ty.as_ref());
                    let (pdecor, _) = self.decor(&p.decorators);
                    let ty = p.ty.as_ref().map(|t| self.ty(t));
                    let default = p.default.as_ref().filter(|e| !self.omitted_default(e)).map(|e| self.expr(e).0);
                    let required = !p.optional && p.default.is_none();
                    params_list.push(
                        Ctx::new()
                            .set("name", &p.name.name)
                            .opt("type", ty)
                            .opt("default", default)
                            .flag("optional", p.optional)
                            .flag("named", p.named)
                            .flag("required", required)
                            .value("tinfo", tinfo)
                            .value("decor", pdecor)
                            .into_value(),
                    );
                }
                let params_text = self.params(&ctor_params, true, d.span);
                let (decor, deco_args) = self.decor(&d.decorators);
                let file_stem = self.file_path.rsplit('/').next().unwrap_or_default().to_string();
                let component = self.component();
                let ctx = Ctx::new()
                    .value("decor", decor)
                    .value("deco_args", deco_args)
                    .set("file", file_stem)
                    .set("project", self.project.map(|p| p.name.clone()).unwrap_or_default())
                    .set("kind", kind.as_ref().map(|(n, _)| n.clone()).unwrap_or_default())
                    .set("component", component)
                    .set("name", &out_name)
                    .set("source_name", &d.name.name)
                    .set("class_name", &out_name)
                    .set("members", members_text_no_ctor)
                    .set("body", members_text)
                    .set("methods_code", methods_code)
                    // a field without `?` and without a default must be given to build the declaration
                    .flag("fields_required", fields_required)
                    .value("fields", Value::List(fields_list))
                    .value("methods", Value::List(methods_list))
                    .set("params", params_text)
                    .value("params_list", Value::List(params_list))
                    .opt("ctor", ctor_text)
                    .set("extends", extends)
                    .set("implements", implements)
                    .set("clauses", clauses_text)
                    .value("clause", clause_map)
                    .set("decorators", decorators)
                    .set("export", export)
                    .set("modifiers", modifiers)
                    .set("generics", generics)
                    .value("generic_list", generic_list(&d.generics))
                    .value("field_decor", field_decor(&members))
                    .opt("doc", class_doc.clone());
                let code = k.template.render(&ctx.into_value());
                // the kind's imports, for the names its template wrote
                for imp in &k.imports {
                    if contains_word(&code, imp) {
                        self.refs.insert(imp.clone());
                    }
                }
                code
            }
        };
        self.module = prev_module;
        self.inherited_params.clear();
        self.kind_syntax = prev_kind_syntax;
        self.cur_kind = prev_kind;
        self.kind_name = prev_kind_name;
        self.type_params = prev_type_params;
        self.owner = prev_owner;
        self.provided = prev_provided;
        self.owner_decor = prev_owner_decor;
        result
    }

    /// Members of the given declarations of this file and of everything they extend or implement.
    fn inherited_members(&self, pending: Vec<String>) -> HashSet<String> {
        self.inherited_members_known(pending).0
    }

    /// Members of the given classes and of everything they extend or implement, looked up in this file
    /// and in the dialect's type definitions (`[dialect] defs`). The flag says whether every class was
    /// found.
    fn inherited_members_known(&self, mut pending: Vec<String>) -> (HashSet<String>, bool) {
        let (mut out, mut seen, mut known) = (HashSet::new(), HashSet::new(), true);
        while let Some(b) = pending.pop() {
            if !seen.insert(b.clone()) {
                continue;
            }
            if let Some(info) = self.decl_names.get(&b).and_then(|o| self.decls.get(o)) {
                out.extend(info.members.iter().cloned());
                pending.extend(info.bases.iter().cloned());
            } else if let Some(def) = self.d.defs.get(&b) {
                out.extend(def.members.iter().cloned());
                pending.extend(def.supers.iter().cloned());
            } else {
                known = false;
            }
        }
        (out, known)
    }

    fn base_is_interface(&self, t: &Type) -> bool {
        matches!(&t.kind, TypeKind::Named { name, .. } if self.is_interface(name))
    }

    /// `: A, B` and `> I`: bases that are interfaces (of this file or of the type definitions) go to `implements`.
    fn bases(&mut self, d: &Decl) -> (String, String) {
        let mut extends = Vec::new();
        let mut implements = Vec::new();
        for b in &d.bases {
            let text = self.ty(b);
            if self.base_is_interface(b) {
                implements.push(text);
            } else {
                extends.push((text, b.span));
            }
        }
        for i in &d.implements {
            implements.push(self.ty(i));
        }
        let max = self.d.options.max_bases;
        if max > 0 && extends.len() > max {
            let span = extends[max].1;
            self.error(
                codes::SINGLE_BASE,
                span,
                format!(
                    "dialect '{}' can extend only {max} base; move the others to '>' if they are interfaces",
                    self.d.name
                ),
            );
        }
        (extends.into_iter().map(|(t, _)| t).collect::<Vec<_>>().join(", "), implements.join(", "))
    }

    /// `on String`, `with Flyer` → rendered text (for classes) and a map (for kind templates).
    fn clauses(&mut self, d: &Decl, in_kind: bool) -> (String, Value) {
        let mut text = String::new();
        let mut map = BTreeMap::new();
        for c in &d.clauses {
            let types: Vec<String> = c.types.iter().map(|t| self.ty(t)).collect();
            let joined = types.join(", ");
            map.insert(c.word.name.clone(), Value::str(joined.clone()));
            match self.d.clauses.get(&c.word.name) {
                Some(def) => {
                    text.push_str(&def.value.template.render(&Ctx::new().set("types", joined).into_value()));
                }
                None if in_kind => {} // the kind template reads it from `clause`
                None => {
                    let hint = closest(&c.word.name, self.d.clauses.keys().map(String::as_str))
                        .map(|s| format!("; did you mean '{s}'?"))
                        .unwrap_or_default();
                    self.error(
                        codes::UNKNOWN_CLAUSE,
                        c.word.span,
                        format!("'{}' is not a clause of dialect '{}'{hint}", c.word.name, self.d.name),
                    );
                }
            }
        }
        (text, Value::Map(map))
    }

    /// When the class extends another and the body does not start with `super(...)`, a target that
    /// needs it (TS) gets the dialect's optional `super_call` template as the first line.
    fn constructor(
        &mut self,
        class_name: &str,
        params: &[Param],
        body: Option<&Block>,
        extends_class: bool,
        span: Span,
    ) -> String {
        let calls_super = body.and_then(|b| b.stmts.first()).is_some_and(|s| {
            matches!(s, Stmt::Expr(Expr { kind: ExprKind::Call { callee, .. }, .. }) if matches!(callee.kind, ExprKind::Super))
        });
        // `super(name)` that passes the inherited parameters as they are, in order: with the dialect's
        // `super_param` template they become super parameters (Dart: `Dog(super.name, this.breed)`)
        let super_args: Option<Vec<String>> = body.and_then(|b| b.stmts.first()).and_then(|s| match s {
            Stmt::Expr(Expr { kind: ExprKind::Call { callee, args, .. }, .. })
                if matches!(callee.kind, ExprKind::Super) =>
            {
                args.iter()
                    .map(|a| match (&a.value.kind, &a.name, a.spread) {
                        (ExprKind::Name(n), None, false) => Some(n.clone()),
                        _ => None,
                    })
                    .collect()
            }
            _ => None,
        });
        let inherited_in_order: Vec<String> =
            params.iter().map(|p| p.name.name.clone()).filter(|n| self.inherited_params.contains(n)).collect();
        let use_super_params = self.has("super_param")
            && !inherited_in_order.is_empty()
            && super_args.as_ref() == Some(&inherited_in_order);
        self.super_params = if use_super_params { inherited_in_order.into_iter().collect() } else { HashSet::new() };
        self.fn_ctx.push(FnCtx::default());
        self.frames.push(Frame::default());
        let rendered = self.params(params, true, span);
        self.super_params.clear();
        // constructor parameters are fields: in the body they are members, not locals
        // a leading `super(...)` goes to the initializer list in targets that require it (Dart)
        let mut initializer = None;
        let mut super_stmt = None;
        let mut stmts: &[Stmt] = body.map_or(&[], |b| b.stmts.as_slice());
        if calls_super && let Some(Stmt::Expr(e)) = stmts.first() {
            // inside super(...) the parameters are the arguments themselves (`this` is not ready yet)
            self.frames.push(Frame::default());
            for p in params {
                self.declare_local(&p.name.name);
            }
            if use_super_params {
                // the arguments are already the super parameters
            } else if self.d.options.super_in_initializer {
                initializer = Some(self.expr(e).0);
            } else {
                super_stmt = Some(self.stmt(&stmts[0]));
            }
            self.frames.pop();
            stmts = &stmts[1..];
        }
        let mut body_text = if stmts.is_empty() { String::new() } else { self.body(stmts, false) };
        let mut inits: Vec<(String, String)> = Vec::new();
        if self.d.options.init_fields {
            let own = params.iter().filter(|p| !self.inherited_params.contains(&p.name.name));
            inits.extend(own.map(|p| (p.name.name.clone(), self.escape(&p.name.name))));
        }
        inits.append(&mut self.ctor_inits);
        if !inits.is_empty() && self.has("field_init") {
            let lines: Vec<String> = inits
                .into_iter()
                .map(|(n, v)| self.tpl("field_init", Ctx::new().set("name", self.escape(&n)).set("value", v), span))
                .collect();
            let i = lines.join("\n");
            body_text = if body_text.is_empty() { i } else { format!("{i}\n{body_text}") };
        }
        if let Some(call) = super_stmt {
            body_text = if body_text.is_empty() { call } else { format!("{call}\n{body_text}") };
        }
        if extends_class && !calls_super && self.has("super_call") {
            let call = self.tpl("super_call", Ctx::new(), span);
            body_text = if body_text.is_empty() { call } else { format!("{call}\n{body_text}") };
        }
        self.frames.pop();
        self.fn_ctx.pop();
        self.tpl(
            "constructor",
            Ctx::new()
                .set("class_name", class_name)
                .set("params", rendered)
                .set("body", body_text)
                .value("owner_decor", self.owner_decor.clone())
                .opt("initializer", initializer),
            span,
        )
    }

    /// `is_override`: replaces a member of an extended class (or a kind's `overrides`);
    /// `implements`: implements a member of an interface (Dart marks both with @override, TS only the first).
    fn method(&mut self, md: &Decl, is_override: bool, implements: bool, owner_abstract: bool) -> String {
        let doc = self.inner_doc.take();
        // `doc_text` / `doc_literal` in `methods`: the description of the method (a tool's description)
        let doc_text = self.doc_summary.take();
        let doc_literal = doc_text.as_ref().map(|t| self.string(&[StrPiece::Text(t.clone())]));
        let method_awaits = md.is_async
            || self.cur_kind.is_some_and(|k| k.async_methods)
            || match &md.body {
                Body::Block(b) => analysis::awaits_block(&b.stmts),
                Body::Expr(e) => analysis::awaits_expr(e),
                _ => false,
            };
        // a kind whose methods return nothing (tests): as if they were written with `void`
        let void_md;
        let md = if self.cur_kind.is_some_and(|k| k.void_methods) && md.ret.is_none() {
            let mut m = md.clone();
            m.ret = Some(Type { span: md.span, kind: TypeKind::Named { name: "void".into(), args: Vec::new() } });
            void_md = m;
            &void_md
        } else {
            md
        };
        let modifiers = self.modifiers(&md.words, false);
        let (decor, deco_args) = self.decor(&md.decorators);
        let mut params_list = self.params_data(&md.params.as_ref().map(|p| p.list.clone()).unwrap_or_default());
        // `route`: the parameter is named in the route of a decorator (`@get(":id")`, `@get("/{id}")`)
        let in_route = route_params(&md.decorators);
        if let Value::List(items) = &mut params_list {
            for item in items {
                if let Value::Map(m) = item {
                    let named = m.get("name").map(|n| in_route.contains(&n.text())).unwrap_or(false);
                    m.insert("route".into(), Value::Bool(named));
                }
            }
        }
        let ret_info = self.type_info(md.ret.as_ref());
        let kind_async = self.cur_kind.is_some_and(|k| k.async_methods);
        let owner = self.owner.clone();
        let owner_decor = self.owner_decor.clone();
        let method_params = md.params.as_ref().map(|p| p.list.clone()).unwrap_or_default();
        let call_args = method_params
            .iter()
            .map(|p| if p.named { format!("{0}: {0}", p.name.name) } else { p.name.name.clone() })
            .collect::<Vec<_>>()
            .join(", ");
        let param_groups = self.param_groups(&params_list);
        self.last_method = Some(
            Ctx::new()
                .set("owner", &owner)
                .value("owner_decor", owner_decor.clone())
                .set("name", &md.name.name)
                .value("is", Value::Map(BTreeMap::from([(md.name.name.clone(), Value::Bool(true))])))
                .value("decor", decor.clone())
                .value("deco_args", deco_args.clone())
                .value("params_list", params_list.clone())
                .value("param_groups", param_groups.clone())
                .value("ret_info", ret_info.clone())
                .opt("doc_text", doc_text)
                .opt("doc_literal", doc_literal)
                .flag("async", method_awaits)
                .into_value(),
        );
        // `mods.get`, `mods.static`...: lets a template place a modifier itself (Dart: `Color get color`)
        let mods = Value::Map(md.words.iter().map(|w| (w.name.clone(), Value::Bool(true))).collect());
        let decorators = self.decorators(&md.decorators, false);
        let generics = self.generics(&md.generics, &md.generic_bounds, md.span);
        if matches!(md.body, Body::None) {
            self.frames.push(Frame::default());
            let params = md.params.as_ref().map(|p| self.params(&p.list, false, md.span)).unwrap_or_default();
            self.frames.pop();
            let ret = self.ret_type(md.ret.as_ref(), md.is_async || kind_async, md.span);
            let text = self.tpl(
                "method_signature",
                Ctx::new()
                    .set("owner", &owner)
                    .value("owner_decor", owner_decor)
                    .set("call_args", call_args)
                    .value("param_groups", param_groups)
                    .value("decor", decor)
                    .value("deco_args", deco_args)
                    .value("params_list", params_list)
                    .value("ret_info", ret_info)
                    .flag("async", md.is_async || kind_async)
                    .set("name", &md.name.name)
                    .set("generics", generics)
                    .set("params", params)
                    .opt("ret", ret)
                    .set("modifiers", modifiers)
                    .value("mods", mods)
                    .flag("abstract", owner_abstract)
                    .set("decorators", decorators)
                    .flag("override", is_override)
                    .opt("doc", doc)
                    .flag("override_interface", implements),
                md.span,
            );
            return text.trim_end().to_string();
        }
        let (params, mut ret, mut is_async, generator, body) = self.callable(md);
        // a kind whose methods are async (`async_methods`): also the ones that await nothing
        if kind_async && !is_async && !generator {
            is_async = true;
            ret = self.ret_type(md.ret.as_ref(), true, md.span);
        }
        let text = self.tpl(
            "method",
            Ctx::new()
                .set("owner", &owner)
                .value("owner_decor", owner_decor)
                .set("call_args", call_args)
                .value("param_groups", param_groups)
                .value("decor", decor)
                .value("deco_args", deco_args)
                .value("params_list", params_list)
                .value("ret_info", ret_info)
                .set("name", &md.name.name)
                .set("generics", generics)
                .set("params", params)
                .opt("ret", ret)
                .flag("async", is_async)
                .flag("generator", generator)
                .set("body", body)
                .set("decorators", decorators)
                .set("modifiers", modifiers)
                .value("mods", mods)
                .flag("override", is_override)
                .opt("doc", doc)
                .flag("override_interface", implements),
            md.span,
        );
        text.trim_end().to_string()
    }

    // ---------- Result<E, T> (spec/nuclear-base.md §3.2) ----------

    /// `Result<E, T>` of the base language (error first) → what is known about E and T.
    fn result_type(&self, t: &Type) -> Option<ResInfo> {
        match &t.kind {
            TypeKind::Named { name, args } if self.result_builtin && name == "Result" && args.len() == 2 => {
                Some(ResInfo { ok: Some(args[1].clone()), err: Some(args[0].clone()) })
            }
            _ => None,
        }
    }

    fn decl_info(&self, name: &str) -> Option<&DeclInfo> {
        let out = self.decl_names.get(name).cloned().unwrap_or_else(|| name.to_string());
        self.decls.get(&out)
    }

    /// The return type of a top-level function of the type definitions (`api.json` `returns`, in Nuclear notation),
    /// looked up by its Nuclear name and, for a snake_case dialect, by the name the target gives it.
    fn def_function_ret(&self, name: &str) -> Option<Type> {
        let (_, d) = self.d.typedefs.get(name).or_else(|| {
            self.d.options.snake_case_names.then(|| self.d.typedefs.get(&crate::naming::snake(name))).flatten()
        })?;
        if d.kind != "function" {
            return None;
        }
        nuclear_syntax::parse_type(d.returns.as_deref()?)
    }

    /// Signature of a method of a module of this file, looked up in the module and its bases.
    fn method_sig(&self, module: &str, method: &str) -> Option<Sig> {
        let mut pending = vec![module.to_string()];
        let mut seen = HashSet::new();
        while let Some(m) = pending.pop() {
            if !seen.insert(m.clone()) {
                continue;
            }
            let Some(info) = self.decl_info(&m) else {
                // a class of the type definitions: its methods that return a Result (index.json `results`)
                if let Some(c) = self.d.defs.get(&m) {
                    if let Some(ret) = c.results.get(method) {
                        return Some(Sig { ret: nuclear_syntax::parse_type(ret), is_async: false });
                    }
                    pending.extend(c.supers.iter().cloned());
                }
                continue;
            };
            if let Some(s) = info.method_sigs.get(method) {
                return Some(s.clone());
            }
            pending.extend(info.bases.iter().cloned());
        }
        None
    }

    /// `sub` is `sup` or extends it (classes of the project or of the type definitions): NotFound is an AppError.
    fn is_subtype(&self, sub: &str, sup: &str) -> bool {
        let mut pending = vec![sub.to_string()];
        let mut seen = HashSet::new();
        while let Some(t) = pending.pop() {
            if t == sup {
                return true;
            }
            if !seen.insert(t.clone()) {
                continue;
            }
            if let Some(info) = self.decl_info(&t) {
                pending.extend(info.bases.iter().cloned());
            } else if let Some(c) = self.d.defs.get(&t) {
                pending.extend(c.supers.iter().cloned());
            }
        }
        false
    }

    /// Known type of a variable or of a member of the module being written.
    fn var_type(&self, n: &str) -> Option<Type> {
        if self.lookup_local(n).is_some() {
            return self.frames.iter().rev().find_map(|f| f.types.get(n).cloned());
        }
        let m = self.module.as_ref().filter(|m| m.members.contains(n))?;
        self.decl_info(&m.name).and_then(|i| i.field_types.get(n).cloned())
    }

    /// The module (of this file) an expression is an instance of, when it is known.
    fn module_of(&self, e: &Expr) -> Option<String> {
        let named = |t: Type| if let TypeKind::Named { name, .. } = t.kind { Some(name) } else { None };
        let m = match &e.kind {
            ExprKind::Name(n) if self.lookup_local(n).is_none() && !self.is_member(n) && is_pascal(n) => n.clone(),
            ExprKind::Name(n) | ExprKind::SelfField(n) => named(self.var_type(n)?)?,
            ExprKind::This => self.module.as_ref()?.name.clone(),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(n) if is_pascal(n) && self.lookup_local(n).is_none() => n.clone(),
                _ => return None,
            },
            _ => return None,
        };
        self.decl_info(&m).map(|_| m)
    }

    /// A Result from a signature: an async call is a Result only when it is awaited.
    fn sig_result(&self, sig: Option<Sig>, awaited: bool) -> Option<ResInfo> {
        let sig = sig?;
        if sig.is_async && !awaited {
            return None;
        }
        self.result_type(sig.ret.as_ref()?)
    }

    /// A call (or `~call`) that returns a Result: a function or method of this file declared with
    /// `Result<E, T>`, a `.ok()`/`.error()` chain on one, and (with `literals`) `Ok(...)`/`Err(...)`.
    fn result_info(&self, e: &Expr, literals: bool) -> Option<ResInfo> {
        if !self.result_builtin {
            return None;
        }
        let (call, awaited) = match &e.kind {
            ExprKind::Unary { op: UnaryOp::Await, expr } => (&**expr, true),
            ExprKind::Call { .. } => (e, false),
            _ => return None,
        };
        let ExprKind::Call { callee, args, .. } = &call.kind else {
            return None;
        };
        match &callee.kind {
            ExprKind::Name(n) if (n == "Ok" || n == "Err") && self.lookup_local(n).is_none() => {
                literals.then(ResInfo::default)
            }
            ExprKind::Name(n) if self.lookup_local(n).is_none() => {
                if self.is_member(n) {
                    let m = self.module.as_ref()?.name.clone();
                    return self.sig_result(self.method_sig(&m, n), awaited);
                }
                let info = self.decl_info(n).filter(|i| i.kind == DeclKind::Function)?;
                self.sig_result(Some(info.sig.clone()), awaited)
            }
            ExprKind::Member { object, name, .. } => {
                if let Some(chain) = self.result_chain(object, &name.name, args) {
                    return if awaited { None } else { Some(chain) };
                }
                let m = self.module_of(object)?;
                self.sig_result(self.method_sig(&m, &name.name), awaited)
            }
            _ => None,
        }
    }

    /// `result.ok(cb)` / `result.error(cb)` on a value known to be a Result.
    fn is_result_callback(&self, object: &Expr, method: &str, args: &[Arg]) -> bool {
        (method == "ok" || method == "error")
            && args.len() == 1
            && !args[0].spread
            && args[0].name.is_none()
            && self.result_value(object).is_some()
    }

    /// `result.ok(cb)` / `result.error(cb)`: what is known of the Result it gives.
    fn result_chain(&self, object: &Expr, method: &str, args: &[Arg]) -> Option<ResInfo> {
        if (method != "ok" && method != "error") || args.len() != 1 || args[0].spread || args[0].name.is_some() {
            return None;
        }
        let inner = self.result_value(object)?;
        let transforms = self.callback_returns_result(&args[0].value) == Some(true);
        let value = !transforms && self.callback_gives_value(&args[0].value);
        match (method, transforms, value) {
            ("ok", true, _) => Some(ResInfo { ok: None, err: inner.err }),
            ("error", true, _) => Some(ResInfo { ok: inner.ok, err: None }),
            // .ok(v => value): the value changes; .error(e => value): recovered, no longer a Result
            ("ok", false, true) => Some(ResInfo { ok: None, err: inner.err }),
            ("error", false, true) => None,
            _ => Some(inner),
        }
    }

    /// A call of an async function that returns a Result, written without `~`: its value is a promise.
    /// Reports it (N0111) and returns true.
    fn result_needs_await(&mut self, e: &Expr) -> bool {
        if !matches!(e.kind, ExprKind::Call { .. }) || self.result_info(e, false).is_some() {
            return false;
        }
        let awaited = Expr { span: e.span, kind: ExprKind::Unary { op: UnaryOp::Await, expr: Box::new(e.clone()) } };
        if self.result_info(&awaited, false).is_none() {
            return false;
        }
        let source = Self::result_source(e);
        self.error(
            codes::RESULT_PATTERN,
            e.span,
            format!("'{source}' is async: write ~ before the call to get its Result (~{source}(...))"),
        );
        true
    }

    /// Any value known to be a Result: a call as in `result_info`, `Ok`/`Err`, or a variable typed Result.
    fn result_value(&self, e: &Expr) -> Option<ResInfo> {
        match &e.kind {
            ExprKind::Name(n) | ExprKind::SelfField(n) => self.result_type(&self.var_type(n)?),
            _ => self.result_info(e, true),
        }
    }

    /// Does a callback of .ok()/.error() return a Result? Some(true) in every branch, Some(false) in
    /// none, None when only some branches do (error N0111).
    fn callback_returns_result(&self, cb: &Expr) -> Option<bool> {
        match &cb.kind {
            ExprKind::Lambda(l) => match &l.body {
                LambdaBody::Expr(x) => self.value_is_result(x),
                LambdaBody::Block(b) => self.block_is_result(b),
            },
            ExprKind::Name(n) if self.lookup_local(n).is_none() => Some(self.decl_info(n).is_some_and(|i| {
                i.kind == DeclKind::Function && self.sig_result(Some(i.sig.clone()), false).is_some()
            })),
            _ => Some(false),
        }
    }

    /// Does a callback of .ok()/.error() that does not return a Result give a value (it transforms the
    /// value or recovers from the error) instead of nothing (it only observes)? A construction, a
    /// literal, a variable, an operation or a call to a function of the program with a return type give
    /// a value; a call to a function of the program that returns void, or to a function outside the
    /// program (print, a logger), gives nothing.
    fn callback_gives_value(&self, cb: &Expr) -> bool {
        match &cb.kind {
            ExprKind::Lambda(l) => match &l.body {
                LambdaBody::Expr(x) => self.expr_gives_value(x),
                LambdaBody::Block(b) => match b.stmts.last() {
                    Some(Stmt::Expr(x)) | Some(Stmt::Return { value: Some(x), .. }) => self.expr_gives_value(x),
                    _ => false,
                },
            },
            ExprKind::Name(n) => self.function_gives_value(n),
            _ => false,
        }
    }

    fn function_gives_value(&self, n: &str) -> bool {
        self.lookup_local(n).is_none()
            && self
                .decl_info(n)
                .is_some_and(|i| i.kind == DeclKind::Function && i.sig.ret.as_ref().is_some_and(|t| !is_void(t)))
    }

    fn expr_gives_value(&self, x: &Expr) -> bool {
        match &x.kind {
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(n) if is_pascal(n) => true,
                ExprKind::Name(n) => self.function_gives_value(n),
                _ => false,
            },
            ExprKind::Unary { op: UnaryOp::Await, expr } => self.expr_gives_value(expr),
            // an if used as a value: every branch, and the else, give a value
            ExprKind::If(i) => {
                let gives = |b: &Block| match b.stmts.last() {
                    Some(Stmt::Expr(x)) => self.expr_gives_value(x),
                    _ => false,
                };
                i.otherwise.as_ref().is_some_and(gives) && i.branches.iter().all(|(_, b)| gives(b))
            }
            _ => true,
        }
    }

    fn value_is_result(&self, e: &Expr) -> Option<bool> {
        match &e.kind {
            ExprKind::If(i) => {
                let mut all: Vec<Option<bool>> = i.branches.iter().map(|(_, b)| self.block_is_result(b)).collect();
                all.push(i.otherwise.as_ref().map_or(Some(false), |b| self.block_is_result(b)));
                combine(all)
            }
            ExprKind::Match(m) => combine(
                m.arms
                    .iter()
                    .map(|a| match &a.body {
                        ArmBody::Expr(x) => self.value_is_result(x),
                        ArmBody::Block(b) => self.block_is_result(b),
                    })
                    .collect(),
            ),
            _ => Some(self.result_value(e).is_some()),
        }
    }

    fn block_is_result(&self, b: &Block) -> Option<bool> {
        match b.stmts.last() {
            Some(Stmt::Expr(x)) | Some(Stmt::Return { value: Some(x), .. }) => self.value_is_result(x),
            Some(Stmt::If(i)) if i.otherwise.is_some() => {
                let mut all: Vec<Option<bool>> = i.branches.iter().map(|(_, b)| self.block_is_result(b)).collect();
                all.extend(i.otherwise.as_ref().map(|b| self.block_is_result(b)));
                combine(all)
            }
            Some(Stmt::Match(m)) => {
                self.value_is_result(&Expr { span: m.span, kind: ExprKind::Match(Box::new(m.clone())) })
            }
            _ => Some(false),
        }
    }

    /// The function or method a Result comes from, for messages and temporary names.
    fn result_source(e: &Expr) -> String {
        match &e.kind {
            ExprKind::Unary { expr, .. } => Self::result_source(expr),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(n) => n.clone(),
                ExprKind::Member { object, name, .. } if name.name == "ok" || name.name == "error" => {
                    Self::result_source(object)
                }
                ExprKind::Member { name, .. } => name.name.clone(),
                _ => "result".into(),
            },
            _ => "result".into(),
        }
    }

    /// A new local for a Result: `{hint}Result`, numbered when the name is taken.
    /// Records the types of a Result variable for the templates.
    fn note_result_types(&mut self, name: &str, info: &ResInfo) {
        let ok = info.ok.as_ref().map(|t| self.ty(t)).unwrap_or_default();
        let err = info.err.as_ref().map(|t| self.ty(t)).unwrap_or_default();
        self.result_types.insert(name.to_string(), (ok, err));
    }

    /// Context of a template that reads a Result variable: `result`, `ok`, `err`.
    fn result_ctx(&self, name: &str) -> Ctx {
        // a type that is not known (a recovered error) is the target's dynamic type
        let any =
            || self.d.types.get("dynamic").map(|e| e.value.render(&Value::Map(BTreeMap::new()))).unwrap_or_default();
        let (ok, err) = self.result_types.get(name).cloned().unwrap_or_default();
        let ok = if ok.is_empty() { any() } else { ok };
        let err = if err.is_empty() { any() } else { err };
        Ctx::new().set("result", name).set("ok", ok).set("err", err)
    }

    fn temp_result(&mut self, hint: &str) -> String {
        let base = format!("{hint}Result");
        // named as the target names variables (options.snake_case_names: task_result)
        let base = if self.d.options.snake_case_names { crate::naming::snake(&base) } else { base };
        let mut name = base.clone();
        let mut i = 2;
        while self.lookup_local(&name).is_some() || self.is_member(&name) || self.decl_names.contains_key(&name) {
            name = format!("{base}{i}");
            i += 1;
        }
        self.declare_local(&name)
    }

    /// Checks that a Result can propagate here and writes the lines that do it before the current
    /// statement: `const r = call; if (r is Err) return its error;`. Returns the temporary name, or
    /// None (after reporting why) when it cannot propagate.
    fn propagate(&mut self, e: &Expr, info: &ResInfo) -> Option<String> {
        // a kind with `results = "keep"` (tests) keeps every Result as a value
        if self.cur_kind.is_some_and(|k| k.keep_results) {
            return None;
        }
        let source = Self::result_source(e);
        let ctx = self.fn_ctx.last().cloned().unwrap_or_default();
        let Some(fn_err) = ctx.result else {
            self.error(
                codes::UNHANDLED_RESULT,
                e.span,
                format!("'{source}' returns a Result that can fail and this function does not return a Result: handle it with match (Ok(x) -> ... / Err(e) -> ...), keep it in a variable typed Result<E, T>, or make this function return Result<E, ...>"),
            );
            return None;
        };
        if self.hoist_blocked > 0 {
            self.error(
                codes::RESULT_NOT_HOISTABLE,
                e.span,
                format!("the Result of '{source}' cannot propagate from a branch that may not run; assign it to a variable first"),
            );
            return None;
        }
        if let (Some(want_t), Some(got_t)) = (&fn_err, &info.err) {
            // only to compare: rendering a type records its import, and this one is not written
            let refs = self.refs.clone();
            let (want, got) = (self.ty(want_t), self.ty(got_t));
            self.refs = refs;
            // compared as written in the target: `Error` of the fastapi dialect is `AppError`
            let subtype = self.is_subtype(&got, &want);
            if want != got && !subtype {
                // a module of this file can be built from the error: Err(AppError(e))
                let convert = match &want_t.kind {
                    TypeKind::Named { name, .. } if self.decl_info(name).is_some() => format!("Err({name}(e))"),
                    _ => "Err(...)".to_string(),
                };
                self.error(
                    codes::RESULT_ERROR_TYPE,
                    e.span,
                    format!("'{source}' fails with '{got}' but this function returns errors of type '{want}'; convert the error first: {source}(...).error(e => {convert})"),
                );
            }
        }
        self.keep_next = true;
        let text = self.expr(e).0;
        let hint = self.temp_hint.take().unwrap_or(source);
        let temp = self.temp_result(&hint);
        self.note_result_types(&temp, info);
        let decl = self.tpl("const_decl", Ctx::new().set("name", temp.clone()).set("value", text), e.span);
        let check = self.tpl("result_propagate", self.result_ctx(&temp), e.span);
        self.pending.push(decl);
        self.pending.push(check);
        self.uses_result = true;
        Some(temp)
    }

    /// A Result used as a value: its Ok value, after propagating its error.
    fn unwrap_result(&mut self, e: &Expr, info: ResInfo) -> (String, u8) {
        match self.propagate(e, &info) {
            Some(temp) => (self.tpl("result_value", self.result_ctx(&temp), e.span), P_POSTFIX),
            None => {
                self.keep_next = true;
                self.expr(e)
            }
        }
    }

    /// `Ok(x)` / `Err(e)`
    fn result_literal(&mut self, ok: bool, args: &[Arg], span: Span) -> String {
        self.uses_result = true;
        let value = args.first().map(|a| self.expr(&a.value).0).unwrap_or_default();
        self.tpl(if ok { "result_ok" } else { "result_err" }, Ctx::new().set("value", value), span)
    }

    /// `result.ok(cb)` / `result.error(cb)`: transforms (cb returns Ok/Err) or observes.
    fn result_method(&mut self, object: &Expr, method: &str, cb: &Expr, span: Span) -> String {
        let transforms = match self.callback_returns_result(cb) {
            Some(t) => t,
            None => {
                self.error(
                    codes::RESULT_PATTERN,
                    cb.span,
                    format!("the callback of .{method}() returns Ok/Err in some branches only; return Ok(...)/Err(...) in every branch to transform, or in none to only observe"),
                );
                false
            }
        };
        self.keep_next = true;
        let target = self.expr(object);
        let target = self.wrap(target, P_POSTFIX);
        self.next_lambda_result = transforms;
        let callback = self.expr(cb).0;
        self.next_lambda_result = false;
        let value = !transforms && self.callback_gives_value(cb);
        let key = match (method, transforms, value) {
            ("ok", true, _) => "result_then",
            ("ok", false, true) => "result_map",
            ("ok", false, false) => "result_tap",
            (_, true, _) => "result_recover",
            (_, false, true) => "result_get_or_else",
            _ => "result_tap_error",
        };
        self.uses_result = true;
        self.tpl(key, Ctx::new().set("target", target).set("callback", callback), span)
    }

    /// Does a `.ok()`/`.error()` chain handle the error branch? (`call().error(...)`)
    fn handles_error(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Unary { expr, .. } => Self::handles_error(expr),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Member { name, .. } if name.name == "error" => true,
                ExprKind::Member { object, name, .. } if name.name == "ok" => Self::handles_error(object),
                _ => false,
            },
            _ => false,
        }
    }

    /// The arms of a match on a Result, or None when the match is not on a Result.
    fn result_arms<'m>(&mut self, m: &'m Match) -> Option<Vec<ResultArm<'m>>> {
        let is_ok_err = |p: &Pattern| {
            matches!(p, Pattern::Value(Expr { kind: ExprKind::Call { callee, .. }, .. })
                if matches!(&callee.kind, ExprKind::Name(n) if n == "Ok" || n == "Err"))
        };
        let on_result = self.result_builtin
            && (m.arms.iter().any(|a| a.patterns.iter().any(is_ok_err)) || self.result_value(&m.subject).is_some());
        if !on_result {
            return None;
        }
        self.result_needs_await(&m.subject);
        let mut arms = Vec::new();
        for arm in &m.arms {
            let parsed = match arm.patterns.as_slice() {
                [Pattern::Wildcard(_)] => Some((None, None)),
                [Pattern::Value(Expr { kind: ExprKind::Call { callee, args, .. }, .. })] => {
                    match (&callee.kind, args.as_slice()) {
                        (ExprKind::Name(n), [a]) if (n == "Ok" || n == "Err") && a.name.is_none() && !a.spread => {
                            match &a.value.kind {
                                ExprKind::Name(v) => Some((Some(n == "Ok"), (v != "_").then(|| v.clone()))),
                                _ => None,
                            }
                        }
                        (ExprKind::Name(n), []) if n == "Ok" || n == "Err" => Some((Some(n == "Ok"), None)),
                        _ => None,
                    }
                }
                _ => None,
            };
            match parsed {
                Some((ok, bind)) => arms.push(ResultArm { ok, bind, body: &arm.body, span: arm.span }),
                None => self.error(
                    codes::RESULT_PATTERN,
                    arm.span,
                    "a match on a Result has one pattern per arm: Ok(name) ->, Err(name) -> or _ ->",
                ),
            }
        }
        Some(arms)
    }

    /// The subject of a match on a Result as a name: itself when it is a variable, otherwise a new
    /// local written before (`lines`, or the pending lines of the statement when `hoist`).
    fn result_subject(&mut self, subject: &Expr, lines: &mut Vec<String>, hoist: bool) -> String {
        if let ExprKind::Name(n) = &subject.kind
            && self.lookup_local(n).is_some()
        {
            let name = self.name_value(n, subject.span);
            let ty = self.frames.iter().rev().find_map(|f| f.types.get(n).cloned());
            if let Some(info) = ty.and_then(|t| self.result_type(&t)) {
                self.note_result_types(&name, &info);
            }
            return name;
        }
        let info = self.result_info(subject, true);
        self.keep_next = true;
        let text = self.expr(subject).0;
        if hoist && self.hoist_blocked > 0 {
            self.error(
                codes::RESULT_NOT_HOISTABLE,
                subject.span,
                "this match on a Result is inside a branch that may not run; assign the Result to a variable first",
            );
        }
        let hint = Self::result_source(subject);
        let temp = self.temp_result(&hint);
        if let Some(info) = &info {
            self.note_result_types(&temp, info);
        }
        let decl = self.tpl("const_decl", Ctx::new().set("name", temp.clone()).set("value", text), subject.span);
        if hoist {
            self.pending.push(decl)
        } else {
            lines.push(decl)
        }
        temp
    }

    /// Ok(x) / Err(e) arms are exhaustive with both, or with `_`.
    fn result_arms_exhaustive(arms: &[ResultArm]) -> bool {
        arms.iter().any(|a| a.ok.is_none())
            || (arms.iter().any(|a| a.ok == Some(true)) && arms.iter().any(|a| a.ok == Some(false)))
    }

    /// Body of one arm with its name bound to the Ok value or the error.
    fn result_arm_body(&mut self, arm: &ResultArm, temp: &str, tail: &Tail) -> String {
        let bound = arm.bind.as_ref().map(|b| {
            let key = if arm.ok == Some(true) { "result_value" } else { "result_error" };
            (b.clone(), self.tpl(key, self.result_ctx(temp), arm.span))
        });
        self.scoped(|e| {
            if let Some((name, text)) = bound
                && let Some(f) = e.frames.last_mut()
            {
                f.vars.insert(name, text);
            }
            match arm.body {
                ArmBody::Expr(x) => {
                    let v = e.tail_expr(x, tail);
                    e.tail_value(tail, v, x.span)
                }
                ArmBody::Block(b) => e.stmts(&b.stmts, tail),
            }
        })
    }

    /// The test that selects an arm: `r.isOk` or `r.isErr` (None for `_`).
    fn result_test(&mut self, ok: Option<bool>, temp: &str, span: Span) -> Option<String> {
        let key = match ok? {
            true => "result_is_ok",
            false => "result_is_err",
        };
        Some(self.tpl(key, self.result_ctx(temp), span))
    }

    /// A match on a Result as statements: `if (r.isOk) { ... } else { ... }`.
    fn result_match_stmt(&mut self, m: &Match, arms: Vec<ResultArm>, tail: &Tail) -> String {
        self.uses_result = true;
        let mut lines = Vec::new();
        let temp = self.result_subject(&m.subject, &mut lines, false);
        let mut out = String::new();
        let mut first = true;
        let mut seen: HashSet<bool> = HashSet::new();
        for arm in &arms {
            if let Some(k) = arm.ok
                && !seen.insert(k)
            {
                continue; // a second Ok (or Err) arm never runs
            }
            let body = self.result_arm_body(arm, &temp, tail);
            // the last possible arm is the else
            let last = arm.ok.is_none() || seen.len() == 2;
            let test = if last && !first { None } else { self.result_test(arm.ok, &temp, arm.span) };
            match test {
                Some(c) => {
                    let key = if first { "if" } else { "else_if" };
                    out.push_str(&self.tpl(key, Ctx::new().set("cond", c).set("body", body), arm.span));
                }
                None if first => out.push_str(&body),
                None => out.push_str(&self.tpl("else", Ctx::new().set("body", body), arm.span)),
            }
            first = false;
            if last {
                break;
            }
        }
        lines.push(out);
        lines.join("\n")
    }

    /// A match on a Result with one expression per arm, used as a value: `r.isOk ? a : b`.
    fn result_match_value(&mut self, m: &Match, arms: Vec<ResultArm>) -> (String, u8) {
        self.uses_result = true;
        if !Self::result_arms_exhaustive(&arms) {
            self.error(
                codes::BRANCHING_VALUE,
                m.span,
                "a match on a Result used as a value needs Ok and Err arms, or '_'",
            );
            return (String::new(), P_PRIMARY);
        }
        let temp = self.result_subject(&m.subject, &mut Vec::new(), true);
        let first = &arms[0];
        let other = arms.iter().find(|a| a.ok.is_none() || a.ok != first.ok).unwrap_or(first);
        let render = |e: &mut Self, arm: &ResultArm| -> (String, u8) {
            let ArmBody::Expr(x) = arm.body else {
                e.error(
                    codes::BRANCHING_VALUE,
                    arm.span,
                    "when match is used as a value, each branch must be a single expression",
                );
                return (String::new(), P_PRIMARY);
            };
            let bound = arm.bind.as_ref().map(|b| {
                let key = if arm.ok == Some(true) { "result_value" } else { "result_error" };
                (b.clone(), e.tpl(key, e.result_ctx(&temp), arm.span))
            });
            e.hoist_blocked += 1;
            let v = e.scoped(|e| {
                if let Some((name, text)) = bound
                    && let Some(f) = e.frames.last_mut()
                {
                    f.vars.insert(name, text);
                }
                e.expr(x)
            });
            e.hoist_blocked -= 1;
            v
        };
        // arms run in order: a first `_` takes every value
        let Some(cond) = self.result_test(first.ok, &temp, m.span) else {
            return render(self, first);
        };
        let t = render(self, first);
        let o = render(self, other);
        let t = self.wrap(t, P_TERNARY);
        let o = self.wrap(o, P_TERNARY);
        (self.tpl("ternary", Ctx::new().set("cond", cond).set("then", t).set("else", o), m.span), P_TERNARY)
    }

    /// The value of a block's last line: a Result is returned as it is where the function allows it.
    fn tail_expr(&mut self, e: &Expr, tail: &Tail) -> String {
        if matches!(tail, Tail::Return) && self.fn_ctx.last().is_some_and(|c| c.keep_tail) {
            self.keep_next = true;
        }
        self.expr(e).0
    }

    /// Runs `f` with its own pending lines and returns them with what `f` wrote.
    fn with_pending(&mut self, f: impl FnOnce(&mut Self) -> String) -> String {
        let saved = std::mem::take(&mut self.pending);
        let code = f(self);
        let pre = std::mem::replace(&mut self.pending, saved);
        match (pre.is_empty(), code.is_empty()) {
            (true, _) => code,
            (false, true) => pre.join("\n"),
            (false, false) => format!("{}\n{code}", pre.join("\n")),
        }
    }

    /// The context of a function or lambda with this return type.
    fn fn_ctx_for(&self, ret: Option<&Type>, result_callback: bool) -> FnCtx {
        let res = ret.and_then(|t| self.result_type(t));
        FnCtx {
            keep_tail: ret.is_none() || res.is_some(),
            result: match res {
                Some(i) => Some(i.err),
                None if result_callback => Some(None),
                None => None,
            },
        }
    }

    // ---------- statements ----------

    /// A function body. With `returns`, the last expression is returned (spec section 4.1).
    fn body(&mut self, stmts: &[Stmt], returns: bool) -> String {
        self.stmts(stmts, if returns { &Tail::Return } else { &Tail::None })
    }

    /// The statements of one block. A variable created here is mutable when it is assigned again later.
    fn stmts(&mut self, stmts: &[Stmt], tail: &Tail) -> String {
        let reassigned = analysis::reassigned_after(stmts);
        if let Some(f) = self.frames.last_mut() {
            f.mutable.extend(reassigned);
        }
        let n = stmts.len();
        let mut lines = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let pre = self.comments_before(s.span().start);
            let code = self.with_pending(|e| if i + 1 == n { e.stmt_tail(s, tail) } else { e.stmt(s) });
            lines.push(attach(pre, mark(s.span().start, code)));
        }
        lines.join("\n")
    }

    /// A nested block (if, for, while, match, try): the variables created in it end with it.
    fn block(&mut self, b: &Block, tail: &Tail) -> String {
        self.scoped(|e| e.stmts(&b.stmts, tail))
    }

    fn scoped<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let Some(frame) = self.frames.last() else {
            return f(self);
        };
        let before: HashSet<String> = frame.vars.keys().cloned().collect();
        let mutable = frame.mutable.clone();
        let out = f(self);
        if let Some(frame) = self.frames.last_mut() {
            let created: Vec<String> = frame.vars.keys().filter(|k| !before.contains(*k)).cloned().collect();
            for k in created {
                frame.vars.remove(&k);
                frame.gone.insert(k);
            }
            frame.mutable = mutable;
        }
        out
    }

    /// The value of the last statement, used as `tail` says.
    fn tail_value(&mut self, tail: &Tail, value: String, span: Span) -> String {
        match tail {
            Tail::None => self.tpl("expr_stmt", Ctx::new().set("expr", value), span),
            Tail::Return => self.tpl("return", Ctx::new().set("value", value), span),
            Tail::Assign(target) => {
                self.tpl("assign", Ctx::new().set("target", target.clone()).set("op", "=").set("value", value), span)
            }
        }
    }

    /// Last statement of a block whose value is returned or assigned (if/match work like Elixir).
    fn stmt_tail(&mut self, s: &Stmt, tail: &Tail) -> String {
        if matches!(tail, Tail::None) {
            return self.stmt(s);
        }
        match s {
            Stmt::Expr(e) => {
                let v = self.tail_expr(e, tail);
                self.tail_value(tail, v, e.span)
            }
            Stmt::Assign(a) if a.op == AssignOp::Set => {
                // `total = a + b` as the last line: the new variable is the value
                if let ExprKind::Name(n) = &a.target.kind
                    && self.lookup_local(n).is_none()
                    && !self.is_member(n)
                {
                    let decl = self.stmt(s);
                    let name = self.lookup_local(n).unwrap_or_else(|| n.clone());
                    let value = self.tail_value(tail, name, a.span);
                    return format!("{decl}\n{value}");
                }
                self.stmt(s)
            }
            Stmt::If(i) if i.otherwise.is_some() => self.if_stmt(i, tail),
            Stmt::Match(m) => self.match_stmt(m, tail),
            // `try` as the last line: the last line of `try` and of `catch` is the value
            Stmt::Try(t) => self.try_stmt(t, tail),
            _ => self.stmt(s),
        }
    }

    fn try_stmt(&mut self, t: &Try, tail: &Tail) -> String {
        let body = self.block(&t.body, tail);
        let mut ctx = Ctx::new().set("body", body);
        if let Some((var, b)) = &t.catch {
            let name = var.as_ref().map_or_else(|| self.d.options.catch_var.clone(), |v| v.name.clone());
            let (out, cb) = self.scoped(|e| {
                let out = e.declare_local(&name);
                // the caught value is an Error of the library: `e.message`
                if let Some(f) = e.frames.last_mut() {
                    f.types.insert(name.clone(), named_type("Error", Vec::new(), t.span));
                }
                (out, e.stmts(&b.stmts, tail))
            });
            ctx = ctx.flag("catch", true).set("catch_var", out).set("catch_body", cb);
        }
        if let Some(b) = &t.finally {
            let fb = self.block(b, &Tail::None);
            ctx = ctx.flag("finally", true).set("finally_body", fb);
        }
        self.tpl("try", ctx, t.span)
    }

    fn stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Expr(e) => {
                if let Some(info) = self.result_info(e, false) {
                    let in_result_fn = self.fn_ctx.last().is_some_and(|c| c.result.is_some());
                    if in_result_fn {
                        // a call whose value is not used: only its error propagates
                        if self.propagate(e, &info).is_some() {
                            return String::new();
                        }
                        self.keep_next = true;
                    } else if Self::handles_error(e) {
                        // `call().error(...)` in a function without Result: the error branch is handled
                        self.keep_next = true;
                    }
                }
                let v = self.expr(e).0;
                self.tpl("expr_stmt", Ctx::new().set("expr", v), e.span)
            }
            Stmt::Assign(a) => self.assign(a),
            Stmt::If(i) => self.if_stmt(i, &Tail::None),
            Stmt::For(f) => self.scoped(|e| e.for_stmt(f)),
            Stmt::While { span, cond, body } => {
                // the condition runs again on every turn: a Result in it cannot be unwrapped before
                self.hoist_blocked += 1;
                let c = self.expr(cond).0;
                self.hoist_blocked -= 1;
                let b = self.block(body, &Tail::None);
                self.tpl("while", Ctx::new().set("cond", c).set("body", b), *span)
            }
            Stmt::Match(m) => self.match_stmt(m, &Tail::None),
            Stmt::Try(t) => self.try_stmt(t, &Tail::None),
            Stmt::Return { span, value } => {
                // `return if ...` / `return match ...` with branches of several lines
                match value.as_ref().map(|e| &e.kind) {
                    Some(ExprKind::If(i)) if !single_expr_if(i) && i.otherwise.is_some() => {
                        return self.if_stmt(i, &Tail::Return);
                    }
                    Some(ExprKind::Match(m)) if !single_expr_match(m) => {
                        return self.match_stmt(m, &Tail::Return);
                    }
                    _ => {}
                }
                let v = value.as_ref().map(|e| self.tail_expr(e, &Tail::Return));
                self.tpl("return", Ctx::new().opt("value", v), *span)
            }
            Stmt::Throw { span, value } => {
                let v = self.expr(value).0;
                self.tpl("throw", Ctx::new().set("value", v), *span)
            }
            Stmt::Yield { span, value, all } => {
                let v = self.expr(value).0;
                self.tpl(if *all { "yield_all" } else { "yield" }, Ctx::new().set("value", v), *span)
            }
            Stmt::Break(span) => self.plain("break", *span),
            Stmt::Continue(span) => self.plain("continue", *span),
        }
    }

    fn assign(&mut self, a: &Assign) -> String {
        // `pieces = ~pieces(raw)` inside the module that has a method `pieces`: a method is not a variable
        if let ExprKind::Name(n) = &a.target.kind
            && self.lookup_local(n).is_none()
            && let Some(m) = self.module.as_ref().map(|m| m.name.clone())
            && self.decl_info(&m).is_some_and(|i| i.methods.contains_key(n))
        {
            self.error(
                codes::BAD_ASSIGN_TARGET,
                a.target.span,
                format!("'{n}' is a method of '{m}': it cannot be assigned; give the variable another name"),
            );
        }
        if a.op == AssignOp::Set
            && let Some(code) =
                self.assign_branching(a).or_else(|| self.destructure(a)).or_else(|| self.cascade_statements(a))
        {
            return code;
        }
        // `r: Result<E, T> = call()` keeps the Result; `x = call()` unwraps it (x is the Ok value)
        let value_result = self.result_info(&a.value, false);
        if a.ty.as_ref().is_some_and(|t| self.result_type(t).is_some()) {
            self.keep_next = true;
        } else if value_result.is_some()
            && let ExprKind::Name(n) = &a.target.kind
        {
            self.temp_hint = Some(n.clone());
        }
        if let ExprKind::Name(n) = &a.target.kind
            && a.op == AssignOp::Set
            && self.lookup_local(n).is_none()
            && !self.is_member(n)
        {
            // first assignment: a declaration
            let value = self.expr(&a.value).0;
            self.temp_hint = None;
            let known = a.ty.clone().or_else(|| match &a.value.kind {
                // a kind that keeps its Results (tests) holds the Result itself; elsewhere it is unwrapped
                _ if value_result.is_some() && self.cur_kind.is_some_and(|k| k.keep_results) => value_result.as_ref().and_then(|r| {
                    let (ok, err) = (r.ok.clone()?, r.err.clone().unwrap_or_else(|| named_type("Error", Vec::new(), a.span)));
                    Some(named_type("Result", vec![err, ok], a.span))
                }),
                _ if value_result.is_some() => value_result.as_ref().and_then(|r| r.ok.clone()),
                ExprKind::Call { callee, type_args, .. } => match &callee.kind {
                    ExprKind::Name(c) if is_pascal(c) && self.decl_info(c).is_some() => {
                        Some(Type { span: a.span, kind: TypeKind::Named { name: c.clone(), args: Vec::new() } })
                    }
                    // `Map<string, int>()`: the platform's collections, for `for k, v in m` and `k in m`
                    ExprKind::Name(c) if matches!(c.as_str(), "Map" | "Set" | "List") => {
                        Some(Type { span: a.span, kind: TypeKind::Named { name: c.clone(), args: type_args.clone() } })
                    }
                    _ => None,
                },
                // a map literal: `k in o` asks for a key
                ExprKind::Object(_) => {
                    let any = || Box::new(Type { span: a.span, kind: TypeKind::Named { name: "dynamic".into(), args: Vec::new() } });
                    Some(Type { span: a.span, kind: TypeKind::Map(any(), any()) })
                }
                _ => None,
            })
            // the static type of the value (a literal, a library member, a declared call): for the library
            .or_else(|| if value_result.is_some() { None } else { self.static_type(&a.value) });
            let mutable = self.frames.iter().rev().any(|f| f.mutable.contains(n));
            let out = self.declare_local(n);
            if let Some(t) = known
                && let Some(f) = self.frames.last_mut()
            {
                f.types.insert(n.clone(), t);
            }
            let ty = a.ty.as_ref().map(|t| self.ty(t));
            let key = if mutable { "let_decl" } else { "const_decl" };
            return self.tpl(key, Ctx::new().set("name", out).opt("type", ty).set("value", value), a.span);
        }
        let keep = std::mem::take(&mut self.keep_next);
        self.assign_target = true;
        let target = self.expr(&a.target).0;
        self.assign_target = false;
        self.keep_next = keep;
        // `x >>>= 1`, `x **= 2`, `x ??= y`: a target without that compound operator (its operator template is
        // not a plain symbol) gets `x = x op y`
        let binary = match a.op {
            AssignOp::Set => None,
            AssignOp::Add => Some(BinaryOp::Add),
            AssignOp::Sub => Some(BinaryOp::Sub),
            AssignOp::Mul => Some(BinaryOp::Mul),
            AssignOp::Div => Some(BinaryOp::Div),
            AssignOp::Rem => Some(BinaryOp::Rem),
            AssignOp::Coalesce => Some(BinaryOp::Coalesce),
            AssignOp::BitAnd => Some(BinaryOp::BitAnd),
            AssignOp::BitOr => Some(BinaryOp::BitOr),
            AssignOp::BitXor => Some(BinaryOp::BitXor),
            AssignOp::Shl => Some(BinaryOp::Shl),
            AssignOp::Shr => Some(BinaryOp::Shr),
            AssignOp::UShr => Some(BinaryOp::UShr),
        };
        if let Some(op) = binary
            && self.d.operators.get(op.key()).is_some_and(|t| !t.value.is_constant())
        {
            let expr = Expr {
                span: a.span,
                kind: ExprKind::Binary { op, left: Box::new(a.target.clone()), right: Box::new(a.value.clone()) },
            };
            let value = self.expr(&expr).0;
            self.temp_hint = None;
            return self.tpl("assign", Ctx::new().set("target", target).set("op", "=").set("value", value), a.span);
        }
        let value = self.expr(&a.value).0;
        self.temp_hint = None;
        self.tpl("assign", Ctx::new().set("target", target).set("op", a.op.as_str()).set("value", value), a.span)
    }

    /// The `..` lines of a cascade, with the object written as `placeholder` (Dart: empty, so the
    /// member template gives `.color` and the cascade template adds the other dot).
    fn cascade_sections(&mut self, sections: &[CascadeSection], placeholder: String) -> Vec<String> {
        self.cascade_self.push(placeholder);
        let mut out = Vec::new();
        for sec in sections {
            let access = self.expr(&sec.access).0;
            out.push(match &sec.assign {
                Some((op, v)) => {
                    let v = self.expr(v).0;
                    format!("{access} {} {v}", op.as_str())
                }
                None => access,
            });
        }
        self.cascade_self.pop();
        out
    }

    /// `paint = Paint()` + `..color = red` with options.cascade_statements: the declaration and one
    /// statement per section (`paint.color = red;`).
    fn cascade_statements(&mut self, a: &Assign) -> Option<String> {
        let ExprKind::Cascade { target, sections } = &a.value.kind else {
            return None;
        };
        let ExprKind::Name(n) = &a.target.kind else {
            return None;
        };
        if !self.d.options.cascade_statements || self.lookup_local(n).is_some() || self.is_member(n) {
            return None;
        }
        let value = self.expr(target).0;
        let mutable = self.frames.iter().rev().any(|f| f.mutable.contains(n));
        let out = self.declare_local(n);
        let ty = a.ty.as_ref().map(|t| self.ty(t));
        let key = if mutable { "let_decl" } else { "const_decl" };
        let mut lines =
            vec![self.tpl(key, Ctx::new().set("name", out.clone()).opt("type", ty).set("value", value), a.span)];
        for sec in self.cascade_sections(sections, out) {
            lines.push(format!("{sec};"));
        }
        Some(lines.join("\n"))
    }

    /// `[a, ...rest] = xs`, `{ name, "k": v } = map`: all the names are new (a declaration) or all
    /// already exist (an assignment).
    fn destructure(&mut self, a: &Assign) -> Option<String> {
        if !matches!(a.target.kind, ExprKind::List(_) | ExprKind::Object(_)) {
            return None;
        }
        let names = analysis::pattern_names(&a.target);
        let existing: Vec<&String> =
            names.iter().filter(|n| self.lookup_local(n).is_some() || self.is_member(n)).collect();
        if !existing.is_empty() && existing.len() != names.len() {
            let list = existing.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", ");
            self.error(
                codes::MIXED_PATTERN,
                a.target.span,
                format!("this pattern mixes names that already exist ({list}) with new ones; use only new names or only existing ones"),
            );
            return Some(String::new());
        }
        let declare = existing.is_empty();
        let value_type = self.static_type(&a.value);
        let value = self.expr(&a.value).0;
        if declare {
            let mutable = names.iter().any(|n| self.frames.iter().rev().any(|f| f.mutable.contains(n)));
            let pattern = self.pattern(&a.target, true);
            // `[first, ...rest] = xs`: the names take the element type, the spread the list's
            if let ExprKind::List(items) = &a.target.kind
                && let Some(t) = value_type
                && let Some(elem) = elem_of(&t)
            {
                for item in items {
                    let (name, ty) = match &item.kind {
                        ExprKind::Name(n) => (n, elem.clone()),
                        ExprKind::Spread(inner) => match &inner.kind {
                            ExprKind::Name(n) => (n, strip_nullable(t.clone())),
                            _ => continue,
                        },
                        _ => continue,
                    };
                    if let Some(f) = self.frames.last_mut() {
                        f.types.insert(name.clone(), ty);
                    }
                }
            }
            let key = if mutable { "let_decl" } else { "const_decl" };
            return Some(self.tpl(key, Ctx::new().set("name", pattern).set("value", value), a.span));
        }
        let pattern = self.pattern(&a.target, false);
        Some(self.tpl("pattern_assign", Ctx::new().set("target", pattern).set("value", value), a.span))
    }

    fn pattern(&mut self, p: &Expr, declare: bool) -> String {
        match &p.kind {
            ExprKind::Name(n) if n == "_" => self.tpl("pattern_ignore", Ctx::new(), p.span),
            ExprKind::Name(n) => {
                if declare {
                    self.declare_local(n)
                } else {
                    self.name_value(n, p.span)
                }
            }
            ExprKind::Spread(x) => {
                let v = self.pattern(x, declare);
                self.tpl("spread_arg", Ctx::new().set("expr", v), p.span)
            }
            ExprKind::List(items) => {
                let items: Vec<String> = items.iter().map(|x| self.pattern(x, declare)).collect();
                self.tpl("list", Ctx::new().set("items", items.join(", ")), p.span)
            }
            ExprKind::Object(entries) => {
                let mut out = Vec::new();
                for en in entries {
                    out.push(match en {
                        ObjEntry::Shorthand(i) => {
                            let value =
                                self.pattern(&Expr { span: i.span, kind: ExprKind::Name(i.name.clone()) }, declare);
                            if value == i.name {
                                self.tpl("object_shorthand", Ctx::new().set("key", &i.name), i.span)
                            } else {
                                let k = self.tpl("object_key_ident", Ctx::new().set("key", &i.name), i.span);
                                self.tpl("object_entry", Ctx::new().set("key", k).set("value", value), i.span)
                            }
                        }
                        ObjEntry::Pair { key, value } => {
                            let k = match key {
                                ObjKey::Ident(i) => {
                                    self.tpl("object_key_ident", Ctx::new().set("key", &i.name), i.span)
                                }
                                ObjKey::Str(s) => self.tpl("object_key_string", Ctx::new().set("key", s), p.span),
                            };
                            let v = self.pattern(value, declare);
                            self.tpl("object_entry", Ctx::new().set("key", k).set("value", v), p.span)
                        }
                        ObjEntry::Spread(_) => String::new(),
                    });
                }
                self.tpl("object", Ctx::new().set("entries", out.join(", ")), p.span)
            }
            _ => self.expr(p).0,
        }
    }

    /// `x = if ...` / `x = match ...` with branches of several lines: x is declared, and the last line
    /// of each branch is assigned to it. Branches of one expression stay a ternary (see `expr`).
    fn assign_branching(&mut self, a: &Assign) -> Option<String> {
        match &a.value.kind {
            ExprKind::If(i) if !single_expr_if(i) => {
                if i.otherwise.is_none() {
                    self.error(codes::BRANCHING_VALUE, i.span, "an if used as a value needs an else branch");
                    return Some(String::new());
                }
            }
            ExprKind::Match(m) if !single_expr_match(m) => {
                let exhaustive = match self.result_arms(m) {
                    Some(arms) => Self::result_arms_exhaustive(&arms),
                    None => m.arms.iter().any(|arm| arm.patterns.iter().any(|p| matches!(p, Pattern::Wildcard(_)))),
                };
                if !exhaustive {
                    self.error(codes::BRANCHING_VALUE, m.span, "a match used as a value needs a '_ ->' arm");
                    return Some(String::new());
                }
            }
            _ => return None,
        }
        let mut out = String::new();
        let target = match &a.target.kind {
            ExprKind::Name(n) if self.lookup_local(n).is_none() && !self.is_member(n) => {
                let name = self.declare_local(n);
                let ty = a.ty.as_ref().map(|t| self.ty(t));
                out.push_str(&self.tpl("declare", Ctx::new().set("name", name.clone()).opt("type", ty), a.span));
                out.push('\n');
                name
            }
            _ => self.expr(&a.target).0,
        };
        let tail = Tail::Assign(target);
        let code = match &a.value.kind {
            ExprKind::If(i) => self.if_stmt(i, &tail),
            ExprKind::Match(m) => self.match_stmt(m, &tail),
            _ => unreachable!(),
        };
        out.push_str(&code);
        Some(out)
    }

    fn if_stmt(&mut self, i: &If, tail: &Tail) -> String {
        let mut out = String::new();
        for (idx, (cond, body)) in i.branches.iter().enumerate() {
            // an else-if condition may not run: it cannot unwrap a Result before the if
            self.hoist_blocked += u32::from(idx > 0);
            let c = self.expr(cond).0;
            self.hoist_blocked -= u32::from(idx > 0);
            let b = self.block(body, tail);
            let key = if idx == 0 { "if" } else { "else_if" };
            out.push_str(&self.tpl(key, Ctx::new().set("cond", c).set("body", b), i.span));
        }
        if let Some(b) = &i.otherwise {
            let body = self.block(b, tail);
            out.push_str(&self.tpl("else", Ctx::new().set("body", body), i.span));
        }
        out
    }

    fn for_stmt(&mut self, f: &For) -> String {
        if f.vars.len() == 2 {
            // a Map object (not a map literal): TypeScript iterates it directly, not with Object.entries
            let is_map = match &f.iter.kind {
                ExprKind::Name(n) => self.frames.iter().rev().find_map(|fr| fr.types.get(n)).is_some_and(
                    |t| matches!(&t.kind, TypeKind::Named { name, .. } if name == "Map"),
                ),
                ExprKind::Call { callee, .. } => matches!(&callee.kind, ExprKind::Name(c) if c == "Map"),
                _ => false,
            };
            let iter = self.expr(&f.iter).0;
            let key = self.declare_local(&f.vars[0].name);
            let value = self.declare_local(&f.vars[1].name);
            if let Some(t) = self.static_type(&f.iter)
                && let Some((k, v)) = map_types(&t)
                && let Some(fr) = self.frames.last_mut()
            {
                fr.types.insert(f.vars[0].name.clone(), k);
                fr.types.insert(f.vars[1].name.clone(), v);
            }
            let body = self.block(&f.body, &Tail::None);
            return self.tpl(
                "for_entries",
                Ctx::new().set("key", key).set("value", value).set("iter", iter).set("body", body).flag("map", is_map),
                f.span,
            );
        }
        if let ExprKind::Binary { op: BinaryOp::Range, left, right } = &f.iter.kind {
            let start = self.expr(left).0;
            let end = self.expr(right).0;
            let var = self.declare_local(&f.vars[0].name);
            if let Some(fr) = self.frames.last_mut() {
                fr.types.insert(f.vars[0].name.clone(), named_type("int", Vec::new(), f.span));
            }
            let body = self.block(&f.body, &Tail::None);
            return self.tpl(
                "for_range",
                Ctx::new().set("var", var).set("start", start).set("end", end).set("body", body),
                f.span,
            );
        }
        let iter = self.expr(&f.iter).0;
        let var = self.declare_local(&f.vars[0].name);
        if let Some(t) = self.static_type(&f.iter).and_then(|t| elem_of(&t))
            && let Some(fr) = self.frames.last_mut()
        {
            fr.types.insert(f.vars[0].name.clone(), t);
        }
        let body = self.block(&f.body, &Tail::None);
        self.tpl("for_in", Ctx::new().set("var", var).set("iter", iter).set("body", body), f.span)
    }

    /// Condition of one match arm: `subject == value`, `subject is T`, joined with `or`.
    fn arm_condition(&mut self, subject: &Expr, patterns: &[Pattern]) -> Option<Expr> {
        let mut cond: Option<Expr> = None;
        for p in patterns {
            let c = match p {
                Pattern::Wildcard(_) => return None,
                Pattern::Value(v) => Expr {
                    span: v.span,
                    kind: ExprKind::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(subject.clone()),
                        right: Box::new(v.clone()),
                    },
                },
                Pattern::Is(t) => {
                    Expr { span: t.span, kind: ExprKind::Is { expr: Box::new(subject.clone()), ty: t.clone() } }
                }
            };
            cond = Some(match cond {
                None => c,
                Some(prev) => Expr {
                    span: prev.span,
                    kind: ExprKind::Binary { op: BinaryOp::Or, left: Box::new(prev), right: Box::new(c) },
                },
            });
        }
        cond
    }

    fn match_stmt(&mut self, m: &Match, tail: &Tail) -> String {
        if let Some(arms) = self.result_arms(m) {
            return self.result_match_stmt(m, arms, tail);
        }
        let mut out = String::new();
        let mut first = true;
        for arm in &m.arms {
            let body = match &arm.body {
                ArmBody::Expr(e) => {
                    let v = self.tail_expr(e, tail);
                    self.tail_value(tail, v, e.span)
                }
                ArmBody::Block(b) => self.block(b, tail),
            };
            match self.arm_condition(&m.subject, &arm.patterns) {
                Some(cond) => {
                    let c = self.expr(&cond).0;
                    let key = if first { "if" } else { "else_if" };
                    out.push_str(&self.tpl(key, Ctx::new().set("cond", c).set("body", body), arm.span));
                    first = false;
                }
                None => {
                    if first {
                        // only a wildcard: run the body directly
                        out.push_str(&body);
                    } else {
                        out.push_str(&self.tpl("else", Ctx::new().set("body", body), arm.span));
                    }
                    break;
                }
            }
        }
        out
    }

    // ---------- expressions ----------

    fn wrap(&self, (text, prec): (String, u8), min: u8) -> String {
        if prec < min { format!("({text})") } else { text }
    }

    fn expr(&mut self, e: &Expr) -> (String, u8) {
        let keep = std::mem::take(&mut self.keep_next);
        if !keep
            && matches!(e.kind, ExprKind::Call { .. } | ExprKind::Unary { op: UnaryOp::Await, .. })
            && let Some(info) = self.result_info(e, false)
        {
            return self.unwrap_result(e, info);
        }
        match &e.kind {
            ExprKind::Name(n) => (self.name_value(n, e.span), P_PRIMARY),
            ExprKind::SelfField(n) => (self.member_access(n, true, e.span), P_POSTFIX),
            ExprKind::This => (self.plain("this", e.span), P_PRIMARY),
            ExprKind::Super => (self.plain("super", e.span), P_PRIMARY),
            ExprKind::Null => (self.plain("null", e.span), P_PRIMARY),
            ExprKind::Bool(b) => (self.plain(if *b { "true" } else { "false" }, e.span), P_PRIMARY),
            ExprKind::Number(n) => {
                let mut text = if self.d.options.number_separators { n.text.clone() } else { n.text.replace('_', "") };
                if n.kind == NumberKind::Bin && !self.d.options.binary_literals {
                    let digits: String = n.text[2..].chars().filter(|c| *c != '_').collect();
                    text = match u128::from_str_radix(&digits, 2) {
                        Ok(v) => format!("0x{v:x}"),
                        Err(_) => text,
                    };
                }
                (text, P_PRIMARY)
            }
            ExprKind::Str(pieces) => (self.string(pieces), P_PRIMARY),
            ExprKind::List(items) => {
                let rendered: Vec<String> = items.iter().map(|i| self.expr(i).0).collect();
                let items = self.join_items(&rendered, 2);
                let text = self.tpl("list", Ctx::new().set("items", items.clone()), e.span);
                (close_on_own_line(text, &items), P_PRIMARY)
            }
            ExprKind::Object(entries) => (self.object(entries, e.span), P_PRIMARY),
            ExprKind::Unary { op, expr } => {
                // `~call()` kept as a Result keeps the call too
                self.keep_next = keep && *op == UnaryOp::Await;
                let operand = self.expr(expr);
                // `-(a ** b)`: TypeScript rejects `-a ** b`
                let pow = matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Pow, .. });
                let operand = self.wrap(operand, if pow { P_POSTFIX } else { P_UNARY });
                let text = match op {
                    UnaryOp::Await => self.tpl("await", Ctx::new().set("expr", operand), e.span),
                    UnaryOp::Not => self.unary_op("not", operand, e.span),
                    UnaryOp::Neg => self.unary_op("neg", operand, e.span),
                    UnaryOp::BitNot => self.unary_op("bit_not", operand, e.span),
                };
                (text, P_UNARY)
            }
            ExprKind::Binary { op, left, right } => self.binary(*op, left, right, e.span),
            ExprKind::Ternary { cond, then, otherwise } => {
                let c = self.expr(cond);
                let c = self.wrap(c, P_TERNARY + 1);
                self.hoist_blocked += 1;
                let t = self.expr(then);
                // a conditional in the first branch keeps its parentheses: in Python's `t if c else o` it would join `o`
                let t = self.wrap(t, P_TERNARY + 1);
                let o = self.expr(otherwise);
                self.hoist_blocked -= 1;
                let o = self.wrap(o, P_TERNARY);
                (self.tpl("ternary", Ctx::new().set("cond", c).set("then", t).set("else", o), e.span), P_TERNARY)
            }
            ExprKind::Is { expr, ty } => {
                let x = self.expr(expr);
                let x = self.wrap(x, P_POSTFIX);
                let t = self.ty(ty);
                let info = self.type_info(Some(ty));
                let text = self.tpl("is", Ctx::new().set("expr", x).set("type", t).value("t", info), e.span);
                let prec = if is_atomic(&text) { P_POSTFIX } else { 6 };
                (text, prec)
            }
            ExprKind::Member { object, name, optional } => {
                self.check_awaited(object);
                self.check_result_member(object, name);
                let callee = std::mem::take(&mut self.callee_member);
                if !*optional && !callee && let Some(text) = self.library_access(object, name, None, &[], e.span) {
                    return (text, P_POSTFIX);
                }
                self.check_project_member(object, name);
                let o = self.expr(object);
                let mut o = self.wrap(o, P_POSTFIX);
                // `(5).toString(2)`: in JavaScript `5.` would start a decimal
                if self.d.options.int_member_parens
                    && matches!(&object.kind, ExprKind::Number(n) if n.kind == NumberKind::Int)
                {
                    o = format!("({o})");
                }
                let key = if *optional { "optional_member" } else { "member" };
                (self.tpl(key, Ctx::new().set("object", o).set("name", &name.name), e.span), P_POSTFIX)
            }
            ExprKind::Index { object, index } => {
                self.check_awaited(object);
                self.check_result_index(object);
                // `m[k]` on a map is `V?` (null when absent): a dialect whose index throws writes a lookup instead
                let (is_map, is_object) = self.map_kind(object);
                let is_map = (is_map || is_object) && !std::mem::take(&mut self.assign_target);
                let o = self.expr(object);
                let o = self.wrap(o, P_POSTFIX);
                let i = self.expr(index).0;
                (self.tpl("index", Ctx::new().set("object", o).set("index", i).flag("map", is_map), e.span), P_POSTFIX)
            }
            ExprKind::Call { callee, args, type_args } => (self.call(callee, args, type_args, e.span), P_POSTFIX),
            ExprKind::ListIf { cond, value } => {
                let c = self.expr(cond).0;
                self.hoist_blocked += 1;
                let v = self.expr(value).0;
                self.hoist_blocked -= 1;
                (self.tpl("list_if", Ctx::new().set("cond", c).set("value", v), e.span), P_PRIMARY)
            }
            ExprKind::Cascade { target, sections } => {
                let t = self.expr(target);
                let t = self.wrap(t, P_POSTFIX);
                let placeholder = self.tpl("cascade_self", Ctx::new(), e.span);
                let secs = self.cascade_sections(sections, placeholder);
                let list = Value::List(secs.into_iter().map(Value::Str).collect());
                let text = self.tpl("cascade", Ctx::new().set("target", t).value("sections", list), e.span);
                (text.trim_end().to_string(), P_POSTFIX)
            }
            ExprKind::CascadeSelf => (self.cascade_self.last().cloned().unwrap_or_default(), P_PRIMARY),
            ExprKind::NonNull(x) => {
                let v = self.expr(x);
                let v = self.wrap(v, P_POSTFIX);
                (self.tpl("non_null", Ctx::new().set("expr", v), e.span), P_POSTFIX)
            }
            ExprKind::Cast { expr, ty } => {
                let x = self.expr(expr);
                let x = self.wrap(x, P_POSTFIX);
                let t = self.ty(ty);
                (self.tpl("cast", Ctx::new().set("expr", x).set("type", t), e.span), 6)
            }
            ExprKind::Spread(x) => {
                let v = self.expr(x).0;
                (self.tpl("spread_arg", Ctx::new().set("expr", v), e.span), P_PRIMARY)
            }
            ExprKind::Lambda(l) => (self.lambda(l), P_LAMBDA),
            ExprKind::If(i) => self.if_value(i),
            ExprKind::Match(m) => self.match_value(m),
        }
    }

    fn unary_op(&mut self, key: &str, operand: String, span: Span) -> String {
        match self.operator(key) {
            Some(t) if t.is_constant() => {
                format!("{}{operand}", t.render(&Value::Map(BTreeMap::new())))
            }
            Some(t) => t.render(&Ctx::new().set("expr", operand).into_value()),
            None => {
                if self.missing_reported.insert(format!("operators.{key}")) {
                    self.error(
                        codes::MISSING_TEMPLATE,
                        span,
                        format!("dialect '{}' has no [operators] entry '{key}'", self.d.name),
                    );
                }
                operand
            }
        }
    }

    fn binary(&mut self, op: BinaryOp, left: &Expr, right: &Expr, span: Span) -> (String, u8) {
        let prec = op.precedence();
        // the right side of and, or, ?? may not run
        let lazy = u32::from(matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce));
        // `f(x) ?? d`, `m[k] ?? d`: a target whose `coalesce` writes the left side twice evaluates it once instead
        let key = if op == BinaryOp::Coalesce && !is_plain_path(left) && self.operator("coalesce_once").is_some() {
            "coalesce_once"
        } else {
            op.key()
        };
        if let Some(h) = self.d.syntax_helpers.get(key) {
            self.helpers.entry(format!("operators.{key}")).or_insert_with(|| h.render(&Value::Map(BTreeMap::new())));
        }
        let Some(t) = self.operator(key) else {
            if self.missing_reported.insert(format!("operators.{}", op.key())) {
                self.error(
                    codes::MISSING_TEMPLATE,
                    span,
                    format!("dialect '{}' has no [operators] entry '{}'", self.d.name, op.key()),
                );
            }
            return (String::new(), prec);
        };
        // the operands of a comparison that are bitwise operations get parentheses: TypeScript (like C)
        // binds `==` tighter than `&`, `^` and `|`
        let operand_prec = if prec == 6 { 11 } else { prec };
        if t.is_constant() {
            let sym = t.render(&Value::Map(BTreeMap::new()));
            let l = self.expr(left);
            let l = self.wrap(l, operand_prec.max(if op == BinaryOp::Pow { prec + 1 } else { prec }));
            self.hoist_blocked += lazy;
            let r = self.expr(right);
            self.hoist_blocked -= lazy;
            let r = self.wrap(r, operand_prec.max(prec + 1));
            return (format!("{l} {sym} {r}"), prec);
        }
        let l = self.expr(left);
        let l_arg = l.0.clone();
        // `left_infix` / `right_infix`: with parentheses only where the operator's precedence needs them
        let l_infix = self.wrap(l.clone(), operand_prec.max(prec + 1));
        let l = self.wrap(l, P_POSTFIX);
        self.hoist_blocked += lazy;
        let r = self.expr(right);
        self.hoist_blocked -= lazy;
        let r_arg = r.0.clone();
        let r_infix = self.wrap(r.clone(), operand_prec.max(prec + 1));
        let r = self.wrap(r, P_POSTFIX);
        // `int` / `double`: both operands are known integers / one is a decimal (Dart's `pow` returns num)
        let (lk, rk) = (self.num_kind(left), self.num_kind(right));
        // `k in m`: `map` = a Map object, `object` = a map literal or a `<K, V>` value (else a list)
        let (is_map, is_object) = if op == BinaryOp::In { self.map_kind(right) } else { (false, false) };
        // `x in s` on a Set of the library (TypeScript: `has`, not `includes`)
        let is_set = op == BinaryOp::In
            && matches!(self.static_type(right).map(|t| receiver_base(&t)), Some(Recv::Base("set")));
        let ctx = Ctx::new()
            .set("left", l)
            .set("right", r)
            // without parentheses, for a template that passes them as arguments: `pow(a + 1, 2)`
            .set("left_arg", l_arg)
            .set("right_arg", r_arg)
            .set("left_infix", l_infix)
            .set("right_infix", r_infix)
            .flag("map", is_map)
            .flag("set", is_set)
            .flag("object", is_object)
            .flag("int", lk == Some(NumKind::Int) && rk == Some(NumKind::Int))
            .flag("double", lk == Some(NumKind::Double) || rk == Some(NumKind::Double))
            // a comparison with `null` (Python: `x is None`)
            .flag("null", matches!(left.kind, ExprKind::Null) || matches!(right.kind, ExprKind::Null));
        let text = t.render(&ctx.into_value());
        let p = if is_atomic(&text) {
            P_POSTFIX
        } else if self.conditional_form(t) {
            P_TERNARY
        } else {
            prec
        };
        (text, p)
    }

    /// An operator written in the shape of the dialect's conditional expression (Python's `??`: `a if a is not None
    /// else b`) binds like it: it needs parentheses wherever a conditional does.
    fn conditional_form(&self, t: &Template) -> bool {
        let Some(ternary) = self.d.syntax("ternary") else { return false };
        let shape = ternary.literals();
        let mine = t.literals().join(" ");
        shape.len() >= 2 && shape.iter().all(|w| mine.contains(w.as_str()))
    }

    /// A member's name as the source writes it (`trimStart`), when naming rewrote it for the target
    /// (`trim_start`): the standard library is looked up by the Nuclear name.
    fn src_name(&self, id: &Ident) -> String {
        self.src_text(id.span).unwrap_or_else(|| id.name.clone())
    }

    /// The identifier a span covers in the source, if it is one.
    fn src_text(&self, span: Span) -> Option<String> {
        let (a, b) = (span.start, span.end);
        match self.source.get(a..b) {
            Some(t) if b > a && t.chars().all(|c| c.is_alphanumeric() || c == '_') => Some(t.to_string()),
            _ => None,
        }
    }

    /// A member of Nuclear's standard library (spec/library.md) on a value of a base type, or on one of its
    /// objects (`Math.sqrt(x)`), rendered with the dialect's `[library.<base>]` template. `None`: not a library
    /// member (an ordinary member access is written), after N0114 for a target's name on a base type and N0115
    /// for a receiver without a known type whose member differs between bases.
    fn library_access(&mut self, object: &Expr, name: &Ident, args: Option<&[Arg]>, type_args: &[Type], span: Span) -> Option<String> {
        let member_name = self.src_name(name);
        if let ExprKind::Name(obj) = &object.kind
            && self.lookup_local(obj).is_none()
            && !self.is_member(obj)
            && !self.decl_names.contains_key(obj)
        {
            if library::OBJECTS.contains(&obj.as_str()) {
                return self.library_static(obj, &member_name, args, type_args, name.span);
            }
            if let Some((_, _, fix)) = library::WRONG_STATIC.iter().find(|(o, n, _)| o == obj && *n == member_name) {
                self.error(codes::LIBRARY_NAME, name.span, format!("'{obj}.{}' is not Nuclear: use {fix}", member_name));
                return None;
            }
        }
        // a class or module (`GetOne.exec`, `Theme.of`) or a cascade target: not a value of a base type
        match &object.kind {
            ExprKind::Name(obj) if self.lookup_local(obj).is_none() && is_pascal(obj) => return None,
            ExprKind::CascadeSelf | ExprKind::Super => return None,
            _ => {}
        }
        let is_call = args.is_some();
        let recv_type = self.static_type(object);
        let base = match recv_type.as_ref().map(receiver_base) {
            Some(Recv::Other) => return None,
            Some(Recv::Base(b)) => b,
            Some(Recv::Unknown) | None => {
                // a property of a value without a type is the value's own (`m.text` of a chat message): only a
                // method call whose name belongs to one base, or is written the same for all of them, is the library's
                if !is_call {
                    return None;
                }
                let candidates = library::bases_with(&member_name);
                if candidates.is_empty() {
                    // `toString` exists on every value
                    library::member_of("any", &member_name)?;
                    "any"
                } else {
                    let mut templates = Vec::new();
                    for b in &candidates {
                        let m = library::member(b, &member_name)?;
                        if m.property == is_call {
                            return None;
                        }
                        templates.push(self.lib_entry(m.base, &member_name).map(|e| e.template));
                    }
                    if templates[0].is_some() && templates.iter().all(|t| t == &templates[0]) {
                        candidates[0]
                    } else {
                        let shown = match &object.kind {
                            ExprKind::Name(n) | ExprKind::SelfField(n) => format!("'{n}'"),
                            _ => "the value before the dot".to_string(),
                        };
                        self.error(
                            codes::RECEIVER_TYPE,
                            object.span,
                            format!(
                                "the type of {shown} is not known, and '{}' is written differently for {} in this dialect: declare its type",
                                member_name,
                                candidates.join(", ")
                            ),
                        );
                        return None;
                    }
                }
            }
        };
        // the dialect's own extension of the base type (`"key".tr` in the mobile platform): a property when it
        // is used as one, a method when it is called
        if library::member(base, &member_name).is_none()
            && let Some(entry) = self.lib_entry(base, &member_name)
        {
            let recv = self.expr(object);
            let recv = self.wrap(recv, P_POSTFIX);
            let key = format!("{base}.{member_name}");
            return Some(self.library_render(&key, &entry, Some(recv), None, Vec::new(), args.unwrap_or(&[]), type_args));
        }
        let Some(m) = library::member(base, &member_name) else {
            // a dialect's errors extend Error (status, code...): only `message` is the library's
            if base == "error" {
                return None;
            }
            let msg = match library::wrong(base, &member_name) {
                Some(fix) => format!("'{}' is not a member of Nuclear's {base}: use {fix}", member_name),
                None => format!("'{}' is not a member of Nuclear's {base} (nuclear help library)", member_name),
            };
            self.error(codes::LIBRARY_NAME, name.span, msg);
            return None;
        };
        if m.property && is_call {
            self.error(codes::LIBRARY_NAME, name.span, format!("'{}' is a property of {base}: write it without parentheses", member_name));
            return None;
        }
        if !m.property && !is_call {
            self.error(codes::LIBRARY_NAME, name.span, format!("'{0}' is a method of {base}: write {0}(...)", member_name));
            return None;
        }
        let Some(entry) = self.lib_entry(m.base, &member_name) else {
            self.error(codes::MISSING_TEMPLATE, span, format!("the dialect has no template for library.{}.{}", m.base, member_name));
            return None;
        };
        let recv = self.expr(object);
        let mut recv = self.wrap(recv, P_POSTFIX);
        // `(5).toFixed(2)`: in JavaScript `5.` would start a decimal
        if self.d.options.int_member_parens && matches!(&object.kind, ExprKind::Number(n) if n.kind == NumberKind::Int) {
            recv = format!("({recv})");
        }
        let elem = recv_type.as_ref().and_then(elem_of);
        // the types of a callback's parameters: the element (reduce: the accumulator first; replaceAll: a Match)
        let hints: Vec<Type> = match (base, member_name.as_str()) {
            ("list", "reduce") => {
                let acc = args.and_then(|a| a.first()).and_then(|a| self.static_type(&a.value));
                acc.into_iter().chain(elem.clone()).collect()
            }
            ("string", "replaceAll") => vec![named_type("Match", Vec::new(), span)],
            _ => elem.clone().into_iter().collect(),
        };
        let key = format!("{}.{}", m.base, member_name);
        Some(self.library_render(&key, &entry, Some(recv), elem.as_ref(), hints, args.unwrap_or(&[]), type_args))
    }

    /// `Math.sqrt(x)`, `Json.encode(v)`, `DateTime.now()`, and the constructors `Regex(...)`, `Duration(...)`,
    /// `Set<int>()` (`new`).
    fn library_static(&mut self, obj: &str, name: &str, args: Option<&[Arg]>, type_args: &[Type], span: Span) -> Option<String> {
        let Some(m) = library::member_of(obj, name) else {
            let fix = library::WRONG_STATIC.iter().find(|(o, n, _)| *o == obj && *n == name).map(|(_, _, f)| *f);
            let msg = match fix {
                Some(fix) => format!("'{obj}.{name}' is not Nuclear: use {fix}"),
                None => format!("'{obj}.{name}' is not Nuclear (nuclear help library)"),
            };
            self.error(codes::LIBRARY_NAME, span, msg);
            return None;
        };
        if m.property && args.is_some() {
            self.error(codes::LIBRARY_NAME, span, format!("'{obj}.{name}' is a property: write it without parentheses"));
            return None;
        }
        if !m.property && args.is_none() {
            self.error(codes::LIBRARY_NAME, span, format!("'{obj}.{name}' is a method: write {obj}.{name}(...)"));
            return None;
        }
        let Some(entry) = self.lib_entry(obj, name) else {
            self.error(codes::MISSING_TEMPLATE, span, format!("the dialect has no template for library.{obj}.{name}"));
            return None;
        };
        let key = format!("{obj}.{name}");
        Some(self.library_render(&key, &entry, None, None, Vec::new(), args.unwrap_or(&[]), type_args))
    }

    fn lib_entry(&self, base: &str, name: &str) -> Option<nuclear_dialect::model::LibMember> {
        self.d.library.get(base)?.get(name).map(|e| e.value.clone())
    }

    /// Renders a library template: `recv`, the arguments (`args`, `argN`, `named.<name>`), `elem`, `type_args`
    /// and, for the first expression lambda among the arguments, `fn` (`param`, `params`, `p0`, `p1`, `body`).
    #[allow(clippy::too_many_arguments)]
    fn library_render(
        &mut self,
        key: &str,
        entry: &nuclear_dialect::model::LibMember,
        recv: Option<String>,
        elem: Option<&Type>,
        hints: Vec<Type>,
        args: &[Arg],
        type_args: &[Type],
    ) -> String {
        let mut ctx = Ctx::new();
        if let Some(r) = recv {
            ctx = ctx.set("recv", r);
        }
        let mut positional = Vec::new();
        let mut named: BTreeMap<String, Value> = BTreeMap::new();
        let mut named_texts = Vec::new();
        let mut fn_parts: Option<Value> = None;
        let mut arg_bases = Vec::new();
        for a in args {
            let lambda = match &a.value.kind {
                ExprKind::Lambda(l) if fn_parts.is_none() => Some(l),
                _ => None,
            };
            if lambda.is_some() {
                self.lambda_hints = hints.clone();
            }
            let text = self.expr(&a.value).0;
            self.lambda_hints.clear();
            if let Some(l) = lambda
                && let LambdaBody::Expr(body) = &l.body
            {
                fn_parts = Some(self.lambda_parts(l, body, &hints));
            }
            match &a.name {
                Some(n) => {
                    // a literal flag (`ignoreCase = true`) is a boolean, so `{{#named.ignoreCase}}` works
                    let value = match a.value.kind {
                        ExprKind::Bool(b) => Value::Bool(b),
                        _ => Value::str(&text),
                    };
                    // by the name as written: naming may have rewritten `ignoreCase` as `ignore_case`
                    let name = self.src_name(n);
                    named_texts.push(if self.d.options.named_args_native {
                        self.tpl("named_arg", Ctx::new().set("name", &name).set("value", &text), a.span)
                    } else {
                        format!("{name}: {text}")
                    });
                    named.insert(name, value);
                }
                None => {
                    let base = match self.static_type(&a.value).map(|t| receiver_base(&t)) {
                        Some(Recv::Base(b)) => Some(b),
                        _ => None,
                    };
                    arg_bases.push(base);
                    positional.push(text);
                }
            }
        }
        ctx = ctx.set("args", positional.join(", ")).flag("has_args", !positional.is_empty()).set("n", positional.len().to_string());
        for (i, t) in positional.iter().enumerate() {
            ctx = ctx.set(&format!("arg{i}"), t.clone());
            // `arg0_is.regex`: what the argument is, for a member that takes a string or a Regex
            let mut is = BTreeMap::new();
            if let Some(b) = arg_bases[i] {
                is.insert(b.to_string(), Value::Bool(true));
            }
            ctx = ctx.value(&format!("arg{i}_is"), Value::Map(is));
        }
        ctx = ctx.value("named", Value::Map(named));
        if let Some(f) = fn_parts {
            ctx = ctx.value("fn", f);
        }
        if let Some(e) = elem {
            let t = self.ty(e);
            ctx = ctx.set("elem", t);
        }
        let targs: Vec<String> = type_args.iter().map(|t| self.ty(t)).collect();
        ctx = ctx.set("type_args", targs.join(", "));
        // `call_args`: every argument as the target writes a call (`days: 1, hours: 2` in Dart, `days=1` in Python)
        let call_args: Vec<String> = positional.iter().cloned().chain(named_texts).collect();
        ctx = ctx.set("call_args", call_args.join(", "));
        if let Some(h) = &entry.helper
            && !self.helpers.contains_key(key)
        {
            let text = h.render(&Value::Map(BTreeMap::new()));
            self.helpers.insert(key.to_string(), text);
        }
        entry.template.render(&ctx.into_value())
    }

    /// The parts of an expression lambda for a library template (Python writes a comprehension with them):
    /// its parameters' emitted names and its body. The body was already checked when the lambda was rendered.
    fn lambda_parts(&mut self, l: &Lambda, body: &Expr, hints: &[Type]) -> Value {
        let reported = self.diags.len();
        self.frames.push(Frame::default());
        let mut names = Vec::new();
        for (i, p) in l.params.iter().enumerate() {
            let n = self.declare_local(&p.name.name);
            if let Some(t) = p.ty.clone().or_else(|| hints.get(i).cloned())
                && let Some(f) = self.frames.last_mut()
            {
                f.types.insert(p.name.name.clone(), t);
            }
            names.push(n);
        }
        self.hoist_blocked += 1;
        let text = self.expr(body).0;
        self.hoist_blocked -= 1;
        self.frames.pop();
        self.diags.truncate(reported);
        let mut m = BTreeMap::new();
        m.insert("param".to_string(), Value::str(names.first().cloned().unwrap_or_default()));
        m.insert("params".to_string(), Value::str(names.join(", ")));
        for (i, n) in names.iter().enumerate() {
            m.insert(format!("p{i}"), Value::str(n));
        }
        m.insert("body".to_string(), Value::str(text));
        Value::Map(m)
    }

    /// The static type of an expression, when it is known: literals, typed or constructed variables, fields
    /// and methods of the program's classes, the members of the standard library, awaited calls.
    fn static_type(&self, e: &Expr) -> Option<Type> {
        let sp = e.span;
        match &e.kind {
            ExprKind::Str(_) => Some(named_type("string", Vec::new(), sp)),
            ExprKind::Number(n) => Some(named_type(if n.kind == NumberKind::Float { "double" } else { "int" }, Vec::new(), sp)),
            ExprKind::Bool(_) => Some(named_type("boolean", Vec::new(), sp)),
            ExprKind::List(items) => {
                let elem = items
                    .iter()
                    .filter(|i| !matches!(i.kind, ExprKind::Spread(_)))
                    .find_map(|i| self.static_type(i))
                    .unwrap_or_else(|| named_type("dynamic", Vec::new(), sp));
                Some(Type { span: sp, kind: TypeKind::List(Box::new(elem)) })
            }
            ExprKind::Object(_) => {
                let any = || Box::new(named_type("dynamic", Vec::new(), sp));
                Some(Type { span: sp, kind: TypeKind::Map(any(), any()) })
            }
            ExprKind::Name(n) => self.var_type(n),
            // `this` inside `extension X on String` is a String
            ExprKind::This => {
                let m = self.module.as_ref()?;
                let info = self.decl_info(&m.name)?;
                match (&info.kind, info.bases.as_slice()) {
                    (DeclKind::Kind(k), [on]) if k == "extension" => Some(named_type(on, Vec::new(), sp)),
                    _ => None,
                }
            }
            ExprKind::SelfField(n) => {
                let m = self.module.as_ref()?;
                self.decl_info(&m.name).and_then(|i| i.field_types.get(n).cloned())
            }
            ExprKind::Member { object, name, .. } => {
                if let ExprKind::Name(obj) = &object.kind
                    && self.lookup_local(obj).is_none()
                    && !self.is_member(obj)
                    && library::OBJECTS.contains(&obj.as_str())
                {
                    let m = library::member_of(obj, &self.src_name(name)).filter(|m| m.property)?;
                    return self.lib_ret(m.returns, None, sp);
                }
                let t = self.static_type(object)?;
                // `r.value` / `r.error` of a Result<E, T>
                if let TypeKind::Named { name: tn, args } = &strip_nullable(t.clone()).kind
                    && tn == "Result"
                    && let [e, v] = args.as_slice()
                {
                    return match name.name.as_str() {
                        "value" => Some(v.clone()),
                        "error" => Some(e.clone()),
                        "isOk" | "isErr" => Some(named_type("boolean", Vec::new(), sp)),
                        _ => None,
                    };
                }
                if let Some(cls) = class_name(&t)
                    && let Some(info) = self.decl_info(&cls)
                {
                    return info.field_types.get(&name.name).cloned();
                }
                let Recv::Base(base) = receiver_base(&t) else { return None };
                let m = library::member(base, &self.src_name(name)).filter(|m| m.property)?;
                self.lib_ret(m.returns, Some(&t), sp)
            }
            ExprKind::Call { callee, args, type_args } => match &callee.kind {
                ExprKind::Name(c) if self.lookup_local(c).is_none() => {
                    // `inject<Repo>()`: a function of the runtime whose only type argument is what it gives
                    if let [t] = type_args.as_slice()
                        && args.is_empty()
                        && !is_pascal(c)
                        && self.decl_info(c).is_none()
                        && !self.is_member(c)
                    {
                        return Some(t.clone());
                    }
                    if is_pascal(c) {
                        if library::OBJECTS.contains(&c.as_str()) {
                            let m = library::member_of(c, "new")?;
                            return self.lib_ret(m.returns, None, sp);
                        }
                        return (self.decl_info(c).is_some() || self.d.defs.contains_key(c.as_str()))
                            .then(|| named_type(c, Vec::new(), sp));
                    }
                    // `env("PORT")`: a function of the type definitions returns what they declare
                    if self.decl_info(c).is_none() && !self.module.as_ref().is_some_and(|m| m.members.contains(c)) {
                        return self.def_function_ret(c);
                    }
                    let sig = match self.decl_info(c) {
                        Some(info) => info.sig.clone(),
                        None => {
                            let m = self.module.as_ref().filter(|m| m.members.contains(c))?;
                            self.sig_of_method(self.decl_info(&m.name)?, c)?
                        }
                    };
                    let ret = sig.ret.clone()?;
                    Some(if sig.is_async { named_type("Future", vec![ret], sp) } else { ret })
                }
                ExprKind::Member { object, name, .. } => {
                    if let ExprKind::Name(obj) = &object.kind
                        && self.lookup_local(obj).is_none()
                        && !self.is_member(obj)
                        && library::OBJECTS.contains(&obj.as_str())
                    {
                        let m = library::member_of(obj, &self.src_name(name)).filter(|m| !m.property)?;
                        return self.lib_ret(m.returns, None, sp);
                    }
                    let Some(t) = self.static_type(object) else {
                        return self.only_base_ret(&self.src_name(name), false, sp);
                    };
                    if let Some(cls) = class_name(&t)
                        && let Some(info) = self.decl_info(&cls)
                    {
                        let sig = self.sig_of_method(info, &name.name)?;
                        let ret = sig.ret.clone()?;
                        return Some(if sig.is_async { named_type("Future", vec![ret], sp) } else { ret });
                    }
                    let base = match receiver_base(&t) {
                        Recv::Base(b) => b,
                        Recv::Unknown => return self.only_base_ret(&self.src_name(name), false, sp),
                        Recv::Other => return None,
                    };
                    let m = library::member(base, &self.src_name(name)).filter(|m| !m.property)?;
                    if m.returns == library::Ret::Any && self.src_name(name) == "reduce" {
                        return args.first().and_then(|a| self.static_type(&a.value));
                    }
                    self.lib_ret(m.returns, Some(&t), sp)
                }
                _ => None,
            },
            ExprKind::Index { object, .. } => {
                let t = self.static_type(object)?;
                let t = strip_nullable(t);
                match &t.kind {
                    TypeKind::List(inner) => Some((**inner).clone()),
                    TypeKind::Map(_, v) => Some(Type { span: sp, kind: TypeKind::Nullable(v.clone()) }),
                    TypeKind::Named { name, args } => match (name.as_str(), args.as_slice()) {
                        ("string" | "String", _) => Some(named_type("string", Vec::new(), sp)),
                        ("Bytes", _) => Some(named_type("int", Vec::new(), sp)),
                        ("Match", _) => Some(Type { span: sp, kind: TypeKind::Nullable(Box::new(named_type("string", Vec::new(), sp))) }),
                        ("List", [a]) => Some(a.clone()),
                        ("Map", [_, v]) => Some(Type { span: sp, kind: TypeKind::Nullable(Box::new(v.clone())) }),
                        _ => None,
                    },
                    _ => None,
                }
            }
            ExprKind::Unary { op: UnaryOp::Await, expr } => {
                let t = self.static_type(expr)?;
                match &t.kind {
                    TypeKind::Named { name, args } if (name == "Future" || name == "Promise") && args.len() == 1 => Some(args[0].clone()),
                    _ => Some(t),
                }
            }
            ExprKind::Unary { op: UnaryOp::Neg | UnaryOp::BitNot, expr } => self.static_type(expr),
            ExprKind::Unary { op: UnaryOp::Not, .. } | ExprKind::Is { .. } => Some(named_type("boolean", Vec::new(), sp)),
            ExprKind::NonNull(inner) => self.static_type(inner).map(strip_nullable),
            ExprKind::Cast { ty, .. } => Some(ty.clone()),
            ExprKind::Ternary { then, .. } => self.static_type(then),
            ExprKind::Binary { op, left, right } => match op {
                BinaryOp::Eq
                | BinaryOp::NotEq
                | BinaryOp::Lt
                | BinaryOp::Gt
                | BinaryOp::LtEq
                | BinaryOp::GtEq
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::In => Some(named_type("boolean", Vec::new(), sp)),
                BinaryOp::Range => Some(Type { span: sp, kind: TypeKind::List(Box::new(named_type("int", Vec::new(), sp))) }),
                BinaryOp::Coalesce => self.static_type(right).or_else(|| self.static_type(left).map(strip_nullable)),
                BinaryOp::Add
                    if [left, right].iter().any(|x| matches!(self.static_type(x).map(|t| receiver_base(&t)), Some(Recv::Base("string")))) =>
                {
                    Some(named_type("string", Vec::new(), sp))
                }
                _ => match self.num_kind(e) {
                    Some(NumKind::Int) => Some(named_type("int", Vec::new(), sp)),
                    Some(NumKind::Double) => Some(named_type("double", Vec::new(), sp)),
                    None => None,
                },
            },
            _ => None,
        }
    }

    /// N0222: a member or an index of what an asynchronous call returns, without `~` (`books.exec(a).length`): the value
    /// is not there yet, so the target reads a promise or a coroutine.
    fn check_awaited(&mut self, object: &Expr) {
        // `found = books.all()` then `found.length`: the variable holds the call not awaited
        if let ExprKind::Name(n) = &object.kind
            && self.lookup_local(n).is_some()
            && let Some(t) = self.static_type(object)
            && let TypeKind::Named { name, args } = &strip_nullable(t).kind
            && name == "Future"
            && args.len() == 1
        {
            self.diags.push(Diagnostic::error(
                codes::NOT_AWAITED,
                object.span,
                format!("'{n}' holds an asynchronous call that was not awaited: its value is there only once awaited; write `{n} = ~...` where it is assigned"),
            ));
            return;
        }
        let ExprKind::Call { callee, .. } = &object.kind else { return };
        let (sig, shown) = match &callee.kind {
            ExprKind::Member { object: recv, name, .. } => {
                let Some(t) = self.static_type(recv) else { return };
                let TypeKind::Named { name: tn, .. } = &strip_nullable(t).kind else { return };
                let Some(info) = self.decl_info(tn) else { return };
                (self.sig_of_method(info, &name.name), name.name.clone())
            }
            ExprKind::Name(f) => (self.decl_info(f).map(|i| i.sig.clone()), f.clone()),
            _ => return,
        };
        if sig.is_some_and(|s| s.is_async) {
            let call = self.src_text_any(object.span);
            self.diags.push(
                Diagnostic::error(
                    codes::NOT_AWAITED,
                    object.span,
                    format!("'{shown}' is asynchronous: its value is there only once awaited; write `(~{call})`"),
                )
                .with_fix(object.span, format!("(~{call})")),
            );
        }
    }

    /// N0114: a `Result` has `value`, `error`, `isOk` and `isErr` only (`found.length` of a `Result<E, T[]>`).
    fn check_result_member(&mut self, object: &Expr, name: &Ident) {
        // inside a function that returns a Result a call's Result unwraps by itself (`findUser(id).name`)
        let unwraps = self.fn_ctx.last().is_some_and(|c| c.result.is_some()) && !self.cur_kind.is_some_and(|k| k.keep_results);
        if unwraps && matches!(object.kind, ExprKind::Call { .. } | ExprKind::Unary { op: UnaryOp::Await, .. }) {
            return;
        }
        let Some(t) = self.static_type(object) else { return };
        let TypeKind::Named { name: tn, args } = &strip_nullable(t).kind else { return };
        if tn != "Result" || args.len() != 2 || matches!(self.src_name(name).as_str(), "value" | "error" | "isOk" | "isErr") {
            return;
        }
        let shown = self.src_name(name);
        let recv = self.src_text_any(object.span);
        let fix = format!("{recv}.value.{shown}");
        self.diags.push(
            Diagnostic::error(
                codes::LIBRARY_NAME,
                name.span,
                format!("'{shown}' is not a member of a Result (it has value, error, isOk and isErr): the value is `{fix}`"),
            )
            .with_fix(Span::new(object.span.start, name.span.end), fix),
        );
    }

    /// N0114: an index of a kept `Result` (`found[0]` of a `Result<E, T[]>`): the list is its `value`.
    fn check_result_index(&mut self, object: &Expr) {
        let Some(t) = self.static_type(object) else { return };
        if !matches!(&strip_nullable(t).kind, TypeKind::Named { name, args } if name == "Result" && args.len() == 2) {
            return;
        }
        let recv = self.src_text_any(object.span);
        self.diags.push(
            Diagnostic::error(codes::LIBRARY_NAME, object.span, format!("a Result has no index: the list is `{recv}.value`"))
                .with_fix(object.span, format!("{recv}.value")),
        );
    }

    /// The source text a span covers (any text, unlike `src_text`).
    fn src_text_any(&self, span: Span) -> String {
        self.source.get(span.start..span.end).unwrap_or("...").to_string()
    }

    /// The signature of a method of a declaration; an alias (`list() = ListTasks.exec`) takes its target's.
    fn sig_of_method(&self, info: &DeclInfo, name: &str) -> Option<Sig> {
        match info.method_sigs.get(name) {
            Some(sig) if sig.ret.is_some() => Some(sig.clone()),
            own => {
                let Some((module, method)) = info.aliases.get(name) else { return own.cloned() };
                let target = self.decl_info(module)?.method_sigs.get(method)?.clone();
                Some(Sig { ret: target.ret, is_async: target.is_async || own.is_some_and(|s| s.is_async) })
            }
        }
    }

    /// What a member of an unknown receiver gives when the name belongs to one base type only, and its result
    /// does not depend on the receiver (`x.split(",")` is a `string[]`).
    fn only_base_ret(&self, name: &str, property: bool, sp: Span) -> Option<Type> {
        let bases = library::bases_with(name);
        let [base] = bases.as_slice() else { return None };
        let m = library::member(base, name).filter(|m| m.property == property)?;
        self.lib_ret(m.returns, None, sp)
    }

    /// The type a library member gives, from the receiver's type.
    fn lib_ret(&self, r: library::Ret, recv: Option<&Type>, sp: Span) -> Option<Type> {
        use library::Ret;
        let t = |n: &str| named_type(n, Vec::new(), sp);
        let list = |inner: Type| Type { span: sp, kind: TypeKind::List(Box::new(inner)) };
        let opt = |inner: Type| Type { span: sp, kind: TypeKind::Nullable(Box::new(inner)) };
        let recv_t = recv.cloned().map(strip_nullable);
        let elem = recv_t.as_ref().and_then(elem_of);
        Some(match r {
            Ret::Str => t("string"),
            Ret::StrOpt => opt(t("string")),
            Ret::Int => t("int"),
            Ret::IntOpt => opt(t("int")),
            Ret::Double => t("double"),
            Ret::DoubleOpt => opt(t("double")),
            Ret::Bool => t("boolean"),
            Ret::Void => t("void"),
            Ret::Same => recv_t?,
            Ret::Elem => elem?,
            Ret::ElemOpt => opt(elem?),
            Ret::ListElem => list(elem?),
            Ret::ListStr => list(t("string")),
            Ret::ListInt => list(t("int")),
            Ret::ListAny => list(t("dynamic")),
            Ret::Keys => list(map_types(recv_t.as_ref()?)?.0),
            Ret::Values => list(map_types(recv_t.as_ref()?)?.1),
            Ret::GroupMap => Type { span: sp, kind: TypeKind::Map(Box::new(t("dynamic")), Box::new(list(elem?))) },
            Ret::Num => match recv_t.as_ref().map(receiver_base) {
                Some(Recv::Base("list")) => elem?,
                Some(Recv::Base("int")) => t("int"),
                _ => t("double"),
            },
            Ret::MatchOpt => opt(t("Match")),
            Ret::ListMatch => list(t("Match")),
            Ret::DateTime => t("DateTime"),
            Ret::DateTimeOpt => opt(t("DateTime")),
            Ret::Duration => t("Duration"),
            Ret::Bytes => t("Bytes"),
            Ret::Regex => t("Regex"),
            Ret::SetAny => named_type("Set", vec![t("dynamic")], sp),
            Ret::Error => t("Error"),
            Ret::Any => return None,
        })
    }

    fn map_kind(&self, e: &Expr) -> (bool, bool) {
        let ty = match &e.kind {
            ExprKind::Object(_) => return (false, true),
            ExprKind::Call { callee, .. } if matches!(&callee.kind, ExprKind::Name(c) if c == "Map") => return (true, false),
            ExprKind::Name(n) => self.frames.iter().rev().find_map(|f| f.types.get(n)).cloned().or_else(|| {
                let m = self.module.as_ref()?;
                self.decl_info(&m.name)?.field_types.get(n).cloned()
            }),
            _ => None,
        };
        let mut ty = ty;
        if let Some(Type { kind: TypeKind::Nullable(inner), .. }) = &ty {
            ty = Some((**inner).clone());
        }
        match ty.map(|t| t.kind) {
            Some(TypeKind::Map(..)) => (false, true),
            Some(TypeKind::Named { name, .. }) if name == "Map" => (true, false),
            _ => (false, false),
        }
    }

    /// Whether an expression is surely an integer or a decimal, from literals, typed variables and
    /// arithmetic between them. `None` when it is not known.
    fn num_kind(&self, e: &Expr) -> Option<NumKind> {
        match &e.kind {
            ExprKind::Number(n) => Some(if n.kind == NumberKind::Float { NumKind::Double } else { NumKind::Int }),
            ExprKind::Name(n) | ExprKind::SelfField(n) => {
                // a local, then a field of the module being written
                let local = self.frames.iter().rev().find_map(|f| f.types.get(n)).cloned();
                let ty = local.or_else(|| {
                    let m = self.module.as_ref()?;
                    self.decl_info(&m.name)?.field_types.get(n).cloned()
                })?;
                num_kind_of(&ty)
            }
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(f) if self.lookup_local(f).is_none() => num_kind_of(self.decl_info(f)?.sig.ret.as_ref()?),
                _ => None,
            },
            ExprKind::Unary { op: UnaryOp::Neg | UnaryOp::BitNot, expr } => self.num_kind(expr),
            ExprKind::Binary { op, left, right } => {
                let (l, r) = (self.num_kind(left), self.num_kind(right));
                match op {
                    BinaryOp::Div => Some(NumKind::Double),
                    BinaryOp::IntDiv
                    | BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::Shl
                    | BinaryOp::Shr
                    | BinaryOp::UShr => Some(NumKind::Int),
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Rem | BinaryOp::Pow => {
                        if l == Some(NumKind::Double) || r == Some(NumKind::Double) {
                            Some(NumKind::Double)
                        } else if l == Some(NumKind::Int) && r == Some(NumKind::Int) {
                            Some(NumKind::Int)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn name_value(&mut self, n: &str, span: Span) -> String {
        if let Some(local) = self.lookup_local(n) {
            return local;
        }
        if let Some(t) = self.provided.get(n) {
            let name = self.decl_names.get(n).cloned().unwrap_or_else(|| n.to_string());
            return t.render(&Ctx::new().set("name", name).into_value());
        }
        if self.is_member(n) {
            return self.member_access(n, false, span);
        }
        if let Some(out) = self.decl_names.get(n) {
            return out.clone();
        }
        self.check_defined(n, span);
        if !self.known_symbol(n) || self.d.auto_imports.contains_key(n) {
            self.check_plugin_name(n, span);
            self.refs.insert(n.to_string());
        }
        n.to_string()
    }

    /// N0105 for a variable of a block that already ended, N0106 for a camelCase name that nothing
    /// defines. Not reported when the file or module brings names this file cannot see (a whole-module
    /// `use`, a base declared elsewhere).
    fn check_defined(&mut self, n: &str, span: Span) {
        if self.known_symbol(n)
            || self.d.types.contains_key(n)
            || !n.starts_with(|c: char| c.is_lowercase() || c == '_')
        {
            return;
        }
        if self.frames.iter().any(|f| f.gone.contains(n)) {
            self.error(
                codes::OUT_OF_SCOPE,
                span,
                format!("'{n}' was created inside a block (if, for, while, match, try) and does not exist here; to take a value out of an if or match, use it as a value: {n} = if ..."),
            );
            return;
        }
        if self.open_imports || self.module.as_ref().is_some_and(|m| m.open) {
            return;
        }
        let candidates: Vec<String> = self
            .frames
            .iter()
            .flat_map(|f| f.vars.keys().cloned())
            .chain(self.module.iter().flat_map(|m| m.members.iter().cloned()))
            .collect();
        let hint = closest(n, candidates.iter().map(String::as_str))
            .map(|c| format!("; did you mean '{c}'?"))
            .unwrap_or_default();
        self.error(codes::UNDEFINED_NAME, span, format!("'{n}' is not defined{hint}"));
    }

    /// F6 of specs/001: a class of the type definitions called with a named argument it does not have
    /// (N0219), or a PascalCase call nothing defines (N0220). Only in a project (a single file cannot see the
    /// classes of the files beside it) and with type definitions, which list every class the target can see.
    fn check_construct(&mut self, n: &str, args: &[Arg], span: Span) {
        if self.project.is_none()
            || self.d.typedefs.packages.is_empty()
            || self.open_imports
            || self.module.as_ref().is_some_and(|m| m.open)
        {
            return;
        }
        let local = self.decl_names.contains_key(n) || self.decls.contains_key(n) || self.imported.contains(n);
        // the class imported under that name must be the one of the definitions (a dialect can import another)
        let same = match self.d.auto_imports.get(n) {
            Some(m) => self.d.def_libraries.get(n).is_some_and(|libs| libs.contains(&m.value)),
            None => true,
        };
        // with named_args = "object" (TypeScript) the named arguments are one options object, not the parameters
        let native = self.d.options.named_args_native;
        if let Some(params) = self.d.defs.get(n).and_then(|c| c.params.as_ref()).filter(|_| !local && same && native) {
            for a in args {
                let Some(name) = &a.name else { continue };
                if params.iter().any(|p| p == &name.name) {
                    continue;
                }
                let hint = closest(&name.name, params.iter().map(String::as_str))
                    .map(|c| format!("did you mean '{c}'? "))
                    .unwrap_or_default();
                let msg = format!("{n}(...) has no parameter '{}'; {hint}its parameters: {}", name.name, params.join(", "));
                self.error(codes::UNKNOWN_NAMED_ARG, name.span, msg);
            }
            return;
        }
        if local || self.known_symbol(n) || self.d.types.contains_key(n) || self.d.defs.contains_key(n) {
            return;
        }
        let names = self.decl_names.keys().chain(self.d.auto_imports.keys()).chain(self.d.globals.iter());
        let hint = closest(n, names.map(String::as_str)).map(|c| format!("; did you mean '{c}'?")).unwrap_or_default();
        self.error(
            codes::UNKNOWN_CONSTRUCTOR,
            span,
            format!("'{n}' is not a class of this project nor of the dialect's type definitions{hint} (nuclear defs search)"),
        );
    }

    /// N0304: a kind or a name of a plugin the project's `config.nuc` does not enable.
    fn plugin_hint(&self, plugin: &str) -> String {
        let file = self.d.project.manifest.as_ref().map_or("config.nuc", |m| m.file.as_str());
        format!("it belongs to the plugin '{plugin}': enable it in {file} with plugins({plugin} = true)")
    }

    fn disabled_plugin_of_kind(&self, kind: &str) -> Option<String> {
        let enabled = self.project?.plugins.as_ref()?;
        let p = self.d.project.plugins.iter().find(|p| p.kinds.iter().any(|k| k == kind))?;
        (!enabled.contains(&p.name)).then(|| p.name.clone())
    }

    fn check_plugin_name(&mut self, name: &str, span: Span) {
        let Some(enabled) = self.project.and_then(|p| p.plugins.as_ref()) else { return };
        if self.plugin_names.is_none() {
            let mut map = HashMap::new();
            for p in self.d.project.plugins.iter().filter(|p| !enabled.contains(&p.name)) {
                for n in &p.names {
                    map.entry(n.clone()).or_insert_with(|| p.name.clone());
                }
                for r in &p.defs {
                    let pkg = r.split('@').next().unwrap_or(r);
                    if let Some(index) = self.d.typedefs.package(pkg).and_then(|pk| pk.index().ok()) {
                        for sym in index.symbols.keys() {
                            map.entry(sym.clone()).or_insert_with(|| p.name.clone());
                        }
                    }
                }
            }
            self.plugin_names = Some(map);
        }
        let plugin = self.plugin_names.as_ref().and_then(|m| m.get(name)).cloned();
        if let Some(plugin) = plugin
            && !self.decl_names.contains_key(name)
            && self.plugin_reported.insert(name.to_string())
        {
            let msg = format!("'{name}' is not available: {}", self.plugin_hint(&plugin));
            self.error(codes::PLUGIN_DISABLED, span, msg);
        }
    }

    /// `= null` is left out when the target already defaults to null (options.omit_null_default).
    fn omitted_default(&self, e: &Expr) -> bool {
        self.d.options.omit_null_default && matches!(e.kind, ExprKind::Null)
    }

    fn string(&mut self, pieces: &[StrPiece]) -> String {
        let o = &self.d.options;
        let interp = pieces.iter().any(|p| matches!(p, StrPiece::Interp(_)));
        let quote = if interp { o.interp_quote.clone() } else { o.string_quote.clone() };
        let dollar = o.dollar_escape.clone();
        let (open, close) = (o.interp_open.clone(), o.interp_close.clone());
        let braces = interp && o.interp_brace_escape;
        let mut out = if interp { format!("{}{quote}", o.interp_prefix) } else { quote.clone() };
        for (i, p) in pieces.iter().enumerate() {
            match p {
                StrPiece::Text(t) => {
                    let mut chars = t.chars().peekable();
                    while let Some(c) = chars.next() {
                        match c {
                            '\\' => out.push_str("\\\\"),
                            '\n' => out.push_str("\\n"),
                            '\t' => out.push_str("\\t"),
                            '\r' => out.push_str("\\r"),
                            '\0' => out.push_str("\\0"),
                            '$' if dollar == "always"
                                || (dollar == "before_brace" && interp && chars.peek() == Some(&'{')) =>
                            {
                                out.push_str("\\$")
                            }
                            c if quote.starts_with(c) => {
                                out.push('\\');
                                out.push(c);
                            }
                            '{' | '}' if braces => {
                                out.push(c);
                                out.push(c);
                            }
                            c => out.push(c),
                        }
                    }
                }
                StrPiece::Interp(e) => {
                    let v = self.expr(e).0;
                    // `$name` when the next character cannot continue the name
                    let bare = self.d.options.interp_bare_names
                        && is_plain_name(&v)
                        && !matches!(pieces.get(i + 1), Some(StrPiece::Text(t)) if t.starts_with(|c: char| c.is_alphanumeric() || c == '_'));
                    if bare {
                        out.push('$');
                        out.push_str(&v);
                    } else {
                        out.push_str(&open);
                        out.push_str(&v);
                        out.push_str(&close);
                    }
                }
            }
        }
        out.push_str(&quote);
        out
    }

    fn object(&mut self, entries: &[ObjEntry], span: Span) -> String {
        let mut rendered = Vec::new();
        for en in entries {
            let text = match en {
                ObjEntry::Pair { key, value } => {
                    let k = match key {
                        ObjKey::Ident(i) => self.tpl("object_key_ident", Ctx::new().set("key", &i.name), i.span),
                        ObjKey::Str(s) => self.tpl("object_key_string", Ctx::new().set("key", s), span),
                    };
                    let v = self.expr(value).0;
                    self.tpl("object_entry", Ctx::new().set("key", k).set("value", v), span)
                }
                ObjEntry::Shorthand(i) => {
                    let value = self.name_value(&i.name, i.span);
                    if value == i.name {
                        self.tpl("object_shorthand", Ctx::new().set("key", &i.name), i.span)
                    } else {
                        // `{ text }` where text is a member: write the full entry
                        let k = self.tpl("object_key_ident", Ctx::new().set("key", &i.name), i.span);
                        self.tpl("object_entry", Ctx::new().set("key", k).set("value", value), i.span)
                    }
                }
                ObjEntry::Spread(e) => {
                    let v = self.expr(e).0;
                    self.tpl("object_spread", Ctx::new().set("expr", v), span)
                }
            };
            rendered.push(text);
        }
        let joined = self.join_items(&rendered, 4);
        let text = self.tpl("object", Ctx::new().set("entries", joined.clone()), span);
        close_on_own_line(text, &joined)
    }

    /// Items of a call, list or object: on one line, or one per line when long or multi-line.
    fn join_items(&self, items: &[String], extra: usize) -> String {
        self.join_items_at(items, &[], extra)
    }

    /// `join_items` that, when it puts one item per line, marks each line with its source offset (`starts`).
    fn join_items_at(&self, items: &[String], starts: &[usize], extra: usize) -> String {
        let inline = items.join(", ");
        let multiline_item = items.iter().any(|i| i.contains('\n'));
        let too_long = inline.len() + extra > self.d.options.line_width;
        if items.is_empty() || (!multiline_item && !too_long) {
            return inline;
        }
        // a single lambda, object or list stays hugged: f((x) => {\n...\n})
        if items.len() == 1
            && (!multiline_item || items[0].starts_with(['(', '{', '[']) || items[0].starts_with("async"))
        {
            return inline;
        }
        let indent = &self.d.indent;
        let mut out = String::from("\n");
        let n = items.len();
        for (i, item) in items.iter().enumerate() {
            match starts.get(i) {
                Some(&pos) => out.push_str(&indent_lines(&mark(pos, item.clone()), indent)),
                None => out.push_str(&indent_lines(item, indent)),
            }
            if i + 1 < n || self.d.options.trailing_comma {
                out.push(',');
            }
            out.push('\n');
        }
        out
    }

    fn call(&mut self, callee: &Expr, args: &[Arg], type_args: &[Type], span: Span) -> String {
        if matches!(callee.kind, ExprKind::Super) && self.has("super_ctor") {
            let a = self.args(args, None, span, 0);
            return self.tpl("super_ctor", Ctx::new().set("args", a), span);
        }
        if self.result_builtin {
            match &callee.kind {
                ExprKind::Name(n) if (n == "Ok" || n == "Err") && self.lookup_local(n).is_none() => {
                    return self.result_literal(n == "Ok", args, span);
                }
                ExprKind::Member { object, name, .. } if self.is_result_callback(object, &name.name, args) => {
                    return self.result_method(object, &name.name, &args[0].value, span);
                }
                ExprKind::Member { object, name, .. } if name.name == "ok" || name.name == "error" => {
                    self.result_needs_await(object);
                }
                _ => {}
            }
        }
        // function shortcuts: `print` → `console.log`, `now()` → template (looked up by the name as written:
        // naming may have rewritten `awaitAll` as `await_all`)
        let shortcut = match &callee.kind {
            ExprKind::Name(n) => self.src_text(callee.span).unwrap_or_else(|| n.clone()),
            _ => String::new(),
        };
        if let ExprKind::Name(n) = &callee.kind
            && self.lookup_local(n).is_none()
            && !self.is_member(n)
            && !self.decl_names.contains_key(n)
            && let Some(f) = self.d.functions.get(&shortcut).map(|e| e.value.clone())
        {
            match f {
                FunctionShortcut::Rename(to) => {
                    let a = self.args(args, None, span, to.len());
                    return self.tpl("call", Ctx::new().set("callee", to).set("args", a), span);
                }
                FunctionShortcut::Template(t) => {
                    let rendered: Vec<String> = args.iter().map(|a| self.expr(&a.value).0).collect();
                    let mut ctx = Ctx::new().set("args", rendered.join(", "));
                    for (i, r) in rendered.iter().enumerate() {
                        ctx = ctx.set(&format!("arg{i}"), r.clone());
                    }
                    if let Some(h) = self.d.function_helpers.get(&shortcut).cloned() {
                        self.helpers.entry(format!("fn.{shortcut}")).or_insert_with(|| h.render(&Value::Map(BTreeMap::new())));
                    }
                    return t.render(&ctx.into_value());
                }
            }
        }
        // the standard library: `xs.add(x)`, `text.upper()`, `Math.sqrt(x)`, `Regex(r"\d+")`
        if let ExprKind::Member { object, name, optional: false } = &callee.kind
            && let Some(text) = self.library_access(object, name, Some(args), type_args, span)
        {
            return text;
        }
        if let ExprKind::Name(n) = &callee.kind
            && self.lookup_local(n).is_none()
            && !self.is_member(n)
            && !self.decl_names.contains_key(n)
        {
            if library::OBJECTS.contains(&n.as_str()) && library::member_of(n, "new").is_some() {
                if let Some(text) = self.library_static(n, "new", Some(args), type_args, callee.span) {
                    return text;
                }
            } else if let Some((_, fix)) = library::WRONG_FUNCTIONS.iter().find(|(w, _)| w == n)
                && !self.d.functions.contains_key(n)
            {
                self.error(codes::LIBRARY_NAME, callee.span, format!("'{n}(...)' is not Nuclear: use {fix}"));
            }
        }
        let with_site = self.call_site(callee, args, span);
        let args = with_site.as_deref().unwrap_or(args);
        let mut params = self.callee_params(callee);
        // a method of this module called without `this`: its parameters only tell which arguments are
        // Results to keep (named arguments to it are not reordered)
        if params.is_none()
            && !args.iter().any(|a| a.name.is_some())
            && let ExprKind::Name(n) = &callee.kind
            && self.lookup_local(n).is_none()
            && self.is_member(n)
            && let Some(m) = self.module.as_ref().map(|m| m.name.clone())
        {
            params = self.decl_info(&m).and_then(|i| i.methods.get(n).cloned());
        }
        let construct = match &callee.kind {
            ExprKind::Name(n) => is_pascal(n) && self.lookup_local(n).is_none() && !self.d.not_constructors.contains(n),
            _ => false,
        };
        if construct && let ExprKind::Name(n) = &callee.kind {
            self.check_construct(n, args, callee.span);
        }
        // `prompt(...)` inside `write(prompt: string)` of a module with a method `prompt`: the call reaches the local
        // (the parameter), not the method it hides
        if let ExprKind::Name(n) = &callee.kind
            && self.lookup_local(n).is_some()
            && let Some(m) = self.module.as_ref().map(|m| m.name.clone())
            && self.decl_info(&m).is_some_and(|i| i.methods.contains_key(n))
            && !matches!(self.static_type(callee).map(|t| t.kind), Some(TypeKind::Func { .. }))
        {
            self.error(
                codes::DUPLICATE_DECL,
                callee.span,
                format!("'{n}' here is a local (a parameter or a variable) that hides the method '{n}' of '{m}': give the local another name"),
            );
        }
        // `inject<Undeclared>()`: a type argument names a class too
        for t in type_args {
            if let TypeKind::Named { name, args: targs } = &t.kind
                && targs.is_empty()
                && is_pascal(name)
                && !name.contains('.')
                && !self.type_params.contains(name)
                && !self.generic_names.contains(name)
            {
                self.check_construct(name, &[], t.span);
            }
        }
        // `res.text()`: the callee is a method of the target, not the library's `text` property
        self.callee_member = matches!(callee.kind, ExprKind::Member { .. });
        let c = self.expr(callee);
        self.callee_member = false;
        let mut c = self.wrap(c, P_POSTFIX);
        let mut type_names = Vec::new();
        if !type_args.is_empty() {
            let names: Vec<String> = type_args.iter().map(|t| self.ty(t)).collect();
            let as_args = match &callee.kind {
                ExprKind::Name(n) => self.d.options.type_args_as_args.applies(n),
                _ => self.d.options.type_args_as_args.applies(""),
            };
            if as_args {
                type_names = names;
            } else {
                c.push_str(&self.tpl("generics", Ctx::new().set("names", names.join(", ")), span));
            }
        }
        let mut a = self.args(args, params.as_deref(), span, c.len());
        if !type_names.is_empty() {
            // Python: inject<Repo>() → inject(Repo)
            type_names.extend((!a.is_empty()).then(|| a.clone()));
            a = type_names.join(", ");
        }
        let key = if construct { "construct" } else { "call" };
        let ctx = Ctx::new().set("callee", c).set("args", a.clone());
        let text = match (construct, &callee.kind) {
            // `[kinds.<k>.syntax] construct`: a declaration of that kind is built its own way (a view is a function)
            (true, ExprKind::Name(n)) if self.kind_construct(n).is_some() => {
                self.kind_construct(n).unwrap().render(&ctx.into_value())
            }
            _ => self.tpl(key, ctx, span),
        };
        close_on_own_line(text, &a)
    }

    /// The `construct` template of the kind of the declaration `name`, when its dialect gives one.
    fn kind_construct(&self, name: &str) -> Option<Template> {
        let DeclKind::Kind(k) = &self.decl_info(name)?.kind else { return None };
        self.d.kinds.get(k)?.value.syntax.get("construct").cloned()
    }

    /// `[call_site]`: `Log.d(...)` also receives where it is written, as a named text argument.
    fn call_site(&self, callee: &Expr, args: &[Arg], span: Span) -> Option<Vec<Arg>> {
        let ExprKind::Member { object, .. } = &callee.kind else { return None };
        let ExprKind::Name(m) = &object.kind else { return None };
        let (arg, text) = self.d.call_site.get(m)?;
        if self.lookup_local(m).is_some()
            || self.is_member(m)
            || self.decl_names.contains_key(m)
            || args.iter().any(|a| a.name.as_ref().is_some_and(|n| &n.name == arg))
        {
            return None;
        }
        let line = self.line_starts.partition_point(|&s| s <= span.start);
        let ctx = Ctx::new()
            .set("path", &self.file_path)
            .set("line", line.to_string())
            .set("component", self.component())
            .set("kind", &self.kind_name)
            .set("module", &self.owner)
            .set("method", self.fn_names.last().cloned().unwrap_or_default());
        let value = Expr { span, kind: ExprKind::Str(vec![StrPiece::Text(text.render(&ctx.into_value()))]) };
        let mut out = args.to_vec();
        out.push(Arg { span, name: Some(Ident { name: arg.clone(), span }), value, spread: false, ty: None });
        Some(out)
    }

    /// Arguments. Named arguments to a callee declared in Nuclear are reordered by position; to an
    /// external callee they become one object or native named arguments (options.named_args).
    /// `callee_len` counts toward the line width (plus a margin for the indentation).
    fn args(&mut self, args: &[Arg], params: Option<&[Param]>, span: Span, callee_len: usize) -> String {
        let has_named = args.iter().any(|a| a.name.is_some());
        let ordered: Option<Vec<Arg>> = match (has_named, params) {
            (true, Some(ps)) => reorder(args, ps, self.d.options.named_args_native),
            _ => None,
        };
        let args = ordered.as_deref().unwrap_or(args);
        let mut rendered = Vec::new();
        let mut starts = Vec::new(); // source offset of each rendered argument
        let mut object_entries = Vec::new();
        for (i, a) in args.iter().enumerate() {
            // an argument for a parameter typed Result is passed as it is
            let param = params.and_then(|ps| match &a.name {
                Some(n) => ps.iter().find(|p| p.name.name == n.name),
                None => ps.get(i),
            });
            if param.and_then(|p| p.ty.as_ref()).is_some_and(|t| self.result_type(t).is_some()) {
                self.keep_next = true;
            }
            if a.spread {
                let v = self.expr(&a.value).0;
                rendered.push(self.tpl("spread_arg", Ctx::new().set("expr", v), a.span));
                starts.push(a.span.start);
                continue;
            }
            let v = self.expr(&a.value).0;
            match &a.name {
                None => {
                    rendered.push(v);
                    starts.push(a.span.start);
                }
                Some(n) if self.d.options.named_args_native => {
                    rendered.push(self.tpl("named_arg", Ctx::new().set("name", &n.name).set("value", v), a.span));
                    starts.push(a.span.start);
                }
                Some(n) => {
                    let k = self.tpl("object_key_ident", Ctx::new().set("key", &n.name), n.span);
                    object_entries.push(self.tpl("object_entry", Ctx::new().set("key", k).set("value", v), a.span));
                }
            }
        }
        if !object_entries.is_empty() {
            let joined = self.join_items(&object_entries, 4);
            let text = self.tpl("object", Ctx::new().set("entries", joined.clone()), span);
            rendered.push(close_on_own_line(text, &joined));
            starts.push(span.start);
        }
        self.join_items_at(&rendered, &starts, callee_len + 2 + 8)
    }

    fn lambda(&mut self, l: &Lambda) -> String {
        let is_async = analysis::lambda_awaits(l);
        let result_callback = std::mem::take(&mut self.next_lambda_result);
        let ctx = self.fn_ctx_for(l.ret.as_ref(), result_callback);
        self.fn_ctx.push(ctx);
        let saved_pending = std::mem::take(&mut self.pending);
        let saved_blocked = std::mem::take(&mut self.hoist_blocked);
        self.frames.push(Frame::default());
        let hints = std::mem::take(&mut self.lambda_hints);
        let params = self.params(&l.params, false, l.span);
        // an untyped parameter of a library callback has the receiver's element type (`xs.map((x) => x.upper())`)
        for (i, p) in l.params.iter().enumerate() {
            if p.ty.is_none()
                && let Some(h) = hints.get(i)
                && let Some(f) = self.frames.last_mut()
            {
                f.types.insert(p.name.name.clone(), h.clone());
            }
        }
        self.lambda_names = l.params.iter().map(|p| self.escape(&p.name.name)).collect::<Vec<_>>().join(", ");
        let text = match &l.body {
            LambdaBody::Expr(e) => {
                let x = self.tail_expr(e, &Tail::Return);
                self.expr_lambda(params, x, is_async, e.span, l.span)
            }
            LambdaBody::Block(b) => {
                let returns = l.ret.as_ref().is_none_or(|t| !is_void(t));
                // a block with one expression is written as an expression lambda
                if let [Stmt::Expr(e)] = b.stmts.as_slice()
                    && !matches!(e.kind, ExprKind::Lambda(_))
                {
                    let x = if returns { self.tail_expr(e, &Tail::Return) } else { self.expr(e).0 };
                    self.expr_lambda(params, x, is_async, e.span, l.span)
                } else {
                    let body = self.body(&b.stmts, returns);
                    self.block_lambda(params, body, is_async, l.span)
                }
            }
        };
        self.frames.pop();
        self.pending = saved_pending;
        self.pending.append(&mut self.hoisted);
        self.hoist_blocked = saved_blocked;
        self.fn_ctx.pop();
        text
    }

    /// A lambda with a block. A target whose lambdas are one expression (`options.hoist_block_lambdas`) gets
    /// a local function declared before the statement, and the lambda is its name.
    fn block_lambda(&mut self, params: String, body: String, is_async: bool, span: Span) -> String {
        if !self.d.options.hoist_block_lambdas || !self.has("local_function") {
            let ctx = Ctx::new().set("params", params).set("names", &self.lambda_names).set("body", body);
            return self.tpl("lambda", ctx.flag("async", is_async), span);
        }
        self.lambda_count += 1;
        let name = format!("_fn{}", self.lambda_count);
        let ctx = Ctx::new().set("name", &name).set("params", params).set("body", body).flag("async", is_async);
        let def = self.tpl("local_function", ctx, span);
        self.hoisted.push(def);
        name
    }

    /// A module or file path with the target's separator (`options.module_separator`).
    fn sep(&self, p: &str) -> String {
        p.replace('/', &self.d.options.module_separator)
    }

    /// A lambda whose body is one expression; with pending lines (unwrapped Results) it needs a block.
    fn expr_lambda(&mut self, params: String, x: String, is_async: bool, x_span: Span, span: Span) -> String {
        if self.pending.is_empty() {
            let ctx = Ctx::new().set("params", params).set("names", &self.lambda_names).set("expr", x);
            return self.tpl("lambda_expr", ctx.flag("async", is_async), span);
        }
        let mut lines = std::mem::take(&mut self.pending);
        lines.push(self.tpl("return", Ctx::new().set("value", x), x_span));
        self.block_lambda(params, lines.join("\n"), is_async, span)
    }

    /// `if` used as a value: a chain of ternaries.
    fn if_value(&mut self, i: &If) -> (String, u8) {
        let Some(otherwise) = &i.otherwise else {
            self.error(codes::BRANCHING_VALUE, i.span, "an if used as a value needs an else branch");
            return (String::new(), P_PRIMARY);
        };
        let mut arms: Vec<(Option<Expr>, &Block)> = i.branches.iter().map(|(c, b)| (Some(c.clone()), b)).collect();
        arms.push((None, otherwise));
        self.ternary_chain(&arms, i.span)
    }

    fn match_value(&mut self, m: &Match) -> (String, u8) {
        if let Some(arms) = self.result_arms(m) {
            return self.result_match_value(m, arms);
        }
        let mut arms_owned: Vec<(Option<Expr>, Block)> = Vec::new();
        for arm in &m.arms {
            let cond = self.arm_condition(&m.subject, &arm.patterns);
            let block = match &arm.body {
                ArmBody::Block(b) => b.clone(),
                ArmBody::Expr(e) => Block { span: e.span, stmts: vec![Stmt::Expr(e.clone())] },
            };
            let wildcard = cond.is_none();
            arms_owned.push((cond, block));
            if wildcard {
                break;
            }
        }
        if arms_owned.last().is_none_or(|(c, _)| c.is_some()) {
            self.error(codes::BRANCHING_VALUE, m.span, "a match used as a value needs a '_ ->' arm");
            return (String::new(), P_PRIMARY);
        }
        let arms: Vec<(Option<Expr>, &Block)> = arms_owned.iter().map(|(c, b)| (c.clone(), b)).collect();
        self.ternary_chain(&arms, m.span)
    }

    fn ternary_chain(&mut self, arms: &[(Option<Expr>, &Block)], span: Span) -> (String, u8) {
        let mut values = Vec::new();
        // the branches (and every condition but the first) may not run
        self.hoist_blocked += 1;
        for (_, b) in arms {
            match b.stmts.as_slice() {
                [Stmt::Expr(e)] => values.push(self.expr(e)),
                _ => {
                    self.error(
                        codes::BRANCHING_VALUE,
                        b.span,
                        "when if/match is used as a value, each branch must be a single expression",
                    );
                    self.hoist_blocked -= 1;
                    return (String::new(), P_PRIMARY);
                }
            }
        }
        // build from the end: c1 ? v1 : (c2 ? v2 : v3)
        let mut acc = values.pop().unwrap();
        let n = arms.len() - 1;
        for (i, ((cond, _), v)) in arms[..n].iter().zip(values).enumerate().rev() {
            self.hoist_blocked -= u32::from(i == 0);
            let c = self.expr(cond.as_ref().unwrap());
            self.hoist_blocked += u32::from(i == 0);
            let c = self.wrap(c, P_TERNARY + 1);
            let t = self.wrap(v, P_TERNARY);
            let o = self.wrap(acc, P_TERNARY);
            acc = (self.tpl("ternary", Ctx::new().set("cond", c).set("then", t).set("else", o), span), P_TERNARY);
        }
        self.hoist_blocked -= 1;
        acc
    }

    // ---------- types ----------

    fn ty(&mut self, t: &Type) -> String {
        match &t.kind {
            TypeKind::Named { name, args } => {
                if self.result_type(t).is_some() {
                    self.uses_result = true;
                }
                let root = name.split('.').next().unwrap_or(name);
                if !self.decl_names.contains_key(root) && !self.imported.contains(root) {
                    self.check_plugin_name(root, t.span);
                    // a type the dialect writes with another class name (`Page` → `PageOf`) is imported by that name
                    let renamed = self.d.types.get(root).map(|e| e.value.render(&Value::Map(BTreeMap::new())));
                    match renamed.filter(|r| is_pascal(r) && r.chars().all(|c| c.is_ascii_alphanumeric())) {
                        Some(r) => self.refs.insert(r),
                        None => self.refs.insert(root.to_string()),
                    };
                }
                // a declaration of the project keeps its name over the dialect's (`Page` of an app is not `PageOf`)
                let mapped = if self.decl_names.contains_key(root) { None } else { self.d.types.get(name.as_str()) };
                let base = match mapped {
                    Some(e) => e.value.render(&Value::Map(BTreeMap::new())),
                    None => name.clone(),
                };
                if args.is_empty() {
                    return base;
                }
                let rendered: Vec<String> = args.iter().map(|a| self.ty(a)).collect();
                self.type_tpl("generic", Ctx::new().set("name", base).set("args", rendered.join(", ")), t.span)
                    .unwrap_or_default()
            }
            TypeKind::List(item) => {
                let i = self.ty(item);
                let i =
                    if matches!(item.kind, TypeKind::Union(_) | TypeKind::Func { .. }) { format!("({i})") } else { i };
                self.type_tpl("list", Ctx::new().set("item", i), t.span).unwrap_or_default()
            }
            TypeKind::Map(k, v) => {
                let k = self.ty(k);
                let v = self.ty(v);
                self.type_tpl("map", Ctx::new().set("key", k).set("value", v), t.span).unwrap_or_default()
            }
            TypeKind::Nullable(inner) => {
                let i = self.ty(inner);
                self.type_tpl("nullable", Ctx::new().set("type", i), t.span).unwrap_or_default()
            }
            TypeKind::Union(list) => {
                let mut rendered: Vec<String> = Vec::new();
                for x in list {
                    let r = self.ty(x);
                    if !rendered.contains(&r) {
                        rendered.push(r);
                    }
                }
                if rendered.len() == 1 {
                    return rendered.pop().unwrap();
                }
                let values = Value::List(rendered.into_iter().map(Value::Str).collect());
                self.type_tpl("union", Ctx::new().value("types", values), t.span).unwrap_or_default()
            }
            TypeKind::StrLit(s) => {
                self.type_tpl("string_literal", Ctx::new().set("value", s), t.span).unwrap_or_default()
            }
            TypeKind::NumLit(n) => {
                self.type_tpl("number_literal", Ctx::new().set("value", n), t.span).unwrap_or_default()
            }
            TypeKind::Func { params, ret } => {
                self.frames.push(Frame::default());
                let p = self.params(params, false, t.span);
                self.frames.pop();
                let r = self.ty(ret);
                let any = self.d.types.get("any").map(|t| t.value.render(&Ctx::new().into_value())).unwrap_or_default();
                let types: Vec<String> =
                    params.iter().map(|p| p.ty.as_ref().map_or_else(|| any.clone(), |t| self.ty(t))).collect();
                let ctx = Ctx::new().set("params", p).set("types", types.join(", ")).set("ret", r);
                self.type_tpl("function", ctx, t.span).unwrap_or_default()
            }
        }
    }
}

/// Some(true) when every branch is true, Some(false) when every branch is false, None when mixed.
fn combine(all: Vec<Option<bool>>) -> Option<bool> {
    if all.iter().all(|x| *x == Some(true)) {
        Some(true)
    } else if all.iter().all(|x| *x == Some(false)) {
        Some(false)
    } else {
        None
    }
}

/// Named arguments → positions, using the callee's parameters. `None` when it cannot be done.
fn reorder(args: &[Arg], params: &[Param], native: bool) -> Option<Vec<Arg>> {
    let positional: Vec<&Arg> = args.iter().filter(|a| a.name.is_none() && !a.spread).collect();
    if args.iter().any(|a| a.spread) || positional.len() > params.len() {
        return None;
    }
    // a kind with named parameters in a dialect whose named arguments are one object (React props): as written
    if !native && params.iter().any(|p| p.named) {
        return None;
    }
    let mut named: HashMap<&str, &Arg> =
        args.iter().filter_map(|a| a.name.as_ref().map(|n| (n.name.as_str(), a))).collect();
    let mut out: Vec<Arg> = positional.iter().map(|a| (*a).clone()).collect();
    let mut keep_named: Vec<Arg> = Vec::new();
    let last_needed = params
        .iter()
        .enumerate()
        .filter(|(_, p)| named.contains_key(p.name.name.as_str()) && !(native && p.named))
        .map(|(i, _)| i)
        .max();
    for (i, p) in params.iter().enumerate().skip(positional.len()) {
        if native && p.named {
            if let Some(a) = named.remove(p.name.name.as_str()) {
                keep_named.push(a.clone());
            }
            continue;
        }
        if last_needed.is_none_or(|l| i > l) {
            break;
        }
        match named.remove(p.name.name.as_str()) {
            Some(a) => out.push(Arg { name: None, ..a.clone() }),
            None => {
                let default = p
                    .default
                    .clone()
                    .or_else(|| p.optional.then(|| Expr { span: Span::default(), kind: ExprKind::Null }))?;
                out.push(Arg { span: Span::default(), name: None, value: default, spread: false, ty: None });
            }
        }
    }
    if !named.is_empty() {
        return None; // a name that is not a parameter: leave the call as written
    }
    out.extend(keep_named);
    Some(out)
}

/// Comment lines above a piece of code.
// Line map (output line → source line, for debuggers): the code of each statement, member and declaration starts
// with `MARK <source offset> END`; `file` removes the marks once the output is laid out.
const MARK: char = '\u{E000}';
const END: char = '\u{E001}';

fn mark(pos: usize, code: String) -> String {
    format!("{MARK}{pos}{END}{code}")
}

/// The code without marks and, for every output line that has one (1-based), the source offset of its first.
fn unmark(code: &str) -> (String, Vec<(u32, usize)>) {
    let mut out = String::with_capacity(code.len());
    let mut marks = Vec::new();
    for (i, line) in code.split_inclusive('\n').enumerate() {
        let mut rest = line;
        let mut first = true;
        while let Some(at) = rest.find(MARK) {
            out.push_str(&rest[..at]);
            let after = &rest[at + MARK.len_utf8()..];
            let end = after.find(END).unwrap_or(0);
            if first && let Ok(pos) = after[..end].parse() {
                marks.push((i as u32 + 1, pos));
                first = false;
            }
            rest = &after[end + END.len_utf8()..];
        }
        out.push_str(rest);
    }
    (out, marks)
}

fn attach(comments: String, code: String) -> String {
    if comments.is_empty() { code } else { format!("{comments}\n{code}") }
}

/// When items were split one per line, the closing bracket goes alone at the start of the last line.
fn close_on_own_line(text: String, items: &str) -> String {
    if !items.starts_with('\n') {
        return text;
    }
    match text.rfind('\n') {
        Some(p) => format!("{}{}", &text[..=p], text[p + 1..].trim_start()),
        None => text,
    }
}

/// Final cleanup: no trailing spaces, at most `gap` blank lines in a row (options.toplevel_gap), one final newline.
fn tidy(s: &str, gap: usize) -> String {
    let mut out = String::new();
    let mut blank = 0;
    for line in s.lines() {
        let l = line.trim_end();
        if l.is_empty() {
            blank += 1;
            if blank > gap {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(l);
        out.push('\n');
    }
    let trimmed = out.trim_matches('\n');
    format!("{trimmed}\n")
}

/// `name` or `_name`, without `$` (a plain identifier in the target).
fn is_plain_name(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_alphabetic() || f == '_') && c.all(|x| x.is_alphanumeric() || x == '_')
}

/// Every branch of the if is a single expression (it can be written as a ternary).
fn single_expr_if(i: &If) -> bool {
    let one = |b: &Block| matches!(b.stmts.as_slice(), [Stmt::Expr(_)]);
    i.branches.iter().all(|(_, b)| one(b)) && i.otherwise.as_ref().is_none_or(one)
}

fn single_expr_match(m: &Match) -> bool {
    m.arms.iter().all(|arm| match &arm.body {
        ArmBody::Expr(_) => true,
        ArmBody::Block(b) => matches!(b.stmts.as_slice(), [Stmt::Expr(_)]),
    })
}

/// The words of generated code that are code: not the comments, and of a string literal only its
/// interpolations (`${expr}`, `$name`). Imports are decided with them, so a name in a comment or in a
/// text does not import its file.
pub(crate) fn code_words(code: &str) -> HashSet<String> {
    let c: Vec<char> = code.chars().collect();
    let mut kept = String::with_capacity(code.len());
    // None = code (with its depth of `{`), Some(quote) = inside a string
    let mut stack: Vec<(Option<char>, usize)> = vec![(None, 0)];
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        let next = c.get(i + 1).copied();
        match stack.last().copied().unwrap_or((None, 0)) {
            (None, depth) => {
                if ch == '/' && next == Some('/') {
                    while i < c.len() && c[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }
                if ch == '/' && next == Some('*') {
                    i += 2;
                    while i < c.len() && !(c[i] == '*' && c.get(i + 1) == Some(&'/')) {
                        i += 1;
                    }
                    i += 2;
                    continue;
                }
                // `# text` (Python comment)
                if ch == '#' && next == Some(' ') {
                    while i < c.len() && c[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }
                match ch {
                    // an f-string (Python) interpolates with `{`: marked with 1
                    '\'' | '"' | '`' => {
                        let f = i > 0 && c[i - 1] == 'f' && (i < 2 || !(c[i - 2].is_alphanumeric() || c[i - 2] == '_'));
                        stack.push((Some(ch), usize::from(f)))
                    }
                    '{' => {
                        if let Some(top) = stack.last_mut() {
                            top.1 += 1;
                        }
                    }
                    '}' if depth == 0 && stack.len() > 1 => {
                        stack.pop();
                    }
                    '}' => {
                        if let Some(top) = stack.last_mut() {
                            top.1 = top.1.saturating_sub(1);
                        }
                    }
                    _ => {
                        kept.push(ch);
                        i += 1;
                        continue;
                    }
                }
                kept.push(' ');
            }
            (Some(q), fstring) => {
                if ch == '\\' {
                    i += 2;
                    continue;
                }
                if fstring == 1 && ch == '{' && next == Some('{') {
                    i += 2;
                    continue;
                }
                if fstring == 1 && ch == '{' {
                    stack.push((None, 0));
                } else if ch == q {
                    stack.pop();
                } else if ch == '$' && next == Some('{') {
                    stack.push((None, 0));
                    i += 1;
                } else if ch == '$' {
                    kept.push(' ');
                    while c.get(i + 1).is_some_and(|n| n.is_alphanumeric() || *n == '_') {
                        i += 1;
                        kept.push(c[i]);
                    }
                }
                kept.push(' ');
            }
        }
        i += 1;
    }
    kept.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// The same Nuclear name, also after a dialect wrote it in snake_case (`onDelete` and `on_delete`).
fn same_name(a: &str, b: &str) -> bool {
    a.len().abs_diff(b.len()) <= 4 && a.replace('_', "").eq_ignore_ascii_case(&b.replace('_', ""))
}

/// `field_decor.<decorator>`: the first field with that decorator (`createdAt: '{{field_decor.created}}'`).
fn field_decor(members: &[Member]) -> Value {
    let mut m = BTreeMap::new();
    for member in members {
        if let Member::Field(f) = member {
            for d in &f.decorators {
                m.entry(d.name.clone()).or_insert_with(|| Value::str(&f.name.name));
            }
        }
    }
    Value::Map(m)
}

/// The names a decorator's route text declares: `:id` and `{id}` in `@get("/tasks/:id")` / `@get("/{id}")`.
fn route_params(decorators: &[Decorator]) -> HashSet<String> {
    let mut out = HashSet::new();
    for d in decorators {
        let Some(Arg { value: Expr { kind: ExprKind::Str(pieces), .. }, .. }) = d.args.as_ref().and_then(|a| a.first())
        else {
            continue;
        };
        for p in pieces {
            let StrPiece::Text(t) = p else { continue };
            for seg in t.split('/') {
                let name = seg.strip_prefix(':').or_else(|| seg.strip_prefix('{').and_then(|x| x.strip_suffix('}')));
                if let Some(n) = name.filter(|n| !n.is_empty()) {
                    out.insert(n.to_string());
                }
            }
        }
    }
    out
}

/// What the value before a dot is, for the standard library.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Recv {
    /// A base type of Nuclear (`library::BASES`).
    Base(&'static str),
    /// A class of the program or of the type definitions, a Result, a function: not the library's.
    Other,
    /// `any`, `dynamic`: not known.
    Unknown,
}

fn named_type(name: &str, args: Vec<Type>, span: Span) -> Type {
    Type { span, kind: TypeKind::Named { name: name.to_string(), args } }
}

fn strip_nullable(t: Type) -> Type {
    match t.kind {
        TypeKind::Nullable(inner) => strip_nullable(*inner),
        _ => t,
    }
}

/// The base of the library a type belongs to.
fn receiver_base(t: &Type) -> Recv {
    match &t.kind {
        TypeKind::Nullable(inner) => receiver_base(inner),
        TypeKind::List(_) => Recv::Base("list"),
        TypeKind::Map(..) => Recv::Base("map"),
        TypeKind::StrLit(_) => Recv::Base("string"),
        TypeKind::NumLit(n) => Recv::Base(if n.contains('.') { "double" } else { "int" }),
        TypeKind::Union(items) if items.iter().all(|i| matches!(i.kind, TypeKind::StrLit(_))) => Recv::Base("string"),
        TypeKind::Union(_) | TypeKind::Func { .. } => Recv::Other,
        TypeKind::Named { name, .. } => match name.as_str() {
            "string" | "String" => Recv::Base("string"),
            "int" => Recv::Base("int"),
            "double" | "num" => Recv::Base("double"),
            "boolean" | "bool" => Recv::Base("boolean"),
            // `Map<K, V>()` and `List()` are the target's classes (type definitions), not Nuclear's literals
            "List" | "Map" => Recv::Other,
            "Set" => Recv::Base("set"),
            "Bytes" => Recv::Base("bytes"),
            "DateTime" => Recv::Base("datetime"),
            "Duration" => Recv::Base("duration"),
            "Regex" | "RegExp" => Recv::Base("regex"),
            "Match" => Recv::Base("match"),
            "Error" => Recv::Base("error"),
            "any" | "dynamic" | "Object" => Recv::Unknown,
            _ => Recv::Other,
        },
    }
}

/// The element type: of a list or set, the value of a map, the character of a string, the byte of Bytes.
fn elem_of(t: &Type) -> Option<Type> {
    let sp = t.span;
    match &t.kind {
        TypeKind::Nullable(inner) => elem_of(inner),
        TypeKind::List(inner) => Some((**inner).clone()),
        TypeKind::Map(_, v) => Some((**v).clone()),
        TypeKind::Named { name, args } => match (name.as_str(), args.as_slice()) {
            ("List" | "Set", [a]) => Some(a.clone()),
            ("List" | "Set", []) => Some(named_type("dynamic", Vec::new(), sp)),
            ("Map", [_, v]) => Some(v.clone()),
            ("Map", []) => Some(named_type("dynamic", Vec::new(), sp)),
            ("string" | "String", _) => Some(named_type("string", Vec::new(), sp)),
            ("Bytes", _) => Some(named_type("int", Vec::new(), sp)),
            _ => None,
        },
        _ => None,
    }
}

/// The key and value types of a map.
fn map_types(t: &Type) -> Option<(Type, Type)> {
    let sp = t.span;
    match &t.kind {
        TypeKind::Nullable(inner) => map_types(inner),
        TypeKind::Map(k, v) => Some(((**k).clone(), (**v).clone())),
        TypeKind::Named { name, args } if name == "Map" => match args.as_slice() {
            [k, v] => Some((k.clone(), v.clone())),
            _ => Some((named_type("dynamic", Vec::new(), sp), named_type("dynamic", Vec::new(), sp))),
        },
        _ => None,
    }
}

/// The class of the program a type names (`User`, `Task?`), when it is not a base type.
fn class_name(t: &Type) -> Option<String> {
    match &t.kind {
        TypeKind::Nullable(inner) => class_name(inner),
        TypeKind::Named { name, .. } if is_pascal(name) && receiver_base(t) == Recv::Other => Some(name.clone()),
        _ => None,
    }
}

/// A plain text route (`"tasks/{id}"`, `":id"`) with each parameter segment written as `pattern` says
/// (`name` stands for the parameter: `":name"` → `tasks/:id`). An interpolated string is not a route.
fn route_path(pieces: &[StrPiece], pattern: &str) -> Option<String> {
    let mut text = String::new();
    for p in pieces {
        let StrPiece::Text(t) = p else { return None };
        text.push_str(t);
    }
    let segments: Vec<String> = text
        .split('/')
        .map(|seg| {
            let name = seg.strip_prefix(':').or_else(|| seg.strip_prefix('{').and_then(|x| x.strip_suffix('}')));
            match name.filter(|n| !n.is_empty()) {
                Some(n) => pattern.replace("name", n),
                None => seg.to_string(),
            }
        })
        .collect();
    Some(segments.join("/"))
}

/// `generic_list` of templates: one entry (`name`) per type parameter.
fn generic_list(generics: &[Ident]) -> Value {
    Value::List(generics.iter().map(|g| Ctx::new().set("name", &g.name).into_value()).collect())
}

/// A name or a chain of members of names (`a`, `this.a.b`): reading it twice costs nothing and a type checker narrows
/// it (`a if a is not None else b`); a call or an index is not one.
fn is_plain_path(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Name(_) | ExprKind::This => true,
        ExprKind::Member { object, .. } => is_plain_path(object),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::code_words;

    #[test]
    fn code_words_skip_comments_and_text() {
        let words = code_words("// LoginScreen\nText('Hi ${user.name} $count Character', a: B()) /* C */ `x${D}`");
        for w in ["Text", "user", "name", "count", "a", "B", "D"] {
            assert!(words.contains(w), "{w}");
        }
        for w in ["LoginScreen", "Hi", "Character", "C", "x"] {
            assert!(!words.contains(w), "{w}");
        }
    }
}
