//! Server state and request dispatch.

use crate::dialects;
use crate::features;
use crate::index::{Index, TOKEN_MODIFIERS, TOKEN_TYPES};
use crate::metrics;
use crate::project::{self, Project};
use crate::rpc::{self, Incoming, Sender};
use crate::text::LineIndex;
use nuclear_diag::{Diagnostic, Severity, Span, codes};
use nuclear_dialect::Dialect;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::Arc;

/// An open `.nuc` document and everything computed from it.
pub struct Doc {
    pub uri: String,
    pub version: i64,
    pub text: String,
    pub lines: LineIndex,
    pub index: Index,
    pub diagnostics: Vec<Diagnostic>,
    /// The syntax tree (for the code metrics, which depend on settings that can change).
    pub ast: nuclear_syntax::ast::File,
    /// The project of the file and the file's path in it (`components/users/user`).
    pub project: Option<(Arc<Project>, String)>,
}

impl Doc {
    pub fn new(uri: String, version: i64, text: String, dialect: Option<&Dialect>) -> Doc {
        Doc::in_project(uri, version, text, dialect, None)
    }

    pub fn in_project(
        uri: String,
        version: i64,
        text: String,
        dialect: Option<&Dialect>,
        project: Option<(Arc<Project>, String)>,
    ) -> Doc {
        let lines = LineIndex::new(&text);
        let (index, diagnostics, ast) =
            analyze(&text, dialect, project.as_ref().map(|(p, path)| (p.as_ref(), path.as_str())));
        Doc { uri, version, text, lines, index, diagnostics, ast, project }
    }

    /// A top-level declaration of another file of the project, visible from this one.
    pub fn project_name(&self, name: &str) -> Option<&project::Name> {
        let (p, path) = self.project.as_ref()?;
        p.visible(path).find(|n| n.name == name)
    }

    /// Code metrics of the document, or `None` when they do not apply (excluded, too big, parse panic).
    pub fn metrics(&self, cfg: &metrics::Config) -> Option<metrics::Model> {
        if !cfg.applies_to(&self.uri, &self.text) {
            return None;
        }
        catch_unwind(AssertUnwindSafe(|| metrics::compute(&self.text, &self.lines, &self.ast, cfg))).ok()
    }
}

/// Parses, indexes and checks a source. A panic in the compiler never takes the server down.
/// In a project, a file saved as it is gets the diagnostics of the project build (other files' names,
/// architecture rules); while it has unsaved changes it is checked alone, without N0106 for the names of the
/// other files.
pub fn analyze(
    text: &str,
    dialect: Option<&Dialect>,
    project: Option<(&Project, &str)>,
) -> (Index, Vec<Diagnostic>, nuclear_syntax::ast::File) {
    let default_kind = project.and_then(|(p, path)| p.default_kind(path));
    let (index, ast) = catch_unwind(AssertUnwindSafe(|| {
        let parsed = nuclear_syntax::parse(text);
        (Index::build(text, &parsed.file, dialect, default_kind.as_deref()), parsed.file)
    }))
    .unwrap_or_default();
    if let Some(built) = project.and_then(|(p, path)| p.built(path, text)) {
        return (index, built.diagnostics.clone(), ast);
    }
    let mut diagnostics =
        catch_unwind(AssertUnwindSafe(|| nuclear_transpile::check(text, dialect))).unwrap_or_default();
    if let Some((p, path)) = project {
        diagnostics.retain(|d| {
            d.code.id != codes::UNDEFINED_NAME.id || !p.knows(path, text.get(d.span.start..d.span.end).unwrap_or(""))
        });
    }
    (index, diagnostics, ast)
}

#[derive(Default)]
struct Config {
    dialect: String,
    dialect_paths: Vec<String>,
    metrics: metrics::Config,
    /// The client accepts `workspace/codeLens/refresh` and `workspace/semanticTokens/refresh`.
    code_lens_refresh: bool,
    semantic_tokens_refresh: bool,
    /// The server runs `SHOW_DETAILS` itself (Zed has no client-side commands; VS Code and JetBrains do).
    show_details: bool,
}

/// Commands the server runs (`workspace/executeCommand`).
pub const TOGGLE_DISPLAYED: &str = "nuclear.toggleCodeMetricsDisplayed";
pub const TOGGLE_ARROW_FUNCTIONS: &str = "nuclear.toggleCodeMetricsForArrowFunctions";
/// `nuclear.codeMetrics [uri]`: the same answer as the `nuclear/codeMetrics` request, for clients that can
/// only send standard requests (JetBrains).
pub const CODE_METRICS: &str = "nuclear.codeMetrics";
/// Client-side command of every code lens: shows the details list (the arguments carry the rows).
pub const SHOW_DETAILS: &str = "nuclear.showCodeMetricsCodeLensInfo";

pub struct Server<W: Write> {
    out: Sender<W>,
    docs: HashMap<String, Doc>,
    config: Config,
    roots: Vec<PathBuf>,
    dialect: Option<Dialect>,
    /// Dialects of projects that use another one than `nuclear.dialect` (`config.nuc` with `dialect = "fastapi"`),
    /// by name, or by `name@root` when the project has its own kind packages.
    project_dialects: HashMap<String, Option<Dialect>>,
    /// Projects of the open files, by root; rebuilt when one of their files is saved.
    projects: HashMap<PathBuf, Arc<Project>>,
    shutdown: bool,
}

impl<W: Write> Server<W> {
    pub fn new(out: W) -> Self {
        Server {
            out: Sender::new(out),
            docs: HashMap::new(),
            config: Config { dialect: "flutter".into(), ..Config::default() },
            roots: Vec::new(),
            dialect: None,
            project_dialects: HashMap::new(),
            projects: HashMap::new(),
            shutdown: false,
        }
    }

    /// Handles one message. Returns `false` when the client asked the server to exit.
    pub fn handle(&mut self, msg: Incoming) -> bool {
        if msg.method == "exit" {
            return false;
        }
        match msg.id {
            Some(id) => {
                if self.shutdown && msg.method != "shutdown" {
                    self.out.respond_error(id, -32600, "server is shutting down");
                    return true;
                }
                let result = catch_unwind(AssertUnwindSafe(|| self.request(&msg.method, &msg.params)));
                match result {
                    Ok(Ok(value)) => self.out.respond(id, value),
                    Ok(Err((code, message))) => self.out.respond_error(id, code, &message),
                    Err(_) => {
                        self.out.respond_error(id, rpc::INTERNAL_ERROR, "internal error while handling the request")
                    }
                }
            }
            None => {
                let _ = catch_unwind(AssertUnwindSafe(|| self.notification(&msg.method, &msg.params)));
            }
        }
        true
    }

    fn request(&mut self, method: &str, p: &Value) -> Result<Value, (i64, String)> {
        if method == "initialize" {
            return Ok(self.initialize(p));
        }
        if method == "shutdown" {
            self.shutdown = true;
            return Ok(Value::Null);
        }
        if method == "nuclear/transpile" {
            return self.transpile(p);
        }
        if method == "nuclear/codeMetrics" {
            return Ok(self.code_metrics(p["textDocument"]["uri"].as_str().unwrap_or_default()));
        }
        if method == "workspace/executeCommand" {
            return self.execute_command(p);
        }
        let uri = p["textDocument"]["uri"].as_str().unwrap_or_default();
        let Some(doc) = self.docs.get(uri) else {
            return match method {
                m if m.starts_with("textDocument/") => Ok(Value::Null),
                _ => Err((rpc::METHOD_NOT_FOUND, format!("unknown method {method}"))),
            };
        };
        let d = self.doc_dialect(doc);
        Ok(match method {
            "textDocument/semanticTokens/full" => features::semantic_tokens(doc),
            "textDocument/hover" => features::hover(doc, d, p),
            "textDocument/completion" => features::completion(doc, d, p),
            "textDocument/signatureHelp" => features::signature_help(doc, d, p),
            "textDocument/definition" => features::definition(doc, p),
            "textDocument/references" => features::references(doc, p),
            "textDocument/documentHighlight" => features::highlights(doc, p),
            "textDocument/prepareRename" => features::prepare_rename(doc, p),
            "textDocument/rename" => features::rename(doc, p).map_err(|e| (rpc::INVALID_PARAMS, e))?,
            "textDocument/documentSymbol" => features::document_symbols(doc),
            "textDocument/foldingRange" => features::folding_ranges(doc),
            "textDocument/codeAction" => features::code_actions(doc, p),
            "textDocument/onTypeFormatting" => features::on_type_formatting(doc, p),
            "textDocument/codeLens" => self.code_lenses(doc),
            _ => return Err((rpc::METHOD_NOT_FOUND, format!("unknown method {method}"))),
        })
    }

    fn notification(&mut self, method: &str, p: &Value) {
        match method {
            "textDocument/didOpen" => {
                let td = &p["textDocument"];
                let uri = td["uri"].as_str().unwrap_or_default().to_string();
                let text = td["text"].as_str().unwrap_or_default().to_string();
                let version = td["version"].as_i64().unwrap_or(0);
                let doc = self.make_doc(uri.clone(), version, text);
                self.docs.insert(uri.clone(), doc);
                self.publish(&uri);
            }
            // the project is built from disk: a saved file changes what the other files see
            "textDocument/didSave" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default();
                let root = self.docs.get(uri).and_then(|d| d.project.as_ref()).map(|(p, _)| p.root.clone());
                match root {
                    Some(root) => {
                        self.projects.remove(&root);
                        self.reanalyze(|d| d.project.as_ref().is_some_and(|(p, _)| p.root == root));
                    }
                    None => self.reanalyze(|d| d.uri == uri),
                }
            }
            "textDocument/didChange" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default().to_string();
                let version = p["textDocument"]["version"].as_i64().unwrap_or(0);
                let Some(doc) = self.docs.get(&uri) else { return };
                let mut text = doc.text.clone();
                for change in p["contentChanges"].as_array().into_iter().flatten() {
                    let new_text = change["text"].as_str().unwrap_or_default();
                    match change.get("range") {
                        // incremental change (the server asks for full sync, but accept both)
                        Some(range) if !range.is_null() => {
                            let lines = LineIndex::new(&text);
                            let span = lines.span_of(&text, range);
                            text.replace_range(span.start..span.end.max(span.start), new_text);
                        }
                        _ => text = new_text.to_string(),
                    }
                }
                let doc = self.make_doc(uri.clone(), version, text);
                self.docs.insert(uri.clone(), doc);
                self.publish(&uri);
            }
            "textDocument/didClose" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default().to_string();
                self.docs.remove(&uri);
                self.out.notify("textDocument/publishDiagnostics", json!({ "uri": uri, "diagnostics": [] }));
            }
            "workspace/didChangeConfiguration" => {
                let settings = &p["settings"]["nuclear"];
                if settings.is_object() {
                    self.apply_settings(settings);
                }
                self.reload();
            }
            // a dialect file changed on disk: dialects are data, reload them without restarting
            "workspace/didChangeWatchedFiles" => self.reload(),
            _ => {}
        }
    }

    fn initialize(&mut self, p: &Value) -> Value {
        for folder in p["workspaceFolders"].as_array().into_iter().flatten() {
            if let Some(path) = folder["uri"].as_str().and_then(uri_to_path) {
                self.roots.push(path);
            }
        }
        if self.roots.is_empty()
            && let Some(path) = p["rootUri"].as_str().and_then(uri_to_path)
        {
            self.roots.push(path);
        }
        let caps = &p["capabilities"]["workspace"];
        self.config.code_lens_refresh = caps["codeLens"]["refreshSupport"].as_bool().unwrap_or(false);
        self.config.semantic_tokens_refresh = caps["semanticTokens"]["refreshSupport"].as_bool().unwrap_or(false);
        self.config.show_details = p["clientInfo"]["name"].as_str().is_some_and(|n| n.starts_with("Zed"));
        let opts = &p["initializationOptions"];
        if opts.is_object() {
            self.apply_settings(opts);
        }
        self.load_dialect();
        let mut commands = vec![TOGGLE_DISPLAYED, TOGGLE_ARROW_FUNCTIONS, CODE_METRICS];
        if self.config.show_details {
            commands.push(SHOW_DETAILS);
        }

        json!({
            "capabilities": {
                "positionEncoding": "utf-16",
                "textDocumentSync": { "openClose": true, "change": 1, "save": { "includeText": false } },
                "hoverProvider": true,
                "completionProvider": { "triggerCharacters": [".", "@", "$"] },
                "signatureHelpProvider": { "triggerCharacters": ["(", ","], "retriggerCharacters": [" "] },
                "definitionProvider": true,
                "referencesProvider": true,
                "documentHighlightProvider": true,
                "documentSymbolProvider": true,
                "renameProvider": { "prepareProvider": true },
                "foldingRangeProvider": true,
                "codeActionProvider": { "codeActionKinds": ["quickfix"] },
                "documentOnTypeFormattingProvider": { "firstTriggerCharacter": "\n" },
                "codeLensProvider": { "resolveProvider": false },
                "executeCommandProvider": { "commands": commands },
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": TOKEN_TYPES, "tokenModifiers": TOKEN_MODIFIERS },
                    "full": true
                }
            },
            "serverInfo": { "name": "nuclear-lsp", "version": env!("CARGO_PKG_VERSION") }
        })
    }

    fn apply_settings(&mut self, s: &Value) {
        if let Some(d) = s["dialect"].as_str() {
            self.config.dialect = d.to_string();
        }
        if let Some(paths) = s["dialectPaths"].as_array() {
            self.config.dialect_paths = paths.iter().filter_map(Value::as_str).map(String::from).collect();
        }
        if s["codemetrics"].is_object() {
            self.config.metrics.apply(&s["codemetrics"]);
        }
    }

    fn load_dialect(&mut self) {
        self.dialect = None;
        if dialects::is_base(&self.config.dialect) {
            return;
        }
        let paths = dialects::search_paths(&self.config.dialect_paths, &self.roots);
        match dialects::load(&self.config.dialect, paths, None) {
            Ok(d) => self.dialect = Some(d),
            Err(e) => {
                let message = format!(
                    "Nuclear: dialect '{}' could not be loaded, only the base language is checked. {e}",
                    self.config.dialect
                );
                self.out.notify("window/showMessage", json!({ "type": 2, "message": message }));
            }
        }
    }

    /// Reloads the dialect and re-checks every open document.
    fn reload(&mut self) {
        self.load_dialect();
        self.project_dialects.clear();
        self.projects.clear();
        self.reanalyze(|_| true);
    }

    /// Analyzes again the open documents that match (their project is rebuilt if it was dropped).
    fn reanalyze(&mut self, which: impl Fn(&Doc) -> bool) {
        let uris: Vec<String> = self.docs.values().filter(|d| which(d)).map(|d| d.uri.clone()).collect();
        for uri in uris {
            let old = self.docs.remove(&uri).expect("document exists");
            let doc = self.make_doc(old.uri, old.version, old.text);
            self.docs.insert(uri.clone(), doc);
            self.publish(&uri);
        }
        if self.config.semantic_tokens_refresh {
            self.out.request("workspace/semanticTokens/refresh", Value::Null);
        }
        self.refresh_code_lenses();
    }

    /// The dialect of a document: its project's, or the active one.
    fn doc_dialect<'a>(&'a self, doc: &'a Doc) -> Option<&'a Dialect> {
        match &doc.project {
            Some((p, _)) => p
                .dialect
                .as_deref()
                .or_else(|| pick(&self.dialect, &self.config.dialect, &self.project_dialects, &p.manifest.dialect)),
            None => self.dialect.as_ref(),
        }
    }

    /// Analyzes a document, inside its project when it is in one (building the project the first time).
    fn make_doc(&mut self, uri: String, version: i64, text: String) -> Doc {
        let found = uri_to_path(&uri).and_then(|path| project::find(&path).map(|(root, m)| (path, root, m)));
        let Some((path, root, manifest)) = found else { return Doc::new(uri, version, text, self.dialect.as_ref()) };
        let manifest = match manifest {
            Ok(m) => m,
            Err(e) => {
                let mut doc = Doc::new(uri, version, text, self.dialect.as_ref());
                doc.diagnostics.insert(0, Diagnostic::error(codes::PROJECT, Span::default(), e));
                return doc;
            }
        };
        let name = manifest.dialect.clone();
        // a project with its own kind packages (`kinds/`) gets its own copy of the dialect
        let key = if root.join("kinds").is_dir() { format!("{name}@{}", root.display()) } else { name.clone() };
        if key != self.config.dialect && !self.project_dialects.contains_key(&key) {
            let paths = dialects::search_paths(&self.config.dialect_paths, &self.roots);
            let loaded = dialects::load(&name, paths, Some(&root));
            if let Err(e) = &loaded {
                let message =
                    format!("Nuclear: dialect '{name}' of the project {} could not be loaded. {e}", root.display());
                self.out.notify("window/showMessage", json!({ "type": 2, "message": message }));
            }
            self.project_dialects.insert(key.clone(), loaded.ok());
        }
        let Some(dialect) = pick(&self.dialect, &self.config.dialect, &self.project_dialects, &key) else {
            return Doc::new(uri, version, text, None);
        };
        let project = match self.projects.get(&root) {
            Some(p) if p.defs_lock == nuclear_transpile::project::defs_lock(&root) => p.clone(),
            _ => {
                let built = catch_unwind(AssertUnwindSafe(|| Project::build(root.clone(), manifest, dialect)));
                let Ok(p) = built else { return Doc::new(uri, version, text, Some(dialect)) };
                let p = Arc::new(p);
                self.projects.insert(root.clone(), p.clone());
                p
            }
        };
        let rel = project::stem(path.strip_prefix(&root).unwrap_or(&path));
        let own = project.dialect.clone();
        let dialect = own.as_deref().unwrap_or(dialect);
        Doc::in_project(uri, version, text, Some(dialect), Some((project, rel)))
    }

    fn refresh_code_lenses(&mut self) {
        if self.config.code_lens_refresh {
            self.out.request("workspace/codeLens/refresh", Value::Null);
        }
    }

    fn publish(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else { return };
        let mut diagnostics: Vec<Value> = doc.diagnostics.iter().map(|d| lsp_diagnostic(doc, d)).collect();
        let cfg = &self.config.metrics;
        if cfg.flag("DiagnosticsEnabled")
            && let Some(root) = doc.metrics(cfg)
        {
            // like the original: a hint per entry that has a code lens, with the same text
            for m in metrics::shown(&root, cfg) {
                diagnostics.push(json!({
                    "range": doc.lines.range(&doc.text, m.span),
                    "severity": 4,
                    "code": "42",
                    "source": "codemetrics",
                    "message": cfg.message(m.collected()),
                }));
            }
        }
        let params = json!({ "uri": uri, "version": doc.version, "diagnostics": diagnostics });
        self.out.notify("textDocument/publishDiagnostics", params);
    }

    /// `textDocument/codeLens`: "Complexity is N <level>" above every function, method, module...
    fn code_lenses(&self, doc: &Doc) -> Value {
        let cfg = &self.config.metrics;
        if !cfg.displayed || !cfg.flag("CodeLensEnabled") {
            return json!([]);
        }
        let Some(root) = doc.metrics(cfg) else { return json!([]) };
        let lenses: Vec<Value> = metrics::shown(&root, cfg)
            .into_iter()
            .map(|m| {
                json!({
                    "range": doc.lines.range(&doc.text, m.span),
                    "command": {
                        "title": cfg.message(m.collected()),
                        "command": SHOW_DETAILS,
                        "arguments": [{ "uri": doc.uri, "items": m.details(&doc.text, &doc.lines) }],
                    }
                })
            })
            .collect();
        json!(lenses)
    }

    /// `nuclear/codeMetrics`: every entry with a code lens, with its level (0 low … 3 extreme) for the colored
    /// marks of the editor, and the client-side settings (colors, decoration template, modes).
    fn code_metrics(&self, uri: &str) -> Value {
        let cfg = &self.config.metrics;
        let items: Vec<Value> = match self.docs.get(uri).and_then(|doc| Some((doc, doc.metrics(cfg)?))) {
            Some((doc, root)) => metrics::shown(&root, cfg)
                .into_iter()
                .map(|m| {
                    let c = m.collected();
                    json!({
                        "range": doc.lines.range(&doc.text, m.span),
                        "complexity": c,
                        "level": cfg.level(c),
                        "message": cfg.message(c),
                    })
                })
                .collect(),
            None => Vec::new(),
        };
        let setting = |k: &str| match k {
            "DecorationTemplate"
            | "ComplexityColorLow"
            | "ComplexityColorNormal"
            | "ComplexityColorHigh"
            | "ComplexityColorExtreme" => json!(cfg.text(k)),
            _ => json!(cfg.flag(k)),
        };
        let settings: serde_json::Map<String, Value> = [
            "DecorationModeEnabled",
            "OverviewRulerModeEnabled",
            "DecorationTemplate",
            "ComplexityColorLow",
            "ComplexityColorNormal",
            "ComplexityColorHigh",
            "ComplexityColorExtreme",
        ]
        .iter()
        .map(|k| (k.to_string(), setting(k)))
        .collect();
        json!({ "items": items, "settings": settings })
    }

    fn execute_command(&mut self, p: &Value) -> Result<Value, (i64, String)> {
        let command = p["command"].as_str().unwrap_or_default();
        match command {
            TOGGLE_DISPLAYED => {
                self.config.metrics.displayed = !self.config.metrics.displayed;
            }
            TOGGLE_ARROW_FUNCTIONS => {
                let now = self.config.metrics.flag("MetricsForArrowFunctionsToggled");
                self.config.metrics.arrow_functions = Some(!now);
            }
            CODE_METRICS => {
                let uri = p["arguments"][0].as_str().unwrap_or_default();
                return Ok(self.code_metrics(uri));
            }
            // a code lens clicked in Zed: its rows as a message
            SHOW_DETAILS if self.config.show_details => {
                let rows = p["arguments"][0]["items"].as_array().into_iter().flatten();
                let text: Vec<&str> = rows.filter_map(|r| r["label"].as_str()).collect();
                self.out.notify("window/showMessage", json!({ "type": 3, "message": text.join("\n") }));
                return Ok(Value::Null);
            }
            _ => return Err((rpc::INVALID_PARAMS, format!("unknown command {command}"))),
        }
        self.refresh_code_lenses();
        Ok(Value::Null)
    }

    /// `nuclear/transpile`: the target code of a document (used by the "Show generated code" command).
    fn transpile(&self, p: &Value) -> Result<Value, (i64, String)> {
        let uri = p["textDocument"]["uri"].as_str().unwrap_or_default();
        let doc = self.docs.get(uri).ok_or((rpc::INVALID_PARAMS, format!("document {uri} is not open")))?;
        let Some(dialect) = self.doc_dialect(doc) else {
            return Err((rpc::INVALID_PARAMS, "no dialect is active (nuclear.dialect = base)".into()));
        };
        // in a project, the file as the project build writes it (imports of the other files)
        let built = doc.project.as_ref().and_then(|(p, path)| p.built(path, &doc.text));
        let (code, errors) = match built {
            Some(f) => (f.code.clone(), f.diagnostics.iter().filter(|d| d.is_error()).count()),
            None => {
                let out = catch_unwind(AssertUnwindSafe(|| nuclear_transpile::transpile(&doc.text, dialect)))
                    .map_err(|_| (rpc::INTERNAL_ERROR, "the transpiler failed on this file".to_string()))?;
                let errors = out.diagnostics.iter().filter(|d| d.is_error()).count();
                (out.code, errors)
            }
        };
        Ok(json!({
            "code": code,
            "extension": dialect.file_extension,
            "dialect": dialect.name,
            "errors": errors,
        }))
    }
}

/// The active dialect when `name` is it, else the project dialect loaded with that name.
fn pick<'a>(
    active: &'a Option<Dialect>,
    active_name: &str,
    others: &'a HashMap<String, Option<Dialect>>,
    name: &str,
) -> Option<&'a Dialect> {
    if name == active_name { active.as_ref() } else { others.get(name).and_then(Option::as_ref) }
}

pub fn lsp_diagnostic(doc: &Doc, d: &Diagnostic) -> Value {
    let severity = match d.severity {
        Severity::Error => 1,
        Severity::Warning => 2,
    };
    let mut v = json!({
        "range": doc.lines.range(&doc.text, d.span),
        "severity": severity,
        "code": d.code.id,
        "source": "nuclear",
        "message": format!("{}\n{}", d.message, d.code.explanation),
    });
    if let Some(fix) = &d.fix {
        v["data"] = json!({ "fix": { "range": doc.lines.range(&doc.text, fix.span), "newText": fix.replacement } });
    }
    v
}

/// `file:///home/a%20b/x` → `/home/a b/x`.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&rest[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    let path = String::from_utf8(out).ok()?;
    // Windows: file:///C:/x → C:/x
    let path = if path.len() > 2 && path.as_bytes()[2] == b':' { path[1..].to_string() } else { path };
    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris() {
        assert_eq!(uri_to_path("file:///home/a%20b/x.nuc"), Some(PathBuf::from("/home/a b/x.nuc")));
        assert_eq!(uri_to_path("untitled:1"), None);
    }
}
