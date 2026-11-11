#!/usr/bin/env python3
"""Agent evaluation: runs each task of tasks.toml with Claude Code (`claude -p`, the user's account) in a fresh
copy of its project, then checks the result. See README.md.

    tools/agent-eval/run.py --mode baseline            # no skill, no AGENTS.md, no hook
    tools/agent-eval/run.py --mode agent               # skill + `dragonfly agent init` in the copy
    tools/agent-eval/run.py --mode agent --only ts-stats,dart-csv
"""
import argparse, json, os, shutil, subprocess, sys, time, tomllib

here = os.path.dirname(os.path.abspath(__file__))
root = os.path.normpath(os.path.join(here, "..", ".."))
projects = {"rick_and_morty": os.path.expanduser("~/Documents/projects/nuc/flutter/rick_and_morty")}
preamble = ("This folder is a NuclearLang project (.nuc files). The `nuclear` and `dragonfly` command line tools are "
            "installed. Do the task completely without asking questions; when you finish, the code must have no "
            "errors.\n\nTask: ")


def run_task(task, mode, work, env, timeout):
    dst = os.path.join(work, task["id"])
    shutil.rmtree(dst, ignore_errors=True)
    src = projects.get(task["project"])
    if src:
        shutil.copytree(src, dst, symlinks=True, ignore=shutil.ignore_patterns(".idea", ".dart_tool"))
    else:
        os.makedirs(dst)
    if mode == "agent" and src:
        subprocess.run(["dragonfly", "agent", "init"], cwd=dst, env=env, capture_output=True)
    if mode == "agent" and not src:
        subprocess.run(["nuclear", "agent", "init", "--dialect", task["dialect"]], cwd=dst, env=env, capture_output=True)
    if mode == "agent":
        # the skill as a project skill: the run reads only project settings (no user plugins nor hooks)
        shutil.copytree(os.path.join(root, "agents", "skills", "nuclear"), os.path.join(dst, ".claude", "skills", "nuclear"))
    cmd = ["claude", "-p", preamble + task["prompt"], "--output-format", "stream-json", "--verbose",
           "--permission-mode", "acceptEdits", "--allowedTools", "Bash", "Read", "Edit", "Write", "Glob", "Grep",
           "--setting-sources", "project,local", "--strict-mcp-config"]
    if mode == "baseline":
        cmd.append("--disable-slash-commands")
    started = time.time()
    try:
        p = subprocess.run(cmd, cwd=dst, env=env, capture_output=True, text=True, timeout=timeout)
        lines = p.stdout.splitlines()
    except subprocess.TimeoutExpired as e:
        lines = (e.stdout.decode() if isinstance(e.stdout, bytes) else (e.stdout or "")).splitlines()
    events = []
    for l in lines:
        try:
            events.append(json.loads(l))
        except json.JSONDecodeError:
            pass
    with open(os.path.join(work, task["id"] + ".jsonl"), "w") as f:
        f.write("\n".join(lines))
    result = next((e for e in reversed(events) if e.get("type") == "result"), {})
    # checks the agent ran itself, and how many of them reported errors
    calls, checks, red = {}, 0, 0
    for e in events:
        for c in (e.get("message") or {}).get("content") or []:
            if not isinstance(c, dict):
                continue
            if c.get("type") == "tool_use" and c.get("name") == "Bash":
                calls[c["id"]] = c["input"].get("command", "")
            if c.get("type") == "tool_result" and c.get("tool_use_id") in calls:
                command = calls[c["tool_use_id"]]
                if any(w in command for w in (" check", " analyze", " build", " test")):
                    checks += 1
                    text = json.dumps(c.get("content"))
                    if c.get("is_error") or '"code"' in text or " error " in text or "error:" in text:
                        red += 1
    verify_env = dict(env, FIXTURES=os.path.join(here, "fixtures"))
    failed = [v for v in task["verify"] if subprocess.run(v, shell=True, cwd=dst, env=verify_env,
                                                          capture_output=True).returncode != 0]
    usage = result.get("usage", {})
    return {
        "id": task["id"], "green": not failed, "failed_checks": failed,
        "seconds": round(time.time() - started), "turns": result.get("num_turns"),
        "cost_usd": result.get("total_cost_usd"),
        "input_tokens": usage.get("input_tokens", 0) + usage.get("cache_read_input_tokens", 0)
                        + usage.get("cache_creation_input_tokens", 0),
        "output_tokens": usage.get("output_tokens"), "check_runs": checks, "red_check_runs": red,
    }


def main():
    a = argparse.ArgumentParser()
    a.add_argument("--mode", choices=["baseline", "agent"], required=True)
    a.add_argument("--only", default="")
    a.add_argument("--timeout", type=int, default=1200)
    args = a.parse_args()
    tasks = tomllib.load(open(os.path.join(here, "tasks.toml"), "rb"))["task"]
    if args.only:
        tasks = [t for t in tasks if t["id"] in args.only.split(",")]
    work = os.path.join(os.environ.get("TMPDIR", "/tmp"), f"nuclear-agent-eval-{args.mode}")
    os.makedirs(work, exist_ok=True)
    env = dict(os.environ)
    flutter = os.environ.get("FLUTTER_ROOT", os.path.expanduser("~/develop/flutter"))
    env["PATH"] = os.path.join(flutter, "bin") + os.pathsep + env["PATH"]
    out = os.path.join(here, "results", f"{args.mode}.json")
    results = json.load(open(out)) if os.path.exists(out) and args.only else {"tasks": {}}
    for t in tasks:
        print(f"{t['id']}...", flush=True)
        r = run_task(t, args.mode, work, env, args.timeout)
        results["tasks"][t["id"]] = r
        print(f"  green={r['green']} turns={r['turns']} checks={r['check_runs']} red={r['red_check_runs']} "
              f"tokens={r['input_tokens']}+{r['output_tokens']} {r['seconds']}s", flush=True)
        rs = list(results["tasks"].values())
        results["summary"] = {
            "tasks": len(rs), "green": sum(x["green"] for x in rs),
            "avg_red_check_runs": round(sum(x["red_check_runs"] for x in rs) / len(rs), 2),
            "avg_turns": round(sum(x["turns"] or 0 for x in rs) / len(rs), 1),
            "avg_tokens": round(sum(x["input_tokens"] + (x["output_tokens"] or 0) for x in rs) / len(rs)),
            "cost_usd": round(sum(x["cost_usd"] or 0 for x in rs), 2),
            "date": time.strftime("%Y-%m-%d"),
        }
        json.dump(results, open(out, "w"), indent=2)
    print(json.dumps(results["summary"], indent=2))


if __name__ == "__main__":
    sys.exit(main())
