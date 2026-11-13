//! The project map (`.nuclear/map.md`): every declaration of a project with its kind, its description (its doc
//! comment) and its members, grouped by component, in Nuclear's notation. What an agent reads before it writes — the
//! business of each component in a few lines instead of every file — and what no dialect changes (spec/projects.md).

use crate::contract::{Lines, type_text};
use crate::project::{self, Manifest};
use nuclear_dialect::Dialect;
use nuclear_syntax::ast::*;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// Where the map is written, next to the project card.
pub const MAP_FILE: &str = ".nuclear/map.md";

/// The map of the project in `root`.
/// One declaration of the project: its component (or `""` outside one), file, kind and name (`dragonfly gen inspect`).
#[derive(Debug, Clone, PartialEq)]
pub struct Declared {
    pub component: String,
    pub file: String,
    pub kind: String,
    pub name: String,
}

/// Every declaration of the project's `.nuc` files, with its kind (its header word, else its folder's).
pub fn inventory(root: &Path, manifest: &Manifest, dialect: &Dialect) -> Vec<Declared> {
    let out_dir = root.join(project::output_dir(manifest, dialect));
    let mut out = Vec::new();
    for rel in project::sources(root, &out_dir) {
        let rel_text = rel.to_string_lossy().replace('\\', "/");
        let Ok(source) = std::fs::read_to_string(root.join(&rel)) else { continue };
        let parsed = nuclear_syntax::parse(&source);
        let by_folder = project::default_kind(dialect, rel_text.trim_end_matches(".nuc"));
        let component = rel_text.strip_prefix("components/").and_then(|r| r.split_once('/')).map(|(c, _)| c.to_string()).unwrap_or_default();
        for item in &parsed.file.items {
            let Item::Decl(d) = item else { continue };
            let kind = d.words.iter().map(|w| w.name.clone()).find(|w| dialect.kinds.contains_key(w)).or(by_folder.clone()).unwrap_or_default();
            out.push(Declared { component: component.clone(), file: rel_text.clone(), kind, name: d.name.name.clone() });
        }
    }
    out
}

/// A declaration as its source writes it: its file, its header's parameters and its methods (`dragonfly gen usecase`
/// reads what it wires).
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub file: String,
    /// The header's parameters: (name, type).
    pub params: Vec<(String, String)>,
    pub methods: Vec<MethodShape>,
}

/// A method: its parameters as written (`id: int, input: Order`), their names, and its return type when it has one.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodShape {
    pub name: String,
    pub params: String,
    pub args: Vec<String>,
    pub ret: Option<String>,
    /// It is asynchronous: its body uses `~` (a call to it is written with `~`).
    pub awaits: bool,
}

/// The declaration `name` of the project's `.nuc` files.
pub fn shape(root: &Path, manifest: &Manifest, dialect: &Dialect, name: &str) -> Option<Shape> {
    let out_dir = root.join(project::output_dir(manifest, dialect));
    for rel in project::sources(root, &out_dir) {
        let Ok(source) = std::fs::read_to_string(root.join(&rel)) else { continue };
        for item in &nuclear_syntax::parse(&source).file.items {
            let Item::Decl(d) = item else { continue };
            if d.name.name != name {
                continue;
            }
            let list = |p: &Option<Params>| p.as_ref().map(|p| p.list.clone()).unwrap_or_default();
            let methods = match &d.body {
                Body::Members(ms) => ms
                    .iter()
                    .filter_map(|m| match m {
                        Member::Method(md) => Some(MethodShape {
                            name: md.name.name.clone(),
                            params: params(&list(&md.params)),
                            args: list(&md.params).iter().map(|p| p.name.name.clone()).collect(),
                            ret: md.ret.as_ref().map(|t| type_text(Some(t))),
                            awaits: md.is_async
                                || match &md.body {
                                    Body::Block(b) => crate::analysis::awaits_block(&b.stmts),
                                    Body::Expr(e) => crate::analysis::awaits_expr(e),
                                    _ => false,
                                },
                        }),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            return Some(Shape {
                file: rel.to_string_lossy().replace('\\', "/"),
                params: list(&d.params).iter().map(|p| (p.name.name.clone(), type_text(p.ty.as_ref()))).collect(),
                methods,
            });
        }
    }
    None
}

pub fn map(root: &Path, manifest: &Manifest, dialect: &Dialect) -> String {
    let out_dir = root.join(project::output_dir(manifest, dialect));
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for rel in project::sources(root, &out_dir) {
        let rel_text = rel.to_string_lossy().replace('\\', "/");
        if dialect.project.manifest.as_ref().is_some_and(|m| rel_text == m.file) {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(root.join(&rel)) else { continue };
        let parsed = nuclear_syntax::parse(&source);
        let lines = Lines::new(&source, &parsed.file.comments);
        let stem = rel_text.trim_end_matches(".nuc");
        let by_folder = project::default_kind(dialect, stem);
        let mut entries = Vec::new();
        for item in &parsed.file.items {
            let Item::Decl(d) = item else { continue };
            let kind = d.words.iter().map(|w| w.name.clone()).find(|w| dialect.kinds.contains_key(w)).or(by_folder.clone());
            entries.push(declaration(d, kind.as_deref(), &lines, &source));
        }
        if entries.is_empty() {
            continue;
        }
        let group = match rel_text.strip_prefix("components/").and_then(|r| r.split_once('/')) {
            Some((c, _)) => format!("Component `{c}`"),
            None => match rel_text.split_once('/') {
                Some((folder, _)) => format!("`{folder}/`"),
                None => "The app".to_string(),
            },
        };
        groups.entry(group).or_default().push(format!("- `{rel_text}`\n{}", entries.join("\n")));
    }
    let mut out = format!(
        "# Project map: {} ({})\n\nWritten by `nuclear check` and `nuclear build`: every declaration with its kind, its description \
         (its doc comment) and its members, in Nuclear. Read it before writing; a declaration without a description \
         gets one with a `//` line above it.\n",
        manifest.name,
        manifest.platform.as_deref().unwrap_or("no platform"),
    );
    // the components first, then the shared folders and the app's root files
    let (components, rest): (Vec<_>, Vec<_>) = groups.into_iter().partition(|(g, _)| g.starts_with("Component"));
    for (group, files) in components.into_iter().chain(rest) {
        let _ = write!(out, "\n## {group}\n\n{}\n", files.join("\n"));
    }
    out
}

/// `  - usecase `GetTask(tasks: TaskRepository)` — what it is` and its members, one per line.
fn declaration(d: &Decl, kind: Option<&str>, lines: &Lines, source: &str) -> String {
    let own = d.params.as_ref().map(|p| format!("({})", params(&p.list))).unwrap_or_default();
    let mut ctor = String::new();
    let mut members = Vec::new();
    if let Body::Members(ms) = &d.body {
        for m in ms {
            match m {
                Member::Field(f) => {
                    let opt = if f.optional { "?" } else { "" };
                    let deco = decorators(&f.decorators, source);
                    members.push((format!("{deco}{}{opt}: {}", f.name.name, type_text(f.ty.as_ref())), lines.doc(f.span.start)));
                }
                Member::Method(md) => members.push((method(md, source), lines.doc(md.span.start))),
                Member::Ctor(c) => ctor = format!("({})", params(&c.params.list)),
                Member::EnumItem(e) => members.push((e.name.name.clone(), None)),
            }
        }
    }
    let ret = match (&d.body, &d.ret) {
        (Body::Members(_), _) | (_, None) => String::new(),
        (_, Some(t)) => format!(" {}", type_text(Some(t))),
    };
    let what = lines.doc(d.span.start).map(|t| format!(" — {t}")).unwrap_or_else(|| " — (no description)".into());
    let kind = kind.map(|k| format!("{k} ")).unwrap_or_default();
    let mut text = format!("  - {kind}`{}{own}{ctor}{ret}`{what}", d.name.name);
    for (code, doc) in members {
        let _ = write!(text, "\n    - `{code}`{}", doc.map(|t| format!(" — {t}")).unwrap_or_default());
    }
    text
}

fn method(md: &Decl, source: &str) -> String {
    // an alias keeps its form: `list() = ListTasks.exec`
    if let Body::Expr(Expr { kind: ExprKind::Member { object, name, .. }, .. }) = &md.body
        && let ExprKind::Name(m) = &object.kind
        && md.ret.is_none()
        && md.params.as_ref().is_none_or(|p| p.list.is_empty())
    {
        return format!("{}() = {m}.{}", md.name.name, name.name);
    }
    let params = md.params.as_ref().map(|p| params(&p.list)).unwrap_or_default();
    let ret = md.ret.as_ref().map(|t| format!(" {}", type_text(Some(t)))).unwrap_or_default();
    format!("{}{}({params}){ret}", decorators(&md.decorators, source), md.name.name)
}

/// `@get("{id}") @status(201) ` as the source writes them.
fn decorators(list: &[Decorator], source: &str) -> String {
    list.iter().map(|x| format!("{} ", source.get(x.span.start..x.span.end).unwrap_or(&x.name).trim())).collect()
}

fn params(list: &[Param]) -> String {
    list.iter()
        .map(|p| match &p.ty {
            Some(t) => format!("{}{}: {}", p.name.name, if p.optional { "?" } else { "" }, type_text(Some(t))),
            None => p.name.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Writes the map next to the card when it changed.
pub fn write(root: &Path, manifest: &Manifest, dialect: &Dialect) -> std::io::Result<()> {
    let text = map(root, manifest, dialect);
    let path = root.join(MAP_FILE);
    if std::fs::read_to_string(&path).ok().as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    std::fs::create_dir_all(path.parent().expect("file in a folder"))?;
    std::fs::write(path, text)
}
