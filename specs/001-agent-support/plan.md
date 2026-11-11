---
layer: L4
component: nuclear
read-when: you work on making agents write correct Nuclear/Dragonfly (brief spec, skill, hooks, diagnostics, defs validation)
---

# Plan: agents master Nuclear and Dragonfly without spending tokens

Status: **F1–F4 done** (2026-11-03, ADR-0087): `tools/agent-eval/` with its baseline, `spec/brief/` and
`nuclear spec --brief`, the `nuclear` skill + `nuclear agent init` / `dragonfly agent init`, the check hook.
F5 and F6 done (2026-11-03, ADR-0088: N0022 habits of TS/Dart/Python with fixes; N0219/N0220 from the type definitions,
in projects). F7 pending (see STATUS.md).

## 1. Goal

An agent (Claude Code or another) writes correct Nuclear/Dragonfly on the first or second try, without having
been trained on the language, and the cost of "teaching" it is much lower than the token savings of the `.nuc`
itself.

No model is trained nor fine-tuned: it is not possible with Claude and it is not needed. The knowledge arrives
three ways: **minimal context** (what changes compared with Dart/TS), **on-demand lookups** (the CLI already
answers everything) and **compiler feedback** (errors that say how to fix themselves).

### Success metric

Measured with the evaluation suite (F1), against today's baseline:

| Metric | Target |
|---|---|
| `check` errors until green, per task | ≤ 1 on average |
| Total tokens per task | ≤ the baseline's, even loading the skill |
| Fixed context tokens (skill) | ≤ 5k |
| Tasks that end green | 100% |

## 2. Diagnosis (measured on 2026-10-27 on `flutter/rick_and_morty`)

**The language already saves what it promises:**

| | Size | Approx. tokens |
|---|---|---|
| The project's `.nuc` source (37 files) | 18.7 KB | ~5k |
| Generated Dart (48 files) | 75.4 KB | ~20k |

About **4 times less** than the equivalent Dart.

**The problem is what explaining it costs:**

| Document | Size | Approx. tokens |
|---|---|---|
| `nuclear spec` (base) | 71 KB | ~18k |
| `nuclear spec --dialect flutter` | 196 KB | ~50k |
| `nuclear help flutter` | 103 KB | ~26k |
| the whole `spec/` | 280 KB | ~70k |

Loading the Flutter spec costs **2.5 times the whole project**. Principle 5 of `nuclear-base.md` ("the whole spec
fits in the context") holds today only in theory: it fits, but it takes all the savings. The spec is the right
reference for humans and for the transpiler; the agent needs something else.

**What already exists and works as it is** (no need to reinvent it):

- `nuclear check <file|folder> --json` and `dragonfly check --json`: errors + architecture rules.
- `nuclear defs show|search <Name> --dialect flutter`: the API of Flutter, Dragonfly and the packages in Nuclear
  notation. `defs show Text` takes about 700 tokens: a cheap, precise lookup.
- `nuclear help <topic>`: a guide split into chapters of 1–4k tokens.
- `tools/spec-check/check.py`: checks that the Nuclear → target examples are the real output.
- `dragonfly make`: generates the skeleton of each piece (screen, usecase, repository...), so the agent does not
  write the structure from scratch.
- LSP + VS Code extension (subproject `editors/`).

**Gaps found** (tested with `nuclear check --dialect flutter`):

| What an agent coming from Dart writes | Result today | Problem |
|---|---|---|
| `Text("a", style: TextStyle())` | N0013 + `fix: replace ':' with ' ='` | ✅ good |
| `return Text("a");` | N0009 + `fix: delete ';'` | ✅ good |
| `build(...) Widget {` | N0013 `expected end of line, found '{'` | no hint: "blocks go by indentation" |
| `Widget build(BuildContext context)` | N0013 `expected end of line, found '('` | no hint: "the return type goes at the end" |
| `x = await g()` | N0013 `found 'g'` | no hint: "use `~g()`" |
| `var x = 3` | N0013 `found 'x'` | no hint: "write `x = 3`" |
| `class Bar extends Foo` | N0201 + a list of 35 kinds | no hint: "`Bar: Foo`" or the right kind |
| `Textt("a")` | **no error** | nonexistent constructor: an invisible hallucination |
| `Text("a", styl = TextStyle())` | **no error** | nonexistent argument: an invisible hallucination |

The last two are the most serious: the error shows up only in `flutter analyze`, on the generated Dart and with
lines the agent never wrote.

There is no agent integration either: `dragonfly new` generates no `CLAUDE.md`, `AGENTS.md`, skill nor hooks.

## 3. Features

Ordered by the work sequence. Each one with its "done" criterion.

### F1. Evaluation suite and baseline (first: without it, there is no way to know whether something helps)

- 10–15 real tasks on a copy of `rick_and_morty`. Examples: "add search to episodes", "a detail screen for a
  location", "a usecase with a 5-minute cache", "a form with validation", "a global event when the network
  changes". Also 2–3 NestJS tasks so it is not biased towards Flutter.
- A script that runs each task with `claude -p --output-format json` (clean, headless session) and records:
  input/output tokens, turns, number of red `check`s and whether it ended green.
- Location: `tools/agent-eval/` (tasks in TOML, runner in Python like `spec-check`).
- **Done when:** the baseline without the skill is stored in `tools/agent-eval/results/baseline.json`.

### F2. A differences sheet: `nuclear spec --brief [--dialect <d>]`

The center of the plan. A short document (target: ≤ 4k tokens with Flutter) written **for someone who already
knows Dart/TypeScript**. It includes only what changes:

- A table of rules: blocks by indentation, return type at the end, `name = value` in arguments, `~` for await
  and automatic async, implicit return, `if`/`match` as values, `Result`/`Ok`/`Err` with propagation, aliases
  `f() = Module.method`, constructors `(dep: Type)`, kinds instead of `class`.
- 10–15 **Nuclear → Dart pairs** chosen from what is used most, generated from the transpiler's real output
  (checked with `spec-check`, as today).
- Dragonfly kinds in one line each (`repository`, `usecase`, `statemanager`, `stateless`, `appcomponent`...) and
  the architecture rules (screen → component → usecase → repository).
- "How to look up the rest": `nuclear help <topic>`, `nuclear defs show X`, `dragonfly make`.

Source: `spec/<dialect>.brief.md` next to each spec, embedded with `include_str!` like the other topics.
`--brief` concatenates the briefs of the dialect's chain (flutter → dart → base), like `spec`.

- **Done when:** `nuclear spec --brief --dialect flutter` measures ≤ 4k tokens, `spec-check` gives 0 problems on
  the briefs, and an agent with only the brief solves a simple F1 task.

### F3. Claude Code skill + `AGENTS.md`

- A `nuclear` skill (folder `SKILL.md` + references). Its body is short: when it applies (`*.nuc`,
  `nuclear.toml`) and the work cycle:
  1. `dragonfly make ...` for the skeleton.
  2. Write following the brief.
  3. `dragonfly check --json` → fix until green.
  4. On a doubtful API: `nuclear defs show/search`, never guess.
- The brief **is not copied** into the skill: the skill asks for `nuclear spec --brief --dialect <d>`, so it is
  never out of date with the installed binary.
- `install.sh` installs the skill in `~/.claude/skills/nuclear/` (option `--no-agent` to skip it).
- `dragonfly new` generates in the project an `AGENTS.md` of 10–20 lines (it works for any agent, not only
  Claude) and a `CLAUDE.md` that imports it with `@AGENTS.md`. A `dragonfly agent init` command is added for
  existing projects like `rick_and_morty`.
- **Done when:** in a new project, Claude Code activates the skill by itself when it touches a `.nuc`, and the F1
  tasks have fewer errors than the baseline.

### F4. Automatic check hook

A Claude Code `PostToolUse` hook for `Edit|Write` on `*.nuc`: it runs `nuclear check <file> --json` (or the
project's) and, if there are errors, gives them back to the agent in the same turn. The agent does not have to
remember to check.

- Installed with the skill (F3) inside the project's `.claude/settings.json` through `dragonfly agent init`.
- Compact output: only code, line, message and `fix`, without the source code block. Few tokens.
- **Done when:** a `.nuc` with an error, edited by the agent, gets the diagnostic without the agent asking.

### F5. Diagnostics with fixes for Dart/TS habits

In the parser/diagnostics (`nuclear-syntax`, `nuclear-diag`), detect and give a `fix`:

| Pattern | Suggested message |
|---|---|
| `{` at the end of a header | `blocks use indentation, not braces` + fix: delete `{` / `}` |
| `Type name(params)` | `the return type goes after the parameters: name(params) Type` |
| `await expr` | `use ~expr (async is automatic)` |
| `var/final/let/const x = ` in a body | `assign directly: x = ...` |
| `class X extends Y` | `use a kind or 'X: Y'` + the most likely kinds, not all 35 |
| `new X(...)` | `NuclearLang does not use 'new'` |

- Probably N0013/N0201 are reused adding `fix`/`help`, or new codes are created and documented in
  `spec/diagnostics.md` (the test `every_code_is_documented` requires it).
- **Done when:** each pattern has a test, `cargo test` and `cargo clippy` are green, and the F1 tasks show fewer
  correction turns.

### F6. Validation against type definitions (the strongest defense against hallucinations)

The transpiler already loads the defs (`Dialect.defs`, `index.json`) for auto-imports and overrides. They are
also used to validate:

- `PascalCase(...)` that is not in the file, the project nor the defs → an error with "did you mean `Text`?"
  (edit distance over the known names).
- A named argument the constructor or method does not have → an error with the valid parameters.
- (Optional, later) A nonexistent member on a known type: `Icons.casinoo`.

Available base (added by another session, ADR-0065–0069): project defs are added with
`Loader::add_defs(&mut Dialect, &project::defs_lock(root))` (reads `build/defs.lock`, refs `name@version`),
`nuclear_dialect::defs_cache_dir()` and `Package::from_project`. The validation must look up the dialect
**after** `add_defs`; otherwise the project's third-party packages (`dragonfly flutter add`) would give false
positives. The LSP already calls `add_defs` before `project::build`.

Care:
- Only when the type resolves with certainty. If the module stays `open` (unknown base or a whole-module `use`),
  nothing is reported: zero false positives before full coverage.
- It goes in N01xx (resolver), documented in `spec/diagnostics.md`.
- If `ast.rs` changes, tell `editors/` which variants changed (AGENTS.md rule 7).
- **Done when:** `Textt("a")` and `styl = ...` fail in `nuclear check`; the 37 files of `rick_and_morty`, the
  examples and the golden files still have no errors; `cargo test` is green.

### F7. Final measurement and tuning

Repeat F1 with everything on. Tune the brief according to where agents still fail: each repeated error is a
missing line in the brief or a missing `fix` in a diagnostic.

## 4. Out of scope (for now)

- **Fine-tuning or training:** not possible with Claude; context + compiler give the same practical result.
- **MCP server:** the CLI already answers everything through Bash, and an MCP adds fixed tokens with its tool
  schemas. It is reconsidered only if an agent without a shell needs it.
- **`llms.txt`, a public site and example repositories:** the way for future models to learn Nuclear by
  themselves. Worth it when the language is published, but it takes months and does not help today.
- **Rewriting the specs:** they stay the source of truth; the brief is a short view on top.

## 5. Work order and where each thing goes

| Phase | Feature | Where | Notes |
|---|---|---|---|
| 1 | F1 baseline | `nuclearlang/tools/agent-eval/` | before touching anything |
| 2 | F2 brief | `spec/*.brief.md`, `crates/nuclear-cli` | `spec-check` on the briefs |
| 3 | F3 skill + AGENTS.md | `install.sh`, `cli/` (subproject, its own context) | `dragonfly agent init` |
| 4 | F4 hook | `cli/` (generates `.claude/settings.json`) | compact output |
| 5 | F5 diagnostics | `crates/nuclear-syntax`, `nuclear-diag`, `spec/diagnostics.md` | tests per pattern |
| 6 | F6 defs validation | `crates/nuclear-transpile` (resolver/emit), `defs.rs` | zero false positives |
| 7 | F7 measurement | `tools/agent-eval/` | compare with the baseline |

Phases 2–4 have the highest impact for the least code (a few days). 5 and 6 touch the compiler and take more
work, but they remove the root cause of hallucinations.

Repository rules that apply (see `AGENTS.md`): everything in English, each new diagnostic documented,
`cargo test` + `cargo clippy` green, `UPDATE_GOLDEN=1` if the output changes, and `editors/` is not modified from
here.

## 6. Decisions of the user (2026-11-03)

1. The brief is written by hand (by Claude, shared with the user) and checked with `spec-check`.
2. The skill and the files for every agent (Claude Code, Codex, OpenCode, Cursor): `install.sh` installs the skill
   globally; `agent init` writes the project files.
3. The evaluation runs with the user's Claude Code account.
