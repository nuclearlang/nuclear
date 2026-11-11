//! End-to-end tests: LSP messages in, LSP messages out, over the repository's real examples.

use nuclear_lsp::rpc::Incoming;
use nuclear_lsp::server::{Doc, Server};
use nuclear_lsp::{dialects, features, index, metrics};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

const URI: &str = "file:///tmp/test.nuc";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `path` in the source root of the working tree that holds it (the platform repositories hold their examples,
/// plan 006: `<repo>/dragonfly-*/examples/...`).
fn in_tree(path: &str) -> PathBuf {
    nuclear_dialect::source_roots(&repo())
        .into_iter()
        .map(|r| r.join(path))
        .find(|p| p.exists())
        .unwrap_or_else(|| repo().join(path))
}

fn example(path: &str) -> String {
    std::fs::read_to_string(in_tree(path)).unwrap()
}

#[derive(Clone, Default)]
struct Buf(Rc<RefCell<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A server with one open document.
struct Session {
    server: Server<Buf>,
    out: Buf,
    text: String,
    uri: String,
    next_id: i64,
}

impl Session {
    fn open(text: &str, dialect: &str) -> Session {
        Session::open_uri(URI, text, dialect)
    }

    fn open_uri(uri: &str, text: &str, dialect: &str) -> Session {
        let out = Buf::default();
        let mut s =
            Session { server: Server::new(out.clone()), out, text: text.to_string(), uri: uri.into(), next_id: 1 };
        s.request("initialize", json!({ "rootUri": null, "initializationOptions": { "dialect": dialect } }));
        s.notify("initialized", json!({}));
        s.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "nuclear", "version": 1, "text": text } }),
        );
        s
    }

    fn messages(&self) -> Vec<Value> {
        let raw = String::from_utf8(self.out.0.borrow().clone()).unwrap();
        raw.split("Content-Length: ")
            .filter_map(|chunk| chunk.split_once("\r\n\r\n").map(|(_, body)| serde_json::from_str(body).unwrap()))
            .collect()
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.server.handle(Incoming { id: None, method: method.into(), params });
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.server.handle(Incoming { id: Some(json!(id)), method: method.into(), params });
        let reply = self.messages().into_iter().rev().find(|m| m["id"] == json!(id)).expect("a response");
        assert!(reply.get("error").is_none(), "{method} failed: {reply}");
        reply["result"].clone()
    }

    /// Position of the `nth` occurrence (0-based) of `needle`, plus `delta` characters.
    fn pos(&self, needle: &str, nth: usize, delta: usize) -> Value {
        let at =
            self.text.match_indices(needle).nth(nth).unwrap_or_else(|| panic!("`{needle}` #{nth} not found")).0 + delta;
        let line = self.text[..at].matches('\n').count();
        let col = at - self.text[..at].rfind('\n').map_or(0, |i| i + 1);
        json!({ "line": line, "character": col })
    }

    fn at(&mut self, method: &str, needle: &str, nth: usize, delta: usize) -> Value {
        let position = self.pos(needle, nth, delta);
        let uri = self.uri.clone();
        self.request(
            method,
            json!({ "textDocument": { "uri": uri }, "position": position, "context": { "includeDeclaration": true } }),
        )
    }

    fn diagnostics(&self) -> Vec<Value> {
        self.messages()
            .into_iter()
            .rev()
            .find(|m| m["method"] == "textDocument/publishDiagnostics")
            .map(|m| m["params"]["diagnostics"].as_array().unwrap().clone())
            .unwrap_or_default()
    }

    /// Decoded semantic tokens: (text, type, modifiers).
    fn tokens(&mut self) -> Vec<(String, String, u32)> {
        let uri = self.uri.clone();
        let result = self.request("textDocument/semanticTokens/full", json!({ "textDocument": { "uri": uri } }));
        let data: Vec<u32> = result["data"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
        let lines: Vec<&str> = self.text.split('\n').collect();
        let (mut line, mut col) = (0usize, 0usize);
        let mut out = Vec::new();
        for t in data.chunks(5) {
            if t[0] > 0 {
                line += t[0] as usize;
                col = t[1] as usize;
            } else {
                col += t[1] as usize;
            }
            let utf16: Vec<u16> = lines[line].encode_utf16().collect();
            let word = String::from_utf16(&utf16[col..col + t[2] as usize]).unwrap();
            out.push((word, index::TOKEN_TYPES[t[3] as usize].to_string(), t[4]));
        }
        out
    }
}

fn token_of<'a>(tokens: &'a [(String, String, u32)], word: &str) -> &'a (String, String, u32) {
    tokens.iter().find(|t| t.0 == word).unwrap_or_else(|| panic!("no token `{word}`"))
}

#[test]
fn flutter_example_has_no_errors() {
    let s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    let errors: Vec<_> = s.diagnostics().into_iter().filter(|d| d["severity"] == 1).collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn semantic_tokens_follow_the_dialect() {
    let mut s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    let tokens = s.tokens();
    let stateless = token_of(&tokens, "stateless");
    assert_eq!(stateless.1, "keyword");
    assert!(stateless.2 & index::MOD_KIND != 0);
    assert_eq!(token_of(&tokens, "TourApp").1, "class");
    let material = token_of(&tokens, "MaterialApp");
    assert_eq!(material.1, "class");
    assert!(material.2 & index::MOD_DEFAULT_LIBRARY != 0);
    assert_eq!(token_of(&tokens, "title").1, "parameter"); // named argument
    assert_eq!(token_of(&tokens, "alive").1, "enumMember");
    assert_eq!(token_of(&tokens, "parseCharacters").1, "function");
    assert_eq!(token_of(&tokens, "setState").1, "function");
    assert_eq!(token_of(&tokens, "get").1, "modifier");
    let override_ = tokens.iter().find(|t| t.0 == "with").unwrap();
    assert_eq!(override_.1, "keyword");
}

#[test]
fn hover_shows_library_and_declarations() {
    let mut s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    let h = s.at("textDocument/hover", "Text(", 0, 1);
    let md = h["contents"]["value"].as_str().unwrap();
    assert!(md.contains("package:flutter/widgets.dart"), "{md}");
    assert!(md.contains("https://api.flutter.dev/flutter/widgets/Text-class.html"), "{md}");

    let h = s.at("textDocument/hover", "StatusBadge(status", 0, 2);
    let md = h["contents"]["value"].as_str().unwrap();
    assert!(md.contains("stateless StatusBadge(status: Status)"), "{md}");
    assert!(md.contains("match como valor") || md.contains("getter"), "doc comment expected: {md}");

    let h = s.at("textDocument/hover", "stateful", 0, 2);
    assert!(h["contents"]["value"].as_str().unwrap().contains("StatefulWidget"));

    let h = s.at("textDocument/hover", "match status", 0, 1);
    assert!(h["contents"]["value"].as_str().unwrap().contains("reserved word"));
}

#[test]
fn definition_references_and_rename() {
    let mut s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    // `load()` in initState → `load() void`
    let def = s.at("textDocument/definition", "load()", 0, 1);
    assert_eq!(def["range"]["start"], s.pos("load() void", 0, 0));

    // the named argument `status =` goes to the parameter of StatusBadge
    let def = s.at("textDocument/definition", "status = character", 0, 1);
    assert_eq!(def["range"]["start"], s.pos("status: Status)", 1, 0));

    // `controller`: field + 4 uses
    let refs = s.at("textDocument/references", "controller: AnimationController", 0, 1);
    assert_eq!(refs.as_array().unwrap().len(), 5, "{refs}");

    let position = s.pos("controller.repeat", 0, 2);
    let edit = s.request(
        "textDocument/rename",
        json!({ "textDocument": { "uri": URI }, "position": position, "newName": "spinner" }),
    );
    assert_eq!(edit["changes"][URI].as_array().unwrap().len(), 5);

    // block scope: `character` inside itemBuilder is a different variable from nothing else
    let refs = s.at("textDocument/references", "character = characters[index]", 0, 1);
    assert_eq!(refs.as_array().unwrap().len(), 4, "{refs}");
}

#[test]
fn completion_members_named_args_and_snippets() {
    let src = "Status\n  alive\n  dead\n\nstateless Badge(status: Status, label?: string)\n  build(context: BuildContext) Widget\n    Text(Status.)\n\nmain() void\n  runApp(Badge())\n\n";
    let mut s = Session::open(src, "flutter");
    let items = s.at("textDocument/completion", "Status.)", 0, 7);
    let labels: Vec<&str> = items["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert_eq!(labels, ["alive", "dead"]);

    let items = s.at("textDocument/completion", "Badge())", 0, 6);
    let labels: Vec<String> =
        items["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap().to_string()).collect();
    assert!(labels.contains(&"status =".to_string()), "{labels:?}");
    assert!(labels.contains(&"label =".to_string()), "{labels:?}");

    // a new line at the top level offers the kinds of the dialect
    let mut s = Session::open("\n", "flutter");
    let items = s.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": URI }, "position": { "line": 0, "character": 0 } }),
    );
    let labels: Vec<&str> = items["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert!(labels.contains(&"stateless") && labels.contains(&"stateful"), "{labels:?}");

    // Flutter names by prefix
    let mut s = Session::open("main() void\n  runApp(Scaf)\n", "flutter");
    let items = s.at("textDocument/completion", "Scaf)", 0, 4);
    let labels: Vec<&str> = items["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert!(labels.contains(&"Scaffold"), "{labels:?}");
}

#[test]
fn signature_help_marks_the_named_parameter() {
    let src = "stateless Badge(status: string, label?: string)\n  build(context: BuildContext) Widget\n    Text(status)\n\nmain() void\n  runApp(Badge(label = 'x', status = ))\n";
    let mut s = Session::open(src, "flutter");
    let help = s.at("textDocument/signatureHelp", "status = )", 0, 9);
    assert_eq!(help["signatures"][0]["label"], "Badge(status: string, label?: string)");
    assert_eq!(help["activeParameter"], 0);
}

#[test]
fn foreign_syntax_has_a_quick_fix() {
    let mut s = Session::open("check(a: bool, b: bool) bool\n  a && b\n", "base");
    let diags = s.diagnostics();
    let d = diags.iter().find(|d| d["code"] == "N0009").expect("N0009");
    let actions = s.request(
        "textDocument/codeAction",
        json!({ "textDocument": { "uri": URI }, "range": d["range"], "context": { "diagnostics": [d] } }),
    );
    let action = &actions[0];
    assert_eq!(action["kind"], "quickfix");
    assert_eq!(action["edit"]["changes"][URI][0]["newText"], "and");
}

#[test]
fn outline_and_folding() {
    let mut s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    let symbols = s.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": URI } }));
    let names: Vec<&str> = symbols.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names[..4], ["main", "TourApp", "Status", "Character"]);
    let home = symbols.as_array().unwrap().iter().find(|s| s["name"] == "HomePage").unwrap();
    let children: Vec<&str> =
        home["children"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(children, ["characters", "loading", "initState", "load", "build"]);

    let folds = s.request("textDocument/foldingRange", json!({ "textDocument": { "uri": URI } }));
    assert!(folds.as_array().unwrap().len() > 20);
}

#[test]
fn transpile_request_returns_dart() {
    let mut s = Session::open("main() void\n  print('hola')\n", "flutter");
    let out = s.request("nuclear/transpile", json!({ "textDocument": { "uri": URI } }));
    assert_eq!(out["extension"], "dart");
    assert!(out["code"].as_str().unwrap().contains("print('hola');"));
}

#[test]
fn unknown_dialect_falls_back_to_base() {
    let s = Session::open("main() void\n  print(1)\n", "does-not-exist");
    let warned = s.messages().iter().any(|m| m["method"] == "window/showMessage");
    assert!(warned);
}

/// Simulates typing: every prefix of every example must be handled without panics.
#[test]
fn every_prefix_of_every_example_is_safe() {
    let flutter = dialects::load("flutter", dialects::search_paths(&[], &[repo()]), None).unwrap();
    let mut files = Vec::new();
    for dir in
        ["examples/examples-flutter-dialect", "examples/spec", "examples/examples-nodejs", "examples/examples-nestjs"]
    {
        for e in std::fs::read_dir(in_tree(dir)).into_iter().flatten().flatten() {
            if e.path().extension().is_some_and(|x| x == "nuc") {
                files.push(std::fs::read_to_string(e.path()).unwrap());
            }
        }
    }
    assert!(!files.is_empty());
    for text in files {
        let cuts: Vec<usize> = (0..text.len()).step_by(37).filter(|&i| text.is_char_boundary(i)).collect();
        for cut in cuts {
            let src = &text[..cut];
            let doc = Doc::new(URI.into(), 1, src.to_string(), Some(&flutter));
            features::semantic_tokens(&doc);
            features::document_symbols(&doc);
            features::folding_ranges(&doc);
            let position = doc.lines.position(src, src.len());
            let p = json!({ "position": { "line": position.0, "character": position.1 }, "context": {} });
            features::completion(&doc, Some(&flutter), &p);
            features::hover(&doc, Some(&flutter), &p);
            features::signature_help(&doc, Some(&flutter), &p);
            features::definition(&doc, &p);
            features::references(&doc, &p);
            // without catch_unwind: a panic here fails the test
            metrics::compute(src, &doc.lines, &doc.ast, &metrics::Config::default());
        }
    }
}

#[test]
fn destructuring_bounds_and_cascades() {
    let src = "first<T: Comparable>(xs: T[]) T\n  [head, ...rest] = xs\n  { name, \"k\": value } = { name: 'a', k: 1 }\n  log = StringBuffer()\n    ..write(name)\n  head\n";
    let mut s = Session::open(src, "flutter");
    let tokens = s.tokens();
    assert_eq!(token_of(&tokens, "Comparable").1, "class");
    let refs = s.at("textDocument/references", "head,", 0, 1);
    assert_eq!(refs.as_array().unwrap().len(), 2, "{refs}");
    let def = s.at("textDocument/definition", "name)", 0, 1);
    assert_eq!(def["range"]["start"], s.pos("name,", 0, 0));
    let def = s.at("textDocument/definition", "rest]", 0, 1);
    assert_eq!(def["range"]["start"], s.pos("rest]", 0, 0));
}

#[test]
fn code_metrics_lenses_commands_and_diagnostics() {
    let src = "\
label(n: int) string
  if n < 0 and n > -10
    return 'small'
  else if n == 0
    return 'zero'
  'big'

double(n: int) int = n * 2
";
    let mut s = Session::open(src, "flutter");
    let doc = json!({ "textDocument": { "uri": URI } });
    let lenses = s.request("textDocument/codeLens", doc.clone());
    let lenses = lenses.as_array().unwrap();
    // `double` (complexity 1) is under CodeLensHiddenUnder (3)
    assert_eq!(lenses.len(), 1, "{lenses:?}");
    let lens = &lenses[0];
    assert_eq!(lens["range"]["start"], json!({ "line": 0, "character": 0 }));
    assert_eq!(lens["command"]["title"], "Complexity is 6 It's time to do something...");
    assert_eq!(lens["command"]["command"], "nuclear.showCodeMetricsCodeLensInfo");
    let items = lens["command"]["arguments"][0]["items"].as_array().unwrap();
    assert_eq!(items[0]["label"], "1 - Ln 1 Col 1 Function declaration");
    assert_eq!(items[0]["description"], "label(n: int) string");
    assert_eq!(items[2]["label"], "1 - Ln 2 Col 6 Binary expression");

    // colored marks for the editor
    let marks = s.request("nuclear/codeMetrics", doc.clone());
    assert_eq!(marks["items"][0]["level"], 1);
    assert_eq!(marks["settings"]["ComplexityColorNormal"], "#ffc208");
    let same = s.request("workspace/executeCommand", json!({ "command": "nuclear.codeMetrics", "arguments": [URI] }));
    assert_eq!(same, marks);

    // "Toggle code metrics" hides the lenses, a second toggle shows them again
    s.request("workspace/executeCommand", json!({ "command": "nuclear.toggleCodeMetricsDisplayed" }));
    assert_eq!(s.request("textDocument/codeLens", doc.clone()), json!([]));
    s.request("workspace/executeCommand", json!({ "command": "nuclear.toggleCodeMetricsDisplayed" }));
    assert_eq!(s.request("textDocument/codeLens", doc.clone()).as_array().unwrap().len(), 1);

    // settings: messages, thresholds and costs, plus the optional hint diagnostics
    s.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "nuclear": { "dialect": "flutter", "codemetrics": {
            "basics": { "CodeLensHiddenUnder": 0, "DiagnosticsEnabled": true, "ComplexityLevelLowDescription": "OK" },
            "nodeconfiguration": { "ReturnStatement": 0 }
        } } } }),
    );
    let lenses = s.request("textDocument/codeLens", doc.clone());
    let titles: Vec<&str> =
        lenses.as_array().unwrap().iter().map(|l| l["command"]["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Complexity is 4 OK", "Complexity is 1 OK"]);
    let hints: Vec<Value> = s.diagnostics().into_iter().filter(|d| d["source"] == "codemetrics").collect();
    assert_eq!(hints.len(), 2);
    assert_eq!((hints[0]["severity"].as_u64(), hints[0]["code"].as_str()), (Some(4), Some("42")));
}

#[test]
fn code_metrics_on_the_flutter_examples() {
    let mut s = Session::open(&example("examples/examples-flutter-dialect/widgets_tour.nuc"), "flutter");
    let lenses = s.request("textDocument/codeLens", json!({ "textDocument": { "uri": URI } }));
    let lenses = lenses.as_array().unwrap();
    assert!(!lenses.is_empty());
    for l in lenses {
        let title = l["command"]["title"].as_str().unwrap();
        assert!(title.starts_with("Complexity is "), "{title}");
        assert!(!l["command"]["arguments"][0]["items"].as_array().unwrap().is_empty());
    }
}

// ---------------------------------------------------------------- type definitions (defs/dart)

fn labels(items: &Value) -> Vec<String> {
    items["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap().to_string()).collect()
}

fn item<'a>(items: &'a Value, label: &str) -> &'a Value {
    items["items"].as_array().unwrap().iter().find(|i| i["label"] == label).unwrap_or_else(|| panic!("no `{label}`"))
}

#[test]
fn framework_libs_example_has_no_diagnostics() {
    let s = Session::open(&example("examples/examples-flutter-dialect/framework_libs.nuc"), "flutter");
    assert!(s.diagnostics().is_empty(), "{:?}", s.diagnostics());
}

#[test]
fn hover_shows_the_type_definitions() {
    let src = "Api\n  dio: Dio = Dio()\n\nstateless Hello\n  build(context: BuildContext) Widget\n    Text('a', textAlign = TextAlign.center)\n";
    let mut s = Session::open(src, "flutter");
    let md = s.at("textDocument/hover", "Text(", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("```nuclear\nText: StatelessWidget"), "{md}");
    assert!(md.contains("  Text(\n    data: string,"), "{md}");
    assert!(md.contains("`flutter@3.47.7`"), "{md}");
    assert!(md.contains("A run of text with a single style."), "{md}");
    assert!(md.contains("https://api.flutter.dev/flutter/widgets/Text-class.html"), "{md}");

    let md = s.at("textDocument/hover", "Dio()", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`dio@5.11.1`") && md.contains("package:dio/dio.dart"), "{md}");

    // a member of an external class
    let md = s.at("textDocument/hover", "center", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("TextAlign.center"), "{md}");
}

#[test]
fn completion_of_external_members() {
    let src = "\
Api
  dio: Dio = Dio()

  load(url: string) void
    a = Colors.
    b = TextAlign.
    c = dio.
    other = Dio()
    d = other.
";
    let mut s = Session::open(src, "flutter");
    let colors = s.at("textDocument/completion", "Colors.", 0, 7);
    let names = labels(&colors);
    assert!(names.contains(&"red".to_string()) && names.contains(&"transparent".to_string()), "{names:?}");
    assert!(item(&colors, "red")["detail"].as_str().unwrap().contains("static red: MaterialColor"));

    let names = labels(&s.at("textDocument/completion", "TextAlign.", 0, 10));
    assert!(names.contains(&"center".to_string()) && names.contains(&"justify".to_string()), "{names:?}");

    for needle in ["dio.", "other."] {
        let dio = s.at("textDocument/completion", needle, 0, needle.len());
        let names = labels(&dio);
        assert!(names.contains(&"get".to_string()) && names.contains(&"options".to_string()), "{needle}: {names:?}");
        assert!(item(&dio, "get")["detail"].as_str().unwrap().starts_with("get<T>("), "{dio}");
    }

    // a named constructor
    let mut s = Session::open("main() void\n  runApp(Text.)\n", "flutter");
    let text = s.at("textDocument/completion", "Text.)", 0, 5);
    assert!(labels(&text).contains(&"rich".to_string()));
}

#[test]
fn completion_of_external_named_arguments() {
    let src = "stateless Hello\n  build(context: BuildContext) Widget\n    Text('a', maxLines = 2, )\n";
    let mut s = Session::open(src, "flutter");
    let items = s.at("textDocument/completion", "2, )", 0, 3);
    let names = labels(&items);
    assert!(names.contains(&"style =".to_string()) && names.contains(&"textAlign =".to_string()), "{names:?}");
    assert!(!names.contains(&"maxLines =".to_string()), "already written: {names:?}");
    assert_eq!(item(&items, "style =")["insertText"], "style = ");

    // required named arguments first
    let mut s =
        Session::open("main() void\n  runApp(ListTile(onTap = null, ))\n  BlocBuilder<int, int>()\n", "flutter");
    let items = s.at("textDocument/completion", ">()", 0, 2);
    let builder = item(&items, "builder =");
    assert_eq!(builder["preselect"], true);
    assert!(builder["sortText"].as_str().unwrap().starts_with("00"));
}

#[test]
fn signature_help_of_external_calls() {
    let src =
        "stateless Hello\n  build(context: BuildContext) Widget\n    Scaffold(appBar = null, body = Text('a', ))\n";
    let mut s = Session::open(src, "flutter");
    let help = s.at("textDocument/signatureHelp", "null, body", 0, 0);
    let label = help["signatures"][0]["label"].as_str().unwrap();
    assert!(label.starts_with("Scaffold({key?: Key, appBar?: PreferredSizeWidget"), "{label}");
    let active = help["activeParameter"].as_u64().unwrap() as usize;
    let range = help["signatures"][0]["parameters"][active]["label"].as_array().unwrap();
    let (a, b) = (range[0].as_u64().unwrap() as usize, range[1].as_u64().unwrap() as usize);
    assert_eq!(&label[a..b], "appBar?: PreferredSizeWidget");

    // positional first: `Text(data: string, {...})`, the cursor after `'a', ` is in the named part
    let help = s.at("textDocument/signatureHelp", "'a', )", 0, 5);
    let label = help["signatures"][0]["label"].as_str().unwrap();
    assert!(label.starts_with("Text(data: string, {key?: Key"), "{label}");
    assert!(help["activeParameter"].is_null(), "{help}");
}

#[test]
fn inherited_members_of_external_bases() {
    let src = "\
CounterCubit: Cubit<int>
  ()
    super(0)

  increment() void
    emit(state + 1)
    n = state.

stateful Counter
  
";
    let mut s = Session::open(src, "flutter");
    // `state` comes from BlocBase<State> with State = int
    let names = labels(&s.at("textDocument/completion", "state.", 0, 6));
    assert!(names.contains(&"isEven".to_string()), "{names:?}");
    let names = labels(&s.at("textDocument/completion", "emit(", 0, 1));
    assert!(names.contains(&"emit".to_string()) && names.contains(&"state".to_string()), "{names:?}");

    // inside a stateful kind: methods of State to override, with their Nuclear signature
    let items = s.at("textDocument/completion", "Counter\n  ", 0, 10);
    let init = item(&items, "initState");
    assert_eq!(init["insertText"], "initState() void\n  $0");
    assert!(labels(&items).contains(&"build".to_string()));
    assert!(labels(&items).contains(&"dispose".to_string()));
}

#[test]
fn free_names_come_from_the_type_definitions() {
    let mut s = Session::open("main() void\n  runApp(Elevated)\n", "flutter");
    let items = s.at("textDocument/completion", "Elevated)", 0, 8);
    let button = item(&items, "ElevatedButton");
    assert!(button["detail"].as_str().unwrap().contains("flutter@3.47.7"), "{button}");
    assert!(button["documentation"]["value"].as_str().is_some());
}

#[test]
fn deprecated_names_are_marked() {
    let mut s = Session::open("main() void\n  runApp(ButtonBar())\n", "flutter");
    let tokens = s.tokens();
    assert!(token_of(&tokens, "ButtonBar").2 & index::MOD_DEPRECATED != 0);
    assert!(token_of(&tokens, "runApp").2 & index::MOD_DEPRECATED == 0);
    let items = s.at("textDocument/completion", "ButtonBar(", 0, 9);
    assert_eq!(item(&items, "ButtonBar")["tags"], json!([1]));
}

// ---------------------------------------------------------------- projects

const PROJECT: &str = "examples/examples-flutter-dialect/clean-flutter-project";

/// A file of the Dragonfly example project (on disk or not), opened with `text`.
fn project_file(rel: &str, text: &str) -> Session {
    let path = repo()
        .join(PROJECT)
        .join(rel)
        .canonicalize()
        .unwrap_or_else(|_| in_tree(PROJECT).canonicalize().unwrap().join(rel));
    Session::open_uri(&nuclear_lsp::project::path_to_uri(&path), text, "flutter")
}

#[test]
fn a_project_file_sees_the_other_files() {
    let rel = "components/characters/domain/usecases/get_page_usecase.nuc";
    let text = example(&format!("{PROJECT}/{rel}"));
    let mut s = project_file(rel, &text);
    let errors: Vec<_> = s.diagnostics().into_iter().filter(|d| d["severity"] == 1).collect();
    assert!(errors.is_empty(), "{errors:?}");

    let md = s.at("textDocument/hover", "CharacterRepository", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("Declared in `components/characters/data/character_repository.nuc`"), "{md}");
    let def = s.at("textDocument/definition", "CharacterRepository", 0, 1);
    assert!(def["uri"].as_str().unwrap().ends_with("/data/character_repository.nuc"), "{def}");

    // the file as the project writes it: imports of the other files
    let out = s.request("nuclear/transpile", json!({ "textDocument": { "uri": s.uri.clone() } }));
    assert!(out["code"].as_str().unwrap().contains("import "), "{out}");

    // a decorator the dialect reads
    let md = s.at("textDocument/hover", "@requires", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("internet = true") && md.contains("Arguments: `internet`, `session`."), "{md}");
}

#[test]
fn unsaved_project_files_know_the_other_names() {
    // a name of a generated file (`[project.files] names`) and a type of another file
    let src = "load() void\n  startDragonfly()\n  c: Character? = null\n";
    let s = project_file("components/characters/unsaved.nuc", src);
    let undefined: Vec<_> = s.diagnostics().into_iter().filter(|d| d["code"] == "N0106").collect();
    assert!(undefined.is_empty(), "{undefined:?}");

    // what test/ declares is only seen from test/
    let src = "x() void\n  character('a', 'alive')\n";
    let s = project_file("components/characters/unsaved.nuc", src);
    assert!(s.diagnostics().iter().any(|d| d["code"] == "N0106"), "{:?}", s.diagnostics());
    let s = project_file("test/unsaved_test.nuc", src);
    assert!(!s.diagnostics().iter().any(|d| d["code"] == "N0106"), "{:?}", s.diagnostics());
}

#[test]
fn completion_of_declarations_of_other_files() {
    let rel = "components/characters/unsaved.nuc";
    // members of a module of another file, by name and by the type of a value
    let mut s = project_file(rel, "x(repo: CharacterRepository) void\n  CharacterRepository.\n");
    assert!(labels(&s.at("textDocument/completion", "CharacterRepository.", 0, 20)).contains(&"getPage".to_string()));
    let mut s = project_file(rel, "x(repo: CharacterRepository) void\n  repo.\n");
    assert!(labels(&s.at("textDocument/completion", "repo.\n", 0, 5)).contains(&"getOne".to_string()));
    // named arguments of its constructor
    let mut s = project_file(rel, "x() void\n  GetPageUseCase()\n");
    let items = labels(&s.at("textDocument/completion", "GetPageUseCase()", 0, 15));
    assert!(items.contains(&"repository =".to_string()), "{items:?}");
}

#[test]
fn a_project_folder_gives_its_kind() {
    let src = "Planet\n  name: string\n";
    let mut s = project_file("components/characters/domain/models/planet.nuc", src);
    let tokens = s.tokens();
    assert_eq!(token_of(&tokens, "Planet").1, "class");
    let md = s.at("textDocument/hover", "Planet", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`model` declaration"), "{md}");
}

#[test]
fn an_invalid_manifest_is_reported() {
    let dir = std::env::temp_dir().join(format!("nuclear-lsp-manifest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("nuclear.toml"), "[project]\nname = \"Bad Name\"\n").unwrap();
    let s = Session::open_uri(&nuclear_lsp::project::path_to_uri(&dir.join("a.nuc")), "main() void\n", "flutter");
    std::fs::remove_dir_all(&dir).ok();
    assert!(s.diagnostics().iter().any(|d| d["code"] == "N0303"), "{:?}", s.diagnostics());
}

#[test]
fn decorators_and_result_of_the_dialect() {
    let mut s = Session::open("@\nload() Result<string, int>\n  Ok(1)\n", "flutter");
    let names = labels(&s.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": URI }, "position": { "line": 0, "character": 1 } }),
    ));
    assert!(names.contains(&"repository".to_string()) && names.contains(&"get".to_string()), "{names:?}");
    let md = s.at("textDocument/hover", "Result", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`Result<E, T>`") && md.contains("error type goes **first**"), "{md}");
}

#[test]
fn arguments_of_a_decorator() {
    let src = "@screen(route = \"/\", loggedIn = { forward: X }, \n  )\nstateless A\n";
    let mut s = Session::open(src, "flutter");
    let complete = |s: &mut Session, line: u32, character: u32| {
        labels(&s.request(
            "textDocument/completion",
            json!({ "textDocument": { "uri": URI }, "position": { "line": line, "character": character } }),
        ))
    };
    let top = complete(&mut s, 1, 2);
    assert!(top.contains(&"providers".to_string()) && top.contains(&"acl".to_string()), "{top:?}");
    assert!(!top.contains(&"forward".to_string()), "{top:?}");
    let fields = complete(&mut s, 0, 36);
    assert!(fields.contains(&"forward".to_string()) && !fields.contains(&"route".to_string()), "{fields:?}");
    let decorators = complete(&mut s, 0, 1);
    assert!(decorators.contains(&"screen".to_string()), "{decorators:?}");
    assert!(!decorators.iter().any(|d| d.contains('.')), "{decorators:?}");
    let md = s.at("textDocument/hover", "forward", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`loggedIn.forward`") && md.contains("Required"), "{md}");
    let md = s.at("textDocument/hover", "screen", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`route`") && md.contains("`providers = [Manager]`"), "{md}");
}

#[test]
fn every_file_of_the_dragonfly_example_is_clean() {
    let root = in_tree(PROJECT);
    let files = nuclear_transpile::project::sources(&root, &root.join("build/app"));
    assert!(files.len() > 20);
    for rel in files {
        let rel = rel.to_string_lossy().to_string();
        let s = project_file(&rel, &example(&format!("{PROJECT}/{rel}")));
        let errors: Vec<_> = s.diagnostics().into_iter().filter(|d| d["severity"] == 1).collect();
        assert!(errors.is_empty(), "{rel}: {errors:?}");
    }
}

/// The packages a project adds (`dragonfly flutter add`, listed in `build/defs.lock`) bring their type
/// definitions: imports, hover and completion; a package whose definitions are missing is a warning on
/// config.nuc.
/// A backend project (`config.nuc` with `dragonfly(platform = "backend", dialect = "fastapi")`) is checked with
/// its own dialect, whatever `nuclear.dialect` says, and gets its project card.
#[test]
fn a_dragonfly_project_uses_the_dialect_of_its_config() {
    let root = std::env::temp_dir().join(format!("nuclear-lsp-backend-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    copy_tree(&in_tree("examples/examples-backend/tasks_api"), &root);
    let config = std::fs::read_to_string(root.join("config.nuc")).unwrap();
    std::fs::write(root.join("config.nuc"), config.replace("dialect = \"nestjs\"", "dialect = \"fastapi\"")).unwrap();
    let file = root.join("components/tasks/api/tasks_controller.nuc");
    let text = std::fs::read_to_string(&file).unwrap();
    let mut s = Session::open_uri(&nuclear_lsp::project::path_to_uri(&file), &text, "flutter");
    let errors: Vec<_> = s.diagnostics().into_iter().filter(|d| d["severity"] == 1).collect();
    assert!(errors.is_empty(), "{errors:?}");
    let md = s.at("textDocument/hover", "controller TasksController", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("`fastapi` dialect · native of the `backend` platform"), "{md}");
    let card: Value = serde_json::from_str(&std::fs::read_to_string(root.join(".nuclear/project.json")).unwrap()).unwrap();
    assert_eq!(card["platform"], "backend");
    assert_eq!(card["dialect"], "fastapi");
    assert_eq!(card["language"], "python");
    assert_eq!(card["kinds"]["controller"], "platform:backend");
    let _ = std::fs::remove_dir_all(&root);
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[test]
fn project_packages_bring_their_definitions() {
    let root = std::env::temp_dir().join(format!("nuclear-lsp-project-defs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let pkg = root.join("defs/dart/vault/1.0.0");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::create_dir_all(root.join("build")).unwrap();
    let lib = "package:vault/vault.dart";
    std::fs::write(
        pkg.join("package.toml"),
        format!("[package]\nname = \"vault\"\nversion = \"1.0.0\"\nlanguage = \"dart\"\ndepends = []\nlibraries = [\"{lib}\"]\n"),
    )
    .unwrap();
    let index = json!({ "format": 1, "libraries": [lib], "reexports": {}, "symbols": { "Vault": 0 },
        "classes": { "Vault": { "supers": ["Object"], "members": ["open"] } } });
    std::fs::write(pkg.join("index.json"), index.to_string()).unwrap();
    let api = json!({ "format": 1, "declarations": [{ "name": "Vault", "library": lib, "kind": "class",
        "doc": "A safe place for secrets.", "constructors": [{ "name": "", "params": [] }],
        "members": [{ "name": "open", "kind": "method", "static": true,
            "params": [{ "name": "name", "type": "String" }], "returns": "void", "doc": "Opens a vault." }] }] });
    std::fs::write(pkg.join("api.json"), api.to_string()).unwrap();
    std::fs::write(root.join("build/defs.lock"), "vault@1.0.0\nghost@0.0.1\n").unwrap();
    let config = "app(\n  name = \"demo\",\n)\n\ndependencies(\n  vault = \"^1.0.0\",\n)\n";
    std::fs::write(root.join("config.nuc"), config).unwrap();
    let main = "main() void\n  Vault.open(\"x\")\n";
    std::fs::write(root.join("main.nuc"), main).unwrap();

    let uri = nuclear_lsp::project::path_to_uri(&root.join("main.nuc"));
    let mut s = Session::open_uri(&uri, main, "flutter");
    let errors: Vec<_> = s.diagnostics().into_iter().filter(|d| d["severity"] == 1).collect();
    assert!(errors.is_empty(), "{errors:?}");
    let md = s.at("textDocument/hover", "Vault", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("A safe place for secrets."), "{md}");
    let members = labels(&s.at("textDocument/completion", "open", 0, 0));
    assert!(members.contains(&"open".to_string()), "{members:?}");
    let out = s.request("nuclear/transpile", json!({ "textDocument": { "uri": uri.clone() } }));
    assert!(out["code"].as_str().unwrap().contains("import 'package:vault/vault.dart';"), "{out}");

    let s = Session::open_uri(&nuclear_lsp::project::path_to_uri(&root.join("config.nuc")), config, "flutter");
    let warnings: Vec<String> = s.diagnostics().iter().map(|d| d["message"].as_str().unwrap().to_string()).collect();
    assert!(warnings.iter().any(|m| m.contains("ghost@0.0.1") && m.contains("dragonfly build")), "{warnings:?}");
    assert!(!warnings.iter().any(|m| m.contains("vault")), "{warnings:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn hover_explains_kinds_words_and_doc_comments() {
    let src = "// Price with tax.\n * @param amount before tax\n * @return the price\n * @deprecated use total()\n * @example\n *   withTax(1.0)\nwithTax(amount: double) double\n  amount * 1.16\n\nstateless Badge(status: string)\n  build(context: BuildContext) Widget\n    Text(status)\n\nmain() void\n  print(withTax(amount = ))\n";
    let mut s = Session::open(src, "flutter");
    // a kind: its editor documentation, with an example
    let md = s.at("textDocument/hover", "stateless", 0, 2)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.starts_with("**`stateless`** — kind · `flutter` dialect"), "{md}");
    assert!(md.contains("```nuclear") && md.contains("StatelessWidget"), "{md}");
    // a doc comment: tags rendered as sections, the example as code
    let md =
        s.at("textDocument/hover", "withTax(amount: double)", 0, 2)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("**Deprecated:** use total()"), "{md}");
    assert!(md.contains("- `amount` — before tax") && md.contains("**Returns** the price"), "{md}");
    assert!(md.contains("```nuclear\nwithTax(1.0)\n```"), "{md}");
    // the parameter's text next to the argument being written
    let help = s.at("textDocument/signatureHelp", "amount = )", 0, 9);
    assert_eq!(help["signatures"][0]["parameters"][0]["documentation"], "before tax");
    // a reserved word and a base type
    let md = s.at("textDocument/hover", "void", 0, 1)["contents"]["value"].as_str().unwrap().to_string();
    assert!(md.contains("In the `flutter` dialect: `void`"), "{md}");
}

#[test]
fn enter_on_an_empty_star_line_ends_the_comment() {
    let src = "  // Price.\n   * @return x\n   *\n  \nx = 1\n";
    let mut s = Session::open(src, "flutter");
    let uri = s.uri.clone();
    let edits = s.request(
        "textDocument/onTypeFormatting",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": 2 }, "ch": "\n", "options": {} }),
    );
    assert_eq!(edits[0]["range"]["start"]["line"], 2);
    assert_eq!(edits[0]["newText"], "");
    // on a line with text, nothing changes
    let edits = s.request(
        "textDocument/onTypeFormatting",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 3 }, "ch": "\n", "options": {} }),
    );
    assert_eq!(edits, json!([]));
}

#[test]
fn zed_runs_the_code_lens_command_on_the_server() {
    let out = Buf::default();
    let mut s = Session { server: Server::new(out.clone()), out, text: String::new(), uri: URI.into(), next_id: 1 };
    let init =
        s.request("initialize", json!({ "rootUri": null, "clientInfo": { "name": "Zed", "version": "1.25.3" } }));
    let commands = &init["capabilities"]["executeCommandProvider"]["commands"];
    assert!(commands.as_array().unwrap().contains(&json!("nuclear.showCodeMetricsCodeLensInfo")));
    let args = json!([{ "uri": URI, "items": [{ "label": "1 - Ln 1 Col 1 Function declaration" }, { "label": "2 - Ln 2 Col 3 If statement" }] }]);
    s.request(
        "workspace/executeCommand",
        json!({ "command": "nuclear.showCodeMetricsCodeLensInfo", "arguments": args }),
    );
    let shown = s.messages().into_iter().find(|m| m["method"] == "window/showMessage").expect("a message");
    assert_eq!(shown["params"]["message"], "1 - Ln 1 Col 1 Function declaration\n2 - Ln 2 Col 3 If statement");

    // VS Code registers that command itself: the server must not claim it
    let mut vscode = Session::open("", "flutter");
    let init = vscode.request("initialize", json!({ "rootUri": null, "clientInfo": { "name": "Visual Studio Code" } }));
    assert!(
        !init["capabilities"]["executeCommandProvider"]["commands"]
            .as_array()
            .unwrap()
            .contains(&json!("nuclear.showCodeMetricsCodeLensInfo"))
    );
}
