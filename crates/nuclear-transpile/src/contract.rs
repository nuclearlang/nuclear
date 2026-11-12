//! The contract of a project (`.nuclear/contract.json`, ADR-0098): what its API exposes (a backend: controllers
//! and routes) or calls (a mobile or web app: repositories and their http methods), with the models those carry,
//! in Nuclear's notation and independent of the dialect; the OpenAPI document of a backend (`.nuclear/openapi.json`);
//! and the Nuclear a client project generates from the contracts it declares (`contracts(name = "path|url")` in
//! its manifest → `contracts/<name>/`), so its models and repositories are never written by hand.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use nuclear_diag::{Diagnostic, Span, codes};
use nuclear_dialect::Dialect;
use nuclear_syntax::ast::{Arg, Body, Decl, Expr, ExprKind, Item, Member, StrPiece, Type, TypeKind};
use nuclear_syntax::token::Comment;
use serde_json::{Map, Value, json};

use crate::project::{self, Manifest};

pub const CONTRACT_FILE: &str = ".nuclear/contract.json";
pub const OPENAPI_FILE: &str = ".nuclear/openapi.json";
/// The section of the manifest that names the backends a client talks to: `contracts(tasks_api = "../tasks_api")`.
pub const CONTRACTS_SECTION: &str = "contracts";
/// Where a client keeps what it generated from each contract.
pub const CONTRACTS_DIR: &str = "contracts";

const VERBS: [&str; 5] = ["get", "post", "put", "patch", "delete"];
const RULES: [&str; 8] = ["required", "email", "url", "minLength", "maxLength", "min", "max", "pattern"];
const PRIMITIVES: [&str; 11] =
    ["string", "int", "double", "number", "boolean", "bool", "void", "any", "dynamic", "DateTime", "Duration"];

// ---------- the contract of a project ----------

/// The contract of the project in `root`: its models, and the routes it exposes (backend) or the calls it makes
/// (mobile, web). Files that do not parse are skipped (the build reports them).
pub fn contract(root: &Path, manifest: &Manifest, dialect: &Dialect) -> Value {
    let out_dir = root.join(project::output_dir(manifest, dialect));
    let mut models = Map::new();
    let mut controllers = Map::new();
    let mut repositories = Map::new();
    for rel in project::sources(root, &out_dir) {
        if dialect.project.manifest.as_ref().is_some_and(|m| rel == Path::new(&m.file)) {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(root.join(&rel)) else { continue };
        let parsed = nuclear_syntax::parse(&source);
        if parsed.diagnostics.iter().any(Diagnostic::is_error) {
            continue;
        }
        let path = rel.with_extension("").to_string_lossy().replace('\\', "/");
        let component = path.strip_prefix("components/").and_then(|r| r.split('/').next()).unwrap_or("").to_string();
        let default_kind = project::default_kind(dialect, &path);
        let lines = Lines::new(&source, &parsed.file.comments);
        for item in &parsed.file.items {
            let Item::Decl(d) = item else { continue };
            let kind = d.words.iter().find(|w| dialect.kinds.contains_key(&w.name)).map(|w| w.name.clone());
            let kind = kind.or_else(|| default_kind.clone()).unwrap_or_default();
            match kind.as_str() {
                "model" | "entity" | "event" => {
                    models.insert(d.name.name.clone(), model(d, &lines));
                }
                "controller" => {
                    controllers.insert(d.name.name.clone(), api(d, &lines, &component, true));
                }
                // a mobile or web repository is http calls; a backend one is a table (no contract)
                "repository" if dialect.platform.as_deref() != Some("backend") && has_verbs(d) => {
                    repositories.insert(d.name.name.clone(), api(d, &lines, &component, false));
                }
                _ => {}
            }
        }
    }
    json!({
        "app": manifest.name,
        "version": manifest.version,
        "platform": manifest.platform,
        "dialect": dialect.name,
        "models": models,
        "controllers": controllers,
        "repositories": repositories,
    })
}

fn has_verbs(d: &Decl) -> bool {
    let Body::Members(ms) = &d.body else { return false };
    ms.iter().any(|m| matches!(m, Member::Method(x) if x.decorators.iter().any(|y| VERBS.contains(&y.name.as_str()))))
}

/// A model: its fields with types, optionality, defaults, JSON names, validation rules and the key field.
fn model(d: &Decl, lines: &Lines) -> Value {
    let mut fields = Vec::new();
    let mut key = Value::Null;
    if let Body::Members(ms) = &d.body {
        for m in ms {
            let Member::Field(f) = m else { continue };
            let mut field = Map::new();
            field.insert("name".into(), f.name.name.clone().into());
            field.insert("type".into(), type_text(f.ty.as_ref()).into());
            if f.optional {
                field.insert("optional".into(), true.into());
            }
            if let Some(v) = f.default.as_ref().and_then(literal) {
                field.insert("default".into(), v);
            }
            if let Some(k) = f.decorators.iter().find(|x| x.name == "key").and_then(|x| first_str(x.args.as_deref())) {
                field.insert("json".into(), k.into());
            }
            let mut rules = Map::new();
            for x in &f.decorators {
                if RULES.contains(&x.name.as_str()) {
                    let v = x.args.as_deref().and_then(|a| a.first()).and_then(|a| literal(&a.value)).unwrap_or(Value::Bool(true));
                    rules.insert(x.name.clone(), v);
                }
                if x.name == "primary" || x.name == "generated" {
                    key = f.name.name.clone().into();
                    field.insert("key".into(), x.name.clone().into());
                }
            }
            if !rules.is_empty() {
                field.insert("rules".into(), Value::Object(rules));
            }
            if let Some(doc) = lines.doc(f.span.start) {
                field.insert("description".into(), doc.into());
            }
            fields.push(Value::Object(field));
        }
    }
    let mut m = Map::new();
    if let Some(doc) = lines.doc(d.span.start) {
        m.insert("description".into(), doc.into());
    }
    if !d.generics.is_empty() {
        m.insert("generics".into(), d.generics.iter().map(|g| Value::String(g.name.clone())).collect());
    }
    m.insert("key".into(), key);
    m.insert("fields".into(), Value::Array(fields));
    Value::Object(m)
}

/// A controller (routes it exposes) or a client repository (calls it makes): the same shape.
fn api(d: &Decl, lines: &Lines, component: &str, backend: bool) -> Value {
    let head = d.decorators.iter().find(|x| x.name == if backend { "controller" } else { "repository" });
    let base = head.and_then(|x| named_or_first(x.args.as_deref(), "path")).unwrap_or_default();
    let connection = head.and_then(|x| named_str(x.args.as_deref(), "connection")).unwrap_or_default();
    let mut routes = Vec::new();
    if let Body::Members(ms) = &d.body {
        for m in ms {
            let Member::Method(x) = m else { continue };
            let Some(verb) = x.decorators.iter().find(|y| VERBS.contains(&y.name.as_str())) else { continue };
            let path = named_or_first(verb.args.as_deref(), "path").unwrap_or_default();
            let mut r = Map::new();
            r.insert("name".into(), x.name.name.clone().into());
            r.insert("verb".into(), verb.name.clone().into());
            r.insert("path".into(), path.clone().into());
            if let Some(s) = x.decorators.iter().find(|y| y.name == "status").and_then(|y| y.args.as_deref()).and_then(|a| a.first()).and_then(|a| literal(&a.value)) {
                r.insert("status".into(), s);
            }
            if let Some(doc) = lines.doc(x.span.start) {
                r.insert("description".into(), doc.into());
            }
            let read = verb.name == "get" || verb.name == "delete";
            let mut params = Vec::new();
            for p in x.params.as_ref().map(|p| p.list.as_slice()).unwrap_or_default() {
                let mut pm = Map::new();
                pm.insert("name".into(), p.name.name.clone().into());
                pm.insert("type".into(), type_text(p.ty.as_ref()).into());
                if p.optional {
                    pm.insert("optional".into(), true.into());
                }
                if let Some(v) = p.default.as_ref().and_then(literal) {
                    pm.insert("default".into(), v);
                }
                let forced = p.decorators.iter().find(|y| ["path", "query", "body"].contains(&y.name.as_str())).map(|y| y.name.clone());
                let place = forced.unwrap_or_else(|| {
                    if in_path(&path, &p.name.name) || in_path(&base, &p.name.name) {
                        "path".into()
                    } else if !read && !is_primitive(p.ty.as_ref()) {
                        "body".into()
                    } else if read {
                        "query".into()
                    } else {
                        "body".into()
                    }
                });
                pm.insert("in".into(), place.into());
                params.push(Value::Object(pm));
            }
            r.insert("params".into(), Value::Array(params));
            let (returns, error) = result_parts(x.ret.as_ref());
            r.insert("returns".into(), returns.into());
            if let Some(e) = error {
                r.insert("error".into(), e.into());
            }
            routes.push(Value::Object(r));
        }
    }
    let mut m = Map::new();
    m.insert("component".into(), component.into());
    m.insert("path".into(), base.into());
    if !backend {
        m.insert("connection".into(), connection.into());
    }
    if let Some(doc) = lines.doc(d.span.start) {
        m.insert("description".into(), doc.into());
    }
    m.insert("routes".into(), Value::Array(routes));
    Value::Object(m)
}

fn in_path(path: &str, name: &str) -> bool {
    path.contains(&format!("{{{name}}}")) || path.split('/').any(|s| s == format!(":{name}"))
}

fn is_primitive(t: Option<&Type>) -> bool {
    match t.map(|t| &t.kind) {
        Some(TypeKind::Named { name, .. }) => PRIMITIVES.contains(&name.as_str()),
        Some(TypeKind::Nullable(inner)) => is_primitive(Some(inner)),
        Some(TypeKind::List(inner)) => is_primitive(Some(inner)),
        _ => true,
    }
}

/// `Result<E, T>` → (T, Some(E)); another type → (type, None).
fn result_parts(t: Option<&Type>) -> (String, Option<String>) {
    if let Some(Type { kind: TypeKind::Named { name, args }, .. }) = t
        && name == "Result"
        && args.len() == 2
    {
        return (type_text(args.get(1)), Some(type_text(args.first())));
    }
    (type_text(t), None)
}

/// A type in Nuclear's notation (what any dialect reads back).
pub fn type_text(t: Option<&Type>) -> String {
    let Some(t) = t else { return "void".into() };
    match &t.kind {
        TypeKind::Named { name, args } if args.is_empty() => name.clone(),
        TypeKind::Named { name, args } => {
            format!("{name}<{}>", args.iter().map(|a| type_text(Some(a))).collect::<Vec<_>>().join(", "))
        }
        TypeKind::List(inner) => format!("{}[]", type_text(Some(inner))),
        TypeKind::Map(k, v) => format!("<{}, {}>", type_text(Some(k)), type_text(Some(v))),
        TypeKind::Nullable(inner) => format!("{}?", type_text(Some(inner))),
        TypeKind::Union(ts) => ts.iter().map(|x| type_text(Some(x))).collect::<Vec<_>>().join(" | "),
        TypeKind::StrLit(s) => format!("'{s}'"),
        TypeKind::NumLit(n) => n.clone(),
        TypeKind::Func { params, ret } => format!(
            "({}) => {}",
            params.iter().map(|p| format!("{}: {}", p.name.name, type_text(p.ty.as_ref()))).collect::<Vec<_>>().join(", "),
            type_text(Some(ret))
        ),
    }
}

/// A literal as JSON (a default, a decorator argument); `None` for an expression.
fn literal(e: &Expr) -> Option<Value> {
    match &e.kind {
        ExprKind::Str(pieces) => {
            let mut s = String::new();
            for p in pieces {
                match p {
                    StrPiece::Text(t) => s.push_str(t),
                    StrPiece::Interp(_) => return None,
                }
            }
            Some(Value::String(s))
        }
        ExprKind::Number(n) => {
            let text = n.text.replace('_', "");
            text.parse::<i64>().map(Value::from).ok().or_else(|| text.parse::<f64>().ok().map(Value::from))
        }
        ExprKind::Bool(b) => Some(Value::Bool(*b)),
        ExprKind::Null => Some(Value::Null),
        _ => None,
    }
}

fn first_str(args: Option<&[Arg]>) -> Option<String> {
    args?.iter().find(|a| a.name.is_none()).and_then(|a| literal(&a.value)).and_then(|v| v.as_str().map(String::from))
}

fn named_str(args: Option<&[Arg]>, name: &str) -> Option<String> {
    args?
        .iter()
        .find(|a| a.name.as_ref().is_some_and(|n| n.name == name))
        .and_then(|a| literal(&a.value))
        .and_then(|v| v.as_str().map(String::from))
}

fn named_or_first(args: Option<&[Arg]>, name: &str) -> Option<String> {
    named_str(args, name).or_else(|| first_str(args))
}

/// Doc comments: the own-line comments right above a position.
pub(crate) struct Lines<'a> {
    starts: Vec<usize>,
    comments: &'a [Comment],
}

impl<'a> Lines<'a> {
    pub(crate) fn new(source: &str, comments: &'a [Comment]) -> Self {
        let mut starts = vec![0];
        starts.extend(source.match_indices('\n').map(|(i, _)| i + 1));
        Lines { starts, comments }
    }

    fn line(&self, pos: usize) -> usize {
        self.starts.partition_point(|&s| s <= pos)
    }

    pub(crate) fn doc(&self, pos: usize) -> Option<String> {
        let mut want = self.line(pos);
        let mut texts = Vec::new();
        for c in self.comments.iter().rev() {
            if c.span.end > pos {
                continue;
            }
            if !c.own_line || self.line(c.span.end) + 1 != want {
                break;
            }
            texts.push(c.text.trim().to_string());
            want = self.line(c.span.start);
        }
        if texts.is_empty() {
            return None;
        }
        texts.reverse();
        Some(texts.join(" "))
    }
}

// ---------- OpenAPI ----------

/// The OpenAPI 3.1 document of a backend contract: its routes and models, so any client (Swagger UI, a generator,
/// an HTTP tool) reads the API without Nuclear.
pub fn openapi(contract: &Value) -> Value {
    let mut paths: Map<String, Value> = Map::new();
    let mut schemas = Map::new();
    for (name, m) in contract["models"].as_object().into_iter().flatten() {
        schemas.insert(name.clone(), schema_of_model(m));
    }
    schemas.insert("Error".into(), json!({
        "type": "object",
        "properties": { "message": { "type": "string" }, "status": { "type": "integer" } },
        "required": ["message"],
    }));
    for (cname, c) in contract["controllers"].as_object().into_iter().flatten() {
        let tag = c["component"].as_str().filter(|s| !s.is_empty()).unwrap_or(cname);
        for r in c["routes"].as_array().into_iter().flatten() {
            let full = openapi_path(c["path"].as_str().unwrap_or(""), r["path"].as_str().unwrap_or(""));
            let mut op = Map::new();
            op.insert("operationId".into(), format!("{cname}.{}", r["name"].as_str().unwrap_or("")).into());
            op.insert("tags".into(), json!([tag]));
            if let Some(d) = r["description"].as_str() {
                op.insert("summary".into(), d.into());
            }
            let mut params = Vec::new();
            for p in r["params"].as_array().into_iter().flatten() {
                let place = p["in"].as_str().unwrap_or("query");
                if place == "body" {
                    op.insert("requestBody".into(), json!({
                        "required": true,
                        "content": { "application/json": { "schema": schema_of_type(p["type"].as_str().unwrap_or("any")) } },
                    }));
                    continue;
                }
                let mut pm = Map::new();
                pm.insert("name".into(), p["name"].clone());
                pm.insert("in".into(), place.into());
                pm.insert("required".into(), (place == "path" || (!p["optional"].as_bool().unwrap_or(false) && p["default"].is_null())).into());
                let mut schema = schema_of_type(p["type"].as_str().unwrap_or("string"));
                if let Some(d) = p.get("default") {
                    schema["default"] = d.clone();
                }
                pm.insert("schema".into(), schema);
                params.push(Value::Object(pm));
            }
            op.insert("parameters".into(), Value::Array(params));
            let status = r["status"].as_i64().unwrap_or(if r["verb"] == "post" { 201 } else { 200 });
            let returns = r["returns"].as_str().unwrap_or("void");
            let ok = if returns == "void" {
                json!({ "description": "ok" })
            } else {
                json!({ "description": "ok", "content": { "application/json": { "schema": schema_of_type(returns) } } })
            };
            op.insert("responses".into(), json!({
                status.to_string(): ok,
                "default": { "description": "the error of the use case", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } },
            }));
            let entry = paths.entry(full).or_insert_with(|| Value::Object(Map::new()));
            entry[r["verb"].as_str().unwrap_or("get")] = Value::Object(op);
        }
    }
    json!({
        "openapi": "3.1.0",
        "info": { "title": contract["app"], "version": contract["version"] },
        "paths": paths,
        "components": { "schemas": schemas },
    })
}

fn openapi_path(base: &str, path: &str) -> String {
    let joined = [base.trim_matches('/'), path.trim_matches('/')].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join("/");
    let converted: Vec<String> = joined
        .split('/')
        .map(|s| if let Some(n) = s.strip_prefix(':') { format!("{{{n}}}") } else { s.to_string() })
        .collect();
    format!("/{}", converted.join("/"))
}

fn schema_of_model(m: &Value) -> Value {
    let mut props = Map::new();
    let mut required = Vec::new();
    for f in m["fields"].as_array().into_iter().flatten() {
        let name = f["json"].as_str().or(f["name"].as_str()).unwrap_or("").to_string();
        let mut s = schema_of_type(f["type"].as_str().unwrap_or("any"));
        if f["optional"].as_bool().unwrap_or(false) && s.is_object() {
            s["nullable"] = true.into();
        }
        if let Some(d) = f.get("default") {
            s["default"] = d.clone();
        }
        if let Some(desc) = f["description"].as_str() {
            s["description"] = desc.into();
        }
        for (rule, v) in f["rules"].as_object().into_iter().flatten() {
            match rule.as_str() {
                "minLength" | "maxLength" | "pattern" => s[rule] = v.clone(),
                "min" => s["minimum"] = v.clone(),
                "max" => s["maximum"] = v.clone(),
                "email" => s["format"] = "email".into(),
                "url" => s["format"] = "uri".into(),
                _ => {}
            }
        }
        if !f["optional"].as_bool().unwrap_or(false) && f.get("default").is_none() {
            required.push(Value::String(name.clone()));
        }
        props.insert(name, s);
    }
    let mut out = json!({ "type": "object", "properties": props });
    if !required.is_empty() {
        out["required"] = Value::Array(required);
    }
    if let Some(d) = m["description"].as_str() {
        out["description"] = d.into();
    }
    out
}

/// A Nuclear type as a JSON schema (`int` → integer, `User[]` → array of `$ref`, `string?` → nullable string).
fn schema_of_type(t: &str) -> Value {
    let t = t.trim();
    if let Some(inner) = t.strip_suffix('?') {
        let mut s = schema_of_type(inner);
        s["nullable"] = true.into();
        return s;
    }
    if let Some(inner) = t.strip_suffix("[]") {
        return json!({ "type": "array", "items": schema_of_type(inner) });
    }
    if t.starts_with('<') && t.ends_with('>') {
        let value = t[1..t.len() - 1].rsplit_once(',').map(|(_, v)| v.trim()).unwrap_or("any");
        return json!({ "type": "object", "additionalProperties": schema_of_type(value) });
    }
    match t {
        "int" => json!({ "type": "integer" }),
        "double" | "number" => json!({ "type": "number" }),
        "string" => json!({ "type": "string" }),
        "boolean" | "bool" => json!({ "type": "boolean" }),
        "DateTime" => json!({ "type": "string", "format": "date-time" }),
        "void" => json!({}),
        "any" | "dynamic" => json!({}),
        _ => {
            let name = t.split('<').next().unwrap_or(t);
            json!({ "$ref": format!("#/components/schemas/{name}") })
        }
    }
}

// ---------- writing ----------

/// Writes the contract (and the OpenAPI document of a backend) under `root/.nuclear/` and in the output folder,
/// when their content changed.
pub fn write(root: &Path, manifest: &Manifest, dialect: &Dialect) -> std::io::Result<()> {
    let c = contract(root, manifest, dialect);
    let out = root.join(project::output_dir(manifest, dialect));
    write_if_changed(&root.join(CONTRACT_FILE), &c)?;
    write_if_changed(&out.join(CONTRACT_FILE), &c)?;
    if manifest.platform.as_deref() == Some("backend") {
        let o = openapi(&c);
        write_if_changed(&root.join(OPENAPI_FILE), &o)?;
        write_if_changed(&out.join(OPENAPI_FILE), &o)?;
    }
    Ok(())
}

fn write_if_changed(path: &Path, value: &Value) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(value)? + "\n";
    if std::fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    std::fs::create_dir_all(path.parent().expect("file in a folder"))?;
    std::fs::write(path, text)
}

// ---------- a client: the contracts it declares ----------

/// `contracts(name = "../path" | "https://...")` of the manifest.
pub fn declared(root: &Path, dialect: &Dialect) -> Vec<(String, String)> {
    let Some(m) = &dialect.project.manifest else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(root.join(&m.file)) else { return Vec::new() };
    let (config, _) = project::read_config(&text, &[CONTRACTS_SECTION.to_string()]);
    config.get(CONTRACTS_SECTION).cloned().unwrap_or_default()
}

/// Regenerates `contracts/<name>/` for every contract the manifest declares: from the path's
/// `.nuclear/contract.json` (a backend built next to this project) or from the copy `nuclear contracts update`
/// fetched for a url. Returns the diagnostics (N0307: a contract that is not there).
pub fn sync(root: &Path, dialect: &Dialect) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (name, source) in declared(root, dialect) {
        let local = root.join(CONTRACTS_DIR).join(&name).join("contract.json");
        let text = if source.starts_with("http://") || source.starts_with("https://") {
            match std::fs::read_to_string(&local) {
                Ok(t) => t,
                Err(_) => {
                    diags.push(Diagnostic::error(
                        codes::CONTRACT,
                        Span::default(),
                        format!("contract '{name}' was never fetched from {source}: run `nuclear contracts update`"),
                    ));
                    continue;
                }
            }
        } else {
            let path = root.join(&source).join(CONTRACT_FILE);
            match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => {
                    diags.push(Diagnostic::error(
                        codes::CONTRACT,
                        Span::default(),
                        format!("contract '{name}' not found at {}: build that project first (nuclear build {source})", path.display()),
                    ));
                    continue;
                }
            }
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                diags.push(Diagnostic::error(codes::CONTRACT, Span::default(), format!("contract '{name}': invalid JSON: {e}")));
                continue;
            }
        };
        if let Err(e) = install(root, &name, &value, dialect) {
            diags.push(Diagnostic::error(codes::CONTRACT, Span::default(), format!("contract '{name}': {e}")));
        }
    }
    diags
}

/// Writes the local copy of a contract and the Nuclear generated from it, removing what is no longer generated.
pub fn install(root: &Path, name: &str, contract: &Value, dialect: &Dialect) -> std::io::Result<()> {
    let dir = root.join(CONTRACTS_DIR).join(name);
    let mut files: BTreeMap<PathBuf, String> = generate(name, contract, dialect)
        .into_iter()
        .map(|(rel, text)| (dir.join(rel), text))
        .collect();
    files.insert(dir.join("contract.json"), serde_json::to_string_pretty(contract)? + "\n");
    // stale files of a previous contract
    let mut pending = vec![dir.clone()];
    while let Some(d) = pending.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                pending.push(p);
            } else if !files.contains_key(&p) {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    for (path, text) in files {
        if std::fs::read_to_string(&path).ok().as_deref() == Some(text.as_str()) {
            continue;
        }
        std::fs::create_dir_all(path.parent().expect("file in a folder"))?;
        std::fs::write(&path, text)?;
    }
    Ok(())
}

/// The Nuclear a client writes for a contract: one model per model a route carries, one repository per controller
/// and the error model. Only decorators the client's dialect consumes are written (the rules of a backend model
/// mean nothing to a dialect without forms).
pub fn generate(name: &str, contract: &Value, dialect: &Dialect) -> Vec<(PathBuf, String)> {
    let pascal = pascal(name);
    let error = format!("{pascal}Error");
    let header = format!(
        "// Generated from the contract {} {} (nuclear contracts). Do not edit: `nuclear contracts update` rewrites it.\n",
        contract["app"].as_str().unwrap_or(name),
        contract["version"].as_str().unwrap_or("")
    );
    let consumes = |d: &str| dialect.consumed_decorators.contains(d) || dialect.void_decorators.contains(d);
    let models = contract["models"].as_object().cloned().unwrap_or_default();
    // the models the routes carry, transitively
    let mut needed: BTreeSet<String> = BTreeSet::new();
    let mut pending: Vec<String> = Vec::new();
    for c in contract["controllers"].as_object().into_iter().flatten().map(|(_, c)| c) {
        for r in c["routes"].as_array().into_iter().flatten() {
            pending.extend(type_names(r["returns"].as_str().unwrap_or("")));
            for p in r["params"].as_array().into_iter().flatten() {
                pending.extend(type_names(p["type"].as_str().unwrap_or("")));
            }
        }
    }
    while let Some(n) = pending.pop() {
        if !models.contains_key(&n) || !needed.insert(n.clone()) {
            continue;
        }
        for f in models[&n]["fields"].as_array().into_iter().flatten() {
            pending.extend(type_names(f["type"].as_str().unwrap_or("")));
        }
    }
    let mut out = Vec::new();
    for n in &needed {
        let m = &models[n];
        let mut text = header.clone();
        if let Some(d) = m["description"].as_str() {
            text.push_str(&format!("// {d}\n"));
        }
        let generics = m["generics"].as_array().map(|g| format!("<{}>", g.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", "))).unwrap_or_default();
        text.push_str(&format!("{n}{generics}\n"));
        for f in m["fields"].as_array().into_iter().flatten() {
            if let Some(d) = f["description"].as_str() {
                text.push_str(&format!("  // {d}\n"));
            }
            if f["key"].is_string() && consumes("primary") {
                text.push_str("  @primary\n");
            }
            if let Some(k) = f["json"].as_str()
                && consumes("key")
            {
                text.push_str(&format!("  @key(\"{k}\")\n"));
            }
            for (rule, v) in f["rules"].as_object().into_iter().flatten() {
                if !consumes(rule) {
                    continue;
                }
                match v {
                    Value::Bool(true) => text.push_str(&format!("  @{rule}\n")),
                    Value::String(s) => text.push_str(&format!("  @{rule}(\"{s}\")\n")),
                    other => text.push_str(&format!("  @{rule}({other})\n")),
                }
            }
            let opt = if f["optional"].as_bool().unwrap_or(false) { "?" } else { "" };
            let default = f.get("default").map(|d| format!(" = {}", nuclear_literal(d))).unwrap_or_default();
            text.push_str(&format!("  {}{opt}: {}{default}\n", f["name"].as_str().unwrap_or(""), f["type"].as_str().unwrap_or("any")));
        }
        out.push((PathBuf::from("models").join(format!("{}.nuc", snake(n))), text));
    }
    // the error of every call: what the API answers when a use case fails
    let mut text = header.clone();
    text.push_str("// The error a call of this contract answers with: the message of the API or of the connection.\n");
    text.push_str(&format!("{error}\n  message: string\n"));
    out.push((PathBuf::from("models").join(format!("{}.nuc", snake(&error))), text));
    for (cname, c) in contract["controllers"].as_object().into_iter().flatten() {
        let repo = format!("{}Repository", cname.strip_suffix("Controller").unwrap_or(cname));
        let mut text = header.clone();
        if let Some(d) = c["description"].as_str() {
            text.push_str(&format!("// {d}\n"));
        }
        let base = c["path"].as_str().unwrap_or("");
        text.push_str(&format!("@repository(path = \"{base}\", connection = \"{name}\")\nrepository {repo}\n"));
        let routes = c["routes"].as_array().cloned().unwrap_or_default();
        for (i, r) in routes.iter().enumerate() {
            if i > 0 {
                text.push('\n');
            }
            if let Some(d) = r["description"].as_str() {
                text.push_str(&format!("  // {d}\n"));
            }
            let verb = r["verb"].as_str().unwrap_or("get");
            let path = client_path(r["path"].as_str().unwrap_or(""));
            let returns = r["returns"].as_str().unwrap_or("void");
            let keyed = type_names(returns).iter().any(|t| models.get(t).is_some_and(|m| m["key"].is_string()))
                && !returns.contains('<');
            let mut args = Vec::new();
            if !path.is_empty() {
                args.push(format!("path = \"{path}\""));
            }
            if verb == "get" && keyed && consumes("primary") {
                args.push("local = true".into());
            }
            let deco = if args.is_empty() { format!("  @{verb}\n") } else { format!("  @{verb}({})\n", args.join(", ")) };
            text.push_str(&deco);
            let mut params = Vec::new();
            for p in r["params"].as_array().into_iter().flatten() {
                let place = p["in"].as_str().unwrap_or("query");
                let read = verb == "get" || verb == "delete";
                let natural = if in_path(&path, p["name"].as_str().unwrap_or("")) {
                    "path"
                } else if read {
                    "query"
                } else {
                    "body"
                };
                let prefix = if place != natural { format!("@{place} ") } else { String::new() };
                let opt = if p["optional"].as_bool().unwrap_or(false) { "?" } else { "" };
                let default = p.get("default").map(|d| format!(" = {}", nuclear_literal(d))).unwrap_or_default();
                params.push(format!("{prefix}{}{opt}: {}{default}", p["name"].as_str().unwrap_or(""), p["type"].as_str().unwrap_or("any")));
            }
            text.push_str(&format!("  {}({}) Result<{error}, {returns}>\n", r["name"].as_str().unwrap_or("call"), params.join(", ")));
        }
        out.push((PathBuf::from(format!("{}.nuc", snake(&repo))), text));
    }
    out
}

/// `{id}` of a backend route is `:id` in a client call.
fn client_path(path: &str) -> String {
    path.split('/')
        .map(|s| if s.starts_with('{') && s.ends_with('}') { format!(":{}", &s[1..s.len() - 1]) } else { s.to_string() })
        .collect::<Vec<_>>()
        .join("/")
}

/// The PascalCase names inside a type text (`Page<Task>` → Page, Task).
fn type_names(t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in t.chars().chain(std::iter::once(' ')) {
        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else {
            if cur.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && !PRIMITIVES.contains(&cur.as_str()) {
                out.push(cur.clone());
            }
            cur.clear();
        }
    }
    out
}

fn nuclear_literal(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 && !out.ends_with('_') {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

fn pascal(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut cs = s.chars();
            cs.next().map(|c| c.to_ascii_uppercase().to_string() + cs.as_str()).unwrap_or_default()
        })
        .collect()
}

/// Fetches a contract from a url (`<url>/.nuclear/contract.json`, with curl) or a path.
pub fn fetch(source: &str, root: &Path) -> Result<String, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let url = format!("{}/{}", source.trim_end_matches('/'), CONTRACT_FILE);
        let out = std::process::Command::new("curl")
            .args(["-fsSL", &url])
            .output()
            .map_err(|e| format!("cannot run curl: {e}"))?;
        if !out.status.success() {
            return Err(format!("{url}: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        return String::from_utf8(out.stdout).map_err(|e| e.to_string());
    }
    let path = root.join(source).join(CONTRACT_FILE);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}
