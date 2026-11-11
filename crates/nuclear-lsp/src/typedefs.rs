//! Type definitions of the dialect (`Dialect.typedefs`) as the editor uses them: a prefix index of the public
//! names (built once per loaded dialect), instance and static members of a type with its generic arguments
//! substituted, and the parameters of external constructors, functions and methods.

use nuclear_dialect::defs::{Decl, Member, Package, TypeDefs};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Public names sorted by their lowercase form, and the extensions of each type.
pub struct Names {
    sorted: Vec<(String, u32, u32)>,
    extensions: HashMap<String, Vec<(u32, u32)>>,
}

/// The index of `td`, built the first time it is asked for. Only the last dialect is kept: the server has one
/// active dialect, and a reload builds a new `TypeDefs`.
pub fn names(td: &Arc<TypeDefs>) -> Arc<Names> {
    static CACHE: Mutex<Option<(Arc<TypeDefs>, Arc<Names>)>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((owner, names)) = cache.as_ref()
        && Arc::ptr_eq(owner, td)
    {
        return names.clone();
    }
    let names = Arc::new(build(td));
    *cache = Some((td.clone(), names.clone()));
    names
}

fn build(td: &TypeDefs) -> Names {
    let mut sorted = Vec::new();
    let mut extensions: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
    let mut seen = HashSet::new();
    for (pi, p) in td.packages.iter().enumerate() {
        for (di, d) in p.api().unwrap_or(&[]).iter().enumerate() {
            if d.name.starts_with('_') {
                continue;
            }
            if d.kind == "extension" {
                for on in &d.on {
                    extensions.entry(base(on).to_string()).or_default().push((pi as u32, di as u32));
                }
            } else if seen.insert(d.name.clone()) {
                sorted.push((d.name.to_lowercase(), pi as u32, di as u32));
            }
        }
    }
    sorted.sort();
    Names { sorted, extensions }
}

impl Names {
    /// Declarations whose name starts with `prefix` (ignoring case), in name order.
    pub fn with_prefix<'t>(&self, td: &'t TypeDefs, prefix: &str) -> impl Iterator<Item = (&'t Package, &'t Decl)> {
        let lower = prefix.to_lowercase();
        let start = self.sorted.partition_point(|(n, ..)| n.as_str() < lower.as_str());
        self.sorted[start..]
            .iter()
            .take_while(move |(n, ..)| n.starts_with(&lower))
            .filter_map(|&(_, pi, di)| decl(td, pi, di))
    }

    fn extensions_on<'t>(&self, td: &'t TypeDefs, ty: &str) -> Vec<&'t Decl> {
        self.extensions.get(ty).into_iter().flatten().filter_map(|&(pi, di)| decl(td, pi, di).map(|(_, d)| d)).collect()
    }
}

fn decl(td: &TypeDefs, pi: u32, di: u32) -> Option<(&Package, &Decl)> {
    let p = td.packages.get(pi as usize)?;
    Some((p, p.api().ok()?.get(di as usize)?))
}

// ---------------------------------------------------------------- types in Nuclear notation

/// `Response<T>?` → (`Response`, [`T`]); `int[]` → (`List`, [`int`]); `<string, int>` → (`Map`, ...);
/// `string` → `String` (the declarations keep the names of the target language).
pub fn split(ty: &str) -> (String, Vec<String>) {
    let t = ty.trim().trim_end_matches('?').trim();
    if let Some(inner) = t.strip_suffix("[]") {
        return ("List".into(), vec![inner.to_string()]);
    }
    if let Some(inner) = t.strip_prefix('<').and_then(|r| r.strip_suffix('>')) {
        return ("Map".into(), top_level(inner));
    }
    let (name, args) = match t.split_once('<') {
        Some((n, rest)) => (n.trim(), top_level(rest.strip_suffix('>').unwrap_or(rest))),
        None => (t, Vec::new()),
    };
    let name = match name {
        "string" => "String",
        "boolean" | "bool" => "bool",
        "number" | "num" => "num",
        n => n,
    };
    (name.to_string(), args)
}

fn base(ty: &str) -> String {
    split(ty).0
}

/// `a, Map<b, c>` → [`a`, `Map<b, c>`].
fn top_level(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Replaces the type parameters of `map` in `ty`: `BlocBase<State>` with {State: int} → `BlocBase<int>`.
fn subst(ty: &str, map: &HashMap<&str, &str>) -> String {
    if map.is_empty() {
        return ty.to_string();
    }
    let mut out = String::with_capacity(ty.len());
    let mut word = String::new();
    for c in ty.chars().chain(std::iter::once('\0')) {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        out.push_str(map.get(word.as_str()).copied().unwrap_or(&word));
        word.clear();
        if c != '\0' {
            out.push(c);
        }
    }
    out
}

/// Type parameter names of a declaration (`T: Comparable<T>` → `T`).
fn params_of(d: &Decl) -> impl Iterator<Item = &str> {
    d.type_params.iter().map(|t| t.split(':').next().unwrap_or(t).trim())
}

/// The declaration of a type written in Nuclear notation.
pub fn type_decl<'t>(td: &'t TypeDefs, ty: &str) -> Option<(&'t Package, &'t Decl)> {
    td.get(&base(ty))
}

/// A member with its type (property) or return type (method) after substituting the generic arguments.
pub struct Found<'t> {
    pub owner: &'t Decl,
    pub member: &'t Member,
    pub ty: Option<String>,
}

/// Instance members of `ty`: its own first, then those of its supertypes (with the generic arguments
/// substituted) and of the extensions on each type of the chain. A name appears once.
pub fn instance_members<'t>(td: &'t TypeDefs, names: &Names, ty: &str) -> Vec<Found<'t>> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::from([ty.to_string()]);
    while let Some(t) = queue.pop_front() {
        let (name, args) = split(&t);
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some((_, d)) = td.get(&name) else { continue };
        let map: HashMap<&str, &str> = params_of(d).zip(args.iter().map(String::as_str)).collect();
        let exts = names.extensions_on(td, &name);
        for (owner, m) in
            d.members.iter().map(|m| (d, m)).chain(exts.iter().flat_map(|e| e.members.iter().map(move |m| (*e, m))))
        {
            if m.is_static || m.kind == "operator" || m.name.starts_with('_') || !seen.insert(m.name.clone()) {
                continue;
            }
            let ty = m.ty.as_deref().or(m.returns.as_deref()).map(|t| subst(t, &map));
            out.push(Found { owner, member: m, ty });
        }
        for s in d.extends.iter().chain(&d.mixins).chain(&d.implements).chain(&d.on) {
            queue.push_back(subst(s, &map));
        }
    }
    out
}

/// The type of member `name` of an instance of `ty`.
pub fn member_type(td: &TypeDefs, names: &Names, ty: &str, name: &str) -> Option<String> {
    instance_members(td, names, ty).into_iter().find(|f| f.member.name == name).and_then(|f| f.ty)
}

/// What `Class.x` offers: static members, named constructors and enum values.
pub enum StaticItem<'t> {
    Member(&'t Member),
    Constructor(&'t nuclear_dialect::defs::Constructor),
    Value(&'t str),
}

pub fn static_items(d: &Decl) -> Vec<StaticItem<'_>> {
    let mut out: Vec<StaticItem> = d.values.iter().map(|v| StaticItem::Value(v)).collect();
    out.extend(d.constructors.iter().filter(|c| !c.name.is_empty()).map(StaticItem::Constructor));
    out.extend(d.members.iter().filter(|m| m.is_static && !m.name.starts_with('_')).map(StaticItem::Member));
    out
}

/// The type that `Class.name(...)` returns, when `name` is a named constructor or a static method.
pub fn static_call_type(d: &Decl, name: &str) -> Option<String> {
    if d.constructors.iter().any(|c| c.name == name) {
        return Some(d.name.clone());
    }
    d.members.iter().find(|m| m.is_static && m.name == name).and_then(|m| m.returns.clone().or(m.ty.clone()))
}
