//! Type definitions packages: the public API of a target language's libraries, like TypeScript's `.d.ts`
//! files and `@types/*` packages. See spec/definitions.md.
//!
//! A package lives in a repository folder `<root>/<language>/<name>/<version>/` with three files:
//! - `package.toml`: name, version, language, description, `depends`, `libraries` (priority order) and
//!   `implicit` libraries (always visible, never imported);
//! - `index.json`: read on every build: symbol → library, re-exports, and each class's kind, direct
//!   supertypes and own instance member names;
//! - `api.json`: the full API (constructors, parameters, types, members, docs), read only when a tool asks
//!   for it (LSP, `nuclear defs show`).
//!
//! Types are written in NuclearLang notation (`string`, `int[]`, `<string, int>`, `(x: int) => void`).

use crate::model::ClassDef;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// One declaration of `api.json`: a class, mixin, enum, extension, function, variable or typedef.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Decl {
    pub name: String,
    /// Library that gives the name when imported; `None` for private supertypes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library: Option<String>,
    /// `class`, `mixin`, `enum`, `extension_type`, `extension`, `function`, `variable` or `typedef`.
    pub kind: String,
    /// Class modifiers of the target: `abstract`, `sealed`, `interface`, `base`, `final`, `mixin`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
    /// `T` or `T: Bound`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(default, rename = "with", skip_serializing_if = "Vec::is_empty")]
    pub mixins: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,
    /// Mixin constraints, or the extended type of an extension.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deprecated: bool,
    /// Enum values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constructors: Vec<Constructor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<Member>,
    /// Function parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Param>,
    /// Function return type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    /// Variable type, or the aliased type of a typedef.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub readonly: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Constructor {
    /// `""` for the unnamed constructor, `rich` for `Text.rich`.
    pub name: String,
    #[serde(default, rename = "const", skip_serializing_if = "is_false")]
    pub is_const: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub factory: bool,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deprecated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    /// `property`, `method` or `operator`.
    pub kind: String,
    #[serde(default, rename = "static", skip_serializing_if = "is_false")]
    pub is_static: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Param>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    /// Property type.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub readonly: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deprecated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    /// Passed by name (`{...}` in a NuclearLang signature).
    #[serde(default, skip_serializing_if = "is_false")]
    pub named: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    /// Default value, as written in the target language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// `index.json`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Index {
    pub format: u32,
    pub libraries: Vec<String>,
    #[serde(default)]
    pub reexports: BTreeMap<String, Vec<String>>,
    /// Symbol → index into `libraries`.
    #[serde(default)]
    pub symbols: HashMap<String, usize>,
    #[serde(default)]
    pub classes: HashMap<String, ClassDef>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    package: PackageMeta,
}

#[derive(Debug, Deserialize)]
struct PackageMeta {
    name: String,
    version: String,
    language: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    depends: Vec<String>,
    #[serde(default)]
    libraries: Vec<String>,
    #[serde(default)]
    implicit: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ApiFile {
    format: u32,
    declarations: Vec<Decl>,
}

/// The `format` of `index.json` and `api.json` this transpiler reads.
pub const FORMAT: u32 = 1;

/// One package of the repository. Its `api.json` is read the first time [`Package::api`] is called.
#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub language: String,
    pub description: String,
    /// `name@version` of the packages it needs.
    pub depends: Vec<String>,
    /// Its libraries in priority order: a symbol is imported from the first one that exports it.
    pub libraries: Vec<String>,
    /// Libraries that are always visible (`dart:core`): their names are never imported.
    pub implicit: Vec<String>,
    pub dir: PathBuf,
    /// Added by a project (`Loader::add_defs`), not by the dialect.
    pub from_project: bool,
    api: OnceLock<Result<Vec<Decl>, String>>,
}

impl Package {
    /// `flutter@3.47.7`
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Reads `package.toml` of a package folder.
    pub fn read(dir: &Path) -> Result<Package, String> {
        let path = dir.join("package.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let m: Manifest = toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e.message()))?;
        let p = m.package;
        Ok(Package {
            name: p.name,
            version: p.version,
            language: p.language,
            description: p.description,
            depends: p.depends,
            libraries: p.libraries,
            implicit: p.implicit,
            dir: dir.to_path_buf(),
            from_project: false,
            api: OnceLock::new(),
        })
    }

    /// Reads `index.json`.
    pub fn index(&self) -> Result<Index, String> {
        let path = self.dir.join("index.json");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let index: Index = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if index.format != FORMAT {
            return Err(format!("{}: format {} (this transpiler reads format {FORMAT})", path.display(), index.format));
        }
        Ok(index)
    }

    /// Every declaration of the package (`api.json`), read once.
    pub fn api(&self) -> Result<&[Decl], &str> {
        let loaded = self.api.get_or_init(|| {
            let path = self.dir.join("api.json");
            let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let api: ApiFile = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            if api.format != FORMAT {
                return Err(format!("{}: format {} (expected {FORMAT})", path.display(), api.format));
            }
            Ok(api.declarations)
        });
        match loaded {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }
}

/// Folders that hold packages as `<language>/<name>/<version>/`.
#[derive(Debug, Clone, Default)]
pub struct Repository {
    pub roots: Vec<PathBuf>,
}

impl Repository {
    /// The folder of a package, from the first root that has it.
    pub fn find(&self, language: &str, name: &str, version: &str) -> Option<PathBuf> {
        self.roots.iter().map(|r| r.join(language).join(name).join(version)).find(|d| d.join("package.toml").is_file())
    }

    /// Versions of a package found in every root (for error messages).
    pub fn versions(&self, language: &str, name: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .roots
            .iter()
            .filter_map(|r| std::fs::read_dir(r.join(language).join(name)).ok())
            .flat_map(|it| it.flatten())
            .filter(|e| e.path().join("package.toml").is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Every package of every root (the first root wins when two have the same one).
    pub fn list(&self) -> Vec<Package> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for root in &self.roots {
            for lang in sorted_dirs(root) {
                for name in sorted_dirs(&lang) {
                    for version in sorted_dirs(&name) {
                        if let Ok(p) = Package::read(&version)
                            && seen.insert((p.language.clone(), p.id()))
                        {
                            out.push(p);
                        }
                    }
                }
            }
        }
        out
    }
}

fn sorted_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|it| it.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    v.sort();
    v
}

/// Splits `flutter@3.47.7`.
pub fn parse_ref(r: &str) -> Option<(&str, &str)> {
    let (name, version) = r.split_once('@')?;
    (!name.is_empty() && !version.is_empty()).then_some((name, version))
}

/// The packages a dialect uses (its own and its parents'), in priority order.
#[derive(Debug, Clone, Default)]
pub struct TypeDefs {
    pub packages: Vec<Package>,
    /// Symbols already given to a package (the first package that exports a name keeps it).
    pub(crate) claimed: HashSet<String>,
    lookup: OnceLock<HashMap<String, (usize, usize)>>,
}

impl TypeDefs {
    pub fn package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.name == name)
    }

    pub(crate) fn push(&mut self, p: Package) {
        self.packages.push(p);
        self.lookup = OnceLock::new();
    }

    /// name → (package, declaration); reads the `api.json` of every package the first time.
    fn lookup(&self) -> &HashMap<String, (usize, usize)> {
        self.lookup.get_or_init(|| {
            let mut m = HashMap::new();
            for (pi, p) in self.packages.iter().enumerate() {
                if let Ok(decls) = p.api() {
                    for (di, d) in decls.iter().enumerate() {
                        m.entry(d.name.clone()).or_insert((pi, di));
                    }
                }
            }
            m
        })
    }

    /// The declaration of a name and the package that has it.
    pub fn get(&self, name: &str) -> Option<(&Package, &Decl)> {
        let (pi, di) = *self.lookup().get(name)?;
        let p = &self.packages[pi];
        Some((p, &p.api().ok()?[di]))
    }

    /// Every public declaration (for completion), in package order.
    pub fn declarations(&self) -> impl Iterator<Item = (&Package, &Decl)> {
        self.packages
            .iter()
            .flat_map(|p| p.api().unwrap_or(&[]).iter().map(move |d| (p, d)))
            .filter(|(_, d)| !d.name.starts_with('_'))
    }

    /// Members of a class, mixin, enum or extension, its own first and then the inherited ones (walking
    /// `extends`, `with`, `implements` and `on`), each with the declaration that has it. A name appears
    /// once: the closest declaration wins.
    pub fn members(&self, name: &str) -> Vec<(&Decl, &Member)> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::from([base_name(name).to_string()]);
        while let Some(n) = queue.pop_front() {
            if !visited.insert(n.clone()) {
                continue;
            }
            let Some((_, d)) = self.get(&n) else { continue };
            for m in &d.members {
                if seen.insert(m.name.clone()) {
                    out.push((d, m));
                }
            }
            for s in d.extends.iter().chain(&d.mixins).chain(&d.implements).chain(&d.on) {
                queue.push_back(base_name(s).to_string());
            }
        }
        out
    }

    /// Extensions whose `on` type is `name` (their members also apply to it).
    pub fn extensions_on(&self, name: &str) -> Vec<&Decl> {
        self.declarations()
            .filter(|(_, d)| d.kind == "extension" && d.on.iter().any(|o| base_name(o) == name))
            .map(|(_, d)| d)
            .collect()
    }
}

/// `List<int>?` → `List`, `int[]` → `int[]` (lists keep their notation).
pub fn base_name(ty: &str) -> &str {
    let t = ty.trim_end_matches('?');
    t.split('<').next().unwrap_or(t)
}

// ---------------------------------------------------------------------------------------------------------
// Rendering in NuclearLang notation (`nuclear defs show`, LSP hover)

const WIDTH: usize = 100;

/// A parameter as written in a NuclearLang signature.
pub fn render_param(p: &Param) -> String {
    let nullable = p.ty.ends_with('?') && !p.ty.ends_with(")?");
    match &p.default {
        Some(d) if d != "null" => format!("{}: {} = {d}", p.name, p.ty),
        _ if p.optional && nullable => format!("{}?: {}", p.name, &p.ty[..p.ty.len() - 1]),
        _ if p.optional => format!("{}?: {}", p.name, p.ty),
        _ => format!("{}: {}", p.name, p.ty),
    }
}

/// `(a: int, {b?: string})`, or one parameter per line when it does not fit in `width`.
pub fn render_params(params: &[Param], indent: &str, width: usize) -> String {
    let positional: Vec<String> = params.iter().filter(|p| !p.named).map(render_param).collect();
    let named: Vec<String> = params.iter().filter(|p| p.named).map(render_param).collect();
    let mut flat = positional.clone();
    if !named.is_empty() {
        flat.push(format!("{{{}}}", named.join(", ")));
    }
    let one = format!("({})", flat.join(", "));
    if indent.len() + one.len() <= width {
        return one;
    }
    let mut s = String::from("(\n");
    for p in &positional {
        s.push_str(&format!("{indent}  {p},\n"));
    }
    if !named.is_empty() {
        s.push_str(&format!("{indent}  {{\n"));
        for p in &named {
            s.push_str(&format!("{indent}    {p},\n"));
        }
        s.push_str(&format!("{indent}  }},\n"));
    }
    s.push_str(&format!("{indent})"));
    s
}

fn generics(tps: &[String]) -> String {
    if tps.is_empty() { String::new() } else { format!("<{}>", tps.join(", ")) }
}

/// One member line: `static of(context: BuildContext) ThemeData`, `style: TextStyle?`.
pub fn render_member(m: &Member, indent: &str) -> String {
    let st = if m.is_static { "static " } else { "" };
    match m.kind.as_str() {
        "property" => {
            let ro = if m.readonly { "  // readonly" } else { "" };
            format!("{st}{}: {}{ro}", m.name, m.ty.as_deref().unwrap_or("dynamic"))
        }
        "operator" => format!(
            "operator {}{} {}",
            m.name,
            render_params(&m.params, indent, WIDTH),
            m.returns.as_deref().unwrap_or("void")
        ),
        _ => format!(
            "{st}{}{}{} {}",
            m.name,
            generics(&m.type_params),
            render_params(&m.params, indent, WIDTH),
            m.returns.as_deref().unwrap_or("void")
        ),
    }
}

/// `Text(data: string, {...})` or `Text.rich(...)`.
pub fn render_constructor(class: &str, c: &Constructor, indent: &str) -> String {
    let name = if c.name.is_empty() { class.to_string() } else { format!("{class}.{}", c.name) };
    format!("{name}{}", render_params(&c.params, indent, WIDTH))
}

/// A declaration in NuclearLang notation, with its library and docs as comments.
pub fn render(p: &Package, d: &Decl) -> String {
    let mut s = String::new();
    let lib = d.library.as_deref().unwrap_or("(not exported)");
    s.push_str(&format!("// {lib} · {}\n", p.id()));
    if d.deprecated {
        s.push_str("// deprecated\n");
    }
    if let Some(doc) = &d.doc {
        s.push_str(&format!("// {doc}\n"));
    }
    let tp = generics(&d.type_params);
    match d.kind.as_str() {
        "function" => {
            s.push_str(&format!(
                "{}{tp}{} {}\n",
                d.name,
                render_params(&d.params, "", WIDTH),
                d.returns.as_deref().unwrap_or("void")
            ));
        }
        "variable" => {
            let ro = if d.readonly { "  // readonly" } else { "" };
            s.push_str(&format!("{}: {}{ro}\n", d.name, d.ty.as_deref().unwrap_or("dynamic")));
        }
        "typedef" => s.push_str(&format!("{}{tp} = {}\n", d.name, d.ty.as_deref().unwrap_or("dynamic"))),
        _ => {
            let mut head = format!("{}{tp}", d.name);
            let bases: Vec<&String> = d.extends.iter().collect();
            if !bases.is_empty() {
                head.push_str(&format!(": {}", bases.iter().map(|b| b.as_str()).collect::<Vec<_>>().join(", ")));
            }
            if !d.implements.is_empty() {
                head.push_str(&format!(" > {}", d.implements.join(", ")));
            }
            if !d.mixins.is_empty() {
                head.push_str(&format!(" with {}", d.mixins.join(", ")));
            }
            if !d.on.is_empty() {
                head.push_str(&format!(" on {}", d.on.join(", ")));
            }
            let mut tags: Vec<&str> = d.modifiers.iter().map(String::as_str).collect();
            if d.kind != "class" {
                tags.insert(0, &d.kind);
            }
            if !tags.is_empty() {
                head.push_str(&format!("  // {}", tags.join(", ")));
            }
            s.push_str(&head);
            s.push('\n');
            for v in &d.values {
                s.push_str(&format!("  {v}\n"));
            }
            for c in &d.constructors {
                if let Some(doc) = &c.doc {
                    s.push_str(&format!("  // {doc}\n"));
                }
                s.push_str(&format!("  {}\n", render_constructor(&d.name, c, "  ")));
            }
            for m in &d.members {
                if let Some(doc) = &m.doc {
                    s.push_str(&format!("  // {doc}\n"));
                }
                s.push_str(&format!("  {}\n", render_member(m, "  ")));
            }
        }
    }
    s
}
