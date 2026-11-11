//! Projects: several `.nuc` files transpiled together (spec/projects.md).

use nuclear_dialect::Loader;
use nuclear_transpile::project::{self, Manifest};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");


/// Tests that build with a platform dialect (flutter, fastapi, nestjs, react) need the platform repositories checked
/// out inside this working tree (plan 006): without them, the test is skipped with a note.
fn without_platforms(test: &str) -> bool {
    let l = nuclear_dialect::Loader::new(nuclear_dialect::dialect_dirs(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    let missing: Vec<&str> = ["flutter", "fastapi", "nestjs", "react"].into_iter().filter(|d| l.load(d).0.is_none()).collect();
    if !missing.is_empty() {
        eprintln!("skipped {test}: the dialects {missing:?} are not in this working tree (their platform repositories are not checked out)");
    }
    !missing.is_empty()
}
fn loader(extra: Option<&Path>) -> Loader {
    let mut paths: Vec<PathBuf> = extra.into_iter().map(Path::to_path_buf).collect();
    paths.extend(nuclear_dialect::dialect_dirs(Path::new(ROOT)));
    Loader::new(paths)
}

/// `rel` (a file or folder) in the source root that has it: the platform repositories hold their own specs and
/// examples (plan 006). `None`, with a note, when that repository is not checked out: the test is skipped.
fn tree(rel: &str) -> Option<PathBuf> {
    let found = nuclear_dialect::source_roots(Path::new(ROOT)).into_iter().map(|r| r.join(rel)).find(|p| p.exists());
    if found.is_none() {
        eprintln!("skipped: {rel} is not in this working tree (its platform repository is not checked out)");
    }
    found
}

/// The golden output of an example: `examples/expected/<name>` of the same source root.
fn expected_of(project: &Path) -> PathBuf {
    project.parent().unwrap().parent().unwrap().join("expected").join(project.file_name().unwrap())
}

/// Every project of examples/projects/<name>/ and the Dragonfly example against examples/expected/<name>/.
/// A Dragonfly project has only .nuc files: without nuclear.toml it is a flutter project named like its folder.
#[test]
fn projects_match_expected_output() {
    if without_platforms("projects_match_expected_output") {
        return;
    }
    let update = std::env::var("UPDATE_GOLDEN").is_ok();
    let mut failures = Vec::new();
    let dir = PathBuf::from(ROOT).join("examples/projects");
    let mut roots: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("nuclear.toml").is_file())
        .collect();
    for example in [
        "examples-flutter-dialect/clean-flutter-project",
        "examples-flutter-dialect/genkit-assistant",
        "examples-fastapi/rick_api",
        "examples-backend/tasks_api",
        "examples-web/characters",
    ] {
        roots.extend(tree(&format!("examples/{example}")));
    }
    roots.sort();
    // projects are found as the editor finds them: nuclear.toml, or the `[project] marker` of a dialect
    let dialects: Vec<_> = ["dart", "flutter", "react"].iter().map(|n| loader(None).load(n).0.unwrap()).collect();
    for root in roots {
        let (found, manifest) = Manifest::find(&root.join("main.nuc"), &dialects).unwrap();
        assert_eq!(found, root);
        let manifest = manifest.unwrap();
        // config.nuc names the platform and the dialect (ADR-0093); the output is the dialect's
        match manifest.platform.as_deref() {
            Some("mobile") => assert_eq!((manifest.dialect.as_str(), manifest.out.as_str()), ("flutter", "build/app")),
            Some("backend") => assert!(["fastapi", "nestjs"].contains(&manifest.dialect.as_str()), "{}", root.display()),
            Some("web") => assert_eq!((manifest.dialect.as_str(), manifest.out.as_str()), ("react", "build/app")),
            None => assert!(root.join("nuclear.toml").is_file(), "{}", root.display()),
            other => panic!("{}: platform {other:?}", root.display()),
        }
        let (d, report) = loader(None).load(&manifest.dialect);
        let d = d.unwrap_or_else(|| panic!("{}: {:#?}", root.display(), report.diags));
        let out = project::build(&root, &manifest, &d);
        for f in &out.files {
            let errors: Vec<_> = f.diagnostics.iter().filter(|x| x.is_error()).collect();
            assert!(errors.is_empty(), "{}: {errors:#?}", f.source_path.display());
        }
        let expected = expected_of(&root);
        let all: Vec<(String, String)> = out
            .files
            .iter()
            .filter(|f| !f.out_path.is_empty())
            .map(|f| (f.out_path.clone(), f.code.clone()))
            .chain(out.generated.clone())
            .collect();
        // an expected file the build no longer produces is a failure too (a dialect key that stopped working)
        if !update {
            let produced: HashSet<String> =
                all.iter().map(|(r, _)| r.clone()).chain(out.copies.iter().map(|(r, _)| r.clone())).collect();
            let mut pending = vec![expected.clone()];
            while let Some(d) = pending.pop() {
                for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        pending.push(p);
                    } else {
                        let rel = p.strip_prefix(&expected).unwrap().to_string_lossy().replace('\\', "/");
                        if !produced.contains(&rel) {
                            failures.push(format!("{rel} is expected but no longer generated"));
                        }
                    }
                }
            }
        }
        for (rel, code) in all {
            let path = expected.join(&rel);
            if update {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, &code).unwrap();
                continue;
            }
            let want = std::fs::read_to_string(&path).unwrap_or_default();
            if want != code {
                failures.push(format!("{} differs from {}:\n--- got ---\n{code}", rel, path.display()));
            }
        }
        // binary assets (images) are copied as they are
        for (rel, source) in &out.copies {
            let path = expected.join(rel);
            if update {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::copy(source, &path).unwrap();
            } else if std::fs::read(&path).ok() != std::fs::read(source).ok() {
                failures.push(format!("{rel} is not copied to {}", path.display()));
            }
        }
    }
    assert!(failures.is_empty(), "{}\n(run UPDATE_GOLDEN=1 cargo test to accept)", failures.join("\n\n"));
}

/// ADR-0093 §9: the backend example is the platform's Nuclear, so the same files build with every backend dialect
/// (`tools/fastapi-check/check.sh` and `tools/nestjs-check/check.sh` then run each output's checks).
#[test]
fn backend_example_builds_with_every_backend_dialect() {
    let Some(example) = tree("examples/examples-backend/tasks_api") else { return };
    assert_eq!(Manifest::read(&example).unwrap().platform.as_deref(), Some("backend"));
    for dialect in ["nestjs", "fastapi"] {
        // the same files, with `config.nuc` naming this dialect (the manifest check wants them to agree)
        let root = std::env::temp_dir().join(format!("nuclear-conformance-{}-{dialect}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        copy_tree(&example, &root);
        let config = std::fs::read_to_string(root.join("config.nuc")).unwrap();
        std::fs::write(root.join("config.nuc"), config.replace("dialect = \"nestjs\"", &format!("dialect = \"{dialect}\""))).unwrap();
        let manifest = Manifest::read(&root).unwrap();
        assert_eq!(manifest.dialect, dialect);
        let (d, report) = loader(None).load(dialect);
        let d = d.unwrap_or_else(|| panic!("{dialect}: {:#?}", report.diags));
        let out = project::build(&root, &manifest, &d);
        for f in &out.files {
            let errors: Vec<_> = f.diagnostics.iter().filter(|x| x.is_error()).collect();
            assert!(errors.is_empty(), "{dialect}: {}: {errors:#?}", f.source_path.display());
        }
        let main = if dialect == "fastapi" { "app/main.py" } else { "src/main.ts" };
        assert!(out.generated.iter().any(|(p, _)| p == main), "{dialect}: no {main}");
        // the project card: what the editors read to know the platform, dialect and kinds of the project
        project::write_metadata(&root, &manifest, &d).unwrap();
        let card: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(project::METADATA_FILE)).unwrap()).unwrap();
        assert_eq!(card["platform"], "backend");
        assert_eq!(card["dialect"], dialect);
        assert_eq!(card["kinds"]["controller"], "platform:backend");
        assert!(card["plugins"].as_array().unwrap().iter().any(|p| p == "sqlite"), "{card}");
    }
}

/// ADR-0098: a backend's build writes its contract; a client that declares it gets the models and repositories
/// generated, and a change in the backend breaks the client's build with the field's name.
#[test]
fn a_client_builds_from_the_backend_contract_and_sees_drift() {
    let tmp = std::env::temp_dir().join(format!("nuclear-contracts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let api = tmp.join("tasks_api");
    let app = tmp.join("tasks_app");
    let (Some(api_src), Some(app_src)) =
        (tree("examples/examples-backend/tasks_api"), tree("examples/examples-flutter-dialect/tasks_app"))
    else {
        return;
    };
    copy_tree(&api_src, &api);
    copy_tree(&app_src, &app);
    let _ = std::fs::remove_dir_all(app.join("contracts"));
    std::fs::write(
        app.join("config.nuc"),
        // the copy's contract is the copied backend next to it, wherever the example points to
        std::fs::read_to_string(app.join("config.nuc"))
            .unwrap()
            .lines()
            .map(|l| if l.starts_with("contracts(") { "contracts(tasks_api = \"../tasks_api\")" } else { l })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    // the backend: build + card writes .nuclear/contract.json and openapi.json
    let (nest, _) = loader(None).load("nestjs");
    let nest = nest.unwrap();
    let api_manifest = Manifest::read(&api).unwrap();
    project::write_metadata(&api, &api_manifest, &nest).unwrap();
    let contract: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(api.join(".nuclear/contract.json")).unwrap()).unwrap();
    assert_eq!(contract["models"]["Task"]["key"], "id");
    let routes = contract["controllers"]["TasksController"]["routes"].as_array().unwrap();
    let one = routes.iter().find(|r| r["name"] == "one").unwrap();
    assert_eq!(one["params"][0]["in"], "path");
    let openapi: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(api.join(".nuclear/openapi.json")).unwrap()).unwrap();
    assert!(openapi["paths"]["/tasks/{id}"]["get"].is_object(), "{openapi}");
    // the same contract whatever the backend dialect
    let (fast, _) = loader(None).load("fastapi");
    let mut with_fastapi = nuclear_transpile::contract::contract(&api, &api_manifest, &fast.unwrap());
    let mut with_nest = contract.clone();
    for c in [&mut with_fastapi, &mut with_nest] {
        c.as_object_mut().unwrap().remove("dialect");
    }
    assert_eq!(with_fastapi, with_nest);
    // the client: the build generates contracts/tasks_api/ and compiles against it
    let (flutter, _) = loader(None).load("flutter");
    let flutter = flutter.unwrap();
    let app_manifest = Manifest::read(&app).unwrap();
    let out = project::build(&app, &app_manifest, &flutter);
    let errors: Vec<_> = out.files.iter().flat_map(|f| f.diagnostics.iter().filter(|d| d.is_error())).collect();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(app.join("contracts/tasks_api/models/task.nuc").is_file());
    assert!(app.join("contracts/tasks_api/tasks_repository.nuc").is_file());
    // drift: the backend renames a field, the client's use of it fails at build time
    let entity = api.join("components/tasks/data/entities/task.nuc");
    std::fs::write(&entity, std::fs::read_to_string(&entity).unwrap().replace("  done: boolean = false", "  completed: boolean = false")).unwrap();
    project::write_metadata(&api, &api_manifest, &nest).unwrap();
    let usecase = app.join("components/tasks/domain/usecases/add_task_usecase.nuc");
    std::fs::write(
        &usecase,
        "usecase AddTaskUseCase\n  (tasks: TasksRepository)\n\n  exec(title: string) Result<TasksApiError, Task>\n    saved = ~tasks.create(input = NewTask(title = title))\n    Log.d(\"created\", data = {\"done\": saved.done})\n    Ok(saved)\n",
    )
    .unwrap();
    let out = project::build(&app, &app_manifest, &flutter);
    let messages: Vec<String> = out.files.iter().flat_map(|f| f.diagnostics.iter().filter(|d| d.is_error()).map(|d| d.message.clone())).collect();
    assert!(messages.iter().any(|m| m.contains("done")), "{messages:#?}");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// ADR-0100: the rules of the relation decorators (`[decorators.fields]` of the backend platform) are N0308, the same
/// in both backend dialects, also when the dialect writes names in snake_case (fastapi).
#[test]
fn relation_decorators_have_rules() {
    let Some(example) = tree("examples/examples-backend/tasks_api") else { return };
    for dialect in ["nestjs", "fastapi"] {
        let tmp = std::env::temp_dir().join(format!("nuclear-relations-{dialect}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        copy_tree(&example, &tmp);
        let (d, _) = loader(None).load(dialect);
        let d = d.unwrap();
        let errors = |root: &std::path::Path| -> Vec<String> {
            let out = project::build(root, &Manifest::read(root).unwrap(), &d);
            out.files
                .iter()
                .flat_map(|f| f.diagnostics.iter().filter(|x| x.code.id == "N0308").map(|x| x.message.clone()))
                .collect()
        };
        assert!(errors(&tmp).is_empty(), "{dialect}: {:#?}", errors(&tmp));
        let task = tmp.join("components/tasks/data/entities/task.nuc");
        let src = std::fs::read_to_string(&task).unwrap();
        let broken = src
            .replace("  project?: Project\n  projectId?: int\n", "  project: Project\n")
            .replace("@manyToMany(inverse = \"tasks\")\n  @joinTable()", "@manyToMany(inverse = \"items\")");
        std::fs::write(&task, broken).unwrap();
        let found = errors(&tmp);
        for want in ["the field is optional", "add the field `projectId`", "names no field of Tag", "@joinTable"] {
            assert!(found.iter().any(|m| m.contains(want)), "{dialect}: no '{want}' in {found:#?}");
        }
        // N0309: a removed decorator is an error that says what replaces it, never silently dropped
        let tag = tmp.join("components/tasks/data/entities/tag.nuc");
        std::fs::write(&tag, std::fs::read_to_string(&tag).unwrap().replace("@primary(generated = true)", "@generated()")).unwrap();
        let out = project::build(&tmp, &Manifest::read(&tmp).unwrap(), &d);
        let n0309: Vec<String> =
            out.files.iter().flat_map(|f| f.diagnostics.iter().filter(|x| x.code.id == "N0309").map(|x| x.message.clone())).collect();
        assert!(n0309.iter().any(|m| m.contains("@primary(generated = true)")), "{dialect}: {n0309:#?}");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

/// Copies a project's sources (not its `build/`).
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() != "build" {
                copy_tree(&path, &target);
            }
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// A temporary project: `files` is (relative path, content); a dialect `arch` extends dart with
/// `dialect_extra`.
fn tmp_project(test: &str, dialect_extra: &str, files: &[(&str, &str)]) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("nuclear-project-test-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let dialects = dir.join("dialects");
    std::fs::create_dir_all(dialects.join("arch")).unwrap();
    std::fs::write(
        dialects.join("arch/dialect.toml"),
        format!("[dialect]\nname = \"arch\"\nextends = \"dart\"\n\n{dialect_extra}"),
    )
    .unwrap();
    let root = dir.join("app");
    for (path, content) in
        files.iter().chain([("nuclear.toml", "[project]\nname = \"app\"\ndialect = \"arch\"\n")].iter())
    {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    (root, dialects)
}

fn build_codes(root: &Path, dialects: &Path) -> Vec<(String, &'static str)> {
    let manifest = Manifest::read(root).unwrap();
    let (d, report) = loader(Some(dialects)).load(&manifest.dialect);
    let d = d.unwrap_or_else(|| panic!("{:#?}", report.diags));
    let out = project::build(root, &manifest, &d);
    out.files
        .iter()
        .flat_map(|f| f.diagnostics.iter().map(move |x| (f.source_path.display().to_string(), x.code.id)))
        .collect()
}

const ARCH: &str = r#"
[names]
globals = ["inject"]

[kinds.usecase]
template = "class {{name}} {\n  {{members}}\n}"
methods = ["exec"]
returns = "Result"

[kinds.appcomponent]
template = "class {{name}} {\n  {{members}}\n}"
alias_via = "inject"

[architecture]
scope = "components"
public = ["appcomponent"]

[architecture.callers]
usecase = ["appcomponent"]
"#;

#[test]
fn a_usecase_is_only_used_from_its_appcomponent() {
    let (root, dialects) = tmp_project(
        "callers",
        ARCH,
        &[
            (
                "components/users/domain/usecases/find_user.nuc",
                "usecase FindUser\n  exec(id: int) Result<int, string>\n    Ok(id)\n",
            ),
            ("components/users/users_component.nuc", "appcomponent UsersComponent\n  find() = FindUser.exec\n"),
            ("components/users/presentation/screen.nuc", "show(id: int) void\n  print(FindUser())\n"),
        ],
    );
    let codes = build_codes(&root, &dialects);
    assert_eq!(codes, [("components/users/presentation/screen.nuc".to_string(), "N0301")], "{codes:?}");
}

#[test]
fn other_components_only_see_the_appcomponent() {
    let (root, dialects) = tmp_project(
        "private",
        ARCH,
        &[
            (
                "components/users/domain/usecases/find_user.nuc",
                "usecase FindUser\n  exec(id: int) Result<int, string>\n    Ok(id)\n",
            ),
            ("components/users/users_component.nuc", "appcomponent UsersComponent\n  find() = FindUser.exec\n"),
            (
                "components/sales/sale.nuc",
                "sell() void\n  print(UsersComponent())\n\nsteal() void\n  print(FindUser())\n",
            ),
        ],
    );
    let codes = build_codes(&root, &dialects);
    assert_eq!(codes, [("components/sales/sale.nuc".to_string(), "N0302")], "{codes:?}");
}

#[test]
fn kinds_can_limit_methods_and_return_types() {
    let (root, dialects) = tmp_project(
        "kind-rules",
        ARCH,
        &[(
            "components/users/domain/usecases/find_user.nuc",
            "usecase FindUser\n  exec(id: int) int\n    id\n  other() void\n    print(1)\n",
        )],
    );
    let mut codes: Vec<&str> = build_codes(&root, &dialects).into_iter().map(|(_, c)| c).collect();
    codes.sort();
    assert_eq!(codes, ["N0215", "N0216", "N0216"]);
}

#[test]
fn folders_give_a_kind_and_files_are_generated() {
    let extra = r#"
[kinds.model]
template = "class {{name}} {}"
file_header = "// {{file}} of {{project}}"

[project]
sources = "lib"

[project.kinds]
"components/*/models" = "model"

[project.files."lib/registry.dart"]
names = ["registry"]
template = """
{{#files}}
import '{{import}}';
{{/files}}

// {{#kinds.model}}{{name}} {{/kinds.model}}in a comment does not import its file
final registry = [{{#kinds.model}}{{name}}, {{/kinds.model}}];
"""
"#;
    let (root, dialects) = tmp_project(
        "folders",
        extra,
        &[
            ("components/users/models/user.nuc", "User\n  name: string\n"),
            ("main.nuc", "main() void\n  print(registry)\n"),
        ],
    );
    let manifest = Manifest::read(&root).unwrap();
    let (d, _) = loader(Some(&dialects)).load("arch");
    let out = project::build(&root, &manifest, &d.unwrap());
    let user = out.files.iter().find(|f| f.out_path == "lib/components/users/models/user.dart").unwrap();
    assert_eq!(user.code, "// user of app\n\nclass User {}\n");
    let main = out.files.iter().find(|f| f.out_path == "lib/main.dart").unwrap();
    assert!(main.code.starts_with("import 'package:app/registry.dart';"), "{}", main.code);
    let (path, registry) = &out.generated[0];
    assert_eq!(path, "lib/registry.dart");
    assert!(registry.contains("import 'package:app/components/users/models/user.dart';"), "{registry}");
    assert!(registry.contains("final registry = [User, ];"), "{registry}");
}

#[test]
fn a_name_declared_in_two_files_is_an_error() {
    let (root, dialects) =
        tmp_project("dup", "", &[("a.nuc", "User\n  (name: string)\n"), ("b.nuc", "User\n  (age: int)\n")]);
    let codes = build_codes(&root, &dialects);
    assert_eq!(codes, [("b.nuc".to_string(), "N0303")]);
}

/// spec/flutter.md shows the real output of the Dragonfly example: each of its ```dart blocks with a
/// project caption is part of a file of the example's expected folder.
#[test]
fn dragonfly_spec_shows_the_real_output() {
    let (Some(spec), Some(expected)) = (tree("spec/flutter.md"), tree("examples/expected/clean-flutter-project")) else {
        return;
    };
    let spec = std::fs::read_to_string(spec).unwrap();
    let mut files = Vec::new();
    let mut pending = vec![expected];
    while let Some(d) = pending.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            if e.path().is_dir() {
                pending.push(e.path());
            } else if let Ok(text) = std::fs::read_to_string(e.path()) {
                files.push(text); // images are not shown in the spec
            }
        }
    }
    // the blocks of the project are captioned "Output (`lib/...`" or "Generated (`lib/...`"
    let blocks: Vec<&str> = spec
        .split("```dart\n")
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|w| {
            let before = w[0].rsplit("```").next().unwrap_or("");
            before.contains("Output (`") || before.contains("Generated (`")
        })
        .map(|w| w[1].split("```").next().unwrap().trim())
        .collect();
    assert!(blocks.len() >= 10);
    for b in blocks {
        assert!(
            files.iter().any(|f| f.contains(b)),
            "spec/flutter.md shows code that the example does not generate:\n{b}"
        );
    }
}

const SCREENS: &str = r#"
[decorators]
consumed = ["screen"]

[decorators.args]
screen = ["route", "loggedIn", "loggedIn.forward!", "providers"]

[decorators.provide]
"screen.providers" = "read<{{name}}>()"

[kinds.screen]
template = "class {{name}} {\n  // {{decor.screen.route}} {{decor.screen.loggedIn.forward}}{{#decor.screen.providers.items}} {{.}}{{/decor.screen.providers.items}}\n  {{members}}\n}"

[kinds.manager]
template = "class {{name}} {\n  {{members}}\n}"

[architecture]
scope = "components"
public = ["@screen"]

[architecture.names]
Context = ["screen"]
"#;

#[test]
fn decorator_arguments_follow_the_dialect() {
    let (root, dialects) = tmp_project(
        "decorator-args",
        SCREENS,
        &[(
            "components/a/views.nuc",
            "@screen(route = \"/\", loggedIn = { })\nscreen Home\n  x() void\n    print(1)\n\n\
             @screen(rute = \"/x\", loggedIn = { forward: Home, to: Home })\nscreen Other\n  x() void\n    print(1)\n\n\
             @screen(loggedIn = Home)\nscreen Third\n  x() void\n    print(1)\n",
        )],
    );
    let codes: Vec<&str> = build_codes(&root, &dialects).into_iter().map(|(_, c)| c).collect();
    // missing forward; unknown rute; unknown field to; loggedIn is not an object
    assert_eq!(codes, ["N0217", "N0217", "N0217", "N0217"], "{codes:?}");
}

#[test]
fn objects_and_lists_of_decorators_reach_templates_and_provide() {
    let (root, dialects) = tmp_project(
        "decorator-values",
        SCREENS,
        &[
            ("components/a/manager.nuc", "manager Counter\n  add() void\n    print(1)\n"),
            (
                "components/a/home.nuc",
                "@screen(route = \"/\", loggedIn = { forward: Login }, providers = [Counter])\nscreen Home\n  tap() void\n    Counter.add()\n",
            ),
            (
                "components/b/login.nuc",
                "@screen(route = \"/login\", loggedIn = { forward: Login })\nscreen Login\n  x() void\n    print(1)\n",
            ),
        ],
    );
    let manifest = Manifest::read(&root).unwrap();
    let (d, _) = loader(Some(&dialects)).load("arch");
    let out = project::build(&root, &manifest, &d.unwrap());
    let home = out.files.iter().find(|f| f.out_path.ends_with("home.dart")).unwrap();
    // a screen of another component is public (@screen); the decorator alone does not import it
    assert!(home.diagnostics.is_empty(), "{:?}", home.diagnostics);
    assert!(home.code.contains("// '/' Login Counter"), "{}", home.code);
    assert!(home.code.contains("read<Counter>().add();"), "{}", home.code);
    assert!(!home.code.contains("login.dart"), "{}", home.code);
}

#[test]
fn some_names_are_only_for_some_kinds() {
    let (root, dialects) = tmp_project(
        "names",
        SCREENS,
        &[
            ("components/a/home.nuc", "screen Home\n  show(c: Context) void\n    print(c)\n"),
            ("components/a/manager.nuc", "manager Counter\n  add(c: Context) void\n    print(c)\n"),
        ],
    );
    let codes = build_codes(&root, &dialects);
    assert_eq!(codes, [("components/a/manager.nuc".to_string(), "N0301")], "{codes:?}");
}

/// config.nuc: the project's manifest in Nuclear is read (sections → named values), checked, and never
/// transpiled.
#[test]
fn the_nuclear_manifest_is_read_and_not_transpiled() {
    let sections: Vec<String> = ["app", "dependencies"].map(String::from).to_vec();
    let src = "app(\n  name = \"shop\",\n  usesMaterialDesign = true,\n)\n\ndependencies(\n  hive = \"^2.2.3\",\n)\n";
    let (config, diags) = project::read_config(src, &sections);
    assert!(diags.is_empty(), "{diags:#?}");
    assert_eq!(config["app"], vec![("name".into(), "shop".into()), ("usesMaterialDesign".into(), "true".into())]);
    assert_eq!(config["dependencies"], vec![("hive".into(), "^2.2.3".into())]);

    let (_, diags) = project::read_config("other(a = 1)\napp(name = x)\n", &sections);
    let messages: Vec<&str> = diags.iter().map(|d| d.message.as_str()).collect();
    assert!(messages.iter().any(|m| m.contains("'other' is not a section")), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("a text, a number or a boolean")), "{messages:?}");

    let (root, dialects) = tmp_project(
        "manifest",
        "[project.manifest]\nfile = \"config.nuc\"\nsections = [\"app\"]\n",
        &[("config.nuc", "app(name = \"shop\")\n"), ("main.nuc", "main() void\n  print(1)\n")],
    );
    let (d, report) = loader(Some(&dialects)).load("arch");
    let d = d.unwrap_or_else(|| panic!("{:#?}", report.diags));
    let out = project::build(&root, &Manifest::read(&root).unwrap(), &d);
    let config = out.files.iter().find(|f| f.source_path == Path::new("config.nuc")).expect("config.nuc is checked");
    assert!(config.out_path.is_empty() && config.code.is_empty() && config.diagnostics.is_empty());

    // only third-party packages: not the dialect's (its defs, like dart) nor the framework's (reserved)
    let (root, dialects) = tmp_project(
        "manifest_reserved",
        "[project.manifest]\nfile = \"config.nuc\"\nsections = [\"dependencies\"]\npackages = [\"dependencies\"]\nreserved = [\"fpdart\"]\n",
        &[("config.nuc", "dependencies(\n  fpdart = \"^1.0.0\",\n  dart = \"1\",\n  hive = \"^2.2.3\",\n)\n")],
    );
    let (d, report) = loader(Some(&dialects)).load("arch");
    let d = d.unwrap_or_else(|| panic!("{:#?}", report.diags));
    let out = project::build(&root, &Manifest::read(&root).unwrap(), &d);
    let config = out.files.iter().find(|f| f.source_path == Path::new("config.nuc")).unwrap();
    let flagged: Vec<&str> = config.diagnostics.iter().map(|x| &config.source[x.span.start..x.span.end]).collect();
    assert_eq!(flagged, ["fpdart", "dart"], "{:#?}", config.diagnostics);
}

/// ADR-0095: a project enables a kind package in config.nuc; it builds with a dialect the package implements and
/// reports N0306 with one it does not.
#[test]
fn kind_package_builds_with_its_dialects_only() {
    if without_platforms("kind_package_builds_with_its_dialects_only") {
        return;
    }
    let root = std::env::temp_dir().join(format!("nuclear-kind-package-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let files = [
        ("config.nuc", "dragonfly(platform = \"backend\", dialect = \"fastapi\")\napp(name = \"demo\")\nplugins(jobs = true)\n"),
        ("config/project_config.nuc", "appConfig ProjectConfig\n  connections: Connection[] = []\n"),
        ("components/cleanup/jobs/purge_job.nuc", "job PurgeJob\n  run() int\n    0\n"),
        ("kinds/jobs/kind.toml", "[package]\nname = \"jobs\"\nversion = \"0.1.0\"\nplatform = \"backend\"\n"),
        ("kinds/jobs/fastapi/kind.toml", "[kinds.job]\ndescription = \"job Name + run()\"\nrequires = [\"run\"]\ntemplate = \"class {{name}}:\\n    {{members}}\"\n"),
    ];
    for (rel, text) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let mut l = loader(None);
    l.packages.push(root.join("kinds"));
    for (dialect, ok) in [("fastapi", true), ("nestjs", false)] {
        let config = std::fs::read_to_string(root.join("config.nuc")).unwrap();
        std::fs::write(root.join("config.nuc"), config.replace("dialect = \"fastapi\"", &format!("dialect = \"{dialect}\"")).replace("dialect = \"nestjs\"", &format!("dialect = \"{dialect}\""))).unwrap();
        let manifest = Manifest::read(&root).unwrap();
        let d = l.load(dialect).0.unwrap();
        let out = project::build(&root, &manifest, &d);
        let errors: Vec<String> = out.files.iter().flat_map(|f| f.diagnostics.iter().filter(|x| x.is_error()).map(|x| format!("{}: {}", x.code.id, x.message))).collect();
        if ok {
            assert!(errors.is_empty(), "{dialect}: {errors:?}");
            let job = out.files.iter().find(|f| f.source_path.ends_with("purge_job.nuc")).unwrap();
            assert!(job.code.contains("class PurgeJob:"), "{}", job.code);
        } else {
            assert!(errors.iter().any(|e| e.starts_with("N0306:") && e.contains("it has: fastapi")), "{dialect}: {errors:?}");
        }
    }
}

/// A backend project adds a package with `dependencies(...)` and keeps its type definitions in `<project>/defs`:
/// the code sees its names (no N0106), imports them, and the generated manifest lists the package.
#[test]
fn a_backend_project_adds_a_package_with_its_own_definitions() {
    if without_platforms("a_backend_project_adds_a_package_with_its_own_definitions") {
        return;
    }
    let root = std::env::temp_dir().join(format!("nuclear-project-defs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let defs = "defs/python/probe_lib/1.0.0";
    let files = [
        ("config.nuc", "dragonfly(platform = \"backend\", dialect = \"fastapi\")\napp(name = \"demo\")\ndependencies(probe_lib = \">=1.0\", local_lib = \"../local_lib\")\n"),
        ("config/project_config.nuc", "appConfig ProjectConfig\n  connections: Connection[] = []\n"),
        ("components/probes/domain/services/probe_service.nuc", "service ProbeService\n  make() Probe\n    Probe()\n"),
        (&format!("{defs}/package.toml") as &str, "[package]\nname = \"probe_lib\"\nversion = \"1.0.0\"\nlanguage = \"python\"\ndepends = []\nlibraries = [\"probe_lib\"]\n"),
        (&format!("{defs}/index.json"), "{\"format\": 1, \"libraries\": [\"probe_lib\"], \"reexports\": {}, \"symbols\": {\"Probe\": 0}, \"classes\": {\"Probe\": {\"supers\": [], \"members\": []}}}"),
        (&format!("{defs}/api.json"), "{\"format\": 1, \"declarations\": [{\"name\": \"Probe\", \"library\": \"probe_lib\", \"kind\": \"class\", \"constructors\": [{\"name\": \"\", \"params\": []}]}]}"),
    ];
    for (rel, text) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let manifest = Manifest::read(&root).unwrap();
    let d = loader(None).load("fastapi").0.unwrap();
    assert_eq!(project::project_defs(&root, &d), ["probe_lib@1.0.0"]);
    let (own, missing) = project::with_project_defs(&root, &d);
    assert!(missing.is_empty(), "{:?}", missing.iter().map(|m| &m.message).collect::<Vec<_>>());
    let out = project::build(&root, &manifest, &own.unwrap());
    let errors: Vec<String> = out.files.iter().flat_map(|f| f.diagnostics.iter().filter(|x| x.is_error()).map(|x| format!("{}: {}", x.code.id, x.message))).collect();
    assert!(errors.is_empty(), "{errors:?}");
    let service = out.files.iter().find(|f| f.source_path.ends_with("probe_service.nuc")).unwrap();
    assert!(service.code.contains("from probe_lib import Probe"), "{}", service.code);
    let pyproject = &out.generated.iter().find(|(p, _)| p == "pyproject.toml").unwrap().1;
    assert!(pyproject.contains("\"probe_lib>=1.0\","), "{pyproject}");
    assert!(pyproject.contains("\"local_lib @ file:///") && pyproject.contains("local_lib\","), "{pyproject}");
    let _ = std::fs::remove_dir_all(&root);
}
