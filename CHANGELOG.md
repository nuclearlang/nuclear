# Changelog

All notable changes to nuclear are written here; versions follow [Semantic Versioning](https://semver.org).

## Unreleased

- First public version.
- ADR-0104: the Smart Graph is a library of typed, linked, versioned rows; the definition of done includes the knowledge base (`nuclear-graph knowledge build | check`, `ingest`) and the end-of-turn hook (`tools/context/check.sh stop`) runs the check.
- `[[generators]]` and `[gen]` in a platform's contract (`Dialect.generators`, `gen_values`); a facade alias named like its use case's field gets the field `<name>Instance` (ADR-0103).
- `nuclear_transpile::map::shape`: a declaration of a project as its source writes it (its file, its header's parameters, each method's parameters, return type and whether it awaits), which `dragonfly gen usecase` reads; the `usecase` generator and whole-member merges in `spec/dialects.md` (ADR-0103).
- `docs/ai/mcp.md`: the MCP server's page (how to add it to each client, the order a model works in, every tool by group, what to do when one fails); the Loop page links to it.
- ADR-0105: a strategy for each kind of request (a whole system, one screen, a rename, a failing test...), as rows of the Smart Graph that the analyst and the Loop follow; `docs/ai/loop.md` (the strategy of a request) and `docs/ai/mcp.md` (`graph_strategy`); the skill names it.
- ADR-0106: the AI services come with the installer. Every release archive carries the code of the Smart Graph and the Nuclear Developer with its knowledge base (`ai/`, `tools/release/stage.sh`); the installers end with their set-up (`nuclear-loop setup`: a profile for the machine, the services at every login), bring a machine already set up to date without questions, and take `--no-ai` and `--profile <id>` (`-NoAi`, `-AiProfile` on Windows); `--uninstall` removes the services; `./install.sh --ai` is the same set-up over the working tree. `docs/ai/setup.md`; the release workflow clones the three AI repositories.
