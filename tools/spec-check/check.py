#!/usr/bin/env python3
"""Checks the equivalences of a dialect spec: every target block (```typescript, ```dart...) that follows a
```nuclear block must appear, line by line, in the real output of the transpiler for that Nuclear block.

    tools/spec-check/check.py spec/typescript.md typescript typescript
    tools/spec-check/check.py spec/dart.md dart dart
    tools/spec-check/check.py spec/dart.md dart dart --fix   # rewrites the blocks that differ
"""
import os, re, subprocess, sys

here = os.path.dirname(os.path.abspath(__file__))
md, dialect, lang = sys.argv[1], sys.argv[2], sys.argv[3]
fix = "--fix" in sys.argv[4:]
subprocess.run(["cargo", "build", "-q", "--release", "--manifest-path", os.path.join(here, "Cargo.toml")], check=True)
gen = subprocess.run([os.path.join(here, "target/release/spec-check"), md, dialect], capture_output=True, text=True, check=True).stdout
outs = dict(re.findall(r"<<<NUCLEAR\n(.*?)>>>OUT\n(.*?)<<<END", gen, re.S))
errors = re.findall(r"<<<NUCLEAR\n(.*?)>>>OUT\n.*?!!! (.*?)\n<<<END", gen, re.S)
text = open(md).read()
# a target block right after a Nuclear block (at most one short paragraph between them, no heading)
pair_re = r"(```nuclear\n((?:(?!```).)*?)```\n\n(?:[^`#]*?\n)?```" + lang + r"\n)(.*?)(```)"
pairs = [(m[1], m[2]) for m in re.findall(pair_re, text, re.S)]
norm = lambda t: [l.strip() for l in t.strip().splitlines() if l.strip()]
bad = 0
for src, want in pairs:
    w, g = norm(want), norm(outs.get(src, ""))
    if not any(g[i:i + len(w)] == w for i in range(len(g) - len(w) + 1)):
        bad += 1
        print("=== MISMATCH\n" + src + "--- spec\n" + want + "--- transpiler\n" + outs.get(src, ""))
for src, msg in errors:
    bad += 1
    print("=== ERROR\n" + src + "--- " + msg)
if fix and bad:
    def repl(m):
        out = outs.get(m.group(2))
        w, g = norm(m.group(3)), norm(out or "")
        if out is None or any(g[i:i + len(w)] == w for i in range(len(g) - len(w) + 1)):
            return m.group(0)
        return m.group(1) + out.strip() + "\n" + m.group(4)
    open(md, "w").write(re.sub(pair_re, repl, text, flags=re.S))
    print(f"{md}: rewrote the blocks that differed")
print(f"{md}: {len(pairs)} equivalences, {bad} problems")
sys.exit(1 if bad else 0)
