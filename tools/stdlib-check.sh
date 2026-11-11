#!/usr/bin/env bash
# Runs every program of examples/stdlib/ in the three language dialects and compares what it prints with
# <area>.out: <area>.nuc → TypeScript (tsc --strict → node), Dart (dart run) and Python (pyright --strict →
# python). The three must print the same text (spec/library.md, ADR-0094).
#   tools/stdlib-check.sh [area...]        (UPDATE=1: the TypeScript output becomes the expected <area>.out)
# Needs node, dart (FLUTTER_ROOT or PATH), TypeScript (TSC=<path to tsc>, or tsc in PATH) and tools/python/.venv
# (pyright there; PY=<python> to use another interpreter).
set -uo pipefail
cd "$(dirname "$0")/.."
DART="${DART:-$(command -v dart || echo "${FLUTTER_ROOT:-$HOME/develop/flutter}/bin/dart")}"
TSC="${TSC:-$(command -v tsc || echo tools/node/node_modules/typescript/bin/tsc)}"
PY="${PY:-tools/python/.venv/bin/python3}"
PYRIGHT="${PYRIGHT:-tools/python/.venv/bin/pyright}"
[ -f "$TSC" ] || { echo "TypeScript not found: set TSC=<path to tsc>"; exit 2; }
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT
cargo build -q --bin nuclear || exit 2
NUC=target/debug/nuclear
areas=("$@")
[ ${#areas[@]} -gt 0 ] || areas=($(ls examples/stdlib/*.out | xargs -n1 basename | sed 's/\.out$//'))
fail=0
for a in "${areas[@]}"; do
  want="examples/stdlib/$a.out"
  src="examples/stdlib/$a.nuc"
  mkdir -p "$OUT/$a"
  if $NUC build "$src" --dialect typescript -o "$OUT/$a/$a.ts" \
    && printf 'import { main } from "./%s.js";\nawait main();\n' "$a" > "$OUT/$a/run.ts" \
    && node "$TSC" --strict --target es2022 --module es2022 --moduleResolution bundler --lib esnext,dom \
         --outDir "$OUT/$a/js" "$OUT/$a/run.ts" \
    && echo '{"type":"module"}' > "$OUT/$a/js/package.json" \
    && { [ -z "${UPDATE:-}" ] || node "$OUT/$a/js/run.js" > "$want"; } \
    && node "$OUT/$a/js/run.js" | diff "$want" - ; then
    echo "ok    $a typescript"
  else
    echo "FAIL  $a typescript"; fail=1
  fi
  if $NUC build "$src" --dialect dart -o "$OUT/$a/$a.dart" \
    && "$DART" run "$OUT/$a/$a.dart" | diff "$want" - ; then
    echo "ok    $a dart"
  else
    echo "FAIL  $a dart"; fail=1
  fi
  if $NUC build "$src" --dialect python -o "$OUT/$a/$a.py" \
    && (cd "$OUT/$a" && "$OLDPWD/$PYRIGHT" --pythonpath "$OLDPWD/$PY" "$a.py" | tail -1 | grep -q "^0 errors") \
    && (cd "$OUT/$a" && "$OLDPWD/$PY" -I -c "import asyncio, inspect, importlib.util as u; s = u.spec_from_file_location('stdlib_$a', '$a.py'); m = u.module_from_spec(s); s.loader.exec_module(m); r = m.main(); asyncio.run(r) if inspect.isawaitable(r) else None" | diff "$OLDPWD/$want" -) ; then
    echo "ok    $a python"
  else
    echo "FAIL  $a python"; fail=1
  fi
done
exit $fail
