#!/bin/sh
# Assembles an installation of NuclearLang in <dest>, the layout the binaries expect (nuclear_dialect::install_dir):
#   bin/  dialects/  defs/  spec/  docs/dragonfly/  kinds/  packages/  tools/defs/  agents/skills/
# from every source root of the working tree (this repository and the platform repositories inside it, plan 006).
# Used by install.sh (from source) and by tools/release/package.sh (the archives the installers download).
#
#   tools/release/stage.sh <dest> <binary>...      the binaries: nuclear, dragonfly, nuclear-lsp (built before)
set -eu
REPO=$(cd "$(dirname "$0")/../.." && pwd)
DEST=$1
shift
ROOTS="$REPO"
# the source roots (nuclear_dialect::source_roots): dragonfly and dragonfly-* with dialects/, docs/ or spec/
for r in "$REPO"/dragonfly "$REPO"/dragonfly-*; do
  { [ -d "$r/dialects" ] || [ -d "$r/docs" ] || [ -d "$r/spec" ]; } && ROOTS="$ROOTS $r"
done

rm -rf "$DEST"
mkdir -p "$DEST/bin" "$DEST/packages"
for b in "$@"; do cp "$b" "$DEST/bin/"; done
# nuclear-loop (ADR-0102): the Loop engine bundled for node by nuclear-vscode (`npm run loop`)
if [ -f "$REPO/nuclear-vscode/out/nuclear-loop.js" ]; then
  cp "$REPO/nuclear-vscode/out/nuclear-loop.js" "$DEST/bin/"
  printf '#!/bin/sh\nexec node "$(dirname "$(readlink -f "$0")")/nuclear-loop.js" "$@"\n' > "$DEST/bin/nuclear-loop"
  chmod +x "$DEST/bin/nuclear-loop"
  printf '@node "%%~dp0nuclear-loop.js" %%*\r\n' > "$DEST/bin/nuclear-loop.cmd"
fi
for r in $ROOTS; do
  # dialects, definitions, specs, the platforms' guides and kind packages, merged
  for d in dialects defs spec docs/dragonfly kinds; do
    if [ -d "$r/$d" ]; then mkdir -p "$DEST/$d"; cp -R "$r/$d/." "$DEST/$d/"; fi
  done
  # the runtimes: every package, as it is (a built JavaScript package needs its dist/)
  for p in "$r"/packages/*; do
    [ -d "$p" ] || continue
    if [ -f "$p/package.json" ] && grep -q '"main": *"dist/' "$p/package.json" && [ ! -d "$p/dist" ]; then
      echo "error: $p is not built (npm run build there)" >&2
      exit 1
    fi
    cp -R "$p" "$DEST/packages/"
  done
done
# what a build or a test leaves inside a package is never part of an installation (a stale build/ would also make
# pip reuse an old wheel)
find "$DEST/packages" -depth \( -name build -o -name .dart_tool -o -name node_modules -o -name __pycache__ \
  -o -name .pytest_cache -o -name '*.egg-info' -o -name dist-test \) -type d -exec rm -rf {} +
find "$DEST/packages" -name pubspec.lock -type f -exec rm -f {} +
# the definitions generator `dragonfly flutter add` runs (its `dart pub get` happens on first use)
mkdir -p "$DEST/tools/defs"
for f in bin pubspec.yaml pubspec.lock python typescript; do
  if [ -e "$REPO/tools/defs/$f" ]; then cp -R "$REPO/tools/defs/$f" "$DEST/tools/defs/"; fi
done
# the skill AI agents read (install.sh and the installers copy it into the agents' folders)
mkdir -p "$DEST/agents"
cp -R "$REPO/agents/skills" "$DEST/agents/"
