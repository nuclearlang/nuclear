use nuclear_syntax::lex;
use nuclear_syntax::token::{Keyword, StrPart, TokenKind};

/// Tokens in short form separated by spaces: `if x NL IN ... DE EOF`.
fn toks(src: &str) -> String {
    let lexed = lex(src);
    assert!(lexed.diagnostics.is_empty(), "unexpected errors: {:#?}", lexed.diagnostics);
    lexed.tokens.iter().map(|t| t.kind.short()).collect::<Vec<_>>().join(" ")
}

/// Error codes produced.
fn codes(src: &str) -> Vec<&'static str> {
    lex(src).diagnostics.iter().map(|d| d.code.id).collect()
}

// ---------- indentation ----------

#[test]
fn blocks_by_indentation() {
    let src = "if age >= 18\n  print(\"adult\")\nelse\n  print(\"minor\")\n";
    assert_eq!(toks(src), r#"if age >= 18 NL IN print ( "adult" ) NL DE else NL IN print ( "minor" ) NL DE EOF"#);
}

#[test]
fn several_dedents_at_once() {
    let src = "Users()\n  add(a: double, b: double)\n    a + b\nlog()\n  x\n";
    assert_eq!(
        toks(src),
        "Users ( ) NL IN add ( a : double , b : double ) NL IN a + b NL DE DE log ( ) NL IN x NL DE EOF"
    );
}

#[test]
fn blank_lines_and_missing_final_newline() {
    assert_eq!(toks("a\n\n\n  b\n\nc"), "a NL IN b NL DE c NL EOF");
}

#[test]
fn newlines_ignored_inside_brackets() {
    let src = "x = foo(\n  1,\n  2\n)\ny = 1\n";
    assert_eq!(toks(src), "x = foo ( 1 , 2 ) NL y = 1 NL EOF");
    // the parser can still tell that `2` started on another line
    let lexed = lex(src);
    let two = lexed.tokens.iter().find(|t| t.kind.short() == "2").unwrap();
    assert!(two.nl_before);
}

#[test]
fn method_chain_continues_line() {
    let src = "app = express()\n  .use(cors())\n  .listen(3000)\n";
    assert_eq!(toks(src), "app = express ( ) . use ( cors ( ) ) . listen ( 3000 ) NL EOF");
}

#[test]
fn lambda_block_inside_parens() {
    let src = "r = items.map(x =>\n  y = x * 2\n  y\n)\nz\n";
    assert_eq!(toks(src), "r = items . map ( x => NL IN y = x * 2 NL y NL DE ) NL z NL EOF");
}

#[test]
fn lambda_block_closed_on_same_line_as_paren() {
    let src = "f(x =>\n  x * 2)\n";
    assert_eq!(toks(src), "f ( x => NL IN x * 2 NL DE ) NL EOF");
}

#[test]
fn lambda_with_typed_params_and_block() {
    let src = "adults = users.filter(user: User)\n  user.age >= 18\n";
    assert_eq!(toks(src), "adults = users . filter ( user : User ) NL IN user . age >= 18 NL DE EOF");
}

#[test]
fn anonymous_function_in_body() {
    let src = "retry(attempt: int)\n  log = (message: string)\n    print(message)\n  connect()\n";
    assert_eq!(
        toks(src),
        "retry ( attempt : int ) NL IN log = ( message : string ) NL IN print ( message ) NL DE connect ( ) NL DE EOF"
    );
}

#[test]
fn constructor_on_its_own_line() {
    let src = "User\n  (name: string)\n    $name = name\n  greet()\n    x\n";
    assert_eq!(toks(src), "User NL IN ( name : string ) NL IN $ name = name NL DE greet ( ) NL IN x NL DE DE EOF");
}

#[test]
fn constructor_with_multiline_params() {
    let src = "User\n  (\n    name: string,\n    email: string\n  )\n    $name = name\n  greet()\n    x\n";
    assert_eq!(
        toks(src),
        "User NL IN ( name : string , email : string ) NL IN $ name = name NL DE greet ( ) NL IN x NL DE DE EOF"
    );
}

#[test]
fn hanging_close_paren_is_not_a_block() {
    // `)` indented deeper than the header does not open a block: the body must match a real level
    let src = "Chat(\n    model: string\n  )\n    $role = \"user\"\n  send()\n    x\n";
    assert_eq!(codes(src), ["N0004"]);
}

// ---------- comments ----------

#[test]
fn comments_continue_on_star_lines() {
    let src =
        "// title\n * goes on\n *   more\n *\nUsers()\n  // doc\n   * @param x amount\n  build(x: double) x * 0.16\n";
    let lexed = lex(src);
    assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
    let short: Vec<String> = lexed.tokens.iter().map(|t| t.kind.short()).collect();
    assert_eq!(short.join(" "), "Users ( ) NL IN build ( x : double ) x * 0.16 NL DE EOF");
    let texts: Vec<&str> = lexed.comments.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(texts, ["title\ngoes on\n  more", "doc\n@param x amount"]);
    assert!(lexed.comments.iter().all(|c| c.own_line));
}

#[test]
fn a_comment_line_without_a_star_is_n0021_with_its_fix() {
    // more indented without ` * `, and a `*` that is not under the second `/`
    let src = "// title\n  goes on\n    * wrong column\nx = 1\n";
    let lexed = lex(src);
    let codes: Vec<&str> = lexed.diagnostics.iter().map(|d| d.code.id).collect();
    assert_eq!(codes, ["N0021", "N0021"]);
    assert_eq!(lexed.comments[0].text, "title\ngoes on\nwrong column");
    let mut fixed = src.to_string();
    for d in lexed.diagnostics.iter().rev() {
        let f = d.fix.as_ref().unwrap();
        fixed.replace_range(f.span.start..f.span.end, &f.replacement);
    }
    assert_eq!(fixed, "// title\n * goes on\n * wrong column\nx = 1\n");
    assert!(lex(&fixed).diagnostics.is_empty());
}

#[test]
fn inline_comment() {
    let lexed = lex("x = 1 // note\n");
    assert_eq!(lexed.comments[0].text, "note");
    assert!(!lexed.comments[0].own_line);
    assert_eq!(toks("x = 1 // note\n"), "x = 1 NL EOF");
}

// ---------- strings ----------

#[test]
fn interpolation() {
    assert_eq!(toks("\"Hello ${name}!\""), r#""Hello ${name}!" NL EOF"#);
    assert_eq!(toks("\"a ${f(\"b\", {k: 1})} c\""), r#""a ${f ( "b" , { k : 1 } )} c" NL EOF"#);
    assert_eq!(toks("\"${a.b}${c}\""), r#""${a . b}${c}" NL EOF"#);
}

#[test]
fn escapes() {
    let lexed = lex(r#""a\n\"b\" \${x} \u{41}""#);
    assert!(lexed.diagnostics.is_empty());
    let TokenKind::Str(s) = &lexed.tokens[0].kind else { panic!() };
    assert_eq!(s.parts, [StrPart::Text("a\n\"b\" ${x} A".into())]);
}

#[test]
fn multiline_string_removes_common_indent() {
    let src = "p = \"\"\"\n  You are an assistant.\n    Answer briefly: ${topic}\n  \"\"\"\n";
    let lexed = lex(src);
    assert!(lexed.diagnostics.is_empty(), "{:#?}", lexed.diagnostics);
    let TokenKind::Str(s) = &lexed.tokens[2].kind else { panic!() };
    assert!(s.multiline);
    assert_eq!(s.parts.len(), 2);
    assert_eq!(s.parts[0], StrPart::Text("You are an assistant.\n  Answer briefly: ".into()));
    assert_eq!(lexed.tokens.iter().map(|t| t.kind.short()).collect::<Vec<_>>().last().unwrap(), "EOF");
}

#[test]
fn single_quotes_work_like_double_quotes() {
    let single = lex(r#"x = 'hi ${name} "you" \'me\''"#);
    let double = lex(r#"x = "hi ${name} \"you\" 'me'""#);
    assert!(single.diagnostics.is_empty() && double.diagnostics.is_empty());
    assert_eq!(single.tokens[2].kind, double.tokens[2].kind);
}

#[test]
fn triple_single_quotes() {
    let lexed = lex("x = '''\n  line \"one\"\n  two\n  '''\n");
    assert!(lexed.diagnostics.is_empty(), "{:#?}", lexed.diagnostics);
    let TokenKind::Str(s) = &lexed.tokens[2].kind else { panic!("not a string") };
    assert!(s.multiline);
}

// ---------- numbers, symbols and words ----------

#[test]
fn numbers_and_ranges() {
    assert_eq!(toks("for i in 0..10\n  x"), "for i in 0 .. 10 NL IN x NL DE EOF");
    assert_eq!(toks("a = 3.14 + 1_000 + 0xff"), "a = 3.14 + 1_000 + 0xff NL EOF");
    assert_eq!(toks("r = (1..10).map(i: int)"), "r = ( 1 .. 10 ) . map ( i : int ) NL EOF");
}

#[test]
fn await_this_and_operators() {
    assert_eq!(toks("users = ~getUsers.get()"), "users = ~ getUsers . get ( ) NL EOF");
    assert_eq!(toks("data = ~res.json()"), "data = ~ res . json ( ) NL EOF");
    assert_eq!(toks("@override\nbuild()"), "@ override NL build ( ) NL EOF");
    assert_eq!(toks("$name = name"), "$ name = name NL EOF");
    assert_eq!(toks("a ?? b?.c ??= d...e"), "a ?? b ?. c ??= d ... e NL EOF");
    assert_eq!(toks("x => y -> z != w"), "x => y -> z != w NL EOF");
}

#[test]
fn dialect_words_are_identifiers() {
    let lexed = lex("extension NumberParsing on String");
    assert!(matches!(&lexed.tokens[0].kind, TokenKind::Ident(s) if s == "extension"));
    assert!(matches!(&lexed.tokens[2].kind, TokenKind::Ident(s) if s == "on"));
    assert!(matches!(lex("if").tokens[0].kind, TokenKind::Keyword(Keyword::If)));
}

#[test]
fn unicode_identifiers() {
    assert_eq!(toks("año = \"está\""), r#"año = "está" NL EOF"#);
}

// ---------- errors ----------

#[test]
fn error_tab_indentation() {
    assert_eq!(codes("a\n\tb\n"), ["N0002"]);
}

#[test]
fn error_bad_dedent() {
    assert_eq!(codes("a\n    b\n  c\n"), ["N0004"]);
}

#[test]
fn error_odd_indent() {
    assert_eq!(codes("a\n   b\n"), ["N0005"]);
}

#[test]
fn error_unexpected_char() {
    assert_eq!(codes("a # b"), ["N0006"]);
}

#[test]
fn error_unterminated_string() {
    assert_eq!(codes("x = \"abc\ny = 1"), ["N0007"]);
}

#[test]
fn error_brackets() {
    assert_eq!(codes("foo(1, 2"), ["N0008"]);
    assert_eq!(codes("foo(1]"), ["N0008"]);
    assert_eq!(codes("x)"), ["N0008"]);
}

#[test]
fn error_foreign_syntax_recovers() {
    let lexed = lex("if a && !b || c === d;");
    let codes: Vec<_> = lexed.diagnostics.iter().map(|d| d.code.id).collect();
    assert_eq!(codes, ["N0009"; 5]);
    let short: Vec<String> = lexed.tokens.iter().map(|t| t.kind.short()).collect();
    assert_eq!(short.join(" "), "if a and not b or c == d NL EOF");
}

#[test]
fn error_interpolation() {
    assert_eq!(codes("\"${}\""), ["N0010"]);
    assert_eq!(codes("\"${a\""), ["N0010", "N0007"]);
}

#[test]
fn error_escape_and_number() {
    assert_eq!(codes(r#""\q""#), ["N0011"]);
    assert_eq!(codes("x = 10px"), ["N0012"]);
    assert_eq!(codes("x = 0x"), ["N0012"]);
}

// ---------- kept in sync with data files ----------

#[test]
fn keywords_match_languages_nuclear_toml() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../languages/nuclear.toml");
    let doc: toml::Table = std::fs::read_to_string(path).unwrap().parse().unwrap();
    let mut in_toml: Vec<&str> = doc["keywords"].as_table().unwrap().keys().map(String::as_str).collect();
    let mut in_rust: Vec<&str> = Keyword::ALL.iter().map(|k| k.as_str()).collect();
    in_toml.sort();
    in_rust.sort();
    assert_eq!(in_rust, in_toml, "token.rs and languages/nuclear.toml must list the same words");
}

#[test]
fn examples_lex_without_errors() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "nuc") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples")), &mut files);
    assert!(!files.is_empty());
    for f in files {
        let lexed = lex(&std::fs::read_to_string(&f).unwrap());
        assert!(lexed.diagnostics.is_empty(), "{}: {:#?}", f.display(), lexed.diagnostics);
        assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Eof, "{}", f.display());
    }
}
