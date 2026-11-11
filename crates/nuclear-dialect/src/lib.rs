//! NuclearLang dialects: data files that teach the transpiler a target language or framework.
//!
//! - [`template`]: the template engine used by every dialect file.
//! - [`contract`]: the list of templates, variables and options a dialect can define.
//! - [`model`]: loading `dialects/<name>/*.toml`, inheritance (`extends`) and validation.
//! - [`defs`]: type definitions packages (the public API of the target's libraries, like `.d.ts`).

pub mod contract;
pub mod defs;
pub mod library;
pub mod model;
pub mod template;

pub use model::{Dialect, Loader, Report};

/// Editor documentation of a word of the base language (`languages/docs.toml`): `section` is `keywords`,
/// `builtins` (`Result`, `Ok`, `Err`), `types` (`int`, `string`...) or `tags` (`@param`...).
pub fn base_doc(section: &str, name: &str) -> Option<&'static str> {
    static DOCS: std::sync::OnceLock<toml::Table> = std::sync::OnceLock::new();
    let docs = DOCS.get_or_init(|| include_str!("../../../languages/docs.toml").parse().expect("languages/docs.toml"));
    docs.get(section)?.get(name)?.as_str().map(str::trim)
}

use std::path::{Path, PathBuf};

/// Where an installed NuclearLang keeps its files (`dialects/`, `defs/`, `spec/`, `packages/`):
/// `$NUCLEAR_HOME`, or the folder above the binary's `bin/` (see `install.sh`). `None` outside an install.
pub fn install_dir() -> Option<PathBuf> {
    let dir = match std::env::var_os("NUCLEAR_HOME") {
        Some(home) => PathBuf::from(home),
        None => std::env::current_exe().ok()?.canonicalize().ok()?.parent()?.parent()?.to_path_buf(),
    };
    dir.join("dialects").is_dir().then_some(dir)
}

/// The source roots of a working tree (plan 006): `root` (the `nuclear` repository) and every `root/dragonfly` or
/// `root/dragonfly-*` folder with `dialects/`, `docs/` or `spec/` (the `dragonfly` repository and the platform
/// repositories checked out inside it). Each holds its own `dialects/`, `defs/`, `spec/`, `docs/` and `examples/`.
pub fn source_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.to_path_buf()];
    let mut inner: Vec<PathBuf> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().is_some_and(|n| {
                let n = n.to_string_lossy();
                n == "dragonfly" || n.starts_with("dragonfly-")
            }) && ["dialects", "docs", "spec"].iter().any(|d| p.join(d).is_dir())
        })
        .collect();
    inner.sort();
    roots.extend(inner);
    roots
}

/// The `dialects/` folders of every source root of a working tree.
pub fn dialect_dirs(root: &Path) -> Vec<PathBuf> {
    source_roots(root).into_iter().map(|r| r.join("dialects")).filter(|d| d.is_dir()).collect()
}

/// The definitions generated for the packages a project adds (`dragonfly flutter add hive`), shared by every
/// project: `$XDG_CACHE_HOME/nuclearlang/defs`, or `~/.cache/nuclearlang/defs`. Same layout as `defs/`.
pub fn defs_cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("nuclearlang").join("defs"))
}
