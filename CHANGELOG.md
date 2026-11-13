# Changelog

All notable changes to nuclear are written here; versions follow [Semantic Versioning](https://semver.org).

## Unreleased

- First public version.
- `nuclear agent hook` checks a project with its own type definitions (`<project>/defs`), as `nuclear check` does: it reported every name of them as an error after each edit.
- The `try` template gets `catch_used` (the block names the caught value): the `python` dialect writes `except Exception:` when it does not, since a name nothing reads is an error of pyright's strict mode (`spec/python.md` §3.12).
- ADR-0107 §4: the planner of the Smart Graph runs an exact query only when its question is closer than the closest card; ADR-0104 §3: the graph's search is exact; ADR-0106 §9: a service that starts at login waits for the keyring.
- ADR-0104: the Smart Graph is a library of typed, linked, versioned rows; the definition of done includes the knowledge base (`nuclear-graph knowledge build | check`, `ingest`) and the end-of-turn hook (`tools/context/check.sh stop`) runs the check.
- `[[generators]]` and `[gen]` in a platform's contract (`Dialect.generators`, `gen_values`); a facade alias named like its use case's field gets the field `<name>Instance` (ADR-0103).
- `nuclear_transpile::map::shape`: a declaration of a project as its source writes it (its file, its header's parameters, each method's parameters, return type and whether it awaits), which `dragonfly gen usecase` reads; the `usecase` generator and whole-member merges in `spec/dialects.md` (ADR-0103).
- `docs/ai/mcp.md`: the MCP server's page (how to add it to each client, the order a model works in, every tool by group, what to do when one fails); the Loop page links to it.
- ADR-0105: a strategy for each kind of request (a whole system, one screen, a rename, a failing test...), as rows of the Smart Graph that the analyst and the Loop follow; `docs/ai/loop.md` (the strategy of a request) and `docs/ai/mcp.md` (`graph_strategy`); the skill names it.
- ADR-0106: the AI services come with the installer. Every release archive carries the code of the Smart Graph and the Nuclear Developer with its knowledge base (`ai/`, `tools/release/stage.sh`); the installers end with their set-up (`nuclear-loop setup`: a profile for the machine, the services at every login), bring a machine already set up to date without questions, and take `--no-ai` and `--profile <id>` (`-NoAi`, `-AiProfile` on Windows); `--uninstall` removes the services; `./install.sh --ai` is the same set-up over the working tree. `docs/ai/setup.md`; the release workflow clones the three AI repositories.
- `[project] package` in `nuclear.toml`: the package a project's files are emitted into (`build/<package>/...`; imports between the project's files are relative to it, `{{package}}` in a dialect's `project_import`); the `python` dialect uses it, so a Nuclear project is a Python package another tool can package (`spec/projects.md`, `spec/python.md`).
- ADR-0107: questions with one right answer are answered from exact facts (SQLite, in the memory of the graph) by a query planner that asks no model; the Smart Graph is written in Nuclear (the `python` dialect), `tools/release/stage.sh` carries its `.nuc` sources and the set-up builds them; rule 12 of `AGENTS.md`: Nuclear and Dragonfly first for every new development.
- ADR-0102 §23: any step, stage or run of the Loop runs again (`nuclear-loop retry <run> --step <id>`, `nuclear-loop restart <run>`), each step keeps the decisions it took and why, the run is drawn as a graph, and the acceptance criteria are held against what the run left (`docs/ai/loop.md`).
