//! Golden tests: every example is transpiled with its dialect and compared with the expected file.
//! To regenerate the expected files after an intended change: `UPDATE_GOLDEN=1 cargo test -p nuclear-transpile`.

use nuclear_dialect::Loader;
use std::path::{Path, PathBuf};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// (source, dialect, expected output)
const CASES: &[(&str, &str, &str)] = &[
    ("examples/examples-nodejs/chat.nuc", "nodejs", "examples/examples-nodejs/chat.ts"),
    ("examples/examples-nestjs/users.nuc", "nestjs", "examples/examples-nestjs/users.ts"),
    ("examples/examples-python/tour.nuc", "python", "examples/examples-python/tour.py"),
    (
        "examples/examples-flutter-dialect/flutter_app.nuc",
        "flutter",
        "examples/examples-flutter-dialect/flutter_app.dart",
    ),
    (
        "examples/examples-flutter-dialect/dart_extension.nuc",
        "dart",
        "examples/examples-flutter-dialect/dart_extension.dart",
    ),
    (
        "examples/examples-flutter-dialect/widgets_tour.nuc",
        "flutter",
        "examples/examples-flutter-dialect/widgets_tour.dart",
    ),
    (
        "examples/examples-flutter-dialect/framework_libs.nuc",
        "flutter",
        "examples/examples-flutter-dialect/framework_libs.dart",
    ),
    ("examples/spec/base.nuc", "typescript", "examples/spec/base.ts"),
    ("examples/spec/base.nuc", "dart", "examples/spec/base.dart"),
];

/// The source root of the working tree that holds `rel` (a platform repository holds its examples, plan 006); `None`
/// when that repository is not checked out (the case is skipped, with a note).
fn root_of(rel: &str) -> Option<PathBuf> {
    let found = nuclear_dialect::source_roots(Path::new(ROOT)).into_iter().find(|r| r.join(rel).is_file());
    if found.is_none() {
        eprintln!("skipped: {rel} is not in this working tree (its platform repository is not checked out)");
    }
    found
}

#[test]
fn examples_match_expected_output() {
    let loader = Loader::new(nuclear_dialect::dialect_dirs(Path::new(ROOT)));
    let update = std::env::var("UPDATE_GOLDEN").is_ok();
    let mut failures = Vec::new();
    for (src, dialect, expected) in CASES {
        let Some(root) = root_of(src) else { continue };
        let (d, report) = loader.load(dialect);
        let d = d.unwrap_or_else(|| panic!("dialect {dialect}: {:#?}", report.diags));
        let text = std::fs::read_to_string(root.join(src)).unwrap();
        let out = nuclear_transpile::transpile(&text, &d);
        let errors: Vec<_> = out.diagnostics.iter().filter(|x| x.is_error()).collect();
        assert!(errors.is_empty(), "{src} with {dialect}: {errors:#?}");
        let path = root.join(expected);
        if update {
            std::fs::write(&path, &out.code).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        if want != out.code {
            failures.push(format!("{src} ({dialect}) differs from {expected}:\n--- got ---\n{}", out.code));
        }
    }
    assert!(
        failures.is_empty(),
        "{}\n(run UPDATE_GOLDEN=1 cargo test -p nuclear-transpile to accept)",
        failures.join("\n\n")
    );
}

/// The line map (for debuggers): every statement's output line points to its source line, in order.
#[test]
fn line_map_points_statements_to_their_source_line() {
    let loader = Loader::new(nuclear_dialect::dialect_dirs(Path::new(ROOT)));
    let d = loader.load("dart").0.unwrap();
    let src = "add(a: int, b: int) int\n  total = a + b\n\n  total\n";
    let out = nuclear_transpile::transpile(src, &d);
    let code: Vec<&str> = out.code.lines().collect();
    let at = |nuc: u32| out.lines.iter().find(|(_, n)| *n == nuc).map(|(o, _)| code[*o as usize - 1].trim());
    assert_eq!(at(1).map(|l| l.contains("add(")), Some(true), "{:?}\n{}", out.lines, out.code);
    assert_eq!(at(2).map(|l| l.contains("total = a + b")), Some(true), "{:?}\n{}", out.lines, out.code);
    assert_eq!(at(4).map(|l| l.contains("return total")), Some(true), "{:?}\n{}", out.lines, out.code);
    // and every example maps to real lines of both files, in output order
    for (src, dialect, _) in CASES {
        let Some(root) = root_of(src) else { continue };
        let d = loader.load(dialect).0.unwrap();
        let text = std::fs::read_to_string(root.join(src)).unwrap();
        let out = nuclear_transpile::transpile(&text, &d);
        let (outs, srcs) = (out.code.lines().count() as u32, text.lines().count() as u32);
        assert!(!out.lines.is_empty(), "{src}");
        assert!(out.lines.windows(2).all(|w| w[0].0 < w[1].0), "{src}: {:?}", out.lines);
        assert!(out.lines.iter().all(|&(o, n)| o >= 1 && o <= outs && n >= 1 && n <= srcs), "{src}");
    }
}

/// The programs of examples/stdlib/ (math, bits, text, regex, collections, types, bytes, dates, async) transpile
/// without errors in both targets and every area has both versions and its expected output.
/// `tools/stdlib-check.sh` runs them with node and dart and compares what they print.
#[test]
fn stdlib_programs_transpile_in_both_targets() {
    let loader = Loader::new(nuclear_dialect::dialect_dirs(Path::new(ROOT)));
    let dir = PathBuf::from(ROOT).join("examples/stdlib");
    let mut areas: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter_map(|e| e.file_name().to_str()?.strip_suffix(".out").map(String::from))
        .collect();
    areas.sort();
    assert!(areas.len() >= 10, "{areas:?}");
    for dialect in ["typescript", "dart", "python"] {
        let (d, report) = loader.load(dialect);
        let d = d.unwrap_or_else(|| panic!("dialect {dialect}: {:#?}", report.diags));
        for a in &areas {
            let src = dir.join(format!("{a}.nuc"));
            let text = std::fs::read_to_string(&src).unwrap_or_else(|_| panic!("{} is missing", src.display()));
            let out = nuclear_transpile::transpile(&text, &d);
            let errors: Vec<_> = out.diagnostics.iter().filter(|x| x.is_error()).collect();
            assert!(errors.is_empty(), "{}: {errors:#?}", src.display());
        }
    }
}
