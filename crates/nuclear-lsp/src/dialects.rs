//! Finding and loading the active dialect, and short documentation of the base language words.

use nuclear_dialect::{Dialect, Loader};
use std::path::{Path, PathBuf};

/// Names that mean "no dialect": only the base language is checked.
pub fn is_base(name: &str) -> bool {
    matches!(name.trim(), "" | "base" | "none" | "nuclear")
}

/// Directories searched for dialects, in priority order:
/// the `nuclear.dialectPaths` setting, `$NUCLEAR_DIALECT_PATH`, `<workspace>/dialects` (and those of the platform
/// repositories inside the workspace, `<workspace>/dragonfly-*/dialects`), the dialects shipped
/// next to the server binary, the ones `install.sh` installed (`nuclear_dialect::install_dir`) and the
/// repository's `dialects/` (development builds).
pub fn search_paths(configured: &[String], roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for p in configured {
        let p = PathBuf::from(p);
        if p.is_absolute() {
            paths.push(p);
        } else {
            paths.extend(roots.iter().map(|r| r.join(&p)));
        }
    }
    if let Ok(env) = std::env::var("NUCLEAR_DIALECT_PATH") {
        paths.extend(env.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    paths.extend(roots.iter().flat_map(|r| nuclear_dialect::dialect_dirs(r)));
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        paths.push(dir.join("dialects"));
        paths.push(dir.join("../dialects"));
        paths.push(dir.join("../share/nuclear/dialects"));
    }
    paths.extend(nuclear_dialect::install_dir().map(|d| d.join("dialects")));
    paths.extend(nuclear_dialect::dialect_dirs(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    paths.retain(|p| p.is_dir());
    paths.dedup();
    paths
}

/// Loads `name` with the kind packages (ADR-0095) of `project` (`<project>/kinds`) and of the installation
/// (`<install>/kinds`). On failure returns the dialect report as plain text.
pub fn load(name: &str, paths: Vec<PathBuf>, project: Option<&Path>) -> Result<Dialect, String> {
    let mut loader = Loader::new(paths);
    loader.packages.extend(project.map(|p| p.join("kinds")));
    loader.packages.extend(nuclear_dialect::install_dir().map(|d| d.join("kinds")));
    let (dialect, report) = loader.load(name);
    match dialect {
        Some(d) => Ok(d),
        None => {
            let msgs: Vec<String> =
                report.diags.iter().map(|(_, d)| format!("[{}] {}", d.code.id, d.message)).collect();
            Err(if msgs.is_empty() { format!("dialect '{name}' could not be loaded") } else { msgs.join("; ") })
        }
    }
}

/// Hover text of a reserved word of the base language (`languages/docs.toml`).
pub fn keyword_doc(word: &str) -> Option<&'static str> {
    nuclear_dialect::base_doc("keywords", word)
}

/// Hover text of the names that exist without declaring them (base §3.2).
pub fn builtin_doc(name: &str) -> Option<&'static str> {
    nuclear_dialect::base_doc("builtins", name)
}

/// Decorators the dialect knows: `[decorators] consumed/void`, `[decorators.provide]` (`screen.providers` →
/// `screen`) and `[decorators.args]`, sorted and without repeats.
pub fn decorator_names(d: &Dialect) -> Vec<&str> {
    let mut names: Vec<&str> = (d.consumed_decorators.iter().chain(&d.void_decorators).map(String::as_str))
        .chain(d.provide.keys().filter_map(|k| k.split('.').next()))
        .chain(d.decorator_args.keys().map(String::as_str))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// `loggedIn.forward!` → `` `loggedIn.forward` (required) ``.
fn decorator_arg_label(arg: &str) -> String {
    match arg.strip_suffix('!') {
        Some(a) => format!("`{a}` (required)"),
        None => format!("`{arg}`"),
    }
}

/// Hover text of an argument (`path` = `loggedIn.forward`) of a decorator, from `[decorators.args]`.
pub fn decorator_arg_doc(decorator: &str, path: &str, d: &Dialect) -> Option<String> {
    let args = d.decorator_args.get(decorator)?;
    let arg = args.iter().find(|a| a.trim_end_matches('!') == path)?;
    let prefix = format!("{path}.");
    let fields: Vec<String> =
        args.iter().filter_map(|a| a.strip_prefix(prefix.as_str())).map(decorator_arg_label).collect();
    let mut md = format!("**`{path}`** — argument of `@{decorator}` ({} dialect)", d.name);
    if arg.ends_with('!') {
        md.push_str("\n\nRequired.");
    }
    if !fields.is_empty() {
        md.push_str(&format!("\n\nAn object with the fields {}.", fields.join(", ")));
    }
    Some(md)
}

/// Hover text of a decorator the dialect knows (`[decorators]`).
pub fn decorator_doc(name: &str, d: &Dialect) -> Option<String> {
    let title = format!("**`@{name}`** — decorator · `{}` dialect", d.name);
    if let Some(doc) = d.doc("decorators", name) {
        let args = d.decorator_args.get(name).map_or(String::new(), |args| {
            let list: Vec<String> = args.iter().map(|a| decorator_arg_label(a)).collect();
            format!("\n\n---\nArguments: {}.", list.join(", "))
        });
        return Some(format!("{title}\n\n{doc}{args}"));
    }
    let mut lines = Vec::new();
    if d.consumed_decorators.contains(name) {
        lines.push(format!(
            "Read by the templates of the `{}` dialect; it is not written as a decorator of the target.",
            d.name
        ));
    }
    if d.void_decorators.contains(name) {
        lines.push("A method with it returns nothing to its callers (the template uses its value).".to_string());
    }
    for key in d.provide.keys().filter(|k| k.split('.').next() == Some(name)) {
        let what = key.split_once('.').map_or(String::new(), |(_, arg)| format!(" in `{arg}`"));
        lines.push(format!("The objects it lists{what} are provided inside the declaration (`CounterManager.x()` reads the provided one)."));
    }
    if let Some(args) = d.decorator_args.get(name) {
        let list: Vec<String> = args.iter().map(|a| decorator_arg_label(a)).collect();
        lines.push(format!("Arguments: {}.", list.join(", ")));
    }
    (!lines.is_empty()).then(|| format!("{title}\n\n{}", lines.join("\n\n")))
}

/// Link to the API reference of a name imported automatically (Flutter and Dart libraries).
pub fn api_doc_url(module: &str, name: &str) -> Option<String> {
    let lib = if let Some(rest) = module.strip_prefix("package:flutter/") {
        rest.trim_end_matches(".dart").to_string()
    } else {
        let rest = module.strip_prefix("dart:")?;
        format!("dart-{rest}")
    };
    let page = if name.chars().next().is_some_and(char::is_uppercase) {
        format!("{name}-class.html")
    } else {
        format!("{name}.html")
    };
    Some(format!("https://api.flutter.dev/flutter/{lib}/{page}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_urls() {
        assert_eq!(
            api_doc_url("package:flutter/widgets.dart", "Text").as_deref(),
            Some("https://api.flutter.dev/flutter/widgets/Text-class.html")
        );
        assert_eq!(
            api_doc_url("dart:convert", "jsonDecode").as_deref(),
            Some("https://api.flutter.dev/flutter/dart-convert/jsonDecode.html")
        );
        assert_eq!(api_doc_url("node:fs", "x"), None);
    }

    #[test]
    fn every_keyword_has_a_doc() {
        for k in nuclear_syntax::token::Keyword::ALL {
            assert!(keyword_doc(k.as_str()).is_some(), "missing doc for `{k}`");
        }
    }
}
