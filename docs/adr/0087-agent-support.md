---
adr: 0087
title: "Agent support: brief, skill, agent init, hook, evaluation"
status: "Accepted"
scope: nuclear
---

# ADR-0087: Agent support: brief, skill, agent init, hook, evaluation

- Status: Accepted
- Scope: `nuclear` (the `dragonfly agent init` part: `dragonfly`)
- Date: 2026-11-03

## Context

Plan `specs/001-agent-support` (F1–F4). The user decided (2026-11-03): the brief written by hand by Claude and
shared with the user; skills and files for every agent (Claude Code, Codex, OpenCode, Cursor); the evaluation
runs with the user's Claude Code account.

## Decision

- **Brief:** `spec/brief/<dialect>.md`, by hand, only what changes from TypeScript/Dart, checked with
  `spec-check` and embedded in the binary; `nuclear spec --brief --dialect <d>` concatenates base + chain
  (flutter ≈ 3k tokens; a test keeps it ≤ 16 KB). A dialect outside NuclearLang may bring `brief.md`.
- **Skill:** one `SKILL.md` (the open Agent Skills format) in `agents/skills/nuclear/`, installed by `install.sh`
  in `~/.claude/skills` (Claude Code; Cursor and OpenCode also read it), `~/.agents/skills` (Codex, Cursor,
  OpenCode) and `~/.codex/skills` (older Codex). It does not copy the brief: it asks the installed binary for it.
- **Per project:** `nuclear agent init` (and `dragonfly agent init`, also run by `dragonfly new`) writes the
  NuclearLang section of `AGENTS.md` (between markers, the rest kept), `CLAUDE.md` with `@AGENTS.md`,
  `.cursor/rules/nuclear.mdc` and a Claude Code `PostToolUse` hook (`nuclear agent hook`) that checks each edited
  `.nuc` with its project and gives the errors back (one line each, exit 2).
- **Evaluation:** `tools/agent-eval/` (10 tasks: 7 Dragonfly on a copy of rick_and_morty, 2 TypeScript, 1 Dart),
  `claude -p` headless with the user's account; `baseline` (no skills) and `agent` modes; results in
  `results/<mode>.json`.

## Consequences

- An agent learns Nuclear with ~3k tokens instead of the 50k of the full Flutter spec, and sees its errors
  without asking.
- Running the evaluation spends the user's quota (about USD 5 per mode in API-equivalent cost): ask first.
- First measurement (2026-11-03): baseline 10/10 green, 9.8 turns, ~460k tokens per task; with skill, AGENTS.md
  and hook 10/10, 10.6 turns, ~456k tokens. The agents already learn from the project's own `.nuc` files, so these
  tasks do not separate the modes; F7 needs harder tasks (a project from scratch, nothing to copy).

## Alternatives rejected

- A brief generated from the specs: less control over what an agent needs first.
- A per-project copy of the skill: it would go stale; the global skill asks the binary.
- An MCP server: more fixed tokens; the CLI already answers through the shell.
