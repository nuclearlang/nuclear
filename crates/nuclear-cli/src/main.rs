//! `nuclear`: the transpiler's command line.

mod agent;

use nuclear_diag::{Diagnostic, SourceFile, json_items, render_text};
use nuclear_dialect::{Dialect, Loader, Report};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage:
  nuclear help [<topic>|<command>]                            this help, a documentation topic or the usage of a command
  nuclear help <word>|words|layout --dialect <name>           a kind, decorator or header word of the dialect (words: all; layout: where each kind's file goes)
  nuclear build <file.nuc> --dialect <name|path> [-o <out>]   transpile (prints to stdout without -o)
  nuclear build <folder> [-o <out>]                           transpile a project (folder with nuclear.toml)
  nuclear check <folder> [--json]                             report the errors of every file of a project
  nuclear check <file.nuc> [--dialect <name|path>] [--json]   report errors (JSON for agents and the LSP)
  nuclear fix <folder|file.nuc> [--dialect|--platform]        apply every exact fix of the errors (the editor's quick
                                                              fixes), then report what is left
  --platform backend|mobile|web                               instead of --dialect: the platform's default dialect
                                                              (the Nuclear of a platform is the same in all of them)
  nuclear fmt <file.nuc|folder>... [--check]                  rewrite the files in the canonical layout (spacing,
                                                              indentation, blank lines; never reorders); --check
                                                              only lists the files that would change (exit 1)
  nuclear tokens <file.nuc>                                   print the tokens produced by the lexer
  nuclear ast <file.nuc>                                      print the syntax tree
  nuclear dialect list                                        list the dialects found
  nuclear dialect check <name|path> [--json]                  validate a dialect's files
  nuclear kind new <name> -p <platform> -d <dialect>          a kind package skeleton in kinds/<name>/ (ADR-0095)
  nuclear spec [--dialect <name>|--platform <p>] [--brief]    print the base language and dialect format (for agents);
                                                              with --dialect, also the spec of the dialect and its parents;
                                                              --brief: the short sheet (what changes from TS/Dart)
  nuclear agent init [--dialect <name>]                       AGENTS.md, CLAUDE.md, a Cursor rule and a Claude Code hook for AI agents
  nuclear agent hook                                          (the hook: checks the .nuc an agent edited; reads the tool call on stdin)
  nuclear contracts list                                      the contracts the project (current folder) declares and their state
  nuclear contracts update [<name>...]                        fetch each contract (a url: GET <url>/.nuclear/contract.json; a path:
                                                              its .nuclear/contract.json) and regenerate contracts/<name>/*.nuc
  nuclear defs list [--dialect <name>]                        list the type definitions packages (all, or the dialect's)
  nuclear defs show <Name|Name.member> --dialect <name> [--json] [--inherited]
                                                              print a declaration of the type definitions
  nuclear defs search <text> --dialect <name>                 find declarations whose name contains <text>

options:
  --dialect-path <dir>   extra directory with dialects (repeatable; also $NUCLEAR_DIALECT_PATH, ':'-separated)
  --defs-path <dir>      extra type definitions repository (repeatable; also $NUCLEAR_DEFS_PATH); the defs/
                         folder next to each dialects/ folder is always searched";

/// The documentation, embedded in the binary: (topic, what it is, text). The guide chapters first.
const TOPICS: &[(&str, &str, &str)] = &[
    ("guide", "the language guide: index of the chapters", include_str!("../../../docs/language/README.md")),
    ("syntax", "guide 1: files, indentation, comments, names", include_str!("../../../docs/language/01-syntax.md")),
    ("variables", "guide 2: variables, const or let, scope", include_str!("../../../docs/language/02-variables.md")),
    ("types", "guide 3: types, lists, maps, nullable", include_str!("../../../docs/language/03-types.md")),
    (
        "functions",
        "guide 4: functions, parameters, return, async, alias",
        include_str!("../../../docs/language/04-functions.md"),
    ),
    ("lambdas", "guide 5: lambdas", include_str!("../../../docs/language/05-lambdas.md")),
    ("modules", "guide 6: modules and constructors", include_str!("../../../docs/language/06-modules.md")),
    (
        "inheritance",
        "guide 7: inheritance, interfaces, override, super",
        include_str!("../../../docs/language/07-inheritance.md"),
    ),
    ("generics", "guide 8: generics", include_str!("../../../docs/language/08-generics.md")),
    (
        "expressions",
        "guide 9: operators, strings, cascades, destructuring",
        include_str!("../../../docs/language/09-expressions.md"),
    ),
    (
        "control",
        "guide 10: if, match, for, while (if and match are values)",
        include_str!("../../../docs/language/10-control.md"),
    ),
    ("use", "guide 11: use (imports)", include_str!("../../../docs/language/11-use.md")),
    ("errors", "guide 12: try, catch, throw", include_str!("../../../docs/language/12-errors.md")),
    (
        "results",
        "guide 13: Result<E, T>, Ok, Err, automatic propagation",
        include_str!("../../../docs/language/13-results.md"),
    ),
    (
        "operations",
        "guide 14: math, bits, text, regex, collections, types, bytes, dates and async in each target",
        include_str!("../../../docs/language/14-operations.md"),
    ),
    (
        "reference",
        "the whole base language in one page, for quick lookup",
        include_str!("../../../docs/language/reference.md"),
    ),
    ("base", "spec: every rule of the base language", include_str!("../../../spec/nuclear-base.md")),
    ("library", "spec: the standard library of Nuclear (strings, lists, maps, Math, Json, Regex, DateTime...)", include_str!("../../../spec/library.md")),
    ("grammar", "spec: the grammar (EBNF)", include_str!("../../../spec/grammar.ebnf")),
    ("keywords", "spec: reserved words", include_str!("../../../spec/reserved-words.md")),
    ("diagnostics", "spec: every error code (N0001...)", include_str!("../../../spec/diagnostics.md")),
    ("projects", "spec: projects of several files", include_str!("../../../spec/projects.md")),
    ("dialects", "spec: the dialect format (TOML + templates)", include_str!("../../../spec/dialects.md")),
    ("defs", "spec: type definitions packages", include_str!("../../../spec/definitions.md")),
    ("typescript", "dialect typescript", include_str!("../../../spec/typescript.md")),
    ("nodejs", "dialect nodejs", include_str!("../../../spec/nodejs.md")),
    ("dart", "dialect dart", include_str!("../../../spec/dart.md")),
    ("python", "dialect python (Python 3.12+, typed)", include_str!("../../../spec/python.md")),
    ("decisions", "every design decision and why", include_str!("../../../spec/decisions.md")),
];

/// The documents of the platforms and their dialects (plan 006: they live in the platform repositories), found when
/// asked: every `spec/<name>.md` of the working tree or the installation that is not embedded (the language's), and
/// every guide `docs/dragonfly/<name>.md` whose name is free. (topic, its title, its path)
fn platform_topics() -> Vec<(String, String, PathBuf)> {
    let mut found: Vec<(String, String, PathBuf)> = Vec::new();
    for dir in ["spec", "docs/dragonfly"] {
        for root in doc_roots() {
            for entry in std::fs::read_dir(root.join(dir)).into_iter().flatten().flatten() {
                let path = entry.path();
                let Some(name) = path.file_stem().map(|s| s.to_string_lossy().to_string()) else { continue };
                if path.extension().is_none_or(|e| e != "md") || name == "README" {
                    continue;
                }
                if TOPICS.iter().any(|(t, _, _)| *t == name) || found.iter().any(|(t, _, _)| *t == name) {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                let title = text.lines().find_map(|l| l.strip_prefix("# ")).unwrap_or(&name).replace('`', "");
                found.push((name, title, path));
            }
        }
    }
    found.sort();
    found
}

/// Where the documents of `platform_topics`, the dialects' specs and briefs are: the installation (which copies them
/// from every repository) and the source roots of this working tree.
fn doc_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = nuclear_dialect::install_dir().into_iter().collect();
    roots.extend(nuclear_dialect::source_roots(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    roots
}

fn read_doc(rel: &str) -> Option<String> {
    doc_roots().iter().find_map(|r| std::fs::read_to_string(r.join(rel)).ok())
}

/// `nuclear help`: what nuclear is, its commands, the documentation topics and the dialects.
fn help(l: &Loader) -> String {
    let mut out = String::from(
        "NuclearLang: a minimal language for AI agents, transpiled to other technologies through declarative \
         dialects (.nuc → .ts / .dart).\n\n",
    );
    out.push_str(USAGE);
    out.push_str("\n\ndocumentation: nuclear help <topic>\n");
    for (name, what, _) in TOPICS {
        out.push_str(&format!("  {name:<13} {what}\n"));
    }
    for (name, what, _) in platform_topics() {
        out.push_str(&format!("  {name:<13} {what}\n"));
    }
    out.push_str("\ndialects: nuclear help <dialect> · nuclear spec --dialect <name> (everything, for agents)\n");
    for name in l.available() {
        if let (Some(d), _) = l.load(&name) {
            out.push_str(&format!("  {name:<13} {} — {}\n", d.chain.join(" → "), d.description));
        }
    }
    out.push_str("\nFlutter apps (projects, generators, VS Code): the dragonfly CLI, dragonfly help\n");
    out
}

/// `nuclear help <topic|command>`: a documentation topic, or the usage lines of a command.
const DOC_SECTIONS: &[&str] = &["kinds", "decorators", "words"];

/// `nuclear help <word> --dialect <d>`: the editor documentation of a kind, decorator or header word of the
/// dialect (docs.toml of its chain); `words` lists them all with their first line.
fn dialect_word(l: &Loader, dialect: &str, word: &str) -> Result<String, String> {
    let (Some(d), _) = l.load(dialect) else { return Err(format!("dialect '{dialect}' not found")) };
    let word = word.trim_start_matches('@');
    if word == "layout" {
        if d.layout.is_empty() {
            return Err(format!("dialect '{dialect}' has no platform layout (only the dialects of a platform have one)"));
        }
        let mut out = format!(
            "where each kind's file goes in a {} project ({{component}}, {{screen}}: folders; {{file}}: the name in snake_case)\n\n",
            d.platform.as_deref().unwrap_or(dialect)
        );
        for (kind, path) in &d.layout {
            out.push_str(&format!("  {kind:<14} {path}\n"));
        }
        return Ok(out);
    }
    let names = |section: &str| d.doc_names(section);
    if word == "words" {
        let mut out = format!("dialect {dialect}: nuclear help <word> --dialect {dialect}\n");
        for section in DOC_SECTIONS {
            let found: Vec<String> = names(section)
                .into_iter()
                .filter_map(|n| {
                    let doc = d.doc(section, &n)?;
                    let first = doc.lines().next().unwrap_or("").replace("**", "");
                    let at = if *section == "decorators" { "@" } else { "" };
                    Some(format!("  {at}{n:<18} {first}"))
                })
                .collect();
            if !found.is_empty() {
                out.push_str(&format!("\n{section}:\n{}\n", found.join("\n")));
            }
        }
        return Ok(out);
    }
    for section in DOC_SECTIONS {
        if let Some(doc) = d.doc(section, word) {
            return Ok(format!("{doc}\n"));
        }
    }
    if let Some(text) = help_topic(word) {
        return Ok(text);
    }
    Err(format!("'{word}' is not a documented word of dialect '{dialect}'; nuclear help words --dialect {dialect}"))
}

fn help_topic(topic: &str) -> Option<String> {
    if let Some((_, _, text)) = TOPICS.iter().find(|(name, _, _)| *name == topic) {
        return Some(text.to_string());
    }
    if let Some(text) = platform_topics().into_iter().find(|(name, _, _)| name == topic).and_then(|(_, _, p)| std::fs::read_to_string(p).ok()) {
        return Some(text);
    }
    let prefix = format!("  nuclear {topic} ");
    let lines: Vec<&str> = USAGE.lines().filter(|l| l.starts_with(&prefix)).collect();
    (!lines.is_empty()).then(|| format!("usage:\n{}\n", lines.join("\n")))
}

struct Args {
    positional: Vec<String>,
    json: bool,
    dialect: Option<String>,
    platform: Option<String>,
    out: Option<String>,
    dialect_paths: Vec<PathBuf>,
    defs_paths: Vec<PathBuf>,
    inherited: bool,
    brief: bool,
    check: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        positional: Vec::new(),
        json: false,
        dialect: None,
        platform: None,
        out: None,
        dialect_paths: Vec::new(),
        defs_paths: Vec::new(),
        inherited: false,
        brief: false,
        check: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => a.json = true,
            "--dialect" | "-d" => a.dialect = Some(it.next().ok_or("--dialect needs a value")?),
            "--platform" | "-p" => a.platform = Some(it.next().ok_or("--platform needs a value")?),
            "--out" | "-o" => a.out = Some(it.next().ok_or("-o needs a value")?),
            "--dialect-path" => a.dialect_paths.push(it.next().ok_or("--dialect-path needs a value")?.into()),
            "--defs-path" => a.defs_paths.push(it.next().ok_or("--defs-path needs a value")?.into()),
            "--inherited" => a.inherited = true,
            "--brief" => a.brief = true,
            "--check" => a.check = true,
            "-h" | "--help" => a.positional.insert(0, "help".into()),
            s if s.starts_with('-') => return Err(format!("unknown option '{s}'")),
            _ => a.positional.push(arg),
        }
    }
    Ok(a)
}

fn loader(args: &Args) -> Loader {
    let mut paths = args.dialect_paths.clone();
    if let Ok(env) = std::env::var("NUCLEAR_DIALECT_PATH") {
        paths.extend(env.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    paths.push(PathBuf::from("dialects"));
    paths.extend(nuclear_dialect::install_dir().map(|d| d.join("dialects")));
    paths.extend(nuclear_dialect::dialect_dirs(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    let mut l = Loader::new(paths);
    // kind packages (ADR-0095): the project's and the installed ones
    l.packages.push(PathBuf::from("kinds"));
    l.packages.extend(nuclear_dialect::install_dir().map(|d| d.join("kinds")));
    l.defs_paths = args.defs_paths.clone();
    if let Ok(env) = std::env::var("NUCLEAR_DEFS_PATH") {
        l.defs_paths.extend(env.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    l
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("{e}\n");
            }
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut args = args;
    // `--platform backend` alone: the platform's default dialect (an agent writes for a platform, ADR-0093); `kind
    // new` takes both
    if args.dialect.is_none()
        && args.positional.first().map(String::as_str) != Some("kind")
        && let Some(p) = args.platform.clone()
    {
        match loader(&args).default_dialect(&p) {
            Some(d) => args.dialect = Some(d),
            None => {
                eprintln!("platform '{p}' not found (platforms: backend, mobile, web)");
                return ExitCode::from(2);
            }
        }
    }
    let pos: Vec<&str> = args.positional.iter().map(String::as_str).collect();
    match pos.as_slice() {
        ["help", ..] if pos.len() <= 2 => match pos.get(1) {
            None => {
                print!("{}", help(&loader(&args)));
                ExitCode::SUCCESS
            }
            Some(topic) if args.dialect.is_some() => match dialect_word(&loader(&args), args.dialect.as_deref().unwrap(), topic) {
                Ok(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::from(2)
                }
            },
            Some(topic) => match help_topic(topic) {
                Some(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("unknown topic or command '{topic}'\n");
                    eprint!("{}", help(&loader(&args)));
                    ExitCode::from(2)
                }
            },
        },
        ["spec"] if args.brief => match brief(&loader(&args), args.dialect.as_deref(), args.platform.is_some()) {
            Ok(text) => {
                print!("{text}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        ["spec"] => match spec(&loader(&args), args.dialect.as_deref()) {
            Ok(text) => {
                print!("{text}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        ["dialect", "list"] => {
            let l = loader(&args);
            for name in l.available() {
                let (d, _) = l.load(&name);
                match d {
                    Some(d) => {
                        println!("{:<12} {:<28} {}", name, d.chain.join(" → "), d.description)
                    }
                    None => println!("{name:<12} (has errors: nuclear dialect check {name})"),
                }
            }
            ExitCode::SUCCESS
        }
        ["dialect", "check", name] => dialect_check(&args, name),
        ["kind", "new", name] => match kind_new(name, args.platform.as_deref(), args.dialect.as_deref()) {
            Ok(path) => {
                println!("✓ kind package {name} created in {}: edit its kind.toml, then plugins({name} = true) in config.nuc", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        ["agent", "init"] => match agent::init(std::path::Path::new("."), args.dialect.as_deref(), loader(&args).default_dialect("mobile")) {
            Ok(done) => {
                for d in done {
                    println!("✓ {d}");
                }
                println!("the skill for the agents is installed by install.sh (~/.claude/skills, ~/.agents/skills)");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        ["agent", "hook"] => agent::hook(&loader(&args)),
        ["contracts", "update", rest @ ..] => contracts_update(&args, rest),
        ["contracts", "list"] => contracts_list(&args),
        ["defs", "list"] => defs_list(&args),
        ["defs", "show", name] => defs_show(&args, name),
        ["defs", "search", text] => defs_search(&args, text),
        ["fmt", paths @ ..] if !paths.is_empty() => fmt_command(&args, paths),
        [cmd @ ("build" | "check" | "fix" | "tokens" | "ast"), path] => file_command(&args, cmd, path),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Prints a dialect report; returns true when it has errors.
fn print_report(report: &Report, json: bool) -> bool {
    if json {
        let mut items = Vec::new();
        for (i, f) in report.files.iter().enumerate() {
            let diags: Vec<Diagnostic> =
                report.diags.iter().filter(|(fi, _)| *fi == i).map(|(_, d)| d.clone()).collect();
            items.extend(json_items(&diags, f));
        }
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        for (i, f) in report.files.iter().enumerate() {
            let diags: Vec<Diagnostic> =
                report.diags.iter().filter(|(fi, _)| *fi == i).map(|(_, d)| d.clone()).collect();
            if !diags.is_empty() {
                eprint!("{}", render_text(&diags, f));
            }
        }
    }
    report.has_errors()
}

fn dialect_check(args: &Args, name: &str) -> ExitCode {
    let (d, mut report) = loader(args).load(name);
    if let Some(d) = &d {
        let missing = d.missing_syntax();
        if !missing.is_empty() {
            let idx = d.files.iter().position(|f| f.name.contains(&d.name)).unwrap_or(0);
            if report.files.is_empty() {
                report.files.push(SourceFile::new(&d.name, ""));
            }
            let msg = format!("dialect '{}' does not define these [syntax] templates: {}", d.name, missing.join(", "));
            report.diags.push((
                idx.min(report.files.len() - 1),
                Diagnostic::error(nuclear_diag::codes::MISSING_TEMPLATE, Default::default(), msg),
            ));
        }
    }
    let errors = print_report(&report, args.json);
    if !args.json {
        match &d {
            Some(d) if !errors => eprintln!(
                "dialect '{}' ({}) is valid: {} kinds, {} import shortcuts, {} auto-imports, {} function shortcuts",
                d.name,
                d.chain.join(" → "),
                d.kinds.len(),
                d.imports.len(),
                d.auto_imports.len(),
                d.functions.len()
            ),
            _ => {}
        }
    }
    if errors { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn load_dialect(args: &Args, name: &str) -> Option<Dialect> {
    let (d, report) = loader(args).load(name);
    if print_report(&report, args.json) {
        return None;
    }
    d
}

/// `nuclear fmt`: formats every `.nuc` of the paths (folders recursively, without build output and hidden folders).
fn fmt_command(args: &Args, paths: &[&str]) -> ExitCode {
    fn collect(p: &std::path::Path, files: &mut Vec<PathBuf>) {
        if p.is_file() {
            files.push(p.to_path_buf());
            return;
        }
        let Ok(entries) = std::fs::read_dir(p) else { return };
        let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        for e in entries {
            let name = e.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if e.is_dir() {
                if !name.starts_with('.') && !matches!(name, "build" | "node_modules" | "target") {
                    collect(&e, files);
                }
            } else if name.ends_with(".nuc") {
                files.push(e);
            }
        }
    }
    let mut files = Vec::new();
    for p in paths {
        collect(std::path::Path::new(p), &mut files);
    }
    let (mut changed, mut failed) = (0, 0);
    for f in &files {
        let text = match std::fs::read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("could not read {}: {e}", f.display());
                failed += 1;
                continue;
            }
        };
        match nuclear_syntax::fmt::format(&text) {
            Ok(out) if out == text => {}
            Ok(out) => {
                changed += 1;
                if args.check {
                    println!("{}", f.display());
                } else if let Err(e) = std::fs::write(f, out) {
                    eprintln!("could not write {}: {e}", f.display());
                    failed += 1;
                }
            }
            Err(e) => {
                eprintln!("{}: {e}", f.display());
                failed += 1;
            }
        }
    }
    if !args.check {
        eprintln!("{} files, {changed} formatted", files.len());
    }
    if failed > 0 || (args.check && changed > 0) { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// The source with the exact fixes of its errors applied (from the end, skipping overlapping ones), and how many;
/// `None` when there is none.
fn apply_fixes(source: &str, diags: &[Diagnostic]) -> Option<(String, usize)> {
    let mut fixes: Vec<&nuclear_diag::Fix> = diags.iter().filter(|d| d.is_error()).filter_map(|d| d.fix.as_ref()).collect();
    fixes.sort_by_key(|f| std::cmp::Reverse(f.span.start));
    let mut text = source.to_string();
    let mut last = usize::MAX;
    let mut n = 0;
    for f in fixes {
        if f.span.end > last || f.span.end > text.len() || !text.is_char_boundary(f.span.start) || !text.is_char_boundary(f.span.end) {
            continue;
        }
        text.replace_range(f.span.start..f.span.end, &f.replacement);
        last = f.span.start;
        n += 1;
    }
    (n > 0).then_some((text, n))
}

fn file_command(args: &Args, cmd: &str, path: &str) -> ExitCode {
    if matches!(cmd, "build" | "check" | "fix") && std::path::Path::new(path).is_dir() {
        return project_command(args, cmd, path);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let file = SourceFile::new(path, text);
    match cmd {
        "tokens" => {
            let lexed = nuclear_syntax::lex(&file.text);
            for t in &lexed.tokens {
                let (line, col) = file.line_col(t.span.start);
                println!("{:<8} {}", format!("{line}:{col}"), t.kind);
            }
            report(&lexed.diagnostics, &file, false)
        }
        "ast" => {
            let parsed = nuclear_syntax::parse(&file.text);
            println!("{:#?}", parsed.file.items);
            report(&parsed.diagnostics, &file, false)
        }
        "fix" => {
            let Some(dialect) = args.dialect.as_deref().and_then(|n| load_dialect(args, n)) else {
                eprintln!("fix of one file needs --dialect <name> or --platform <p>");
                return ExitCode::from(2);
            };
            let mut text = file.text.clone();
            let mut applied = 0;
            for _ in 0..5 {
                let diags = nuclear_transpile::check(&text, Some(&dialect));
                let Some((next, n)) = apply_fixes(&text, &diags) else { break };
                text = next;
                applied += n;
            }
            if applied > 0 && let Err(e) = std::fs::write(path, &text) {
                eprintln!("could not write {path}: {e}");
                return ExitCode::from(2);
            }
            println!("{path}: {applied} fix(es) applied");
            let left = nuclear_transpile::check(&text, Some(&dialect));
            report(&left, &SourceFile::new(path, text), args.json)
        }
        "check" => {
            let dialect = match &args.dialect {
                Some(name) => match load_dialect(args, name) {
                    Some(d) => Some(d),
                    None => return ExitCode::FAILURE,
                },
                None => None,
            };
            let diags = nuclear_transpile::check(&file.text, dialect.as_ref());
            if !args.json && diags.is_empty() {
                println!("{path}: no errors");
                return ExitCode::SUCCESS;
            }
            report(&diags, &file, args.json)
        }
        _ => {
            let Some(name) = &args.dialect else {
                eprintln!("build needs --dialect <name> (see: nuclear dialect list)");
                return ExitCode::from(2);
            };
            let Some(dialect) = load_dialect(args, name) else {
                return ExitCode::FAILURE;
            };
            let out = nuclear_transpile::transpile(&file.text, &dialect);
            let code = report(&out.diagnostics, &file, args.json);
            if out.diagnostics.iter().any(Diagnostic::is_error) {
                return code;
            }
            match &args.out {
                Some(o) => {
                    if let Err(e) = std::fs::write(o, &out.code) {
                        eprintln!("could not write {o}: {e}");
                        return ExitCode::from(2);
                    }
                }
                None => print!("{}", out.code),
            }
            ExitCode::SUCCESS
        }
    }
}

/// The project of the current folder and its dialect (for `nuclear contracts`).
fn current_project(args: &Args) -> Option<(PathBuf, nuclear_transpile::project::Manifest, nuclear_dialect::Dialect)> {
    use nuclear_transpile::project;
    let root = PathBuf::from(".");
    let manifest = match project::Manifest::read(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error[N0303]: {e}");
            return None;
        }
    };
    let dialect = load_dialect(args, &manifest.dialect.clone())?;
    Some((root, manifest, dialect))
}

/// `nuclear contracts update [name...]`: fetches the declared contracts and regenerates `contracts/<name>/`.
fn contracts_update(args: &Args, only: &[&str]) -> ExitCode {
    use nuclear_transpile::contract;
    let Some((root, _, dialect)) = current_project(args) else { return ExitCode::FAILURE };
    let declared = contract::declared(&root, &dialect);
    if declared.is_empty() {
        eprintln!("the manifest declares no contracts: add contracts(<name> = \"../backend\" | \"https://api...\")");
        return ExitCode::FAILURE;
    }
    let mut code = ExitCode::SUCCESS;
    for (name, source) in declared {
        if !only.is_empty() && !only.contains(&name.as_str()) {
            continue;
        }
        let text = match contract::fetch(&source, &root) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error[N0307]: contract '{name}': {e}");
                code = ExitCode::FAILURE;
                continue;
            }
        };
        let value: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error[N0307]: contract '{name}': invalid JSON: {e}");
                code = ExitCode::FAILURE;
                continue;
            }
        };
        match contract::install(&root, &name, &value, &dialect) {
            Ok(()) => println!(
                "✓ contract {name} {} from {source} → {}/{name}/",
                value["version"].as_str().unwrap_or(""),
                contract::CONTRACTS_DIR
            ),
            Err(e) => {
                eprintln!("error[N0307]: contract '{name}': {e}");
                code = ExitCode::FAILURE;
            }
        }
    }
    code
}

/// `nuclear contracts list`: each declared contract, its source and the version of the local copy.
fn contracts_list(args: &Args) -> ExitCode {
    use nuclear_transpile::contract;
    let Some((root, _, dialect)) = current_project(args) else { return ExitCode::FAILURE };
    let declared = contract::declared(&root, &dialect);
    if declared.is_empty() {
        println!("no contracts declared (contracts(<name> = ...) in the manifest)");
        return ExitCode::SUCCESS;
    }
    for (name, source) in declared {
        let local = root.join(contract::CONTRACTS_DIR).join(&name).join("contract.json");
        let version = std::fs::read_to_string(&local)
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| v["version"].as_str().map(String::from));
        match version {
            Some(v) => println!("{name:<20} {v:<10} {source}"),
            None => println!("{name:<20} {:<10} {source}  (not fetched: nuclear contracts update)", "-"),
        }
    }
    ExitCode::SUCCESS
}

/// `nuclear build <folder>` / `nuclear check <folder>`: a project with a nuclear.toml.
fn project_command(args: &Args, cmd: &str, path: &str) -> ExitCode {
    use nuclear_transpile::project;
    let root = PathBuf::from(path);
    let manifest = match project::Manifest::read(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error[N0303]: {e}");
            return ExitCode::FAILURE;
        }
    };
    let name = args.dialect.clone().unwrap_or_else(|| manifest.dialect.clone());
    let Some(dialect) = load_dialect(args, &name) else { return ExitCode::FAILURE };
    // the project's own type definitions (`<project>/defs`, `build/defs.lock`) on top of the dialect's
    let (own, missing) = project::with_project_defs(&root, &dialect);
    for d in &missing {
        eprintln!("warning[{}]: {}", d.code.id, d.message);
    }
    let dialect = own.unwrap_or(dialect);
    if cmd == "fix" {
        // every exact fix the compiler gives (the ones the editor offers), applied until none is left
        let mut applied = 0;
        for _ in 0..5 {
            let out = project::build(&root, &manifest, &dialect);
            let mut n = 0;
            for f in &out.files {
                if let Some((text, k)) = apply_fixes(&f.source, &f.diagnostics)
                    && std::fs::write(root.join(&f.source_path), text).is_ok()
                {
                    n += k;
                }
            }
            if n == 0 {
                break;
            }
            applied += n;
        }
        eprintln!("{path}: {applied} fix(es) applied");
        return project_command(args, "check", path);
    }
    let out = project::build(&root, &manifest, &dialect);
    if let Err(e) = project::write_metadata(&root, &manifest, &dialect) {
        eprintln!("warning: cannot write {}: {e}", project::METADATA_FILE);
    }
    let mut items = Vec::new();
    for f in &out.files {
        let file = SourceFile::new(f.source_path.display().to_string(), f.source.clone());
        if args.json {
            items.extend(json_items(&f.diagnostics, &file));
        } else if !f.diagnostics.is_empty() {
            eprint!("{}", render_text(&f.diagnostics, &file));
        }
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    }
    if out.has_errors() {
        return ExitCode::FAILURE;
    }
    if cmd == "check" {
        if !args.json {
            println!("{path}: {} files, no errors", out.files.len());
        }
        return ExitCode::SUCCESS;
    }
    let out_dir = args.out.as_ref().map(PathBuf::from).unwrap_or_else(|| root.join(project::output_dir(&manifest, &dialect)));
    let files = out.files.iter().filter(|f| !f.out_path.is_empty()).map(|f| (f.out_path.as_str(), f.code.as_str()));
    let generated = out.generated.iter().map(|(p, c)| (p.as_str(), c.as_str()));
    for (rel, code) in files.chain(generated) {
        let target = out_dir.join(rel);
        if let Some(parent) = target.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            eprintln!("could not create {}: {e}", parent.display());
            return ExitCode::from(2);
        }
        if let Err(e) = std::fs::write(&target, code) {
            eprintln!("could not write {}: {e}", target.display());
            return ExitCode::from(2);
        }
    }
    for (rel, source) in &out.copies {
        let target = out_dir.join(rel);
        let copied =
            target.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::copy(source, &target));
        if let Err(e) = copied {
            eprintln!("could not copy {}: {e}", source.display());
            return ExitCode::from(2);
        }
    }
    // the line map of the editors' debuggers (a breakpoint in a .nuc, the .nuc line of a paused frame); the previous
    // one lists the files the last build wrote from a .nuc: those this build no longer writes (their .nuc was removed
    // or renamed) are removed too
    let map = out_dir.join(project::LINE_MAP_FILE);
    let text = project::line_map(&out);
    if let Some(old) = std::fs::read_to_string(&map).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) {
        let now: std::collections::HashSet<&str> = out.files.iter().map(|f| f.out_path.as_str()).collect();
        for stale in old["files"].as_object().into_iter().flat_map(|m| m.keys()).filter(|k| !now.contains(k.as_str())) {
            let _ = std::fs::remove_file(out_dir.join(stale));
        }
    }
    if std::fs::read_to_string(&map).ok().as_deref() != Some(text.as_str()) {
        let written = map.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&map, &text));
        if let Err(e) = written {
            eprintln!("could not write {}: {e}", map.display());
            return ExitCode::from(2);
        }
    }
    if !args.json {
        eprintln!("{}: {} files → {}", manifest.name, out.files.len() + out.generated.len(), out_dir.display());
    }
    ExitCode::SUCCESS
}

fn report(diags: &[Diagnostic], file: &SourceFile, json: bool) -> ExitCode {
    if json {
        println!("{}", serde_json::to_string_pretty(&json_items(diags, file)).unwrap());
    } else if !diags.is_empty() {
        eprint!("{}", render_text(diags, file));
    }
    if diags.iter().any(Diagnostic::is_error) { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// Everything an agent needs to write NuclearLang code or a new dialect. With a dialect, its spec and the
/// specs of the dialects it extends follow the base (base → dart → flutter), so the agent also gets the
/// Nuclear → target equivalences.
fn spec(l: &Loader, dialect: Option<&str>) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(include_str!("../../../spec/nuclear-base.md"));
    if let Some(name) = dialect {
        let (d, _) = l.load(name);
        let d = d.ok_or_else(|| format!("dialect '{name}' not found or invalid (nuclear dialect check {name})"))?;
        for n in d.chain.iter().rev() {
            match dialect_spec(&d, n) {
                Some(text) => {
                    out.push_str("\n\n---\n\n");
                    out.push_str(&text);
                }
                None => out.push_str(&format!(
                    "\n\n---\n\n# Dialecto `{n}`\n\n(no spec file: spec/{n}.md or dialects/{n}/spec.md)\n"
                )),
            }
        }
    }
    out.push_str("\n\n---\n\n");
    out.push_str(include_str!("../../../spec/dialects.md"));
    out.push_str("\n\n---\n\n## Contract (generated from the transpiler)\n\n");
    out.push_str(&nuclear_dialect::contract::markdown());
    out.push_str("\n## Available dialects\n\n");
    for name in l.available() {
        if let (Some(d), _) = l.load(&name) {
            out.push_str(&format!("- `{name}` ({}): {}\n", d.chain.join(" → "), d.description));
        }
    }
    Ok(out)
}

/// The short sheets for agents (spec/brief/): what changes compared with TypeScript/Dart, per dialect.
const BRIEFS: &[(&str, &str)] = &[
    ("typescript", include_str!("../../../spec/brief/typescript.md")),
    ("dart", include_str!("../../../spec/brief/dart.md")),
    ("python", include_str!("../../../spec/brief/python.md")),
];

/// `nuclear spec --brief`: the base brief, the brief of the dialect's platform (`spec/brief/<platform>.md`) and the
/// brief of each dialect of the chain (a dialect outside NuclearLang may bring its own `brief.md` in its folder).
/// `platform_only` (`--platform`): the base and the platform's sheet, nothing of the language underneath — what an agent
/// that writes only Nuclear reads.
fn brief(l: &Loader, dialect: Option<&str>, platform_only: bool) -> Result<String, String> {
    let mut out = String::from(include_str!("../../../spec/brief/nuclear-base.md"));
    let Some(name) = dialect else {
        out.push_str("
Dialects: ");
        out.push_str(&l.available().join(", "));
        out.push_str(" (`nuclear spec --brief --dialect <name>` adds the dialect's sheet).
");
        return Ok(out);
    };
    let (d, _) = l.load(name);
    let d = d.ok_or_else(|| format!("dialect '{name}' not found or invalid (nuclear dialect check {name})"))?;
    if let Some(text) = d.platform.as_ref().and_then(|p| read_doc(&format!("spec/brief/{p}.md"))) {
        out.push_str("\n---\n\n");
        out.push_str(&text);
    }
    if platform_only {
        return Ok(out);
    }
    for n in d.chain.iter().rev() {
        let own = d.files.iter().find_map(|f| {
            let dir = PathBuf::from(&f.name).parent()?.to_path_buf();
            (dir.file_name()? == n.as_str()).then(|| std::fs::read_to_string(dir.join("brief.md")).ok())?
        });
        let shared = || BRIEFS.iter().find(|(b, _)| b == n).map(|(_, t)| t.to_string());
        // a platform dialect's sheet is in its platform repository: spec/brief/<name>.md
        if let Some(text) = own.or_else(shared).or_else(|| read_doc(&format!("spec/brief/{n}.md"))) {
            out.push_str("
---

");
            out.push_str(&text);
        }
    }
    Ok(out)
}

/// The spec of one dialect of the chain: `spec.md` in its folder, or `spec/<name>.md` of NuclearLang.
fn dialect_spec(d: &Dialect, name: &str) -> Option<String> {
    let folder = d.files.iter().find_map(|f| {
        let dir = PathBuf::from(&f.name).parent()?.to_path_buf();
        (dir.file_name()? == name).then_some(dir)
    });
    folder
        .map(|f| f.join("spec.md"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .or_else(|| read_doc(&format!("spec/{name}.md")))
}

/// The dialect of `--dialect`, required by the `defs` commands that read declarations.
fn defs_dialect(args: &Args) -> Option<Dialect> {
    match args.dialect.as_deref() {
        Some(name) => load_dialect(args, name),
        None => {
            eprintln!("this command needs --dialect <name>: the dialect decides which packages are used");
            None
        }
    }
}

fn defs_list(args: &Args) -> ExitCode {
    let packages = match args.dialect.as_deref() {
        Some(name) => match load_dialect(args, name) {
            Some(d) => d.typedefs.packages.clone(),
            None => return ExitCode::FAILURE,
        },
        None => loader(args).repository().list(),
    };
    if args.json {
        let items: Vec<_> = packages
            .iter()
            .map(|p| {
                serde_json::json!({
                    "language": p.language, "name": p.name, "version": p.version,
                    "description": p.description, "depends": p.depends, "path": p.dir.display().to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        for p in &packages {
            println!("{:<6} {:<24} {}", p.language, p.id(), p.description);
        }
    }
    ExitCode::SUCCESS
}

fn defs_show(args: &Args, name: &str) -> ExitCode {
    use nuclear_dialect::defs;
    let Some(d) = defs_dialect(args) else { return ExitCode::FAILURE };
    let td = &d.typedefs;
    let (class, member) = match name.split_once('.') {
        Some((c, m)) => (c, Some(m)),
        None => (name, None),
    };
    let Some((package, decl)) = td.get(class) else {
        let similar: Vec<&str> = td
            .declarations()
            .map(|(_, x)| x.name.as_str())
            .filter(|n| nuclear_dialect::model::similar(n, class) || n.eq_ignore_ascii_case(class))
            .take(5)
            .collect();
        let hint = if similar.is_empty() { String::new() } else { format!("; did you mean {}?", similar.join(", ")) };
        eprintln!("'{class}' is not in the type definitions of '{}'{hint}", d.name);
        return ExitCode::FAILURE;
    };
    match member {
        None if args.json => println!("{}", serde_json::to_string_pretty(decl).unwrap()),
        None => {
            print!("{}", defs::render(package, decl));
            if args.inherited {
                for (owner, m) in td.members(class).into_iter().filter(|(o, _)| o.name != decl.name) {
                    println!("  {}  // from {}", defs::render_member(m, "  "), owner.name);
                }
                for ext in td.extensions_on(class) {
                    for m in &ext.members {
                        println!("  {}  // extension {}", defs::render_member(m, "  "), ext.name);
                    }
                }
            }
        }
        Some(m) => {
            if let Some(c) = decl.constructors.iter().find(|c| c.name == m) {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(c).unwrap());
                } else {
                    if let Some(doc) = &c.doc {
                        println!("// {doc}");
                    }
                    println!("{}", defs::render_constructor(class, c, ""));
                }
                return ExitCode::SUCCESS;
            }
            let Some((owner, found)) = td.members(class).into_iter().find(|(_, x)| x.name == m) else {
                eprintln!("'{class}' has no member or constructor '{m}'");
                return ExitCode::FAILURE;
            };
            if args.json {
                println!("{}", serde_json::to_string_pretty(found).unwrap());
            } else {
                println!("// {}.{m}", owner.name);
                if let Some(doc) = &found.doc {
                    println!("// {doc}");
                }
                println!("{}", defs::render_member(found, ""));
            }
        }
    }
    ExitCode::SUCCESS
}

fn defs_search(args: &Args, text: &str) -> ExitCode {
    let Some(d) = defs_dialect(args) else { return ExitCode::FAILURE };
    let needle = text.to_lowercase();
    for (p, decl) in d.typedefs.declarations().filter(|(_, x)| x.name.to_lowercase().contains(&needle)) {
        let lib = decl.library.as_deref().unwrap_or("");
        println!("{:<40} {:<10} {:<44} {}", decl.name, decl.kind, lib, p.id());
    }
    ExitCode::SUCCESS
}

/// `nuclear kind new <name> -p <platform> -d <dialect>`: the skeleton of a kind package (ADR-0095) in
/// `kinds/<name>/`: `kind.toml` (`[package]`), `docs.toml`, and `<dialect>/kind.toml` with one kind and its template.
fn kind_new(name: &str, platform: Option<&str>, dialect: Option<&str>) -> Result<PathBuf, String> {
    let platform = platform.ok_or("--platform <web|mobile|backend> is required (the package belongs to one platform)")?;
    let dialect = dialect.ok_or("--dialect <name> is required (the first dialect the package implements)")?;
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || name.is_empty() {
        return Err(format!("'{name}' must be snake_case (letters, digits, _)"));
    }
    let root = PathBuf::from("kinds").join(name);
    if root.exists() {
        return Err(format!("{} already exists", root.display()));
    }
    let kind = name.trim_end_matches('s');
    let templates = root.join(dialect).join("templates");
    std::fs::create_dir_all(&templates).map_err(|e| e.to_string())?;
    let write = |rel: &str, text: String| std::fs::write(root.join(rel), text).map_err(|e| e.to_string());
    write(
        "kind.toml",
        format!(
            "# A package of extension kinds (ADR-0095, spec/dialects.md §7): the platform it belongs to and one folder per\n# dialect it implements. Enable it in a project with plugins({name} = true).\n[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nplatform = \"{platform}\"\ndescription = \"What the kinds of this package are for.\"\n"
        ),
    )?;
    write(
        "docs.toml",
        format!("# Editor documentation of each word (spec/dialects.md §5.7).\n[kinds]\n{kind} = \"\"\"\n{kind} Name(deps) + methods → what it generates in the target.\n\"\"\"\n"),
    )?;
    write(
        &format!("{dialect}/kind.toml"),
        format!(
            "# How the {dialect} dialect writes the kinds of this package: the same sections as a dialect (kinds, decorators,\n# functions, auto_imports, library...), never [dialect], [project] or [syntax] (nuclear spec --dialect {dialect}).\n[plugin]\n# the runtime packages the kinds need in the target: packages = {{ some_package = \"^1.0\" }}\npackages = {{}}\n\n[kinds.{kind}]\ndescription = \"{kind} Name(deps) + methods → ...\"\ntemplate = {{ file = \"templates/{kind}.tpl\" }}\n"
        ),
    )?;
    // the first template: a class in the target's shape, from the dialect's own `[syntax] class` (indentation in
    // Python, braces elsewhere), with the kind's variables left for the template
    let l = Loader::new(nuclear_dialect::dialect_dirs(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))).into_iter().chain(nuclear_dialect::install_dir().map(|d| d.join("dialects"))).collect());
    let (d, _) = l.load(dialect);
    let d = d.ok_or_else(|| format!("unknown dialect '{dialect}' (nuclear dialect list)"))?;
    let vars = nuclear_dialect::template::Ctx::new().set("name", "{{name}}").set("members", "{{members}}").into_value();
    let template = d.syntax("class").map(|t| t.render(&vars)).ok_or_else(|| format!("dialect '{dialect}' has no [syntax] class"))?;
    write(&format!("{dialect}/templates/{kind}.tpl"), format!("{}\n", template.trim_end()))?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every guide chapter and every spec file of the language is embedded in TOPICS (a new one must be added there);
    /// every spec of a platform repository is a topic by itself.
    #[test]
    fn every_document_is_a_help_topic() {
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        for dir in ["docs/language", "spec"] {
            for entry in std::fs::read_dir(root.join(dir)).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(TOPICS.iter().any(|(_, _, t)| *t == text), "{} is not in TOPICS", path.display());
            }
        }
        let topics = platform_topics();
        for src in nuclear_dialect::source_roots(root).into_iter().skip(1) {
            for entry in std::fs::read_dir(src.join("spec")).into_iter().flatten().flatten() {
                let name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
                if entry.path().is_file() {
                    assert!(topics.iter().any(|(t, _, _)| *t == name), "{name} is not a topic");
                }
            }
        }
    }

    /// Every brief of spec/brief/ is embedded (the languages) or found in its source root (the platforms), and the
    /// flutter sheet stays short (≤ 4k tokens ≈ 16 KB).
    #[test]
    fn briefs_are_found_and_short() {
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        for src in nuclear_dialect::source_roots(root) {
            for entry in std::fs::read_dir(src.join("spec/brief")).into_iter().flatten() {
                let path = entry.unwrap().path();
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                if name != "nuclear-base" {
                    let found = BRIEFS.iter().any(|(b, _)| *b == name) || read_doc(&format!("spec/brief/{name}.md")).is_some();
                    assert!(found, "spec/brief/{name}.md is not found");
                }
            }
        }
        let l = Loader::new(nuclear_dialect::dialect_dirs(root));
        // the flutter dialect lives in dragonfly-mobile: checked only when it is in this working tree
        if let Ok(flutter) = brief(&l, Some("flutter"), false) {
            assert!(flutter.len() <= 16_000, "the flutter brief has {} bytes", flutter.len());
        }
        // an agent that writes only Nuclear gets the platform's sheet and nothing of the language underneath
        if let Ok(backend) = brief(&l, Some("fastapi"), true) {
            assert!(backend.contains("Platform `backend` brief") && !backend.contains("Dialect `python`") && !backend.contains("Dialect `fastapi`"), "{backend}");
        }
    }

    #[test]
    fn help_finds_topics_and_commands() {
        assert!(help_topic("results").unwrap().contains("Result"));
        assert!(help_topic("defs").unwrap().contains("nuclear defs show"));
        assert!(help_topic("build").unwrap().contains("nuclear build <folder>"));
        assert!(help_topic("nothing").is_none());
    }
}
