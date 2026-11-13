---
adr: 0107
title: Questions with one right answer are answered from exact facts (SQLite) by a query planner; the Smart Graph is written in Nuclear
status: Accepted
scope: nuclear-graph
---

# ADR-0107: Exact facts, a query planner, and the Smart Graph in Nuclear

- Status: Accepted (asked by the user, 2026-11-13: the chat answered "which services does the MCP offer" with part
  of the list and invented commands for "how do I create a use case"; "implement a relational database like SQLite
  for the deterministic queries"; "create the listing in assets as you did for the vector database, now for SQLite";
  "create the planner"; "if nuclear-smartgraph is not made with Nuclear and one of its dialects, migrate it right now";
  "as a rule for every new development: Nuclear and Dragonfly first")
- Scope: `nuclear-graph` (the facts, the planner, its sources), `nuclear-developer` (the assistant, the knowledge base's
  `queries` shelf), `nuclear` (`[project] package`, rule 12 of `AGENTS.md`)
- Date: 2026-11-13

## Context

The local assistant answered every question from the four cards closest to it by meaning. That is right for *what is*
and *how*, and wrong for *which*, *how many* and *what are*: the answer of "every tool of the MCP server" is 27 rows,
and no search by meaning gives all of something. The model (2–4 B parameters) filled what was missing from memory: a
partial list, a command that does not exist. The knowledge was there: the generated rows of the base already carry
each command, tool, parameter, kind and diagnostic as structured fields.

The Smart Graph itself was 6,000 lines of Python written by hand, in an ecosystem whose language transpiles to Python.

## Decision

1. **The exact facts are a relational database** (SQLite, in the standard library of the target). Its tables are the
   rows of the base without their bodies: `cards` (id, shelf, type, title, summary), `platforms`, `keys`, `links`,
   `fields` and `items` (every value of a row's `meta` and `data`, flattened), `about` (the revision). One **view** per
   thing a question can list: `commands`, `mcp_tools`, `mcp_params`, `routes`, `kinds`, `kind_names`, `decorators`,
   `plugins`, `dialects`, `generator_pieces`, `layout`, `diagnostics`, `habits`, `library`, `stages`, `agents`,
   `components`, `editor_commands`, `editor_settings`, `strategies`, `capabilities`. The generated rows carry what the
   views read as structured `data` (a tool's parameters, a command's usage, a generator's pieces).

2. **The database lives in memory only.** The store's rule is encryption at rest; a SQLite file would keep names and
   summaries in clear. So the graph's process makes the database in memory from the cards the store keeps sealed
   (at the first question after it starts, and again at every ingest), marks it read-only (`PRAGMA query_only`), and
   writes nothing of it to disk. It holds no body and nothing an agent kept at run time.

3. **The questions are data: the `queries` shelf** (`assets/queries/exact.json` of the knowledge base, like every other
   shelf). A row is one question with one right answer: the ways people ask it (English and Spanish), its parameters,
   and its SQL over the views. A parameter names the query that lists *what people write* and *the value it means*
   (`generadores` → `generate`, `caso de uso` → `usecase`, `generate resource` → `generate_resource`): no word list
   lives in code. The build runs every query against empty tables and refuses a base whose query does not run, names a
   parameter it does not declare, or declares one it never uses.

4. **The planner asks no model** (`planner.nuc`; `POST /v1/answer`, `nuclear-graph answer`, MCP `graph_answer`). It
   matches the question to the catalogue by meaning (the same embeddings) and by the names it writes: a query whose
   parameter the question names is run from further away (`NAMED`) than one it only resembles (`NEAR`); the thresholds
   were measured with `questions.toml`. **The shelves decide whether a question is exact at all** (2026-11-13): a
   query runs only when its exact question is closer to what was asked than the closest card. A name is a weak sign
   alone, because the things of the ecosystem share names: "que es nuclear lang?" writes the name of a tool and of a
   component, and the thresholds alone answered it with the 21 commands of `nuclear` (measured before the rule: 57 of
   the 107 open questions of `questions.toml` ran a query; after: 17, the ones that name one thing, such as a
   diagnostic's code or where a kind's file goes, and none of the exact ones lost its query). The question's own words fill the parameters; a follow-up that names nothing
   ("describe each of them") takes the names from the user's earlier messages. A needed parameter no word gives is
   not guessed: the query is answered under `needs`. What is left of the question is searched in the shelves as
   always. The answer is `facts` (the rows, whole, with their columns), `knowledge` (the cards) and `needs`.

5. **The rows reach the developer as they are.** The chat calls the planner before the model; the prompt has an
   "Exact facts" section above the documentation (left out whole, with its instructions, when no query ran: a 2B
   model copies instructions about rows it was not given into its answer), and the model explains them. The rows themselves end the answer,
   put there by the assistant's code and never rewritten by the model: measured, a 2B model given 27 rows retold them
   whole in one run and as six groups in the next. The step is in the trace (`exact`: which queries; `recall`: which
   cards by title).

6. **Acceptance.** `questions.toml` has `exact = true` entries: the planner's first query for the question must be
   the expected one (`nuclear-graph knowledge ask`); the ones with `says` are also asked to the assistant
   (`eval/run.py chat`). An open question that reads like an exact one ("what is nuclear", "how does the mcp work")
   has `exact = false`: besides finding its card, the planner must run no query for it.

7. **The Smart Graph is a Nuclear project** (the `python` dialect; `nuclear-graph/nuclear.toml`). Its `.nuc` files
   are the source; `nuclear build` writes the package `build/nuclear_graph` that `pyproject.toml` packages; the
   set-up builds before it installs; the tests are `tests/*.nuc`. `[project] package` of `nuclear.toml` names the
   package the files are emitted into (imports between a project's files are relative to it).

8. **Nuclear and Dragonfly first** (rule 12 of `AGENTS.md`): every new development of the platform is written in
   Nuclear, with Dragonfly when a platform fits. What the language could not express while migrating the graph is in
   `STATUS.md` ("Pending: nuclear") as findings of the compiler. Nothing else is migrated until the user asks.

## Consequences

- "Which tools does the MCP have", "the parameters of `generate_resource`", "how is a use case created", "what does
  N0114 mean", "the stages of the Loop" are answered with every row, the same on every run, whatever the model.
- A new kind of fact is a view; a new question is a row of the catalogue with an entry of `questions.toml`: no code.
- The facts are only as complete as the structured fields of the generated rows: what only a curated card's prose
  says is still found by meaning.
- A small model can still misread a question the rows answer (its explanation may dwell on the wrong rows); the
  rows under it are right and complete.
- An answer with exact facts is long: the model's explanation and then every row.
- A graph that starts makes its facts at the first question (a read of every card of the store, once).
