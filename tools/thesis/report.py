#!/usr/bin/env python3
"""The report of the thesis (README.md): reads results/bench.json and results/agents.json (when it exists), draws
the charts (results/charts/*.png and .svg) and writes results/REPORT.md with the tables, the charts and the verdict
of each thesis against the thresholds of the README.

    tools/thesis/report.py
"""
import json, os, statistics, sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

here = os.path.dirname(os.path.abspath(__file__))
results = os.path.join(here, "results")
charts = os.path.join(results, "charts")

# the reference palette of the data-viz method, validated for two series (light mode): nuclear, native
SURFACE, TEXT, TEXT2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e6e5e1"
SERIES = {"nuclear": "#2a78d6", "native": "#eb6834", "generated": "#2a78d6", "hand": "#eb6834"}
THRESHOLDS = {"tokens": 0.7, "cost": 0.7, "runtime": 1.2, "lines": 0.7}


def grouped_bars(title, subtitle, groups, series, values, unit, name, hline=None):
    """Bars: one group per task/program, one thin bar (<= 24 px) per series with a 2 px surface gap; a legend when
    there are two series; recessive grid; an optional reference line (a threshold) with its label."""
    n = len(groups)
    fig_w = max(4.5, 0.9 * n + 1.5)
    fig, ax = plt.subplots(figsize=(fig_w, 3.4), dpi=100)
    fig.patch.set_facecolor(SURFACE)
    ax.set_facecolor(SURFACE)
    width = 24 * n / (fig_w * 100 * 0.85)          # 24 px per bar on a 100 dpi figure
    gap = 2 * n / (fig_w * 100 * 0.85)              # a 2 px surface gap between the bars of a group
    top = max([v for row in values for v in row if v is not None] + ([hline] if hline else []), default=1) or 1
    for si, s_ in enumerate(series):
        xs = [gi + (si - (len(series) - 1) / 2) * (width + gap) for gi in range(n)]
        ys = [values[gi][si] if values[gi][si] is not None else 0 for gi in range(n)]
        ax.bar(xs, ys, width=width, color=SERIES[s_], edgecolor=SURFACE, linewidth=1, label=s_, zorder=3)
    if hline is not None:
        ax.axhline(hline, color=TEXT2, linewidth=0.8, linestyle=(0, (4, 3)), zorder=2)
        ax.text(n - 0.45, hline, f" threshold {hline}", color=TEXT2, fontsize=7, va="bottom", ha="right")
    ax.set_xlim(-0.6, n - 0.4)
    ax.set_ylim(0, top * 1.15)
    ax.set_xticks(range(n), groups, color=TEXT, fontsize=8)
    ax.tick_params(axis="y", colors=TEXT2, labelsize=7, length=0)
    ax.tick_params(axis="x", length=0)
    ax.yaxis.grid(True, color=GRID, linewidth=0.6, zorder=0)
    ax.set_axisbelow(True)
    for side in ("top", "right", "left"):
        ax.spines[side].set_visible(False)
    ax.spines["bottom"].set_color(GRID)
    ax.set_ylabel(unit, color=TEXT2, fontsize=7)
    fig.text(0.02, 0.95, title, ha="left", va="center", color=TEXT, fontsize=10, fontweight="bold")
    fig.text(0.02, 0.89, subtitle, ha="left", va="center", color=TEXT2, fontsize=7.5)
    if len(series) > 1:
        ax.legend(loc="lower right", bbox_to_anchor=(1, 1.0), frameon=False, fontsize=7, labelcolor=TEXT,
                  ncol=len(series), handlelength=1, handleheight=0.8)
    fig.tight_layout(rect=(0, 0, 1, 0.85))
    os.makedirs(charts, exist_ok=True)
    for ext in ("png", "svg"):
        fig.savefig(os.path.join(charts, f"{name}.{ext}"), facecolor=SURFACE, dpi=200)
    plt.close(fig)
    return f"charts/{name}.png"


def mean(xs):
    xs = [x for x in xs if x is not None]
    return round(statistics.mean(xs), 1) if xs else None


def agents_section(data):
    runs = data["runs"]
    tasks = sorted({r["task"] for r in runs})
    arms = ["nuclear", "native"]
    by = {(t, a): [r for r in runs if r["task"] == t and r["arm"] == a] for t in tasks for a in arms}
    rows, charts_md = [], []
    metrics = [("tokens", "input_tokens", "tokens (input + output)", lambda r: r["input_tokens"] + (r["output_tokens"] or 0)),
               ("cost", "cost_usd", "USD", lambda r: r["cost_usd"]),
               ("turns", "turns", "turns", lambda r: r["turns"]),
               ("lines", "lines_written", "source lines written", lambda r: r["lines_written"])]
    table = ["| Task | Arm | Runs | Green | Tokens | Cost (USD) | Turns | Lines written | Seconds | Red checks |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for t in tasks:
        for a in arms:
            rs = by[(t, a)]
            if not rs:
                continue
            table.append(f"| {t} | {a} | {len(rs)} | {sum(r['green'] for r in rs)}/{len(rs)} | "
                         f"{mean([x['input_tokens'] + (x['output_tokens'] or 0) for x in rs])} | "
                         f"{mean([x['cost_usd'] for x in rs])} | {mean([x['turns'] for x in rs])} | "
                         f"{mean([x['lines_written'] for x in rs])} | {mean([x['seconds'] for x in rs])} | "
                         f"{mean([x['red_check_runs'] for x in rs])} |")
    for key, _field, unit, f in metrics:
        values = [[mean([f(r) for r in by[(t, a)]]) for a in arms] for t in tasks]
        charts_md.append((key, grouped_bars(f"Agent {key} per task", "mean over the runs; lower is better",
                                            tasks, arms, values, unit, f"agents_{key}")))
    green = [[sum(r["green"] for r in by[(t, a)]) / len(by[(t, a)]) * 100 if by[(t, a)] else None for a in arms]
             for t in tasks]
    charts_md.append(("green", grouped_bars("Agent success per task", "runs whose verify commands all pass",
                                            tasks, arms, green, "% green", "agents_green")))
    # the verdict: ratios nuclear / native over every run with both arms
    tot = {a: {"tokens": 0, "cost": 0.0, "green": 0, "n": 0} for a in arms}
    for t in tasks:
        if not by[(t, "nuclear")] or not by[(t, "native")]:
            continue
        for a in arms:
            for r in by[(t, a)]:
                tot[a]["tokens"] += r["input_tokens"] + (r["output_tokens"] or 0)
                tot[a]["cost"] += r["cost_usd"] or 0
                tot[a]["green"] += r["green"]
                tot[a]["n"] += 1
    verdict = []
    if tot["native"]["n"] and tot["nuclear"]["n"]:
        rt = (tot["nuclear"]["tokens"] / tot["nuclear"]["n"]) / (tot["native"]["tokens"] / tot["native"]["n"])
        rc = (tot["nuclear"]["cost"] / tot["nuclear"]["n"]) / max(tot["native"]["cost"] / tot["native"]["n"], 1e-9)
        gn, gv = tot["nuclear"]["green"] / tot["nuclear"]["n"], tot["native"]["green"] / tot["native"]["n"]
        ok = rt <= THRESHOLDS["tokens"] and rc <= THRESHOLDS["cost"] and gn >= gv
        verdict = [f"- tokens: nuclear / native = **{rt:.2f}** (threshold ≤ {THRESHOLDS['tokens']})",
                   f"- cost: nuclear / native = **{rc:.2f}** (threshold ≤ {THRESHOLDS['cost']})",
                   f"- green rate: nuclear **{gn:.0%}**, native **{gv:.0%}** (nuclear must not be lower)",
                   f"- **Thesis A {'HOLDS' if ok else 'DOES NOT HOLD'}** on {tot['nuclear']['n']} + {tot['native']['n']} runs"]
    else:
        verdict = ["- not enough runs with both arms: `tools/thesis/run.py --arm both`"]
    md = ["## A. Agents: the same app with less work", "", f"Runs of {data.get('date', '?')}; mean per task and arm.", ""]
    md += table + [""]
    for key, path in charts_md:
        md.append(f"![agent {key}]({path})")
        md.append("")
    md += ["### Verdict A", ""] + verdict + [""]
    return md


def bench_section(data):
    progs = data["programs"]
    names = sorted(progs)
    langs = ["ts", "dart", "py"]
    md = ["## B. Generated code: as fast as an expert's", "",
          f"Bench of {data['date']}, median of {data['runs']} runs each; the generated program and the hand-written one must print the same text.", "",
          "| Program | Language | Generated (ms) | Hand-written (ms) | Ratio | Same output |", "|---|---|---|---|---|---|"]
    worst, groups, values = 0, [], []
    for n in names:
        for l in langs:
            r = progs[n]["langs"][l]
            ratio = r["generated_ms"] / max(r["hand_ms"], 0.1)
            worst = max(worst, ratio)
            md.append(f"| {n} | {l} | {r['generated_ms']} | {r['hand_ms']} | {ratio:.2f} | {'yes' if r['same_output'] else 'NO'} |")
            groups.append(f"{n}/{l}")
            values.append([round(ratio, 2)])
    chart = grouped_bars("Run time of the generated code, relative to hand-written",
                         "median wall time, generated / hand-written (1.0 = the same); lower is better",
                         groups, ["generated"], values, "ratio", "bench_runtime", hline=THRESHOLDS["runtime"])
    same = all(r["same_output"] for n in names for r in progs[n]["langs"].values())
    okb = worst <= THRESHOLDS["runtime"] and same
    md += ["", f"![run time]({chart})", "", "### Verdict B", "",
           f"- worst ratio generated / hand-written = **{worst:.2f}** (threshold ≤ {THRESHOLDS['runtime']}); identical output: **{'yes' if same else 'NO'}**",
           f"- **Thesis B {'HOLDS' if okb else 'DOES NOT HOLD'}** on {len(names)} programs × {len(langs)} languages", "",
          "## C. Nuclear is shorter than the code it replaces", "",
          "| Program | Nuclear lines | TypeScript (hand) | Dart (hand) | Python (hand) | Nuclear / mean hand |", "|---|---|---|---|---|---|"]
    groups, values, worst_c = [], [], 0
    for n in names:
        hand = [progs[n]["langs"][l]["hand_lines"] for l in langs]
        ratio = progs[n]["nuclear_lines"] / statistics.mean(hand)
        worst_c = max(worst_c, ratio)
        md.append(f"| {n} | {progs[n]['nuclear_lines']} | {hand[0]} | {hand[1]} | {hand[2]} | {ratio:.2f} |")
        groups.append(n)
        values.append([round(ratio, 2)])
    chart = grouped_bars("Source lines of the Nuclear, relative to hand-written",
                         "Nuclear lines / mean hand-written lines; lower is better",
                         groups, ["nuclear"], values, "ratio", "bench_lines", hline=THRESHOLDS["lines"])
    okc = worst_c <= THRESHOLDS["lines"]
    md += ["", f"![lines]({chart})", "", "### Verdict C", "",
           f"- worst ratio Nuclear / hand-written lines = **{worst_c:.2f}** (threshold ≤ {THRESHOLDS['lines']})",
           f"- **Thesis C {'HOLDS' if okc else 'DOES NOT HOLD'}** on {len(names)} programs", ""]
    return md


def main():
    md = ["# The thesis, measured", "",
          "Generated by `tools/thesis/report.py` from `results/bench.json` and `results/agents.json`. What each thesis "
          "claims, how it is measured and the thresholds are in `tools/thesis/README.md`. Every chart has its table "
          "above it (the table is the accessible view; the charts are light mode).", ""]
    agents = os.path.join(results, "agents.json")
    if os.path.exists(agents):
        md += agents_section(json.load(open(agents)))
    else:
        md += ["## A. Agents: the same app with less work", "", "No runs yet: `tools/thesis/run.py --arm both --repeats 3` "
               "(it uses the user's Claude account; see the README for the cost).", ""]
    bench = os.path.join(results, "bench.json")
    if os.path.exists(bench):
        md += bench_section(json.load(open(bench)))
    else:
        md += ["## B and C. Generated code", "", "No bench yet: `tools/thesis/bench.py`.", ""]
    open(os.path.join(results, "REPORT.md"), "w").write("\n".join(md))
    print("written results/REPORT.md")


if __name__ == "__main__":
    sys.exit(main())
