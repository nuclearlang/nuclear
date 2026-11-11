//! Tokens: the "words" the lexer hands to the parser.

use nuclear_diag::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// There was a line break before this token. Inside `( )` no NEWLINE is emitted, but the parser
    /// needs this to separate parameters written one per line without commas.
    pub nl_before: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Keyword(Keyword),
    Number(Number),
    Str(StrLit),
    Punct(Punct),
    /// End of a logical line.
    Newline,
    /// A block starts (the line is more indented).
    Indent,
    /// A block ends (the line returns to an outer level).
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    /// Text as written (`1_000`, `0xff`). Each dialect decides how to emit it.
    pub text: String,
    pub kind: NumberKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    Int,
    Float,
    Hex,
    /// `0b1010`
    Bin,
}

/// Processed string: escapes resolved and, in multi-line strings, the common indentation removed.
#[derive(Debug, Clone, PartialEq)]
pub struct StrLit {
    pub parts: Vec<StrPart>,
    pub multiline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrPart {
    Text(String),
    /// `${expr}`: the expression's tokens. The parser turns them into an expression.
    Interp {
        tokens: Vec<Token>,
        span: Span,
    },
}

/// Comments are not in the token list; they are kept apart for documentation and the LSP.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub span: Span,
    /// Text without `//`. Continuation lines keep their relative indentation.
    pub text: String,
    /// `true` if the comment is on its own line (it may document the next declaration).
    pub own_line: bool,
}

/// Generates an enum of fixed words with their text, the full list and lookup by text.
macro_rules! word_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $variant:ident = $text:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($(#[$vm])* $variant),* }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),*];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$variant => $text),* }
            }

            pub fn from_text(s: &str) -> Option<Self> {
                match s { $($text => Some($name::$variant),)* _ => None }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

word_enum! {
    /// Reserved words of the base language. Source: languages/nuclear.toml (a test checks they match).
    /// Dialect words (`stateless`, `extension`, `static`...) are not here: they arrive as Ident.
    Keyword {
        Use = "use", From = "from", As = "as",
        If = "if", Else = "else", For = "for", In = "in", While = "while", Match = "match",
        Return = "return", Break = "break", Continue = "continue", Yield = "yield",
        Try = "try", Catch = "catch", Finally = "finally", Throw = "throw",
        And = "and", Or = "or", Not = "not", Is = "is",
        True = "true", False = "false", Null = "null", This = "this", Super = "super",
    }
}

word_enum! {
    Punct {
        LParen = "(", RParen = ")", LBracket = "[", RBracket = "]", LBrace = "{", RBrace = "}",
        Comma = ",", Dot = ".", DotDot = "..", Ellipsis = "...", Colon = ":",
        Question = "?", QuestionDot = "?.", QuestionQuestion = "??", QuestionQuestionEq = "??=",
        Eq = "=", EqEq = "==", NotEq = "!=", Lt = "<", Gt = ">", LtEq = "<=", GtEq = ">=",
        Plus = "+", Minus = "-", Star = "*", Slash = "/", Percent = "%",
        PlusEq = "+=", MinusEq = "-=", StarEq = "*=", SlashEq = "/=", PercentEq = "%=",
        /// `a ** b` = power; `a ~/ b` = integer division.
        StarStar = "**", TildeSlash = "~/",
        /// Bitwise: `a & b`, `a | b` (Pipe), `a ^ b`, `^a` (complement), `a << b`; `>>` and `>>>` are
        /// two or three `>` written together (so `List<List<int>>` still closes two generics).
        Amp = "&", Caret = "^", Shl = "<<",
        AmpEq = "&=", PipeEq = "|=", CaretEq = "^=", ShlEq = "<<=",
        FatArrow = "=>", ThinArrow = "->", Pipe = "|",
        /// `@Name(...)` = decorator (`@Get(":id")`, `@override`).
        At = "@",
        /// `~call()` = await. A function that awaits becomes async automatically.
        Tilde = "~",
        /// `$field` = module attribute (this.field).
        Dollar = "$",
        /// `value!` = the value is not null (postfix, written right after the expression).
        Bang = "!",
    }
}

impl Punct {
    pub fn is_open(self) -> bool {
        matches!(self, Punct::LParen | Punct::LBracket | Punct::LBrace)
    }

    pub fn is_close(self) -> bool {
        matches!(self, Punct::RParen | Punct::RBracket | Punct::RBrace)
    }

    pub fn closer(self) -> Option<Punct> {
        match self {
            Punct::LParen => Some(Punct::RParen),
            Punct::LBracket => Some(Punct::RBracket),
            Punct::LBrace => Some(Punct::RBrace),
            _ => None,
        }
    }
}

impl TokenKind {
    /// Short form, used in tests and inside interpolations: `if`, `(`, `NL`, `IN`, `DE`.
    pub fn short(&self) -> String {
        match self {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Keyword(k) => k.to_string(),
            TokenKind::Number(n) => n.text.clone(),
            TokenKind::Str(s) => s.to_string(),
            TokenKind::Punct(p) => p.to_string(),
            TokenKind::Newline => "NL".into(),
            TokenKind::Indent => "IN".into(),
            TokenKind::Dedent => "DE".into(),
            TokenKind::Eof => "EOF".into(),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "ident   {s}"),
            TokenKind::Keyword(k) => write!(f, "kw      {k}"),
            TokenKind::Number(n) => write!(f, "number  {}", n.text),
            TokenKind::Str(s) => write!(f, "string  {s}"),
            TokenKind::Punct(p) => write!(f, "punct   {p}"),
            TokenKind::Newline => f.write_str("NEWLINE"),
            TokenKind::Indent => f.write_str("INDENT"),
            TokenKind::Dedent => f.write_str("DEDENT"),
            TokenKind::Eof => f.write_str("EOF"),
        }
    }
}

impl fmt::Display for StrLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        for part in &self.parts {
            match part {
                StrPart::Text(t) => write!(f, "{}", t.escape_debug())?,
                StrPart::Interp { tokens, .. } => {
                    let inner: Vec<String> = tokens.iter().map(|t| t.kind.short()).collect();
                    write!(f, "${{{}}}", inner.join(" "))?;
                }
            }
        }
        f.write_str("\"")
    }
}
