use nuclear_syntax::ast::*;
use nuclear_syntax::parse;

fn ok(src: &str) -> File {
    let parsed = parse(src);
    assert!(parsed.diagnostics.is_empty(), "unexpected errors in:\n{src}\n{:#?}", parsed.diagnostics);
    parsed.file
}

fn codes(src: &str) -> Vec<&'static str> {
    parse(src).diagnostics.iter().map(|d| d.code.id).collect()
}

fn decl(item: &Item) -> &Decl {
    match item {
        Item::Decl(d) => d,
        other => panic!("expected a declaration, found {other:#?}"),
    }
}

fn members(d: &Decl) -> &[Member] {
    match &d.body {
        Body::Members(m) => m,
        other => panic!("expected members, found {other:#?}"),
    }
}

fn block(d: &Decl) -> &[Stmt] {
    match &d.body {
        Body::Block(b) => &b.stmts,
        other => panic!("expected a block, found {other:#?}"),
    }
}

// ---------- declarations ----------

#[test]
fn module_with_methods() {
    let f =
        ok("Users()\n  addDoubles(a: double, b: double)\n    a + b\n  substract(a, b)\n    a - b + addDoubles(a, b)\n");
    let users = decl(&f.items[0]);
    assert_eq!(users.name.name, "Users");
    let m = members(users);
    assert_eq!(m.len(), 2);
    let Member::Method(add) = &m[0] else { panic!() };
    assert_eq!(add.params.as_ref().unwrap().list.len(), 2);
    assert_eq!(block(add).len(), 1);
}

#[test]
fn function_forms() {
    let f = ok(concat!(
        "greet(name: string) string\n  \"Hello ${name}\"\n",
        "add(a: number, b: number) number = a + b\n",
        "log(message: string) void\n  print(message)\n",
        "~load(path: string) string\n  ~readFile(path)\n",
    ));
    let greet = decl(&f.items[0]);
    assert!(greet.ret.is_some());
    assert!(matches!(decl(&f.items[1]).body, Body::Expr(_)));
    assert!(decl(&f.items[3]).is_async);
}

#[test]
fn void_one_liner_is_an_error() {
    assert_eq!(codes("add(a: number, b: number) void = a + b\n"), ["N0020"]);
}

#[test]
fn fields_constructor_and_enum_items() {
    let f = ok(concat!(
        "User\n",
        "  (\n    name: string,\n    email: string\n  )\n    $name = name\n",
        "  role: Role = Role.user\n",
        "  token?: string\n",
        "  isAdmin() boolean\n    role == Role.admin\n",
        "Role\n  user\n  admin(\"administrator\")\n",
    ));
    let user = members(decl(&f.items[0]));
    assert!(matches!(&user[0], Member::Ctor(c) if c.params.list.len() == 2 && c.body.is_some()));
    assert!(matches!(&user[1], Member::Field(fl) if fl.default.is_some()));
    assert!(matches!(&user[2], Member::Field(fl) if fl.optional));
    let role = members(decl(&f.items[1]));
    assert!(matches!(&role[0], Member::EnumItem(e) if e.args.is_empty()));
    assert!(matches!(&role[1], Member::EnumItem(e) if e.args.len() == 1));
}

#[test]
fn interface_signatures() {
    let f = ok("Message\n  role: 'user' | 'assistant'\n  content: string\n  some(name: string) void\n");
    let m = members(decl(&f.items[0]));
    assert!(matches!(&m[2], Member::Method(d) if matches!(d.body, Body::None)));
}

#[test]
fn inheritance_and_implements() {
    let f = ok("Admin: User, Person > Serializable\n  hi()\n    \"Say HI!\"\n");
    let d = decl(&f.items[0]);
    assert_eq!(d.bases.len(), 2);
    assert_eq!(d.implements.len(), 1);
}

#[test]
fn dialect_words_and_clauses() {
    let f = ok(concat!(
        "extension NumberParsing on String\n  parseInt() int\n    int.parse(this)\n",
        "mixin Flyer on Animal\n  fly()\n    print(\"flying\")\n",
        "Bird: Animal with Flyer\n",
        "stateless MyApp\n  build(context: BuildContext) Widget\n    MaterialApp(title = \"Demo\")\n",
    ));
    let ext = decl(&f.items[0]);
    assert_eq!(ext.words[0].name, "extension");
    assert_eq!(ext.name.name, "NumberParsing");
    assert_eq!(ext.clauses[0].word.name, "on");
    let bird = decl(&f.items[2]);
    assert_eq!(bird.bases.len(), 1);
    assert_eq!(bird.clauses[0].word.name, "with");
    assert_eq!(decl(&f.items[3]).words[0].name, "stateless");
}

#[test]
fn decorators() {
    let f = ok(concat!(
        "@Controller(\"users\")\n",
        "UsersController\n",
        "  @Get(\":id\")\n",
        "  find(@Param(\"id\") id: string) User\n",
        "    users.findOne(id)\n",
    ));
    let c = decl(&f.items[0]);
    assert_eq!(c.decorators[0].name, "Controller");
    let Member::Method(find) = &members(c)[0] else { panic!() };
    assert_eq!(find.decorators[0].name, "Get");
    assert_eq!(find.params.as_ref().unwrap().list[0].decorators[0].name, "Param");
}

#[test]
fn inferred_params_method() {
    let f = ok("Sales\n  getSales\n    \"sale by: ${name}\"\n");
    let Member::Method(m) = &members(decl(&f.items[0]))[0] else { panic!() };
    assert!(m.params.is_none());
}

#[test]
fn named_params_group() {
    let f = ok("User({name: string, role: string = \"\"})\n");
    let p = &decl(&f.items[0]).params.as_ref().unwrap().list;
    assert!(p.iter().all(|p| p.named));
}

#[test]
fn map_types() {
    let f = ok("get(name: string) <string, string>\n  {\"name\": \"julian\"}\nusers: <string, User> = {}\n");
    let get = decl(&f.items[0]);
    assert!(matches!(get.ret.as_ref().unwrap().kind, TypeKind::Map(..)));
}

#[test]
fn module_body_rejects_statements() {
    assert_eq!(codes("User\n  localAttr = \"x\"\n"), ["N0017"]);
}

// ---------- statements and expressions ----------

#[test]
fn lambda_with_block() {
    let f = ok(
        "users = (1..10).map(iterator: int)\n  User(name = \"Julian\")\nfiltered = users.filter(user: User)\n  user.birthday < now()\n",
    );
    let Item::Stmt(Stmt::Assign(a)) = &f.items[0] else { panic!() };
    let ExprKind::Call { args, .. } = &a.value.kind else { panic!() };
    let ExprKind::Lambda(l) = &args[0].value.kind else { panic!() };
    assert_eq!(l.params[0].name.name, "iterator");
    assert!(l.params[0].ty.is_some());
}

#[test]
fn call_without_params_and_block() {
    let f = ok("_handleTap() void\n  setState()\n    _active = not _active\n");
    let stmts = block(decl(&f.items[0]));
    let Stmt::Expr(e) = &stmts[0] else { panic!() };
    let ExprKind::Call { args, .. } = &e.kind else { panic!() };
    assert!(matches!(&args[0].value.kind, ExprKind::Lambda(l) if l.params.is_empty()));
}

#[test]
fn anonymous_functions() {
    let f = ok(concat!(
        "Network\n  retry(attempt: int)\n    log = (message: string)\n      print(message)\n    connect()\n",
        "double = (n: int) => n * 2\n",
        "format = (n: int) string\n  \"${n}\"\n",
    ));
    let Member::Method(retry) = &members(decl(&f.items[0]))[0] else { panic!() };
    let Stmt::Assign(a) = &block(retry)[0] else { panic!() };
    assert!(matches!(&a.value.kind, ExprKind::Lambda(_)));
    let Item::Stmt(Stmt::Assign(fmt)) = &f.items[2] else { panic!() };
    let ExprKind::Lambda(l) = &fmt.value.kind else { panic!() };
    assert!(l.ret.is_some());
}

#[test]
fn one_line_lambdas() {
    let f = ok("doubles = numbers.map(n => n * 2)\nadults = users.filter((user: User) => user.age >= 18)\n");
    assert_eq!(f.items.len(), 2);
}

#[test]
fn nested_function_is_an_error() {
    assert_eq!(codes("run()\n  helper(x: int) int\n    x\n"), ["N0016"]);
}

#[test]
fn typed_lambda_param_needs_block() {
    assert_eq!(codes("adults = users.filter(user: User)\n"), ["N0015"]);
}

#[test]
fn range_needs_parentheses() {
    let parsed = parse("users = 1..10.map(i: int)\n  i\n");
    assert_eq!(parsed.diagnostics[0].code.id, "N0014");
    assert_eq!(parsed.diagnostics[0].fix.as_ref().unwrap().replacement, "(1..10).map(i: int)");
}

#[test]
fn named_and_positional_args() {
    let f = ok("user = User(\"Ana\", admin = true)\nres = ~fetch(url, method = \"POST\")\n");
    let Item::Stmt(Stmt::Assign(a)) = &f.items[0] else { panic!() };
    let ExprKind::Call { args, .. } = &a.value.kind else { panic!() };
    assert!(args[0].name.is_none());
    assert_eq!(args[1].name.as_ref().unwrap().name, "admin");
    let Item::Stmt(Stmt::Assign(b)) = &f.items[1] else { panic!() };
    assert!(matches!(b.value.kind, ExprKind::Unary { op: UnaryOp::Await, .. }));
}

#[test]
fn control_flow() {
    ok(concat!(
        "run(x: int)\n",
        "  if x > 0\n    print(\"pos\")\n  else if x < 0\n    print(\"neg\")\n  else\n    print(\"zero\")\n",
        "  for user in users\n    print(user.name)\n",
        "  for i in 0..10\n    print(i)\n",
        "  for key, value in config\n    print(key)\n",
        "  while pending > 0\n    pending -= 1\n",
        "  match event.kind\n    'message' -> process(event)\n    'error' | 'fail' ->\n      log(event.error)\n      retry()\n    _ -> ignore()\n",
        "  try\n    data = ~fetchJson(url)\n  catch error\n    log(error.message)\n  finally\n    close()\n",
        "  label = x > 0 ? \"pos\" : \"neg\"\n",
        "  name = user?.name ?? \"anon\"\n",
        "  ok = x is User and not done\n",
        "  throw Error(\"boom\")\n",
    ));
}

#[test]
fn objects_and_lists() {
    ok(
        "a = { role: \"user\", content: text }\nb = { text }\nc = { ...base, a: 1 }\nd = {\"julian\": User(name = \"j\")}\ne = [1, 2, 3]\n",
    );
}

#[test]
fn use_forms() {
    let f = ok(concat!(
        "use fs/promises{ readFile, writeFile }\n",
        "use * as path from path\n",
        "use \"./setup\"\n",
        "use lib2/lib2 as lib2\n",
        "use lib1/lib1 show foo\n",
        "use greetings/hello deferred as hello\n",
        "use @nestjs/common{ Get }\n",
        "use material\n",
    ));
    let Item::Use(u) = &f.items[0] else { panic!() };
    assert_eq!(u.path, UsePath::Module("fs/promises".into()));
    assert_eq!(u.names.len(), 2);
    let Item::Use(u) = &f.items[5] else { panic!() };
    assert_eq!(u.clauses[0].word.name, "deferred");
    assert_eq!(u.alias.as_ref().unwrap().name, "hello");
    let Item::Use(u) = &f.items[6] else { panic!() };
    assert_eq!(u.path, UsePath::Module("@nestjs/common".into()));
}

#[test]
fn interpolation_is_parsed() {
    let f = ok("x = \"Hi ${user.name}!\"\n");
    let Item::Stmt(Stmt::Assign(a)) = &f.items[0] else { panic!() };
    let ExprKind::Str(pieces) = &a.value.kind else { panic!() };
    assert!(matches!(&pieces[1], StrPiece::Interp(e) if matches!(e.kind, ExprKind::Member { .. })));
}

#[test]
fn reserved_word_as_name() {
    assert_eq!(codes("if = 3\n"), ["N0001"]);
}

#[test]
fn examples_parse_without_errors() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            // generated apps (build/) and hidden folders hold no sources
            let skip = p.file_name().is_some_and(|n| n == "build" || n.to_string_lossy().starts_with('.'));
            if p.is_dir() && !skip {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "nuc") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples")), &mut files);
    for f in files {
        let parsed = parse(&std::fs::read_to_string(&f).unwrap());
        assert!(parsed.diagnostics.is_empty(), "{}: {:#?}", f.display(), parsed.diagnostics);
    }
}
