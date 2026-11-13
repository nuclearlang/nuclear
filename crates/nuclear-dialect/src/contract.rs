//! The dialect contract: every template the transpiler renders, the variables it passes, and every
//! option and section a dialect file can use. `nuclear spec` prints this table for agents, and the
//! loader uses it to validate dialect files.

pub struct TemplateSpec {
    pub key: &'static str,
    pub vars: &'static [&'static str],
    pub doc: &'static str,
}

const fn t(key: &'static str, vars: &'static [&'static str], doc: &'static str) -> TemplateSpec {
    TemplateSpec { key, vars, doc }
}

/// `[syntax]`: how each construct of the base language is written in the target language.
/// A target dialect (one that does not extend another) must define all of them.
pub const SYNTAX: &[TemplateSpec] = &[
    // imports
    t(
        "import_names",
        &["module", "names", "clauses"],
        "`use fs/promises{ readFile }` and auto-imports of named symbols",
    ),
    t("import_alias", &["module", "alias", "clauses"], "`use lib/lib as x`, `use * as x from lib`"),
    t(
        "import_default",
        &["module", "default_name", "names", "clauses"],
        "`use express`; also auto-imports when import_names is not defined (whole-library imports)",
    ),
    t("import_file", &["path", "clauses"], "`use \"./setup\"` (a project file)"),
    t(
        "type_imports",
        &["imports"],
        "the imports for the type checker only of a kind with `type_imports` (two entities that refer to each other); optional",
    ),
    t(
        "project_import",
        &["project", "package", "path", "relative"],
        "optional: import of another file of the project (path without extension, relative to the project root; `package`: the folder the project's files go to, as a module)",
    ),
    t("module_path", &["path"], "turns a `use` path that is not a shortcut into the target's module string"),
    t("file_path", &["path"], "turns a `use \"./file\"` path into the target's path"),
    // declarations
    t("export", &[], "prefix for top-level declarations (`export ` in TypeScript); passed as {{export}}"),
    t(
        "class",
        &[
            "name",
            "generics",
            "extends",
            "implements",
            "clauses",
            "clause",
            "modifiers",
            "decorators",
            "export",
            "members",
            "abstract", "doc"],
        "module with code (fields with values, methods with bodies, constructor)",
    ),
    t(
        "interface",
        &["name", "generics", "extends", "members", "decorators", "export", "doc"],
        "module with only attributes and signatures",
    ),
    t("interface_field", &["name", "type", "optional"], "attribute of an interface"),
    t("interface_method", &["name", "generics", "params", "ret", "async"], "signature inside an interface"),
    t(
        "enum",
        &["name", "items", "export", "values", "value_type", "items_list"],
        "enum; items are already rendered with enum_item and joined with \",\\n\" (items_list: the same, one per item)",
    ),
    t("enum_item", &["name", "value"], "`user` or `admin(\"administrator\")`"),
    t(
        "function",
        &[
            "name",
            "generics",
            "params",
            "params_list",
            "ret",
            "ret_info",
            "async",
            "generator",
            "body",
            "export",
            "decorators",
            "decor",
            "deco_args", "doc"],
        "top-level function; generator = the body uses yield. Structured data for code generation: decor (decorator name → {arg, args, items, <named>}), deco_args (named arguments of every decorator), params_list (each with name, type, tinfo, decor, decorated), ret_info (type, list, item, model, primitive, result, err, void, is.<name>)",
    ),
    t(
        "method",
        &[
            "name",
            "generics",
            "params",
            "ret",
            "async",
            "generator",
            "body",
            "decorators",
            "modifiers",
            "mods",
            "override",
            "override_interface",
            "owner",
            "params_list",
            "ret_info",
            "decor",
            "deco_args", "doc"],
        "method with a body; owner = name of the class; params_list, ret_info, decor, deco_args as in function; `mods.<word>` is true for each header word (`{{#mods.get}}`); override = replaces a member of an extended class, override_interface = implements a member of an interface",
    ),
    t(
        "method_signature",
        &[
            "name",
            "generics",
            "params",
            "ret",
            "async",
            "modifiers",
            "mods",
            "abstract",
            "decorators",
            "override",
            "override_interface",
            "owner",
            "params_list",
            "ret_info",
            "decor",
            "deco_args", "doc"],
        "method without a body inside a class; `mods`, owner, params_list, ret_info, decor, deco_args as in method",
    ),
    t(
        "field",
        &["name", "type", "default", "optional", "modifiers", "decorators", "uninitialized", "override", "decor", "in_init", "tinfo", "decorated"],
        "attribute of a class; uninitialized = no value, not optional and not set by the constructor; override = it replaces a member of a base",
    ),
    t(
        "constructor",
        &["class_name", "params", "body", "initializer"],
        "constructor; initializer = the leading super(...) call when options.super_in_initializer",
    ),
    t(
        "super_param",
        &["name", "type", "default", "optional", "named", "required", "decorators"],
        "optional: a constructor parameter passed as is to super(...) (Dart: `super.name`)",
    ),
    t("file_prelude", &[], "optional: the first lines of every generated file, before the imports (Python: `from __future__ import annotations`)"),
    t(
        "field_init",
        &["name", "value"],
        "optional (options.init_fields): one line of the constructor that sets a field (Python: `self.name = value`)",
    ),
    t("super_ctor", &["args"], "optional: `super(args)` in a constructor, when the target writes it otherwise (Python: `super().__init__(args)`)"),
    t(
        "local_function",
        &["name", "params", "body", "async"],
        "optional (options.hoist_block_lambdas): the local function a lambda with a block becomes",
    ),
    t(
        "super_call",
        &[],
        "optional: first line of the constructor of a class that extends another, when the body does not call super(...) (TS: `super();`)",
    ),
    t(
        "ctor_param",
        &["name", "type", "default", "optional", "named", "required", "decorators"],
        "constructor parameter (it is also a field)",
    ),
    t(
        "param",
        &["name", "type", "default", "optional", "named", "required", "decorators"],
        "parameter of a function, method or lambda",
    ),
    t("params_optional_group", &["params"], "optional: wraps trailing optional positional parameters (Dart `[...]`)"),
    t("params_named_group", &["params"], "optional: wraps parameters written inside `{ }` (Dart `{...}`)"),
    t(
        "decorator",
        &["name", "args", "has_args", "pascal"],
        "`@Get(\":id\")`, `@override`; has_args is true when the decorator has parentheses, pascal when its name starts with an uppercase letter (a class)",
    ),
    t("generics", &["names"], "`<T, U>`; names are already rendered with generic_param"),
    t("generic_param", &["name", "bound"], "one generic: `T` or `T: Comparable` (bound)"),
    t("main", &["body"], "entry point that receives top-level statements when options.toplevel_statements = \"main\""),
    t("comment", &["text"], "one line of a comment"),
    t(
        "doc_comment",
        &[
            "summary",
            "params",
            "returns",
            "throws",
            "examples",
            "see",
            "deprecated",
            "deprecated_text",
            "deprecated_literal",
            "since",
            "other",
            "has_tags",
            "one_line",
        ],
        "optional: the doc comment of a declaration (base §2.3). summary, throws and see are lists of lines; params = [{name, text}]; examples = [{caption, lines}]; other = [{tag, text}]; deprecated_literal: the reason as a string literal of the target; has_tags: there is a summary and at least one tag; one_line: only one summary line. Without it a doc comment is written with `comment`",
    ),
    // statements
    t("const_decl", &["name", "type", "value"], "variable that is never reassigned"),
    t("let_decl", &["name", "type", "value"], "variable that is reassigned later"),
    t(
        "declare",
        &["name", "type"],
        "variable declared before a block because it is assigned inside the block and used after it",
    ),
    t("assign", &["target", "op", "value"], "assignment; op is =, +=, -=, *=, /= or ??="),
    t(
        "pattern_assign",
        &["target", "value"],
        "destructuring into existing variables: `[a, b] = [b, a]`; a declaration uses const_decl/let_decl with the pattern as name",
    ),
    t("pattern_ignore", &[], "`_` inside a list pattern"),
    t("expr_stmt", &["expr"], "expression used as a statement"),
    t("return", &["value"], "return (value may be empty)"),
    t("if", &["cond", "body"], "if"),
    t("else_if", &["cond", "body"], "appended after if"),
    t("else", &["body"], "appended after if / else_if"),
    t("for_in", &["var", "iter", "body"], "`for x in xs`"),
    t("for_range", &["var", "start", "end", "body"], "`for i in 0..10` (end excluded)"),
    t("for_entries", &["key", "value", "iter", "body", "map"], "`for key, value in map` (`map`: the value is a Map object, not a map literal)"),
    t("while", &["cond", "body"], "while"),
    t("try", &["body", "catch", "catch_var", "catch_used", "catch_body", "finally", "finally_body"], "try / catch / finally"),
    t("throw", &["value"], "throw"),
    t("break", &[], "break"),
    t("yield", &["value"], "`yield value` in a generator"),
    t("yield_all", &["value"], "`yield ...values`: every value of another generator"),
    t("continue", &[], "continue"),
    // expressions
    t("self_member", &["name"], "access to a module member: `this.name`"),
    t("this", &[], "`this`"),
    t("super", &[], "`super`"),
    t("null", &[], "`null`"),
    t("true", &[], "`true`"),
    t("false", &[], "`false`"),
    t("list", &["items"], "`[1, 2, 3]`"),
    t("list_if", &["cond", "value"], "`if cond value` inside a list"),
    t(
        "cascade",
        &["target", "sections"],
        "an object followed by `..member = value` lines; sections is a list, each rendered from cascade_self",
    ),
    t("cascade_self", &[], "the object inside a cascade section (empty in Dart: `..color`)"),
    t("object", &["entries"], "`{ a: 1 }` / `{\"k\": v}`; entries is empty for `{}`"),
    t("object_entry", &["key", "value"], "one entry; key is already rendered"),
    t("object_key_ident", &["key"], "key written as a name: `role`"),
    t("object_key_string", &["key"], "key written as a string: `\"julian\"` (key holds the raw text)"),
    t("object_shorthand", &["key"], "`{ text }`"),
    t("object_spread", &["expr"], "`{ ...base }`"),
    t("call", &["callee", "args"], "function call"),
    t("construct", &["callee", "args"], "call of a PascalCase name: builds an object"),
    t("named_arg", &["name", "value"], "`name = value` in a call when options.named_args = \"native\""),
    t("spread_arg", &["expr"], "`...list` in a call"),
    t("member", &["object", "name"], "`a.b`"),
    t("optional_member", &["object", "name"], "`a?.b`"),
    t("index", &["object", "index", "map"], "`a[i]`; `map`: the object is a map (`m[k]`, null when absent)"),
    t("await", &["expr"], "`~call()`"),
    t("ternary", &["cond", "then", "else"], "`c ? a : b`"),
    t("is", &["expr", "type", "t"], "`x is User` (`t`: the type's info, as `tinfo`)"),
    t("cast", &["expr", "type"], "`value as Type`"),
    t("non_null", &["expr"], "`value!`: the value is not null"),
    t("lambda", &["params", "names", "body", "async"], "lambda with a block (names: the parameters without types)"),
    t("lambda_expr", &["params", "names", "expr", "async"], "lambda whose body is one expression (names: the parameters without types)"),
    // Result<T, E> (all optional: a dialect without them cannot transpile code that uses Result)
    t(
        "result_runtime",
        &[],
        "optional: the Result type with Ok and Err, written once after the imports of a file that uses it",
    ),
    t("result_ok", &["value"], "optional: `Ok(value)`; value is empty for `Ok()`"),
    t("result_err", &["value"], "optional: `Err(error)`"),
    t("result_is_ok", &["result", "ok", "err"], "optional: test of a match on a Result: `result.isOk`"),
    t("result_is_err", &["result", "ok", "err"], "optional: test of a match whose first arm is Err: `result.isErr`"),
    t("result_value", &["result", "ok", "err"], "optional: the value of a Result that is Ok: `result.value`"),
    t("result_error", &["result", "ok", "err"], "optional: the error of a Result that is Err: `result.error`"),
    t(
        "result_propagate",
        &["result", "ok", "err"],
        "optional: returns the error of `result` from the current function when it is Err (automatic propagation)",
    ),
    t("result_then", &["target", "callback"], "optional: `.ok(cb)` where cb returns Ok/Err: transforms the Ok branch"),
    t("result_tap", &["target", "callback"], "optional: `.ok(cb)` where cb only observes the value"),
    t(
        "result_recover",
        &["target", "callback"],
        "optional: `.error(cb)` where cb returns Ok/Err: recovers or changes the error",
    ),
    t("result_tap_error", &["target", "callback"], "optional: `.error(cb)` where cb only observes the error"),
    t("result_map", &["target", "callback"], "optional: `.ok(cb)` where cb returns a value: the Ok value becomes it"),
    t(
        "result_get_or_else",
        &["target", "callback"],
        "optional: `.error(cb)` where cb returns a value: the error is recovered and the expression is a plain value",
    ),
];

/// `[types]`: base type names and type constructors. A missing simple name is written as is.
pub const TYPES: &[TemplateSpec] = &[
    t("string", &[], "string"),
    t("int", &[], "int"),
    t("double", &[], "double"),
    t("number", &[], "number"),
    t("boolean", &[], "boolean"),
    t("bool", &[], "bool"),
    t("void", &[], "void"),
    t("any", &[], "any"),
    t("dynamic", &[], "dynamic"),
    t("unknown", &[], "type used when a parameter has no type and none can be inferred"),
    t("list", &["item"], "`User[]`"),
    t("map", &["key", "value"], "`<string, User>`"),
    t("nullable", &["type"], "`string?`"),
    t("union", &["types"], "`'a' | 'b'`; types is a list (use join). Identical members are merged first"),
    t("string_literal", &["value"], "`'user'` as a type"),
    t("number_literal", &["value"], "`42` as a type"),
    t("function", &["params", "types", "ret"], "`(a: int) => int`"),
    t("generic", &["name", "args"], "`Promise<string>`"),
    t("async", &["type"], "return type of an async function: `Promise<T>`, `Future<T>`"),
    t("generator", &["type"], "return type of a generator; type is the type of each value: `Iterable<T>`"),
    t("async_generator", &["type"], "return type of an async generator: `Stream<T>`"),
];

/// `[operators]`: a symbol (`"&&"`) or a template with `{{left}}`/`{{right}}` (unary: `{{expr}}`).
pub const OPERATORS: &[TemplateSpec] = &[
    t("and", &["left", "right"], "and"),
    t("or", &["left", "right"], "or"),
    t("not", &["expr"], "not (unary)"),
    t("neg", &["expr"], "- (unary)"),
    t("eq", &["left", "right", "left_infix", "right_infix", "null"], "=="),
    t("not_eq", &["left", "right", "left_infix", "right_infix", "null"], "!="),
    t("lt", &["left", "right"], "<"),
    t("gt", &["left", "right"], ">"),
    t("lt_eq", &["left", "right"], "<="),
    t("gt_eq", &["left", "right"], ">="),
    t("add", &["left", "right"], "+"),
    t("sub", &["left", "right"], "-"),
    t("mul", &["left", "right"], "*"),
    t("div", &["left", "right"], "/"),
    t("rem", &["left", "right"], "%"),
    t("int_div", &["left", "right", "left_arg", "right_arg", "int", "double"], "`~/` (integer division)"),
    t("pow", &["left", "right", "left_arg", "right_arg", "int", "double"], "`**` (power; `int`: both operands are integers, `double`: one is a decimal)"),
    t("bit_and", &["left", "right"], "&"),
    t("bit_or", &["left", "right"], "|"),
    t("bit_xor", &["left", "right"], "^"),
    t("bit_not", &["expr"], "`^x` (bitwise complement, unary)"),
    t("shl", &["left", "right"], "<<"),
    t("shr", &["left", "right"], ">>"),
    t("ushr", &["left", "right"], ">>> (unsigned shift)"),
    t("coalesce", &["left", "right"], "??"),
    t(
        "coalesce_once",
        &["left", "right", "left_arg", "right_arg"],
        "`??` whose left side is not a plain name or member chain (a call, a map read): optional, for a target whose `coalesce` writes the left side twice (Python); usually a `{ template, helper }`",
    ),
    t("in", &["left", "right", "map", "set", "object"], "`x in list`; `map`: the right side is a Map object, `set`: a Set, `object`: a map literal or `<K, V>` (`key in map`)"),
    t("range", &["left", "right"], "`a..b` used as a value (end excluded)"),
];

/// Variables of a `[kinds.<word>]` template.
pub const KIND_VARS: &[&str] = &[
    "name",
    "source_name",
    "class_name",
    "members",
    "body",
    "methods_code",
    "fields_required",
    "fields",
    "methods",
    "params",
    "params_list",
    "ctor",
    "extends",
    "implements",
    "clauses",
    "clause",
    "decorators",
    "export",
    "modifiers",
    "generics",
    "generic_list",
    "decor",
    "deco_args",
    "file",
    "project",
    "kind",
    "component", "doc", "field_decor",
];

pub struct OptionSpec {
    pub key: &'static str,
    pub kind: &'static str,
    pub default: &'static str,
    pub doc: &'static str,
}

const fn o(key: &'static str, kind: &'static str, default: &'static str, doc: &'static str) -> OptionSpec {
    OptionSpec { key, kind, default, doc }
}

pub const OPTIONS: &[OptionSpec] = &[
    o("implicit_self", "bool", "false", "members can be used without `this.` in the target (Dart)"),
    o(
        "named_args",
        "\"object\" | \"native\"",
        "\"object\"",
        "named arguments to external calls: grouped in one object (TS) or kept as named arguments (Dart)",
    ),
    o("declare_ctor_fields", "bool", "false", "constructor parameters need their own field declaration (Dart)"),
    o("const_collection_defaults", "bool", "false", "list/map defaults of parameters are written as const (Dart)"),
    o("optional_params_nullable", "bool", "false", "`name?: T` parameters get a nullable type (Dart)"),
    o("number_separators", "bool", "true", "the target accepts 1_000"),
    o("binary_literals", "bool", "true", "the target accepts 0b1010 (false: written in hexadecimal)"),
    o("int_member_parens", "bool", "false", "a member of an integer literal needs parentheses: (5).toString()"),
    o("string_quote", "string", "\"'\"", "quote for strings without interpolation"),
    o("interp_quote", "string", "\"`\"", "quote for strings with interpolation"),
    o("interp_open", "string", "\"${\"", "opens an interpolation"),
    o("interp_close", "string", "\"}\"", "closes an interpolation"),
    o(
        "dollar_escape",
        "\"always\" | \"before_brace\" | \"never\"",
        "\"before_brace\"",
        "how a literal $ is escaped inside strings",
    ),
    o(
        "reserved_escape",
        "\"suffix\" | \"prefix\"",
        "\"suffix\"",
        "renames names that are reserved in the target: name_ or _name",
    ),
    o("interp_prefix", "string", "\"\"", "written before the quote of a string with interpolation (Python: f)"),
    o("interp_brace_escape", "bool", "false", "`{` and `}` of the text of an interpolated string are doubled (Python f-strings)"),
    o("module_separator", "string", "\"/\"", "separator of the parts of a module or project path (Python: .)"),
    o("toplevel_gap", "integer", "1", "blank lines between top-level declarations (Python, PEP 8: 2)"),
    o(
        "hoist_block_lambdas",
        "bool",
        "false",
        "a lambda with a block becomes a local function declared before the statement (`local_function`): the target's lambdas are one expression (Python)",
    ),
    o(
        "init_fields",
        "bool",
        "false",
        "fields with a default and constructor parameters are assigned in the constructor (`field_init`); the `field` template gets in_init (Python)",
    ),
    o("type_args_as_args", "bool | string[]", "false", "the type arguments of a call are its first arguments: inject<Repo>() → inject(Repo); true for every call (Python) or the names of the functions (TypeScript: [\"inject\"])"),
    o(
        "builtin_result",
        "bool",
        "true",
        "false: the parent's result_runtime is not written; Result, Ok and Err come from the dialect's auto_imports (a framework runtime)",
    ),
    o(
        "doc_inside",
        "bool",
        "false",
        "the doc comment of a function, method or class goes inside it: its template gets `doc` (Python docstrings)",
    ),
    o(
        "snake_case_names",
        "bool",
        "false",
        "the camelCase names of the .nuc are written in snake_case (getOne → get_one); names that exist as they are in the type definitions are kept (Python)",
    ),
    o(
        "route_param",
        "string",
        "\"{name}\"",
        "how a path parameter is written in `decor.<decorator>.path`, the first text argument of a decorator with its `{x}` segments rewritten (`name` stands for the parameter): \":name\" for NestJS or Express",
    ),
    o("max_bases", "integer", "1", "how many bases (`: A, B`) a class can extend; 0 = unlimited"),
    o("catch_var", "string", "\"error\"", "variable name for `catch` without a name"),
    o(
        "toplevel_statements",
        "\"allow\" | \"main\"",
        "\"allow\"",
        "where top-level statements go: in place, or inside the `main` template",
    ),
    o("line_width", "integer", "80", "calls, lists and objects longer than this are split one item per line"),
    o("trailing_comma", "bool", "true", "add a comma after the last item of a split call, list or object"),
    o(
        "omit_null_default",
        "bool",
        "false",
        "`= null` defaults of fields and parameters are left out (Dart: null is already the default)",
    ),
    o(
        "interp_bare_names",
        "bool",
        "false",
        "an interpolated plain name is written `$name` instead of `${name}` (Dart)",
    ),
    o(
        "implicit_base",
        "string",
        "\"\"",
        "class every class extends in the target (Dart: Object); with [dialect] defs, overriding its members (toString) is marked",
    ),
    o(
        "super_in_initializer",
        "bool",
        "false",
        "a `super(...)` at the start of a constructor body goes to the initializer list (Dart: `User(this.a) : super(a)`)",
    ),
    o(
        "cascade_statements",
        "bool",
        "false",
        "`x = T()` + `..` lines is written as the declaration and one statement per line (targets without cascades)",
    ),
];

pub fn syntax_spec(key: &str) -> Option<&'static TemplateSpec> {
    SYNTAX.iter().find(|s| s.key == key)
}

/// The full contract as Markdown (for `nuclear spec` and agents).
pub fn markdown() -> String {
    let mut out = String::new();
    let table = |out: &mut String, title: &str, specs: &[TemplateSpec]| {
        out.push_str(&format!("### {title}\n\n| key | variables | what it renders |\n|---|---|---|\n"));
        for s in specs {
            let vars = if s.vars.is_empty() {
                "—".to_string()
            } else {
                s.vars.iter().map(|v| format!("`{v}`")).collect::<Vec<_>>().join(" ")
            };
            out.push_str(&format!("| `{}` | {} | {} |\n", s.key, vars, s.doc.replace('|', "\\|")));
        }
        out.push('\n');
    };
    table(&mut out, "[syntax]", SYNTAX);
    table(&mut out, "[types]", TYPES);
    table(&mut out, "[operators]", OPERATORS);
    out.push_str("### [kinds.<word>] template variables\n\n");
    out.push_str(&KIND_VARS.iter().map(|v| format!("`{v}`")).collect::<Vec<_>>().join(" "));
    out.push_str("\n\n### [library.<base>] (spec/library.md)\n\nOne template per member, as the target writes it; variables: ");
    out.push_str(&crate::library::VARS.iter().map(|v| format!("`{v}`")).collect::<Vec<_>>().join(" "));
    out.push_str(". A member with `helper = \"...\"` gets that function written once per file.\n\n| base | members |\n|---|---|\n");
    for base in crate::library::BASES.iter().chain(crate::library::OBJECTS) {
        let members: Vec<String> = crate::library::MEMBERS
            .iter()
            .filter(|m| m.base == *base)
            .map(|m| if m.property { format!("`{}`", m.name) } else { format!("`{}()`", m.name) })
            .collect();
        out.push_str(&format!("| `{base}` | {} |\n", members.join(" ")));
    }
    out.push_str("\n\n### [options]\n\n| key | type | default | meaning |\n|---|---|---|---|\n");
    for o in OPTIONS {
        out.push_str(&format!("| `{}` | {} | `{}` | {} |\n", o.key, o.kind.replace('|', "\\|"), o.default, o.doc));
    }
    out
}
