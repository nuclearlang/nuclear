//! Projects of several files (spec/projects.md): the folder with `nuclear.toml`, a Dragonfly project (the
//! folder with `config.nuc` naming its platform and dialect) or a Flutter app (the folder with `main.nuc`). The whole project is transpiled from disk with
//! `nuclear_transpile::project::build` (a few ms) when a file is opened or saved; each open file then sees the
//! top-level names of the other files.

use crate::index::Index;
use crate::text::LineIndex;
use nuclear_diag::{Diagnostic, Span};
use nuclear_dialect::{Dialect, Loader};
use nuclear_syntax::ast::Item;
use nuclear_transpile::project::{self, BuiltFile, Manifest};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A top-level declaration of a project file.
pub struct Name {
    pub name: String,
    /// File, relative to the project root, without extension (`components/users/user`).
    pub path: String,
    pub uri: String,
    /// LSP range of the name in its file.
    pub range: Value,
    /// First line of the declaration.
    pub signature: String,
}

pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
    files: HashMap<String, BuiltFile>,
    /// Index of each file as on disk: members and parameters of the other files' declarations (completion).
    indexes: HashMap<String, Index>,
    names: Vec<Name>,
    /// Names of the files the dialect generates (`[project.files] names`: `startDragonfly`).
    generated: HashSet<String>,
    /// `[project.outputs]` folders (test, integration_test): what they declare is only seen inside them.
    isolated: Vec<String>,
    kinds: Vec<(String, String)>,
    /// The dialect plus the definitions of the packages the project adds (`build/defs.lock`, written by
    /// `dragonfly flutter add`); `None` when it adds none.
    pub dialect: Option<Arc<Dialect>>,
    /// `build/defs.lock` when the project was built: a new package (`dragonfly flutter add`) rebuilds it.
    pub defs_lock: Vec<String>,
}

/// The dialect with the project's own definitions packages (`build/defs.lock`, found in the cache of
/// `nuclear_dialect::defs_cache_dir` or in `<project>/defs`), and a warning for each package not found.
fn with_project_defs(root: &Path, dialect: &Dialect) -> (Option<Dialect>, Vec<Diagnostic>) {
    let refs = project::project_defs(root, dialect);
    if refs.is_empty() {
        return (None, Vec::new());
    }
    let mut loader = Loader::new(Vec::new());
    loader.defs_paths.push(root.join("defs"));
    let mut own = dialect.clone();
    let report = loader.add_defs(&mut own, &refs);
    let missing = report
        .diags
        .iter()
        .map(|(_, d)| {
            let msg = format!("{} (run `dragonfly build` to generate them)", d.message);
            Diagnostic::warning(d.code, Span::default(), msg)
        })
        .collect();
    (Some(own), missing)
}

/// The project that contains `file`: the closest folder with `nuclear.toml`, with `config.nuc` naming
/// `dragonfly(platform, dialect)` (ADR-0093: a backend or mobile project), or with `main.nuc` (a Flutter app
/// before that line; same order as `Manifest::find`). The manifest is an error when `nuclear.toml` cannot be
/// read (N0303).
pub fn find(file: &Path) -> Option<(PathBuf, Result<Manifest, String>)> {
    for dir in file.parent()?.ancestors() {
        if dir.join("nuclear.toml").is_file() {
            return Some((dir.to_path_buf(), Manifest::read(dir)));
        }
        if dir.join(project::CONFIG_FILE).is_file()
            && let Ok(m) = Manifest::read_dragonfly(dir)
        {
            return Some((dir.to_path_buf(), Ok(m)));
        }
        if dir.join("main.nuc").is_file() {
            return Some((dir.to_path_buf(), Ok(Manifest::infer(dir, "flutter", "build/app"))));
        }
    }
    None
}

/// `components/users/user.nuc` → `components/users/user`.
pub fn stem(rel: &Path) -> String {
    rel.with_extension("").to_string_lossy().replace('\\', "/")
}

impl Project {
    pub fn build(root: PathBuf, manifest: Manifest, dialect: &Dialect) -> Project {
        let (own, missing) = with_project_defs(&root, dialect);
        let dialect = own.as_ref().unwrap_or(dialect);
        let out = project::build(&root, &manifest, dialect);
        // the project card (`.nuclear/project.json`): the editor client reads the platform and dialect from it
        let _ = project::write_metadata(&root, &manifest, dialect);
        let mut names = Vec::new();
        let mut files = HashMap::new();
        let mut indexes = HashMap::new();
        for mut f in out.files {
            // the manifest shows the packages whose definitions are missing (imports, hover, completion)
            if f.out_path.is_empty() {
                f.diagnostics.extend(missing.iter().cloned());
            }
            let path = stem(&f.source_path);
            let uri = path_to_uri(&root.join(&f.source_path));
            let lines = LineIndex::new(&f.source);
            let parsed = nuclear_syntax::parse(&f.source);
            let kind = default_kind(&dialect.project.kinds, &path);
            indexes.insert(path.clone(), Index::build(&f.source, &parsed.file, Some(dialect), kind.as_deref()));
            for item in parsed.file.items {
                let Item::Decl(d) = item else { continue };
                if d.name.name == "main" {
                    continue;
                }
                names.push(Name {
                    name: d.name.name.clone(),
                    path: path.clone(),
                    uri: uri.clone(),
                    range: lines.range(&f.source, d.name.span),
                    signature: line_at(&f.source, d.name.span).to_string(),
                });
            }
            files.insert(path, f);
        }
        Project {
            generated: dialect.project.files.iter().flat_map(|f| f.names.iter().cloned()).collect(),
            isolated: dialect.project.outputs.iter().map(|(from, _)| from.clone()).collect(),
            kinds: dialect.project.kinds.clone(),
            dialect: own.map(Arc::new),
            defs_lock: project::defs_lock(&root),
            root,
            manifest,
            files,
            indexes,
            names,
        }
    }

    /// The file as transpiled with the project, when `text` is still what is on disk.
    pub fn built(&self, path: &str, text: &str) -> Option<&BuiltFile> {
        self.files.get(path).filter(|f| f.source == text)
    }

    /// Declarations of the other files that `path` can use.
    pub fn visible<'a>(&'a self, path: &'a str) -> impl Iterator<Item = &'a Name> {
        self.names.iter().filter(move |n| n.path != path && self.can_see(path, &n.path))
    }

    /// Is `name` declared by another file of the project or by a generated file, visible from `path`?
    pub fn knows(&self, path: &str, name: &str) -> bool {
        self.generated.contains(name) || self.visible(path).any(|n| n.name == name)
    }

    fn can_see(&self, from: &str, path: &str) -> bool {
        match self.isolated.iter().find(|f| path.starts_with(&format!("{f}/"))) {
            Some(f) => from.starts_with(&format!("{f}/")),
            None => true,
        }
    }

    /// `[project.kinds]`: the kind of the declarations without a kind word in this file's folder (the longest
    /// pattern that matches; `*` is one folder).
    pub fn default_kind(&self, path: &str) -> Option<String> {
        default_kind(&self.kinds, path)
    }

    /// A top-level declaration of another file visible from `from`: its file's source, index and binding.
    pub fn decl(&self, from: &str, name: &str) -> Option<(&str, &Index, usize)> {
        let n = self.visible(from).find(|n| n.name == name)?;
        let (f, ix) = (self.files.get(&n.path)?, self.indexes.get(&n.path)?);
        let b = ix.toplevel.iter().copied().find(|&b| ix.bindings[b].name == name)?;
        Some((f.source.as_str(), ix, b))
    }
}

fn default_kind(kinds: &[(String, String)], path: &str) -> Option<String> {
    let folders: Vec<&str> = path.split('/').collect();
    let folders = &folders[..folders.len().saturating_sub(1)];
    kinds
        .iter()
        .filter(|(pattern, _)| {
            let parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
            parts.len() <= folders.len() && parts.iter().zip(folders).all(|(p, f)| *p == "*" || p == f)
        })
        .max_by_key(|(pattern, _)| pattern.len())
        .map(|(_, k)| k.clone())
}

fn line_at(src: &str, span: Span) -> &str {
    let start = src[..span.start].rfind('\n').map_or(0, |i| i + 1);
    let end = src[span.start..].find('\n').map_or(src.len(), |i| span.start + i);
    src[start..end].trim()
}

/// `/home/a b/x.nuc` → `file:///home/a%20b/x.nuc`.
pub fn path_to_uri(path: &Path) -> String {
    let mut out = String::from("file://");
    let p = path.to_string_lossy().replace('\\', "/");
    if !p.starts_with('/') {
        out.push('/');
    }
    for b in p.bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~:".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uris_round_trip() {
        let p = PathBuf::from("/home/a b/ñ.nuc");
        assert_eq!(crate::server::uri_to_path(&path_to_uri(&p)), Some(p));
    }
}
