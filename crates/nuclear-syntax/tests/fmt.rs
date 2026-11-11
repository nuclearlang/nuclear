//! `nuclear fmt` keeps the repository in the canonical layout: every `.nuc` of `examples/`, `dialects/`, `packages/`
//! and `kinds/` is a fixed point (fix: `nuclear fmt .`).

use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn nuc_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !name.starts_with('.') && !matches!(name.as_str(), "build" | "node_modules" | "target") {
                nuc_files(&p, out);
            }
        } else if name.ends_with(".nuc") {
            out.push(p);
        }
    }
}

#[test]
fn every_nuc_of_the_repository_is_formatted() {
    let mut files = Vec::new();
    // this repository and the platform repositories checked out inside it (`dragonfly-*`, plan 006)
    let mut roots = vec![Path::new(ROOT).to_path_buf()];
    roots.extend(std::fs::read_dir(ROOT).unwrap().flatten().map(|e| e.path()).filter(|p| {
        p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("dragonfly-"))
    }));
    for root in &roots {
        for d in ["examples", "dialects", "packages", "kinds"] {
            nuc_files(&root.join(d), &mut files);
        }
    }
    assert!(!files.is_empty(), "found no .nuc files");
    let bad: Vec<String> = files
        .iter()
        .filter(|f| {
            let src = std::fs::read_to_string(f).unwrap();
            nuclear_syntax::fmt::format(&src).map_or(true, |out| out != src)
        })
        .map(|f| f.display().to_string())
        .collect();
    assert!(bad.is_empty(), "not formatted (run `nuclear fmt .`):\n{}", bad.join("\n"));
}
