---
adr: 0105
title: A strategy for each kind of request: the agent recognises what it is asked and follows the plan of that kind
status: Accepted
scope: nuclear-graph
---

# ADR-0105: A strategy for each kind of request

- Status: Accepted (asked by the user, 2026-11-12: "formulate each strategy for the agent depending on the kind of
  feature: only an interface, a whole system, a whole component, a repository, a class, a refactor, a kind of bug...
  the most support is in the generator, the RAG and the agent's tools")
- Scope: `nuclear-knowledge` (the strategy rows), `nuclear-graph` (the selection, the MCP tool, the rename tool),
  `nuclear-developer` (the analyst), `nuclear-vscode` (the Loop's analysis step), `dragonfly` and the three platforms
  (what the generators do with a resource that exists)
- Date: 2026-11-12

## Context

The local agent treated every request the same way. The planner had one prompt for everything, and the Loop ran the
same nine stages for a whole system, a rename and a compile error: a bug went through *Generate*, *Plan* and *Tests
first* as if it were a feature. The spec's `Use case` field was kept and changed nothing. The knowledge base had 22
playbooks, found only when someone searched for them. And the generators, which write most of a feature without a
model, were reached only by a spec that wrote a `GET <url>` line or a `Resources` section: a request in plain words
("an order has a total and a status") went to a 2B model, which planned badly.

A small model follows steps better than it chooses them. So choosing what to do with a request cannot be the
model's work.

## Decision

1. **A strategy is a row of the knowledge base** (`strategy:<name>`, type `strategy`, on the playbooks shelf): one
   per kind of request a user makes. It holds how the kind is recognised, its plan step by step with the tool of
   every step, and what it asks of a run. Changing how a kind of request is handled is changing a row, never code.

2. **The kinds of request**, in five families:

   | Family | Strategies |
   |---|---|
   | create | `system`, `project`, `component`, `resource-api`, `resource-db`, `client-of-backend`, `interface`, `repository`, `usecase`, `endpoint`, `declaration` |
   | change | `member`, `refactor-rename`, `refactor-move-extract`, `refactor-architecture`, `data-migration`, `configuration`, `remove` |
   | fix | `bug-compile`, `bug-failing-test`, `bug-behaviour`, `bug-runtime`, `bug-data-contract`, `bug-interface`, `bug-environment` |
   | verify and explain | `tests`, `security-review`, `document`, `question`, `operate` |
   | beyond | `beyond` |

3. **What a strategy row says**, besides its card for people and lead models:

   | Field | Meaning |
   |---|---|
   | `data.signals` | regular expressions over the request: its exact signs (a diagnostic code, a test's output, `rename A to B`, a `GET <url>` line) |
   | `meta.triggers` | the words that point to it |
   | `data.target` | `existing`: the request names a declaration the project has; `new`: one it has not |
   | `asks` | how people ask for it, in English and Spanish: found by meaning |
   | `data.mode` | `write`, `repair` (the compiler's errors drive), `fix` (a failing test drives), `review`, `answer`, `commands`, `lead` |
   | `data.stages` | the stages of the Loop it runs; empty: this kind of request is not a run |
   | `data.pieces` | where its pieces come from: `plan` (the planner splits what is left), `declaring` (the files that declare what the request names), `naming` (every file that writes a name: a rename), `none` |
   | `data.generators` | the generators it calls |
   | `data.needs` | the facts it cannot start without, each with its question for the user |
   | `data.lead` | `frontier` when it needs a model that understands several files at once |

4. **The selection asks no model** (`POST /v1/strategy`, `nuclear-graph strategy "<request>"`, MCP
   `graph_strategy`). An exact sign of the request outweighs any closeness of meaning; meaning decides between
   strategies with no exact sign, or the same ones; whether the project has the names the request writes moves the
   score. The answer says which signs chose the strategy. A run of the Loop cannot follow a strategy without stages
   (a question, a command): such a strategy comes first for a run only when an exact sign chose it. A spec may name
   its strategy itself (`- Strategy: usecase`).

5. **The analyst** (`nuclear-developer`, `POST /developer/analyze`) runs before anything is generated or planned,
   in fixed steps: the project's files say which names of the request exist; the graph gives the strategy; the
   strategy gives the stages, the source of the pieces and the facts it needs. A model is asked for one thing only:
   the resources a request in plain words describes (`component | Name | fields | source | actions`). Each line
   becomes a `dragonfly gen resource` call and is kept only if its `--dry-run` accepts it. A fact the request lacks is
   a question for the user and stops the run: nothing is guessed.

6. **Tools before models, in every strategy**: a generator writes what is standard; the compiler's exact fixes
   (`nuclear fix`) come before any repair; a rename is done by a tool (`Workspace.renamed`: every file that writes
   the name as a word, with the new one in its place, and with it the project's names built from it, `NewBook` and
   `BookRepository` for `Book`) and no model writes; the files a change starts in are found
   by the project's declarations, not planned. A model writes only a piece of one file that no tool can write.

7. **The Loop follows the strategy.** The analysis is the last step of the *Specification* stage. Then: the stages
   the strategy has no use for are skipped; *Generate* runs the spec's own resources and the calls the analysis
   derived; *Plan* takes the pieces the tools gave, and asks the planner only when the strategy's pieces come from a
   plan; *Code* writes a piece a tool already wrote as it is, and afterwards wires each use case the coder wrote with
   `dragonfly gen usecase` (its alias and, when the spec's sentence gives it, its route); *Verification*, for a bug,
   repairs the file the failure names whoever wrote it, and never the test. A Developer without an analyst leaves the
   run as it was: every stage.

8. **The classification is tested like the rest of the knowledge base**: `questions.toml` holds requests in English
   and Spanish (`request = true`), none of them among the `asks` of its row; `nuclear-graph knowledge ask` gives each
   to the selection and its first strategy must be the expected one.

9. **A generator finishes what it starts on a resource that exists.** `dragonfly gen resource` with a new action
   adds the repository's method and the controller's route too (they are merged, member by member); it refuses a
   search of a stored resource, because which column is searched is a decision (a query of the repository, wired with
   `dragonfly gen usecase --calls`).

## Consequences

- A request in plain words reaches the generators. "Suppliers with a name and a phone: add, change and remove them"
  becomes `dragonfly gen resource suppliers Supplier --fields "name:string phone:string" --source db --actions
  create,delete,update`, shown in the *Generate* stage before it runs.
- A compile error is repaired without planning; a failing test without generating; a rename without a model.
- A run says which kind of request it understood and why, at its first stage: a wrong reading is seen before any
  file is written, and the spec can name the right strategy.
- A lead model gets the same routing (`graph_strategy`) and the same plans.
- Adding a kind of request, or changing how one is handled, is a row and its test requests.

## Limits

- The selection reads words: a request that names its kind in an unusual way is classified by meaning alone, which a
  small embedding model does less well than an exact sign. Its test set is 62 requests; a wrong reading is fixed by a
  sign or a question in the row.
- `refactor-move-extract`, `remove` and `beyond` need a lead: the local agents do their mechanical steps only.
- A rename by the tool covers a declaration's name and the declarations built from it by a word before or after
  (the platform binds a repository to its entity by its name, so an entity renamed alone compiles and fails when it
  runs). That also takes a declaration that only starts with the name (`OrderLine` with `Order`): the analysis lists
  every name it changes. A field or a method is renamed by hand (the same word may be another declaration's member);
  the file of a renamed declaration keeps its path; a plural (`ListBooks`) and an alias of the facade stay. A rename
  runs no security review: it writes no new logic.
- A bug whose failure names only its test stops at *Verification*: where the cause is needs a lead or the user.
- The Loop writes files and never deletes one.
- Still by hand or by a model, with no generator: relations in a stored resource, the tests of a backend resource
  over another API, what a screen shows of a use case added with `gen usecase` (mobile, web).
- The analyst's one model call (the resources of a plain-words request) is measured by `eval/run.py analyze`: 10 of
  11 cases, 4 of the 5 that ask the model. A model asked about a request that lists no field invents them, so it is
  asked only when the request lists some, and its answer is held to that list: the number of fields must be the
  request's, it is asked once more when it is not, and left out (the fields are then asked to the user) when the
  second answer fails too. The case that fails is that one: a field dropped twice. Types it reads from words (`total`
  as `double`) are shown in the *Generate* step, to be checked there.
