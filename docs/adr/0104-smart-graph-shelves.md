---
adr: 0104
title: The Smart Graph is a library of typed, linked, versioned rows
status: Accepted
scope: nuclear-graph
---

# ADR-0104: The Smart Graph is a library of typed, linked, versioned rows

- Status: Accepted (asked by the user, 2026-11-11: "the knowledge base was not designed; the context and the
  documentation were copied into the RAG without sense: give it sense")
- Scope: `nuclear-graph` (store, ingest, search, MCP), `nuclear-knowledge` (`nuclear-developer/data`: the assets),
  `nuclear-developer` (what the agents recall), the working tree's workflow (the check at the end of every change)
- Date: 2026-11-12

## Context

Until now the graph held two collections made by cutting every markdown file of the tree into pieces of 1400
characters (`knowledge`, 3069 pieces; `nuclear`, the 2029 a local agent may read). A piece knew its file and little
else. Asking "what is nuclear" answered the agent skill and a chapter about modules; "advantages" answered the VS Code
README; an error code found its table cut in the middle. Nothing said what the ecosystem is for, what it can and
cannot do, which tool does what, how a task is done or what an error has to do with another. The log of `STATUS.md`
(148 KB) was read by the local agents. And nothing kept the base in step with the code: it was rebuilt when someone
remembered.

The user's requirements: a library that knows the theory, the documentation, the rules, the purpose and every tool;
what Nuclear can and cannot do; every error and how errors relate; organised as JSON assets with their metadata in
Chroma collections designed for what Chroma can do; with versions, updated with every feature; tested with the
questions people ask ("how does the MCP work", "what is nuclear", "advantages"); good enough to replace the markdown
files one day.

What the installed Chroma (1.5.9) gives, checked on a scratch database: collections with their own metadata that
survives a restart; rows with a vector, a document and metadata of texts, numbers, booleans and **lists of texts**;
filters `$eq $ne $gt $gte $lt $lte $in $nin $and $or` and, on a list, `$contains`; a query restricted to given ids;
lookup by id. Its full-text filters read the document, which this graph stores encrypted, so they are of no use here;
it has no keyword index of its own. An empty list is refused.

## Decision

1. **Shelves.** One Chroma collection per kind of question, each with its own row types and filters
   (`nuclear_graph/shelves.py` is the schema; the base carries it as `schema.json`):

   | Shelf | The question it answers | Row types |
   |---|---|---|
   | `concepts` | What is it, why does it exist, how does it work? | purpose, concept, theory, architecture, comparison, faq, term |
   | `language` | How is it written in Nuclear? | guide, rule, library, reference |
   | `platforms` | What does a platform have, and where does each thing go? | platform, kind, decorator, layout, plugin, spec |
   | `tools` | What can run, and how is it called? | cli, mcp, http, generator, stage, agent, editor, service, component |
   | `errors` | What went wrong, and how is it fixed? | diagnostic, habit, failure, pitfall |
   | `capabilities` | Can it be done, by NuclearLang or by this agent? | can, cannot, limit, agent-limit |
   | `playbooks` | How is this task done, step by step, and by whom? | playbook, role, skill |
   | `examples` | What does the code look like? | file, solved |
   | `docs` | What does the full document say? | spec, adr, context, guide, readme, changelog, status, doc |

   `prompts` (the prompt files of the graph's package) and `memory` (what an agent keeps about a project) stay as
   they were. The collections `knowledge` and `nuclear` are removed by the first ingest.

2. **A row is a card.** `id` (`<prefix>:<name>`: `error:N0114`, `kind:backend.usecase`, `tool:dragonfly.gen.resource`,
   `concept:mcp`; a prefix belongs to one shelf, so an id is fetched without a search), `type`, `title`, `summary`
   (the answer in a sentence or two), `body` (the whole card), `keys` (exact names in lower case), `asks` (questions
   it answers, as people write them), `tags`, `platforms`, `agents`, `links` (`{rel, to}`), `sources`, `meta` (the
   shelf's own filters), `data`.
   - In Chroma, the metadata (in clear, as the vectors are): `card`, `facet`, `type`, `title`, `agents`, `platforms`
     (`["all"]` when the row is of every platform), `keys`, `tags`, `links` (the ids it points to, so "who links to
     X" is a filter), `origin`, `source`, `hash`, `rev`, and the shelf's own. The document (encrypted): the card.
   - What is embedded is not the document: the title goes in the embedding model's title slot, with the summary and
     the first 1200 characters of the body as its text. Each `ask` is one more entry of the collection (`facet: ask`)
     that points to its card: a question finds the card whose author wrote that question.

3. **Search is by meaning and by exact name, over several shelves** (`retrieve.py`). The query is embedded once and
   every shelf asked answers its closest rows. The runs of one to four words of the query are looked up in the rows'
   `keys` (`$contains`): a row that carries one is brought in and moves up (0.12 for a word, 0.22 for a longer name or
   a code), so `N0114`, `gen resource`, `usecase` or `mcp` reach their card whatever the rest of the sentence says and
   whatever its language. Entries are ranked together, each card once; a `docs` row yields to a card (0.04) and a
   document gives at most two sections. `expand` adds to each card the title and summary of the rows it links to.
   Filters: `agents`, `platform`, and any metadata (`type`, `kind`, `code`). A card is also read by id
   (`GET /v1/cards/{id}`, `nuclear-graph card`, MCP `graph_card`).

4. **Two readers, one flag.** `agents: false` marks what only frontier models and people read: the target languages,
   the dialects underneath, the runtimes, the decisions, the state. A row the local agents read holds no code of a
   target language: document sections lose those blocks, and a hand-written row that has one fails the build.

5. **Assets.** The knowledge base (`nuclear-developer/data`, the repository `nuclear-knowledge`) is
   `assets/<shelf>/<name>.json`: `{shelf, origin, about, rows}`; a body is a list of lines.
   - `origin: generated`: written by `nuclear-graph knowledge build` from a structured source, never by hand
     (`extract.py`): the diagnostics (the compiler's `Code` table joined with `spec/diagnostics.md`; a code without
     documentation fails the build), the habits (the base brief's table), the standard library and the names of other
     languages it replaces (`library.rs`), each platform's kinds, decorators, layout and plugins (`platform.toml`,
     the default dialect's `docs.toml` and `dialect.toml`), what each generator writes, the commands of `nuclear`,
     `dragonfly`, `nuclear-loop` and `nuclear-graph` (their help), the MCP tools and instructions, the HTTP routes of
     the graph and the Developer, the Loop's stages, the local agents (their own sources), the editor's commands and
     settings (`package.json`), each component's version, every `.nuc` of the example projects, the validated answers
     of the evaluation's `train` tasks (never the held-out ones), and every markdown file of the tree and the website,
     one row per heading section, on the shelf its section of `sections.toml` names.
   - `origin: curated`: written by hand, because no file says it in one place: the purpose and the advantages, the
     concepts and the theory, the questions people ask, what can and cannot be done, the limits of the local agents,
     the playbooks, the roles and skills, the failures of the tools and services, the mistakes small models repeat.
   - `<name>.overlay.json`: hand-written fields merged into generated rows by id (the relations between diagnostics,
     a right example, who reads it).
   The copies of the tree's files of before (`NN-section/`) are gone: the assets are what the graph ingests.

6. **The base is valid or it is not built.** `build` and `ingest` refuse: a link to a row that does not exist, a
   duplicated id, a prefix or a type of another shelf, a filter the shelf does not have, a missing title, summary or
   body, target-language code in an agents' row, a diagnostic without documentation, a document no section claims.

7. **Versions.** `manifest.json` holds the schema (the row format; it changes when an old row could not be read), the
   revision (one more whenever any row changes, is added or is removed), each row's hash and the revision that last
   changed it, and for each curated row the hash of each of its `sources` (a file, or `file#Heading` for one section)
   at its last review. `revisions.json` is the log. An ingest writes the schema and the revision into the collections'
   metadata; `GET /v1/stats` and `nuclear-graph stats` say which revision the graph holds.

8. **The base never stays behind the code.** `nuclear-graph knowledge check` fails when a generated file is not what
   the tree gives now, when a row is invalid, when the manifest is of other rows, when a source of a curated row
   changed since its review, or when the running graph holds another revision. The end-of-turn hook of the working
   tree (`tools/context/check.sh stop`) runs it: a session that changed the ecosystem cannot finish while it fails.
   The procedure is itself a row (`playbook:update-the-knowledge-base`): build, review the rows whose sources changed
   (update them, or `knowledge review <id>` when they still hold), check, ingest. A row written by hand is reviewed
   when it is written or changed.

9. **What agents write at run time survives.** `POST /v1/documents` writes only `memory` and `examples`, as rows of
   `origin: runtime` that an ingest keeps; the rows of the knowledge base are written by ingest alone.

10. **Retrieval is tested with people's questions.** `questions.toml` of the knowledge base lists questions and
    the row each must return among the first (English and Spanish); `nuclear-graph knowledge ask` runs them against
    the running graph. A question a change answers is added with the change. Every ```nuclear block of a hand-written
    row is compiled by `nuclear-graph/tests/check_examples.py`.

## Consequences

- The local agents stop reading document pieces: a coder gets the card of the kind it writes, validated examples of
  that kind and the pitfalls of that kind; a repair gets the card of each error code by id.
- A new command, MCP tool, kind, decorator, diagnostic, version or example reaches the base by building it. A new
  capability, limit, failure or procedure needs its row written; the check says when an old one must be read again.
- The chat can answer what the ecosystem is for, what it can do and where the agent stops.
- More Python in `nuclear-graph` (`shelves.py`, `assets.py`, `extract.py`, `retrieve.py`): it joins what is to be
  rewritten in Nuclear.

## Limits

- The help of `nuclear`, `dragonfly` and `nuclear-loop` is read from the installed binaries: after changing a CLI,
  install it (`./install.sh`) before building the base.
- Keys are exact: a name written with a typo is found by meaning only.
- A curated row without `sources` is never asked for a review; one whose source is a long section is asked often.
- `STATUS.md` is not in the base: it changes with every task. What matters of it for a reader of the graph (what
  cannot be done yet, what was never run for real) is in `capabilities` rows that name its sections as sources, so a
  change there asks for their review.
