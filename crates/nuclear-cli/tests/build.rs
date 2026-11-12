//! `nuclear build` of a project, through the binary.

use std::path::Path;
use std::process::Command;

fn build(root: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_nuclear")).args(["build", "."]).current_dir(root).status().unwrap();
    assert!(status.success());
}

/// A `.nuc` removed (or renamed) takes its generated file with it on the next build: the line map lists what the
/// previous build wrote.
#[test]
fn a_removed_source_takes_its_output_with_it() {
    let root = std::env::temp_dir().join(format!("nuclear-build-stale-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("nuclear.toml"), "[project]\nname = \"stale\"\ndialect = \"typescript\"\n").unwrap();
    std::fs::write(root.join("a.nuc"), "a() int\n  1\n").unwrap();
    std::fs::write(root.join("b.nuc"), "b() int\n  2\n").unwrap();
    build(&root);
    assert!(root.join("build/a.ts").is_file() && root.join("build/b.ts").is_file());
    std::fs::remove_file(root.join("b.nuc")).unwrap();
    build(&root);
    assert!(root.join("build/a.ts").is_file());
    assert!(!root.join("build/b.ts").exists(), "the output of a removed .nuc stays");
    let _ = std::fs::remove_dir_all(&root);
}

/// `nuclear fix` applies the compiler's exact fixes (an async call used without `~`) and reports what is left.
#[test]
fn fix_applies_the_exact_fixes() {
    let dir = std::env::temp_dir().join(format!("nuclear-fix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("books.nuc");
    std::fs::write(&file, "Books()\n  all() string[]\n    ~fetchAll()\n\n  fetchAll() string[]\n    []\n\n  count() int\n    books = Books()\n    books.all().length\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_nuclear")).args(["fix", file.to_str().unwrap(), "--dialect", "typescript"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(std::fs::read_to_string(&file).unwrap().contains("(~books.all()).length"));
    let _ = std::fs::remove_dir_all(&dir);
}
