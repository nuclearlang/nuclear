//! NuclearLang transpiler: `.nuc` source + a dialect → target code and diagnostics.
//!
//! This is the API the CLI, the future LSP and the MCP server call. Dialects are loaded at run time
//! from files (see `nuclear_dialect::Loader`), so editing a dialect needs no recompilation.

mod alias;
mod analysis;
mod emit;
mod naming;
pub mod contract;
pub mod project;

use nuclear_diag::Diagnostic;
use nuclear_dialect::Dialect;

pub struct Output {
    /// Target code; empty when the source has syntax errors.
    pub code: String,
    pub diagnostics: Vec<Diagnostic>,
    /// Line map for debuggers: (output line, source line), both from 1, in output order. A line of the output
    /// that is not listed belongs to the closest listed line above it.
    pub lines: Vec<(u32, u32)>,
}

/// Parses and emits `src` with `dialect`.
pub fn transpile(src: &str, dialect: &Dialect) -> Output {
    let parsed = nuclear_syntax::parse(src);
    let mut diagnostics = parsed.diagnostics;
    if diagnostics.iter().any(Diagnostic::is_error) {
        return Output { code: String::new(), diagnostics, lines: Vec::new() };
    }
    let mut file = parsed.file;
    naming::apply(&mut file, dialect);
    diagnostics.extend(alias::expand(&mut file, &Default::default(), &|d| alias_via(dialect, d)));
    let mut emitter = emit::Emitter::new(dialect, file.comments.clone()).with_source(src);
    let code = emitter.file(&file);
    diagnostics.append(&mut emitter.diags);
    diagnostics.sort_by_key(|d| d.span.start);
    Output { code, diagnostics, lines: line_map(src, &emitter.marks) }
}

/// (output line, source offset) → (output line, source line).
pub(crate) fn line_map(src: &str, marks: &[(u32, usize)]) -> Vec<(u32, u32)> {
    let starts: Vec<usize> = std::iter::once(0).chain(src.match_indices('\n').map(|(i, _)| i + 1)).collect();
    marks.iter().map(|&(out, pos)| (out, starts.partition_point(|&s| s <= pos) as u32)).collect()
}

/// `alias_via` of the kind of a declaration.
pub(crate) fn alias_via(dialect: &Dialect, d: &nuclear_syntax::ast::Decl) -> Option<String> {
    d.words.iter().find_map(|w| dialect.kinds.get(&w.name).and_then(|k| k.value.alias_via.clone()))
}

/// Diagnostics only (what the LSP and `nuclear check` need).
pub fn check(src: &str, dialect: Option<&Dialect>) -> Vec<Diagnostic> {
    match dialect {
        Some(d) => transpile(src, d).diagnostics,
        None => nuclear_syntax::parse(src).diagnostics,
    }
}
