---
layer: L3
component: nuclear
read-when: you change what code is generated (crates/nuclear-transpile/src/emit.rs or analysis.rs)
---

# The emitter

## Values and declarations

- `const` or `let` depending on whether the name is reassigned in its block (`analysis::reassigned_after`).
- Block scope (`scoped()`): after an if/for/while/match/try its variables go to `Frame.gone`; reading them
  later is N0105. A camelCase name nothing defines is N0106 (`check_defined`), unless there is a whole-module
  `use` or the module inherits from something declared outside (`ModCtx.open`). Known names: locals, members
  (also inherited from bases in the file and a kind's `members`), declarations, imports, `[names] globals`,
  auto-imports, `[functions]`, base types (`int.parse`).
- `if`/`match` are values: `x = if ...` with multi-line branches → declaration + if that assigns the last
  line of each branch (`Tail::Assign`); one expression per branch → ternary. The last if/match of a function
  with a return type → `return` in each branch (`Tail::Return`). `match` becomes if/else or chained ternaries.
- Implicit return of the last line in functions, methods and block lambdas, except with return type `void`
  and in setters. An assignment as last line returns the created variable. A modifier with `void = true`
  (`set`) never returns.
- Implicit `this` (`implicit_self`); in kinds, `param_access` (e.g. `widget.{{name}}`).
- async is automatic when there is a `~call()`.
- `PascalCase(...)` constructs with the `construct` template; `not_constructors` excludes names.
- Named arguments: `named_args = "native"` uses `named_arg`; otherwise they are reordered by position.
- Reserved words escaped from `dialects/<target>/keywords.toml`.
- In a text enum an item without a value takes its own name.

## Imports

`use` shortcuts, auto-imports by free name and a kind's `imports`. Decided after emitting from the words of
the code (`emit::code_words`: no comments, no string text). A module is not imported if another needed
module re-exports it (`[reexports]`).

## Layout and comments

- `line_width` and `trailing_comma`; "hug" when the only argument is a lambda, object or list.
- Comments stay in place with their blank line. A doc comment goes to the optional `doc_comment` template
  (`Emitter::doc_before`; TS JSDoc, Dart `///` + `@Deprecated(...)`).
- Line map: the emitter writes `\u{E000}offset\u{E001}` markers on statements, members, declarations and
  split arguments (`join_items_at`); `Emitter::file` removes them and fills `Output::lines` /
  `BuiltFile::lines` (ADR-0068). The generated text does not change (golden tests prove it).

## Constructs

- Generic calls `X<T>(...)` (`generics` template), spread (`spread_arg`), `x as T` (`cast`), `x!` (`non_null`).
- `method`/`method_signature` get `mods` (one flag per header word): Dart writes `Color get color` with
  `{{#mods.get}}`.
- Destructuring (`destructure`/`pattern`): `[a, ...r] = xs`, `{ a, "k": b } = m`, `_`; existing names →
  `pattern_assign`; a mix → N0107.
- Generators: `yields_block` → `generator` flag, `generator`/`async_generator` types, no implicit return;
  `yield` / `yield_all`.
- Operators (ADR-0086): `[operators]` templates get `left`/`right` (wrapped), `left_arg`/`right_arg` (bare, for
  calls like Dart's `pow(a, b)`), `int`/`double` (`num_kind`: literals, typed locals, module fields, function
  returns) and, for `in`, `map` (a `Map` object) / `object` (a map literal or `<K, V>`). A bitwise operand of a
  comparison gets parentheses (TS binds `==` tighter); `-(a ** b)` under unary minus. `is` gets `t` (the type's
  `tinfo`: TS writes `typeof`/`Number.isInteger`/`Array.isArray`). `for_entries` gets `map`. Constructed
  `Map/Set/List<...>()` locals record their type.
- `try` as the last line returns the last line of `try` and `catch` (`try_stmt` with the tail).
- Dart top level (`toplevel_main`): statements go in a generated `main`; a variable that a declaration uses
  stays top-level; other statements plus an own `main` → N0113.
- `if` in lists → `list_if`. Cascades → `cascade` + `cascade_self` (Dart "", TS "$"); with
  `options.cascade_statements` (TS) `x = T()..a = 1` becomes declaration + statements.

## Inheritance

- A method named like a member of a base in the file → `override` (extended bases; TS `override`, Dart
  `@override`) or `override_interface` (interfaces; Dart only). `inherited_members(_known)` walks the bases in
  the file and the dialect's type definitions (`Dialect.defs`). A base found in neither leaves the module
  `open` (no N0106). `options.implicit_base` (Dart: `Object`) makes `toString()` an override. A base with
  `kind = "interface"` in the defs goes to `implements`. Kinds use `inherits = ["State"]`.
- Constructors of inheriting modules: `super(...)` first in the body; its arguments see the parameters as
  locals. TS: optional `super_call`. Dart: `options.super_in_initializer` moves it to the initializer list and
  `super_param` gives `Dog(super.name, this.breed)`. A parameter named like a base member is not a new field.
- Generic bounds: `generic_param` template.

## Options (`[options]`)

`implicit_self named_args declare_ctor_fields const_collection_defaults optional_params_nullable
number_separators string_quote interp_quote interp_open interp_close dollar_escape reserved_escape max_bases
catch_var toplevel_statements line_width trailing_comma omit_null_default interp_bare_names binary_literals
int_member_parens` (`omit_null_default`, `interp_bare_names`: Dart without `= null` and with `$name`, to avoid
lint warnings; `binary_literals = false`: Dart writes `0b1010` as `0xa`; `int_member_parens`: TS `(5).toString()`).

## Targets without braces (Python, ADR-0089)

`hoist_block_lambdas` (`block_lambda` pushes a `local_function` to `pending`, the lambda is its name `_fnN`),
`init_fields` (`field_inits` + constructor params → `field_init` lines after the super call; a deferred constructor
when written before the fields; the `field` template gets `in_init`; not for a kind with its own `field` nor a
subclass without its own constructor), `super_ctor`, `doc_inside` (`doc_into` stores the rendered doc in
`inner_doc`; `function`/`method`/`class`/kind templates read `doc`), `interp_prefix`/`interp_brace_escape`,
`module_separator` (`sep`), `toplevel_gap` (also the blank lines `tidy` keeps), `type_args_as_args`,
`builtin_result` (`Dialect::syntax` hides `result_runtime`), `file_prelude`, `snake_case_names` (`naming.rs`: renames
the camelCase names of the AST before emitting — the `.nuc` keeps Nuclear's camelCase; names that exist as they are in
the type definitions stay; string text and map keys are data). `code_words` reads f-string `{x}` and
`# ` comments. Decorator named arguments use `named_arg` when `named_args = "native"`. An empty constructor text is
not inserted.

Operator templates also get `left_infix`/`right_infix` (parenthesized only where the operator's precedence needs it)
and, for `eq`/`not_eq`, `null` (a side is `null`: Python `is None`). A file imports an auto-import name only if its
generated code writes it (`written_words`). Methods of a base from the type definitions that return a Result come
from `ClassDef.results` (index.json), so `x = ~repo.find(id)` unwraps; an error type that extends the function's
(`NotFound` of `AppError`, `is_subtype`) propagates without N0109. A kind's `inherits` count as bases.

ADR-0091: `project_import` gets `relative` (from the output file to the import, `ProjectCtx::relative`);
`alias_via = "constructor"` (alias.rs adds the target modules as constructor parameters and calls `field.method`);
`params_list` entries get `route` (named in a decorator's route string, `route_params`); kinds get `field_decor`;
`field` gets `tinfo`; `methods` entries get `doc_text`/`doc_literal`; project templates get `components` and `uses`;
`type_args_as_args` takes a list of functions; N0219 only with native named arguments.

## Checks against the type definitions (ADR-0088)

`check_construct`: N0219 (named argument not in `ClassDef.params`, only when the auto-import of the name comes from
the same package: `def_libraries`) and N0220 (PascalCase call unknown to the project, auto-imports, globals, types and
defs; skipped without type definitions, with an open import or an open module).

## Call sites (logs)

`[call_site.<Module>] arg, text` (`Dialect.call_site`, `Emitter::call_site`/`with_source`): every
`Log.x(...)` gets `at: '<component> › <kind> › <Class.method> · <path>.nuc:<line>'` (ADR-0078).
