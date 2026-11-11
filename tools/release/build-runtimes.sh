#!/bin/sh
# Builds the JavaScript runtimes of every source root (packages/<name>/ with a "build" script: dragonfly-nest,
# dragonfly-web...): their dist/ is what stage.sh copies into an installation. Run before ./install.sh or
# tools/release/package.sh on a fresh checkout (a developer's tree already has them built by the check scripts).
set -eu
REPO=$(cd "$(dirname "$0")/../.." && pwd)
for p in "$REPO"/packages/*/package.json "$REPO"/dragonfly/packages/*/package.json "$REPO"/dragonfly-*/packages/*/package.json; do
  [ -f "$p" ] && grep -q '"build"' "$p" || continue
  d=$(dirname "$p")
  echo "building $(basename "$d")"
  if [ -f "$d/package-lock.json" ]; then (cd "$d" && npm ci --no-audit --no-fund >/dev/null)
  else (cd "$d" && npm install --no-audit --no-fund >/dev/null); fi
  (cd "$d" && npm run --silent build)
done
