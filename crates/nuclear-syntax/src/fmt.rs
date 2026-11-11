//! `nuclear fmt`: the canonical layout of a `.nuc` file (plan 005 F1).
//!
//! Only the space between tokens changes: indentation (2 spaces per block), one space or none between tokens on a
//! line, at most one blank line, one blank line before a top-level declaration that follows a block, no trailing
//! spaces, one final newline. Members are never reordered and lines are never joined or split. Every token is
//! copied as written, and the result is checked to lex to the same tokens and comments; if it does not, the file
//! is left as it is (an error, never a changed program).

use crate::lexer::lex;
use crate::token::{Comment, Keyword, Punct, Token, TokenKind};

/// The formatted text, or why the file was not formatted.
pub fn format(src: &str) -> Result<String, String> {
    let lexed = lex(src);
    if lexed.diagnostics.iter().any(|d| d.is_error()) {
        return Err("the file has syntax errors (run nuclear check)".into());
    }
    let out = Printer::new(src, &lexed.comments).run(&lexed.tokens);
    if signature(src) != signature(&out) {
        return Err("formatting would change the tokens or comments; file left as it is (please report it)".into());
    }
    Ok(out)
}

/// What must not change: each token (without its position) with its line break, and each comment's text.
fn signature(src: &str) -> (Vec<(String, bool)>, Vec<String>, bool) {
    let l = lex(src);
    let tokens = l.tokens.iter().map(|t| (t.kind.short(), t.nl_before)).collect();
    let comments = l.comments.iter().map(|c| c.text.clone()).collect();
    (tokens, comments, l.diagnostics.iter().any(|d| d.is_error()))
}

struct Printer<'a> {
    src: &'a str,
    comments: &'a [Comment],
    next_comment: usize,
    out: String,
    /// Block level of the current line (INDENT/DEDENT seen so far).
    level: usize,
    /// Open brackets: inside them indentation is free, so lines keep their columns, shifted with their block.
    depth: usize,
    /// New column − old column of the last line that starts a block statement.
    delta: isize,
    /// Old and new column of each block level (to place a comment at the level it was written at).
    columns: Vec<(usize, usize)>,
    /// Level of the last block line, to add a blank line before a top-level declaration after a block.
    line_level: usize,
}

impl<'a> Printer<'a> {
    fn new(src: &'a str, comments: &'a [Comment]) -> Self {
        Printer {
            src,
            comments,
            next_comment: 0,
            out: String::with_capacity(src.len()),
            level: 0,
            depth: 0,
            delta: 0,
            columns: Vec::new(),
            line_level: 0,
        }
    }

    fn run(mut self, tokens: &[Token]) -> String {
        let mut prev: Option<&Token> = None;
        let mut before: Option<&Token> = None;
        let mut layout_break = false;
        for t in tokens {
            match t.kind {
                TokenKind::Newline => layout_break = true,
                TokenKind::Indent => {
                    self.level += 1;
                    layout_break = true;
                }
                TokenKind::Dedent => {
                    self.level = self.level.saturating_sub(1);
                    layout_break = true;
                }
                TokenKind::Eof => {}
                // tokens the lexer adds (the `,` after a lambda block) are not in the text
                _ if t.span.start == t.span.end => {}
                _ => {
                    let gap_start = prev.map_or(0, |p| p.span.end);
                    let gap = &self.src[gap_start..t.span.start];
                    if prev.is_none() || gap.contains('\n') {
                        self.line_gap(gap_start, t.span.start, Some(layout_break));
                    } else if let Some(p) = prev
                        && space(before, p, t, !gap.is_empty())
                    {
                        self.out.push(' ');
                    }
                    self.out.push_str(&self.src[t.span.start..t.span.end]);
                    if let TokenKind::Punct(p) = &t.kind {
                        if p.is_open() {
                            self.depth += 1;
                        } else if p.is_close() {
                            self.depth = self.depth.saturating_sub(1);
                        }
                    }
                    before = prev;
                    prev = Some(t);
                    layout_break = false;
                }
            }
        }
        let end = prev.map_or(0, |p| p.span.end);
        self.line_gap(end, self.src.len(), None);
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        self.out
    }

    /// A gap that crosses lines: a trailing comment, line breaks (at most one blank line), comments on their own
    /// lines, and the indentation of the next token (`next`: whether it starts a block line; `None` at the end).
    fn line_gap(&mut self, from: usize, to: usize, next: Option<bool>) {
        let starts_block_line = next == Some(true) && self.depth == 0;
        // a top-level line after a block gets a blank line before it (and before its comments)
        let mut want_blank = starts_block_line && self.level == 0 && self.line_level > 0;
        let mut cursor = from;
        while self.next_comment < self.comments.len() && self.comments[self.next_comment].span.start < to {
            let c = &self.comments[self.next_comment];
            self.next_comment += 1;
            if c.span.start < from {
                continue;
            }
            let breaks = self.src[cursor..c.span.start].matches('\n').count();
            if breaks == 0 && !self.out.is_empty() {
                self.out.push(' ');
                self.out.push_str(self.src[c.span.start..c.span.end].trim_end());
            } else {
                let col = self.comment_column(column(self.src, c.span.start));
                self.breaks(breaks, want_blank && col == 0);
                if col == 0 {
                    want_blank = false;
                }
                self.comment(c, col);
            }
            cursor = c.span.end;
        }
        let breaks = self.src[cursor..to].matches('\n').count();
        if next.is_none() {
            return;
        }
        self.breaks(breaks, want_blank);
        let old = column(self.src, to);
        let new = if starts_block_line {
            let new = 2 * self.level;
            self.delta = new as isize - old as isize;
            self.columns.truncate(self.level);
            while self.columns.len() < self.level {
                self.columns.push((usize::MAX, 2 * self.columns.len()));
            }
            self.columns.push((old, new));
            self.line_level = self.level;
            new
        } else {
            self.shifted(old)
        };
        self.out.extend(std::iter::repeat_n(' ', new));
    }

    /// Line breaks: at least one (except at the start of the file), at most one blank line.
    fn breaks(&mut self, breaks: usize, blank: bool) {
        if self.out.is_empty() {
            return;
        }
        let n = breaks.clamp(1, 2).max(if blank { 2 } else { 1 });
        while self.out.ends_with(' ') {
            self.out.pop();
        }
        self.out.extend(std::iter::repeat_n('\n', n));
    }

    /// A comment on its own line at column `col`; its ` * ` continuation lines move with it.
    fn comment(&mut self, c: &Comment, col: usize) {
        let shift = col as isize - column(self.src, c.span.start) as isize;
        let text = &self.src[c.span.start..c.span.end];
        for (i, line) in text.split('\n').enumerate() {
            if i == 0 {
                self.out.extend(std::iter::repeat_n(' ', col));
                self.out.push_str(line.trim_end());
            } else {
                self.out.push('\n');
                let lead = line.len() - line.trim_start().len();
                let n = (lead as isize + shift).max(0) as usize;
                self.out.extend(std::iter::repeat_n(' ', n));
                self.out.push_str(line.trim());
            }
        }
    }

    /// Where a comment written at `old` goes: the new column of the block level written there, or shifted.
    fn comment_column(&self, old: usize) -> usize {
        if self.depth == 0
            && let Some((_, new)) = self.columns.iter().find(|(o, _)| *o == old)
        {
            return *new;
        }
        self.shifted(old)
    }

    fn shifted(&self, old: usize) -> usize {
        (old as isize + self.delta).max(0) as usize
    }
}

/// Column of a byte position (bytes since the last line break: indentation is spaces).
fn column(src: &str, pos: usize) -> usize {
    pos - src[..pos].rfind('\n').map_or(0, |i| i + 1)
}

fn punct(t: &Token) -> Option<Punct> {
    if let TokenKind::Punct(p) = t.kind { Some(p) } else { None }
}

/// A token that ends a value: after it, `-` is a subtraction, `(` a call and `[` an index.
fn operand(t: &Token) -> bool {
    match &t.kind {
        TokenKind::Ident(_) | TokenKind::Number(_) | TokenKind::Str(_) => true,
        TokenKind::Keyword(k) => matches!(k, Keyword::True | Keyword::False | Keyword::Null | Keyword::This | Keyword::Super),
        TokenKind::Punct(p) => matches!(p, Punct::RParen | Punct::RBracket | Punct::RBrace | Punct::Bang),
        _ => false,
    }
}

/// One space between `a` and `b` on the same line? `before` is the token before `a`; `had` says whether the
/// source had a space there (kept as it is around the tokens whose spacing depends on the meaning: `<` and `>`
/// of generics or comparisons, `?` of a nullable type, `*`, `|`, `&`, `^`, `..`, `...`, `$`).
fn space(before: Option<&Token>, a: &Token, b: &Token, had: bool) -> bool {
    use Punct::*;
    let (pa, pb) = (punct(a), punct(b));
    let free = |p: Option<Punct>| matches!(p, Some(Lt | Gt | Question | Star | Pipe | Amp | Caret | DotDot | Ellipsis | Dollar));
    if free(pa) || free(pb) {
        return had;
    }
    // `a: int` and `{ "a": 1 }`, but `c ? a : b`: the space before `:` is kept as written
    if pb == Some(Colon) {
        return had;
    }
    if matches!(pb, Some(RParen | RBracket | Comma | Dot | QuestionDot | Bang)) {
        return false;
    }
    if pb == Some(RBrace) {
        return pa != Some(LBrace);
    }
    if matches!(pa, Some(LParen | LBracket | Dot | QuestionDot | At | Tilde)) {
        return false;
    }
    if matches!(pa, Some(Minus | Plus)) {
        // `-x` after an operator or at the start of an expression; `a - b` after a value
        return before.is_some_and(operand);
    }
    if matches!(pb, Some(LParen | LBracket)) {
        return !operand(a);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::format;

    fn same(src: &str) {
        assert_eq!(format(src).unwrap(), src, "not a fixed point");
    }

    #[test]
    fn spaces_between_tokens() {
        let src = "usecase GetUser(repo:UserRepository)\n  exec(id:int)   Result<UserError, User>\n    ~repo.find( id )\n";
        let out = format(src).unwrap();
        assert_eq!(out, "usecase GetUser(repo: UserRepository)\n  exec(id: int) Result<UserError, User>\n    ~repo.find(id)\n");
        same(&out);
    }

    #[test]
    fn indentation_comments_and_blank_lines() {
        let src = "\n\n// header\nPoint\n    x: int   // the x\n\n\n\n    // y below\n    y: int\nf(a: int) int\n    a - -1\n";
        let out = format(src).unwrap();
        assert_eq!(out, "// header\nPoint\n  x: int // the x\n\n  // y below\n  y: int\n\nf(a: int) int\n  a - -1\n");
        same(&out);
    }

    #[test]
    fn generics_maps_and_calls_keep_their_meaning() {
        same("items: list<Map<string, int>> = []\nx = where({ \"id\": id })\ny = a < b and c > d\nz = xs[0]?.name!\n");
        same("v = ok ? a : b\n");
        assert_eq!(format("m = {\"a\":1}\n").unwrap(), "m = { \"a\": 1 }\n");
    }

    #[test]
    fn strings_are_copied_as_written() {
        same("s = \"\"\"\n  line ${a +b}\n    more\n  \"\"\"\nr = r\"a\\d\"\n");
    }

    #[test]
    fn a_broken_file_is_not_formatted() {
        assert!(format("x = (\n").is_err());
    }
}

