//! Parser: tokens → AST (spec/nuclear-base.md, spec/grammar.ebnf).
//!
//! Recursive descent over the token list produced by the lexer. INDENT/DEDENT play the role of braces.
//! The parser knows nothing about dialects: words before a declaration's name (`stateless`, `static`)
//! and after it (`on String`) are kept as data and validated later by the active dialect.
//!
//! Like the lexer, it never stops at the first error: it reports, skips to the next line and continues.

use crate::ast::*;
use crate::lexer::lex;
use crate::token::*;
use nuclear_diag::{Code, Diagnostic, Span, codes};

pub struct Parsed {
    pub file: File,
    pub diagnostics: Vec<Diagnostic>,
}

/// Lexes and parses a `.nuc` source.
pub fn parse(src: &str) -> Parsed {
    let lexed = lex(src);
    let mut p = Parser::new(src, lexed.tokens);
    let items = p.file();
    let mut diagnostics = lexed.diagnostics;
    diagnostics.append(&mut p.diags);
    // a habit of another language explains its line better than the errors it causes there
    let habits = crate::habits::scan(src);
    if !habits.is_empty() {
        let line = |at: usize| src[..at.min(src.len())].matches('\n').count();
        let lines: std::collections::HashSet<usize> = habits.iter().map(|d| line(d.span.start)).collect();
        diagnostics.retain(|d| !lines.contains(&line(d.span.start)));
        diagnostics.extend(habits);
    }
    diagnostics.sort_by_key(|d| d.span.start);
    Parsed { file: File { items, comments: lexed.comments }, diagnostics }
}

/// A type written in Nuclear notation (`Result<NotFound, T>`, `User[]?`), as the type definitions store them.
pub fn parse_type(text: &str) -> Option<Type> {
    let lexed = lex(text);
    if !lexed.diagnostics.is_empty() {
        return None;
    }
    let mut p = Parser::new(text, lexed.tokens);
    let t = p.ty()?;
    p.diags.is_empty().then_some(t)
}

/// Where a declaration's block lives: decides what an indented line means.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    File,
    Module,
}

struct Parser<'s> {
    src: &'s str,
    toks: Vec<Token>,
    pos: usize,
    diags: Vec<Diagnostic>,
    /// Parsing a cascade section: a `..` line belongs to the outer cascade, not to this section.
    in_cascade: bool,
}

impl<'s> Parser<'s> {
    fn new(src: &'s str, toks: Vec<Token>) -> Self {
        Parser { src, toks, pos: 0, diags: Vec::new(), in_cascade: false }
    }

    // ---------- token helpers ----------

    fn tok(&self) -> &Token {
        let i = self.pos.min(self.toks.len() - 1);
        &self.toks[i]
    }

    fn peek(&self) -> &TokenKind {
        &self.tok().kind
    }

    fn peek_at(&self, n: usize) -> &TokenKind {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].kind
    }

    fn tok_at(&self, n: usize) -> &Token {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i]
    }

    fn span(&self) -> Span {
        self.tok().span
    }

    /// End of the previous token (for spans that end at the last consumed token).
    fn prev_end(&self) -> usize {
        if self.pos == 0 { 0 } else { self.toks[self.pos - 1].span.end }
    }

    fn prev_is_dedent(&self) -> bool {
        self.pos > 0 && self.toks[self.pos - 1].kind == TokenKind::Dedent
    }

    fn bump(&mut self) -> Token {
        let t = self.tok().clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn at_punct(&self, p: Punct) -> bool {
        matches!(self.peek(), TokenKind::Punct(q) if *q == p)
    }

    /// `>>`, `>>>`, `>>=` and `>>>=` are `>` tokens written together (the lexer keeps `>` apart so that
    /// `List<List<int>>` closes two generics). Returns the operator and how many tokens it takes.
    fn glued_gt(&self) -> Option<(&'static str, usize)> {
        if !self.at_punct(Punct::Gt) {
            return None;
        }
        let glued = |n: usize| self.tok_at(n - 1).span.end == self.tok_at(n).span.start;
        let mut text = String::from(">");
        let mut n = 1;
        while n < 4 && glued(n) {
            match self.peek_at(n) {
                TokenKind::Punct(Punct::Gt) => text.push('>'),
                TokenKind::Punct(Punct::GtEq) => {
                    text.push_str(">=");
                    break;
                }
                _ => break,
            }
            n += 1;
        }
        match text.as_str() {
            ">>" => Some((">>", 2)),
            ">>>" => Some((">>>", 3)),
            ">>=" => Some((">>=", 2)),
            ">>>=" => Some((">>>=", 3)),
            _ => None,
        }
    }

    fn at_punct_at(&self, n: usize, p: Punct) -> bool {
        matches!(self.peek_at(n), TokenKind::Punct(q) if *q == p)
    }

    fn eat_punct(&mut self, p: Punct) -> bool {
        if self.at_punct(p) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn at_kw(&self, k: Keyword) -> bool {
        matches!(self.peek(), TokenKind::Keyword(q) if *q == k)
    }

    fn eat_kw(&mut self, k: Keyword) -> bool {
        if self.at_kw(k) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn at_ident(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_))
    }

    fn at_newline(&self) -> bool {
        matches!(self.peek(), TokenKind::Newline)
    }

    fn at_line_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof)
    }

    /// NEWLINE followed by INDENT: the current line opens a block.
    fn block_ahead(&self) -> bool {
        self.at_newline() && matches!(self.peek_at(1), TokenKind::Indent)
    }

    fn skip_newlines(&mut self) {
        while self.at_newline() {
            self.bump();
        }
    }

    fn report(&mut self, code: Code, span: Span, msg: impl Into<String>) {
        self.diags.push(Diagnostic::error(code, span, msg));
    }

    fn describe(kind: &TokenKind) -> String {
        match kind {
            TokenKind::Ident(s) => format!("'{s}'"),
            TokenKind::Keyword(k) => format!("'{k}'"),
            TokenKind::Number(n) => format!("number {}", n.text),
            TokenKind::Str(_) => "a string".into(),
            TokenKind::Punct(p) => format!("'{p}'"),
            TokenKind::Newline => "end of line".into(),
            TokenKind::Indent => "an indented block".into(),
            TokenKind::Dedent => "end of block".into(),
            TokenKind::Eof => "end of file".into(),
        }
    }

    fn unexpected(&mut self, expected: &str) {
        let found = Self::describe(self.peek());
        let span = self.span();
        self.report(codes::UNEXPECTED_TOKEN, span, format!("expected {expected}, found {found}"));
    }

    fn expect_punct(&mut self, p: Punct) -> bool {
        if self.eat_punct(p) {
            return true;
        }
        self.unexpected(&format!("'{p}'"));
        false
    }

    fn ident(&mut self) -> Option<Ident> {
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                let span = self.bump().span;
                Some(Ident { name, span })
            }
            TokenKind::Keyword(k) => {
                let span = self.span();
                self.report(codes::RESERVED_NAME, span, format!("'{k}' is a reserved word and cannot be a name"));
                self.bump();
                Some(Ident { name: k.to_string(), span })
            }
            _ => {
                self.unexpected("a name");
                None
            }
        }
    }

    /// Member names after `.` may be keywords (`res.type`, `list.in`).
    fn member_name(&mut self) -> Option<Ident> {
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                let span = self.bump().span;
                Some(Ident { name, span })
            }
            TokenKind::Keyword(k) => {
                let span = self.bump().span;
                Some(Ident { name: k.to_string(), span })
            }
            _ => {
                self.unexpected("a member name");
                None
            }
        }
    }

    /// Statement end: NEWLINE (or the end of the block/file).
    fn end_line(&mut self) {
        if self.at_newline() {
            self.bump();
        } else if !matches!(self.peek(), TokenKind::Dedent | TokenKind::Eof) {
            self.unexpected("end of line");
            self.sync();
        }
    }

    /// Error recovery: skip the rest of the line, including any block it opens.
    fn sync(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Eof | TokenKind::Dedent => return,
                TokenKind::Newline => {
                    self.bump();
                    if matches!(self.peek(), TokenKind::Indent) {
                        self.skip_block();
                    }
                    return;
                }
                TokenKind::Indent => self.skip_block(),
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn skip_block(&mut self) {
        let mut depth = 0usize;
        loop {
            match self.peek() {
                TokenKind::Indent => depth += 1,
                TokenKind::Dedent => {
                    depth -= 1;
                    if depth == 0 {
                        self.bump();
                        return;
                    }
                }
                TokenKind::Eof => return,
                _ => {}
            }
            self.bump();
        }
    }

    /// Index of the token that closes the bracket at `open` (relative offset from `pos`).
    fn matching_close(&self, open: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut i = open;
        loop {
            match self.peek_at(i) {
                TokenKind::Punct(p) if p.is_open() => depth += 1,
                TokenKind::Punct(p) if p.is_close() => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                TokenKind::Eof => return None,
                _ => {}
            }
            i += 1;
        }
    }

    // ---------- file ----------

    fn file(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::Indent | TokenKind::Dedent => {
                    let span = self.span();
                    self.report(codes::UNEXPECTED_TOKEN, span, "unexpected indentation");
                    self.bump();
                    continue;
                }
                _ => {}
            }
            let before = self.pos;
            match self.item() {
                Some(item) => items.push(item),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.sync();
                }
            }
        }
        items
    }

    fn item(&mut self) -> Option<Item> {
        if self.at_kw(Keyword::Use) {
            return self.use_decl().map(Item::Use);
        }
        if self.at_punct(Punct::At) {
            let decorators = self.decorators();
            return self.decl(Ctx::File, decorators).map(Item::Decl);
        }
        if self.file_decl_ahead() {
            return self.decl(Ctx::File, Vec::new()).map(Item::Decl);
        }
        self.stmt().map(Item::Stmt)
    }

    /// At the top level, does this line start a declaration (and not a statement)?
    fn file_decl_ahead(&self) -> bool {
        let start = usize::from(self.at_punct(Punct::Tilde));
        let TokenKind::Ident(first) = self.peek_at(start) else {
            return false;
        };
        let next = self.peek_at(start + 1);
        if matches!(next, TokenKind::Ident(_)) {
            return true; // words before the name: `stateless MyApp`
        }
        if is_pascal(first) {
            // `Logger.info(...)` and `MAX = 1` are statements
            return !matches!(next, TokenKind::Punct(Punct::Dot | Punct::Eq | Punct::QuestionDot | Punct::LBracket));
        }
        match next {
            // `greet` + block: function without parameters
            TokenKind::Newline => matches!(self.peek_at(start + 2), TokenKind::Indent),
            TokenKind::Punct(Punct::LParen) => {
                let Some(close) = self.matching_close(start + 1) else {
                    return false;
                };
                match self.peek_at(close + 1) {
                    TokenKind::Newline => matches!(self.peek_at(close + 2), TokenKind::Indent),
                    TokenKind::Punct(Punct::Eq) => true,
                    TokenKind::Ident(_) | TokenKind::Punct(Punct::Lt) => true,
                    _ => false,
                }
            }
            // `first<T>(...)`, `first<T: Comparable>(...)`
            TokenKind::Punct(Punct::Lt) => {
                matches!(self.peek_at(start + 2), TokenKind::Ident(_))
                    && matches!(self.peek_at(start + 3), TokenKind::Punct(Punct::Gt | Punct::Comma | Punct::Colon))
            }
            _ => false,
        }
    }

    // ---------- use / type ----------

    fn use_decl(&mut self) -> Option<Use> {
        let start = self.bump().span.start; // `use`
        let mut alias = None;
        let mut star = false;
        if self.eat_punct(Punct::Star) {
            star = true;
            if !self.eat_kw(Keyword::As) {
                self.unexpected("'as' after 'use *'");
                return None;
            }
            alias = Some(self.ident()?);
            if !self.eat_kw(Keyword::From) {
                self.unexpected("'from'");
                return None;
            }
        }
        let path = if let TokenKind::Str(s) = self.peek().clone() {
            self.bump();
            UsePath::File(str_text(&s))
        } else {
            UsePath::Module(self.module_path()?)
        };
        let mut names = Vec::new();
        if self.eat_punct(Punct::LBrace) {
            while !self.at_punct(Punct::RBrace) && !self.at_eof() {
                names.push(self.ident()?);
                if !self.eat_punct(Punct::Comma) {
                    break;
                }
            }
            self.expect_punct(Punct::RBrace);
        }
        let mut clauses = Vec::new();
        while !self.at_line_end() {
            if self.eat_kw(Keyword::As) {
                if star || alias.is_some() {
                    self.unexpected("end of line");
                    return None;
                }
                alias = Some(self.ident()?);
                continue;
            }
            if !self.at_ident() {
                self.unexpected("'as', a clause word or end of line");
                return None;
            }
            let word = self.ident()?;
            let mut clause_names = Vec::new();
            if self.at_ident() {
                loop {
                    clause_names.push(self.ident()?);
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
            }
            clauses.push(UseClause { word, names: clause_names });
        }
        let span = Span::new(start, self.prev_end());
        self.end_line();
        Some(Use { span, path, names, alias, clauses })
    }

    /// `fs/promises`, `@nestjs/common`, `lib1/lib1`: tokens written together, with no spaces.
    fn module_path(&mut self) -> Option<String> {
        let mut text = String::new();
        let mut last_end = None;
        loop {
            let t = self.tok().clone();
            if let Some(end) = last_end
                && t.span.start != end
            {
                break;
            }
            let piece = match &t.kind {
                TokenKind::Ident(s) => s.clone(),
                TokenKind::Keyword(k) => k.to_string(),
                TokenKind::Number(n) => n.text.clone(),
                TokenKind::Punct(p @ (Punct::Slash | Punct::Minus | Punct::Dot | Punct::At)) => p.to_string(),
                _ => break,
            };
            if last_end.is_none() && !matches!(t.kind, TokenKind::Ident(_) | TokenKind::Punct(Punct::At)) {
                break;
            }
            text.push_str(&piece);
            last_end = Some(t.span.end);
            self.bump();
        }
        if text.is_empty() {
            self.unexpected("a module path");
            return None;
        }
        Some(text)
    }

    // ---------- declarations ----------

    fn decorators(&mut self) -> Vec<Decorator> {
        let mut out = Vec::new();
        while self.at_punct(Punct::At) {
            if let Some(d) = self.decorator() {
                out.push(d);
            }
            self.skip_newlines();
        }
        out
    }

    fn decorator(&mut self) -> Option<Decorator> {
        let start = self.bump().span.start; // `@`
        let mut name = self.member_name()?.name;
        while self.at_punct(Punct::Dot) {
            self.bump();
            name.push('.');
            name.push_str(&self.member_name()?.name);
        }
        let args = if self.at_punct(Punct::LParen) { Some(self.args()?) } else { None };
        Some(Decorator { span: Span::new(start, self.prev_end()), name, args })
    }

    /// Words before the name and the name itself.
    /// The name is the first PascalCase word (`extension NumberParsing on String`) or, when every word
    /// is camelCase, the last one (`static count`). `~` marks an async function: `~load()`.
    fn header_words(&mut self) -> Option<(Vec<Ident>, Ident, bool)> {
        let mut run = Vec::new();
        let mut is_async = false;
        loop {
            if self.at_punct(Punct::Tilde) && matches!(self.peek_at(1), TokenKind::Ident(_)) {
                self.bump();
                is_async = true;
                let name = self.ident()?;
                return Some((run, name, is_async));
            }
            if !self.at_ident() {
                break;
            }
            let id = self.ident()?;
            let pascal = id.is_pascal();
            run.push(id);
            if pascal || !self.at_ident() {
                break;
            }
        }
        let Some(name) = run.pop() else {
            self.unexpected("a declaration name");
            return None;
        };
        Some((run, name, is_async))
    }

    fn decl(&mut self, ctx: Ctx, decorators: Vec<Decorator>) -> Option<Decl> {
        let start = decorators.first().map_or(self.span().start, |d| d.span.start);
        let (words, name, is_async) = self.header_words()?;
        let (generics, generic_bounds) =
            if self.at_punct(Punct::Lt) { self.generics()? } else { (Vec::new(), Vec::new()) };
        let params = if self.at_punct(Punct::LParen) { Some(self.params()?) } else { None };
        let mut bases = Vec::new();
        if self.eat_punct(Punct::Colon) {
            bases = self.type_list()?;
        }
        let mut implements = Vec::new();
        if self.eat_punct(Punct::Gt) {
            implements = self.type_list()?;
        }
        let mut ret = None;
        let mut clauses = Vec::new();
        if self.at_ident() && matches!(self.peek_at(1), TokenKind::Ident(_)) {
            // clauses: `on String`, `with Flyer, Other`
            while self.at_ident() && matches!(self.peek_at(1), TokenKind::Ident(_)) {
                let word = self.ident()?;
                let types = self.type_list()?;
                clauses.push(Clause { word, types });
            }
        } else if !self.at_line_end() && !self.at_punct(Punct::Eq) {
            ret = Some(self.ty()?);
        }
        let module_like = name.is_pascal();
        let body = if self.eat_punct(Punct::Eq) {
            let e = self.expr()?;
            if ret.as_ref().is_some_and(is_void) {
                let span = Span::new(start, e.span.end);
                self.report(
                    codes::VOID_ONE_LINER,
                    span,
                    format!("'{}' returns void, so it needs an indented block instead of '='", name.name),
                );
            }
            self.end_line();
            Body::Expr(e)
        } else if self.block_ahead() {
            self.bump();
            if module_like { Body::Members(self.members()) } else { Body::Block(self.block()) }
        } else {
            self.end_line();
            Body::None
        };
        if ctx == Ctx::Module && module_like {
            let span = name.span;
            self.report(codes::NESTED_MODULE, span, format!("'{}' cannot be declared inside a module", name.name));
        }
        let end = self.prev_end().max(start);
        Some(Decl {
            span: Span::new(start, end),
            decorators,
            words,
            is_async,
            name,
            generics,
            generic_bounds,
            params,
            bases,
            implements,
            ret,
            clauses,
            body,
        })
    }

    /// `<T, U: Comparable>`: names and their bounds (`:` = extends, as in inheritance).
    fn generics(&mut self) -> Option<(Vec<Ident>, Vec<Option<Type>>)> {
        self.bump(); // `<`
        let (mut names, mut bounds) = (Vec::new(), Vec::new());
        loop {
            names.push(self.ident()?);
            bounds.push(if self.eat_punct(Punct::Colon) { Some(self.ty()?) } else { None });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::Gt);
        Some((names, bounds))
    }

    fn params(&mut self) -> Option<Params> {
        let start = self.bump().span.start; // `(`
        let mut list = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            if self.eat_punct(Punct::LBrace) {
                while !self.at_punct(Punct::RBrace) && !self.at_eof() {
                    list.push(self.param(true)?);
                    if !self.eat_punct(Punct::Comma) && !self.tok().nl_before {
                        break;
                    }
                }
                self.expect_punct(Punct::RBrace);
            } else {
                list.push(self.param(false)?);
            }
            if !self.eat_punct(Punct::Comma) && !self.tok().nl_before {
                break;
            }
        }
        if !self.expect_punct(Punct::RParen) {
            return None;
        }
        Some(Params { span: Span::new(start, self.prev_end()), list })
    }

    fn param(&mut self, named: bool) -> Option<Param> {
        let start = self.span().start;
        let mut decorators = Vec::new();
        while self.at_punct(Punct::At) {
            decorators.push(self.decorator()?);
        }
        let name = self.ident()?;
        let optional = self.eat_punct(Punct::Question);
        let ty = if self.eat_punct(Punct::Colon) { Some(self.ty()?) } else { None };
        let default = if self.eat_punct(Punct::Eq) { Some(self.expr()?) } else { None };
        Some(Param { span: Span::new(start, self.prev_end()), decorators, name, optional, ty, default, named })
    }

    // ---------- module members ----------

    fn members(&mut self) -> Vec<Member> {
        let mut out = Vec::new();
        if !matches!(self.peek(), TokenKind::Indent) {
            return out;
        }
        self.bump();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => {
                    self.bump();
                    break;
                }
                TokenKind::Eof => break,
                _ => {}
            }
            let before = self.pos;
            match self.member() {
                Some(m) => out.push(m),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.sync();
                }
            }
        }
        out
    }

    fn member(&mut self) -> Option<Member> {
        let decorators = self.decorators();
        let start = decorators.first().map_or(self.span().start, |d| d.span.start);
        if self.at_punct(Punct::LParen) {
            let params = self.params()?;
            let body = if self.block_ahead() {
                self.bump();
                Some(self.block())
            } else {
                self.end_line();
                None
            };
            return Some(Member::Ctor(Ctor { span: Span::new(start, self.prev_end()), params, body }));
        }
        if !(self.at_ident() || self.at_punct(Punct::Tilde)) {
            if let TokenKind::Keyword(_) = self.peek() {
                self.ident(); // reports N0001
            } else {
                self.unexpected("a field, a method or the constructor");
            }
            return None;
        }
        // look past the words to see what follows the name
        let mut i = 0;
        while matches!(self.peek_at(i), TokenKind::Ident(_)) && matches!(self.peek_at(i + 1), TokenKind::Ident(_)) {
            if let TokenKind::Ident(w) = self.peek_at(i)
                && is_pascal(w)
            {
                break;
            }
            i += 1;
        }
        let after_name = self.peek_at(i + 1).clone();
        let is_field = matches!(after_name, TokenKind::Punct(Punct::Colon))
            || (matches!(after_name, TokenKind::Punct(Punct::Question))
                && matches!(self.peek_at(i + 2), TokenKind::Punct(Punct::Colon)));
        if is_field {
            let mut words = Vec::new();
            for _ in 0..i {
                words.push(self.ident()?);
            }
            let name = self.ident()?;
            let optional = self.eat_punct(Punct::Question);
            self.bump(); // `:`
            let ty = Some(self.ty()?);
            let default = if self.eat_punct(Punct::Eq) { Some(self.expr()?) } else { None };
            let span = Span::new(start, self.prev_end());
            self.end_line();
            return Some(Member::Field(Field { span, decorators, words, name, optional, ty, default }));
        }
        if matches!(after_name, TokenKind::Punct(Punct::Eq | Punct::PlusEq | Punct::MinusEq)) {
            let span = self.tok_at(i).span;
            self.report(
                codes::STATEMENT_IN_MODULE,
                span,
                "a module body cannot hold statements: declare a field (name: Type = value) or assign it in the constructor block",
            );
            return None;
        }
        // enum items: `user` (no block) or `admin("administrator")` (literal values)
        if i == 0 && decorators.is_empty() {
            let bare = matches!(after_name, TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof)
                && !matches!(self.peek_at(2), TokenKind::Indent);
            let literal_args = matches!(after_name, TokenKind::Punct(Punct::LParen))
                && matches!(
                    self.peek_at(2),
                    TokenKind::Str(_)
                        | TokenKind::Number(_)
                        | TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::Null)
                        | TokenKind::Punct(Punct::Minus)
                );
            if bare || literal_args {
                let name = self.ident()?;
                let args = if self.at_punct(Punct::LParen) {
                    self.args()?.into_iter().map(|a| a.value).collect()
                } else {
                    Vec::new()
                };
                let span = Span::new(start, self.prev_end());
                self.end_line();
                return Some(Member::EnumItem(EnumItem { span, name, args }));
            }
        }
        self.decl(Ctx::Module, decorators).map(Member::Method)
    }

    // ---------- statements ----------

    /// `NEWLINE INDENT stmts DEDENT`, with the NEWLINE already consumed.
    fn block(&mut self) -> Block {
        let start = self.span().start;
        let mut stmts = Vec::new();
        if !matches!(self.peek(), TokenKind::Indent) {
            self.unexpected("an indented block");
            return Block { span: Span::new(start, start), stmts };
        }
        self.bump();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => {
                    self.bump();
                    break;
                }
                TokenKind::Eof => break,
                _ => {}
            }
            let before = self.pos;
            match self.stmt() {
                Some(s) => stmts.push(s),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.sync();
                }
            }
        }
        Block { span: Span::new(start, self.prev_end()), stmts }
    }

    /// Header line + block: `NEWLINE INDENT ...`.
    fn line_block(&mut self) -> Option<Block> {
        if !self.block_ahead() {
            self.unexpected("an indented block on the next line");
            return None;
        }
        self.bump();
        Some(self.block())
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let start = self.span().start;
        if let TokenKind::Keyword(k) = self.peek()
            && matches!(self.peek_at(1), TokenKind::Punct(Punct::Eq | Punct::Colon))
        {
            let (k, span) = (*k, self.span());
            self.report(codes::RESERVED_NAME, span, format!("'{k}' is a reserved word and cannot be a name"));
            return None;
        }
        match self.peek() {
            TokenKind::Keyword(Keyword::If) => return self.if_chain().map(Stmt::If),
            TokenKind::Keyword(Keyword::For) => return self.for_stmt(),
            TokenKind::Keyword(Keyword::While) => {
                self.bump();
                let cond = self.expr()?;
                let body = self.line_block()?;
                return Some(Stmt::While { span: Span::new(start, self.prev_end()), cond, body });
            }
            TokenKind::Keyword(Keyword::Match) => return self.match_expr().map(Stmt::Match),
            TokenKind::Keyword(Keyword::Try) => return self.try_stmt(),
            TokenKind::Keyword(Keyword::Return) => {
                self.bump();
                let value = if self.at_line_end() { None } else { Some(self.expr_with_block()?) };
                let span = Span::new(start, self.prev_end());
                if !self.prev_is_dedent() {
                    self.end_line();
                }
                return Some(Stmt::Return { span, value });
            }
            TokenKind::Keyword(Keyword::Throw) => {
                self.bump();
                let value = self.expr()?;
                let span = Span::new(start, self.prev_end());
                self.end_line();
                return Some(Stmt::Throw { span, value });
            }
            TokenKind::Keyword(Keyword::Yield) => {
                self.bump();
                let all = self.eat_punct(Punct::Ellipsis);
                let value = self.expr_with_block()?;
                let span = Span::new(start, self.prev_end());
                if !self.prev_is_dedent() {
                    self.end_line();
                }
                return Some(Stmt::Yield { span, value, all });
            }
            TokenKind::Keyword(Keyword::Break) => {
                let span = self.bump().span;
                self.end_line();
                return Some(Stmt::Break(span));
            }
            TokenKind::Keyword(Keyword::Continue) => {
                let span = self.bump().span;
                self.end_line();
                return Some(Stmt::Continue(span));
            }
            TokenKind::Punct(Punct::At) => {
                let span = self.span();
                self.report(codes::UNEXPECTED_TOKEN, span, "decorators go only before declarations; for await use '~'");
                return None;
            }
            _ => {}
        }
        self.simple_stmt()
    }

    /// Expression, assignment or typed declaration, possibly followed by a lambda block.
    fn simple_stmt(&mut self) -> Option<Stmt> {
        let start = self.span().start;
        let target = self.expr()?;

        // `name(params) Type` + block inside a body: a nested function declaration
        if let ExprKind::Call { callee, .. } = &target.kind
            && matches!(callee.kind, ExprKind::Name(_))
            && self.at_ident()
        {
            let span = target.span;
            self.report(
                codes::NESTED_FUNCTION,
                span,
                "functions cannot be declared inside a body; assign an anonymous function: name = (params) Type",
            );
            return None;
        }

        let mut ty = None;
        if self.at_punct(Punct::Colon) && matches!(target.kind, ExprKind::Name(_)) {
            self.bump();
            ty = Some(self.ty()?);
        }
        let op = match self.peek() {
            TokenKind::Punct(Punct::Eq) => Some(AssignOp::Set),
            TokenKind::Punct(Punct::PlusEq) => Some(AssignOp::Add),
            TokenKind::Punct(Punct::MinusEq) => Some(AssignOp::Sub),
            TokenKind::Punct(Punct::StarEq) => Some(AssignOp::Mul),
            TokenKind::Punct(Punct::SlashEq) => Some(AssignOp::Div),
            TokenKind::Punct(Punct::PercentEq) => Some(AssignOp::Rem),
            TokenKind::Punct(Punct::QuestionQuestionEq) => Some(AssignOp::Coalesce),
            TokenKind::Punct(Punct::AmpEq) => Some(AssignOp::BitAnd),
            TokenKind::Punct(Punct::PipeEq) => Some(AssignOp::BitOr),
            TokenKind::Punct(Punct::CaretEq) => Some(AssignOp::BitXor),
            TokenKind::Punct(Punct::ShlEq) => Some(AssignOp::Shl),
            _ => match self.glued_gt() {
                Some((">>=", _)) => Some(AssignOp::Shr),
                Some((">>>=", _)) => Some(AssignOp::UShr),
                _ => None,
            },
        };
        let Some(op) = op else {
            if ty.is_some() {
                self.unexpected("'=' after the type");
                return None;
            }
            let expr = self.attach_block(target)?;
            let expr = self.chain_after_block(expr)?;
            if !self.prev_is_dedent() {
                self.end_line();
            }
            return Some(Stmt::Expr(expr));
        };
        let glued = match op {
            AssignOp::Shr => 2,
            AssignOp::UShr => 3,
            _ => 1,
        };
        for _ in 0..glued {
            self.bump();
        }
        match &target.kind {
            ExprKind::Name(_) | ExprKind::SelfField(_) | ExprKind::Member { .. } | ExprKind::Index { .. } => {}
            // destructuring: `[a, ...rest] = xs`, `{ name, "first-name": first } = map`
            ExprKind::List(_) | ExprKind::Object(_) if op == AssignOp::Set => self.check_pattern(&target),
            _ => {
                let span = target.span;
                self.report(codes::BAD_ASSIGN_TARGET, span, "this expression cannot be assigned");
            }
        }
        let value = self.expr_with_block()?;
        let span = Span::new(start, self.prev_end());
        if !self.prev_is_dedent() {
            self.end_line();
        }
        Some(Stmt::Assign(Assign { span, target, ty, op, value }))
    }

    /// A destructuring pattern holds names, `_`, `...rest` (last item of a list) and nested patterns.
    fn check_pattern(&mut self, p: &Expr) {
        match &p.kind {
            ExprKind::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    match &item.kind {
                        ExprKind::Spread(inner) if i + 1 == items.len() && matches!(inner.kind, ExprKind::Name(_)) => {}
                        ExprKind::Spread(_) => {
                            self.report(
                                codes::BAD_ASSIGN_TARGET,
                                item.span,
                                "'...rest' goes last in a list pattern and takes a name",
                            );
                        }
                        _ => self.check_pattern(item),
                    }
                }
            }
            ExprKind::Object(entries) => {
                for en in entries {
                    match en {
                        ObjEntry::Shorthand(_) => {}
                        ObjEntry::Pair { value, .. } => self.check_pattern(value),
                        ObjEntry::Spread(e) => {
                            self.report(codes::BAD_ASSIGN_TARGET, e.span, "a map pattern takes names, not '...rest'");
                        }
                    }
                }
            }
            ExprKind::Name(_) => {}
            _ => {
                self.report(codes::BAD_ASSIGN_TARGET, p.span, "a pattern holds names, '_', '...rest' or other patterns")
            }
        }
    }

    fn expr_with_block(&mut self) -> Option<Expr> {
        let e = self.expr()?;
        let e = self.attach_block(e)?;
        self.chain_after_block(e)
    }

    /// `list.map(x)` + block, then `.toList()` on its own line: the chain goes on after the block.
    fn chain_after_block(&mut self, mut e: Expr) -> Option<Expr> {
        while self.prev_is_dedent() && (self.at_punct(Punct::Dot) || self.at_punct(Punct::QuestionDot)) {
            e = self.postfix_ops(e)?;
            e = self.attach_block(e)?;
        }
        Some(e)
    }

    /// `call(params)` followed by an indented block: the parentheses hold the lambda's parameters and
    /// the block is its body. The lambda becomes the call's only argument.
    fn attach_block(&mut self, expr: Expr) -> Option<Expr> {
        if !self.block_ahead() {
            self.check_typed_args(&expr);
            return Some(expr);
        }
        let ExprKind::Call { callee, args, type_args } = expr.kind else {
            let span = self.tok_at(1).span;
            self.report(codes::UNEXPECTED_TOKEN, span, "unexpected indented block");
            self.bump();
            self.skip_block();
            return Some(expr);
        };
        let mut params = Vec::new();
        for a in args {
            match (&a.value.kind, a.name.is_none() && !a.spread) {
                (ExprKind::Name(n), true) => params.push(Param {
                    span: a.span,
                    decorators: Vec::new(),
                    name: Ident { name: n.clone(), span: a.value.span },
                    optional: false,
                    ty: a.ty,
                    default: None,
                    named: false,
                }),
                _ => {
                    self.report(
                        codes::UNEXPECTED_TOKEN,
                        a.span,
                        "a call followed by a block takes lambda parameters (name or name: Type) in its parentheses",
                    );
                }
            }
        }
        self.bump(); // NEWLINE
        let body = self.block();
        let lambda_span = Span::new(expr.span.start, body.span.end);
        let lambda = Expr {
            span: lambda_span,
            kind: ExprKind::Lambda(Box::new(Lambda {
                span: lambda_span,
                params,
                ret: None,
                body: LambdaBody::Block(body),
            })),
        };
        let arg = Arg { span: lambda_span, name: None, value: lambda, spread: false, ty: None };
        Some(Expr { span: lambda_span, kind: ExprKind::Call { callee, args: vec![arg], type_args } })
    }

    /// `x: Type` in call arguments without a block after the call.
    fn check_typed_args(&mut self, expr: &Expr) {
        if let ExprKind::Call { args, .. } = &expr.kind {
            for a in args {
                if a.ty.is_some() {
                    let text = &self.src[a.span.start..a.span.end];
                    let named = text.replacen(':', " =", 1);
                    self.diags.push(
                        Diagnostic::error(
                            codes::LAMBDA_NEEDS_BLOCK,
                            a.span,
                            format!(
                                "'{text}' declares a lambda parameter, so the call needs an indented block; for a named argument write '{named}'"
                            ),
                        )
                        .with_fix(a.span, named),
                    );
                }
            }
        }
    }

    fn if_chain(&mut self) -> Option<If> {
        let start = self.bump().span.start; // `if`
        let mut branches = Vec::new();
        let cond = self.expr()?;
        let body = self.line_block()?;
        branches.push((cond, body));
        let mut otherwise = None;
        while self.at_kw(Keyword::Else) {
            self.bump();
            if self.eat_kw(Keyword::If) {
                let cond = self.expr()?;
                let body = self.line_block()?;
                branches.push((cond, body));
            } else {
                otherwise = Some(self.line_block()?);
                break;
            }
        }
        Some(If { span: Span::new(start, self.prev_end()), branches, otherwise })
    }

    fn for_stmt(&mut self) -> Option<Stmt> {
        let start = self.bump().span.start; // `for`
        let mut vars = vec![self.ident()?];
        if self.eat_punct(Punct::Comma) {
            vars.push(self.ident()?);
        }
        if !self.eat_kw(Keyword::In) {
            self.unexpected("'in'");
            return None;
        }
        let iter = self.expr()?;
        let body = self.line_block()?;
        Some(Stmt::For(For { span: Span::new(start, self.prev_end()), vars, iter, body }))
    }

    fn match_expr(&mut self) -> Option<Match> {
        let start = self.bump().span.start; // `match`
        let subject = self.expr()?;
        if !self.block_ahead() {
            self.unexpected("the match arms in an indented block");
            return None;
        }
        self.bump();
        self.bump(); // INDENT
        let mut arms = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => {
                    self.bump();
                    break;
                }
                TokenKind::Eof => break,
                _ => {}
            }
            let arm_start = self.span().start;
            let mut patterns = Vec::new();
            loop {
                patterns.push(self.pattern()?);
                if !self.eat_punct(Punct::Pipe) {
                    break;
                }
            }
            if !self.expect_punct(Punct::ThinArrow) {
                self.sync();
                continue;
            }
            let body = if self.block_ahead() {
                self.bump();
                ArmBody::Block(self.block())
            } else {
                let e = self.expr()?;
                self.end_line();
                ArmBody::Expr(e)
            };
            arms.push(MatchArm { span: Span::new(arm_start, self.prev_end()), patterns, body });
        }
        Some(Match { span: Span::new(start, self.prev_end()), subject, arms })
    }

    fn pattern(&mut self) -> Option<Pattern> {
        if let TokenKind::Ident(n) = self.peek()
            && n == "_"
        {
            return Some(Pattern::Wildcard(self.bump().span));
        }
        if self.eat_kw(Keyword::Is) {
            return Some(Pattern::Is(self.ty()?));
        }
        Some(Pattern::Value(self.unary()?))
    }

    fn try_stmt(&mut self) -> Option<Stmt> {
        let start = self.bump().span.start; // `try`
        let body = self.line_block()?;
        let mut catch = None;
        if self.eat_kw(Keyword::Catch) {
            let var = if self.at_ident() { Some(self.ident()?) } else { None };
            catch = Some((var, self.line_block()?));
        }
        let finally = if self.eat_kw(Keyword::Finally) { Some(self.line_block()?) } else { None };
        if catch.is_none() && finally.is_none() {
            self.unexpected("'catch' or 'finally'");
        }
        Some(Stmt::Try(Try { span: Span::new(start, self.prev_end()), body, catch, finally }))
    }

    // ---------- expressions ----------

    fn expr(&mut self) -> Option<Expr> {
        if let Some(l) = self.try_lambda() {
            return l;
        }
        self.ternary()
    }

    /// `x => ...`, `(a, b: int) => ...`, `(message: string)` + block, `(n: int) string` + block.
    /// Returns `None` when the tokens are not a lambda.
    fn try_lambda(&mut self) -> Option<Option<Expr>> {
        let start = self.span().start;
        if self.at_ident() && self.at_punct_at(1, Punct::FatArrow) {
            let name = self.ident()?;
            let param = Param {
                span: name.span,
                decorators: Vec::new(),
                name,
                optional: false,
                ty: None,
                default: None,
                named: false,
            };
            self.bump(); // `=>`
            return Some(self.lambda_body(start, vec![param], None));
        }
        if !self.at_punct(Punct::LParen) {
            return None;
        }
        let close = self.matching_close(0)?;
        let after = self.peek_at(close + 1).clone();
        let arrow = matches!(after, TokenKind::Punct(Punct::FatArrow));
        let block = matches!(after, TokenKind::Newline) && matches!(self.peek_at(close + 2), TokenKind::Indent);
        let typed_block = matches!(after, TokenKind::Ident(_) | TokenKind::Punct(Punct::Lt)) && {
            // `(n: int) string` + block: a type must fill the rest of the line
            let save = (self.pos, self.diags.len());
            self.pos += close + 1;
            let ok = self.ty().is_some() && self.block_ahead();
            self.pos = save.0;
            self.diags.truncate(save.1);
            ok
        };
        if !(arrow || block || typed_block) {
            return None;
        }
        let params = match self.params() {
            Some(p) => p.list,
            None => return Some(None),
        };
        let ret = if typed_block {
            match self.ty() {
                Some(t) => Some(t),
                None => return Some(None),
            }
        } else {
            None
        };
        if arrow {
            self.bump();
            return Some(self.lambda_body(start, params, ret));
        }
        self.bump(); // NEWLINE
        let body = self.block();
        let span = Span::new(start, body.span.end);
        Some(Some(Expr {
            span,
            kind: ExprKind::Lambda(Box::new(Lambda { span, params, ret, body: LambdaBody::Block(body) })),
        }))
    }

    /// After `=>`: an expression, or NEWLINE INDENT block.
    fn lambda_body(&mut self, start: usize, params: Vec<Param>, ret: Option<Type>) -> Option<Expr> {
        let body = if self.block_ahead() {
            self.bump();
            LambdaBody::Block(self.block())
        } else {
            let e = self.expr()?;
            if matches!(
                self.peek(),
                TokenKind::Punct(
                    Punct::Eq
                        | Punct::PlusEq
                        | Punct::MinusEq
                        | Punct::StarEq
                        | Punct::SlashEq
                        | Punct::QuestionQuestionEq
                )
            ) {
                let span = self.span();
                self.report(
                    codes::UNEXPECTED_TOKEN,
                    span,
                    "a one-line lambda cannot assign; write `(params)` and the assignment in an indented block",
                );
                return None;
            }
            LambdaBody::Expr(e)
        };
        let span = Span::new(start, self.prev_end());
        Some(Expr { span, kind: ExprKind::Lambda(Box::new(Lambda { span, params, ret, body })) })
    }

    fn ternary(&mut self) -> Option<Expr> {
        let cond = self.coalesce()?;
        if !self.at_punct(Punct::Question) {
            return Some(cond);
        }
        self.bump();
        let then = self.expr()?;
        if !self.expect_punct(Punct::Colon) {
            return None;
        }
        let otherwise = self.expr()?;
        let span = Span::new(cond.span.start, otherwise.span.end);
        Some(Expr {
            span,
            kind: ExprKind::Ternary { cond: Box::new(cond), then: Box::new(then), otherwise: Box::new(otherwise) },
        })
    }

    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        let span = Span::new(left.span.start, right.span.end);
        Expr { span, kind: ExprKind::Binary { op, left: Box::new(left), right: Box::new(right) } }
    }

    fn coalesce(&mut self) -> Option<Expr> {
        let mut left = self.or_expr()?;
        while self.eat_punct(Punct::QuestionQuestion) {
            let right = self.or_expr()?;
            left = Self::binary(left, BinaryOp::Coalesce, right);
        }
        Some(left)
    }

    fn or_expr(&mut self) -> Option<Expr> {
        let mut left = self.and_expr()?;
        while self.eat_kw(Keyword::Or) {
            let right = self.and_expr()?;
            left = Self::binary(left, BinaryOp::Or, right);
        }
        Some(left)
    }

    fn and_expr(&mut self) -> Option<Expr> {
        let mut left = self.not_expr()?;
        while self.eat_kw(Keyword::And) {
            let right = self.not_expr()?;
            left = Self::binary(left, BinaryOp::And, right);
        }
        Some(left)
    }

    fn not_expr(&mut self) -> Option<Expr> {
        if self.at_kw(Keyword::Not) {
            let start = self.bump().span.start;
            let expr = self.not_expr()?;
            let span = Span::new(start, expr.span.end);
            return Some(Expr { span, kind: ExprKind::Unary { op: UnaryOp::Not, expr: Box::new(expr) } });
        }
        self.compare()
    }

    fn compare(&mut self) -> Option<Expr> {
        let left = self.range()?;
        if self.eat_kw(Keyword::Is) {
            let ty = self.ty()?;
            let span = Span::new(left.span.start, ty.span.end);
            return Some(Expr { span, kind: ExprKind::Is { expr: Box::new(left), ty } });
        }
        if self.eat_kw(Keyword::As) {
            let ty = self.ty()?;
            let span = Span::new(left.span.start, ty.span.end);
            return Some(Expr { span, kind: ExprKind::Cast { expr: Box::new(left), ty } });
        }
        let op = match self.peek() {
            TokenKind::Punct(Punct::EqEq) => BinaryOp::Eq,
            TokenKind::Punct(Punct::NotEq) => BinaryOp::NotEq,
            TokenKind::Punct(Punct::Lt) => BinaryOp::Lt,
            TokenKind::Punct(Punct::Gt) if self.glued_gt().is_none() => BinaryOp::Gt,
            TokenKind::Punct(Punct::LtEq) => BinaryOp::LtEq,
            TokenKind::Punct(Punct::GtEq) => BinaryOp::GtEq,
            TokenKind::Keyword(Keyword::In) => BinaryOp::In,
            _ => return Some(left),
        };
        self.bump();
        let right = self.range()?;
        Some(Self::binary(left, op, right))
    }

    fn range(&mut self) -> Option<Expr> {
        let left = self.bit_or()?;
        // a `..` at the start of a line is a cascade, not a range
        if !self.at_punct(Punct::DotDot) || self.tok().nl_before {
            return Some(left);
        }
        self.bump();
        let right = self.bit_or()?;
        // `1..10.map(...)`: the method would apply to 10, not to the range
        if let Some(num_end) = method_on_number(&right) {
            let span = Span::new(left.span.start, right.span.end);
            let fixed = format!("({}){}", &self.src[left.span.start..num_end], &self.src[num_end..right.span.end]);
            self.diags.push(
                Diagnostic::error(codes::RANGE_NEEDS_PARENS, span, "a range followed by a method needs parentheses")
                    .with_fix(span, fixed),
            );
        }
        Some(Self::binary(left, BinaryOp::Range, right))
    }

    fn bit_or(&mut self) -> Option<Expr> {
        let mut left = self.bit_xor()?;
        while self.eat_punct(Punct::Pipe) {
            let right = self.bit_xor()?;
            left = Self::binary(left, BinaryOp::BitOr, right);
        }
        Some(left)
    }

    fn bit_xor(&mut self) -> Option<Expr> {
        let mut left = self.bit_and()?;
        while self.eat_punct(Punct::Caret) {
            let right = self.bit_and()?;
            left = Self::binary(left, BinaryOp::BitXor, right);
        }
        Some(left)
    }

    fn bit_and(&mut self) -> Option<Expr> {
        let mut left = self.shift()?;
        while self.eat_punct(Punct::Amp) {
            let right = self.shift()?;
            left = Self::binary(left, BinaryOp::BitAnd, right);
        }
        Some(left)
    }

    fn shift(&mut self) -> Option<Expr> {
        let mut left = self.additive()?;
        loop {
            let (op, n) = if self.at_punct(Punct::Shl) {
                (BinaryOp::Shl, 1)
            } else {
                match self.glued_gt() {
                    Some((">>", n)) => (BinaryOp::Shr, n),
                    Some((">>>", n)) => (BinaryOp::UShr, n),
                    _ => return Some(left),
                }
            };
            for _ in 0..n {
                self.bump();
            }
            let right = self.additive()?;
            left = Self::binary(left, op, right);
        }
    }

    fn additive(&mut self) -> Option<Expr> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                TokenKind::Punct(Punct::Plus) => BinaryOp::Add,
                TokenKind::Punct(Punct::Minus) => BinaryOp::Sub,
                _ => return Some(left),
            };
            self.bump();
            let right = self.multiplicative()?;
            left = Self::binary(left, op, right);
        }
    }

    fn multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Punct(Punct::Star) => BinaryOp::Mul,
                TokenKind::Punct(Punct::Slash) => BinaryOp::Div,
                TokenKind::Punct(Punct::Percent) => BinaryOp::Rem,
                TokenKind::Punct(Punct::TildeSlash) => BinaryOp::IntDiv,
                _ => return Some(left),
            };
            self.bump();
            let right = self.unary()?;
            left = Self::binary(left, op, right);
        }
    }

    fn unary(&mut self) -> Option<Expr> {
        let op = match self.peek() {
            TokenKind::Punct(Punct::Minus) => UnaryOp::Neg,
            TokenKind::Punct(Punct::Tilde) => UnaryOp::Await,
            TokenKind::Punct(Punct::Caret) => UnaryOp::BitNot,
            TokenKind::Keyword(Keyword::Not) => return self.not_expr(),
            _ => return self.power(),
        };
        let start = self.bump().span.start;
        let expr = self.unary()?;
        let span = Span::new(start, expr.span.end);
        Some(Expr { span, kind: ExprKind::Unary { op, expr: Box::new(expr) } })
    }

    /// `a ** b`: right associative, and the exponent may be negative (`2 ** -1`).
    fn power(&mut self) -> Option<Expr> {
        let base = self.postfix()?;
        if !self.eat_punct(Punct::StarStar) {
            return Some(base);
        }
        let exponent = self.unary()?;
        Some(Self::binary(base, BinaryOp::Pow, exponent))
    }

    fn postfix(&mut self) -> Option<Expr> {
        let e = self.primary()?;
        self.postfix_ops(e)
    }

    fn postfix_ops(&mut self, mut e: Expr) -> Option<Expr> {
        loop {
            match self.peek() {
                TokenKind::Punct(Punct::Dot) | TokenKind::Punct(Punct::QuestionDot) => {
                    let optional = self.at_punct(Punct::QuestionDot);
                    self.bump();
                    let name = self.member_name()?;
                    let span = Span::new(e.span.start, name.span.end);
                    e = Expr { span, kind: ExprKind::Member { object: Box::new(e), name, optional } };
                }
                TokenKind::Punct(Punct::LParen) => {
                    let args = self.args()?;
                    let span = Span::new(e.span.start, self.prev_end());
                    e = Expr { span, kind: ExprKind::Call { callee: Box::new(e), args, type_args: Vec::new() } };
                }
                // `FutureBuilder<String>(...)`, `context.read<Counter>()`
                TokenKind::Punct(Punct::Lt) if matches!(e.kind, ExprKind::Name(_) | ExprKind::Member { .. }) => {
                    let Some(type_args) = self.call_type_args() else {
                        return Some(e);
                    };
                    let args = self.args()?;
                    let span = Span::new(e.span.start, self.prev_end());
                    e = Expr { span, kind: ExprKind::Call { callee: Box::new(e), args, type_args } };
                }
                // `..member = value` / `..method()` on the following lines
                TokenKind::Punct(Punct::DotDot) if self.tok().nl_before && !self.in_cascade => {
                    let outer = std::mem::replace(&mut self.in_cascade, true);
                    let mut sections = Vec::new();
                    while self.at_punct(Punct::DotDot) && self.tok().nl_before {
                        let start = self.bump().span.start;
                        let name = self.member_name()?;
                        let root = Expr { span: Span::new(start, start), kind: ExprKind::CascadeSelf };
                        let first = Expr {
                            span: Span::new(start, name.span.end),
                            kind: ExprKind::Member { object: Box::new(root), name, optional: false },
                        };
                        let access = self.postfix_ops(first)?;
                        let op = match self.peek() {
                            TokenKind::Punct(Punct::Eq) => Some(AssignOp::Set),
                            TokenKind::Punct(Punct::PlusEq) => Some(AssignOp::Add),
                            TokenKind::Punct(Punct::MinusEq) => Some(AssignOp::Sub),
                            TokenKind::Punct(Punct::StarEq) => Some(AssignOp::Mul),
                            TokenKind::Punct(Punct::SlashEq) => Some(AssignOp::Div),
                            TokenKind::Punct(Punct::PercentEq) => Some(AssignOp::Rem),
                            TokenKind::Punct(Punct::QuestionQuestionEq) => Some(AssignOp::Coalesce),
                            TokenKind::Punct(Punct::AmpEq) => Some(AssignOp::BitAnd),
                            TokenKind::Punct(Punct::PipeEq) => Some(AssignOp::BitOr),
                            TokenKind::Punct(Punct::CaretEq) => Some(AssignOp::BitXor),
                            TokenKind::Punct(Punct::ShlEq) => Some(AssignOp::Shl),
                            _ => None,
                        };
                        let assign = match op {
                            Some(op) => {
                                self.bump();
                                Some((op, self.expr()?))
                            }
                            None => None,
                        };
                        sections.push(CascadeSection { span: Span::new(start, self.prev_end()), access, assign });
                    }
                    self.in_cascade = outer;
                    let span = Span::new(e.span.start, self.prev_end());
                    e = Expr { span, kind: ExprKind::Cascade { target: Box::new(e), sections } };
                }
                TokenKind::Punct(Punct::Bang) => {
                    let end = self.bump().span.end;
                    let span = Span::new(e.span.start, end);
                    e = Expr { span, kind: ExprKind::NonNull(Box::new(e)) };
                }
                TokenKind::Punct(Punct::LBracket) => {
                    self.bump();
                    let index = self.expr()?;
                    self.expect_punct(Punct::RBracket);
                    let span = Span::new(e.span.start, self.prev_end());
                    e = Expr { span, kind: ExprKind::Index { object: Box::new(e), index: Box::new(index) } };
                }
                _ => return Some(e),
            }
        }
    }

    /// `<T, U>` right before `(`. Otherwise (`a < b`) nothing is consumed.
    fn call_type_args(&mut self) -> Option<Vec<Type>> {
        // `List<int>()`: the `<` is written right after the name; `a < b` has a space
        if self.span().start != self.prev_end() {
            return None;
        }
        let save = (self.pos, self.diags.len());
        self.bump(); // `<`
        let types = self.type_list();
        if types.is_some() && self.eat_punct(Punct::Gt) && self.at_punct(Punct::LParen) {
            return types;
        }
        self.pos = save.0;
        self.diags.truncate(save.1);
        None
    }

    fn args(&mut self) -> Option<Vec<Arg>> {
        self.bump(); // `(`
        let mut out = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            let start = self.span().start;
            if self.eat_punct(Punct::Ellipsis) {
                let value = self.expr()?;
                out.push(Arg { span: Span::new(start, self.prev_end()), name: None, value, spread: true, ty: None });
            } else if self.at_ident() && self.at_punct_at(1, Punct::Eq) {
                let name = self.ident()?;
                self.bump(); // `=`
                let value = self.expr()?;
                out.push(Arg {
                    span: Span::new(start, self.prev_end()),
                    name: Some(name),
                    value,
                    spread: false,
                    ty: None,
                });
            } else if self.at_ident() && self.at_punct_at(1, Punct::Colon) {
                let name = self.ident()?;
                let colon = self.bump().span; // `:`
                // `name: Type` is a lambda parameter; `name: value` (a call, a literal...) is a named
                // argument written with ':' instead of '='
                let save = (self.pos, self.diags.len());
                let ty = self.ty();
                if let Some(ty) = ty.filter(|_| self.at_punct(Punct::Comma) || self.at_punct(Punct::RParen)) {
                    let value = Expr { span: name.span, kind: ExprKind::Name(name.name) };
                    out.push(Arg {
                        span: Span::new(start, self.prev_end()),
                        name: None,
                        value,
                        spread: false,
                        ty: Some(ty),
                    });
                } else {
                    self.pos = save.0;
                    self.diags.truncate(save.1);
                    self.diags.push(
                        Diagnostic::error(
                            codes::UNEXPECTED_TOKEN,
                            colon,
                            format!("named arguments use '=': {} = ...", name.name),
                        )
                        .with_fix(Span::new(name.span.end, colon.end), " ="),
                    );
                    let value = self.expr()?;
                    out.push(Arg {
                        span: Span::new(start, self.prev_end()),
                        name: Some(name),
                        value,
                        spread: false,
                        ty: None,
                    });
                }
            } else {
                let value = self.expr()?;
                out.push(Arg { span: Span::new(start, self.prev_end()), name: None, value, spread: false, ty: None });
            }
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        if !self.expect_punct(Punct::RParen) {
            return None;
        }
        Some(out)
    }

    fn primary(&mut self) -> Option<Expr> {
        let t = self.tok().clone();
        let span = t.span;
        let kind = match t.kind {
            TokenKind::Ident(name) => {
                self.bump();
                ExprKind::Name(name)
            }
            TokenKind::Number(n) => {
                self.bump();
                ExprKind::Number(n)
            }
            TokenKind::Str(s) => {
                self.bump();
                ExprKind::Str(self.str_pieces(&s))
            }
            TokenKind::Keyword(Keyword::True) => {
                self.bump();
                ExprKind::Bool(true)
            }
            TokenKind::Keyword(Keyword::False) => {
                self.bump();
                ExprKind::Bool(false)
            }
            TokenKind::Keyword(Keyword::Null) => {
                self.bump();
                ExprKind::Null
            }
            TokenKind::Keyword(Keyword::This) => {
                self.bump();
                ExprKind::This
            }
            TokenKind::Keyword(Keyword::Super) => {
                self.bump();
                ExprKind::Super
            }
            TokenKind::Keyword(Keyword::If) => {
                let i = self.if_chain()?;
                return Some(Expr { span: i.span, kind: ExprKind::If(Box::new(i)) });
            }
            TokenKind::Keyword(Keyword::Match) => {
                let m = self.match_expr()?;
                return Some(Expr { span: m.span, kind: ExprKind::Match(Box::new(m)) });
            }
            TokenKind::Punct(Punct::Dollar) => {
                self.bump();
                let name = self.member_name()?;
                return Some(Expr { span: Span::new(span.start, name.span.end), kind: ExprKind::SelfField(name.name) });
            }
            TokenKind::Punct(Punct::LBracket) => {
                self.bump();
                let mut items = Vec::new();
                while !self.at_punct(Punct::RBracket) && !self.at_eof() {
                    let start = self.span().start;
                    if self.eat_punct(Punct::Ellipsis) {
                        let inner = self.expr()?;
                        let span = Span::new(start, inner.span.end);
                        items.push(Expr { span, kind: ExprKind::Spread(Box::new(inner)) });
                        if !self.eat_punct(Punct::Comma) {
                            break;
                        }
                        continue;
                    }
                    // `if cond value`: the item is in the list only when cond holds
                    if self.eat_kw(Keyword::If) {
                        let cond = self.expr()?;
                        let value = self.expr()?;
                        let span = Span::new(start, value.span.end);
                        items.push(Expr {
                            span,
                            kind: ExprKind::ListIf { cond: Box::new(cond), value: Box::new(value) },
                        });
                        if !self.eat_punct(Punct::Comma) {
                            break;
                        }
                        continue;
                    }
                    items.push(self.expr()?);
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RBracket);
                return Some(Expr { span: Span::new(span.start, self.prev_end()), kind: ExprKind::List(items) });
            }
            TokenKind::Punct(Punct::LBrace) => return self.object(),
            TokenKind::Punct(Punct::LParen) => {
                self.bump();
                let inner = self.expr()?;
                self.expect_punct(Punct::RParen);
                return Some(Expr { span: Span::new(span.start, self.prev_end()), kind: inner.kind });
            }
            TokenKind::Keyword(k) => {
                self.report(codes::RESERVED_NAME, span, format!("'{k}' is a reserved word and cannot be used here"));
                self.bump();
                return None;
            }
            _ => {
                self.unexpected("an expression");
                return None;
            }
        };
        Some(Expr { span, kind })
    }

    fn object(&mut self) -> Option<Expr> {
        let start = self.bump().span.start; // `{`
        let mut entries = Vec::new();
        while !self.at_punct(Punct::RBrace) && !self.at_eof() {
            if self.eat_punct(Punct::Ellipsis) {
                entries.push(ObjEntry::Spread(self.expr()?));
            } else if let TokenKind::Str(s) = self.peek().clone() {
                self.bump();
                self.expect_punct(Punct::Colon);
                let value = self.expr()?;
                entries.push(ObjEntry::Pair { key: ObjKey::Str(str_text(&s)), value });
            } else {
                let key = self.member_name()?;
                if self.eat_punct(Punct::Colon) {
                    let value = self.expr()?;
                    entries.push(ObjEntry::Pair { key: ObjKey::Ident(key), value });
                } else {
                    entries.push(ObjEntry::Shorthand(key));
                }
            }
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RBrace);
        Some(Expr { span: Span::new(start, self.prev_end()), kind: ExprKind::Object(entries) })
    }

    fn str_pieces(&mut self, s: &StrLit) -> Vec<StrPiece> {
        let mut out = Vec::new();
        for part in &s.parts {
            match part {
                StrPart::Text(t) => out.push(StrPiece::Text(t.clone())),
                StrPart::Interp { tokens, span } => {
                    let mut toks = tokens.clone();
                    toks.push(Token {
                        kind: TokenKind::Eof,
                        span: Span::new(span.end - 1, span.end - 1),
                        nl_before: false,
                    });
                    let mut sub = Parser::new(self.src, toks);
                    if let Some(e) = sub.expr() {
                        if !sub.at_eof() {
                            sub.unexpected("'}' to close the interpolation");
                        }
                        out.push(StrPiece::Interp(e));
                    }
                    self.diags.append(&mut sub.diags);
                }
            }
        }
        out
    }

    // ---------- types ----------

    fn type_list(&mut self) -> Option<Vec<Type>> {
        let mut out = vec![self.ty()?];
        while self.eat_punct(Punct::Comma) {
            out.push(self.ty()?);
        }
        Some(out)
    }

    fn ty(&mut self) -> Option<Type> {
        let first = self.postfix_type()?;
        if !self.at_punct(Punct::Pipe) {
            return Some(first);
        }
        let mut list = vec![first];
        while self.eat_punct(Punct::Pipe) {
            list.push(self.postfix_type()?);
        }
        let span = Span::new(list[0].span.start, list.last().unwrap().span.end);
        Some(Type { span, kind: TypeKind::Union(list) })
    }

    fn postfix_type(&mut self) -> Option<Type> {
        let mut t = self.primary_type()?;
        loop {
            if self.at_punct(Punct::LBracket) && self.at_punct_at(1, Punct::RBracket) {
                self.bump();
                self.bump();
                t = Type { span: Span::new(t.span.start, self.prev_end()), kind: TypeKind::List(Box::new(t)) };
            } else if self.at_punct(Punct::Question) {
                self.bump();
                t = Type { span: Span::new(t.span.start, self.prev_end()), kind: TypeKind::Nullable(Box::new(t)) };
            } else {
                return Some(t);
            }
        }
    }

    fn primary_type(&mut self) -> Option<Type> {
        let t = self.tok().clone();
        let start = t.span.start;
        match t.kind {
            TokenKind::Punct(Punct::Lt) => {
                self.bump();
                let key = self.ty()?;
                self.expect_punct(Punct::Comma);
                let value = self.ty()?;
                self.expect_punct(Punct::Gt);
                Some(Type {
                    span: Span::new(start, self.prev_end()),
                    kind: TypeKind::Map(Box::new(key), Box::new(value)),
                })
            }
            TokenKind::Ident(_) | TokenKind::Keyword(Keyword::Null) => {
                let mut name = self.member_name()?.name;
                while self.at_punct(Punct::Dot) {
                    self.bump();
                    name.push('.');
                    name.push_str(&self.member_name()?.name);
                }
                let mut args = Vec::new();
                if self.eat_punct(Punct::Lt) {
                    args = self.type_list()?;
                    self.expect_punct(Punct::Gt);
                }
                Some(Type { span: Span::new(start, self.prev_end()), kind: TypeKind::Named { name, args } })
            }
            TokenKind::Str(s) => {
                self.bump();
                Some(Type { span: t.span, kind: TypeKind::StrLit(str_text(&s)) })
            }
            TokenKind::Number(n) => {
                self.bump();
                Some(Type { span: t.span, kind: TypeKind::NumLit(n.text) })
            }
            TokenKind::Punct(Punct::LParen) => {
                let close = self.matching_close(0)?;
                if self.at_punct_at(close + 1, Punct::FatArrow) {
                    let params = self.params()?.list;
                    self.bump(); // `=>`
                    let ret = self.ty()?;
                    return Some(Type {
                        span: Span::new(start, self.prev_end()),
                        kind: TypeKind::Func { params, ret: Box::new(ret) },
                    });
                }
                self.bump();
                let inner = self.ty()?;
                self.expect_punct(Punct::RParen);
                Some(inner)
            }
            _ => {
                self.unexpected("a type");
                None
            }
        }
    }
}

/// Plain text of a string token (interpolations are not allowed where this is used).
fn str_text(s: &StrLit) -> String {
    s.parts
        .iter()
        .map(|p| match p {
            StrPart::Text(t) => t.clone(),
            StrPart::Interp { .. } => String::new(),
        })
        .collect()
}

fn is_void(t: &Type) -> bool {
    matches!(&t.kind, TypeKind::Named { name, args } if name == "void" && args.is_empty())
}

/// `10.map(...)` / `10.length`: returns where the number ends.
fn method_on_number(e: &Expr) -> Option<usize> {
    let mut cur = e;
    let mut chained = false;
    loop {
        match &cur.kind {
            ExprKind::Member { object, .. } => {
                chained = true;
                cur = object;
            }
            ExprKind::Call { callee, .. } => cur = callee,
            ExprKind::Index { object, .. } => cur = object,
            ExprKind::Number(_) if chained => return Some(cur.span.end),
            _ => return None,
        }
    }
}
