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
