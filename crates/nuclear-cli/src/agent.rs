//! `nuclear agent init` and `nuclear agent hook`: what AI agents (Claude Code, Codex, OpenCode, Cursor) need in
//! a project. The skill itself is installed by `install.sh` (agents/skills/nuclear).

use nuclear_diag::{SourceFile, json_items};
use nuclear_dialect::Loader;
use nuclear_transpile::project::{self, Manifest};
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

const AGENTS: &str = include_str!("../../../agents/project/AGENTS.md");
const CURSOR_RULE: &str = include_str!("../../../agents/project/nuclear.mdc");
const START: &str = "<!-- nuclearlang -->";
const END: &str = "<!-- /nuclearlang -->";
const HOOK: &str = "nuclear agent hook";

/// Writes or updates AGENTS.md (its NuclearLang section), CLAUDE.md (`@AGENTS.md`), the Cursor rule and the
/// Claude Code hook. Files of the project are kept: only NuclearLang's parts change. Returns what it did.
/// `mobile_default`: the mobile platform's default dialect (an older Dragonfly app without `dragonfly(...)` is one).
pub fn init(dir: &Path, dialect: Option<&str>, mobile_default: Option<String>) -> Result<Vec<String>, String> {
    // a Dragonfly project (config.nuc or main.nuc) is built with the dragonfly CLI, whatever its platform
    let dragonfly = dir.join("config.nuc").is_file() || dir.join("main.nuc").is_file();
    let older = || mobile_default.clone().ok_or_else(|| "no default dialect for the mobile platform".to_string());
    let dialect = match dialect {
        Some(d) => d.to_string(),
        None if dir.join("nuclear.toml").is_file() => Manifest::read(dir)?.dialect,
        // config.nuc names the dialect (ADR-0093)
        None if dir.join("config.nuc").is_file() => match Manifest::read(dir) {
            Ok(m) => m.dialect,
            Err(_) => older()?,
        },
        None if dir.join("main.nuc").is_file() => older()?,
        None => return Err("which dialect? nuclear agent init --dialect <name>".into()),
    };
    let check = if dragonfly {
        "dragonfly check --json".to_string()
    } else if dir.join("nuclear.toml").is_file() {
        "nuclear check . --json".to_string()
    } else {
        format!("nuclear check <file>.nuc --dialect {dialect} --json")
    };
    let fill = |t: &str| {
        t.replace("{dialect}", &dialect).replace("{check}", &check)
            .replace(
                "{dragonfly}",
                if dragonfly {
                    "- Create each piece with `dragonfly make ...` (`dragonfly help`); after `dragonfly check`, run \
                     `dragonfly analyze` and, with tests, `dragonfly test`.\n"
                } else {
                    ""
                },
            )
            .replace("{generated}", if dragonfly { "`build/`" } else { "`build/` and the `.ts`/`.dart`/`.py` built from the `.nuc`" })
    };
    let mut done = Vec::new();

    let section = fill(AGENTS);
    let path = dir.join("AGENTS.md");
    let text = match std::fs::read_to_string(&path) {
        Ok(old) => match (old.find(START), old.find(END)) {
            (Some(a), Some(b)) if a < b => format!("{}{}{}", &old[..a], section.trim_end(), &old[b + END.len()..]),
            _ => format!("{}\n\n{section}", old.trim_end()),
        },
        Err(_) => format!("# AGENTS.md\n\n{section}"),
    };
    write(&path, &text)?;
    done.push("AGENTS.md (Codex, OpenCode, Cursor and other agents)".to_string());

    let path = dir.join("CLAUDE.md");
    match std::fs::read_to_string(&path) {
        Ok(old) if old.contains("@AGENTS.md") => {}
        Ok(old) => write(&path, &format!("{}\n\n@AGENTS.md\n", old.trim_end()))?,
        Err(_) => write(&path, "@AGENTS.md\n")?,
    }
    done.push("CLAUDE.md (Claude Code: imports AGENTS.md)".to_string());

    write(&dir.join(".cursor/rules/nuclear.mdc"), &fill(CURSOR_RULE))?;
    done.push(".cursor/rules/nuclear.mdc (Cursor, for *.nuc)".to_string());

    let path = dir.join(".claude/settings.json");
    let mut settings: Value = match std::fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))?,
        Err(_) => json!({}),
    };
    add_hook(&mut settings)?;
    write(&path, &(serde_json::to_string_pretty(&settings).unwrap() + "\n"))?;
    done.push(".claude/settings.json (hook: every edited .nuc is checked)".to_string());
    Ok(done)
}

/// Adds the PostToolUse hook once; the other hooks and settings stay.
fn add_hook(settings: &mut Value) -> Result<(), String> {
    let obj = settings.as_object_mut().ok_or("settings.json is not an object")?;
    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    let post = hooks.as_object_mut().ok_or("hooks is not an object")?.entry("PostToolUse").or_insert_with(|| json!([]));
    let list = post.as_array_mut().ok_or("hooks.PostToolUse is not a list")?;
    let present = list.iter().any(|e| {
        e["hooks"].as_array().is_some_and(|hs| hs.iter().any(|h| h["command"].as_str() == Some(HOOK)))
    });
    if !present {
        list.push(json!({ "matcher": "Edit|Write|MultiEdit", "hooks": [{ "type": "command", "command": HOOK }] }));
    }
    Ok(())
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The Claude Code hook: reads the tool call (stdin), and when it edited a `.nuc` checks it (with its project, so
/// architecture rules count too). Errors go to stderr with exit code 2, which Claude Code gives back to the agent;
/// one line each, without source excerpts, to spend few tokens.
pub fn hook(l: &Loader) -> ExitCode {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let call: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    let Some(file) = call["tool_input"]["file_path"].as_str().filter(|f| f.ends_with(".nuc")) else {
        return ExitCode::SUCCESS;
    };
    let lines = errors(l, Path::new(file));
    if lines.is_empty() {
        return ExitCode::SUCCESS;
    }
    eprintln!("nuclear check found {} error(s); fix them:", lines.len());
    for line in lines.iter().take(20) {
        eprintln!("{line}");
    }
    ExitCode::from(2)
}

/// The errors of a file's project (or of the file alone), as `path:line:col code message [fix: ...]`.
fn errors(l: &Loader, file: &Path) -> Vec<String> {
    let dialects: Vec<_> = l.available().iter().filter_map(|n| l.load(n).0).collect();
    let mut out = Vec::new();
    let mut add = |diags: &[nuclear_diag::Diagnostic], source: &SourceFile| {
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).cloned().collect();
        for item in json_items(&errors, source) {
            let fix = match &item["fix"] {
                Value::Null => String::new(),
                f => format!(" [fix: replace {:?} with {:?}]", f["replace"].as_str().unwrap_or(""), f["with"].as_str().unwrap_or("")),
            };
            out.push(format!(
                "{}:{}:{} {} {}{fix}",
                item["file"].as_str().unwrap_or(""),
                item["line"],
                item["col"],
                item["code"].as_str().unwrap_or(""),
                item["message"].as_str().unwrap_or("")
            ));
        }
    };
    if let Some((root, Ok(manifest))) = Manifest::find(file, &dialects)
        && let (Some(d), _) = l.load(&manifest.dialect)
    {
        // with the project's own type definitions (`<project>/defs`), as `nuclear check` reads the project
        let d = project::with_project_defs(&root, &d).0.unwrap_or(d);
        let built = project::build(&root, &manifest, &d);
        for f in &built.files {
            add(&f.diagnostics, &SourceFile::new(f.source_path.display().to_string(), f.source.clone()));
        }
        return out;
    }
    if let Ok(text) = std::fs::read_to_string(file) {
        let diags = nuclear_transpile::check(&text, None);
        add(&diags, &SourceFile::new(file.display().to_string(), text));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_keeps_the_project_files_and_runs_twice() {
        let dir = std::env::temp_dir().join(format!("nuclear-agent-init-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".claude")).unwrap();
        std::fs::write(dir.join("main.nuc"), "main() void\n  print(1)\n").unwrap();
        std::fs::write(dir.join("AGENTS.md"), "# Mine\n\nkeep this\n").unwrap();
        std::fs::write(dir.join(".claude/settings.json"), r#"{"model": "x"}"#).unwrap();
        init(&dir, None, Some("flutter".into())).unwrap();
        init(&dir, None, Some("flutter".into())).unwrap();
        let agents = std::fs::read_to_string(dir.join("AGENTS.md")).unwrap();
        assert!(agents.starts_with("# Mine\n\nkeep this"), "{agents}");
        assert_eq!(agents.matches(START).count(), 1, "{agents}");
        assert!(agents.contains("--dialect flutter") && agents.contains("dragonfly check --json"), "{agents}");
        let settings: Value = serde_json::from_str(&std::fs::read_to_string(dir.join(".claude/settings.json")).unwrap()).unwrap();
        assert_eq!(settings["model"], "x");
        assert_eq!(settings["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap(), "@AGENTS.md\n");
        assert!(dir.join(".cursor/rules/nuclear.mdc").is_file());
    }

    /// The hook checks a project as `nuclear check` does: a name of the project's own type definitions
    /// (`<project>/defs`) is no error.
    #[test]
    fn the_hook_reads_the_type_definitions_of_the_project() {
        let dir = std::env::temp_dir().join(format!("nuclear-agent-hook-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let defs = "defs/python/probe_lib/1.0.0";
        let files = [
            ("nuclear.toml", "[project]\nname = \"demo\"\ndialect = \"python\"\n"),
            ("probes.nuc", "make() Probe\n  Probe()\n"),
            (&format!("{defs}/package.toml") as &str, "[package]\nname = \"probe_lib\"\nversion = \"1.0.0\"\nlanguage = \"python\"\ndepends = []\nlibraries = [\"probe_lib\"]\n"),
            (&format!("{defs}/index.json"), "{\"format\": 1, \"libraries\": [\"probe_lib\"], \"reexports\": {}, \"symbols\": {\"Probe\": 0}, \"classes\": {\"Probe\": {\"supers\": [], \"members\": []}}}"),
            (&format!("{defs}/api.json"), "{\"format\": 1, \"declarations\": [{\"name\": \"Probe\", \"library\": \"probe_lib\", \"kind\": \"class\", \"constructors\": [{\"name\": \"\", \"params\": []}]}]}"),
        ];
        for (rel, text) in files {
            let path = dir.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let l = Loader::new(nuclear_dialect::dialect_dirs(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
        assert_eq!(errors(&l, &dir.join("probes.nuc")), Vec::<String>::new());
        std::fs::write(dir.join("probes.nuc"), "make() Probe\n  Missing()\n").unwrap();
        assert_eq!(errors(&l, &dir.join("probes.nuc")).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
