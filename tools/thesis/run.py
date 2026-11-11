#!/usr/bin/env python3
"""Thesis A (README.md): an agent writes the same app with less work in Nuclear than in the target technology.
Each task of tasks.toml runs in two arms (`nuclear`, `native`), `--repeats` times each, with Claude Code
(`claude -p`, the user's account, real cost). The run records tokens, turns, cost, time, the checks the agent ran,
the lines it wrote and whether the task's verify commands pass. See tools/agent-eval/run.py for the harness this
reuses.

    tools/thesis/run.py --dry-run                       # the plan: tasks, arms, folders, prompts
    tools/thesis/run.py --arm both --repeats 3          # everything (hours; see README for the cost)
    tools/thesis/run.py --arm nuclear --only web-app    # one task, one arm
"""
import argparse, hashlib, json, os, shutil, subprocess, sys, time, tomllib

here = os.path.dirname(os.path.abspath(__file__))
root = os.path.normpath(os.path.join(here, "..", ".."))
preamble = ("Do the task completely without asking questions; when you finish, the code must build, the checks must "
            "be clean and the tests must pass. Task: ")
SOURCE = {"nuclear": (".nuc",), "native": (".dart", ".py", ".ts", ".tsx")}
SKIP_DIRS = {"build", ".dart_tool", "node_modules", ".venv", ".git", ".idea", "dist", "__pycache__"}


def snapshot(folder):
    """Hash of every source file, to count what the agent wrote or changed."""
    out = {}
    for d, dirs, files in os.walk(folder):
        dirs[:] = [x for x in dirs if x not in SKIP_DIRS]
        for f in files:
            if f.endswith(SOURCE["nuclear"] + SOURCE["native"]):
                p = os.path.join(d, f)
                out[os.path.relpath(p, folder)] = hashlib.sha1(open(p, "rb").read()).hexdigest()
    return out


def lines_written(folder, before, arm):
    n = 0
    for rel, h in snapshot(folder).items():
        if rel.endswith(SOURCE[arm]) and before.get(rel) != h:
            n += sum(1 for l in open(os.path.join(folder, rel), encoding="utf-8", errors="ignore") if l.strip())
    return n


def parse_events(lines):
    """The usage of the run and the checks the agent ran (and how many reported errors), from stream-json."""
    events = []
    for l in lines:
        try:
            events.append(json.loads(l))
        except json.JSONDecodeError:
            pass
    result = next((e for e in reversed(events) if e.get("type") == "result"), {})
    calls, checks, red = {}, 0, 0
    for e in events:
        for c in (e.get("message") or {}).get("content") or []:
            if not isinstance(c, dict):
                continue
            if c.get("type") == "tool_use" and c.get("name") == "Bash":
                calls[c["id"]] = c["input"].get("command", "")
            if c.get("type") == "tool_result" and c.get("tool_use_id") in calls:
                command = calls[c["tool_use_id"]]
                if any(w in command for w in (" check", " analyze", " build", " test", "pytest", "tsc")):
                    checks += 1
                    text = json.dumps(c.get("content"))
                    if c.get("is_error") or '"code"' in text or " error " in text or "error:" in text:
                        red += 1
    usage = result.get("usage", {})
    return {
        "turns": result.get("num_turns"), "cost_usd": result.get("total_cost_usd"),
        "input_tokens": usage.get("input_tokens", 0) + usage.get("cache_read_input_tokens", 0)
                        + usage.get("cache_creation_input_tokens", 0),
        "output_tokens": usage.get("output_tokens", 0), "check_runs": checks, "red_check_runs": red,
    }


def prepare(task, arm, work, repeat):
    dst = os.path.join(work, f"{task['id']}-{arm}-{repeat}")
    shutil.rmtree(dst, ignore_errors=True)
    src = task[arm]["project"]
    if src == "empty":
        os.makedirs(dst)
    else:
        shutil.copytree(os.path.expandvars(src), dst, symlinks=True,
                        ignore=shutil.ignore_patterns(".idea", ".dart_tool", "node_modules", ".venv"))
    if arm == "nuclear":
        # the agent support an installed Nuclear gives (ADR-0087): the skill, AGENTS.md and the hook
        dialect = task[arm].get("dialect", "flutter")
        if src == "empty":
            subprocess.run(["nuclear", "agent", "init", "--dialect", dialect], cwd=dst, capture_output=True)
        else:
            subprocess.run(["dragonfly", "agent", "init"], cwd=dst, capture_output=True)
        skill = os.path.join(root, "agents", "skills", "nuclear")
        if os.path.isdir(skill):
            shutil.copytree(skill, os.path.join(dst, ".claude", "skills", "nuclear"), dirs_exist_ok=True)
    return dst


def run_arm(task, arm, work, repeat, env, timeout):
    dst = prepare(task, arm, work, repeat)
    before = snapshot(dst)
    prompt = preamble + task[arm]["prompt"] + "\n\n" + task["spec"]
    cmd = ["claude", "-p", prompt, "--output-format", "stream-json", "--verbose",
           "--permission-mode", "acceptEdits", "--allowedTools", "Bash", "Read", "Edit", "Write", "Glob", "Grep",
           "--setting-sources", "project,local", "--strict-mcp-config"]
    if arm == "native":
        cmd.append("--disable-slash-commands")
    started = time.time()
    try:
        p = subprocess.run(cmd, cwd=dst, env=env, capture_output=True, text=True, timeout=timeout)
        lines = p.stdout.splitlines()
    except subprocess.TimeoutExpired as e:
        lines = (e.stdout.decode() if isinstance(e.stdout, bytes) else (e.stdout or "")).splitlines()
    open(dst + ".jsonl", "w").write("\n".join(lines))
    verify_env = dict(env, FIXTURES=os.path.join(root, "tools", "agent-eval", "fixtures"))
    failed = [v for v in task[arm]["verify"]
              if subprocess.run(v, shell=True, cwd=dst, env=verify_env, capture_output=True).returncode != 0]
    r = {"task": task["id"], "arm": arm, "repeat": repeat, "green": not failed, "failed_checks": failed,
         "seconds": round(time.time() - started), "lines_written": lines_written(dst, before, arm)}
    r.update(parse_events(lines))
    return r


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--arm", choices=["nuclear", "native", "both"], default="both")
    a.add_argument("--only", default="")
    a.add_argument("--repeats", type=int, default=1)
    a.add_argument("--timeout", type=int, default=2400)
    a.add_argument("--dry-run", action="store_true")
    args = a.parse_args()
    tasks = tomllib.load(open(os.path.join(here, "tasks.toml"), "rb"))["task"]
    if args.only:
        tasks = [t for t in tasks if t["id"] in args.only.split(",")]
    arms = ["nuclear", "native"] if args.arm == "both" else [args.arm]
    work = os.path.join(os.environ.get("TMPDIR", "/tmp"), "nuclear-thesis")
    os.makedirs(work, exist_ok=True)
    env = dict(os.environ)
    flutter = os.environ.get("FLUTTER_ROOT", os.path.expanduser("~/develop/flutter"))
    env["PATH"] = os.path.join(flutter, "bin") + os.pathsep + env["PATH"]
    if args.dry_run:
        for t in tasks:
            for arm in arms:
                print(f"{t['id']} [{arm}] x{args.repeats}: project {t[arm]['project']} → {work}/{t['id']}-{arm}-N")
                print("  prompt: " + (preamble + t[arm]["prompt"] + " " + t["spec"])[:160] + "...")
                print("  verify: " + " && ".join(t[arm]["verify"]))
        return 0
    out = os.path.join(here, "results", "agents.json")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    results = json.load(open(out)) if os.path.exists(out) else {"runs": []}
    for t in tasks:
        for arm in arms:
            for repeat in range(1, args.repeats + 1):
                print(f"{t['id']} [{arm}] {repeat}/{args.repeats}...", flush=True)
                r = run_arm(t, arm, work, repeat, env, args.timeout)
                results["runs"] = [x for x in results["runs"]
                                   if not (x["task"] == t["id"] and x["arm"] == arm and x["repeat"] == repeat)]
                results["runs"].append(r)
                results["date"] = time.strftime("%Y-%m-%d")
                json.dump(results, open(out, "w"), indent=2)
                print(f"  green={r['green']} turns={r['turns']} tokens={r['input_tokens']}+{r['output_tokens']} "
                      f"cost={r['cost_usd']} lines={r['lines_written']} {r['seconds']}s", flush=True)
    print(f"written {out}; now: tools/thesis/report.py")


if __name__ == "__main__":
    sys.exit(main())
