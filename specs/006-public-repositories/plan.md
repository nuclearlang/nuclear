---
layer: L4
component: nuclear
read-when: you split the working tree into the public repositories (step 1 of the competitiveness plan, ADR-0083/0093)
---

# 006 — Public repositories: the organization `nuclearlang`

Progress: stage 1 in progress; `dragonfly-web/` moved (STATUS.md "Half done").

Step 1 of the competitiveness plan (STATUS.md "Next"). Decided by the user (2026-11-10): the GitHub organization
**`nuclearlang`** (it exists, created 2026-10-18, with an empty repository `nuclear`), repositories **public from the
first push**, license **Apache-2.0**, the split of ADR-0083 as amended by ADR-0093 (one repository per piece with its
own release cycle), done **in stages**.

## 1. The repositories and what goes into each

| Repository | From this tree |
|---|---|
| `nuclear` | `crates/` + `editors/lsp` → `crates/nuclear-lsp`, `languages/`, `dialects/{typescript,dart,python,nodejs}`, `defs/{dart/dart,python/python,typescript/typescript}` (the languages' standard libraries), `tools/{spec-check,defs,stdlib-check.sh,context}`, `spec/` of the base and languages (`nuclear-base`, `grammar`, `typescript`, `dart`, `python`, `nodejs`, `dialects`, `definitions`, `projects`, `diagnostics`, `reserved-words`, `library`, `decisions`, `brief/` of those), `docs/{language,dialects,architecture.md,adr}`, `examples/{spec,stdlib,examples-python,examples-nodejs,projects}`, `agents/`, `specs/` (plans), `install.sh` |
| `nuclear-vscode` | `editors/vscode`, `editors/theme`, `editors/scripts` (it downloads the LSP binary of a `nuclear` release) |
| `nuclear-jetbrains` | `editors/jetbrains` |
| `nuclear-zed` | `editors/zed` |
| `dragonfly` | `cli/` (the `dragonfly` CLI), `docs/dragonfly/{README,kinds,tutorial}.md`, `spec/dragonfly.md` |
| `dragonfly-mobile` | `dialects/mobile`, `packages/{dragonfly,dragonfly_genkit}`, `defs/dart/*` except `dart`, `spec/{mobile,flutter}.md`, `examples/examples-flutter-dialect`, `tools/analyze-flutter.sh`, `docs/dragonfly/genkit.md` |
| `dragonfly-backend` | `dialects/backend`, `packages/{dragonfly-py,dragonfly-nest}`, `defs/python/*` except `python`, `defs/typescript/*` of the backend, `spec/{backend,fastapi,nestjs}.md`, `examples/{examples-backend,examples-fastapi,examples-nestjs}`, `tools/{fastapi-check,nestjs-check,python,node}`, `docs/dragonfly/{python,nestjs,contracts}.md` |
| `dragonfly-web` | `dialects/web`, `packages/dragonfly-web`, `defs/typescript/{react,react_router,vitest,testing_library_react,dragonfly-web}`, `spec/{web,react}.md`, `examples/examples-web`, `tools/react-check`, `docs/dragonfly/react.md` |
| `website` | `~/Documents/projects/nuc/react/nuclearlang_website` (the site in Nuclear; it reads the docs of the other repositories at build time; the old VitePress `website/` was removed 2026-11-10) |
| `.github` | the organization's profile (`profile/README.md`) |

`examples/expected/` (goldens) goes with each example. `docs/context/` splits by its own folders (it is already cut
by future repository, ADR-0084). `STATUS.md` and `AGENTS.md`: each repository gets its own, plus the working folder
below.

Not public: `CodeBuilder/` (teaching material, rule 8) and `docs/aprende/` (the user's teaching material in Spanish)
stay out unless the user says otherwise; `tools/thesis/results` and `tools/agent-eval/results` are kept (they are the
published numbers).

**Layout of the working tree (decided 2026-11-10 with the pilot):** the root folder is the `nuclear` repository and
every other repository is a folder inside it (`dragonfly-web/`, `dragonfly-backend/`, `editors/`…), ignored by the
root's git, as `editors/` and `cli/` are today. `nuclear_dialect::source_roots(root)` lists the root and each
`dragonfly-*` folder with `dialects/`; the CLI, the LSP, the tests, `install.sh`, the editors' bundles and the website
merge the `dialects/`, `defs/`, `spec/`, `docs/` and `examples/` of every source root. A CI of a platform repository
checks out `nuclear` and places itself inside it.

Commits and pushes are made with the user's own GitHub user (decided 2026-11-10).

## 2. Stages

1. **Local reorganization** (no network): one future repository at a time, moved under a working folder
   `nuclearlang/<repo>/`; after each move `cargo test`, clippy, the check scripts and `./install.sh` green; every path
   in the tests, scripts, `install.sh` and `docs/context/` updated. The compiler's tests that build the platforms'
   examples (`crates/nuclear-transpile/tests/projects.rs`) move to the platform repositories or read them from the
   sibling folder; the `nuclear` repository keeps language-only tests. `editors/` and `cli/` keep their history
   (`git` of their own today): their sessions are told before each move (rule 7).
2. **Publishing files** in every repository: `LICENSE` (Apache-2.0), `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`,
   `.gitignore` (build/, target/, node_modules/, .dart_tool/, .venv/, dist/), CI (GitHub Actions: the checks run by
   hand today), the Zed manifest's `repository` pointing to GitHub.
3. **Create and push** with the user's GitHub session (`gh auth login`): the repositories in `nuclearlang`, the first
   commit of each, CI green.
4. Then (still step 1 of "Next"): packages on pub.dev / PyPI / npm, extensions in the marketplaces, binaries for
   macOS and Windows, a one-command installer, the 1.0 spec.

## 3. Open with the user

- `docs/aprende/` public or not.
