//! Dialect files: model, loading, inheritance (`extends`) and validation.
//!
//! A dialect is a directory `dialects/<name>/` with one or more `.toml` files (or a single
//! `<name>.toml`). Every file uses the same sections; one of them holds `[dialect]`.
//! See spec/dialects.md for the format.

use crate::contract;
use crate::defs::{self, Package, Repository, TypeDefs};
use crate::template::{Ctx, Template};
use nuclear_diag::{Code, Diagnostic, SourceFile, Span, codes};
use nuclear_syntax::token::Keyword;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Where a dialect entry was defined, for diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct Origin {
    pub dialect: String,
    /// Index into `Report::files` / `Dialect::files`.
    pub file: usize,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub value: T,
    pub origin: Origin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportShortcut {
    /// `material = "package:flutter/material.dart"`: a module string.
    Module(String),
    /// `materialui = "import 'package:material_ui/material_ui.dart';"`: a full line, copied as is.
    Line(String),
}

#[derive(Debug, Clone)]
pub enum FunctionShortcut {
    /// `print = "console.log"`
    Rename(String),
    /// `now = { template = "new Date()" }` with `{{args}}`, `{{arg0}}`...
    Template(Template),
}

#[derive(Debug, Clone)]
pub struct Kind {
    pub description: String,
    pub template: Template,
    pub overrides: Vec<String>,
    pub requires: Vec<String>,
    pub members: Vec<String>,
    pub param_access: Option<Template>,
    pub imports: Vec<String>,
    pub rename: Option<Template>,
    pub syntax: BTreeMap<String, Template>,
    /// Constructor parameters are named in the target (Flutter widgets): calls pass them by name.
    pub named_params: bool,
    /// Classes of the type definitions the generated class extends (`State`): their members are
    /// members, and a method with one of their names overrides it.
    pub inherits: Vec<String>,
    /// Every method is asynchronous (its return type is the target's async type): a repository whose
    /// methods call the network.
    pub async_methods: bool,
    /// The declarations of this kind refer to each other only in types (two entities related both ways): their
    /// imports of each other are for the type checker only (`[syntax] type_imports`), never a circular import.
    pub type_imports: bool,
    /// The only method names allowed (empty: any). Another method is error N0215.
    pub methods: Vec<String>,
    /// Name of the type every method must return (`Result`). Another return type is error N0216.
    pub returns: Option<String>,
    /// Written once per file after the imports when the file declares this kind (`part '{{file}}.g.dart';`).
    pub file_header: Option<Template>,
    /// Function that gives the instance an alias calls (`inject` → `inject<Module>().method(...)`).
    pub alias_via: Option<String>,
    /// Other classes the template writes next to the declaration (`{{name}}Builder`): the program can use
    /// them as if they were declared.
    pub declares: Vec<Template>,
    /// Members the method template writes next to each method (`purge{{name|pascal}}`, `{{name}}AsLocal`; `name` is
    /// the method's): the program can call them, and the member check (N0116) knows them.
    pub method_members: Vec<Template>,
    /// `provide = { template = "inject<{{name}}>()", kinds = ["usecase"] }`: inside this kind, a
    /// declaration of the program of one of those kinds is read with the template (`GetUser.exec()` →
    /// `inject<GetUser>().exec()`).
    pub provide: Option<(Template, Vec<String>)>,
    /// `results = "keep"`: a Result is never unwrapped or propagated inside this kind (tests compare it).
    pub keep_results: bool,
    /// Its methods never return their last line (tests).
    pub void_methods: bool,
    /// Every method has a body or is an alias (`find() = GetOne.exec`): a method without one is N0221 (an appcomponent
    /// is a facade; an empty method would do nothing).
    pub needs_body: bool,
}

/// One member of the standard library as a dialect writes it (`[library.list] add = "{{recv}}.push({{args}})"`).
#[derive(Debug, Clone)]
pub struct LibMember {
    /// Variables: `recv`, `args`, `argN`, `has_args`, `n`, `named.<name>`, `type_args`, `elem` and, when an argument
    /// is an expression lambda, `fn.param`, `fn.params`, `fn.p0`, `fn.p1`, `fn.body` (`library::VARS`).
    pub template: Template,
    /// A function the template calls, written once per file that uses the member (`helper = "..."`).
    pub helper: Option<Template>,
}

/// A package of extension kinds (ADR-0095): `kinds/<name>/kind.toml` with `[package]` (name, version, platform,
/// description), a `docs.toml`, and one folder per dialect it implements (`fastapi/kind.toml` + templates; its
/// `[plugin]` table lists the runtime packages, defs and names, like `[project.plugins.<name>]`).
#[derive(Debug, Clone, Default)]
pub struct KindPackage {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub description: String,
    /// The dialects with an implementation folder.
    pub dialects: Vec<String>,
    pub root: PathBuf,
}

/// Where a kind comes from (ADR-0095 §7): the editor shows it and lists only the project's platform kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindSource {
    /// The language dialect's own (`dart`'s `data`, `mixin`).
    Language,
    /// Native of the platform (its contract).
    Platform(String),
    /// An extension package (or a project dialect): its name, version and the dialects it implements.
    Package { name: String, version: String, dialects: Vec<String> },
}

/// `[project]`: what a dialect adds when it builds a whole project (spec/projects.md).
#[derive(Debug, Clone, Default)]
pub struct ProjectDef {
    /// Folder of the output where the transpiled files go (`lib` in Flutter): `components/x.nuc` →
    /// `lib/components/x.dart`. Imports between files are relative to it.
    pub sources: String,
    /// Folder pattern → kind of the declarations without a kind word in it
    /// (`"components/*/domain/models" = "model"`). `*` matches one folder.
    pub kinds: Vec<(String, String)>,
    /// Files generated once per project from every declaration (an injector, a router).
    pub files: Vec<ProjectFileDef>,
    /// Source folder → output folder for the files that do not go to `sources` (`test` = "test").
    pub outputs: Vec<(String, String)>,
    /// Folders of the project copied as they are to the output (`assets`: translations, data files).
    pub assets: Vec<String>,
    /// `[project.manifest]`: a `.nuc` file at the project root that configures the app (name, packages) and
    /// is not transpiled: `config.nuc` with `app(...)`, `dependencies(...)`.
    pub manifest: Option<ManifestDef>,
    /// File that marks a project folder without `nuclear.toml` (`main.nuc`): its dialect is this one.
    pub marker: String,
    /// Output folder of a project found by its `marker`, relative to it (`build/app`; default `build`).
    pub output: String,
    /// `[project.plugins.<name>]`: optional packages a project enables in its manifest (`plugins(genkit = true)`).
    pub plugins: Vec<PluginDef>,
}

/// An optional part of a framework (Genkit, Kafka, LangChain): a project carries it only when it enables it in
/// `config.nuc` (`plugins(name = true)`). Its kinds and the names of its type definitions need it.
#[derive(Debug, Clone, Default)]
pub struct PluginDef {
    pub name: String,
    pub description: String,
    /// Packages of the target's package manager: (name, version). `"local"` is a package of NuclearLang's
    /// `packages/` folder (the framework's own runtime).
    pub packages: Vec<(String, String)>,
    pub dev_packages: Vec<(String, String)>,
    /// Type definitions it brings (`genkit@1.0.1`): their names need the plugin.
    pub defs: Vec<String>,
    /// Kinds that need the plugin.
    pub kinds: Vec<String>,
    /// More names that need the plugin (classes of the runtime: `Redis`, `Kafka`).
    pub names: Vec<String>,
    /// Plugins it needs (enabled by it).
    pub requires: Vec<String>,
}

impl ProjectDef {
    pub fn plugin(&self, name: &str) -> Option<&PluginDef> {
        self.plugins.iter().find(|p| p.name == name)
    }

    /// The enabled plugins and the ones they require, in catalog order.
    pub fn with_requires(&self, enabled: &[String]) -> Vec<&PluginDef> {
        let mut want: Vec<String> = enabled.to_vec();
        let mut i = 0;
        while i < want.len() {
            if let Some(p) = self.plugin(&want[i]) {
                for r in &p.requires {
                    if !want.contains(r) {
                        want.push(r.clone());
                    }
                }
            }
            i += 1;
        }
        self.plugins.iter().filter(|p| want.contains(&p.name)).collect()
    }
}

/// The project's manifest written in Nuclear: top-level calls (`sections`) with named literal arguments.
#[derive(Debug, Clone, Default)]
pub struct ManifestDef {
    pub file: String,
    pub sections: Vec<String>,
    /// Sections that list packages of the target (`dependencies`): their names are checked against `reserved`.
    pub packages: Vec<String>,
    /// Packages the framework already adds by itself (besides the ones of `defs`): a project cannot add them.
    pub reserved: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectFileDef {
    /// Output path, relative to the output folder (`lib/dragonfly.g.dart`).
    pub path: String,
    pub template: Template,
    /// Top-level names the file declares: the project's code can use them.
    pub names: Vec<String>,
}

/// `[architecture]`: rules between the declarations of a project (spec/projects.md).
#[derive(Debug, Clone, Default)]
pub struct Architecture {
    /// Folder whose subfolders are components (`components`).
    pub scope: Option<String>,
    /// Kinds visible from other components (`appcomponent`), and `@decorator`s that make a declaration
    /// visible (`@screen`); everything else is private to its component (N0302).
    pub public: Vec<String>,
    /// Kind → kinds allowed to use it (`usecase = ["appcomponent"]`); another caller is N0301.
    pub callers: BTreeMap<String, Vec<String>>,
    /// Kinds whose declarations can use anything (tests).
    pub exempt: Vec<String>,
    /// A name (not declared in the project: `BuildContext`, `appRouter`) → the only kinds that can use it;
    /// another kind is N0301. Declarations without a kind are not checked.
    pub names: BTreeMap<String, Vec<String>>,
}

/// One class of the type definitions (`index.json`): its direct supertypes and its own instance members.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ClassDef {
    /// `Some("interface")` (only `implements`) or `Some("mixin")` (only `with`); `None` for a class.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub supers: Vec<String>,
    #[serde(default)]
    pub members: Vec<String>,
    /// The parameters of the unnamed constructor (`None`: no unnamed constructor, or not generated).
    #[serde(default)]
    pub params: Option<Vec<String>>,
    /// Methods that return a `Result<E, T>` → that type, in Nuclear notation (`find` → `Result<NotFound, T>`).
    #[serde(default)]
    pub results: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Modifier {
    pub prefix: String,
    pub applies_to: Vec<String>,
    /// The declaration never returns a value (a setter): no implicit return.
    pub void: bool,
}

#[derive(Debug, Clone)]
pub struct ClauseDef {
    pub template: Template,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeywordInfo {
    pub level: String,
    pub via: String,
}

/// `options.type_args_as_args`: `true` (every call, Python) or the functions whose type arguments are values
/// (`["inject"]`: TypeScript erases types, so `inject<Repo>()` must pass the class).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeArgsAsArgs {
    None,
    All,
    Names(Vec<String>),
}

impl TypeArgsAsArgs {
    pub fn applies(&self, callee: &str) -> bool {
        match self {
            TypeArgsAsArgs::None => false,
            TypeArgsAsArgs::All => true,
            TypeArgsAsArgs::Names(n) => n.iter().any(|x| x == callee),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub implicit_self: bool,
    pub named_args_native: bool,
    pub declare_ctor_fields: bool,
    pub const_collection_defaults: bool,
    pub optional_params_nullable: bool,
    pub number_separators: bool,
    pub binary_literals: bool,
    pub int_member_parens: bool,
    pub string_quote: String,
    pub interp_quote: String,
    pub interp_open: String,
    pub interp_close: String,
    pub dollar_escape: String,
    pub reserved_suffix: bool,
    pub max_bases: usize,
    pub catch_var: String,
    pub toplevel_main: bool,
    pub line_width: usize,
    pub trailing_comma: bool,
    pub omit_null_default: bool,
    pub interp_bare_names: bool,
    pub cascade_statements: bool,
    pub super_in_initializer: bool,
    pub implicit_base: String,
    pub interp_prefix: String,
    pub interp_brace_escape: bool,
    pub module_separator: String,
    pub toplevel_gap: usize,
    pub hoist_block_lambdas: bool,
    pub init_fields: bool,
    pub type_args_as_args: TypeArgsAsArgs,
    pub builtin_result: bool,
    pub doc_inside: bool,
    pub snake_case_names: bool,
    /// How a path parameter of a route is written in `decor.<decorator>.path` (`name` stands for the
    /// parameter): `"{name}"` (FastAPI, the platform's form) or `":name"` (NestJS, Express).
    pub route_param: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            implicit_self: false,
            named_args_native: false,
            declare_ctor_fields: false,
            const_collection_defaults: false,
            optional_params_nullable: false,
            number_separators: true,
            binary_literals: true,
            int_member_parens: false,
            string_quote: "'".into(),
            interp_quote: "`".into(),
            interp_open: "${".into(),
            interp_close: "}".into(),
            dollar_escape: "before_brace".into(),
            reserved_suffix: true,
            max_bases: 1,
            catch_var: "error".into(),
            toplevel_main: false,
            line_width: 80,
            trailing_comma: true,
            omit_null_default: false,
            interp_bare_names: false,
            cascade_statements: false,
            super_in_initializer: false,
            implicit_base: String::new(),
            interp_prefix: String::new(),
            interp_brace_escape: false,
            module_separator: "/".into(),
            toplevel_gap: 1,
            hoist_block_lambdas: false,
            init_fields: false,
            type_args_as_args: TypeArgsAsArgs::None,
            builtin_result: true,
            doc_inside: false,
            snake_case_names: false,
            route_param: "{name}".into(),
        }
    }
}

impl Dialect {
    /// A `[toolchain]` entry for this operating system: its `[toolchain.<os>]` override, else the common one.
    pub fn tool(&self, key: &str) -> Option<&str> {
        let os = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };
        self.toolchain_os.get(os).and_then(|m| m.get(key)).or_else(|| self.toolchain.get(key)).map(String::as_str)
    }
}

/// The rules of a field decorator (`[decorators.fields] manyToOne = { optional = true, list = false, target =
/// "entity", sibling = "{name}Id", inverse = "inverse" }`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldRule {
    /// The field is optional (`project?: Project`).
    pub optional: bool,
    /// The field's type is a list (`true`) or not (`false`).
    pub list: Option<bool>,
    /// The field's type (or its items) is a declaration with this decorator (`entity`).
    pub target: Option<String>,
    /// A field of the same declaration that must exist; `{name}` is the decorated field's name.
    pub sibling: Option<String>,
    /// An argument of the decorator whose value names a field of the field's type.
    pub inverse: Option<String>,
    /// The decorator needs another decorator on the same field, or on the field named by `inverse` (`joinColumn`).
    pub with: Option<String>,
}

/// One file a generator writes (`[[generators.<what>]]` of a platform, ADR-0103).
#[derive(Debug, Clone)]
pub struct GenPiece {
    /// The kind of the file: where it goes (`[layout]`).
    pub kind: String,
    /// The declaration's name, a template (`{{Name}}Repository`); the file is it in snake_case.
    pub name: Template,
    /// Written only when this context value is true (`rest`, `db`, `actions.list`, `screen`).
    pub when: Option<String>,
    /// One file per item of this context list (`models`, `actions`): the item's values over the resource's.
    pub each: Option<String>,
    /// When the file exists, add the lines of the body it lacks (a facade's aliases) instead of leaving it.
    pub merge: bool,
    /// The file.
    pub template: Template,
}

#[derive(Debug, Clone, Default)]
pub struct Dialect {
    pub name: String,
    /// The Dragonfly platform it implements (`web`, `mobile`, `backend`; ADR-0093), from `[dialect] platform`
    /// or inherited. `None` for a language dialect.
    pub platform: Option<String>,
    /// This dialect and the ones it extends: `["flutter", "dart"]`.
    pub chain: Vec<String>,
    pub description: String,
    pub file_extension: String,
    pub indent: String,
    pub options: Options,
    raw_options: toml::Table,
    pub syntax: BTreeMap<String, Entry<Template>>,
    /// `[syntax] key = { template, helper }`: a function the template calls, written once per file.
    pub syntax_helpers: BTreeMap<String, Template>,
    pub types: BTreeMap<String, Entry<Template>>,
    pub operators: BTreeMap<String, Entry<Template>>,
    pub keywords: BTreeMap<String, KeywordInfo>,
    pub imports: BTreeMap<String, Entry<ImportShortcut>>,
    /// symbol → module (`readFile` → `node:fs/promises`)
    pub auto_imports: BTreeMap<String, Entry<String>>,
    /// module → modules it re-exports entirely (material.dart → widgets.dart)
    pub reexports: BTreeMap<String, BTreeSet<String>>,
    /// module → names it exports beyond the auto-imported ones (`Task` of fpdart): a project declaration with
    /// one of them hides it from the import (`[exports]`)
    pub exports: BTreeMap<String, BTreeSet<String>>,
    pub functions: BTreeMap<String, Entry<FunctionShortcut>>,
    /// `[functions] name = { template, helper }`: the helper is written once per file that calls the function.
    pub function_helpers: BTreeMap<String, Template>,
    /// `[library.<base>]`: how each member of Nuclear's standard library (`library::MEMBERS`) is written:
    /// base (`string`, `list`, `Math`...) → member → its template.
    pub library: BTreeMap<String, BTreeMap<String, Entry<LibMember>>>,
    pub globals: BTreeSet<String>,
    pub not_constructors: BTreeSet<String>,
    /// `[names] written`: names the templates write (`Right`, `Left`): imported (`[auto_imports]`) when
    /// the generated code contains them.
    pub written: BTreeSet<String>,
    pub kinds: BTreeMap<String, Entry<Kind>>,
    /// Where each kind comes from (ADR-0095 §7).
    pub kind_sources: BTreeMap<String, KindSource>,
    /// The chain members that declared `[dialect] platform` themselves (their kinds are the platform's).
    pub platform_dialects: Vec<String>,
    /// The kind packages of this dialect's platform the loader found (`Loader.packages`), applied as plugins.
    pub packages: Vec<KindPackage>,
    /// Packages of the platform without an implementation for this dialect → the dialects they do have (N0306
    /// when a project enables one).
    pub package_gaps: BTreeMap<String, Vec<String>>,
    /// Folders whose `docs.toml` is read before the chain's (the packages').
    pub extra_docs: Vec<PathBuf>,
    /// `[toolchain]` (ADR-0072): the commands the `dragonfly` CLI runs in the generated project (`install`, `run`,
    /// `test`), with `{{packages}}` for the install's `packages/` folder.
    pub toolchain: BTreeMap<String, String>,
    /// `[toolchain.windows|macos|linux]`: the commands that differ on one operating system (`Dialect::tool`).
    pub toolchain_os: BTreeMap<String, BTreeMap<String, String>>,
    /// `[[toolchain.launch]]`: the editor's debug launches of the generated app, as written (placeholders `{{app}}`,
    /// `{{url}}`); the dragonfly CLI puts them in the project's VS Code workspace.
    pub launches: Vec<toml::Table>,
    /// `[toolchain.settings]` and `[toolchain] extensions`: the editor settings and recommended extensions of a
    /// project of this dialect (its VS Code workspace).
    pub editor_settings: toml::Table,
    pub editor_extensions: Vec<String>,
    /// Classes of the type definitions packages (`[dialect] defs`): class → supertypes and members.
    pub defs: HashMap<String, ClassDef>,
    /// Class of `defs` → the libraries of the package it was read from (to tell whether the class imported
    /// under that name is the same one).
    pub def_libraries: HashMap<String, Arc<Vec<String>>>,
    /// The type definitions packages of this dialect and its parents, with their full API (spec/definitions.md).
    pub typedefs: Arc<TypeDefs>,
    pub modifiers: BTreeMap<String, Entry<Modifier>>,
    pub clauses: BTreeMap<String, Entry<ClauseDef>>,
    pub use_clauses: BTreeSet<String>,
    /// `[decorators] consumed`: decorators the dialect's templates read (`decor`) and that are not written
    /// as decorators of the target.
    pub consumed_decorators: BTreeSet<String>,
    /// `[decorators.provide]`: decorator → how the objects it lists are read inside the declaration
    /// (`provider = "context.read<{{name}}>()"`: with `@provider([CounterManager])`, `CounterManager.x()`
    /// is `context.read<CounterManager>().x()`).
    pub provide: BTreeMap<String, Template>,
    /// `[decorators] void`: a method with one of these decorators returns nothing to its callers (the
    /// template uses its value: `@state` turns it into a state).
    pub void_decorators: BTreeSet<String>,
    /// `[decorators.args]`: decorator → its named arguments (`route`, `loggedIn.forward!`: a field of an
    /// object argument; `!` = required). Any other argument is N0217.
    pub decorator_args: BTreeMap<String, Vec<String>>,
    /// `[decorators.fields]`: decorator of a field → the rules the field must meet (N0308; the relation decorators
    /// of the backend platform, ADR-0100). A platform's `platform.toml` may declare them for all its dialects.
    pub decorator_fields: BTreeMap<String, FieldRule>,
    /// The decorators of the dialect's platform (`decorators` of `platform.toml`): any other one in lower case is N0309.
    /// Empty for a dialect without a platform.
    pub platform_decorators: BTreeSet<String>,
    /// `[decorators.replaced]` of the platform: a removed decorator → what replaces it (`generated` →
    /// `@primary(generated = true)`), for the message of N0309.
    pub replaced_decorators: BTreeMap<String, String>,
    /// `[layout]` of the platform: kind → the path of its file (`components/{component}/domain/usecases/{file}.nuc`;
    /// `{component}`, `{screen}`, `{file}` = the name in snake_case). The same for every dialect of the platform.
    pub layout: BTreeMap<String, String>,
    /// `[skeletons]` of the platform: kind → the smallest file of the kind (`dragonfly make`), with `{{Name}}`, `{{name}}`,
    /// `{{component}}`, `{{Component}}`, `{{table}}`.
    pub skeletons: BTreeMap<String, String>,
    /// `[[generators.<what>]]` of the platform (ADR-0103): the files `dragonfly gen <what>` writes, in order, each a
    /// template over the resource (its names, fields, models, actions, source). The same for every dialect.
    pub generators: BTreeMap<String, Vec<GenPiece>>,
    /// `[gen]` of the platform: how `dragonfly gen` writes an `Http` connection (`http`, and `http_named` when the app
    /// already has one: `{{name}}`, `{{baseUrl}}`).
    pub gen_values: BTreeMap<String, String>,
    pub project: ProjectDef,
    pub architecture: Architecture,
    /// `[docs]`: module prefix → URL of the API reference of a name (`"dart:" = ".../dart-{{library}}/..."`).
    pub docs: BTreeMap<String, Template>,
    /// `[call_site]`: module → (argument, text). Every `Module.method(...)` call also receives the named
    /// argument `arg` with `text` rendered where the call is written (`path`, `line`, `component`, `kind`,
    /// `module`, `method`): a constant, so it costs nothing at run time.
    pub call_site: BTreeMap<String, (String, Template)>,
    /// Every file loaded for this dialect and its parents.
    pub files: Vec<SourceFile>,
    /// `docs.toml` of this dialect and its parents (child first), read on the first `doc()`.
    doc_tables: std::sync::OnceLock<Vec<toml::Table>>,
}

impl Dialect {
    /// URL of the API reference of `name`, imported from `module`, from the `[docs]` entry with the longest
    /// matching prefix. Variables: `name`, `library` (the rest of the module without its extension) and
    /// `class` (the name is PascalCase).
    pub fn docs_url(&self, module: &str, name: &str) -> Option<String> {
        let (prefix, tpl) =
            self.docs.iter().filter(|(p, _)| module.starts_with(p.as_str())).max_by_key(|(p, _)| p.len())?;
        let rest = &module[prefix.len()..];
        let library = match rest.rfind('.') {
            Some(dot) if !rest[dot..].contains('/') => &rest[..dot],
            _ => rest,
        };
        let ctx = Ctx::new()
            .set("name", name)
            .set("library", library)
            .flag("class", name.starts_with(|c: char| c.is_ascii_uppercase()));
        Some(tpl.render(&ctx.into_value()))
    }

    /// Editor documentation of a dialect word (spec/dialects.md §5.7): `section` is `kinds`, `decorators` or
    /// `words` (modifiers and clauses). It is read from `docs.toml` next to `dialect.toml`, only when asked
    /// (a build never reads it); a dialect overrides what it inherits.
    pub fn doc(&self, section: &str, name: &str) -> Option<String> {
        let tables = self.doc_tables.get_or_init(|| {
            let packages = self
                .extra_docs
                .iter()
                .filter_map(|dir| std::fs::read_to_string(dir.join(DOCS_FILE)).ok()?.parse::<toml::Table>().ok());
            packages
                .chain(self.chain.iter().filter_map(|n| {
                    let dir = self
                        .files
                        .iter()
                        .filter_map(|f| Path::new(&f.name).parent())
                        .find(|d| d.file_name().is_some_and(|x| x == n.as_str()))?;
                    std::fs::read_to_string(dir.join(DOCS_FILE)).ok()?.parse::<toml::Table>().ok()
                }))
                .collect()
        });
        // the word's text is Nuclear only (what the agents read); `[underneath.<section>]` of the same file says what
        // the dialect writes under it (its framework), shown after it
        tables.iter().find_map(|t| {
            let text = t.get(section)?.get(name)?.as_str()?.trim().to_string();
            let under = t.get("underneath").and_then(|u| u.get(section)?.get(name)?.as_str()).map(str::trim);
            Some(match under {
                Some(u) if !u.is_empty() => format!("{text}\n\n**Underneath ({}):** {u}", t.get("dialect").and_then(|d| d.as_str()).unwrap_or(&self.name)),
                _ => text,
            })
        })
    }

    /// The names documented in a section of the docs.toml files of the chain, sorted.
    pub fn doc_names(&self, section: &str) -> Vec<String> {
        let _ = self.doc(section, "");
        let mut v: Vec<String> = self
            .doc_tables
            .get()
            .into_iter()
            .flatten()
            .filter_map(|t| t.get(section)?.as_table())
            .flat_map(|t| t.keys().cloned())
            .collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn syntax(&self, key: &str) -> Option<&Template> {
        // options.builtin_result = false: Result comes from a library (auto_imports), not from the parent's runtime
        if key == "result_runtime" && !self.options.builtin_result {
            return None;
        }
        self.syntax.get(key).map(|e| &e.value)
    }

    /// Is `word` reserved in the target language (so a name must be renamed)?
    pub fn is_reserved(&self, word: &str) -> bool {
        self.keywords.get(word).is_some_and(|k| k.level == "reserved" || k.level == "strict")
    }

    /// Required `[syntax]` templates that are missing (empty for a complete target).
    pub fn missing_syntax(&self) -> Vec<&'static str> {
        const OPTIONAL: &[&str] = &[
            "params_optional_group",
            "params_named_group",
            "main",
            "import_alias",
            "import_names",
            "type_imports",
            "super_call",
            "super_param",
            "field_init",
            "file_prelude",
            "super_ctor",
            "local_function",
            "doc_comment",
            "project_import",
            "result_runtime",
            "result_ok",
            "result_err",
            "result_is_ok",
            "result_is_err",
            "result_value",
            "result_error",
            "result_propagate",
            "result_then",
            "result_tap",
            "result_recover",
            "result_tap_error",
            "result_map",
            "result_get_or_else",
        ];
        contract::SYNTAX
            .iter()
            .map(|s| s.key)
            .filter(|k| !OPTIONAL.contains(k) && !self.syntax.contains_key(*k))
            .collect()
    }
}

/// Diagnostics found while loading, each tied to one of `files`.
#[derive(Debug, Default)]
pub struct Report {
    pub files: Vec<SourceFile>,
    pub diags: Vec<(usize, Diagnostic)>,
}

impl Report {
    pub fn has_errors(&self) -> bool {
        self.diags.iter().any(|(_, d)| d.is_error())
    }

    fn add(&mut self, file: usize, d: Diagnostic) {
        self.diags.push((file, d));
    }

    fn file(&mut self, name: &str, text: &str) -> usize {
        self.files.push(SourceFile::new(name, text));
        self.files.len() - 1
    }
}

/// Finds and loads dialects from a list of directories.
pub struct Loader {
    pub search_paths: Vec<PathBuf>,
    /// Folders that hold kind packages (`<project>/kinds`, `<install>/kinds`; ADR-0095).
    pub packages: Vec<PathBuf>,
    /// Extra roots of the type definitions repository, searched before the `defs/` folder next to each
    /// dialect search path (`dialects/` → `defs/`).
    pub defs_paths: Vec<PathBuf>,
}

/// Where a dialect's files are.
struct Located {
    name: String,
    files: Vec<PathBuf>,
    /// `dialects/<platform>/platform.toml` when the dialect lives in a platform folder (ADR-0093).
    platform: Option<PathBuf>,
}

/// The contract file of a platform folder: `dialects/<platform>/platform.toml`.
pub const PLATFORM_FILE: &str = "platform.toml";
/// The header of a kind package (ADR-0095), and of each of its dialect folders.
pub const KIND_FILE: &str = "kind.toml";

/// The kind package in `root` (its `kind.toml` with `[package]`), with the dialects it implements.
pub fn kind_package_at(root: &Path) -> Option<KindPackage> {
    let table = std::fs::read_to_string(root.join(KIND_FILE)).ok()?.parse::<toml::Table>().ok()?;
    let p = table.get("package")?.as_table()?;
    let get = |k: &str| p.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut dialects: Vec<String> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|e| e.path().join(KIND_FILE).is_file())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();
    dialects.sort();
    let name = root.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    Some(KindPackage {
        name,
        version: get("version"),
        platform: get("platform"),
        description: get("description"),
        dialects,
        root: root.to_path_buf(),
    })
}

/// Where each kind of `d` comes from (ADR-0095 §7).
fn kind_sources(d: &mut Dialect) {
    let language = d.chain.last().cloned().unwrap_or_default();
    d.kind_sources = d
        .kinds
        .iter()
        .map(|(k, e)| {
            let origin = &e.origin.dialect;
            let source = if *origin == language {
                KindSource::Language
            } else if let Some(p) = d.packages.iter().find(|p| p.name == *origin) {
                KindSource::Package { name: p.name.clone(), version: p.version.clone(), dialects: p.dialects.clone() }
            } else if d.platform_dialects.contains(origin) {
                KindSource::Platform(d.platform.clone().unwrap_or_default())
            } else {
                // a project dialect: an extension of the dialect it extends
                let parent = d.chain.iter().skip_while(|n| *n != origin).nth(1).cloned().unwrap_or_default();
                KindSource::Package { name: origin.clone(), version: String::new(), dialects: vec![parent] }
            };
            (k.clone(), source)
        })
        .collect();
}

/// A folder with dialect files that is not a platform folder.
fn is_dialect_dir(p: &Path) -> bool {
    p.is_dir() && has_toml(p) && !p.join(PLATFORM_FILE).is_file()
}

impl Loader {
    /// The dialect a new project of `platform` gets when none is named: `default_dialect` of its `platform.toml`.
    pub fn default_dialect(&self, platform: &str) -> Option<String> {
        self.search_paths.iter().find_map(|dir| {
            let text = std::fs::read_to_string(dir.join(platform).join(PLATFORM_FILE)).ok()?;
            let table = text.parse::<toml::Table>().ok()?;
            table.get("platform")?.get("default_dialect")?.as_str().map(str::to_string)
        })
    }
}

/// The platform folders of a search directory: those with a `platform.toml`.
fn platform_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.join(PLATFORM_FILE).is_file()).collect())
        .unwrap_or_default();
    v.sort();
    v
}

impl Loader {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Loader { search_paths, packages: Vec::new(), defs_paths: Vec::new() }
    }

    /// The type definitions repository: `defs_paths`, then `<search path>/../defs`, then the cache of the
    /// definitions generated for projects (`crate::defs_cache_dir`).
    pub fn repository(&self) -> Repository {
        let mut roots: Vec<PathBuf> = Vec::new();
        let dialect_defs = self.search_paths.iter().map(|p| p.join("..").join("defs"));
        for r in self.defs_paths.iter().cloned().chain(dialect_defs).chain(crate::defs_cache_dir()) {
            let key = r.canonicalize().unwrap_or_else(|_| r.clone());
            if r.is_dir() && !roots.iter().any(|x| x.canonicalize().unwrap_or_else(|_| x.clone()) == key) {
                roots.push(r);
            }
        }
        Repository { roots }
    }

    /// Names of every dialect found in the search paths.
    pub fn available(&self) -> Vec<String> {
        let mut out = BTreeSet::new();
        for dir in &self.search_paths {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            for e in entries.flatten() {
                let p = e.path();
                if p.join(PLATFORM_FILE).is_file() {
                    // a platform folder: its dialects
                    if let Ok(sub) = std::fs::read_dir(&p) {
                        out.extend(sub.flatten().map(|e| e.path()).filter(|s| is_dialect_dir(s)).map(|s| file_stem(&s)));
                    }
                } else if is_dialect_dir(&p) || p.extension().is_some_and(|x| x == "toml") {
                    out.insert(file_stem(&p));
                }
            }
        }
        out.into_iter().collect()
    }

    fn locate(&self, name_or_path: &str) -> Option<Located> {
        let direct = Path::new(name_or_path);
        let candidates: Vec<PathBuf> = if direct.exists() {
            vec![direct.to_path_buf()]
        } else {
            self.search_paths
                .iter()
                .flat_map(|d| {
                    let mut v = vec![d.join(name_or_path), d.join(format!("{name_or_path}.toml"))];
                    v.extend(platform_dirs(d).into_iter().map(|p| p.join(name_or_path)));
                    v
                })
                .collect()
        };
        for c in candidates {
            let platform = c.parent().map(|p| p.join(PLATFORM_FILE)).filter(|p| p.is_file());
            if is_dialect_dir(&c) {
                let mut files: Vec<PathBuf> = std::fs::read_dir(&c)
                    .ok()?
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|x| x == "toml") && !p.ends_with(DOCS_FILE))
                    .collect();
                files.sort();
                return Some(Located { name: file_stem(&c), files, platform });
            }
            if c.is_file() {
                return Some(Located { name: file_stem(&c), files: vec![c], platform });
            }
        }
        None
    }

    /// Loads a dialect by name (searched in `search_paths`) or by path.
    pub fn load(&self, name_or_path: &str) -> (Option<Dialect>, Report) {
        let mut report = Report::default();
        let mut stack = Vec::new();
        let mut dialect = self.load_inner(name_or_path, &mut report, &mut stack);
        if let Some(d) = &mut dialect {
            self.apply_packages(d, &mut report);
            kind_sources(d);
            validate(d, &mut report);
        }
        let ok = !report.has_errors();
        (dialect.filter(|_| ok), report)
    }

    /// The kind packages found in `self.packages` (ADR-0095): each folder with a `kind.toml`.
    pub fn kind_packages(&self) -> Vec<KindPackage> {
        let mut out = Vec::new();
        for dir in &self.packages {
            let Ok(entries) = std::fs::read_dir(dir) else { continue };
            let mut roots: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.join(KIND_FILE).is_file()).collect();
            roots.sort();
            out.extend(roots.iter().filter_map(|r| kind_package_at(r)));
        }
        out
    }

    /// The packages of the dialect's platform, applied as plugins: the implementation for this dialect (or a
    /// parent) is loaded as a layer named after the package; a package without one is a gap (N0306 if enabled).
    fn apply_packages(&self, d: &mut Dialect, report: &mut Report) {
        let Some(platform) = d.platform.clone() else { return };
        for pkg in self.kind_packages() {
            if pkg.platform != platform {
                continue;
            }
            let mut plugin = PluginDef { name: pkg.name.clone(), description: pkg.description.clone(), ..Default::default() };
            if let Some(dialect_name) = d.chain.iter().find(|n| pkg.dialects.contains(n)).cloned() {
                let dir = pkg.root.join(&dialect_name);
                let before: BTreeSet<String> = d.kinds.keys().cloned().collect();
                let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
                    .ok()
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|x| x == "toml") && !p.ends_with(DOCS_FILE))
                    .collect();
                files.sort();
                for path in files {
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    let idx = report.file(&path.display().to_string(), &text);
                    let table = match text.parse::<toml::Table>() {
                        Ok(t) => t,
                        Err(e) => {
                            let span = e.span().map_or(Span::default(), |r| Span::new(r.start, r.end));
                            report.add(idx, Diagnostic::error(codes::BAD_DIALECT_FILE, span, format!("invalid TOML: {}", e.message())));
                            continue;
                        }
                    };
                    if let Some(p) = table.get("plugin").and_then(|v| v.as_table()) {
                        let pairs = |k: &str| -> Vec<(String, String)> {
                            p.get(k)
                                .and_then(|v| v.as_table())
                                .map(|t| t.iter().filter_map(|(n, v)| v.as_str().map(|s| (n.clone(), s.to_string()))).collect())
                                .unwrap_or_default()
                        };
                        plugin.packages = pairs("packages");
                        plugin.dev_packages = pairs("dev_packages");
                        plugin.defs = Layer::string_list(p.get("defs"));
                        plugin.names = Layer::string_list(p.get("names"));
                    }
                    let mut filtered = toml::Table::new();
                    for (k, v) in table {
                        if matches!(k.as_str(), "plugin" | "package") {
                            continue;
                        }
                        if matches!(k.as_str(), "dialect" | "project" | "architecture" | "syntax" | "types" | "operators" | "options") {
                            let span = locate_key(&report.files[idx].text, &[], &k);
                            report.add(idx, Diagnostic::error(codes::BAD_DIALECT_FILE, span, format!("a kind package cannot define [{k}]: only kinds, decorators, functions, imports, auto_imports, library, modifiers, clauses and names")));
                            continue;
                        }
                        filtered.insert(k, v);
                    }
                    let mut layer = Layer { dialect: pkg.name.clone(), replaces: BTreeSet::new(), report, d };
                    layer.apply(idx, &filtered);
                }
                plugin.kinds = d.kinds.keys().filter(|k| !before.contains(*k)).cloned().collect();
                d.extra_docs.push(pkg.root.clone());
            } else {
                d.package_gaps.insert(pkg.name.clone(), pkg.dialects.clone());
            }
            d.project.plugins.push(plugin);
            d.packages.push(pkg);
        }
    }

    fn load_inner(&self, name_or_path: &str, report: &mut Report, stack: &mut Vec<String>) -> Option<Dialect> {
        let Some(located) = self.locate(name_or_path) else {
            let f = report.file(name_or_path, "");
            let mut msg = format!("dialect '{name_or_path}' not found");
            let available = self.available();
            if let Some(s) = closest(name_or_path, available.iter().map(String::as_str)) {
                msg.push_str(&format!("; did you mean '{s}'?"));
            } else if !available.is_empty() {
                msg.push_str(&format!(" (available: {})", available.join(", ")));
            }
            report.add(f, Diagnostic::error(codes::DIALECT_NOT_FOUND, Span::default(), msg));
            return None;
        };
        if stack.contains(&located.name) {
            let f = report.file(&located.name, "");
            let cycle = format!("{} -> {}", stack.join(" -> "), located.name);
            report.add(
                f,
                Diagnostic::error(
                    codes::DIALECT_NOT_FOUND,
                    Span::default(),
                    format!("dialect inheritance cycle: {cycle}"),
                ),
            );
            return None;
        }
        stack.push(located.name.clone());

        // parse every file
        let mut parsed = Vec::new();
        for path in &located.files {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            let idx = report.file(&path.display().to_string(), &text);
            match text.parse::<toml::Table>() {
                Ok(table) => parsed.push((idx, table)),
                Err(e) => {
                    let span = e.span().map_or(Span::default(), |r| Span::new(r.start, r.end));
                    report.add(
                        idx,
                        Diagnostic::error(codes::BAD_DIALECT_FILE, span, format!("invalid TOML: {}", e.message())),
                    );
                }
            }
        }

        // [dialect]
        let headers: Vec<&(usize, toml::Table)> = parsed.iter().filter(|(_, t)| t.contains_key("dialect")).collect();
        let mut extends = None;
        let mut replaces = BTreeSet::new();
        let mut meta = toml::Table::new();
        let mut header_file = None;
        match headers.as_slice() {
            [] => {
                let f = parsed.first().map_or_else(|| report.file(&located.name, ""), |p| p.0);
                report.add(
                    f,
                    Diagnostic::error(
                        codes::BAD_DIALECT_FILE,
                        Span::default(),
                        format!("dialect '{}' has no [dialect] section", located.name),
                    ),
                );
            }
            [(idx, table), rest @ ..] => {
                for (other, _) in rest {
                    let span = locate_key(&report.files[*other].text, &[], "dialect");
                    report.add(
                        *other,
                        Diagnostic::error(
                            codes::BAD_DIALECT_FILE,
                            span,
                            "only one file of a dialect can have a [dialect] section",
                        ),
                    );
                }
                if let Some(toml::Value::Table(t)) = table.get("dialect") {
                    meta = t.clone();
                    header_file = Some(*idx);
                    if let Some(name) = t.get("name").and_then(|v| v.as_str())
                        && name != located.name
                    {
                        let span = locate_key(&report.files[*idx].text, &["dialect"], "name");
                        report.add(
                            *idx,
                            Diagnostic::error(
                                codes::BAD_DIALECT_FILE,
                                span,
                                format!("[dialect] name is '{name}' but the dialect is '{}'", located.name),
                            ),
                        );
                    }
                    extends = t.get("extends").and_then(|v| v.as_str()).map(str::to_string);
                    if let Some(toml::Value::Array(list)) = t.get("replaces") {
                        replaces = list.iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
                    }
                }
            }
        }

        let mut d = match &extends {
            Some(parent) => match self.load_inner(parent, report, stack) {
                Some(p) => p,
                None => {
                    stack.pop();
                    return None;
                }
            },
            None => Dialect { indent: "  ".into(), ..Dialect::default() },
        };
        d.name = located.name.clone();
        d.chain.insert(0, located.name.clone());
        if let Some(v) = meta.get("description").and_then(|v| v.as_str()) {
            d.description = v.to_string();
        }
        if let Some(v) = meta.get("file_extension").and_then(|v| v.as_str()) {
            d.file_extension = v.to_string();
        }
        if let Some(v) = meta.get("indent").and_then(|v| v.as_str()) {
            d.indent = v.to_string();
        }
        // the platform this dialect implements (ADR-0093); a child (a project dialect) inherits it
        let own_platform = meta.get("platform").and_then(|v| v.as_str()).map(str::to_string);
        if let Some(p) = &own_platform {
            d.platform = Some(p.clone());
            d.platform_dialects.push(located.name.clone());
        }
        let plugins_before: Vec<String> = d.project.plugins.iter().map(|p| p.name.clone()).collect();
        let mut layer = Layer { dialect: located.name.clone(), replaces, report, d: &mut d };
        for (idx, table) in &parsed {
            layer.apply(*idx, table);
        }
        if let Some(idx) = header_file {
            self.load_defs(&meta, idx, report, &mut d);
            // the type definitions of this layer's plugins: always loaded (only their index), so an editor
            // knows the names; a project that uses them without enabling the plugin gets N0304
            let list: Vec<String> = d
                .project
                .plugins
                .iter()
                .filter(|p| !plugins_before.contains(&p.name))
                .flat_map(|p| p.defs.clone())
                .collect();
            if !list.is_empty() {
                let span = locate_key(&report.files[idx].text, &["project", "plugins"], "");
                self.load_packages(&list, idx, span, report, &mut d, false);
            }
        }
        let raw = d.raw_options.clone();
        d.options = parse_options(&raw);
        d.files = report.files.clone();
        if let Some(platform) = &own_platform {
            check_platform(&mut d, platform, located.platform.as_deref(), report);
        }
        stack.pop();
        Some(d)
    }

    /// `[dialect] defs = ["flutter@3.47.7"]`: loads each package and its `depends` (after it), adds their
    /// classes to `d.defs`, their symbols to `d.auto_imports` and their re-exports. A symbol goes to the
    /// first package that exports it (the parent dialect's packages come first); an `[auto_imports]` or
    /// `[functions]` entry written in a dialect file wins over a package.
    fn load_defs(&self, meta: &toml::Table, file: usize, report: &mut Report, d: &mut Dialect) {
        let Some(value) = meta.get("defs") else { return };
        let span = locate_key(&report.files[file].text, &["dialect"], "defs");
        let Some(list) = value.as_array().filter(|l| l.iter().all(|v| v.is_str())) else {
            let msg = "[dialect] defs must be a list of packages: defs = [\"name@version\"]";
            report.add(file, Diagnostic::error(codes::BAD_DIALECT_FILE, span, msg));
            return;
        };
        let list: Vec<String> = list.iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
        self.load_packages(&list, file, span, report, d, false);
    }

    /// Adds type definitions packages to a loaded dialect: the packages a project uses besides the
    /// dialect's (`hive@2.2.3`, from the project's manifest). Errors (N0213, N0214) go to the report.
    pub fn add_defs(&self, d: &mut Dialect, list: &[String]) -> Report {
        let mut report = Report::default();
        let file = report.file("project defs", &list.join("\n"));
        self.load_packages(list, file, Span::default(), &mut report, d, true);
        report
    }

    fn load_packages(
        &self,
        list: &[String],
        file: usize,
        span: Span,
        report: &mut Report,
        d: &mut Dialect,
        from_project: bool,
    ) {
        let language = d.chain.last().cloned().unwrap_or_default();
        let repo = self.repository();
        let mut pending: Vec<String> = list.iter().rev().cloned().collect();
        let typedefs = Arc::make_mut(&mut d.typedefs);
        while let Some(r) = pending.pop() {
            let Some((name, version)) = defs::parse_ref(&r) else {
                let msg = format!("'{r}' is not a package reference: use name@version");
                report.add(file, Diagnostic::error(codes::BAD_DIALECT_FILE, span, msg));
                continue;
            };
            if let Some(p) = typedefs.package(name) {
                if p.version != version {
                    let msg = format!("type definitions '{r}' conflict with '{}' already loaded", p.id());
                    report.add(file, Diagnostic::error(codes::DEFS_VERSION_CONFLICT, span, msg));
                }
                continue;
            }
            let loaded = match repo.find(&language, name, version) {
                Some(dir) => Package::read(&dir).and_then(|p| p.index().map(|i| (p, i))),
                None => {
                    let mut msg = format!("type definitions '{r}' not found for language '{language}'");
                    let versions = repo.versions(&language, name);
                    if !versions.is_empty() {
                        msg.push_str(&format!(" (available: {})", versions.join(", ")));
                    } else if repo.roots.is_empty() {
                        msg.push_str(" (no defs/ folder next to the dialects; see --defs-path)");
                    }
                    Err(msg)
                }
            };
            let (mut package, index) = match loaded {
                Ok(x) => x,
                Err(msg) => {
                    report.add(file, Diagnostic::error(codes::DEFS_NOT_FOUND, span, msg));
                    continue;
                }
            };
            let pfile = report.file(&package.dir.join("index.json").display().to_string(), "");
            let origin = Origin { dialect: d.name.clone(), file: pfile, span: Span::default() };
            let libraries = Arc::new(index.libraries.clone());
            for (class, def) in index.classes {
                if !d.defs.contains_key(&class) {
                    d.def_libraries.insert(class.clone(), libraries.clone());
                    d.defs.insert(class, def);
                }
            }
            let keywords: BTreeSet<&str> = Keyword::ALL.iter().map(|k| k.as_str()).collect();
            let mut symbols: Vec<(String, usize)> = index.symbols.into_iter().collect();
            symbols.sort();
            for (sym, lib) in symbols {
                if !typedefs.claimed.insert(sym.clone()) {
                    continue;
                }
                let Some(module) = index.libraries.get(lib) else { continue };
                if package.implicit.contains(module) {
                    d.globals.insert(sym);
                    continue;
                }
                if keywords.contains(sym.as_str())
                    || d.auto_imports.contains_key(&sym)
                    || d.functions.contains_key(&sym)
                {
                    continue;
                }
                d.auto_imports.insert(sym, Entry { value: module.clone(), origin: origin.clone() });
            }
            for (module, included) in index.reexports {
                d.reexports.entry(module).or_default().extend(included);
            }
            pending.extend(package.depends.iter().rev().cloned());
            package.from_project = from_project;
            typedefs.push(package);
        }
    }
}

/// The editor documentation of a dialect: it is not part of the dialect's rules (`Dialect::doc`).
pub const DOCS_FILE: &str = "docs.toml";

/// A dialect under `dialects/<platform>/` implements that platform's contract (`platform.toml`, ADR-0093):
/// the same kinds (the language's own kinds aside), no consumed decorator or plugin outside the platform's
/// lists. The platform plugins it does not bring yet are warnings.
fn check_platform(d: &mut Dialect, platform: &str, file: Option<&Path>, report: &mut Report) {
    let shown = file.map_or(PLATFORM_FILE.to_string(), |p| p.display().to_string());
    let idx = report.file(&shown, "");
    let err = |report: &mut Report, msg: String| {
        report.add(idx, Diagnostic::error(codes::PLATFORM_CONTRACT, Span::default(), msg));
    };
    let Some(path) = file else {
        err(
            report,
            format!(
                "dialect '{0}' names platform '{platform}' but is not in its folder: dialects/{platform}/{0}/ next to a {PLATFORM_FILE}",
                d.name
            ),
        );
        return;
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let table = match text.parse::<toml::Table>() {
        Ok(t) => t,
        Err(e) => {
            err(report, format!("{shown}: invalid TOML: {}", e.message()));
            return;
        }
    };
    let Some(p) = table.get("platform").and_then(|v| v.as_table()) else {
        err(report, format!("{shown}: missing [platform]"));
        return;
    };
    let declared = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if declared != platform {
        err(report, format!("dialect '{}' names platform '{platform}' but {shown} declares '{declared}'", d.name));
        return;
    }
    // the rules of the platform's decorators, the same for every dialect (a dialect's own entries win)
    if let Some(decorators) = table.get("decorators").and_then(|v| v.as_table()) {
        for (dec, by) in decorators.get("replaced").and_then(|v| v.as_table()).into_iter().flatten() {
            d.replaced_decorators.insert(dec.clone(), by.as_str().unwrap_or_default().to_string());
        }
        for (dec, list) in decorators.get("args").and_then(|v| v.as_table()).into_iter().flatten() {
            let list = list.as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_string)).collect();
            d.decorator_args.entry(dec.clone()).or_insert(list);
        }
        for (dec, rule) in decorators.get("fields").and_then(|v| v.as_table()).into_iter().flatten() {
            d.decorator_fields.entry(dec.clone()).or_insert_with(|| field_rule(rule));
        }
    }
    let list = |key: &str| -> BTreeSet<String> {
        p.get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let join = |set: Vec<&String>| set.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
    d.platform_decorators = list("decorators");
    // kinds: the language dialect's own kinds (dart's extension, mixin, data) are not the platform's
    let language = d.chain.last().cloned().unwrap_or_default();
    let kinds: BTreeSet<String> =
        d.kinds.iter().filter(|(_, e)| e.origin.dialect != language).map(|(k, _)| k.clone()).collect();
    let want = list("kinds");
    for (kind, path) in table.get("layout").and_then(|v| v.as_table()).into_iter().flatten() {
        if !want.contains(kind) {
            err(report, format!("{shown}: [layout] names '{kind}', which is not a kind of platform '{platform}'"));
        }
        d.layout.insert(kind.clone(), path.as_str().unwrap_or_default().to_string());
    }
    for (key, v) in table.get("gen").and_then(|v| v.as_table()).into_iter().flatten() {
        d.gen_values.insert(key.clone(), v.as_str().unwrap_or_default().to_string());
    }
    for (what, pieces) in table.get("generators").and_then(|v| v.as_table()).into_iter().flatten() {
        for piece in pieces.as_array().into_iter().flatten() {
            let get = |k: &str| piece.get(k).and_then(|v| v.as_str()).map(str::to_string);
            let kind = get("kind").unwrap_or_default();
            if !d.layout.contains_key(&kind) {
                err(report, format!("{shown}: [[generators.{what}]] writes a '{kind}', which has no place in [layout]"));
                continue;
            }
            let mut compile = |src: &str, field: &str| match Template::compile(src) {
                Ok(t) => Some(t),
                Err(e) => {
                    err(report, format!("{shown}: [[generators.{what}]] {kind} {field}: {}", e.message));
                    None
                }
            };
            let (Some(name), Some(template)) = (compile(&get("name").unwrap_or_default(), "name"), compile(get("template").unwrap_or_default().trim_start(), "template")) else {
                continue;
            };
            d.generators.entry(what.clone()).or_default().push(GenPiece {
                kind,
                name,
                when: get("when"),
                each: get("each"),
                merge: piece.get("merge").and_then(|v| v.as_bool()).unwrap_or(false),
                template,
            });
        }
    }
    for (kind, text) in table.get("skeletons").and_then(|v| v.as_table()).into_iter().flatten() {
        if !d.layout.contains_key(kind) {
            err(report, format!("{shown}: [skeletons] names '{kind}', which has no place in [layout]"));
        }
        d.skeletons.insert(kind.clone(), text.as_str().unwrap_or_default().trim_start().to_string());
    }
    let missing = join(want.difference(&kinds).collect());
    if !missing.is_empty() {
        err(report, format!("dialect '{}' misses kinds of platform '{platform}': {missing}", d.name));
    }
    let extra = join(kinds.difference(&want).collect());
    if !extra.is_empty() {
        err(
            report,
            format!(
                "dialect '{}' adds kinds platform '{platform}' does not declare: {extra} (the Nuclear of a platform never changes with its dialect: declare them in {shown} and implement them in every dialect, or drop them)",
                d.name
            ),
        );
    }
    let extra = join(d.consumed_decorators.difference(&list("decorators")).collect());
    if !extra.is_empty() {
        err(report, format!("dialect '{}' consumes decorators platform '{platform}' does not declare: {extra}", d.name));
    }
    let plugins = list("plugins");
    let own: BTreeSet<String> = d.project.plugins.iter().map(|p| p.name.clone()).collect();
    let extra = join(own.difference(&plugins).collect());
    if !extra.is_empty() {
        err(report, format!("dialect '{}' declares plugins platform '{platform}' does not: {extra}", d.name));
    }
    let missing = join(plugins.difference(&own).collect());
    if !missing.is_empty() {
        report.add(
            idx,
            Diagnostic::warning(
                codes::PLATFORM_CONTRACT,
                Span::default(),
                format!("dialect '{}' does not bring these plugins of platform '{platform}' yet: {missing}", d.name),
            ),
        );
    }
}

fn field_rule(v: &toml::Value) -> FieldRule {
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_string);
    FieldRule {
        optional: v.get("optional").and_then(|x| x.as_bool()).unwrap_or(false),
        list: v.get("list").and_then(|x| x.as_bool()),
        target: s("target"),
        sibling: s("sibling"),
        inverse: s("inverse"),
        with: s("with"),
    }
}

fn has_toml(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|it| it.flatten().any(|e| e.path().extension().is_some_and(|x| x == "toml")))
        .unwrap_or(false)
}

fn file_stem(p: &Path) -> String {
    p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Applies one dialect's files on top of what it inherited.
struct Layer<'a> {
    dialect: String,
    replaces: BTreeSet<String>,
    report: &'a mut Report,
    d: &'a mut Dialect,
}

const SECTIONS: &[&str] = &[
    "dialect",
    "options",
    "syntax",
    "types",
    "operators",
    "keywords",
    "imports",
    "auto_imports",
    "reexports",
    "exports",
    "functions",
    "library",
    "names",
    "kinds",
    "modifiers",
    "clauses",
    "use",
    "decorators",
    "project",
    "architecture",
    "docs",
    "call_site",
    "toolchain",
];

impl Layer<'_> {
    fn origin(&self, file: usize, section: &[&str], key: &str) -> Origin {
        let span = locate_key(&self.report.files[file].text, section, key);
        Origin { dialect: self.dialect.clone(), file, span }
    }

    fn error(&mut self, file: usize, code: Code, span: Span, msg: String) {
        self.report.add(file, Diagnostic::error(code, span, msg));
    }

    /// Checks that `section.key` may be defined here. Returns false when it collides.
    fn may_define<T>(&mut self, map: &BTreeMap<String, Entry<T>>, section: &str, key: &str, file: usize) -> bool {
        let Some(existing) = map.get(key) else {
            return true;
        };
        let full = format!("{section}.{key}");
        if existing.origin.dialect != self.dialect && self.replaces.contains(&full) {
            return true;
        }
        let span = locate_key(&self.report.files[file].text, &[section], key);
        let prev = &self.report.files[existing.origin.file];
        let (line, _) = prev.line_col(existing.origin.span.start);
        let msg = if existing.origin.dialect == self.dialect {
            format!("'{full}' is defined twice (also in {}:{line})", prev.name)
        } else {
            format!(
                "'{full}' is already defined by dialect '{}' ({}:{line}); to replace it add \"{full}\" to [dialect] replaces",
                existing.origin.dialect, prev.name
            )
        };
        self.error(file, codes::SHORTCUT_COLLISION, span, msg);
        false
    }

    /// A template written inline (`"..."`) or in its own file (`{ file = "templates/x.tpl" }`, relative
    /// to the dialect file that references it).
    fn template(&mut self, file: usize, section: &[&str], key: &str, value: &toml::Value) -> Option<Template> {
        let what = format!("{}.{key}", section.join("."));
        let key_span = locate_key(&self.report.files[file].text, section, key);
        let (text, text_file) = match value {
            toml::Value::String(s) => (s.clone(), file),
            toml::Value::Table(t) if t.get("file").is_some_and(|f| f.is_str()) => {
                let rel = t["file"].as_str().unwrap();
                let base = Path::new(&self.report.files[file].name).parent().unwrap_or(Path::new("."));
                let path = base.join(rel);
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        // the file's final newline is not part of the template
                        let content = content.strip_suffix('\n').unwrap_or(&content).to_string();
                        let idx = self.report.file(&path.display().to_string(), &content);
                        (content, idx)
                    }
                    Err(e) => {
                        self.error(
                            file,
                            codes::BAD_DIALECT_FILE,
                            key_span,
                            format!("'{what}': cannot read template file {}: {e}", path.display()),
                        );
                        return None;
                    }
                }
            }
            _ => {
                self.error(
                    file,
                    codes::BAD_DIALECT_FILE,
                    key_span,
                    format!("'{what}' must be a string or {{ file = \"path\" }}"),
                );
                return None;
            }
        };
        match Template::compile(&text) {
            Ok(t) => Some(t),
            Err(e) => {
                // inline: point at the key; in a template file: point at the exact place
                let span = if text_file == file { key_span } else { Span::new(e.offset, e.offset + 2) };
                self.error(text_file, codes::BAD_DIALECT_FILE, span, format!("template '{what}': {}", e.message));
                None
            }
        }
    }

    fn apply(&mut self, file: usize, table: &toml::Table) {
        for (section, value) in table {
            if !SECTIONS.contains(&section.as_str()) {
                let span = locate_key(&self.report.files[file].text, &[], section);
                let hint = closest(section, SECTIONS.iter().copied())
                    .map(|s| format!("; did you mean [{s}]?"))
                    .unwrap_or_default();
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("unknown section [{section}]{hint}"));
                continue;
            }
            let Some(t) = value.as_table() else {
                let span = locate_key(&self.report.files[file].text, &[], section);
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("[{section}] must be a table"));
                continue;
            };
            match section.as_str() {
                "dialect" => {}
                "options" => self.options(file, t),
                "syntax" => self.templates(file, t, "syntax", contract::SYNTAX),
                "types" => self.templates(file, t, "types", contract::TYPES),
                "operators" => self.templates(file, t, "operators", contract::OPERATORS),
                "keywords" => self.keywords(t),
                "imports" => self.imports(file, t),
                "auto_imports" => self.auto_imports(file, t),
                "reexports" => {
                    for (module, v) in t {
                        self.d.reexports.entry(module.clone()).or_default().extend(Self::string_list(Some(v)));
                    }
                }
                "exports" => {
                    for (module, v) in t {
                        self.d.exports.entry(module.clone()).or_default().extend(Self::string_list(Some(v)));
                    }
                }
                "functions" => self.functions(file, t),
                "library" => self.library_section(file, t),
                "names" => self.names(file, t),
                "kinds" => self.kinds(file, t),
                "modifiers" => self.modifiers(file, t),
                "clauses" => self.clauses(file, t),
                "use" => self.use_section(file, t),
                "decorators" => self.decorators_section(file, t),
                "project" => self.project_section(file, t),
                "architecture" => self.architecture_section(file, t),
                "toolchain" => {
                    for (k, v) in t {
                        match (k.as_str(), v) {
                            (_, toml::Value::String(cmd)) => {
                                self.d.toolchain.insert(k.clone(), cmd.clone());
                            }
                            // the commands of one operating system, over the others (`.venv\\Scripts` on Windows)
                            ("windows" | "macos" | "linux", toml::Value::Table(os)) => {
                                let map = self.d.toolchain_os.entry(k.clone()).or_default();
                                for (name, cmd) in os {
                                    if let Some(cmd) = cmd.as_str() {
                                        map.insert(name.clone(), cmd.to_string());
                                    }
                                }
                            }
                            // the editor's settings and recommended extensions for a project of this dialect
                            ("settings", toml::Value::Table(set)) => self.d.editor_settings = set.clone(),
                            ("extensions", toml::Value::Array(list)) => {
                                self.d.editor_extensions = list.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
                            }
                            // the editor's debug launches of the generated app (`{{app}}`, `{{url}}`)
                            ("launch", toml::Value::Table(one)) => self.d.launches = vec![one.clone()],
                            ("launch", toml::Value::Array(many)) => {
                                self.d.launches = many.iter().filter_map(|x| x.as_table().cloned()).collect();
                            }
                            _ => {
                                let span = locate_key(&self.report.files[file].text, &["toolchain"], k);
                                self.error(
                                    file,
                                    codes::BAD_DIALECT_FILE,
                                    span,
                                    format!("toolchain.{k} must be a command (a string), an OS table (windows, macos, linux), launch, settings or extensions"),
                                );
                            }
                        }
                    }
                }
                "call_site" => {
                    for (module, v) in t {
                        let arg = v.get("arg").and_then(|a| a.as_str());
                        match (arg, v.get("text")) {
                            (Some(arg), Some(text)) => {
                                if let Some(tpl) = self.template(file, &["call_site", module], "text", text) {
                                    self.d.call_site.insert(module.clone(), (arg.to_string(), tpl));
                                }
                            }
                            _ => {
                                let span = locate_key(&self.report.files[file].text, &["call_site"], module);
                                self.error(
                                    file,
                                    codes::BAD_DIALECT_FILE,
                                    span,
                                    format!("call_site.{module} needs `arg` and `text`"),
                                );
                            }
                        }
                    }
                }
                "docs" => {
                    for (prefix, v) in t {
                        if let Some(tpl) = self.template(file, &["docs"], prefix, v) {
                            self.d.docs.insert(prefix.clone(), tpl);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn options(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            if !contract::OPTIONS.iter().any(|o| o.key == k) {
                let span = locate_key(&self.report.files[file].text, &["options"], k);
                let hint = closest(k, contract::OPTIONS.iter().map(|o| o.key))
                    .map(|s| format!("; did you mean '{s}'?"))
                    .unwrap_or_default();
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("unknown option '{k}'{hint}"));
                continue;
            }
            self.d.raw_options.insert(k.clone(), v.clone());
        }
    }

    fn templates(&mut self, file: usize, t: &toml::Table, section: &str, specs: &[contract::TemplateSpec]) {
        let types_section = section == "types";
        for (k, v) in t {
            // [types] also accepts any simple type name (`Widget = "Widget"`)
            if !types_section && !specs.iter().any(|s| s.key == k) {
                let span = locate_key(&self.report.files[file].text, &[section], k);
                let hint = closest(k, specs.iter().map(|s| s.key))
                    .map(|s| format!("; did you mean '{s}'?"))
                    .unwrap_or_default();
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("unknown template '{section}.{k}'{hint}"));
                continue;
            }
            // `{ template = "...", helper = "..." }`: the helper is written once per file that uses the template
            let value = match v {
                toml::Value::Table(tb) if tb.get("template").is_some() => {
                    if let Some(h) = tb.get("helper")
                        && let Some(helper) = self.template(file, &[section, k], "helper", h)
                    {
                        self.d.syntax_helpers.insert(k.clone(), helper);
                    }
                    &tb["template"]
                }
                other => other,
            };
            if let Some(tpl) = self.template(file, &[section], k, value) {
                let origin = self.origin(file, &[section], k);
                let entry = Entry { value: tpl, origin };
                match section {
                    "syntax" => self.d.syntax.insert(k.clone(), entry),
                    "types" => self.d.types.insert(k.clone(), entry),
                    _ => self.d.operators.insert(k.clone(), entry),
                };
            }
        }
    }

    fn keywords(&mut self, t: &toml::Table) {
        for (k, v) in t {
            let get = |f: &str| v.get(f).and_then(|x| x.as_str()).unwrap_or("").to_string();
            self.d.keywords.insert(k.clone(), KeywordInfo { level: get("level"), via: get("via") });
        }
    }

    fn imports(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            let shortcut = match v {
                toml::Value::String(s)
                    if s.trim_start().starts_with("import ") || s.contains(';') || s.contains('\n') =>
                {
                    ImportShortcut::Line(s.clone())
                }
                toml::Value::String(s) => ImportShortcut::Module(s.clone()),
                toml::Value::Table(tb) => {
                    match (tb.get("module").and_then(|x| x.as_str()), tb.get("line").and_then(|x| x.as_str())) {
                        (Some(m), _) => ImportShortcut::Module(m.into()),
                        (None, Some(l)) => ImportShortcut::Line(l.into()),
                        _ => {
                            let span = locate_key(&self.report.files[file].text, &["imports"], k);
                            self.error(
                                file,
                                codes::BAD_DIALECT_FILE,
                                span,
                                format!("imports.{k} needs `module` or `line`"),
                            );
                            continue;
                        }
                    }
                }
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["imports"], k);
                    self.error(file, codes::BAD_DIALECT_FILE, span, format!("imports.{k} must be a string or a table"));
                    continue;
                }
            };
            if self.may_define(&self.d.imports.clone(), "imports", k, file) {
                let origin = self.origin(file, &["imports"], k);
                self.d.imports.insert(k.clone(), Entry { value: shortcut, origin });
            }
        }
    }

    fn auto_imports(&mut self, file: usize, t: &toml::Table) {
        for (module, v) in t {
            let Some(list) = v.as_array() else {
                let span = locate_key(&self.report.files[file].text, &["auto_imports"], module);
                self.error(
                    file,
                    codes::BAD_DIALECT_FILE,
                    span,
                    format!("auto_imports.\"{module}\" must be a list of names"),
                );
                continue;
            };
            let text = self.report.files[file].text.clone();
            let module_pos = locate_key(&text, &["auto_imports"], module).start;
            for name in list.iter().filter_map(|x| x.as_str()) {
                let span = find_word_after(&text, module_pos, name).unwrap_or(Span::new(module_pos, module_pos));
                if let Some(existing) = self.d.auto_imports.get(name) {
                    let full = format!("auto_imports.{name}");
                    if existing.origin.dialect != self.dialect && self.replaces.contains(&full) {
                        // replaced below
                    } else {
                        let prev = &self.report.files[existing.origin.file];
                        let (line, _) = prev.line_col(existing.origin.span.start);
                        let msg = format!(
                            "'{name}' is auto-imported from '{module}' and from '{}' ({}:{line}); a symbol can come from one module only",
                            existing.value, prev.name
                        );
                        self.error(file, codes::SHORTCUT_COLLISION, span, msg);
                        continue;
                    }
                }
                let origin = Origin { dialect: self.dialect.clone(), file, span };
                self.d.auto_imports.insert(name.to_string(), Entry { value: module.clone(), origin });
            }
        }
    }

    fn functions(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            let f = match v {
                toml::Value::String(s) => FunctionShortcut::Rename(s.clone()),
                toml::Value::Table(tb) => match tb.get("template") {
                    Some(tv) => match self.template(file, &["functions", k], "template", tv) {
                        Some(tpl) => {
                            if let Some(h) = tb.get("helper")
                                && let Some(helper) = self.template(file, &["functions", k], "helper", h)
                            {
                                self.d.function_helpers.insert(k.clone(), helper);
                            }
                            FunctionShortcut::Template(tpl)
                        }
                        None => continue,
                    },
                    None => {
                        let span = locate_key(&self.report.files[file].text, &["functions"], k);
                        self.error(file, codes::BAD_DIALECT_FILE, span, format!("functions.{k} needs `template`"));
                        continue;
                    }
                },
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["functions"], k);
                    self.error(
                        file,
                        codes::BAD_DIALECT_FILE,
                        span,
                        format!("functions.{k} must be a string or a table"),
                    );
                    continue;
                }
            };
            if self.may_define(&self.d.functions.clone(), "functions", k, file) {
                let origin = self.origin(file, &["functions"], k);
                self.d.functions.insert(k.clone(), Entry { value: f, origin });
            }
        }
    }

    fn names(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            let list: Vec<String> = v
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            match k.as_str() {
                "globals" => self.d.globals.extend(list),
                "not_constructors" => self.d.not_constructors.extend(list),
                "written" => self.d.written.extend(list),
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["names"], k);
                    self.error(
                        file,
                        codes::BAD_DIALECT_FILE,
                        span,
                        format!("unknown key names.{k} (expected globals or not_constructors)"),
                    );
                }
            }
        }
    }

    fn string_list(v: Option<&toml::Value>) -> Vec<String> {
        v.and_then(|x| x.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    }

    fn kinds(&mut self, file: usize, t: &toml::Table) {
        const KEYS: &[&str] = &[
            "description",
            "template",
            "overrides",
            "requires",
            "members",
            "param_access",
            "imports",
            "rename",
            "syntax",
            "named_params",
            "inherits",
            "async_methods",
            "type_imports",
            "methods",
            "returns",
            "file_header",
            "alias_via",
            "declares",
            "method_members",
            "provide",
            "results",
            "void_methods",
            "needs_body",
        ];
        for (k, v) in t {
            let Some(tb) = v.as_table() else { continue };
            for key in tb.keys() {
                if !KEYS.contains(&key.as_str()) {
                    let span = locate_key(&self.report.files[file].text, &["kinds", k], key);
                    self.error(
                        file,
                        codes::BAD_DIALECT_FILE,
                        span,
                        format!("unknown key kinds.{k}.{key} (expected one of: {})", KEYS.join(", ")),
                    );
                }
            }
            let Some(tv) = tb.get("template") else {
                let span = locate_key(&self.report.files[file].text, &["kinds"], k);
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("kinds.{k} needs a `template`"));
                continue;
            };
            let Some(template) = self.template(file, &["kinds", k], "template", tv) else {
                continue;
            };
            let param_access = match tb.get("param_access") {
                Some(pv) => self.template(file, &["kinds", k], "param_access", pv),
                None => None,
            };
            let rename = match tb.get("rename") {
                Some(rv) => self.template(file, &["kinds", k], "rename", rv),
                None => None,
            };
            let mut syntax = BTreeMap::new();
            if let Some(toml::Value::Table(st)) = tb.get("syntax") {
                for (sk, sv) in st {
                    if contract::syntax_spec(sk).is_none() {
                        let span = locate_key(&self.report.files[file].text, &["kinds", k, "syntax"], sk);
                        self.error(
                            file,
                            codes::BAD_DIALECT_FILE,
                            span,
                            format!("unknown template 'kinds.{k}.syntax.{sk}'"),
                        );
                        continue;
                    }
                    if let Some(tpl) = self.template(file, &["kinds", k, "syntax"], sk, sv) {
                        syntax.insert(sk.clone(), tpl);
                    }
                }
            }
            let kind = Kind {
                description: tb.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                template,
                overrides: Self::string_list(tb.get("overrides")),
                requires: Self::string_list(tb.get("requires")),
                members: Self::string_list(tb.get("members")),
                param_access,
                imports: Self::string_list(tb.get("imports")),
                rename,
                syntax,
                named_params: tb.get("named_params").and_then(|x| x.as_bool()).unwrap_or(false),
                inherits: Self::string_list(tb.get("inherits")),
                async_methods: tb.get("async_methods").and_then(|x| x.as_bool()).unwrap_or(false),
                type_imports: tb.get("type_imports").and_then(|x| x.as_bool()).unwrap_or(false),
                methods: Self::string_list(tb.get("methods")),
                returns: tb.get("returns").and_then(|x| x.as_str()).map(str::to_string),
                file_header: match tb.get("file_header") {
                    Some(fv) => self.template(file, &["kinds", k], "file_header", fv),
                    None => None,
                },
                alias_via: tb.get("alias_via").and_then(|x| x.as_str()).map(str::to_string),
                declares: Self::string_list(tb.get("declares"))
                    .iter()
                    .filter_map(|t| Template::compile(t).ok())
                    .collect(),
                method_members: Self::string_list(tb.get("method_members"))
                    .iter()
                    .filter_map(|t| Template::compile(t).ok())
                    .collect(),
                provide: match tb.get("provide").and_then(|p| p.as_table()) {
                    Some(pt) => match pt.get("template") {
                        Some(tv) => self
                            .template(file, &["kinds", k, "provide"], "template", tv)
                            .map(|t| (t, Self::string_list(pt.get("kinds")))),
                        None => None,
                    },
                    None => None,
                },
                keep_results: tb.get("results").and_then(|x| x.as_str()) == Some("keep"),
                void_methods: tb.get("void_methods").and_then(|x| x.as_bool()).unwrap_or(false),
                needs_body: tb.get("needs_body").and_then(|x| x.as_bool()).unwrap_or(false),
            };
            if self.may_define(&self.d.kinds.clone(), "kinds", k, file) {
                let origin = self.origin(file, &["kinds"], k);
                self.d.kinds.insert(k.clone(), Entry { value: kind, origin });
            }
        }
    }

    fn modifiers(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            let m = match v {
                toml::Value::String(prefix) => Modifier { prefix: prefix.clone(), applies_to: Vec::new(), void: false },
                toml::Value::Table(tb) => Modifier {
                    prefix: tb.get("prefix").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    applies_to: Self::string_list(tb.get("applies_to")),
                    void: tb.get("void").and_then(|x| x.as_bool()).unwrap_or(false),
                },
                _ => continue,
            };
            if self.may_define(&self.d.modifiers.clone(), "modifiers", k, file) {
                let origin = self.origin(file, &["modifiers"], k);
                self.d.modifiers.insert(k.clone(), Entry { value: m, origin });
            }
        }
    }

    fn clauses(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            let tv = match v {
                // `with = { file = "..." }` is itself the template
                toml::Value::Table(tb) if tb.contains_key("file") => Some(v.clone()),
                toml::Value::Table(tb) => tb.get("template").cloned(),
                other => Some(other.clone()),
            };
            let Some(tv) = tv else { continue };
            let Some(template) = self.template(file, &["clauses", k], "template", &tv) else {
                continue;
            };
            if self.may_define(&self.d.clauses.clone(), "clauses", k, file) {
                let origin = self.origin(file, &["clauses"], k);
                self.d.clauses.insert(k.clone(), Entry { value: ClauseDef { template }, origin });
            }
        }
    }

    /// `[library.<base>]`: member = template, or `{ template = "...", helper = "..." }`. The base is one of
    /// `library::BASES` or `library::OBJECTS`, the member one of `library::MEMBERS` (N0206 otherwise).
    fn library_section(&mut self, file: usize, t: &toml::Table) {
        for (base, members) in t {
            let Some(members) = members.as_table() else {
                let span = locate_key(&self.report.files[file].text, &["library"], base);
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("[library.{base}] must be a table of members"));
                continue;
            };
            if !crate::library::BASES.contains(&base.as_str()) && !crate::library::OBJECTS.contains(&base.as_str()) {
                let span = locate_key(&self.report.files[file].text, &["library"], base);
                let known = crate::library::BASES.iter().chain(crate::library::OBJECTS).copied().collect::<Vec<_>>().join(", ");
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("unknown library type '{base}' (expected {known})"));
                continue;
            }
            for (name, v) in members {
                // a member Nuclear does not define is the dialect's own extension of the base type (a platform's
                // `"key".tr` on strings): rendered like the others, its result type unknown
                if crate::library::member_of(base, name).is_none() && !crate::library::BASES.contains(&base.as_str()) {
                    let span = locate_key(&self.report.files[file].text, &["library", base], name);
                    self.error(file, codes::BAD_DIALECT_FILE, span, format!("'{name}' is not a member of Nuclear's {base} (spec/library.md)"));
                    continue;
                }
                let (tv, hv) = match v {
                    toml::Value::Table(tbl) => (tbl.get("template"), tbl.get("helper")),
                    other => (Some(other), None),
                };
                let Some(tv) = tv else {
                    let span = locate_key(&self.report.files[file].text, &["library", base], name);
                    self.error(file, codes::BAD_DIALECT_FILE, span, format!("library.{base}.{name} needs a template"));
                    continue;
                };
                let Some(template) = self.template(file, &["library", base], name, tv) else { continue };
                let helper = hv.and_then(|h| self.template(file, &["library", base], name, h));
                let origin = self.origin(file, &["library", base], name);
                self.d
                    .library
                    .entry(base.clone())
                    .or_default()
                    .insert(name.clone(), Entry { value: LibMember { template, helper }, origin });
            }
        }
    }

    fn decorators_section(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            match k.as_str() {
                "consumed" => self.d.consumed_decorators.extend(Self::string_list(Some(v))),
                "provide" => {
                    for (dec, tv) in v.as_table().cloned().unwrap_or_default() {
                        if let Some(t) = self.template(file, &["decorators", "provide"], &dec, &tv) {
                            self.d.provide.insert(dec, t);
                        }
                    }
                }
                "void" => self.d.void_decorators.extend(Self::string_list(Some(v))),
                "args" => {
                    for (dec, list) in v.as_table().cloned().unwrap_or_default() {
                        self.d.decorator_args.insert(dec, Self::string_list(Some(&list)));
                    }
                }
                "fields" => {
                    for (dec, rule) in v.as_table().cloned().unwrap_or_default() {
                        self.d.decorator_fields.insert(dec, field_rule(&rule));
                    }
                }
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["decorators"], k);
                    let msg = format!("unknown key decorators.{k} (expected consumed, provide, void, args, fields)");
                    self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                }
            }
        }
    }

    fn project_section(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            match (k.as_str(), v) {
                ("sources", toml::Value::String(src)) => self.d.project.sources = src.trim_matches('/').to_string(),
                ("marker", toml::Value::String(m)) => self.d.project.marker = m.clone(),
                ("output", toml::Value::String(o)) => self.d.project.output = o.trim_matches('/').to_string(),
                ("assets", toml::Value::Array(_)) => {
                    let dirs = Self::string_list(Some(v));
                    self.d.project.assets = dirs.into_iter().map(|d| d.trim_matches('/').to_string()).collect();
                }
                ("outputs", toml::Value::Table(outs)) => {
                    for (from, to) in outs {
                        if let Some(to) = to.as_str() {
                            self.d
                                .project
                                .outputs
                                .push((from.trim_matches('/').to_string(), to.trim_matches('/').to_string()));
                        }
                    }
                }
                ("kinds", toml::Value::Table(kinds)) => {
                    for (folder, kind) in kinds {
                        if let Some(kind) = kind.as_str() {
                            self.d.project.kinds.push((folder.clone(), kind.to_string()));
                        }
                    }
                }
                ("files", toml::Value::Table(files)) => {
                    for (path, fv) in files {
                        let Some(ft) = fv.as_table() else { continue };
                        let Some(tv) = ft.get("template") else {
                            let span = locate_key(&self.report.files[file].text, &["project", "files"], path);
                            let msg = format!("project.files.\"{path}\" needs a `template`");
                            self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                            continue;
                        };
                        let Some(template) = self.template(file, &["project", "files"], path, tv) else { continue };
                        let names = Self::string_list(ft.get("names"));
                        self.d.project.files.retain(|f| &f.path != path);
                        self.d.project.files.push(ProjectFileDef { path: path.clone(), template, names });
                    }
                }
                ("plugins", toml::Value::Table(plugins)) => {
                    for (name, pv) in plugins {
                        let Some(pt) = pv.as_table() else { continue };
                        let pairs = |key: &str| -> Vec<(String, String)> {
                            pt.get(key)
                                .and_then(|v| v.as_table())
                                .map(|t| t.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
                                .unwrap_or_default()
                        };
                        let known = ["description", "packages", "dev_packages", "defs", "kinds", "names", "requires"];
                        for k in pt.keys().filter(|k| !known.contains(&k.as_str())) {
                            let span = locate_key(&self.report.files[file].text, &["project", "plugins", name], k);
                            let msg = format!("unknown key project.plugins.{name}.{k} (expected {})", known.join(", "));
                            self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                        }
                        let plugin = PluginDef {
                            name: name.clone(),
                            description: pt.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            packages: pairs("packages"),
                            dev_packages: pairs("dev_packages"),
                            defs: Self::string_list(pt.get("defs")),
                            kinds: Self::string_list(pt.get("kinds")),
                            names: Self::string_list(pt.get("names")),
                            requires: Self::string_list(pt.get("requires")),
                        };
                        self.d.project.plugins.retain(|p| &p.name != name);
                        self.d.project.plugins.push(plugin);
                    }
                }
                ("manifest", toml::Value::Table(m)) => {
                    for k in m.keys().filter(|k| !matches!(k.as_str(), "file" | "sections" | "packages" | "reserved")) {
                        let span = locate_key(&self.report.files[file].text, &["project", "manifest"], k);
                        let msg =
                            format!("unknown key project.manifest.{k} (expected file, sections, packages, reserved)");
                        self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                    }
                    self.d.project.manifest = Some(ManifestDef {
                        file: m.get("file").and_then(|v| v.as_str()).unwrap_or("config.nuc").to_string(),
                        sections: Self::string_list(m.get("sections")),
                        packages: Self::string_list(m.get("packages")),
                        reserved: Self::string_list(m.get("reserved")),
                    });
                }
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["project"], k);
                    let msg = format!(
                        "unknown key project.{k} (expected sources, assets, marker, output and the tables kinds, files, outputs, manifest, plugins)"
                    );
                    self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                }
            }
        }
    }

    fn architecture_section(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            match (k.as_str(), v) {
                ("scope", _) => self.d.architecture.scope = v.as_str().map(str::to_string),
                ("public", _) => self.d.architecture.public = Self::string_list(Some(v)),
                ("exempt", _) => self.d.architecture.exempt = Self::string_list(Some(v)),
                ("callers", toml::Value::Table(c)) => {
                    for (kind, list) in c {
                        self.d.architecture.callers.insert(kind.clone(), Self::string_list(Some(list)));
                    }
                }
                ("names", toml::Value::Table(c)) => {
                    for (name, list) in c {
                        self.d.architecture.names.insert(name.clone(), Self::string_list(Some(list)));
                    }
                }
                _ => {
                    let span = locate_key(&self.report.files[file].text, &["architecture"], k);
                    let msg = format!("unknown key architecture.{k} (expected scope, public, exempt, callers, names)");
                    self.error(file, codes::BAD_DIALECT_FILE, span, msg);
                }
            }
        }
    }

    fn use_section(&mut self, file: usize, t: &toml::Table) {
        for (k, v) in t {
            if k == "clauses" {
                self.d.use_clauses.extend(Self::string_list(Some(v)));
            } else {
                let span = locate_key(&self.report.files[file].text, &["use"], k);
                self.error(file, codes::BAD_DIALECT_FILE, span, format!("unknown key use.{k} (expected clauses)"));
            }
        }
    }
}

fn parse_options(t: &toml::Table) -> Options {
    let mut o = Options::default();
    let b = |k: &str, d: bool| t.get(k).and_then(|v| v.as_bool()).unwrap_or(d);
    let s = |k: &str, d: &str| t.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
    let n = |k: &str, d: usize| t.get(k).and_then(|v| v.as_integer()).map_or(d, |i| i.max(0) as usize);
    o.implicit_self = b("implicit_self", o.implicit_self);
    o.named_args_native = s("named_args", "object") == "native";
    o.declare_ctor_fields = b("declare_ctor_fields", o.declare_ctor_fields);
    o.const_collection_defaults = b("const_collection_defaults", o.const_collection_defaults);
    o.optional_params_nullable = b("optional_params_nullable", o.optional_params_nullable);
    o.number_separators = b("number_separators", o.number_separators);
    o.binary_literals = b("binary_literals", o.binary_literals);
    o.int_member_parens = b("int_member_parens", o.int_member_parens);
    o.string_quote = s("string_quote", &o.string_quote);
    o.interp_quote = s("interp_quote", &o.interp_quote);
    o.interp_open = s("interp_open", &o.interp_open);
    o.interp_close = s("interp_close", &o.interp_close);
    o.dollar_escape = s("dollar_escape", &o.dollar_escape);
    o.reserved_suffix = s("reserved_escape", "suffix") != "prefix";
    o.max_bases = n("max_bases", o.max_bases);
    o.catch_var = s("catch_var", &o.catch_var);
    o.toplevel_main = s("toplevel_statements", "allow") == "main";
    o.line_width = n("line_width", o.line_width);
    o.trailing_comma = b("trailing_comma", o.trailing_comma);
    o.omit_null_default = b("omit_null_default", o.omit_null_default);
    o.interp_bare_names = b("interp_bare_names", o.interp_bare_names);
    o.cascade_statements = b("cascade_statements", o.cascade_statements);
    o.super_in_initializer = b("super_in_initializer", o.super_in_initializer);
    o.implicit_base = s("implicit_base", "");
    o.interp_prefix = s("interp_prefix", "");
    o.interp_brace_escape = b("interp_brace_escape", o.interp_brace_escape);
    o.module_separator = s("module_separator", &o.module_separator);
    o.toplevel_gap = n("toplevel_gap", o.toplevel_gap).max(1);
    o.hoist_block_lambdas = b("hoist_block_lambdas", o.hoist_block_lambdas);
    o.init_fields = b("init_fields", o.init_fields);
    o.type_args_as_args = match t.get("type_args_as_args") {
        Some(toml::Value::Boolean(true)) => TypeArgsAsArgs::All,
        Some(toml::Value::Array(a)) => {
            TypeArgsAsArgs::Names(a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        }
        _ => TypeArgsAsArgs::None,
    };
    o.builtin_result = b("builtin_result", o.builtin_result);
    o.doc_inside = b("doc_inside", o.doc_inside);
    o.snake_case_names = b("snake_case_names", o.snake_case_names);
    o.route_param = s("route_param", &o.route_param);
    o
}

/// Checks that do not depend on a single file: name collisions across sections, names that are base
/// keywords, very similar names and unknown template variables.
fn validate(d: &Dialect, report: &mut Report) {
    let top = &d.name;
    let base_keywords: BTreeSet<&str> = Keyword::ALL.iter().map(|k| k.as_str()).collect();

    // header words share one namespace
    let mut header: Vec<(&str, &str, &Origin)> = Vec::new();
    header.extend(d.kinds.iter().map(|(k, e)| (k.as_str(), "kinds", &e.origin)));
    header.extend(d.modifiers.iter().map(|(k, e)| (k.as_str(), "modifiers", &e.origin)));
    header.extend(d.clauses.iter().map(|(k, e)| (k.as_str(), "clauses", &e.origin)));
    collide_across(&header, top, report, "a header word");

    // names that rewrite identifiers
    let mut idents: Vec<(&str, &str, &Origin)> = Vec::new();
    idents.extend(d.functions.iter().map(|(k, e)| (k.as_str(), "functions", &e.origin)));
    idents.extend(d.auto_imports.iter().map(|(k, e)| (k.as_str(), "auto_imports", &e.origin)));
    collide_across(&idents, top, report, "an identifier shortcut");

    // base keywords
    let all = header
        .iter()
        .chain(idents.iter())
        .chain(d.imports.iter().map(|(k, e)| (k.as_str(), "imports", &e.origin)).collect::<Vec<_>>().iter())
        .cloned()
        .collect::<Vec<_>>();
    for (name, section, origin) in &all {
        if base_keywords.contains(name) && origin.dialect == *top {
            report.add(
                origin.file,
                Diagnostic::error(
                    codes::SHORTCUT_IS_KEYWORD,
                    origin.span,
                    format!("'{section}.{name}' uses the base keyword '{name}' as its name"),
                ),
            );
        }
    }

    // similar names (typo traps) in the shortcut namespaces written by people
    for group in [
        d.imports.iter().map(|(k, e)| (k.as_str(), &e.origin)).collect::<Vec<_>>(),
        d.functions.iter().map(|(k, e)| (k.as_str(), &e.origin)).collect::<Vec<_>>(),
        header.iter().map(|(k, _, o)| (*k, *o)).collect::<Vec<_>>(),
    ] {
        for (i, (a, oa)) in group.iter().enumerate() {
            for (b, ob) in &group[i + 1..] {
                if oa.dialect != *top && ob.dialect != *top {
                    continue;
                }
                if similar(a, b) {
                    let (name, other, o) = if ob.dialect == *top { (b, a, ob) } else { (a, b, oa) };
                    report.add(
                        o.file,
                        Diagnostic::warning(
                            codes::SIMILAR_SHORTCUT,
                            o.span,
                            format!("'{name}' is very similar to '{other}'"),
                        ),
                    );
                }
            }
        }
    }

    // template variables
    for (key, e) in &d.syntax {
        if e.origin.dialect != *top {
            continue;
        }
        if let Some(spec) = contract::syntax_spec(key) {
            check_vars(&e.value, spec.vars, &format!("syntax.{key}"), &e.origin, report);
        }
    }
    for (key, e) in &d.types {
        if e.origin.dialect != *top {
            continue;
        }
        let vars = contract::TYPES.iter().find(|s| s.key == key).map_or(&[][..], |s| s.vars);
        check_vars(&e.value, vars, &format!("types.{key}"), &e.origin, report);
    }
    for (key, e) in &d.kinds {
        if e.origin.dialect != *top {
            continue;
        }
        check_vars(&e.value.template, contract::KIND_VARS, &format!("kinds.{key}.template"), &e.origin, report);
    }
}

fn check_vars(t: &Template, allowed: &[&str], what: &str, origin: &Origin, report: &mut Report) {
    for v in t.used_vars() {
        if v.depth == 0 && !allowed.contains(&v.name.as_str()) && v.name != "." && !v.name.starts_with('@') {
            let hint =
                closest(&v.name, allowed.iter().copied()).map(|s| format!("; did you mean '{s}'?")).unwrap_or_default();
            report.add(
                origin.file,
                Diagnostic::warning(
                    codes::UNKNOWN_TEMPLATE_VAR,
                    origin.span,
                    format!("{what} uses '{{{{{}}}}}', which the transpiler does not provide{hint}", v.name),
                ),
            );
        }
    }
}

fn collide_across(names: &[(&str, &str, &Origin)], top: &str, report: &mut Report, what: &str) {
    for (i, (a, sa, oa)) in names.iter().enumerate() {
        for (b, sb, ob) in &names[i + 1..] {
            if a == b && sa != sb && (oa.dialect == top || ob.dialect == top) {
                let o = if ob.dialect == top { ob } else { oa };
                report.add(
                    o.file,
                    Diagnostic::error(
                        codes::SHORTCUT_COLLISION,
                        o.span,
                        format!("'{a}' is defined in [{sa}] and in [{sb}]; {what} must be unique"),
                    ),
                );
            }
        }
    }
}

/// Same name ignoring case, or (for names of 5+ chars) one edit apart.
pub fn similar(a: &str, b: &str) -> bool {
    if a == b {
        return false;
    }
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    a.len().min(b.len()) >= 6 && levenshtein(a, b) == 1
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The closest candidate within 2 edits, for "did you mean" hints.
pub fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .map(|c| (levenshtein(word, c), c))
        .filter(|(d, c)| *d <= 2 && *d < c.len().max(1) && *c != word)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

/// Best-effort position of `key` inside `[section]` of a TOML text.
pub fn locate_key(text: &str, section: &[&str], key: &str) -> Span {
    let header_names = |sec: &[&str]| -> Vec<String> {
        if sec.is_empty() {
            return Vec::new();
        }
        let dotted = sec.join(".");
        vec![format!("[{dotted}]")]
    };
    // a table of its own: [section.key]
    let mut own = section.to_vec();
    own.push(key);
    for h in header_names(&own) {
        if let Some(p) = text.find(&h) {
            return Span::new(p, p + h.len());
        }
    }
    let start = header_names(section).iter().find_map(|h| text.find(h.as_str()).map(|p| p + h.len())).unwrap_or(0);
    let mut offset = start;
    for line in text[start..].split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') && offset != start && !section.is_empty() {
            break;
        }
        let lead = line.len() - trimmed.len();
        for candidate in [key.to_string(), format!("\"{key}\"")] {
            if let Some(rest) = trimmed.strip_prefix(candidate.as_str())
                && rest.trim_start().starts_with(['=', '.'])
            {
                let s = offset + lead;
                return Span::new(s, s + candidate.len());
            }
        }
        offset += line.len();
    }
    Span::new(start, start)
}

/// Position of `"name"` after `from` (for items inside a list).
fn find_word_after(text: &str, from: usize, name: &str) -> Option<Span> {
    let needle = format!("\"{name}\"");
    text[from..].find(&needle).map(|p| Span::new(from + p, from + p + needle.len()))
}
