#!/usr/bin/env python3
"""Thesis B and C (README.md): the code the transpiler writes runs as fast as an expert's, and the Nuclear that
produced it is shorter. Each program of bench/ exists twice: <name>.nuc (built with the typescript, dart and python
dialects) and hand/<name>.{ts,dart,py} (idiomatic, written by a person). Both are run `--runs` times (median wall
time), their output must be identical, and their source lines are counted.

    tools/thesis/bench.py [--runs 5] [--only wordfreq]      # writes results/bench.json
"""
import argparse, json, os, shutil, statistics, subprocess, sys, tempfile, time

here = os.path.dirname(os.path.abspath(__file__))
root = os.path.normpath(os.path.join(here, "..", ".."))
flutter = os.environ.get("FLUTTER_ROOT", os.path.expanduser("~/develop/flutter"))
dart = shutil.which("dart") or os.path.join(flutter, "bin", "dart")
tsc = os.path.join(root, "tools", "node", "node_modules", "typescript", "bin", "tsc")
nuclear = os.path.join(root, "target", "debug", "nuclear")
if not os.path.exists(nuclear):
    nuclear = shutil.which("nuclear") or nuclear

LANGS = {"ts": "typescript", "dart": "dart", "py": "python"}


def lines(path):
    """Source lines: not blank, not a comment line."""
    n = 0
    for l in open(path, encoding="utf-8"):
        s = l.strip()
        if s and not s.startswith(("//", "#", "*", "/*", '"""')):
            n += 1
    return n


def build(name, lang, work):
    """The generated program of <name>.nuc for `lang`, with a runner that calls main(); returns the command."""
    src = os.path.join(here, "bench", f"{name}.nuc")
    out = os.path.join(work, f"{name}.{lang}")
    subprocess.run([nuclear, "build", src, "--dialect", LANGS[lang], "-o", out], check=True)
    return runner(out, lang, work, generated=True), out


def runner(path, lang, work, generated):
    """How to run `path`: TypeScript is compiled with tsc, Dart to a native executable, Python as it is."""
    d = os.path.dirname(path)
    stem = os.path.splitext(os.path.basename(path))[0]
    if lang == "ts":
        js = os.path.join(work, "js-" + ("gen" if generated else "hand") + "-" + stem)
        entry = path
        if generated:
            entry = os.path.join(d, f"run_{stem}.ts")
            open(entry, "w").write(f'import {{ main }} from "./{stem}.js";\nmain();\n')
        subprocess.run([
            "node", tsc, "--strict", "--target", "es2022", "--module", "es2022", "--moduleResolution", "bundler",
            "--lib", "esnext,dom", "--outDir", js, entry,
        ], check=True)
        open(os.path.join(js, "package.json"), "w").write('{"type":"module"}')
        return ["node", os.path.join(js, os.path.basename(entry).replace(".ts", ".js"))]
    if lang == "dart":
        exe = os.path.join(work, ("gen" if generated else "hand") + "-" + stem)
        subprocess.run([dart, "compile", "exe", path, "-o", exe], check=True, capture_output=True)
        return [exe]
    if generated:
        entry = os.path.join(d, f"run_{stem}.py")
        open(entry, "w").write(f"import sys\nsys.path.insert(0, {d!r})\nimport {stem}\n{stem}.main()\n")
        return [sys.executable, "-I", entry]
    return [sys.executable, "-I", path]


def measure(cmd, runs):
    times, out = [], None
    for _ in range(runs):
        t = time.perf_counter()
        p = subprocess.run(cmd, capture_output=True, text=True, check=True)
        times.append((time.perf_counter() - t) * 1000)
        out = p.stdout
    return round(statistics.median(times), 1), out


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--runs", type=int, default=5)
    a.add_argument("--only", default="")
    args = a.parse_args()
    names = sorted(f[:-4] for f in os.listdir(os.path.join(here, "bench")) if f.endswith(".nuc"))
    if args.only:
        names = [n for n in names if n in args.only.split(",")]
    results = {"date": time.strftime("%Y-%m-%d"), "runs": args.runs, "programs": {}}
    work = tempfile.mkdtemp(prefix="nuclear-thesis-bench-")
    for name in names:
        print(f"{name}...", flush=True)
        r = {"nuclear_lines": lines(os.path.join(here, "bench", f"{name}.nuc")), "langs": {}}
        for lang in LANGS:
            hand = os.path.join(here, "bench", "hand", f"{name}.{lang}")
            gen_cmd, gen_src = build(name, lang, work)
            hand_cmd = runner(hand, lang, work, generated=False)
            gen_ms, gen_out = measure(gen_cmd, args.runs)
            hand_ms, hand_out = measure(hand_cmd, args.runs)
            r["langs"][lang] = {
                "generated_ms": gen_ms, "hand_ms": hand_ms, "same_output": gen_out == hand_out,
                "generated_lines": lines(gen_src), "hand_lines": lines(hand),
            }
            print(f"  {lang}: generated {gen_ms} ms, hand {hand_ms} ms, same output {gen_out == hand_out}, "
                  f"lines nuclear {r['nuclear_lines']} / generated {lines(gen_src)} / hand {lines(hand)}", flush=True)
        results["programs"][name] = r
    os.makedirs(os.path.join(here, "results"), exist_ok=True)
    json.dump(results, open(os.path.join(here, "results", "bench.json"), "w"), indent=2)
    shutil.rmtree(work, ignore_errors=True)
    print("written results/bench.json")


if __name__ == "__main__":
    sys.exit(main())
