#!/usr/bin/env sh
# Installs NuclearLang from this repository: the `nuclear` and `dragonfly` CLIs, their dialects, type
# definitions, specs and the dragonfly Dart package, the language server (nuclear-lsp, used by Zed), plus the VS Code
# extension (which bundles its own copy of the LSP) and the Zed extension.
#
#   ./install.sh                  install into ~/.local (PREFIX=/usr/local ./install.sh for another prefix)
#   ./install.sh --no-vscode      without the VS Code extension
#   ./install.sh --no-zed         without the Zed extension
#   ./install.sh --no-agent       without the `nuclear` skill for AI agents
#   ./install.sh --uninstall      remove what an install created
#
# Layout: $PREFIX/share/nuclearlang/{bin,dialects,defs,spec,docs/dragonfly,packages/{dragonfly,dragonfly_genkit,dragonfly-py,dragonfly-nest,dragonfly-web},tools/defs} and links in $PREFIX/bin.
# Definitions generated for projects go to ~/.cache/nuclearlang/defs (kept by --uninstall).
# The binaries find their files from their own location (nuclear_dialect::install_dir).
set -eu

PREFIX=${PREFIX:-$HOME/.local}
HOME_DIR=$PREFIX/share/nuclearlang
BIN_DIR=$PREFIX/bin
REPO=$(cd "$(dirname "$0")" && pwd)
VSCODE=1
ZED=1
AGENT=1
# where agents look for skills: Claude Code (also read by Cursor and OpenCode), the open .agents folder (Codex,
# Cursor, OpenCode) and Codex's own
SKILL_DIRS="$HOME/.claude/skills $HOME/.agents/skills $HOME/.codex/skills"

for arg in "$@"; do
  case $arg in
    --no-vscode) VSCODE=0 ;;
    --no-zed) ZED=0 ;;
    --no-agent) AGENT=0 ;;
    --uninstall)
      rm -f "$BIN_DIR/nuclear" "$BIN_DIR/dragonfly" "$BIN_DIR/nuclear-lsp"
      rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/zed/extensions/installed/nuclearlang"
      rm -rf "$HOME_DIR"
      for d in $SKILL_DIRS; do rm -rf "$d/nuclear"; done
      rm -rf "${XDG_STATE_HOME:-$HOME/.local/state}/dragonfly"   # the intro shows again on the next install
      command -v code >/dev/null 2>&1 && code --uninstall-extension nuclearlang.nuclearlang >/dev/null 2>&1 || true
      echo "NuclearLang uninstalled from $PREFIX"
      exit 0 ;;
    -h|--help) sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || { echo "cargo is required: https://rustup.rs" >&2; exit 1; }

echo "building nuclear and dragonfly (release)..."
cargo build --release --quiet --manifest-path "$REPO/Cargo.toml" -p nuclear-cli -p nuclear-lsp
cargo build --release --quiet --manifest-path "$REPO/dragonfly/Cargo.toml"

echo "installing into $HOME_DIR"
mkdir -p "$BIN_DIR"
EXE=""
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) EXE=".exe" ;; esac
"$REPO/tools/release/stage.sh" "$HOME_DIR" "$REPO/target/release/nuclear$EXE" "$REPO/dragonfly/target/release/dragonfly$EXE" "$REPO/target/release/nuclear-lsp$EXE"
ln -sf "$HOME_DIR/bin/nuclear" "$BIN_DIR/nuclear"
ln -sf "$HOME_DIR/bin/dragonfly" "$BIN_DIR/dragonfly"
ln -sf "$HOME_DIR/bin/nuclear-lsp" "$BIN_DIR/nuclear-lsp"

if [ "$AGENT" = 1 ]; then
  for d in $SKILL_DIRS; do
    rm -rf "$d/nuclear"
    mkdir -p "$d"
    cp -R "$HOME_DIR/agents/skills/nuclear" "$d/"
  done
  echo "skill 'nuclear' for AI agents in: $SKILL_DIRS (per project: nuclear agent init / dragonfly agent init)"
fi

if [ "$VSCODE" = 1 ]; then
  VSIX=$(ls "$REPO"/nuclear-vscode/nuclearlang-*.vsix 2>/dev/null | sort -V | tail -n 1)
  if ! command -v code >/dev/null 2>&1; then
    echo "note: 'code' is not in the PATH; VS Code extension skipped"
  elif [ -z "$VSIX" ]; then
    echo "note: no .vsix in nuclear-vscode (npm run package there); VS Code extension skipped"
  else
    code --install-extension "$VSIX" --force >/dev/null && echo "VS Code extension: $(basename "$VSIX")"
  fi
fi

if [ "$ZED" = 1 ]; then
  if ! command -v zed >/dev/null 2>&1 && ! command -v zeditor >/dev/null 2>&1; then
    echo "note: Zed is not installed; Zed extension skipped"
  elif ! rustup target list --installed 2>/dev/null | grep -q wasm32-wasip2; then
    echo "note: the Zed extension needs 'rustup target add wasm32-wasip2'; skipped"
  else
    "$REPO/nuclear-zed/build.sh" >/dev/null && echo "Zed extension: nuclearlang"
  fi
fi

"$BIN_DIR/nuclear" dialect list >/dev/null || { echo "error: the installed nuclear does not find its dialects" >&2; exit 1; }
echo "done: nuclear and dragonfly in $BIN_DIR"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "add $BIN_DIR to your PATH: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
echo "Flutter apps (dragonfly) need Flutter: set FLUTTER or FLUTTER_ROOT if it is not in the PATH"
