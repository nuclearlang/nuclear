//! Lexer: `.nuc` text → tokens.
//!
//! Turns indentation into NEWLINE / INDENT / DEDENT tokens so the parser can work as if there were
//! braces. Rules (spec/nuclear-base.md, section 2):
//! - Inside `( ) [ ] { }` there is no NEWLINE/INDENT/DEDENT (line continuation).
//! - A line starting with `.` continues the previous one (method chains).
//! - `=>` at the end of a line inside brackets opens a block (multi-line lambda). So does a
//!   parenthesized parameter list (`onPressed = ()`) followed by a more indented line.
//! - After a lambda block inside brackets, the `,` before the next item is optional.
//! - A `//` comment at the start of a line continues on the following lines that start with ` * `: the `*`
//!   right under the second `/` (base §2.3). A more indented line in another form is N0021 (with its fix).
//!
//! The lexer never stops: on an error it reports it, recovers and keeps going.

use crate::token::*;
use nuclear_diag::{Code, Diagnostic, Span, codes};

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(src: &str) -> Lexed {
    let mut lx = Lexer::new(src, 0, src.len(), false);
    lx.run();
    lx.finish();
    Lexed { tokens: lx.tokens, comments: lx.comments, diagnostics: lx.diags }
}

/// What is currently "open".
enum Ctx {
    /// Indentation-sensitive zone, with the stack of open levels (`[0, 2, 4]`).
    Layout(Vec<usize>),
    /// Inside `( [ {`: indentation is ignored. `params`: a `(` where an expression starts (after
    /// `=`, `(`, `[` or `,`), so it may hold the parameters of a lambda.
    Bracket { open: Punct, span: Span, params: bool },
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    end: usize,
    /// Sub-lexer for a `${...}` interpolation: no indentation and no EOF.
    sub: bool,
    ctx: Vec<Ctx>,
    tokens: Vec<Token>,
    comments: Vec<Comment>,
    diags: Vec<Diagnostic>,
    at_line_start: bool,
    nl_pending: bool,
    /// Indentation of the current physical line.
    line_indent: usize,
    line_has_token: bool,
    /// The last `)` closed a `(` that may hold lambda parameters.
    closed_params: bool,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str, start: usize, end: usize, sub: bool) -> Self {
        let ctx = if sub {
            vec![Ctx::Bracket { open: Punct::LBrace, span: Span::new(start, start), params: false }]
        } else {
            vec![Ctx::Layout(vec![0])]
        };
        Lexer {
            src,
            pos: start,
            end,
            sub,
            ctx,
            tokens: Vec::new(),
            comments: Vec::new(),
            diags: Vec::new(),
            at_line_start: !sub,
            nl_pending: false,
            line_indent: 0,
            line_has_token: false,
            closed_params: false,
        }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..self.end]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn in_layout(&self) -> bool {
        matches!(self.ctx.last(), Some(Ctx::Layout(_)))
    }

    fn report(&mut self, d: Diagnostic) {
        self.diags.push(d);
    }

    fn error(&mut self, code: Code, span: Span, msg: impl Into<String>) {
        self.diags.push(Diagnostic::error(code, span, msg));
    }

    fn run(&mut self) {
        loop {
            if self.at_line_start && self.in_layout() {
                self.line_start();
                continue;
            }
            let Some(c) = self.peek() else { break };
            match c {
                ' ' | '\t' | '\r' => self.pos += 1,
                '\n' => self.newline(),
                '/' if self.rest().starts_with("//") => self.inline_comment(),
                '"' | '\'' => self.string(c),
                // raw text, `r"\d+"`: no escapes and no interpolation (regular expressions, Windows paths)
                'r' if self.rest()[1..].starts_with(['"', '\'']) => self.raw_string(),
                '0'..='9' => self.number(),
                c if c.is_alphabetic() || c == '_' => self.ident(),
                _ => self.punct(c),
            }
        }
    }

    fn finish(&mut self) {
        if self.sub {
            let unclosed: Vec<(Punct, Span)> = self.ctx[1..]
                .iter()
                .filter_map(|c| match c {
                    Ctx::Bracket { open, span, .. } => Some((*open, *span)),
                    Ctx::Layout(_) => None,
                })
                .collect();
            for (open, span) in unclosed {
                self.error(codes::BRACKET, span, format!("unclosed '{open}'"));
            }
            return;
        }
        while self.ctx.len() > 1 {
            match self.ctx.pop() {
                Some(Ctx::Bracket { open, span, .. }) => {
                    let close = open.closer().map_or("", Punct::as_str);
                    self.error(codes::BRACKET, span, format!("unclosed '{open}': missing '{close}'"));
                }
                Some(Ctx::Layout(ind)) => self.close_levels(ind.len()),
                None => {}
            }
        }
        let depth = self.indents().len();
        self.close_levels(depth);
        self.newline_tok();
        self.synth(TokenKind::Eof);
    }

    // ---------- tokens ----------

    fn push(&mut self, kind: TokenKind, span: Span) {
        self.line_has_token = true;
        let nl_before = std::mem::take(&mut self.nl_pending);
        self.tokens.push(Token { kind, span, nl_before });
    }

    /// Synthetic token (NEWLINE, INDENT, DEDENT, EOF): takes no source text.
    fn synth(&mut self, kind: TokenKind) {
        let span = Span::new(self.pos, self.pos);
        self.tokens.push(Token { kind, span, nl_before: false });
    }

    /// NEWLINE only if the line had content (never two in a row, nor after INDENT/DEDENT).
    fn newline_tok(&mut self) {
        if let Some(t) = self.tokens.last()
            && !matches!(t.kind, TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent)
        {
            self.synth(TokenKind::Newline);
        }
    }

    /// Closes every level of an indentation stack of depth `depth` (keeps the base one).
    fn close_levels(&mut self, depth: usize) {
        if depth > 1 {
            self.newline_tok();
            for _ in 1..depth {
                self.synth(TokenKind::Dedent);
            }
        }
    }

    fn indents(&mut self) -> &mut Vec<usize> {
        match self.ctx.last_mut() {
            Some(Ctx::Layout(v)) => v,
            _ => unreachable!("indents() called outside an indentation zone"),
        }
    }

    // ---------- indentation ----------

    /// Measures indentation starting at `from`: (columns, position after the spaces, were there tabs?).
    fn measure_indent(&self, from: usize) -> (usize, usize, bool) {
        let b = self.src.as_bytes();
        let (mut cols, mut p, mut tab) = (0, from, false);
        while p < self.end {
            match b[p] {
                b' ' => cols += 1,
                b'\t' => {
                    cols += 2;
                    tab = true;
                }
                _ => break,
            }
            p += 1;
        }
        (cols, p, tab)
    }

    fn begin_line(&mut self, indent: usize) {
        self.line_indent = indent;
        self.line_has_token = false;
    }

    fn tab_error(&mut self, line_begin: usize, p: usize, indent: usize) {
        let span = Span::new(line_begin, p);
        self.report(
            Diagnostic::error(codes::TAB_INDENT, span, "indentation uses tabs; use 2 spaces per level")
                .with_fix(span, " ".repeat(indent)),
        );
    }

    /// Start of a line in an indentation zone: skips blank lines and comments, then emits NEWLINE,
    /// INDENT or DEDENT according to the new indentation.
    fn line_start(&mut self) {
        let src = self.src;
        loop {
            let line_begin = self.pos;
            let (indent, p, tab) = self.measure_indent(self.pos);
            let rest = &src[p..self.end];
            if rest.is_empty() {
                self.pos = p;
                self.at_line_start = false;
                return;
            }
            if rest.starts_with('\n') || rest.starts_with("\r\n") {
                self.pos = p + if rest.starts_with('\n') { 1 } else { 2 };
                continue;
            }
            if rest.starts_with("//") {
                self.pos = p;
                self.block_comment(indent);
                continue;
            }
            self.pos = p;
            self.at_line_start = false;
            self.begin_line(indent);
            if tab {
                self.tab_error(line_begin, p, indent);
            }
            if rest.starts_with('.') && !rest.starts_with("...") {
                // `.method()` and `..member` (cascade) continue the statement at their indentation:
                // blocks opened deeper (the body of a lambda) are closed first
                let mut top = *self.indents().last().unwrap();
                if indent < top {
                    self.newline_tok();
                    while indent < top && self.indents().len() > 1 {
                        self.indents().pop();
                        self.synth(TokenKind::Dedent);
                        top = *self.indents().last().unwrap();
                    }
                }
                self.nl_pending = true;
                return;
            }
            if indent % 2 != 0 {
                self.error(
                    codes::ODD_INDENT,
                    Span::new(line_begin, p),
                    format!("indentation must be a multiple of 2 spaces (found {indent})"),
                );
            }
            self.layout(indent);
            return;
        }
    }

    fn layout(&mut self, i: usize) {
        let is_inner = self.ctx.len() > 1;
        let (base, mut top, depth) = match self.ctx.last() {
            Some(Ctx::Layout(ind)) => (ind[0], *ind.last().unwrap(), ind.len()),
            _ => return,
        };

        // Lambda block inside brackets: it ends when the line returns to the indentation of the `=>`.
        // The `,` before the next item may be left out.
        if is_inner && i <= base {
            self.close_levels(depth);
            self.ctx.pop();
            if !self.rest().starts_with([',', ')', ']', '}']) {
                self.synth(TokenKind::Punct(Punct::Comma));
            }
            return;
        }

        if i > top {
            self.newline_tok();
            self.indents().push(i);
            self.synth(TokenKind::Indent);
            return;
        }

        self.newline_tok();
        while i < top && self.indents().len() > 1 {
            self.indents().pop();
            self.synth(TokenKind::Dedent);
            top = *self.indents().last().unwrap();
        }
        if i != top {
            let span = Span::new(self.pos - i.min(self.pos), self.pos);
            self.error(
                codes::BAD_DEDENT,
                span,
                format!("indentation ({i}) does not match any open block (the closest one is at {top})"),
            );
        }
    }

    fn newline(&mut self) {
        let prev_indent = self.line_indent;
        self.pos += 1;
        self.nl_pending = true;
        if self.in_layout() {
            self.at_line_start = true;
            return;
        }
        // Inside ( [ {: indentation is ignored, but tabs are still reported and a trailing `=>`
        // opens a lambda block.
        let line_begin = self.pos;
        let (indent, p, tab) = self.measure_indent(self.pos);
        let blank = self.src[p..self.end].starts_with(['\n', '\r']);
        if tab && !blank {
            self.tab_error(line_begin, p, indent);
        }
        self.begin_line(indent);
        let last = self.tokens.last().map(|t| &t.kind);
        let arrow = matches!(last, Some(TokenKind::Punct(Punct::FatArrow)));
        // `onPressed = ()` + a more indented line: the parameters of a lambda with a block
        let params = matches!(last, Some(TokenKind::Punct(Punct::RParen)))
            && self.closed_params
            && !blank
            && indent > prev_indent
            && !self.src[p..self.end].starts_with(['.', ')', ']', '}', ',']);
        if arrow || params {
            self.ctx.push(Ctx::Layout(vec![prev_indent]));
            self.at_line_start = true;
        }
    }

    // ---------- comments ----------

    /// `//` at the start of a line: continues on the following lines that start with ` * `, the `*` right under
    /// the second `/`. The text of each one is what follows `* ` (its extra indentation is kept).
    fn block_comment(&mut self, indent: usize) {
        let (src, end) = (self.src, self.end);
        let line_end = |from: usize| src[from..end].find('\n').map_or(end, |i| from + i);
        let start = self.pos;
        let first_end = line_end(start);
        let first = src[start + 2..first_end].trim_end();
        let mut text = first.strip_prefix(' ').unwrap_or(first).to_string();
        let mut last_end = first_end;
        while last_end < end {
            let (j, q, _) = self.measure_indent(last_end + 1);
            let le = line_end(q);
            let line = src[q..le].trim_end();
            if line.is_empty() || j <= indent {
                break;
            }
            let rest = match line.strip_prefix('*') {
                Some(rest) if j == indent + 1 && (rest.is_empty() || rest.starts_with(' ')) => {
                    rest.strip_prefix(' ').unwrap_or(rest)
                }
                // a more indented line without ` * ` (or with the `*` elsewhere): still part of the comment
                other => {
                    let rest = other.map_or(line, str::trim_start);
                    let line_start = src[..q].rfind('\n').map_or(0, |i| i + 1);
                    let stars = line.len() - line.trim_start_matches('*').len();
                    let space = usize::from(stars > 0 && line[stars..].starts_with(' '));
                    let span = Span::new(line_start, q + stars + space);
                    let fix = format!("{}* ", " ".repeat(indent + 1));
                    let msg = "a comment continues on lines that start with ` * `, the `*` under the second `/`";
                    self.report(Diagnostic::error(codes::COMMENT_LINE, span, msg).with_fix(span, fix));
                    rest
                }
            };
            text.push('\n');
            text.push_str(rest);
            last_end = le;
        }
        // a ` *` left at the end (the editor's Enter) is not an empty line of the text
        while text.ends_with('\n') {
            text.pop();
        }
        self.comments.push(Comment { span: Span::new(start, last_end), text, own_line: true });
        self.pos = if last_end < end { last_end + 1 } else { end };
    }

    /// `//` after code or inside brackets: runs to the end of the line.
    fn inline_comment(&mut self) {
        let start = self.pos;
        let e = self.src[start..self.end].find('\n').map_or(self.end, |i| start + i);
        let text = self.src[start + 2..e].trim().to_string();
        self.comments.push(Comment { span: Span::new(start, e), text, own_line: !self.line_has_token });
        self.pos = e;
    }

    // ---------- words and numbers ----------

    fn ident(&mut self) {
        let start = self.pos;
        let len: usize = self.rest().chars().take_while(|c| c.is_alphanumeric() || *c == '_').map(char::len_utf8).sum();
        self.pos += len;
        let word = &self.src[start..self.pos];
        let kind = match Keyword::from_text(word) {
            Some(k) => TokenKind::Keyword(k),
            None => TokenKind::Ident(word.to_string()),
        };
        self.push(kind, Span::new(start, self.pos));
    }

    fn number(&mut self) {
        let (src, end, start) = (self.src, self.end, self.pos);
        let b = src.as_bytes();
        let take = |p: &mut usize, f: fn(u8) -> bool| {
            while *p < end && (f(b[*p]) || b[*p] == b'_') {
                *p += 1;
            }
        };
        let mut p = start;
        let mut bad = false;
        let kind = if src[p..end].starts_with("0x") || src[p..end].starts_with("0X") {
            p += 2;
            let digits = p;
            take(&mut p, |c| c.is_ascii_hexdigit());
            bad |= p == digits;
            NumberKind::Hex
        } else if src[p..end].starts_with("0b") || src[p..end].starts_with("0B") {
            p += 2;
            let digits = p;
            take(&mut p, |c| c == b'0' || c == b'1');
            bad |= p == digits;
            NumberKind::Bin
        } else {
            take(&mut p, |c| c.is_ascii_digit());
            // `0..10` is a range, not a decimal: the dot must be followed by a digit
            let mut float = false;
            if p + 1 < end && b[p] == b'.' && b[p + 1].is_ascii_digit() {
                p += 1;
                take(&mut p, |c| c.is_ascii_digit());
                float = true;
            }
            // exponent: `1e3`, `1.5e-3`
            if p < end && (b[p] == b'e' || b[p] == b'E') {
                let mut q = p + 1;
                if q < end && (b[q] == b'+' || b[q] == b'-') {
                    q += 1;
                }
                if q < end && b[q].is_ascii_digit() {
                    p = q;
                    take(&mut p, |c| c.is_ascii_digit());
                    float = true;
                }
            }
            if float { NumberKind::Float } else { NumberKind::Int }
        };
        let suffix: usize =
            src[p..end].chars().take_while(|c| c.is_alphanumeric() || *c == '_').map(char::len_utf8).sum();
        bad |= suffix > 0;
        p += suffix;
        self.pos = p;
        let span = Span::new(start, p);
        let text = src[start..p].to_string();
        if bad {
            self.error(codes::BAD_NUMBER, span, format!("invalid number '{text}'"));
        }
        self.push(TokenKind::Number(Number { text, kind }), span);
    }

    // ---------- symbols ----------

    fn punct(&mut self, c: char) {
        let start = self.pos;
        let rest = self.rest();

        // `value!` (not null): `!` right after a name, `)` or `]`
        let after_value = self.src[..start]
            .chars()
            .next_back()
            .is_some_and(|p| p.is_alphanumeric() || p == '_' || p == ')' || p == ']');
        if c == '!' && after_value && !rest.starts_with("!=") {
            self.pos = start + 1;
            self.push(TokenKind::Punct(Punct::Bang), Span::new(start, start + 1));
            return;
        }

        // Syntax from other languages: reported with the fix, and recovered as the equivalent token.
        let foreign: Option<(&str, &str, Option<TokenKind>)> = if rest.starts_with("===") {
            Some(("===", "==", Some(TokenKind::Punct(Punct::EqEq))))
        } else if rest.starts_with("!==") {
            Some(("!==", "!=", Some(TokenKind::Punct(Punct::NotEq))))
        } else if rest.starts_with("&&") {
            Some(("&&", "and", Some(TokenKind::Keyword(Keyword::And))))
        } else if rest.starts_with("||") {
            Some(("||", "or", Some(TokenKind::Keyword(Keyword::Or))))
        } else if rest.starts_with('!') && !rest.starts_with("!=") {
            Some(("!", "not ", Some(TokenKind::Keyword(Keyword::Not))))
        } else if rest.starts_with(';') {
            Some((";", "", None))
        } else {
            None
        };
        if let Some((pat, with, kind)) = foreign {
            let span = Span::new(start, start + pat.len());
            let msg = if with.is_empty() {
                format!("NuclearLang does not use '{pat}'")
            } else {
                format!("NuclearLang does not use '{pat}'; use '{}'", with.trim())
            };
            self.report(Diagnostic::error(codes::FOREIGN_SYNTAX, span, msg).with_fix(span, with));
            self.pos = span.end;
            if let Some(k) = kind {
                self.push(k, span);
            }
            return;
        }

        let best = Punct::ALL.iter().copied().filter(|p| rest.starts_with(p.as_str())).max_by_key(|p| p.as_str().len());
        let Some(p) = best else {
            let span = Span::new(start, start + c.len_utf8());
            self.pos = span.end;
            self.error(codes::UNEXPECTED_CHAR, span, format!("unexpected character '{c}'"));
            return;
        };
        let span = Span::new(start, start + p.as_str().len());
        self.pos = span.end;
        if p.is_open() {
            let params = p == Punct::LParen
                && matches!(
                    self.tokens.last().map(|t| &t.kind),
                    Some(TokenKind::Punct(Punct::Eq | Punct::LParen | Punct::LBracket | Punct::Comma))
                );
            self.ctx.push(Ctx::Bracket { open: p, span, params });
            self.push(TokenKind::Punct(p), span);
        } else if p.is_close() {
            self.close_bracket(p, span);
        } else {
            self.push(TokenKind::Punct(p), span);
        }
    }

    fn close_bracket(&mut self, p: Punct, span: Span) {
        // `)` also closes the lambda blocks opened inside the bracket
        while self.ctx.len() > 1
            && let Some(Ctx::Layout(ind)) = self.ctx.last()
        {
            let depth = ind.len();
            self.close_levels(depth);
            self.ctx.pop();
        }
        match self.ctx.last() {
            Some(Ctx::Bracket { open, params, .. }) if self.ctx.len() > 1 => {
                let open = *open;
                self.closed_params = *params && p == Punct::RParen;
                if open.closer() != Some(p) {
                    let want = open.closer().map_or("", Punct::as_str);
                    self.error(codes::BRACKET, span, format!("expected '{want}' to close '{open}', found '{p}'"));
                }
                self.ctx.pop();
            }
            _ => self.error(codes::BRACKET, span, format!("'{p}' has no matching opener")),
        }
        self.push(TokenKind::Punct(p), span);
    }

    // ---------- strings ----------

    fn raw_string(&mut self) {
        let start = self.pos;
        self.pos += 1; // r
        let q = self.peek().unwrap_or('"');
        let fence = if q == '"' { "\"\"\"" } else { "'''" };
        let triple = self.rest().starts_with(fence);
        self.pos += if triple { 3 } else { 1 };
        let body = self.pos;
        let mut terminated = false;
        let mut text_end = self.pos;
        while let Some(c) = self.peek() {
            if (triple && self.rest().starts_with(fence)) || (!triple && c == q) {
                text_end = self.pos;
                self.pos += if triple { 3 } else { 1 };
                terminated = true;
                break;
            }
            if c == '\n' && !triple {
                break;
            }
            self.pos += c.len_utf8();
        }
        if !terminated {
            text_end = self.pos;
        }
        let span = Span::new(start, self.pos);
        if !terminated {
            self.error(codes::UNTERMINATED_STRING, span, "unterminated string");
        }
        let mut parts = vec![StrPart::Text(self.src[body..text_end].to_string())];
        if triple {
            parts = dedent(parts);
        }
        self.push(TokenKind::Str(StrLit { parts, multiline: triple }), span);
    }

    fn string(&mut self, q: char) {
        let start = self.pos;
        // `"""` or `'''`: both quote kinds behave the same
        let fence = if q == '"' { "\"\"\"" } else { "'''" };
        let triple = self.rest().starts_with(fence);
        self.pos += if triple { 3 } else { 1 };
        let mut parts = Vec::new();
        let mut buf = String::new();
        let mut terminated = false;
        while let Some(c) = self.peek() {
            if triple {
                if self.rest().starts_with(fence) {
                    self.pos += 3;
                    terminated = true;
                    break;
                }
            } else if c == q {
                self.pos += 1;
                terminated = true;
                break;
            }
            if c == '\n' && !triple {
                break;
            }
            if c == '\\' {
                buf.push('\\');
                self.pos += 1;
                if let Some(n) = self.peek()
                    && (n != '\n' || triple)
                {
                    buf.push(n);
                    self.pos += n.len_utf8();
                }
                continue;
            }
            if c == '$' && self.rest().starts_with("${") {
                if !buf.is_empty() {
                    parts.push(StrPart::Text(std::mem::take(&mut buf)));
                }
                self.interpolation(&mut parts, triple);
                continue;
            }
            buf.push(c);
            self.pos += c.len_utf8();
        }
        if !buf.is_empty() {
            parts.push(StrPart::Text(buf));
        }
        let span = Span::new(start, self.pos);
        if !terminated {
            self.error(codes::UNTERMINATED_STRING, span, "unterminated string");
        }
        if triple {
            parts = dedent(parts);
        }
        let mut bad = Vec::new();
        for part in &mut parts {
            if let StrPart::Text(t) = part {
                *t = unescape(t, &mut bad);
            }
        }
        for b in bad {
            self.error(codes::BAD_ESCAPE, span, format!("invalid escape sequence '\\{b}'"));
        }
        self.push(TokenKind::Str(StrLit { parts, multiline: triple }), span);
    }

    /// `${expr}`: finds the closing `}` and lexes the inside with a sub-lexer.
    fn interpolation(&mut self, parts: &mut Vec<StrPart>, triple: bool) {
        let src = self.src;
        let dollar = self.pos;
        let inner_start = dollar + 2;
        let mut i = inner_start;
        let mut depth = 1usize;
        let mut close = None;
        while i < self.end {
            let c = src[i..self.end].chars().next().unwrap();
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
                '"' | '\'' => {
                    i = self.skip_quoted(i, c);
                    continue;
                }
                '\n' if !triple => break,
                _ => {}
            }
            i += c.len_utf8();
        }
        let Some(ci) = close else {
            self.error(codes::BAD_INTERPOLATION, Span::new(dollar, i), "unclosed interpolation: missing '}'");
            self.pos = i;
            return;
        };
        if src[inner_start..ci].trim().is_empty() {
            self.error(codes::BAD_INTERPOLATION, Span::new(dollar, ci + 1), "empty interpolation");
        }
        let mut sub = Lexer::new(src, inner_start, ci, true);
        sub.run();
        sub.finish();
        self.diags.append(&mut sub.diags);
        parts.push(StrPart::Interp { tokens: sub.tokens, span: Span::new(dollar, ci + 1) });
        self.pos = ci + 1;
    }

    /// Skips a string nested inside an interpolation. Returns the position after the closing quote.
    fn skip_quoted(&self, i: usize, q: char) -> usize {
        let mut j = i + 1;
        while j < self.end {
            let c = self.src[j..self.end].chars().next().unwrap();
            if c == '\\' {
                j += 1;
                if let Some(n) = self.src[j..self.end].chars().next() {
                    j += n.len_utf8();
                }
                continue;
            }
            if c == q {
                return j + 1;
            }
            if c == '\n' {
                return j;
            }
            j += c.len_utf8();
        }
        self.end
    }
}

/// Multi-line strings: drops the first and last lines if blank, and the common indentation.
fn dedent(parts: Vec<StrPart>) -> Vec<StrPart> {
    const HOLE: char = '\u{0}'; // marks where each interpolation was
    let mut s = String::new();
    let mut interps = Vec::new();
    for p in parts {
        match p {
            StrPart::Text(t) => s.push_str(&t),
            interp @ StrPart::Interp { .. } => {
                s.push(HOLE);
                interps.push(interp);
            }
        }
    }
    if let Some(nl) = s.find('\n')
        && s[..nl].trim().is_empty()
    {
        s.drain(..=nl);
    }
    if let Some(nl) = s.rfind('\n')
        && s[nl + 1..].trim().is_empty()
    {
        s.truncate(nl);
    }
    let indent_of = |l: &str| l.len() - l.trim_start_matches(' ').len();
    let min = s.split('\n').filter(|l| !l.trim().is_empty()).map(indent_of).min().unwrap_or(0);
    let lines: Vec<&str> =
        s.split('\n').map(|l| if indent_of(l) >= min { &l[min..] } else { l.trim_start_matches(' ') }).collect();
    let joined = lines.join("\n");
    let mut out = Vec::new();
    let mut interps = interps.into_iter();
    for (idx, seg) in joined.split(HOLE).enumerate() {
        if idx > 0
            && let Some(i) = interps.next()
        {
            out.push(i);
        }
        if !seg.is_empty() {
            out.push(StrPart::Text(seg.to_string()));
        }
    }
    out
}

fn unescape(t: &str, bad: &mut Vec<char>) -> String {
    let mut out = String::with_capacity(t.len());
    let mut it = t.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some(e @ ('\\' | '"' | '\'' | '$')) => out.push(e),
            Some('u') if it.peek() == Some(&'{') => {
                it.next();
                let hex: String = it.by_ref().take_while(|c| *c != '}').collect();
                match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    Some(ch) => out.push(ch),
                    None => bad.push('u'),
                }
            }
            Some(other) => {
                bad.push(other);
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}
