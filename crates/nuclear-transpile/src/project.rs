//! Projects: a folder of `.nuc` files with a `nuclear.toml` manifest, transpiled together
//! (spec/projects.md).
//!
//! Every file sees the declarations of the others: their names are known (no N0106), aliases can point
//! to modules of other files, Result propagates through their functions, and each file imports the files
//! it uses. The dialect decides the rest: where the files go (`[project] sources`), the kind of the
//! declarations of each folder (`[project] kinds`), files generated from every declaration
//! (`[project.files]`) and the rules between declarations (`[architecture]`).

use crate::alias;
use crate::emit::{DeclKind, DeclSummary, Emitter, ExtDecl, ProjectCtx, RESULT_PATH};
use nuclear_diag::{Diagnostic, Span, codes};
use nuclear_dialect::{Dialect, Loader, model::KindSource};
use nuclear_dialect::template::{Ctx, Value};
use nuclear_syntax::ast::{File, Item};
use nuclear_syntax::token::{StrPart, Token, TokenKind};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// `nuclear.toml`.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// Name of the generated app or package (`package:<name>/...` in Dart).
    pub name: String,
    pub dialect: String,
    /// The Dragonfly platform of the project (`dragonfly(platform = ...)` in `config.nuc`, ADR-0093); `None`
    /// for a project of a language dialect (`nuclear.toml`).
    pub platform: Option<String>,
    /// `app(version = ..., description = ...)` of `config.nuc` (`[project] version` in `nuclear.toml`): what the
    /// generated manifest (pyproject.toml, package.json, pubspec.yaml) says of the app.
    pub version: String,
    pub description: String,
    /// `app(title = ...)`: the app's name as people read it (`NuclearLang`; the window or tab title). Empty: the name.
    pub title: String,
    /// Output folder, relative to the project. Empty when the manifest is `config.nuc`: the dialect's
    /// `[project] output` decides (`output_dir`).
    pub out: String,
    /// Packages of the target's package manager the app needs (name → version).
    pub dependencies: BTreeMap<String, String>,
}

impl Manifest {
    /// The manifest of a project without `nuclear.toml`: its name is the folder's (in snake_case) and
    /// its output `build` (a framework CLI knows the dialect: Dragonfly projects have only `.nuc` files).
    pub fn infer(root: &Path, dialect: &str, out: &str) -> Manifest {
        let folder = root
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "app".into());
        let mut name = String::new();
        for (i, c) in folder.chars().enumerate() {
            if c.is_ascii_uppercase() {
                if i > 0 && !name.ends_with('_') {
                    name.push('_');
                }
                name.push(c.to_ascii_lowercase());
            } else if c.is_ascii_alphanumeric() {
                name.push(c);
            } else if !name.ends_with('_') {
                name.push('_');
            }
        }
        let name = name.trim_matches('_').to_string();
        let name = if name.starts_with(|c: char| c.is_ascii_digit()) || name.is_empty() {
            format!("app_{name}")
        } else {
            name
        };
        Manifest {
            name,
            dialect: dialect.into(),
            platform: None,
            version: "0.1.0".into(),
            description: String::new(),
            title: String::new(),
            out: out.into(),
            dependencies: BTreeMap::new(),
        }
    }

    /// The manifest of a Dragonfly project: `config.nuc` with `dragonfly(platform = ..., dialect = ...)` and,
    /// optionally, `app(name = ...)` (the folder's name otherwise). `out` stays empty: see `output_dir`.
    pub fn read_dragonfly(root: &Path) -> Result<Manifest, String> {
        let path = root.join(CONFIG_FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let sections = [DRAGONFLY_SECTION.to_string(), APP_SECTION.to_string(), DEPENDENCIES_SECTION.to_string()];
        let (config, _) = read_config(&text, &sections);
        let shown = path.display();
        let Some(section) = config.get(DRAGONFLY_SECTION) else {
            return Err(format!(
                "{shown}: missing dragonfly(platform = \"mobile\", dialect = \"flutter\") (the platform and the dialect of the project)"
            ));
        };
        let get = |k: &str| section.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
        let platform = get("platform").ok_or(format!("{shown}: dragonfly(...) needs platform"))?;
        let dialect = get("dialect").ok_or(format!("{shown}: dragonfly(...) needs dialect"))?;
        let mut m = Manifest::infer(root, &dialect, "");
        m.platform = Some(platform);
        let app = |k: &str| config.get(APP_SECTION).and_then(|a| a.iter().find(|(n, _)| n == k)).map(|(_, v)| v.clone());
        if let Some(name) = app("name") {
            if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                return Err(format!("{shown}: app(name = \"{name}\") must be snake_case (letters, digits, _)"));
            }
            m.name = name;
        }
        if let Some(version) = app("version") {
            m.version = version;
        }
        if let Some(description) = app("description") {
            m.description = description;
        }
        if let Some(title) = app("title") {
            m.title = title;
        }
        m.dependencies = config.get(DEPENDENCIES_SECTION).into_iter().flatten().cloned().collect();
        Ok(m)
    }

    /// The project that contains `path`: the closest folder with `nuclear.toml`, or with the `[project] marker`
    /// of one of `dialects` (`main.nuc` → flutter). When several dialects share a marker, the most specific
    /// one (longest `extends` chain) wins. The manifest is an error when `nuclear.toml` cannot be read.
    pub fn find(path: &Path, dialects: &[Dialect]) -> Option<(PathBuf, Result<Manifest, String>)> {
        let start = if path.is_dir() { path } else { path.parent()? };
        for dir in start.ancestors() {
            if dir.join("nuclear.toml").is_file() {
                return Some((dir.to_path_buf(), Manifest::read(dir)));
            }
            // a Dragonfly project names its platform and dialect in config.nuc (ADR-0093); one without the
            // section falls through to the marker, and check_config asks for the line
            if let Some(Ok(mut m)) = dir.join(CONFIG_FILE).is_file().then(|| Manifest::read_dragonfly(dir)) {
                if let Some(d) = dialects.iter().find(|d| d.name == m.dialect) {
                    m.out = output_dir(&m, d);
                }
                return Some((dir.to_path_buf(), Ok(m)));
            }
            let found = dialects
                .iter()
                .filter(|d| !d.project.marker.is_empty() && dir.join(&d.project.marker).is_file())
                .max_by_key(|d| d.chain.len());
            if let Some(d) = found {
                let out = if d.project.output.is_empty() { "build" } else { &d.project.output };
                return Some((dir.to_path_buf(), Ok(Manifest::infer(dir, &d.name, out))));
            }
        }
        None
    }

    /// `nuclear.toml`, or `config.nuc` with `dragonfly(...)` (`read_dragonfly`).
    pub fn read(root: &Path) -> Result<Manifest, String> {
        let path = root.join("nuclear.toml");
        if !path.is_file() && root.join(CONFIG_FILE).is_file() {
            return Manifest::read_dragonfly(root);
        }
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let t: toml::Table =
            text.parse().map_err(|e: toml::de::Error| format!("{}: {}", path.display(), e.message()))?;
        let p = t.get("project").and_then(|v| v.as_table()).ok_or(format!("{}: missing [project]", path.display()))?;
        let get = |k: &str| p.get(k).and_then(|v| v.as_str()).map(str::to_string);
        let name = get("name").ok_or(format!("{}: [project] needs name", path.display()))?;
        if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(format!("{}: project name '{name}' must be snake_case (letters, digits, _)", path.display()));
        }
        let dependencies = t
            .get("dependencies")
            .and_then(|v| v.as_table())
            .map(|d| d.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("any").to_string())).collect())
            .unwrap_or_default();
        Ok(Manifest {
            name,
            dialect: get("dialect").ok_or(format!("{}: [project] needs dialect", path.display()))?,
            platform: None,
            version: get("version").unwrap_or_else(|| "0.1.0".into()),
            description: get("description").unwrap_or_default(),
            title: get("title").unwrap_or_default(),
            out: get("out").unwrap_or_else(|| "build".into()),
            dependencies,
        })
    }
}

/// One transpiled file.
pub struct BuiltFile {
    /// Source, relative to the project root (`components/users/data/user_repository.nuc`).
    pub source_path: PathBuf,
    pub source: String,
    /// Output, relative to the output folder (`lib/components/users/data/user_repository.dart`).
    pub out_path: String,
    pub code: String,
    pub diagnostics: Vec<Diagnostic>,
    /// Line map for debuggers, as `Output::lines`.
    pub lines: Vec<(u32, u32)>,
}

pub struct ProjectOutput {
    pub files: Vec<BuiltFile>,
    /// Files that do not come from a `.nuc`: the Result runtime, the dialect's `[project.files]` and the
    /// text files of its `[project] assets` folders.
    pub generated: Vec<(String, String)>,
    /// The other files of the `[project] assets` folders (images, fonts): output path → source file, to
    /// copy as they are.
    pub copies: Vec<(String, PathBuf)>,
}

impl ProjectOutput {
    pub fn has_errors(&self) -> bool {
        self.files.iter().any(|f| f.diagnostics.iter().any(Diagnostic::is_error))
    }
}

/// The definitions packages a project adds to its dialect's (`hive@2.2.3`, one per line in
/// `build/defs.lock`): the framework CLI resolves them from the manifest; every tool loads them with
/// `Loader::add_defs`.
pub fn defs_lock(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(DEFS_LOCK))
        .map(|t| t.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

pub const DEFS_LOCK: &str = "build/defs.lock";

/// The folder where a project keeps its own type definitions (`defs/<language>/<name>/<version>/`).
pub const PROJECT_DEFS: &str = "defs";

/// The type definitions packages a project adds: `build/defs.lock` and every package kept in
/// `<project>/defs/<language>/` for the dialect's language (spec/definitions.md §1).
pub fn project_defs(root: &Path, dialect: &Dialect) -> Vec<String> {
    let mut refs = defs_lock(root);
    let language = dialect.chain.last().cloned().unwrap_or_default();
    let Ok(names) = std::fs::read_dir(root.join(PROJECT_DEFS).join(&language)) else { return refs };
    let mut own: Vec<String> = names
        .flatten()
        .flat_map(|n| {
            let name = n.file_name().to_string_lossy().into_owned();
            std::fs::read_dir(n.path())
                .into_iter()
                .flatten()
                .flatten()
                .filter(|v| v.path().join("package.toml").is_file())
                .map(move |v| format!("{name}@{}", v.file_name().to_string_lossy()))
        })
        .filter(|r| !refs.contains(r))
        .collect();
    own.sort();
    refs.extend(own);
    refs
}

/// The dialect plus the project's own type definitions (`project_defs`, searched in `<project>/defs` and the
/// cache); `None` when the project adds none. Each package that cannot be loaded is a warning.
pub fn with_project_defs(root: &Path, dialect: &Dialect) -> (Option<Dialect>, Vec<Diagnostic>) {
    let refs = project_defs(root, dialect);
    if refs.is_empty() {
        return (None, Vec::new());
    }
    let mut loader = Loader::new(Vec::new());
    loader.defs_paths.push(root.join(PROJECT_DEFS));
    let mut own = dialect.clone();
    let report = loader.add_defs(&mut own, &refs);
    let missing = report.diags.iter().map(|(_, d)| Diagnostic::warning(d.code, Span::default(), d.message.clone())).collect();
    (Some(own), missing)
}

/// The project's manifest written in Nuclear (`[project.manifest]`): each section (a top-level call) → its
/// named arguments as text. `dependencies(hive = "^2.2.3")` → `dependencies: [("hive", "^2.2.3")]`.
pub type Config = BTreeMap<String, Vec<(String, String)>>;

/// Reads a manifest: only top-level calls of `sections`, each with named arguments whose values are a text,
/// a number or a boolean. Anything else is N0303.
pub fn read_config(source: &str, sections: &[String]) -> (Config, Vec<Diagnostic>) {
    let (spanned, diags) = read_config_spans(source, sections);
    (without_spans(spanned), diags)
}

fn without_spans(c: SpannedConfig) -> Config {
    c.into_iter().map(|(k, v)| (k, v.into_iter().map(|(n, val, _)| (n, val)).collect())).collect()
}

/// A package the dialect or its framework already uses: its definitions are in the dialect's `defs` (dio,
/// go_router...) or it is in `[project.manifest] reserved` (fpdart, build_runner...). A project cannot add it.
pub fn reserved_package(dialect: &Dialect, name: &str) -> bool {
    dialect.typedefs.package(name).is_some_and(|p| !p.from_project)
        || dialect.project.manifest.as_ref().is_some_and(|m| m.reserved.iter().any(|r| r == name))
}

/// The section of `config.nuc` that enables plugins: `plugins(genkit = true, kafka = true)`.
pub const PLUGINS_SECTION: &str = "plugins";
/// The manifest of a Dragonfly project and its sections: `dragonfly(platform = ..., dialect = ...)` and
/// `app(name = ..., description = ..., version = ...)` (ADR-0093).
pub const CONFIG_FILE: &str = "config.nuc";
pub const DRAGONFLY_SECTION: &str = "dragonfly";
pub const APP_SECTION: &str = "app";
/// `dependencies(name = "version")`: packages of the target's package manager the project adds. A version that
/// starts with `.` or `/` is a folder on disk (resolved from the project root).
pub const DEPENDENCIES_SECTION: &str = "dependencies";

/// The output folder of a project: the manifest's `out`, else the dialect's `[project] output`, else `build`.
pub fn output_dir(manifest: &Manifest, dialect: &Dialect) -> String {
    if !manifest.out.is_empty() {
        manifest.out.clone()
    } else if !dialect.project.output.is_empty() {
        dialect.project.output.clone()
    } else {
        "build".into()
    }
}

/// The project card, written by every tool that builds the project (`nuclear build`, `dragonfly build`, the
/// editors): what the project is, for a client or a script that must not read Nuclear or load the dialect
/// (spec/projects.md §1.3).
pub const METADATA_FILE: &str = ".nuclear/project.json";

/// The content of `METADATA_FILE`: name, platform, dialect, language, output folder, enabled plugins and the
/// kinds the project can use with their source (`language:dart`, `platform:backend`, `package:jobs 1.2.0`).
pub fn metadata(root: &Path, manifest: &Manifest, dialect: &Dialect) -> serde_json::Value {
    let language = dialect.chain.last().cloned().unwrap_or_default();
    let kinds: serde_json::Map<String, serde_json::Value> = dialect
        .kind_sources
        .iter()
        .map(|(k, s)| {
            let source = match s {
                KindSource::Language => format!("language:{language}"),
                KindSource::Platform(p) => format!("platform:{p}"),
                KindSource::Package { name, version, .. } => format!("package:{name} {version}").trim_end().to_string(),
            };
            (k.clone(), source.into())
        })
        .collect();
    serde_json::json!({
        "name": manifest.name,
        "platform": manifest.platform,
        "dialect": dialect.name,
        "language": language,
        "output": output_dir(manifest, dialect),
        "plugins": enabled_plugins(root, dialect),
        "kinds": kinds,
        // what an editor needs to run the app without knowing the dialect (`[toolchain]` for this system): where it
        // answers, how to serve it on a port it picks, what exists once installed
        "toolchain": {
            "url": dialect.tool("url"),
            "serve": dialect.tool("serve"),
            "installed": dialect.tool("installed"),
        },
    })
}

/// Writes `METADATA_FILE` under `root` when its content changed.
pub fn write_metadata(root: &Path, manifest: &Manifest, dialect: &Dialect) -> std::io::Result<()> {
    // the contract of the project (and its OpenAPI document) go next to the card (ADR-0098)
    crate::contract::write(root, manifest, dialect)?;
    // what an agent reads before it writes: every declaration with its description (ADR-0102)
    crate::map::write(root, manifest, dialect)?;
    let text = serde_json::to_string_pretty(&metadata(root, manifest, dialect))? + "\n";
    let path = root.join(METADATA_FILE);
    if std::fs::read_to_string(&path).ok().as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    std::fs::create_dir_all(path.parent().expect("file in a folder"))?;
    std::fs::write(path, text)
}

/// The plugins the project enables in its manifest, with the ones they require (catalog order). A dialect without
/// plugins, or a project without the manifest, enables none.
pub fn enabled_plugins(root: &Path, dialect: &Dialect) -> Vec<String> {
    let Some(m) = &dialect.project.manifest else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(root.join(&m.file)) else { return Vec::new() };
    let (config, _) = read_config(&text, &m.sections);
    let on: Vec<String> =
        config.get(PLUGINS_SECTION).into_iter().flatten().filter(|(_, v)| v == "true").map(|(n, _)| n.clone()).collect();
    dialect.project.with_requires(&on).into_iter().map(|p| p.name.clone()).collect()
}

/// `read_config` plus the rules of the dialect: a package section cannot list a reserved package, and
/// `plugins(...)` names only plugins of the dialect, with `true` or `false` (N0303).
pub fn check_config(source: &str, dialect: &Dialect) -> (Config, Vec<Diagnostic>) {
    let Some(m) = &dialect.project.manifest else { return (Config::new(), Vec::new()) };
    let (spanned, mut diags) = read_config_spans(source, &m.sections);
    // dragonfly(platform = ..., dialect = ...): required by a Dragonfly dialect, and it must be this one
    if let Some(platform) = &dialect.platform {
        match spanned.get(DRAGONFLY_SECTION) {
            None => diags.push(Diagnostic::error(
                codes::PROJECT,
                Span::default(),
                format!("missing dragonfly(platform = \"{platform}\", dialect = \"{}\"): the first line of the manifest", dialect.name),
            )),
            Some(args) => {
                for (name, value, span) in args {
                    let expected = match name.as_str() {
                        "platform" => platform.as_str(),
                        "dialect" => dialect.name.as_str(),
                        _ => {
                            let msg = format!("dragonfly(...) takes only platform and dialect, not {name}");
                            diags.push(Diagnostic::error(codes::PROJECT, *span, msg));
                            continue;
                        }
                    };
                    if value != expected {
                        let msg = format!("this project is built with the {} dialect of the {platform} platform: {name} = \"{expected}\"", dialect.name);
                        diags.push(Diagnostic::error(codes::PROJECT, *span, msg));
                    }
                }
                for key in ["platform", "dialect"] {
                    if !args.iter().any(|(n, _, _)| n == key) {
                        diags.push(Diagnostic::error(codes::PROJECT, Span::default(), format!("dragonfly(...) needs {key}")));
                    }
                }
            }
        }
    }
    for (name, value, span) in spanned.get(PLUGINS_SECTION).into_iter().flatten() {
        // a kind package of the platform without an implementation for this dialect (ADR-0095 §3)
        if value == "true"
            && let Some(dialects) = dialect.package_gaps.get(name)
        {
            let msg = format!(
                "the kind package '{name}' has no implementation for the {} dialect (it has: {}); build with one of them or add {name}/{}/ to the package",
                dialect.name,
                dialects.join(", "),
                dialect.name
            );
            diags.push(Diagnostic::error(codes::KIND_PACKAGE, *span, msg));
            continue;
        }
        if dialect.project.plugin(name).is_none() {
            let names: Vec<&str> = dialect.project.plugins.iter().map(|p| p.name.as_str()).collect();
            let msg = format!("'{name}' is not a plugin of the {} dialect (plugins: {})", dialect.name, names.join(", "));
            diags.push(Diagnostic::error(codes::PROJECT, *span, msg));
        } else if value != "true" && value != "false" {
            diags.push(Diagnostic::error(codes::PROJECT, *span, format!("plugins({name} = true) or false")));
        }
    }
    for section in &m.packages {
        for (name, _, span) in spanned.get(section).into_iter().flatten() {
            if reserved_package(dialect, name) {
                let msg = format!(
                    "'{name}' is already part of the framework (the {} dialect uses it): it cannot be added; \
                     only third-party packages the framework does not use",
                    dialect.name
                );
                diags.push(Diagnostic::error(codes::PROJECT, *span, msg));
            }
        }
    }
    (without_spans(spanned), diags)
}

type SpannedConfig = BTreeMap<String, Vec<(String, String, Span)>>;

fn read_config_spans(source: &str, sections: &[String]) -> (SpannedConfig, Vec<Diagnostic>) {
    use nuclear_syntax::ast::{ExprKind, Stmt, StrPiece};
    let p = nuclear_syntax::parse(source);
    let mut diags = p.diagnostics;
    let mut config = SpannedConfig::new();
    let expected = sections.join(", ");
    for item in &p.file.items {
        let call = match item {
            Item::Stmt(Stmt::Expr(e)) => match &e.kind {
                ExprKind::Call { callee, args, .. } => match &callee.kind {
                    ExprKind::Name(n) if sections.contains(n) => Some((n.clone(), args, e.span)),
                    ExprKind::Name(n) => {
                        let msg = format!("'{n}' is not a section of the manifest (expected {expected})");
                        diags.push(Diagnostic::error(codes::PROJECT, callee.span, msg));
                        None
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
        let Some((section, args, span)) = call else {
            if !matches!(item, Item::Stmt(Stmt::Expr(_))) {
                let msg = format!("the manifest only has sections: {expected}");
                diags.push(Diagnostic::error(codes::PROJECT, item_span(item), msg));
            }
            continue;
        };
        if config.contains_key(&section) {
            diags.push(Diagnostic::error(codes::PROJECT, span, format!("'{section}' is written twice")));
        }
        let entries = config.entry(section).or_default();
        for a in args {
            let value = match &a.value.kind {
                ExprKind::Str(pieces) if pieces.iter().all(|x| matches!(x, StrPiece::Text(_))) => Some(
                    pieces.iter().map(|x| if let StrPiece::Text(t) = x { t.as_str() } else { "" }).collect::<String>(),
                ),
                ExprKind::Bool(b) => Some(b.to_string()),
                ExprKind::Number(_) => source.get(a.value.span.start..a.value.span.end).map(str::to_string),
                _ => None,
            };
            match (&a.name, value) {
                (Some(name), Some(v)) => entries.push((name.name.clone(), v, name.span)),
                (None, _) => diags.push(Diagnostic::error(codes::PROJECT, a.span, "write it as name = value")),
                (_, None) => {
                    let msg = "a value of the manifest is a text, a number or a boolean";
                    diags.push(Diagnostic::error(codes::PROJECT, a.value.span, msg));
                }
            }
        }
    }
    (config, diags)
}

fn item_span(item: &Item) -> Span {
    match item {
        Item::Decl(d) => d.name.span,
        Item::Use(u) => u.span,
        Item::Stmt(_) => Span::default(),
    }
}

/// Every `.nuc` under `root`, sorted, skipping hidden folders, `skip` (the output folder) and a folder that is a
/// repository of its own (it has a `.git`: another project, like a knowledge base kept inside this one).
pub fn sources(root: &Path, skip: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            let hidden = p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if hidden || p == skip || (p.is_dir() && p.join(".git").exists()) {
                continue;
            }
            if p.is_dir() {
                pending.push(p);
            } else if p.extension().is_some_and(|x| x == "nuc") {
                out.push(p.strip_prefix(root).unwrap_or(&p).to_path_buf());
            }
        }
    }
    out.sort();
    out
}

/// `components/users/data/user_repository.nuc` → `components/users/data/user_repository`.
fn stem_path(rel: &Path) -> String {
    rel.with_extension("").to_string_lossy().replace('\\', "/")
}

/// `[project] kinds`: the kind of the declarations without a kind word in this file's folder. A pattern
/// matches its folder and every folder inside it; `*` matches one folder name.
pub fn default_kind(d: &Dialect, path: &str) -> Option<String> {
    let folders: Vec<&str> = path.split('/').collect();
    let folders = &folders[..folders.len().saturating_sub(1)];
    d.project
        .kinds
        .iter()
        .filter(|(pattern, _)| {
            let parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
            parts.len() <= folders.len() && parts.iter().zip(folders).all(|(p, f)| *p == "*" || p == f)
        })
        .max_by_key(|(pattern, _)| pattern.len())
        .map(|(_, k)| k.clone())
}

/// Identifiers written in the source (also inside `${...}`), with their positions.
fn identifiers(tokens: &[Token], out: &mut Vec<(String, Span)>) {
    for t in tokens {
        match &t.kind {
            TokenKind::Ident(n) => out.push((n.clone(), t.span)),
            TokenKind::Str(s) => {
                for p in &s.parts {
                    if let StrPart::Interp { tokens, .. } = p {
                        identifiers(tokens, out);
                    }
                }
            }
            _ => {}
        }
    }
}

struct Parsed {
    rel: PathBuf,
    path: String,
    source: String,
    file: Option<File>,
    diagnostics: Vec<Diagnostic>,
    default_kind: Option<String>,
}

/// Transpiles every `.nuc` of the project in `root` with `dialect`.
pub fn build(root: &Path, manifest: &Manifest, dialect: &Dialect) -> ProjectOutput {
    let out_dir = root.join(output_dir(manifest, dialect));
    // the contracts the manifest declares become `contracts/<name>/*.nuc` before the sources are read
    let contract_diags = crate::contract::sync(root, dialect);
    let mut parsed: Vec<Parsed> = Vec::new();
    // output name → source name of every file: a message may name a member declared in another file
    let mut names = HashMap::new();
    let mut config_file = None;
    for rel in sources(root, &out_dir) {
        let source = std::fs::read_to_string(root.join(&rel)).unwrap_or_default();
        // the manifest (config.nuc) is checked, not transpiled: it has no output (`out_path` is empty)
        if dialect.project.manifest.as_ref().is_some_and(|m| rel == Path::new(&m.file)) {
            let (_, mut diagnostics) = check_config(&source, dialect);
            diagnostics.extend(contract_diags.iter().cloned());
            config_file = Some(BuiltFile {
                source_path: rel,
                source,
                out_path: String::new(),
                code: String::new(),
                diagnostics,
                lines: Vec::new(),
            });
            continue;
        }
        let mut p = nuclear_syntax::parse(&source);
        names.extend(crate::naming::apply(&mut p.file, dialect));
        let ok = !p.diagnostics.iter().any(Diagnostic::is_error);
        let path = stem_path(&rel);
        parsed.push(Parsed {
            default_kind: default_kind(dialect, &path),
            rel,
            path,
            source,
            file: ok.then_some(p.file),
            diagnostics: p.diagnostics,
        });
    }

    // pass 1: every declaration of every file
    let mut ctx = ProjectCtx {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        title: manifest.title.clone(),
        dependencies: manifest
            .dependencies
            .iter()
            .map(|(name, version)| {
                let path = (version.starts_with('.') || version.starts_with('/')).then(|| {
                    let p = root.join(version);
                    p.canonicalize().unwrap_or(p).to_string_lossy().replace('\\', "/")
                });
                (name.clone(), version.clone(), path)
            })
            .collect(),
        isolated: dialect.project.outputs.iter().map(|(from, _)| from.clone()).collect(),
        sources: dialect.project.sources.clone(),
        outputs: dialect.project.outputs.clone(),
        plugins: (!dialect.project.plugins.is_empty()).then(|| enabled_plugins(root, dialect).into_iter().collect()),
        ..Default::default()
    };
    let mut kinds: HashMap<String, Option<String>> = HashMap::new();
    // declarations made public by a decorator (`public = ["@screen"]`)
    let mut public_decls: HashSet<String> = HashSet::new();
    let mut modules = HashMap::new();
    let mut declared_in: HashMap<String, String> = HashMap::new();
    for p in &mut parsed {
        let Some(file) = &p.file else { continue };
        let empty = ProjectCtx::default();
        let mut e =
            Emitter::new(dialect, Vec::new()).in_project(&empty, &p.path, HashSet::new(), p.default_kind.clone());
        for item in &file.items {
            if let Item::Decl(x) = item
                && x.decorators.iter().any(|d| dialect.architecture.public.contains(&format!("@{}", d.name)))
            {
                public_decls.insert(x.name.name.clone());
            }
        }
        for (name, out_name, info) in e.declarations(file) {
            // the entry point is never imported by other files (a test file has its own main)
            if name == "main" {
                continue;
            }
            if let Some(other) = declared_in.get(&name) {
                let span = file_decl_span(file, &name);
                p.diagnostics.push(Diagnostic::error(
                    codes::PROJECT,
                    span,
                    format!("'{name}' is also declared in {other}.nuc; each name is declared once in a project"),
                ));
                continue;
            }
            declared_in.insert(name.clone(), p.path.clone());
            kinds.insert(name.clone(), if let DeclKind::Kind(k) = &info.kind { Some(k.clone()) } else { None });
            ctx.decls.insert(name, ExtDecl { out_name, path: p.path.clone(), info });
        }
        modules.extend(alias::modules(file));
    }
    let sources_dir = dialect.project.sources.clone();
    for f in &dialect.project.files {
        let rel = f.path.strip_prefix(&format!("{sources_dir}/")).unwrap_or(&f.path);
        let path = Path::new(rel).with_extension("").to_string_lossy().replace('\\', "/");
        for n in &f.names {
            ctx.generated.insert(n.clone(), path.clone());
        }
    }

    // pass 2: emit each file
    let ext = &dialect.file_extension;
    // `[project.outputs]`: a folder (test/) that goes to its own output folder instead of `sources`
    let out_path = |path: &str| {
        for (from, to) in &dialect.project.outputs {
            if let Some(rest) = path.strip_prefix(&format!("{from}/")) {
                return format!("{to}/{rest}.{ext}");
            }
        }
        if sources_dir.is_empty() { format!("{path}.{ext}") } else { format!("{sources_dir}/{path}.{ext}") }
    };
    let mut files = Vec::new();
    let mut summaries: Vec<(String, DeclSummary)> = Vec::new();
    let mut uses_result = false;
    for p in parsed {
        let mut diagnostics = p.diagnostics;
        let mut code = String::new();
        let mut lines = Vec::new();
        if let Some(mut file) = p.file {
            let mut idents = Vec::new();
            identifiers(&nuclear_syntax::lex(&p.source).tokens, &mut idents);
            diagnostics.extend(architecture(dialect, &ctx, &kinds, &public_decls, &file, &p.path, &idents));
            let others: HashMap<_, _> = modules.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            diagnostics.extend(alias::expand(&mut file, &others, &|d| crate::alias_via(dialect, d)));
            let mut e = Emitter::new(dialect, file.comments.clone()).with_source(&p.source).in_project(
                &ctx,
                &p.path,
                HashSet::new(),
                p.default_kind,
            );
            code = e.file(&file);
            lines = crate::line_map(&p.source, &e.marks);
            uses_result |= e.uses_result();
            diagnostics.append(&mut e.diags);
            summaries.extend(e.summaries.into_iter().map(|s| (p.path.clone(), s)));
        }
        crate::naming::restore(&mut diagnostics, &names);
        diagnostics.sort_by_key(|d| d.span.start);
        files.push(BuiltFile {
            out_path: out_path(&p.path),
            source_path: p.rel,
            source: p.source,
            code,
            diagnostics,
            lines,
        });
    }

    let mut generated = Vec::new();
    if uses_result && let Some(t) = dialect.syntax("result_runtime") {
        let prelude = dialect.syntax("file_prelude").map(|p| format!("{}\n\n", p.render(&Ctx::new().into_value()).trim_end()));
        let runtime = t.render(&Ctx::new().into_value());
        generated.push((out_path(RESULT_PATH), format!("{}{}\n", prelude.unwrap_or_default(), runtime.trim_end())));
    }
    // uses.<Name>: the generated code of the project writes that name (a template asks only for what is used)
    let uses: BTreeMap<String, Value> =
        files.iter().flat_map(|f| crate::emit::code_words(&f.code)).map(|w| (w, Value::Bool(true))).collect();
    // `asset.<path>`: the files of the `[project] assets` folders, the path with `/` and `.` as `_`
    // (`asset.public_styles_css`), so a project file links what the project has (index.html its stylesheet)
    let asset_flags: BTreeMap<String, Value> = dialect
        .project
        .assets
        .iter()
        .flat_map(|dir| {
            let (text, other) = assets(root, dir);
            text.into_iter().map(|(p, _)| p).chain(other.into_iter().map(|(p, _)| p)).collect::<Vec<_>>()
        })
        .map(|p| (p.replace(['/', '.', '-'], "_"), Value::Bool(true)))
        .collect();
    let with_assets = |v: Value| match v {
        Value::Map(mut m) => {
            m.insert("asset".into(), Value::Map(asset_flags.clone()));
            Value::Map(m)
        }
        other => other,
    };
    for f in &dialect.project.files {
        // `files` = the files whose names the generated code writes: render with all, then only with those
        let all = f.template.render(&with_assets(project_context(dialect, &ctx, &summaries, None, &uses, &f.path)));
        let words = crate::emit::code_words(&all);
        let text =
            f.template.render(&with_assets(project_context(dialect, &ctx, &summaries, Some(&words), &uses, &f.path)));
        // a template that renders nothing (`{{#decorated.splash}}` without a @splash) writes no file
        if !text.trim().is_empty() {
            generated.push((f.path.clone(), format!("{}\n", text.trim_end())));
        }
    }
    let mut copies = Vec::new();
    for dir in &dialect.project.assets {
        let (text, other) = assets(root, dir);
        generated.extend(text);
        copies.extend(other);
    }
    files.extend(config_file);
    ProjectOutput { files, generated, copies }
}

/// The line map of a build, under its output folder: what the editors' debuggers read to put a breakpoint of a `.nuc`
/// on its generated line and to show the `.nuc` line of a paused frame (any dialect: node, chrome, python, dart).
pub const LINE_MAP_FILE: &str = ".nuclear/lines.json";

/// The line map of a build: for each generated file (relative to the output), its `.nuc` (relative to the project)
/// and the pairs (output line, `.nuc` line).
pub fn line_map(out: &ProjectOutput) -> String {
    let files: serde_json::Map<String, serde_json::Value> = out
        .files
        .iter()
        .filter(|f| !f.out_path.is_empty())
        .map(|f| {
            let source = f.source_path.to_string_lossy().replace('\\', "/");
            (f.out_path.clone(), serde_json::json!({ "source": source, "lines": f.lines }))
        })
        .collect();
    serde_json::json!({ "version": 1, "files": files }).to_string()
}

/// The files under `root/dir`, with their path relative to `root`, sorted: the text ones with their
/// content, the others (images) with their source path.
type Assets = (Vec<(String, String)>, Vec<(String, PathBuf)>);

fn assets(root: &Path, dir: &str) -> Assets {
    let mut out = Vec::new();
    let mut copies = Vec::new();
    let mut pending = vec![root.join(dir)];
    while let Some(d) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
                continue;
            }
            if p.is_dir() {
                pending.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
                match std::fs::read_to_string(&p) {
                    Ok(text) => out.push((rel, text)),
                    Err(_) => copies.push((rel, p)),
                }
            }
        }
    }
    out.sort();
    copies.sort();
    (out, copies)
}

fn file_decl_span(file: &File, name: &str) -> Span {
    file.items
        .iter()
        .find_map(|i| if let Item::Decl(d) = i { (d.name.name == name).then_some(d.name.span) } else { None })
        .unwrap_or_default()
}

/// Component of a file: the folder under `[architecture] scope`.
fn component_of(d: &Dialect, path: &str) -> String {
    let Some(scope) = &d.architecture.scope else { return String::new() };
    path.strip_prefix(&format!("{scope}/")).and_then(|r| r.split('/').next()).unwrap_or_default().to_string()
}

/// `[architecture]`: which kinds may use each kind, and what is private to its component.
fn architecture(
    d: &Dialect,
    ctx: &ProjectCtx,
    kinds: &HashMap<String, Option<String>>,
    public_decls: &HashSet<String>,
    file: &File,
    path: &str,
    idents: &[(String, Span)],
) -> Vec<Diagnostic> {
    let a = &d.architecture;
    if a.callers.is_empty() && a.scope.is_none() && a.names.is_empty() {
        return Vec::new();
    }
    let here = component_of(d, path);
    let decls: Vec<(&str, Span)> = file
        .items
        .iter()
        .filter_map(|i| if let Item::Decl(x) = i { Some((x.name.name.as_str(), x.span)) } else { None })
        .collect();
    let mut out = Vec::new();
    let mut reported = HashSet::new();
    for (name, span) in idents {
        // a name only some kinds can use (`BuildContext` only in views)
        if let Some(allowed) = a.names.get(name)
            && let Some(&(source, _)) = decls.iter().find(|(_, s)| s.start <= span.start && span.end <= s.end)
            && let Some(k) = kinds.get(source).cloned().flatten()
            && !allowed.contains(&k)
            && !a.exempt.contains(&k)
            && reported.insert((source.to_string(), name.clone()))
        {
            out.push(Diagnostic::error(
                codes::ARCH_CALLER,
                *span,
                format!("'{name}' can only be used in a {}; a {k} ('{source}') cannot use it", allowed.join(" or ")),
            ));
        }
        let Some(target) = ctx.decls.get(name) else { continue };
        let Some(&(source, _)) = decls.iter().find(|(_, s)| s.start <= span.start && span.end <= s.end) else {
            continue;
        };
        if source == name {
            continue;
        }
        let target_kind = kinds.get(name).cloned().flatten();
        let source_kind = kinds.get(source).cloned().flatten();
        if source_kind.as_ref().is_some_and(|k| a.exempt.contains(k)) {
            continue;
        }
        if !reported.insert((source.to_string(), name.clone())) {
            continue;
        }
        let there = component_of(d, &target.path);
        let public = target_kind.as_ref().is_some_and(|k| a.public.contains(k)) || public_decls.contains(name);
        if !there.is_empty() && there != here && !public {
            let what = target_kind.as_deref().unwrap_or("declaration");
            out.push(Diagnostic::error(
                codes::ARCH_PRIVATE,
                *span,
                format!(
                    "'{name}' ({what}) is private to the component '{there}'; use it through the {} of '{there}'",
                    a.public.first().map_or("public declarations", String::as_str)
                ),
            ));
            continue;
        }
        if let Some(k) = &target_kind
            && let Some(allowed) = a.callers.get(k)
            && !source_kind.as_ref().is_some_and(|s| allowed.contains(s))
        {
            let who = source_kind.as_deref().map_or_else(|| format!("'{source}'"), |s| format!("a {s} ('{source}')"));
            out.push(Diagnostic::error(
                codes::ARCH_CALLER,
                *span,
                format!("a {k} ('{name}') can only be used from {}; {who} cannot use it", allowed.join(" or ")),
            ));
        }
    }
    out
}

/// What `[project.files]` templates receive: `project`, `app` (`name`, `version`, `description` of the
/// manifest), `decls` (each with name, kind, component, path,
/// import, params, decor), `kinds.<kind>` (the declarations of each kind), `decorated.<decorator>` (the
/// declarations with that decorator), `files` (one import per file whose names the generated code writes) and
/// `has.<kind>` / `has.@<decorator>` (the project has one).
fn project_context(
    d: &Dialect,
    ctx: &ProjectCtx,
    summaries: &[(String, DeclSummary)],
    written: Option<&HashSet<String>>,
    uses: &BTreeMap<String, Value>,
    from_out: &str,
) -> Value {
    let import = |path: &str| {
        d.syntax("project_import")
            .map(|t| {
                let c = Ctx::new()
                    .set("project", &ctx.name)
                    .set("path", path.replace('/', &d.options.module_separator))
                    .set("relative", ctx.relative(from_out, path));
                t.render(&c.into_value())
            })
            .unwrap_or_default()
    };
    let mut decls = Vec::new();
    let mut by_kind: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut decorated: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut files: Vec<String> = Vec::new();
    for (path, s) in summaries {
        let v = Ctx::new()
            .set("name", &s.name)
            .set("kind", s.kind.clone().unwrap_or_default())
            .set("component", component_of(d, path))
            .set("path", path)
            .set("import", import(path))
            .value("params", s.params.clone())
            .value("fields", s.fields.clone())
            .value("field_decor", field_decor(&s.fields))
            .value("methods", s.methods.clone())
            .value("generics", s.generics.clone())
            .value("decor", s.decor.clone())
            .into_value();
        if let Some(k) = &s.kind {
            by_kind.entry(k.clone()).or_default().push(v.clone());
        }
        if let Value::Map(m) = &s.decor {
            for k in m.keys() {
                decorated.entry(k.clone()).or_default().push(v.clone());
            }
        }
        // the entry point is never imported
        if s.name != "main" && written.is_none_or(|w| w.contains(&s.name)) && !files.contains(path) {
            files.push(path.clone());
        }
        decls.push(v);
    }
    let files = files.iter().map(|p| Ctx::new().set("path", p).set("import", import(p)).into_value()).collect();
    // has.<kind> / has.@<decorator>: the project has at least one (for imports a template needs once)
    let mut has: BTreeMap<String, Value> = BTreeMap::new();
    for k in by_kind.keys() {
        has.insert(k.clone(), Value::Bool(true));
    }
    for k in decorated.keys() {
        has.insert(format!("@{k}"), Value::Bool(true));
    }
    // components: each component with its declarations by kind (a framework module per component, NestJS)
    let mut components: BTreeMap<String, BTreeMap<String, Vec<Value>>> = BTreeMap::new();
    for d in &decls {
        let Value::Map(m) = d else { continue };
        let (Some(c), Some(k)) = (m.get("component").map(Value::text), m.get("kind").map(Value::text)) else { continue };
        if !c.is_empty() && !k.is_empty() {
            components.entry(c).or_default().entry(k).or_default().push(d.clone());
        }
    }
    let components: Vec<Value> = components
        .into_iter()
        .map(|(name, kinds)| {
            let kinds = Value::Map(kinds.into_iter().map(|(k, v)| (k, Value::List(v))).collect());
            Ctx::new().set("name", name).value("kinds", kinds).into_value()
        })
        .collect();
    // dependencies(...) of the manifest: `name`, `version`, `path` (a folder on disk, absolute) for the target's manifest
    let dependencies = ctx
        .dependencies
        .iter()
        .map(|(name, version, path)| {
            let c = Ctx::new().set("name", name).set("version", version);
            match path {
                Some(p) => c.set("path", p),
                None => c,
            }
            .into_value()
        })
        .collect();
    let app = Ctx::new()
        .set("name", &ctx.name)
        .set("version", &ctx.version)
        .set("description", &ctx.description)
        .set("title", if ctx.title.is_empty() { &ctx.name } else { &ctx.title })
        .into_value();
    Ctx::new()
        .set("project", &ctx.name)
        .value("app", app)
        .value("components", Value::List(components))
        .value("decls", Value::List(decls))
        .value("kinds", Value::Map(by_kind.into_iter().map(|(k, v)| (k, Value::List(v))).collect()))
        .value("decorated", Value::Map(decorated.into_iter().map(|(k, v)| (k, Value::List(v))).collect()))
        .value("files", Value::List(files))
        .value("has", Value::Map(has))
        .value("uses", Value::Map(uses.clone()))
        .value("plugins", plugin_context(d, ctx))
        .value("dependencies", Value::List(dependencies))
        .into_value()
}

/// `plugins.<name>` (enabled), `plugins.packages` / `plugins.dev_packages` (`name`, `version`, `local`: the packages of
/// the enabled plugins, once each) for the templates of the target's manifest (pyproject.toml, package.json).
fn plugin_context(d: &Dialect, ctx: &ProjectCtx) -> Value {
    let enabled: Vec<String> = ctx.plugins.iter().flatten().cloned().collect();
    let mut m: BTreeMap<String, Value> = BTreeMap::new();
    let mut seen = HashSet::new();
    let mut packages = Vec::new();
    let mut dev = Vec::new();
    for p in d.project.plugins.iter().filter(|p| enabled.contains(&p.name)) {
        m.insert(p.name.clone(), Value::Bool(true));
        for (list, out) in [(&p.packages, &mut packages), (&p.dev_packages, &mut dev)] {
            for (name, version) in list {
                if seen.insert(name.clone()) {
                    out.push(
                        Ctx::new().set("name", name).set("version", version).flag("local", version == "local").into_value(),
                    );
                }
            }
        }
    }
    m.insert("packages".into(), Value::List(packages));
    m.insert("dev_packages".into(), Value::List(dev));
    Value::Map(m)
}

/// `field_decor.<decorator>` of a declaration in the project context: the first field with that decorator.
fn field_decor(fields: &Value) -> Value {
    let mut out = BTreeMap::new();
    if let Value::List(items) = fields {
        for f in items {
            let Value::Map(m) = f else { continue };
            let (Some(name), Some(Value::Map(decor))) = (m.get("name"), m.get("decor")) else { continue };
            for k in decor.keys() {
                out.entry(k.clone()).or_insert_with(|| name.clone());
            }
        }
    }
    Value::Map(out)
}
