# Proving the thesis, step by step

NuclearLang's thesis (`docs/context/product/prd.md`, ADR-0038) has three measurable claims. This folder is the
programmed way to test them, with numbers and charts instead of opinions. Step 7 of the competitiveness plan
(STATUS.md "Next").

| Thesis | Claim | Measured by | Threshold to hold |
|---|---|---|---|
| **A** | An agent builds the same app with less work in Nuclear than in the target technology | `run.py`: the same spec given to Claude Code twice (arm `nuclear`, arm `native`); tokens, cost, turns, time, lines written, checks that failed, and whether the result is green | nuclear / native ≤ 0.7 in tokens and in cost, with a green rate not lower than native |
| **B** | The generated code is as fast as an expert's | `bench.py`: each program of `bench/` built from `.nuc` with the typescript, dart and python dialects against the hand-written `bench/hand/*`; median wall time, identical output | generated / hand-written ≤ 1.2 in every program and language, same output |
| **C** | Nuclear is shorter than the code it replaces | `bench.py`: source lines of the `.nuc` against the hand-written versions | Nuclear / mean hand-written ≤ 0.7 |

The thresholds are a first proposal (2026-11-06); the user owns them. A thesis that does not hold is a finding,
not a failure of the method: the report says by how much, and the numbers tell what to fix.

## 0. Prerequisites

- The repository built and installed: `cargo build` (the bench uses `target/debug/nuclear`) and `./install.sh
  --no-vscode --no-zed` (the agents use the installed `nuclear` and `dragonfly`).
- `tools/node` installed (`npm install` there; the bench compiles TypeScript with its `tsc`).
- Dart: `dart` in the PATH or `FLUTTER_ROOT` (default `~/develop/flutter`); Flutter itself for the mobile tasks.
- Python 3.12+ with `matplotlib` (the charts; `python3 -c "import matplotlib"`).
- For thesis A: Claude Code logged in (`claude`). The runs cost real money: the 2026-11-03 evaluation
  (`tools/agent-eval/`) averaged USD 0.5 per easy task; these tasks are apps from scratch, so budget **USD 2–6 per
  run** and 4 tasks × 2 arms × `--repeats` runs. Three repeats ≈ USD 100.
- For the `mobile-feature` task: the external app `~/Documents/projects/nuc/flutter/rick_and_morty` built once
  (`dragonfly build` there) so its `build/app` exists for the native arm.

## 1. Bench the generated code (theses B and C) — minutes, free

```sh
tools/thesis/bench.py --runs 5
```

Builds every `bench/<name>.nuc` three times (TypeScript compiled with `tsc --strict` and run with node; Dart compiled
to a native executable; Python run as it is), runs each version and its hand-written twin five times, keeps the
median, checks both print the same text, and counts source lines. Output: `results/bench.json`.

To add a program: write `bench/<name>.nuc` (a `main()`), then `bench/hand/<name>.ts`, `.dart`, `.py` the way an
expert in that language would write it (they must print exactly the same). Keep the work a few hundred
milliseconds so the measurement is not process start-up.

## 2. Run the agents (thesis A) — hours, paid

```sh
tools/thesis/run.py --dry-run                         # see the plan first
tools/thesis/run.py --arm both --repeats 3            # everything
tools/thesis/run.py --arm nuclear --only web-app      # one task, one arm (results are merged into agents.json)
```

Each task of `tasks.toml` has one functional `spec` and two arms. The `nuclear` arm gets the Nuclear framing, the
installed skill and `nuclear agent init` / `dragonfly agent init` (what a user of Nuclear has, ADR-0087); the
`native` arm gets the same spec with the target's framing and no Nuclear. Both run in a fresh folder with the same
model, tools and permission mode; both are judged by the target's own tools (`verify` commands: analyzers and the
tests the agent itself had to write and pass). Everything the agent printed is kept next to the folder
(`$TMPDIR/nuclear-thesis/<task>-<arm>-<n>.jsonl`) to read what happened in a red run.

Output: `results/agents.json` (one record per run; a repeated run replaces its record).

## 3. Build the report

```sh
tools/thesis/report.py
```

Writes `results/REPORT.md` and the charts (`results/charts/*.png|svg`): per task, the agent tokens, cost, turns,
lines written and success rate of each arm; per program and language, the run time of the generated code against
the hand-written one; per program, the source lines. Every chart has its table above it. The verdict of each thesis
is computed against the thresholds of this README.

## 4. Read it, then decide

- **Thesis A holds** → publish the numbers (website, README) and keep the tasks as a regression suite: re-run after
  a change to the skill, the diagnostics or a dialect.
- **Thesis A does not hold** → read the `.jsonl` of the red or expensive runs: where did the agent lose turns
  (a diagnostic it did not understand, a missing word in the brief, a check it did not run)? Fix that
  (`spec/brief/`, `docs.toml`, N02xx messages), re-run the task.
- **Thesis B does not hold** → the generated code of that language has an inefficiency an expert would not write:
  compare the two sources side by side and fix the dialect's template or the emitter.
- **Thesis C does not hold** → the Nuclear of that program is as long as the target: a missing library member or
  a construct the base language lacks (`specs/005-language-1-0/plan.md`).

## What this does and does not prove

- Four tasks and two programs are a start, not a study: add tasks that look like the user's real work, and repeat
  (`--repeats 3` or more) before trusting a ratio.
- Both arms use the same model and the same tools; the native arm is not handicapped (it gets the ecosystem's
  standard libraries in its framing). The comparison is Nuclear + Dragonfly against the raw technology, which is
  the choice a team makes.
- The bench measures CPU-bound code; an app's real cost is mostly its framework (http, rendering), which the
  dragonfly runtimes carry and the bench does not measure.
- The tasks were written by the people building Nuclear. A fair study adds tasks written by someone else.

## Files

| File | Role |
|---|---|
| `tasks.toml` | the tasks: spec, arms, verify commands |
| `run.py` | thesis A: runs the arms with `claude -p`, collects the metrics |
| `bench/*.nuc`, `bench/hand/*` | theses B and C: the programs |
| `bench.py` | builds, runs and times them |
| `report.py` | charts (matplotlib, the data-viz palette validated for two series) and `results/REPORT.md` |
| `results/` | `bench.json`, `agents.json`, `charts/`, `REPORT.md` |
