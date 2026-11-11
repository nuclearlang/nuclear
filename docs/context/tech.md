---
layer: L1
component: system
read-when: you need a command, a tool version, an SDK path on this machine, or how to validate generated code
---

# Technology and commands

## Stack

| Part | Technology |
|---|---|
| Compiler, CLIs, LSP | Rust (Cargo workspaces; `rustfmt.toml` max_width 120, the same in every workspace) |
| Dialects | TOML + Mustache-subset templates (`crates/nuclear-dialect/src/template.rs`) |
| Type definitions generator | Dart with `package:analyzer` (`tools/defs`) |
| Dragonfly runtime | Dart/Flutter package `packages/dragonfly` (bloc, fpdart, go_router, dio, get_it, json_serializable) |
| VS Code extension | TypeScript (Node 24 globals: WebSocket) |
| JetBrains plugin | Java, built with `javac` against `$IDE_HOME` (no Gradle) |
| Zed extension | Rust → wasm32-wasip2, `zed_extension_api` 0.7, tree-sitter token grammar |
| Website | Nuclear + Dragonfly web (react), its own repository |

## Machine paths

- Flutter SDK: `~/develop/flutter` (3.47.8, on the PATH). Type definitions are still `flutter@3.47.7`; to
  regenerate only dragonfly without moving the others, run `bin/defs.dart` with `FV=3.47.7` (the `pub
  dragonfly` line of `tools/defs/generate.sh`). Older docs say `~/development/flutter`: outdated.
- Installation: `./install.sh` → `~/.local/bin/{nuclear,dragonfly,nuclear-lsp}`, files in
  `~/.local/share/nuclearlang` (`nuclear_dialect::install_dir`, `NUCLEAR_HOME`). Re-run after every change
  the user should get. `cargo install` puts binaries in `~/.cargo/bin`, which is not on the user's PATH.
  The layout is `tools/release/stage.sh`, shared with the release archives (`tools/release/package.sh [target]` →
  `dist/`) that the public installers `tools/release/install.sh` / `install.ps1` download (ADR-0101,
  `.github/workflows/release.yml`). Test an installer with a fake `HOME` and `NUCLEAR_DOWNLOAD=<local http server of
  dist/>`; never run `--uninstall` without the fake `HOME`.
- Project definitions cache: `~/.cache/nuclearlang/defs` (`nuclear_dialect::defs_cache_dir`).
- User's test project: `~/Documents/projects/nuc/flutter/rick_and_morty`.
- There is no global `tsc`: `nuclear-vscode/node_modules/.bin/tsc`.

## Commands

```sh
cargo test                                   # all crates (191 tests); compiles in debug
cargo clippy --all-targets
cargo build --release                        # before using target/release/nuclear by hand
UPDATE_GOLDEN=1 cargo test                   # regenerate golden files, then review the diff
cargo run -q -p nuclear-cli -- build <file.nuc> --dialect flutter [-o out.dart]
cargo run -q -p nuclear-cli -- check <file.nuc|folder> --dialect nodejs --json
cargo run -q -p nuclear-cli -- tokens|ast <file.nuc>
cargo run -q -p nuclear-cli -- dialect list | dialect check <name>
cargo run -q -p nuclear-cli -- spec [--dialect flutter]     # spec for agents: base + chain specs + contract
cargo run -q -p nuclear-cli -- spec --brief --dialect flutter   # the short sheet for agents (spec/brief/)
nuclear agent init [--dialect d]             # AGENTS.md, CLAUDE.md, Cursor rule, Claude Code hook in a project
TSC=<path to tsc> tools/stdlib-check.sh [area]   # examples/stdlib/ run in TypeScript and Dart (same output)
tools/agent-eval/run.py --mode baseline|agent [--only id,...]   # agent evaluation (the user's Claude quota)
cargo run -q -p nuclear-cli -- defs list|show Text[.member]|search button --dialect flutter [--inherited|--json]
cargo run -q -p nuclear-cli -- build <folder-with-nuclear.toml> [-o out]
nuclear fmt <file|folder>... [--check]       # canonical layout (crates/nuclear-syntax/src/fmt.rs); a test keeps the repo formatted
nuclear help [<topic>|<command>]             # embedded docs (TOPICS in crates/nuclear-cli/src/main.rs)
dragonfly-mobile/tools/analyze-flutter.sh [file.nuc...]       # Flutter examples → flutter analyze (flutter_lints, 0 warnings)
tools/spec-check/check.py spec/<d>.md <dialect> <language> [--fix]   # spec equivalences = real output
dragonfly-mobile/tools/spec-check-dragonfly.py --fix          # project blocks of spec/flutter.md
dragonfly-backend/tools/nestjs-check/check.sh  # NestJS: tasks_api built, tsc --strict, node:test, HTTP smoke (tools/node)
tools/defs/typescript/generate.sh            # defs/typescript/ from the .d.ts in tools/node
dragonfly-web/tools/react-check/check.sh     # React: characters web app built, tsc --strict, vitest, vite build (tools/node)
dragonfly-backend/tools/fastapi-check/check.sh  # Python: rick_api built, pyright --strict, pytest, end to end (tools/python/.venv)
tools/defs/python/generate.sh                # defs/python/ from the installed Python packages
FLUTTER_ROOT=~/develop/flutter tools/defs/generate.sh                # regenerate defs/dart/* (~45 s)
nuclear-vscode/node_modules/.bin/tsc --strict --noEmit --target es2022 --lib es2022,dom x.ts
(cd packages/dragonfly && flutter test && flutter analyze)            # runtime package (58 tests)
(cd cli && cargo test && cargo clippy --all-targets)                  # dragonfly CLI
(cd editors && cargo test && cargo clippy --all-targets)              # LSP
(cd ~/Documents/projects/nuc/react/nuclearlang_website && node tools/content.mjs && dragonfly build)   # the website
echo '{"session_id":"t"}' | tools/context/check.sh session|stop      # context hooks by hand
```

Search paths: dialects from `--dialect-path`, `NUCLEAR_DIALECT_PATH`, `./dialects` and the repository's;
definitions from `--defs-path`, `NUCLEAR_DEFS_PATH` and the `defs/` next to each dialects folder
(`Loader::repository()`, so the LSP finds them unchanged).
