---
adr: 0089
title: "Python dialect and the generic emitter options it needed"
status: "Accepted"
scope: nuclear
---

# ADR-0089: Python dialect and the generic emitter options it needed

- Status: Accepted
- Scope: `nuclear` (`dialects/python`, `crates/nuclear-transpile`, `crates/nuclear-dialect`, `tools/defs/python`)
- Date: 2026-11-03

## Context

The user asked (2026-11-03) for a dialect with Python completely in its base, made with the dialect system, and the
type definitions of LangChain, LangGraph, FastAPI, Chroma, Postgres, Ollama and OpenAI by default. Python is the first
target whose blocks have no braces, whose lambdas are one expression and whose classes set their fields in
`__init__`: plan 002 D7 said such needs become generic `[options]`, never a case of one language.

## Decision

- **Target:** Python 3.12+ typed, checked with `pyright --strict` (the role of `tsc --strict` and `flutter
  analyze`); PEP 8 layout (4 spaces, 88 columns, two blank lines); PEP 695 generics; docstrings.
- **New generic options and templates** (`nuclear spec` lists them): `hoist_block_lambdas` + `local_function` (a lambda
  with a block becomes a local function before its statement), `init_fields` + `field_init` (constructor parameters
  and fields with a default are set in the constructor: no shared mutable defaults; a subclass without its own
  constructor keeps class defaults), `super_ctor`, `doc_inside` (the doc comment goes to the template's `doc`),
  `interp_prefix` + `interp_brace_escape` (f-strings), `module_separator`, `toplevel_gap`, `type_args_as_args`
  (`inject<Repo>()` → `inject(Repo)`), `builtin_result` (a child uses a library's Result), `snake_case_names`, `file_prelude`
  (`from __future__ import annotations`), template variables `names` (lambda parameters without types), `types`
  (function types), `items_list` (enums), filter `strip_suffix`. A named argument of a decorator is written with
  the dialect's `named_arg`. `code_words` reads f-string interpolations and `#` comments.
- **Result:** the file's runtime (`templates/result.tpl`), with explicit covariant TypeVars (a frozen dataclass is
  invariant from Python 3.13 on because of `__replace__`).
- **Names:** the `.nuc` keeps Nuclear's `camelCase` (spec/nuclear-base.md §2.4; a dialect never changes the
  language); `snake_case_names` writes them in Python's `snake_case` (`getOne` → `get_one`; `naming.rs` renames the
  AST before emitting, keeping names that exist as they are in the type definitions; strings and map keys are data).
  Correction of the first version, which asked for `snake_case` in the `.nuc`: the user rejected it (2026-11-03).
- **Type definitions of Python** come from the installed modules (`tools/defs/python/gen.py`, `inspect`; the venv
  `tools/python/.venv` pinned by `tools/python/requirements.txt`): `python@3.14` (stdlib, builtins implicit) and, for
  `langgraph`, pydantic, pydantic-settings, starlette, fastapi, sqlalchemy, asyncpg, redis, aiokafka, openai, ollama,
  chromadb, langchain_core, langchain, langgraph, langchain_openai, langchain_ollama, langchain_chroma and the runtime.

## Consequences

- `examples/examples-python/tour.nuc` (golden) runs and passes `pyright --strict`; `spec/python.md` has 15 checked
  equivalences.
- The options are available to any later target with the same needs (Ruby, Elixir, Kotlin scripts).
