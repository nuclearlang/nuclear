//! Loading, inheritance and validation of dialect files.

use nuclear_dialect::model::KindSource;
use nuclear_dialect::Loader;
use std::path::PathBuf;

/// A temporary dialects directory: `files` is (relative path, content).
fn tmp(test: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nuclear-dialect-test-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, content) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    dir
}

/// The `dialects/` folders of the working tree (the `nuclear` repository and the platform repositories inside it).
fn repo_dialects() -> Vec<PathBuf> {
    nuclear_dialect::dialect_dirs(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")))
}

/// The platform dialects these tests read live in the platform repositories (plan 006): without one of them checked
/// out inside this working tree, the test is skipped with a note.
fn without_platforms(test: &str) -> bool {
    let l = Loader::new(repo_dialects());
    let missing: Vec<&str> = ["flutter", "fastapi", "nestjs", "react"].into_iter().filter(|d| l.load(d).0.is_none()).collect();
    if !missing.is_empty() {
        eprintln!("skipped {test}: the dialects {missing:?} are not in this working tree (their platform repositories are not checked out)");
    }
    !missing.is_empty()
}

fn codes(dirs: Vec<PathBuf>, name: &str) -> Vec<&'static str> {
    let (_, report) = Loader::new(dirs).load(name);
    report.diags.iter().map(|(_, d)| d.code.id).collect()
}

#[test]
fn repo_dialects_are_valid_and_complete() {
    if without_platforms("repo_dialects_are_valid_and_complete") {
        return;
    }
    let loader = Loader::new(repo_dialects());
    for name in ["typescript", "nodejs", "nestjs", "dart", "flutter"] {
        let (d, report) = loader.load(name);
        let errors: Vec<_> = report.diags.iter().filter(|(_, d)| d.is_error()).collect();
        assert!(errors.is_empty(), "{name}: {errors:#?}");
        let d = d.unwrap();
        assert!(d.missing_syntax().is_empty(), "{name} misses {:?}", d.missing_syntax());
    }
}

#[test]
fn inheritance_chain() {
    if without_platforms("inheritance_chain") {
        return;
    }
    let (d, _) = Loader::new(repo_dialects()).load("nestjs");
    let d = d.unwrap();
    assert_eq!(d.chain, ["nestjs", "nodejs", "typescript"]);
    assert!(d.syntax.contains_key("class")); // from typescript
    assert!(d.auto_imports.contains_key("readFile")); // from nodejs
    assert!(d.kinds.contains_key("controller")); // its own
}

#[test]
fn files_of_a_dialect_are_merged() {
    if without_platforms("files_of_a_dialect_are_merged") {
        return;
    }
    let dir = tmp(
        "merge",
        &[
            ("app/dialect.toml", "[dialect]\nname = \"app\"\nextends = \"flutter\"\n"),
            ("app/app_shortcuts.toml", "[imports]\nui = \"package:app_ui/app_ui.dart\"\n"),
        ],
    );
    let (d, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("app");
    assert!(report.diags.is_empty(), "{:#?}", report.diags);
    let d = d.unwrap();
    assert!(d.imports.contains_key("ui"));
    assert!(d.kinds.contains_key("stateless"));
}

#[test]
fn duplicate_shortcut_in_two_files() {
    if without_platforms("duplicate_shortcut_in_two_files") {
        return;
    }
    let dir = tmp(
        "dup",
        &[
            (
                "app/dialect.toml",
                "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[imports]\nui = \"package:a/a.dart\"\n",
            ),
            ("app/more.toml", "[imports]\nui = \"package:b/b.dart\"\n"),
        ],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").contains(&"N0204"));
}

#[test]
fn child_cannot_redefine_parent_shortcut_without_replaces() {
    if without_platforms("child_cannot_redefine_parent_shortcut_without_replaces") {
        return;
    }
    let base = "[dialect]\nname = \"app\"\nextends = \"flutter\"\n";
    let dir = tmp("redef", &[("app/dialect.toml", &format!("{base}[imports]\nmaterial = \"package:x/x.dart\"\n"))]);
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").contains(&"N0204"));
    let dir = tmp(
        "replace",
        &[(
            "app/dialect.toml",
            "[dialect]\nname = \"app\"\nextends = \"flutter\"\nreplaces = [\"imports.material\"]\n[imports]\nmaterial = \"package:x/x.dart\"\n",
        )],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").is_empty());
}

#[test]
fn symbol_auto_imported_from_two_modules() {
    if without_platforms("symbol_auto_imported_from_two_modules") {
        return;
    }
    let dir = tmp(
        "auto",
        &[(
            "app/dialect.toml",
            "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[auto_imports]\n\"package:ui/ui.dart\" = [\"Text\"]\n",
        )],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").contains(&"N0204"));
}

#[test]
fn header_words_share_a_namespace() {
    if without_platforms("header_words_share_a_namespace") {
        return;
    }
    let dir = tmp(
        "header",
        &[("app/dialect.toml", "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[modifiers]\nstateless = \"x \"\n")],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").contains(&"N0204"));
}

#[test]
fn shortcut_named_like_a_base_keyword() {
    if without_platforms("shortcut_named_like_a_base_keyword") {
        return;
    }
    let dir = tmp(
        "kw",
        &[(
            "app/dialect.toml",
            "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[imports]\nmatch = \"package:m/m.dart\"\n",
        )],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "app").contains(&"N0212"));
}

#[test]
fn similar_shortcut_is_a_warning() {
    if without_platforms("similar_shortcut_is_a_warning") {
        return;
    }
    let dir = tmp(
        "similar",
        &[(
            "app/dialect.toml",
            "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[imports]\nmateral = \"package:m/m.dart\"\n",
        )],
    );
    let (d, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("app");
    assert!(d.is_some());
    assert!(report.diags.iter().any(|(_, d)| d.code.id == "N0205" && !d.is_error()));
}

#[test]
fn bad_template_and_unknown_keys() {
    let dir = tmp(
        "bad",
        &[(
            "x/dialect.toml",
            "[dialect]\nname = \"x\"\nextends = \"typescript\"\n[syntax]\nwhle = \"x\"\nclass = \"{{#members}}\"\n[optons]\na = 1\n",
        )],
    );
    let c = codes([vec![dir], repo_dialects()].concat(), "x");
    assert_eq!(c.iter().filter(|c| **c == "N0206").count(), 3, "{c:?}");
}

#[test]
fn unknown_template_variable_is_a_warning() {
    let dir = tmp(
        "var",
        &[(
            "x/dialect.toml",
            "[dialect]\nname = \"x\"\nextends = \"typescript\"\n[syntax]\nwhile = \"while ({{cnd}}) {}\"\n",
        )],
    );
    assert!(codes([vec![dir], repo_dialects()].concat(), "x").contains(&"N0210"));
}

#[test]
fn missing_dialect_and_cycles() {
    assert_eq!(codes(repo_dialects(), "flutr"), ["N0207"]);
    let dir = tmp(
        "cycle",
        &[
            ("a.toml", "[dialect]\nname = \"a\"\nextends = \"b\"\n"),
            ("b.toml", "[dialect]\nname = \"b\"\nextends = \"a\"\n"),
        ],
    );
    assert!(codes(vec![dir], "a").contains(&"N0207"));
}

#[test]
fn template_from_a_file() {
    if without_platforms("template_from_a_file") {
        return;
    }
    let dir = tmp(
        "tplfile",
        &[
            (
                "app/dialect.toml",
                "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[kinds.screen]\ntemplate = { file = \"templates/screen.tpl\" }\n",
            ),
            ("app/templates/screen.tpl", "class {{name}} {\n  {{members}}\n}\n"),
        ],
    );
    let (d, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("app");
    assert!(report.diags.is_empty(), "{:#?}", report.diags);
    let d = d.unwrap();
    let ctx = nuclear_dialect::template::Ctx::new().set("name", "Home").set("members", "x;").into_value();
    // the file's final newline is not part of the template
    assert_eq!(d.kinds["screen"].value.template.render(&ctx), "class Home {\n  x;\n}");
}

#[test]
fn template_file_errors() {
    if without_platforms("template_file_errors") {
        return;
    }
    let dir = tmp(
        "tplmissing",
        &[(
            "app/dialect.toml",
            "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[kinds.screen]\ntemplate = { file = \"templates/nope.tpl\" }\n",
        )],
    );
    assert_eq!(codes([vec![dir], repo_dialects()].concat(), "app"), ["N0206"]);
    // a template error points inside the template file
    let dir = tmp(
        "tplbad",
        &[
            (
                "app/dialect.toml",
                "[dialect]\nname = \"app\"\nextends = \"flutter\"\n[kinds.screen]\ntemplate = { file = \"t.tpl\" }\n",
            ),
            ("app/t.tpl", "class {{name}} {\n  {{#members}}\n}\n"),
        ],
    );
    let (_, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("app");
    let (file, d) = &report.diags[0];
    assert!(report.files[*file].name.ends_with("t.tpl"), "{}", report.files[*file].name);
    assert_eq!(d.span.start, 19);
}

#[test]
fn type_definitions_are_loaded_and_inherited() {
    if without_platforms("type_definitions_are_loaded_and_inherited") {
        return;
    }
    let (d, _) = Loader::new(repo_dialects()).load("flutter");
    let d = d.unwrap();
    // flutter's own definitions and dart's (inherited)
    assert!(d.defs["CustomPainter"].members.contains(&"paint".to_string()));
    assert!(d.defs["Object"].members.contains(&"toString".to_string()));
    // the package's index gives the auto-imports: the first package that exports a name keeps it
    assert_eq!(d.auto_imports["Uint8List"].value, "dart:typed_data");
    assert_eq!(d.auto_imports["Scaffold"].value, "package:flutter/material.dart");
    assert_eq!(d.auto_imports["Cubit"].value, "package:flutter_bloc/flutter_bloc.dart");
    assert_eq!(d.auto_imports["Dio"].value, "package:dio/dio.dart");
    assert_eq!(d.auto_imports["GetIt"].value, "package:get_it/get_it.dart");
    assert!(!d.auto_imports.contains_key("String")); // dart:core is implicit
    assert!(d.reexports["package:flutter_bloc/flutter_bloc.dart"].contains("package:bloc/bloc.dart"));
    // parent packages first, then each package before its dependencies
    let ids: Vec<String> = d.typedefs.packages.iter().map(|p| p.id()).collect();
    assert_eq!(
        ids,
        [
            "dart@3.13.5",
            "flutter@3.47.7",
            "dragonfly@0.1.0",
            "flutter_bloc@9.1.1",
            "bloc@9.2.1",
            "dio@5.11.1",
            "get_it@9.3.0",
            "go_router@18.1.1",
            "connectivity_plus@7.3.2",
            "flutter_secure_storage@11.2.0",
            "json_annotation@4.12.0",
            // the definitions of the plugins (only their index; a project that does not enable them gets N0304)
            "genkit@1.0.1",
            "schemantic@1.0.0",
            "dragonfly_genkit@0.1.0",
        ]
    );
    // the WebSocket of dragonfly replaces the one of dart:io
    assert_eq!(d.auto_imports["BlocBuilder"].value, "package:flutter_bloc/flutter_bloc.dart");
    assert_eq!(d.auto_imports["WebSocket"].value, "package:dragonfly/dragonfly.dart");
}

#[test]
fn type_definitions_give_the_full_api() {
    if without_platforms("type_definitions_give_the_full_api") {
        return;
    }
    let (d, _) = Loader::new(repo_dialects()).load("flutter");
    let defs = &d.unwrap().typedefs;
    let (package, text) = defs.get("Text").unwrap();
    assert_eq!(package.id(), "flutter@3.47.7");
    assert_eq!(text.library.as_deref(), Some("package:flutter/widgets.dart"));
    let ctor = &text.constructors[0];
    assert!(ctor.is_const);
    assert_eq!((ctor.params[0].name.as_str(), ctor.params[0].ty.as_str()), ("data", "string"));
    let style = ctor.params.iter().find(|p| p.name == "style").unwrap();
    assert!(style.named && style.optional);
    assert_eq!(nuclear_dialect::defs::render_param(style), "style?: TextStyle");
    assert!(text.constructors.iter().any(|c| c.name == "rich"));
    // members: own first, then inherited through the supertypes
    let members = defs.members("Text");
    assert!(members.iter().any(|(owner, m)| owner.name == "Text" && m.name == "build"));
    assert!(members.iter().any(|(owner, m)| owner.name == "Widget" && m.name == "key"));
    // static members, enums and types in NuclearLang notation
    let theme = defs.get("Theme").unwrap().1;
    let of = theme.members.iter().find(|m| m.name == "of").unwrap();
    assert!(of.is_static);
    assert_eq!(of.returns.as_deref(), Some("ThemeData"));
    assert!(defs.get("TextAlign").unwrap().1.values.contains(&"center".to_string()));
    let json = defs.get("jsonDecode").unwrap().1;
    assert_eq!(json.kind, "function");
    assert_eq!(json.params[0].ty, "string");
    let callback = defs.get("ValueChanged").unwrap().1;
    assert_eq!(callback.ty.as_deref(), Some("(value: T) => void"));
    // a framework library
    let dio = defs.get("Dio").unwrap();
    assert_eq!(dio.0.id(), "dio@5.11.1");
    assert!(defs.members("Dio").iter().any(|(_, m)| m.name == "get"));
    let rendered = nuclear_dialect::defs::render(dio.0, dio.1);
    assert!(rendered.starts_with("// package:dio/dio.dart · dio@5.11.1\n"), "{rendered}");
}

#[test]
fn type_definitions_errors() {
    if without_platforms("type_definitions_errors") {
        return;
    }
    let app = |test: &str, defs: &str| {
        let dir = tmp(
            test,
            &[("app/dialect.toml", &format!("[dialect]\nname = \"app\"\nextends = \"flutter\"\ndefs = {defs}\n"))],
        );
        codes([vec![dir], repo_dialects()].concat(), "app")
    };
    assert_eq!(app("defs-string", "\"defs/nope.toml\""), ["N0206"]);
    assert_eq!(app("defs-ref", "[\"dio\"]"), ["N0206"]);
    assert_eq!(app("defs-missing", "[\"nope@1.0.0\"]"), ["N0213"]);
    assert_eq!(app("defs-version", "[\"dio@1.0.0\"]"), ["N0214"]);
    let (_, report) = Loader::new(repo_dialects()).load("flutter");
    assert!(report.diags.is_empty());
    let dir = tmp(
        "defs-avail",
        &[("app/dialect.toml", "[dialect]\nname = \"app\"\nextends = \"dart\"\ndefs = [\"dio@1.0.0\"]\n")],
    );
    let (_, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("app");
    let msg = &report.diags[0].1.message;
    assert!(msg.contains("available: 5.11.1"), "{msg}");
}

#[test]
fn project_type_definitions_extend_the_repository() {
    // a project keeps its own packages in <project>/defs, next to <project>/dialects
    let root = tmp(
        "defs-project",
        &[
            ("dialects/app/dialect.toml", "[dialect]\nname = \"app\"\nextends = \"dart\"\ndefs = [\"acme@1.0.0\"]\n"),
            (
                "defs/dart/acme/1.0.0/package.toml",
                "[package]\nname = \"acme\"\nversion = \"1.0.0\"\nlanguage = \"dart\"\nlibraries = [\"package:acme/acme.dart\"]\n",
            ),
            (
                "defs/dart/acme/1.0.0/index.json",
                r#"{"format": 1, "libraries": ["package:acme/acme.dart"], "symbols": {"AcmeButton": 0, "Duration": 0}, "classes": {"AcmeButton": {"supers": ["Object"], "members": ["press"]}}}"#,
            ),
            (
                "defs/dart/acme/1.0.0/api.json",
                r#"{"format": 1, "declarations": [{"name": "AcmeButton", "library": "package:acme/acme.dart", "kind": "class", "members": [{"name": "press", "kind": "method", "returns": "void"}]}]}"#,
            ),
        ],
    );
    let (d, report) = Loader::new([vec![root.join("dialects")], repo_dialects()].concat()).load("app");
    assert!(report.diags.is_empty(), "{:#?}", report.diags);
    let d = d.unwrap();
    assert_eq!(d.auto_imports["AcmeButton"].value, "package:acme/acme.dart");
    assert!(!d.auto_imports.contains_key("Duration")); // dart:core already has it
    assert_eq!(d.defs["AcmeButton"].members, ["press"]);
    assert_eq!(d.typedefs.members("AcmeButton")[0].1.name, "press");
}

#[test]
fn docs_urls_come_from_the_dialect_chain() {
    if without_platforms("docs_urls_come_from_the_dialect_chain") {
        return;
    }
    let (d, report) = Loader::new(repo_dialects()).load("flutter");
    let d = d.unwrap_or_else(|| panic!("{:#?}", report.diags));
    assert_eq!(
        d.docs_url("package:flutter/widgets.dart", "Text").as_deref(),
        Some("https://api.flutter.dev/flutter/widgets/Text-class.html")
    );
    assert_eq!(
        d.docs_url("dart:convert", "jsonDecode").as_deref(),
        Some("https://api.flutter.dev/flutter/dart-convert/jsonDecode.html")
    );
    assert_eq!(d.docs_url("package:dio/dio.dart", "Dio"), None);
    assert_eq!(d.project.marker, "main.nuc");
}

#[test]
fn dragonfly_dialects_implement_their_platform() {
    if without_platforms("dragonfly_dialects_implement_their_platform") {
        return;
    }
    // ADR-0093: a dialect in dialects/<platform>/ names its platform and matches platform.toml; a platform plugin a
    // dialect does not bring is a warning (none today: every backend dialect brings the whole list)
    let loader = Loader::new(repo_dialects());
    for (name, platform, missing) in [("flutter", "mobile", ""), ("fastapi", "backend", ""), ("nestjs", "backend", "")] {
        let (d, report) = loader.load(name);
        let d = d.unwrap_or_else(|| panic!("{name}: {:?}", report.diags.iter().map(|(_, x)| &x.message).collect::<Vec<_>>()));
        assert_eq!(d.platform.as_deref(), Some(platform), "{name}");
        let contract: Vec<&str> =
            report.diags.iter().filter(|(_, x)| x.code.id == "N0305").map(|(_, x)| x.message.as_str()).collect();
        if missing.is_empty() {
            assert!(contract.is_empty(), "{name}: {contract:?}");
        } else {
            assert_eq!(contract.len(), 1, "{name}: {contract:?}");
            assert!(contract[0].ends_with(&format!("yet: {missing}")), "{name}: {}", contract[0]);
        }
    }
    for name in ["typescript", "dart", "python", "nodejs"] {
        assert_eq!(loader.load(name).0.unwrap().platform, None, "{name}");
    }
    // a dialect that names a platform it is not under, and one that adds a kind the platform does not declare
    let dir = tmp("platform-stray", &[("stray/dialect.toml", "[dialect]\nname = \"stray\"\nextends = \"dart\"\nplatform = \"mobile\"\n")]);
    let (d, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("stray");
    assert!(d.is_none() && report.diags.iter().any(|(_, x)| x.code.id == "N0305" && x.message.contains("not in its folder")));
    let dir = tmp(
        "platform-extra",
        &[
            ("mobile/platform.toml", "[platform]\nname = \"mobile\"\nkinds = [\"stateless\"]\ndecorators = []\nplugins = []\n"),
            ("mobile/mini/dialect.toml", "[dialect]\nname = \"mini\"\nextends = \"dart\"\nplatform = \"mobile\"\n[kinds.stateless]\ndescription = \"x\"\ntemplate = \"class {{name}} {}\"\n[kinds.widget]\ndescription = \"x\"\ntemplate = \"class {{name}} {}\"\n"),
        ],
    );
    let (d, report) = Loader::new([vec![dir], repo_dialects()].concat()).load("mini");
    let msgs: Vec<&str> = report.diags.iter().filter(|(_, x)| x.code.id == "N0305").map(|(_, x)| x.message.as_str()).collect();
    assert!(d.is_none() && msgs.iter().any(|m| m.contains("adds kinds") && m.contains("widget")), "{msgs:?}");
}

#[test]
fn every_dialect_word_has_editor_docs() {
    if without_platforms("every_dialect_word_has_editor_docs") {
        return;
    }
    let loader = Loader::new(repo_dialects());
    for name in ["typescript", "nodejs", "nestjs", "dart", "flutter", "python", "fastapi"] {
        let d = loader.load(name).0.unwrap();
        let decorators = (d.consumed_decorators.iter().chain(&d.void_decorators).cloned())
            .chain(d.provide.keys().filter_map(|k| k.split('.').next().map(str::to_string)))
            .chain(d.decorator_args.keys().cloned());
        let words = d.modifiers.keys().chain(d.clauses.keys()).cloned();
        let wanted = (d.kinds.keys().map(|k| ("kinds", k.clone())))
            .chain(decorators.map(|k| ("decorators", k)))
            .chain(words.map(|k| ("words", k)));
        for (section, word) in wanted {
            let doc = d.doc(section, &word);
            assert!(doc.as_ref().is_some_and(|t| t.len() > 40), "{name}: no [{section}] {word} in docs.toml");
        }
    }
    // a docs.toml is not a dialect file: it does not break loading, and the child's entry wins
    let dir = tmp(
        "docs",
        &[
            ("base/dialect.toml", "[dialect]\nname = \"base\"\n[modifiers]\nstatic = \"static \"\n"),
            ("base/docs.toml", "[words]\nstatic = \"from base\"\n[kinds]\nx = \"only base\"\n"),
            ("child/dialect.toml", "[dialect]\nname = \"child\"\nextends = \"base\"\n"),
            ("child/docs.toml", "[words]\nstatic = \"from child\"\n"),
        ],
    );
    let (d, report) = Loader::new(vec![dir]).load("child");
    assert!(report.diags.is_empty(), "{:?}", report.diags);
    let d = d.unwrap();
    assert_eq!(d.doc("words", "static").as_deref(), Some("from child"));
    assert_eq!(d.doc("kinds", "x").as_deref(), Some("only base"));
    assert_eq!(d.doc("kinds", "y"), None);
}

#[test]
fn every_base_word_has_editor_docs() {
    for k in nuclear_syntax::token::Keyword::ALL {
        assert!(
            nuclear_dialect::base_doc("keywords", k.as_str()).is_some(),
            "no [keywords] {k} in languages/docs.toml"
        );
    }
    for name in ["Result", "Ok", "Err"] {
        assert!(nuclear_dialect::base_doc("builtins", name).is_some(), "no [builtins] {name}");
    }
}

#[test]
fn kind_packages_extend_a_platform_dialect() {
    if without_platforms("kind_packages_extend_a_platform_dialect") {
        return;
    }
    // ADR-0095: a package in kinds/<name>/ with an implementation for fastapi adds its kinds to fastapi only; nestjs
    // sees the gap (N0306 when a project enables it); a language dialect sees no packages
    let dir = tmp(
        "kind-package",
        &[
            ("kinds/jobs/kind.toml", "[package]\nname = \"jobs\"\nversion = \"0.1.0\"\nplatform = \"backend\"\ndescription = \"scheduled jobs\"\n"),
            ("kinds/jobs/docs.toml", "[kinds]\njob = \"a scheduled job\"\n"),
            ("kinds/jobs/fastapi/kind.toml", "[plugin]\npackages = { apscheduler = \">=3\" }\n\n[kinds.job]\ndescription = \"job Name + run()\"\nrequires = [\"run\"]\ntemplate = \"class {{name}}:\\n    {{members}}\"\n"),
        ],
    );
    let mut loader = Loader::new(repo_dialects());
    loader.packages.push(dir.join("kinds"));
    let (d, report) = loader.load("fastapi");
    let d = d.unwrap_or_else(|| panic!("{:?}", report.diags.iter().map(|(_, x)| &x.message).collect::<Vec<_>>()));
    assert!(d.kinds.contains_key("job"));
    assert_eq!(
        d.kind_sources.get("job"),
        Some(&KindSource::Package { name: "jobs".into(), version: "0.1.0".into(), dialects: vec!["fastapi".into()] })
    );
    assert_eq!(d.kind_sources.get("controller"), Some(&KindSource::Platform("backend".into())));
    let plugin = d.project.plugin("jobs").expect("the package is a plugin");
    assert_eq!((plugin.kinds.clone(), plugin.packages.clone()), (vec!["job".to_string()], vec![("apscheduler".to_string(), ">=3".to_string())]));
    assert_eq!(d.doc("kinds", "job").as_deref(), Some("a scheduled job"));
    assert!(d.package_gaps.is_empty());
    let (n, _) = loader.load("nestjs");
    let n = n.unwrap();
    assert!(!n.kinds.contains_key("job"));
    assert_eq!(n.package_gaps.get("jobs"), Some(&vec!["fastapi".to_string()]));
    assert!(n.project.plugin("jobs").is_some());
    let (t, _) = loader.load("typescript");
    assert!(t.unwrap().packages.is_empty());
}

/// `[layout]` of every platform says where the file of each of its kinds goes (agents and generators follow it).
#[test]
fn every_platform_kind_has_a_place_in_the_layout() {
    if without_platforms("every_platform_kind_has_a_place_in_the_layout") {
        return;
    }
    let l = Loader::new(repo_dialects());
    for dialect in ["flutter", "fastapi", "nestjs", "react"] {
        let d = l.load(dialect).0.unwrap();
        let language = d.chain.last().cloned().unwrap_or_default();
        let missing: Vec<&String> =
            d.kinds.iter().filter(|(k, e)| e.origin.dialect != language && !d.layout.contains_key(*k)).map(|(k, _)| k).collect();
        assert!(missing.is_empty(), "{dialect}: no layout for {missing:?}");
        assert!(d.layout.values().all(|p| p.ends_with(".nuc")), "{dialect}: {:?}", d.layout);
    }
}
