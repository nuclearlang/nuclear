//! `nuclear-lsp`: the NuclearLang language server, over stdin/stdout.

use nuclear_dialect::Loader;
use nuclear_lsp::{metrics, rpc, server};
use std::io::{self, BufReader};

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("nuclear-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    // every code metrics setting with its default: the editors build their settings pages from it
    if std::env::args().any(|a| a == "--codemetrics-defaults") {
        println!("{:#}", metrics::defaults_json());
        return;
    }
    // packaging check: the dialects shipped next to the binary load on their own, with their type definitions
    if std::env::args().any(|a| a == "--check-bundle") {
        std::process::exit(check_bundle());
    }
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut server = server::Server::new(io::stdout());
    while let Ok(Some(msg)) = rpc::read_message(&mut input) {
        if !server.handle(msg) {
            break;
        }
    }
}

/// Loads every dialect of `<binary>/dialects` (types from `<binary>/defs`). Exit code 1 on any diagnostic or
/// when a dialect with type definitions cannot read their full API (the language's root object).
fn check_bundle() -> i32 {
    let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) else { return 1 };
    let loader = Loader::new(vec![dir.join("dialects")]);
    let mut code = 0;
    for name in loader.available() {
        let (dialect, report) = loader.load(&name);
        for (_, d) in &report.diags {
            eprintln!("{name}: [{}] {}", d.code.id, d.message);
        }
        // a dialect with type definitions must read its root object (python's is lowercase); a missing declared
        // package is already a diagnostic
        let root = if dialect.as_ref().and_then(|d| d.chain.last()).is_some_and(|l| l == "python") { "object" } else { "Object" };
        match dialect {
            Some(d) if report.diags.is_empty() && (d.typedefs.packages.is_empty() || d.typedefs.get(root).is_some()) => {
                println!("{name}: ok ({} type definition packages)", d.typedefs.packages.len());
            }
            _ => {
                eprintln!("{name}: not loadable from {}", dir.display());
                code = 1;
            }
        }
    }
    code
}
