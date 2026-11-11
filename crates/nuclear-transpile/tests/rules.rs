//! One test per rule of spec/nuclear-base.md and spec/dialects.md, using the dialects in dialects/.

use nuclear_dialect::{Dialect, Loader};

fn dialect(name: &str) -> Dialect {
    let loader = Loader::new(nuclear_dialect::dialect_dirs(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))));
    let (d, report) = loader.load(name);
    d.unwrap_or_else(|| panic!("{name}: {:#?}", report.diags))
}

fn ts(src: &str) -> String {
    build(src, "typescript")
}

fn build(src: &str, d: &str) -> String {
    let out = nuclear_transpile::transpile(src, &dialect(d));
    let errors: Vec<_> = out.diagnostics.iter().filter(|x| x.is_error()).collect();
    assert!(errors.is_empty(), "{src}\n{errors:#?}");
    out.code
}

fn codes(src: &str, d: &str) -> Vec<&'static str> {
    nuclear_transpile::transpile(src, &dialect(d)).diagnostics.iter().map(|x| x.code.id).collect()
}

fn has(code: &str, needle: &str) {
    assert!(code.contains(needle), "expected:\n{needle}\n--- in ---\n{code}");
}

#[test]
fn const_or_let_depends_on_reassignment() {
    let code = ts("run()\n  a = 1\n  b = 2\n  b = b + a\n  b\n");
    has(&code, "const a = 1;");
    has(&code, "let b = 2;");
    has(&code, "b = b + a;");
}

#[test]
fn implicit_return_and_void() {
    let code = ts("twice(x: int) int\n  x * 2\nlog(m: string) void\n  print(m)\n");
    has(&code, "return x * 2;");
    has(&code, "console.log(m);\n}");
    assert!(!code.contains("return console.log"));
}

#[test]
fn last_assignment_is_returned() {
    has(&ts("sum(a: int, b: int)\n  total = a + b\n"), "const total = a + b;\n  return total;");
}

#[test]
fn async_is_inferred_from_tilde() {
    let code = ts("load(url: string) string\n  res = ~fetch(url)\n  ~res.text()\n");
    has(&code, "export async function load(url: string): Promise<string> {");
    has(&code, "const res = await fetch(url);");
    has(&code, "return await res.text();");
}

#[test]
fn tilde_in_declaration_forces_async() {
    has(&ts("~hello(name: string) string\n  \"hi ${name}\"\n"), "async function hello(name: string): Promise<string>");
}

#[test]
fn members_get_this_and_dollar_is_explicit() {
    let code = ts(
        "Counter\n  count: int = 0\n  add(count: int) void\n    $count = $count + count\n  inc() void\n    count += 1\n",
    );
    has(&code, "this.count = this.count + count;");
    has(&code, "this.count += 1;");
}

#[test]
fn dart_members_need_no_this() {
    let code = build("Counter\n  count: int = 0\n  inc() void\n    count += 1\n", "dart");
    has(&code, "count += 1;");
}

#[test]
fn pascal_case_call_constructs() {
    let code = ts("make() User\n  User(\"Ana\")\n");
    has(&code, "return new User('Ana');");
    // `String(x)` is the target's conversion, not a class: Nuclear writes `x.toString()` (N0114)
    assert!(codes("n(x: string) string\n  String(x)\n", "typescript").contains(&"N0114"));
}

#[test]
fn named_args_reorder_for_nuclear_functions() {
    let code = ts(
        "greet(name: string, polite: boolean = false) string\n  name\nrun()\n  greet(polite = true, name = \"Ana\")\n",
    );
    has(&code, "greet('Ana', true)");
}

#[test]
fn named_args_to_external_functions() {
    has(&ts("run(url: string)\n  fetch(url, method = \"POST\")\n"), "fetch(url, { method: 'POST' })");
    has(&build("run(s: TextStyle) void\n  Text(\"hi\", style = s)\n", "flutter"), "Text('hi', style: s)");
}

#[test]
fn interface_is_inferred_from_signatures_only() {
    let code = ts(
        "Shape\n  name: string\n  area() double\nSquare > Shape\n  name: string = \"square\"\n  (side: double)\n  area() double\n    side * side\n",
    );
    has(&code, "export interface Shape {\n  name: string;\n  area(): number;\n}");
    has(&code, "export class Square implements Shape {");
}

#[test]
fn base_that_is_an_interface_goes_to_implements() {
    has(&ts("Shape\n  area() double\nCircle: Shape\n  area() double\n    1.0\n"), "class Circle implements Shape");
}

#[test]
fn single_base_only() {
    assert!(codes("A\n  x: int = 1\nB\n  y: int = 1\nC: A, B\n  z: int = 1\n", "typescript").contains(&"N0208"));
}

#[test]
fn enums() {
    let code = ts("Role\n  user\n  admin(\"administrator\")\n");
    has(&code, "export enum Role {\n  user = 'user',\n  admin = 'administrator'\n}");
}

#[test]
fn undefined_names_are_errors() {
    // no inferred parameters: `page` is not declared anywhere
    let src = "Repo\n  get(name: string, page: int) string\n    name\nUseCase\n  list(name: string) string\n    Repo().get(name, page)\n";
    assert!(codes(src, "typescript").contains(&"N0106"));
    // a typo suggests the closest name
    let out = nuclear_transpile::transpile("f(total: int) int\n  totl + 1\n", &dialect("typescript"));
    assert!(
        out.diagnostics.iter().any(|d| d.code.id == "N0106" && d.message.contains("did you mean 'total'")),
        "{:#?}",
        out.diagnostics
    );
    // names this file cannot see: a whole-module `use`, or a base declared elsewhere
    assert!(!codes("use lib1/lib1\nf() int\n  helper()\n", "dart").contains(&"N0106"));
    assert!(!codes("Painter: ThirdPartyPainter\n  run() void\n    inheritedThing()\n", "flutter").contains(&"N0106"));
    // a base in the dialect's type definitions is known: its members exist, other names do not
    assert!(codes("Painter: CustomPainter\n  run() void\n    inheritedThing()\n", "flutter").contains(&"N0106"));
    assert!(codes("Painter: CustomPainter\n  run() void\n    addListener(() => print('x'))\n", "flutter").is_empty());
    // globals, auto-imports and base types are known
    assert!(codes("f(s: string) int\n  print(s)\n  Json.decode(s)\n  s.toInt()!\n", "flutter").is_empty());
}

#[test]
fn block_variables_end_with_the_block() {
    let src = "pick(a: boolean) string\n  if a\n    r = \"x\"\n  else\n    r = \"y\"\n  r\n";
    assert!(codes(src, "typescript").contains(&"N0105"));
    // each branch has its own `r`
    assert!(
        codes(
            "pick(a: boolean) void\n  if a\n    r = 1\n    print(r)\n  else\n    r = 2\n    print(r)\n",
            "typescript"
        )
        .is_empty()
    );
    assert!(codes("f(xs: int[]) void\n  for x in xs\n    print(x)\n  print(x)\n", "typescript").contains(&"N0105"));
}

#[test]
fn if_and_match_are_values_with_blocks() {
    let code = ts("pick(a: boolean) string\n  r = if a\n    p = \"x\"\n    p + \"!\"\n  else\n    \"y\"\n  r\n");
    has(&code, "let r;\n  if (a) {\n    const p = 'x';\n    r = p + '!';\n  } else {\n    r = 'y';\n  }\n  return r;");
    // one expression per branch: a ternary
    has(&ts("pick(a: boolean) string\n  r = if a\n    \"x\"\n  else\n    \"y\"\n  r\n"), "const r = a ? 'x' : 'y';");
    let code = build(
        "size(n: int) string\n  label = match n\n    0 -> \"zero\"\n    _ ->\n      big = n > 100\n      big ? \"big\" : \"some\"\n  label\n",
        "dart",
    );
    has(
        &code,
        "var label;\n  if (n == 0) {\n    label = 'zero';\n  } else {\n    final big = n > 100;\n    label = big ? 'big' : 'some';\n  }",
    );
    // the last if of a function returns from each branch
    has(
        &ts("pick(a: boolean) string\n  if a\n    p = \"x\"\n    p\n  else\n    \"y\"\n"),
        "const p = 'x';\n    return p;",
    );
}

#[test]
fn mutability_is_decided_per_block() {
    let code = ts("f(a: boolean) void\n  if a\n    n = 1\n    print(n)\n  else\n    n = 2\n    n += 1\n    print(n)\n");
    has(&code, "const n = 1;");
    has(&code, "let n = 2;");
}

#[test]
fn lambdas_with_block_and_one_line() {
    let code = ts(
        "run()\n  xs = list.filter(x: int)\n    x > 1\n  ys = list.map(x => x + 1)\n  list.forEach(x)\n    print(x)\n    print(x)\n",
    );
    has(&code, "list.filter((x: number) => x > 1)");
    has(&code, "list.map((x) => x + 1)");
    has(&code, "list.forEach((x) => {\n    console.log(x);\n    return console.log(x);\n  });");
}

#[test]
fn match_as_statement_and_value() {
    let code = ts(
        "run(e: string) void\n  match e\n    'a' -> print(1)\n    'b' | 'c' ->\n      print(2)\n    _ -> print(3)\n",
    );
    has(&code, "if (e === 'a') {\n    console.log(1);\n  } else if (e === 'b' || e === 'c') {");
    has(&code, "} else {\n    console.log(3);");
    // as the last statement of a function that returns, every arm returns
    has(&ts("pick(e: string)\n  match e\n    'a' -> 1\n    _ -> 2\n"), "} else {\n    return 2;");
    // as a value: a chain of ternaries
    has(
        &ts("size(n: int) string\n  label = match n\n    0 -> \"zero\"\n    _ -> \"many\"\n  label\n"),
        "const label = n === 0 ? 'zero' : 'many';",
    );
}

#[test]
fn ranges_and_for() {
    let code = ts("run(m: <string, int>)\n  for i in 0..3\n    print(i)\n  for k, v in m\n    print(k)\n");
    has(&code, "for (let i = 0; i < 3; i++) {");
    has(&code, "for (const [k, v] of Object.entries(m)) {");
}

#[test]
fn strings_escape_per_target() {
    has(&ts("x = \"it's\"\n"), "const x = 'it\\'s';");
    has(&ts("b = 1\nx = \"a ${b} $c\"\n"), "const x = `a ${b} $c`;");
    has(&build("x = \"cost $5\"\n", "dart"), "final x = 'cost \\$5';");
}

#[test]
fn reserved_words_of_the_target_are_renamed() {
    has(&ts("run()\n  delete = 1\n  delete + 1\n"), "const delete_ = 1;\n  return delete_ + 1;");
}

#[test]
fn function_shortcuts() {
    has(&ts("run()\n  t = now()\n  print(t)\n"), "const t = new Date();");
}

#[test]
fn auto_imports_only_when_used_and_not_declared() {
    let code = build("load(p: string) string\n  ~readFile(p, 'utf8')\n", "nodejs");
    has(&code, "import { readFile } from 'node:fs/promises';");
    let code = build("readFile(p: string) string\n  p\nrun()\n  readFile('x')\n", "nodejs");
    assert!(!code.contains("import"), "{code}");
}

#[test]
fn use_shortcuts_and_module_paths() {
    has(
        &build("use fs/promises{ readFile, writeFile }\n", "nodejs"),
        "import { readFile, writeFile } from 'node:fs/promises';",
    );
    has(&build("use material\n", "flutter"), "import 'package:flutter/material.dart';");
    has(
        &build("use lib2/lib2 as lib2\nuse greetings/hello deferred as hello\nuse lib1/lib1 show foo\n", "dart"),
        "import 'package:greetings/hello.dart' deferred as hello;",
    );
    has(&build("use lib1/lib1 show foo\n", "dart"), "import 'package:lib1/lib1.dart' show foo;");
}

#[test]
fn unknown_use_clause() {
    assert_eq!(codes("use lib show foo\n", "typescript"), ["N0211"]);
}

#[test]
fn unknown_kind_suggests_the_closest() {
    let out = nuclear_transpile::transpile(
        "stateles MyApp\n  build(context: BuildContext) Widget\n    Text('x')\n",
        &dialect("flutter"),
    );
    let d = out.diagnostics.iter().find(|d| d.code.id == "N0201").expect("N0201");
    assert!(d.message.contains("did you mean 'stateless'"), "{}", d.message);
    // the same kind does not exist in typescript
    assert!(codes("stateless MyApp\n  build() Widget\n    x\n", "typescript").contains(&"N0201"));
}

#[test]
fn kind_requires_its_methods() {
    assert!(codes("stateless MyApp\n  title: string = ''\n", "flutter").contains(&"N0209"));
}

#[test]
fn stateful_params_are_read_through_widget() {
    let code = build(
        "stateful Counter(start: int)\n  count: int = 0\n  build(context: BuildContext) Widget\n    Text(\"${start}\")\n",
        "flutter",
    );
    has(&code, "const Counter({super.key, required this.start});");
    has(&code, "final int start;");
    has(&code, "Text('${widget.start}')");
}

#[test]
fn nest_kinds_inject_and_route() {
    // Dragonfly for NestJS: names stay as written, dependencies are injected, @get is Nest's @Get, the route of the
    // controller is its name without the suffix, and a route parameter is read and parsed by itself
    let code = build(
        "service UsersService\n  all() string[]\n    []\ncontroller UsersController(users: UsersService)\n  @get(\":id\")\n  one(id: int) string[]\n    users.all()\n",
        "nestjs",
    );
    has(&code, "import { Controller, Get, Injectable, Param, ParseIntPipe } from '@nestjs/common';");
    has(&code, "@Injectable()\nexport class UsersService {");
    has(
        &code,
        "@Controller('users')\nexport class UsersController {\n  constructor(private readonly users: UsersService) {}",
    );
    has(&code, "@Get(':id')\n  one(@Param('id', ParseIntPipe) id: number): string[] {");
}

#[test]
fn toplevel_statements_go_to_main_in_dart() {
    has(&build("print('hi')\n", "dart"), "void main() {\n  print('hi');\n}");
}

#[test]
fn comments_are_kept_in_place() {
    let code =
        ts("// top\nA\n  // field doc\n  x: int = 1\n  // method doc\n  run() void\n    // inside\n    print(x)\n");
    has(
        &code,
        "/** top */\nexport class A {\n  /** field doc */\n  x: number = 1;\n\n  /** method doc */\n  run(): void {\n    // inside\n    console.log(this.x);",
    );
}

#[test]
fn doc_comments_become_doc_comments_of_the_target() {
    let src = "// loose\n\n// Price with tax.\n * @param amount before tax\n * @return the price\n * @deprecated use total()\n * @example\n *   withTax(1.0)\nwithTax(amount: double) double\n  amount * 1.16\n";
    has(
        &ts(src),
        "// loose\n\n/**\n * Price with tax.\n *\n * @deprecated use total()\n * @param amount before tax\n * @returns the price\n * @example\n * withTax(1.0)\n */\nexport function withTax(",
    );
    has(
        &dart(src),
        "// loose\n\n/// Price with tax.\n///\n/// * [amount]: before tax\n/// * Returns the price\n///\n/// ```nuclear\n/// withTax(1.0)\n/// ```\n@Deprecated('use total()')\ndouble withTax(",
    );
    // a blank line between the comment and the declaration: an ordinary comment
    has(&ts("// note\n\nf() int\n  1\n"), "// note\n\nexport function f()");
}

fn dart(src: &str) -> String {
    build(src, "flutter")
}

#[test]
fn block_lambda_as_named_argument() {
    // `name = (params)` + an indented block, inside the parentheses; the `,` after the block is optional
    let src = "stateful Counter\n  count: int = 0\n\n  build(context: BuildContext) Widget\n    FloatingActionButton(\n      onPressed = ()\n        setState()\n          count += 1\n      child = Icon(Icons.add),\n    )\n";
    let code = dart(src);
    has(&code, "onPressed: () => setState(() {\n        count += 1;\n      }),\n      child: Icon(Icons.add),");
    // the `=>` form still works
    let arrow = src.replace("onPressed = ()\n", "onPressed = () =>\n");
    assert_eq!(dart(&arrow), code);
}

#[test]
fn block_lambda_with_params_and_return() {
    let code = dart(
        "stateless Home\n  build(context: BuildContext) Widget\n    StreamBuilder<int>(\n      stream = Stream.value(1),\n      builder = (context, snapshot)\n        if not snapshot.hasData\n          return Text('-')\n        Text('${snapshot.data}')\n    )\n",
    );
    has(
        &code,
        "builder: (context, snapshot) {\n        if (!snapshot.hasData) {\n          return Text('-');\n        }\n        return Text('${snapshot.data}');\n      },",
    );
}

#[test]
fn one_line_lambda_cannot_assign() {
    assert!(codes("f()\n  g(onSaved = (v) => email = v)\n", "flutter").contains(&"N0013"));
}

#[test]
fn generic_calls() {
    let code = dart(
        "stateless Home\n  build(context: BuildContext) Widget\n    FutureBuilder<string>(future = load(), builder = (context, snapshot) => Text(snapshot.data ?? ''))\n\n  load() Future<string>\n    Future.value('x')\n",
    );
    has(&code, "FutureBuilder<String>(");
    has(&ts("run(a: int, b: int)\n  a < b\n"), "a < b");
    has(&ts("run()\n  key = GlobalKey<FormState>()\n"), "new GlobalKey<FormState>()");
}

#[test]
fn spread_in_lists() {
    has(
        &dart("f(items: string[]) Widget[]\n  [Text('a'), ...items.map(i => Text(i))]\n"),
        "[Text('a'), ...items.map((i) => Text(i)).toList()]",
    );
}

#[test]
fn cast_and_not_null() {
    let code = dart("parse(body: string) int\n  json = Json.decode(body) as <string, dynamic>\n  json['id'] as int\n");
    has(&code, "final json = jsonDecode(body) as Map<String, dynamic>;");
    has(&code, "return json['id'] as int;");
    has(&code, "import 'dart:convert';");
    has(&dart("f(key: GlobalKey<FormState>) bool\n  key.currentState!.validate()\n"), "key.currentState!.validate()");
    // `!` before a value is still `not`
    assert!(codes("f(a: bool) bool\n  !a\n", "flutter").contains(&"N0009"));
}

#[test]
fn dart_getters() {
    let code = dart("Badge(status: int)\n  get label() string\n    'n ${status}'\n");
    has(&code, "String get label {");
    has(&ts("Badge(status: int)\n  get label() string\n    'n ${status}'\n"), "get label(): string {");
}

#[test]
fn stateful_mixins_go_to_the_state() {
    let code = dart(
        "stateful Spinner with SingleTickerProviderStateMixin\n  build(context: BuildContext) Widget\n    Text('x')\n",
    );
    has(&code, "class _SpinnerState extends State<Spinner> with SingleTickerProviderStateMixin {");
}

#[test]
fn dart_style_details() {
    let code = dart(
        "stateful Loader\n  data: string? = null\n\n  build(context: BuildContext) Widget\n    Text('${data} of ${data}s ${widget}')\n",
    );
    has(&code, "String? data;");
    // `$name` unless the next character continues the name
    has(&code, "'$data of ${data}s $widget'");
}

#[test]
fn auto_imports_use_the_smallest_set_of_libraries() {
    // widgets.dart is not imported when material.dart (which re-exports it) is needed
    let code = dart("stateless Home\n  build(context: BuildContext) Widget\n    Scaffold(body = Text('x'))\n");
    has(&code, "import 'package:flutter/material.dart';");
    assert!(!code.contains("widgets.dart"), "{code}");
    let code = dart(
        "stateless Home\n  build(context: BuildContext) Widget\n    CupertinoButton(onPressed = () => HapticFeedback.lightImpact(), child = Text('x'))\n",
    );
    has(&code, "import 'package:flutter/cupertino.dart';\nimport 'package:flutter/services.dart';");
    assert!(!code.contains("material.dart") && !code.contains("widgets.dart"), "{code}");
}

#[test]
fn chain_continues_after_a_lambda_block() {
    let code = dart("names(xs: int[]) string[]\n  xs.map(x: int)\n    y = x * 2\n    '${y}'\n");
    has(&code, "return xs.map((int x) {\n    final y = x * 2;\n    return '$y';\n  }).toList();");
}

#[test]
fn type_is_an_ordinary_name() {
    let code = build("Character\n  id: int\n  type: string\n\n  describe() string\n    '${id} ${type}'\n", "dart");
    has(&code, "late String type;");
    has(&code, "return '$id $type';");
}

#[test]
fn setters_never_return() {
    let src = "Box(c: int)\n  set count(value: int)\n    print(value)\n";
    has(&build(src, "dart"), "set count(int value) {\n    print(value);\n  }");
    has(&ts(src), "set count(value: number) {\n    console.log(value);\n  }");
}

#[test]
fn constructor_body_assigns_the_field() {
    let src = "User\n  (name: string, localAttr: string = \"\")\n    localAttr = \"other value\"\n";
    has(&ts(src), "this.localAttr = 'other value';");
    has(&build(src, "dart"), "{\n    localAttr = 'other value';\n  }");
}

#[test]
fn enum_item_without_value_takes_its_name() {
    has(&ts("Level\n  low(\"bajo\")\n  medium\n"), "low = 'bajo',\n  medium = 'medium'");
    has(&build("Level\n  low(\"bajo\")\n  medium\n", "dart"), "low('bajo'),\n  medium('medium');");
}

#[test]
fn list_destructuring() {
    let src = "f(xs: int[]) void\n  [first, _, ...rest] = xs\n  print(first)\n  print(rest)\n";
    has(&ts(src), "const [first, , ...rest] = xs;");
    has(&build(src, "dart"), "final [first, _, ...rest] = xs;");
}

#[test]
fn map_destructuring() {
    let src = "f(m: <string, int>) void\n  { min, \"max-value\": max } = m\n  print(min + max)\n";
    has(&ts(src), "const { min, 'max-value': max } = m;");
    has(&build(src, "dart"), "final {'min': min, 'max-value': max} = m;");
}

#[test]
fn destructuring_into_existing_names_and_mixing() {
    let src = "f() void\n  p = 1\n  q = 2\n  [p, q] = [q, p]\n";
    has(&ts(src), "([p, q] = [q, p]);");
    has(&build(src, "dart"), "[p, q] = [q, p];");
    assert!(codes("f(xs: int[]) void\n  a = 1\n  [a, b] = xs\n", "typescript").contains(&"N0107"));
    // a pattern variable reassigned later is mutable
    has(&ts("f(xs: int[]) void\n  [a, b] = xs\n  a += b\n"), "let [a, b] = xs;");
}

#[test]
fn generators_from_yield() {
    let src = "contar(n: int) int\n  for i in 0..n\n    yield i\n\ntodos(n: int) int\n  yield ...contar(n)\n";
    let code = ts(src);
    has(&code, "export function* contar(n: number): Generator<number> {");
    has(&code, "yield* contar(n);");
    let code = build(src, "dart");
    has(&code, "Iterable<int> contar(int n) sync* {");
    // a generator never returns its last line
    assert!(!code.contains("return"), "{code}");
    let code = build("ticks() int\n  ~Future.delayed(Duration(seconds = 1))\n  yield 1\n", "dart");
    has(&code, "Stream<int> ticks() async* {");
}

#[test]
fn generic_bounds() {
    let src = "Box<T: Comparable, U>\n  value: T\n  other: U\n";
    has(&ts(src), "interface Box<T extends Comparable, U> {");
    has(&build(src, "dart"), "class Box<T extends Comparable, U> {");
    has(&ts("first<T: Comparable>(xs: T[]) T\n  xs[0]\n"), "function first<T extends Comparable>(xs: T[]): T {");
    // the `<` of a call is written right after the name; with a space it is a comparison
    has(&ts("f(a: int, b: int) boolean\n  a < b\n"), "a < b");
}

#[test]
fn if_inside_lists() {
    let src = "f(loading: boolean, xs: string[]) string[]\n  ['a', if loading 'b', ...xs]\n";
    has(&ts(src), "['a', ...(loading ? ['b'] : []), ...xs]");
    has(&build(src, "dart"), "['a', if (loading) 'b', ...xs]");
}

#[test]
fn cascades() {
    let src = "f() void\n  sb = StringBuffer()\n    ..write('a')\n    ..length = 0\n  print(sb)\n";
    has(&build(src, "dart"), "final sb = StringBuffer()\n    ..write('a')\n    ..length = 0;");
    has(&ts(src), "const sb = new StringBuffer();\n  sb.write('a');\n  sb.length = 0;");
    // as a value inside an expression, TypeScript uses a function that returns the object
    has(&ts("f() void\n  print(StringBuffer()\n    ..write('a'))\n"), "(($) => {\n");
    // a range still works
    has(&ts("f() int[]\n  (1..3).map(i => i)\n"), "Array.from");
}

#[test]
fn overrides_of_bases_declared_in_the_file() {
    let src = "Shape\n  area() double\n\nSerializable\n  toJson() string\n\nabstract Base\n  name() string\n    'base'\n\nCircle: Base > Shape, Serializable\n  (radius: double)\n  name() string\n    'circle'\n  area() double\n    3.14 * radius * radius\n  toJson() string\n    'c'\n  extra() int\n    1\n";
    let dart = build(src, "dart");
    has(&dart, "@override\n  String name() {");
    has(&dart, "@override\n  double area() {");
    has(&dart, "@override\n  String toJson() {");
    assert!(!dart.contains("@override\n  int extra()"), "{dart}");
    // TypeScript marks only members of an extended class (override on an interface member is an error)
    let code = ts(src);
    has(&code, "override name(): string {");
    has(&code, "  area(): number {");
    assert!(!code.contains("override area") && !code.contains("override toJson"), "{code}");
}

#[test]
fn constructors_of_derived_classes_call_super() {
    let src = "abstract Base\n  (n: int)\n  name() string\n    'b'\nC: Base\n  (r: int)\n    super(r)\n    print(r)\n";
    // TypeScript: the call stays first; its arguments are the parameters (this is not ready yet)
    has(&ts(src), "constructor(public r: number) {\n    super(r);\n    console.log(this.r);\n  }");
    // Dart: the call goes to the initializer list
    has(&build(src, "dart"), "C(this.r) : super(r) {\n    print(r);\n  }");
    // without super(...), TypeScript calls the base constructor without arguments
    has(
        &ts("abstract Base\n  name() string\n    'b'\nC: Base\n  (r: int)\n"),
        "constructor(public r: number) {\n    super();\n  }",
    );
}

#[test]
fn inherited_constructor_parameters_are_not_new_fields() {
    let src = "Animal\n  (name: string)\n\nDog: Animal\n  (name: string, breed: string)\n    super(name)\n\nBird: Animal\n  (wings: int)\n    super('bird')\n";
    let code = ts(src);
    has(&code, "constructor(name: string, public breed: string) {\n    super(name);\n  }");
    let dart = build(src, "dart");
    // passed as is, in order: Dart super parameters
    has(&dart, "class Dog extends Animal {\n  String breed;\n\n  Dog(super.name, this.breed);");
    has(&dart, "Bird(this.wings) : super('bird');");
}

#[test]
fn type_definitions_mark_overrides_of_external_classes() {
    // CustomPainter comes from dialects/mobile/flutter/defs: paint and shouldRepaint override it
    let code = dart(
        "Ring: CustomPainter\n  paint(canvas: Canvas, size: Size) void\n    print(size)\n  shouldRepaint(old: CustomPainter) bool\n    true\n  extra() int\n    1\n",
    );
    has(&code, "@override\n  void paint(Canvas canvas, Size size) {");
    has(&code, "@override\n  bool shouldRepaint(CustomPainter old) {");
    assert!(!code.contains("@override\n  int extra()"), "{code}");
    // every Dart class extends Object (options.implicit_base): toString overrides it
    has(&build("Point\n  x: int = 0\n  toString() string\n    'p'\n", "dart"), "@override\n  String toString() {");
    // kinds take their members and overrides from the classes they inherit (State)
    let code = dart(
        "stateful Counter\n  count: int = 0\n  initState() void\n    super.initState()\n  build(context: BuildContext) Widget\n    Text('${count} ${mounted}')\n",
    );
    has(&code, "@override\n  void initState() {");
    has(&code, "@override\n  Widget build(BuildContext context) {");
}

#[test]
fn external_interface_classes_are_implemented() {
    // Comparable is an `interface class` in the Dart definitions: it cannot be extended
    let code = dart("Score(value: int): Comparable<Score>\n  compareTo(other: Score) int\n    value - other.value\n");
    has(&code, "class Score implements Comparable<Score> {");
    has(&code, "@override\n  int compareTo(Score other) {");
}

// ---------- Result<E, T> (spec/nuclear-base.md §3.2) ----------

const FIND: &str = "User(name: string)\nfindUser(id: string) Result<string, User>\n  if id == ''\n    return Err('empty')\n  Ok(User(id))\n";

#[test]
fn result_runtime_is_written_once_when_used() {
    let code = ts(FIND);
    assert_eq!(code.matches("export class Result<E, T>").count(), 1, "{code}");
    has(&code, "return Err('empty');");
    has(&code, "return Ok(new User(id));");
    assert!(code.find("class Result").unwrap() < code.find("class User").unwrap(), "{code}");
    let dart = build(FIND, "dart");
    // Dart uses fpdart: Result<E, T> is Either<E, T>, Ok is Right and Err is Left; no runtime is written
    has(&dart, "import 'package:fpdart/fpdart.dart';");
    has(&dart, "Either<String, User> findUser(String id) {");
    has(&dart, "return Left('empty');");
    assert!(!dart.contains("class Result"), "{dart}");
    // without Result there is no runtime
    assert!(!ts("f() int\n  1\n").contains("class Result"));
}

#[test]
fn result_errors_propagate_automatically() {
    let src = format!(
        "{FIND}buy(id: string) Result<string, string>\n  user = findUser(id)\n  findUser('x')\n  Ok(user.name)\n"
    );
    let code = ts(&src);
    has(
        &code,
        "const userResult = findUser(id);\n  if (userResult.isErr) return Err(userResult.error);\n  const user = userResult.value;",
    );
    // a call whose value is not used: only its error propagates
    has(
        &code,
        "const findUserResult = findUser('x');\n  if (findUserResult.isErr) return Err(findUserResult.error);\n  return Ok(user.name);",
    );
    let dart = build(&src, "dart");
    has(
        &dart,
        "final userResult = findUser(id);\n  if (userResult case Left(:final value)) return Left(value);\n  final user = (userResult as Right<String, User>).value;",
    );
}

#[test]
fn result_inside_expressions_is_unwrapped_before_the_statement() {
    let src = format!(
        "{FIND}Pair(a: User, b: User)\nboth(x: string, y: string) Result<string, Pair>\n  Ok(Pair(findUser(x), findUser(y)))\n"
    );
    has(
        &ts(&src),
        "const findUserResult = findUser(x);\n  if (findUserResult.isErr) return Err(findUserResult.error);\n  const findUserResult2 = findUser(y);\n  if (findUserResult2.isErr) return Err(findUserResult2.error);\n  return Ok(new Pair(findUserResult.value, findUserResult2.value));",
    );
}

#[test]
fn result_typed_variables_and_results_returned_are_kept() {
    let src = format!(
        "{FIND}keep(id: string) Result<string, User>\n  r: Result<string, User> = findUser(id)\n  findUser(id)\n"
    );
    let code = ts(&src);
    has(&code, "const r: Result<string, User> = findUser(id);\n  return findUser(id);");
    // a parameter typed Result receives it as it is
    let src =
        format!("{FIND}show(r: Result<string, User>) string\n  'x'\nrun(id: string) string\n  show(findUser(id))\n");
    has(&ts(&src), "return show(findUser(id));");
}

#[test]
fn result_ok_and_error_callbacks_observe_or_transform() {
    let src = format!(
        "{FIND}log(m: string) void\n  print(m)\nbuy(id: string) Result<string, string>\n  a = findUser(id).error(e => log(e))\n  b = findUser(id).error(e => Ok(User('guest')))\n  c = findUser(id).ok(u => Ok(u.name))\n  d = findUser(id).ok(u => log(u.name))\n  Ok(c)\n"
    );
    let code = ts(&src);
    has(&code, "findUser(id).inspectErr((e) => log(e))");
    has(&code, "findUser(id).orElse((e) => Ok(new User('guest')))");
    has(&code, "findUser(id).andThen((u) => Ok(u.name))");
    has(&code, "findUser(id).inspect((u) => log(u.name))");
    // a callback that returns Ok/Err in some branches only
    let bad = format!(
        "{FIND}f(id: string) Result<string, User>\n  findUser(id).error(e)\n    if e == ''\n      Ok(User('a'))\n    else\n      print(e)\n"
    );
    assert!(codes(&bad, "typescript").contains(&"N0111"));
}

#[test]
fn result_callbacks_that_return_a_result_propagate_inside() {
    let src = format!(
        "{FIND}rename(id: string, to: string) Result<string, User>\n  findUser(id).ok(u: User)\n    other = findUser(to)\n    Ok(User(other.name))\n"
    );
    has(
        &ts(&src),
        "return findUser(id).andThen((u: User) => {\n    const otherResult = findUser(to);\n    if (otherResult.isErr) return Err(otherResult.error);",
    );
}

#[test]
fn result_match_binds_ok_and_err() {
    let src = format!(
        "{FIND}show(id: string) string\n  match findUser(id)\n    Ok(user) -> 'hi ${{user.name}}'\n    Err(e) -> 'error: ${{e}}'\n"
    );
    has(
        &ts(&src),
        "const findUserResult = findUser(id);\n  if (findUserResult.isOk) {\n    return `hi ${findUserResult.value.name}`;\n  } else {\n    return `error: ${findUserResult.error}`;\n  }",
    );
    let src = format!(
        "{FIND}short(id: string) string\n  label = match findUser(id)\n    Ok(user) -> user.name\n    Err(e) -> e\n  label\n"
    );
    let dart = build(&src, "dart");
    has(
        &dart,
        "final findUserResult = findUser(id);\n  final label = findUserResult.isRight() ? (findUserResult as Right<String, User>).value.name : (findUserResult as Left<String, User>).value;",
    );
    // a variable is matched as it is; Err first
    let src = format!("{FIND}f(r: Result<string, User>) string\n  match r\n    Err(e) -> e\n    _ -> 'ok'\n");
    has(&ts(&src), "if (r.isErr) {\n    return r.error;\n  } else {\n    return 'ok';\n  }");
    // invalid patterns and missing arms
    assert!(
        codes(
            &format!("{FIND}f(id: string) string\n  match findUser(id)\n    Ok(1) -> 'a'\n    _ -> 'b'\n"),
            "typescript"
        )
        .contains(&"N0111")
    );
    assert!(
        codes(
            &format!("{FIND}f(id: string) string\n  x = match findUser(id)\n    Ok(u) -> u.name\n  x\n"),
            "typescript"
        )
        .contains(&"N0103")
    );
}

#[test]
fn result_async_calls_propagate_with_tilde() {
    let src = "User(name: string)\nload(id: string) Result<string, User>\n  ~fetch(id)\n  Ok(User(id))\nbuy(id: string) Result<string, string>\n  user = ~load(id)\n  Ok(user.name)\n";
    let code = ts(src);
    has(&code, "export async function buy(id: string): Promise<Result<string, string>> {");
    has(&code, "const userResult = await load(id);\n  if (userResult.isErr) return Err(userResult.error);");
    // without ~ the value of an async function is not its Result
    let bad = "User(name: string)\nload(id: string) Result<string, User>\n  ~fetch(id)\n  Ok(User(id))\nshow(id: string) string\n  match load(id)\n    Ok(u) -> u.name\n    Err(e) -> e\n";
    assert!(codes(bad, "typescript").contains(&"N0111"));
}

#[test]
fn result_methods_of_modules_propagate() {
    let src = "User(name: string)\nRepo\n  find(id: string) Result<string, User>\n    Ok(User(id))\nShop(repo: Repo)\n  check(id: string) Result<string, int>\n    Ok(1)\n  buy(id: string) Result<string, string>\n    user = repo.find(id)\n    n = check(id)\n    Ok('${user.name} ${n}')\n";
    let code = ts(src);
    has(&code, "const userResult = this.repo.find(id);");
    has(&code, "const nResult = this.check(id);");
    let dart = build(src, "dart");
    has(&dart, "final userResult = repo.find(id);");
}

#[test]
fn result_must_be_handled_outside_result_functions() {
    let ids = |s: &str| codes(&format!("{FIND}{s}"), "typescript");
    assert!(ids("f(id: string) string\n  user = findUser(id)\n  user.name\n").contains(&"N0108"));
    assert!(ids("f(id: string) void\n  findUser(id)\n").contains(&"N0108"));
    assert!(ids("f(id: string) User\n  findUser(id)\n").contains(&"N0108"));
    // handled: the error branch of a chain, a typed variable, a function without a written type
    assert!(!ids("f(id: string) void\n  findUser(id).error(e => print(e))\n").contains(&"N0108"));
    assert!(!ids("f(id: string) void\n  r: Result<string, User> = findUser(id)\n  print(r)\n").contains(&"N0108"));
    assert!(!ids("f(id: string)\n  findUser(id)\n").contains(&"N0108"));
}

#[test]
fn result_error_types_must_match() {
    let src = format!(
        "{FIND}AppError(message: string)\nbuy(id: string) Result<AppError, User>\n  user = findUser(id)\n  Ok(user)\n"
    );
    let out = nuclear_transpile::transpile(&src, &dialect("typescript"));
    let d = out.diagnostics.iter().find(|d| d.code.id == "N0109").expect("N0109");
    assert!(d.message.contains("Err(AppError(e))"), "{}", d.message);
    // converted with .error(e => Err(...)): its error type is the new one
    let ok = format!(
        "{FIND}AppError(message: string)\nbuy(id: string) Result<AppError, User>\n  user = findUser(id).error(e => Err(AppError(e)))\n  Ok(user)\n"
    );
    assert!(!codes(&ok, "typescript").contains(&"N0109"));
}

#[test]
fn result_cannot_propagate_from_lazy_branches() {
    let ids = |s: &str| codes(&format!("{FIND}{s}"), "typescript");
    assert!(
        ids("f(a: boolean, id: string) Result<string, boolean>\n  x = a and findUser(id).name == ''\n  Ok(x)\n")
            .contains(&"N0110")
    );
    assert!(
        ids("f(a: boolean, id: string) Result<string, string>\n  x = a ? findUser(id).name : ''\n  Ok(x)\n")
            .contains(&"N0110")
    );
    assert!(
        ids("f(id: string) Result<string, boolean>\n  while findUser(id).name == ''\n    print(1)\n  Ok(true)\n")
            .contains(&"N0110")
    );
    // the first condition of an if runs before the if: allowed
    assert!(
        ids("f(id: string) Result<string, boolean>\n  if findUser(id).name == ''\n    print(1)\n  Ok(true)\n")
            .is_empty()
    );
}

#[test]
fn a_file_with_its_own_result_does_not_get_the_builtin() {
    let src = "Result\n  ok: boolean\nOk(value: int)\nf() Ok\n  Ok(1)\n";
    let code = ts(src);
    has(&code, "return new Ok(1);");
    assert!(!code.contains("andThen"));
}

const USE_CASES: &str = "Character(name: string)\nGetOne\n  exec(id: int) Character\n    Character('${id}')\n  ~load(id: int) Character\n    Character('${id}')\n  static count(n: int) int = n\n  ids(limit: int) int\n    for i in 0..limit\n      yield i\n";

#[test]
fn alias_is_the_same_function_as_a_module_method() {
    let src = format!(
        "{USE_CASES}Component()\n  getOne() = GetOne.exec\n  load() = GetOne.load\n  count() = GetOne.count\n  ids() = GetOne.ids\n  first() Character\n    getOne(1)\ntop() = GetOne.exec\n"
    );
    let code = ts(&src);
    has(&code, "getOne(id: number): Character {\n    return new GetOne().exec(id);\n  }");
    has(&code, "async load(id: number): Promise<Character> {\n    return await new GetOne().load(id);\n  }");
    // a static method is called on the module
    has(&code, "count(n: number): number {\n    return GetOne.count(n);\n  }");
    has(&code, "*ids(limit: number): Generator<number> {\n    yield* new GetOne().ids(limit);\n  }");
    has(&code, "return this.getOne(1);");
    has(&code, "export function top(id: number): Character {\n  return new GetOne().exec(id);\n}");
    let dart = build(&src, "dart");
    has(&dart, "Character getOne(int id) {\n    return GetOne().exec(id);\n  }");
    has(&dart, "Iterable<int> ids(int limit) sync* {\n    yield* GetOne().ids(limit);\n  }");
}

#[test]
fn alias_keeps_named_parameters_and_results() {
    let src = "Finder\n  find(id: int) Result<string, string>\n    id > 0 ? Ok('x') : Err('bad')\n  greet(name: string, {loud: boolean = false}) string\n    name\nComp()\n  find() = Finder.find\n  greet() = Finder.greet\n  size(id: int) Result<string, int>\n    name = find(id)\n    Ok(name.length)\n";
    let dart = build(src, "dart");
    has(&dart, "String greet(String name, {bool loud = false}) {\n    return Finder().greet(name, loud: loud);\n  }");
    has(&dart, "Either<String, String> find(int id) {\n    return Finder().find(id);\n  }");
    // calls to the alias propagate its errors like calls to the target
    has(&dart, "final nameResult = find(id);\n    if (nameResult case Left(:final value)) return Left(value);");
}

#[test]
fn alias_rules_do_not_change_other_short_functions() {
    // an enum item, a field or a function that is not a method keep their meaning: the function returns it
    let src = "Role\n  user\n  admin\nBox(size: int = 1)\nf() = Role.admin\ng() = Box.size\n";
    let code = ts(src);
    has(&code, "export function f() {\n  return Role.admin;\n}");
    has(&code, "export function g() {\n  return Box.size;\n}");
    // the target module must be buildable with no arguments
    assert_eq!(codes("Repo(url: string)\n  find(id: int) int = id\nf() = Repo.find\n", "typescript"), vec!["N0112"]);
}

#[test]
fn result_callbacks_recover_transform_or_observe_by_what_they_return() {
    let src = format!(
        "{FIND}log(m: string) void\n  print(m)\nfallbackUser() User\n  User('guest')\nbuy(id: string) Result<string, string>\n  a = findUser(id).error(e => log(e))\n  b = findUser(id).error(e => fallbackUser())\n  c = findUser(id).ok(u => u.name)\n  Ok('${{a.name}} ${{b.name}} ${{c}}')\n"
    );
    let dart = build(&src, "dart");
    // nothing returned: it only observes, and the error still propagates
    has(&dart, "final aResult = findUser(id).mapLeft((error) {");
    has(&dart, "if (aResult case Left(:final value)) return Left(value);");
    // a value from .error(): the error is recovered and the expression is that value
    has(&dart, "final b = findUser(id).getOrElse((e) => fallbackUser());");
    // a value from .ok(): it transforms the Ok value
    has(&dart, "final cResult = findUser(id).map((u) => u.name);");
    let code = ts(&src);
    has(&code, "const b = findUser(id).getOrElse((e) => fallbackUser());");
}

#[test]
fn a_kind_that_does_not_write_generics_or_interfaces_says_so() {
    assert!(codes("event Box<T>\n  value: T\n", "flutter").contains(&"N0218"));
    assert!(codes("event Ping > Comparable\n  at: int\n", "flutter").contains(&"N0218"));
    // the plain class and the model kind write both
    assert!(codes("Box<T>\n  value: T\n", "flutter").is_empty());
}
