---
layer: L4
component: nuclear
read-when: you work on dialects created by agents, `dialect new/test`, `[toolchain]`, the neutral Dragonfly spec or installing dialects from a repository
---

# Plan: new dialects created by agents

Status: **decided, not started** (2026-10-27; ADR-0072 and ADR-0073). It starts when the user asks.

## 1. Goal

A developer tells a coding agent (Claude, Codex, OpenCode) "turn Crystal and Amber into Dragonfly", the agent
creates the dialect following Nuclear's rules as we did with `flutter` and `nestjs`, pushes it to a repository,
and anyone uses it with `dragonfly crystal/amber new`.

- The user **does not program nor edit the dialect by hand**: they describe the *what for* and review.
  Everything goes through the agent.
- The agent **cannot leave** the rules, philosophy nor restrictions of Nuclear and Dragonfly
  (`spec/dialects.md` §8.4). What cannot be expressed is reported; it is not faked.

**Out of scope (postponed):** kinds, decorators and shortcuts of each project (detecting repeated code,
extension points in the framework's templates, derived kinds). They are taken up later; the project dialect
format of `spec/dialects.md` §7 stays the same and is not extended.

## 2. What already exists (2026-10-27)

- Declarative dialects (TOML + templates) with inheritance; `nuclear dialect check --json`;
  `nuclear spec [--dialect]` (the full contract for agents); `nuclear build --dialect <path>`.
- `spec/dialects.md` §8: purpose, what a dialect can do, process, **restrictions** and the done criterion. It is
  the source of truth the agent reads.
- User tutorial: `docs/dialects/create-with-agent.md`.
- `tools/spec-check/check.py` checks that a spec's equivalences are the real output, but only inside the
  NuclearLang repository.

### Gaps

1. **The `dragonfly` CLI has Flutter written in Rust** (`Manifest::infer(root, "flutter", ...)`, `doctor`, the
   SDK, the pubspec, build/run/test). A new dialect cannot have its own `dragonfly <d> new`.
2. **Dragonfly's rules as a concept are mixed with Flutter** in `spec/flutter.md` (§1, §3, §4, §6). An agent
   building `crystal/amber` has no neutral spec of what Dragonfly is.
3. **There is no example checking outside the repository**: a user's agent has no `spec-check`.
4. **Agents do not load the rules by themselves**: the tutorial's prompt has to be pasted.
5. **A dialect cannot be shared**: there is no way to install it from a repository.
6. **Targets with emitter needs**: something that does not come out of a template (Python: single-expression
   lambdas, `pass` in empty blocks) needs a new option in Rust, which only the NuclearLang team can add, never
   the user's agent (§8.4.1).

## 3. Features

### D1. `nuclear dialect test <path>` (the base of everything)

Promotes `tools/spec-check` to the CLI: it compiles each example of the dialect's `spec.md`, compares with the
expected output, shows the diff and the coverage of base constructs. `--json` for agents. Step 5 of §8.3 becomes
a command.

### D2. `nuclear dialect new <name> (--target | --extends <parent>)`

A skeleton: `dialect.toml` with every required template empty and commented with its base example,
`keywords.toml` and `spec.md` with **every base example already written on the Nuclear side** and the target
side empty. The agent completes it; `dialect test` says how much is left.

### D3. The rules reach the agent by themselves

No MCP server for now: the CLI already answers everything through the shell and an MCP adds fixed tokens to
every session (`specs/001-agent-support/plan.md` §4). Instead:

- **A Claude Code skill** `nuclear-dialect` (installed by `install.sh` in `~/.claude/skills/`): it loads only
  when the task is creating or fixing a dialect; it says "read `nuclear spec`, follow §8, use
  `dialect new/check/test`".
- **`AGENTS.md`** with the same short content for Codex and OpenCode (`nuclear dialect new` writes it in the
  dialect's folder).
- **MCP (later):** a thin layer over the same commands, for agents without a shell. Same rules.

### D4. Dragonfly as a concept: one spec per platform (`spec/mobile.md`, `spec/backend.md`; ADR-0093)

Separates from `spec/flutter.md` what Dragonfly is in any language (components, layers, facade, architecture
kinds and their meaning, `[architecture]` rules, project structure) and leaves in `flutter.md` only how it is
written in Flutter. A new Dragonfly dialect implements that document.

### D5. Dragonfly language independent (ADR-0072)

The dialect declares what the CLI has in Rust today:

```toml
[toolchain]
sdk = "flutter"                                  # binary doctor looks for
new = ["flutter create --empty {{out}}"]
build = ["flutter pub get"]
run = ["flutter run -d {{device}}"]
test = ["flutter test"]
add_package = "flutter pub add {{package}}"
manifest = { file = "config.nuc", writes = "pubspec.yaml" }
skeleton = "skeleton/"                           # files of `dragonfly <d> new`
```

`dragonfly flutter` stays the same: its section says what is in the code today. It goes in `cli/` (session
`nuclearlang-53`) and in the dialect loader.

### D6. Dialects installable from a repository

`dragonfly dialect add github.com/x/crystal-amber@1.2.0`: downloads the dialect (TOML, templates, spec,
skeleton, `[toolchain]`), runs `dialect check` and `dialect test` before accepting it, and leaves it in the
dialect cache. The project stores the version it uses, so another developer or agent gets the same one.

### D7. Emitter capabilities per target

The ones a new target asks for are added in Rust as a generic `[options]` option, never as a case of one
language. `dialect check` says which capability is missing instead of producing invalid code. First known case:
Python (`hoist_block_lambdas`, `empty_block`).

## 4. Work order

| Phase | Feature | Where |
|---|---|---|
| 1 | D1 `dialect test` | `crates/nuclear-cli` (replaces `tools/spec-check/check.py`) |
| 2 | D2 `dialect new` | `crates/nuclear-cli`, `contract.rs` |
| 3 | D3 skill + `AGENTS.md` | `install.sh`, `crates/nuclear-cli` |
| 4 | D4 neutral Dragonfly spec | `spec/`, `docs/context/dragonfly/CONTEXT.md` |
| 5 | D5 `[toolchain]` | `crates/nuclear-dialect` (format), `cli/`, `dialects/mobile/flutter` |
| 6 | D6 install from a repository | `cli/`, `crates/nuclear-dialect` (lookup) |
| 7 | A real test: an agent creates `crystal` + `amber` with only the tutorial | count how many times it stops because of §8.4.1 |
| 8 | D3 MCP | a new project, if needed |

Each phase updates specs, ADRs, diagnostics, the tutorial and the README (`docs/context/workflow.md`), with
`cargo test` and `cargo clippy` green. When D1–D3 exist, the tutorial replaces the "compare by hand" step with
`nuclear dialect test`.
