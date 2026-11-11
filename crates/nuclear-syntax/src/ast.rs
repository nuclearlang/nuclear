//! AST of the NuclearLang base language: what the developer wrote, with no target-language details.
//!
//! Every node keeps its `Span` so diagnostics and the LSP can point at the source.

use crate::token::{Comment, Number};
use nuclear_diag::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn is_pascal(&self) -> bool {
        is_pascal(&self.name)
    }
}

/// `User`, `MyApp` → true. `_State`, `user`, `_active` → false.
pub fn is_pascal(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_uppercase())
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct File {
    pub items: Vec<Item>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Use(Use),
    Decl(Decl),
    Stmt(Stmt),
}

// ---------- use ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub span: Span,
    pub path: UsePath,
    /// `use fs/promises{ readFile, writeFile }`
    pub names: Vec<Ident>,
    /// `use lib2/lib2 as lib2`, `use * as path from path`
    pub alias: Option<Ident>,
    /// Dialect words after the path: `show foo`, `hide bar`, `deferred`.
    pub clauses: Vec<UseClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UsePath {
    /// `fs/promises`, `@nestjs/common`, `material` (written without quotes).
    Module(String),
    /// `"./setup"` (a file of the project).
    File(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseClause {
    pub word: Ident,
    pub names: Vec<Ident>,
}

// ---------- declarations: "header + block" ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Decorator {
    pub span: Span,
    /// `Get`, `override`, `nest.Injectable`
    pub name: String,
    pub args: Option<Vec<Arg>>,
}

/// Every declaration has the same shape:
/// `[decorators] [words...] [~]Name[<G>][(params)] [: Bases] [> Interfaces] [ReturnType | clauses]` + body.
#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub span: Span,
    pub decorators: Vec<Decorator>,
    /// Words before the name: dialect kinds (`stateless`) and modifiers (`static`, `abstract`).
    pub words: Vec<Ident>,
    /// `~name()` in the declaration: forces the function to be async.
    pub is_async: bool,
    pub name: Ident,
    pub generics: Vec<Ident>,
    /// `<T: Comparable>`: the bound of each generic (same order as `generics`).
    pub generic_bounds: Vec<Option<Type>>,
    /// `None` = no parentheses. For a function with a body this means "inferred parameters".
    pub params: Option<Params>,
    /// `: User, Person` (inheritance).
    pub bases: Vec<Type>,
    /// `> Serializable` (implements).
    pub implements: Vec<Type>,
    pub ret: Option<Type>,
    /// `on String`, `with Flyer`.
    pub clauses: Vec<Clause>,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub span: Span,
    pub list: Vec<Param>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub span: Span,
    pub decorators: Vec<Decorator>,
    pub name: Ident,
    pub optional: bool,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    /// Written inside `{ }` in the signature (named parameter; only meaningful in Dart).
    pub named: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub word: Ident,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    /// No body: signature, enum item, module with only a constructor.
    None,
    /// Block of a PascalCase declaration (module, interface, enum, dialect kind).
    Members(Vec<Member>),
    /// Block of a function or method.
    Block(Block),
    /// `sumar(a: int, b: int) int = a + b`
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Field(Field),
    Method(Decl),
    Ctor(Ctor),
    EnumItem(EnumItem),
}

impl Member {
    pub fn span(&self) -> Span {
        match self {
            Member::Field(f) => f.span,
            Member::Method(d) => d.span,
            Member::Ctor(c) => c.span,
            Member::EnumItem(e) => e.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub span: Span,
    pub decorators: Vec<Decorator>,
    pub words: Vec<Ident>,
    pub name: Ident,
    pub optional: bool,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
}

/// `(params)` on its own line inside a module, with an optional body.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctor {
    pub span: Span,
    pub params: Params,
    pub body: Option<Block>,
}

/// `user` or `admin("administrator")`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumItem {
    pub span: Span,
    pub name: Ident,
    pub args: Vec<Expr>,
}

// ---------- statements ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Assign(Assign),
    If(If),
    For(For),
    While {
        span: Span,
        cond: Expr,
        body: Block,
    },
    Match(Match),
    Try(Try),
    Return {
        span: Span,
        value: Option<Expr>,
    },
    Throw {
        span: Span,
        value: Expr,
    },
    Break(Span),
    Continue(Span),
    /// `yield value`; `all` = `yield ...values` (every value of another generator).
    Yield {
        span: Span,
        value: Expr,
        all: bool,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expr(e) => e.span,
            Stmt::Assign(a) => a.span,
            Stmt::If(i) => i.span,
            Stmt::For(f) => f.span,
            Stmt::While { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Throw { span, .. }
            | Stmt::Yield { span, .. }
            | Stmt::Break(span)
            | Stmt::Continue(span) => *span,
            Stmt::Match(m) => m.span,
            Stmt::Try(t) => t.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Coalesce,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UShr,
}

impl AssignOp {
    pub fn as_str(self) -> &'static str {
        match self {
            AssignOp::Set => "=",
            AssignOp::Add => "+=",
            AssignOp::Sub => "-=",
            AssignOp::Mul => "*=",
            AssignOp::Div => "/=",
            AssignOp::Rem => "%=",
            AssignOp::Coalesce => "??=",
            AssignOp::BitAnd => "&=",
            AssignOp::BitOr => "|=",
            AssignOp::BitXor => "^=",
            AssignOp::Shl => "<<=",
            AssignOp::Shr => ">>=",
            AssignOp::UShr => ">>>=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub span: Span,
    pub target: Expr,
    pub ty: Option<Type>,
    pub op: AssignOp,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub span: Span,
    /// `if` and every `else if`.
    pub branches: Vec<(Expr, Block)>,
    pub otherwise: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub span: Span,
    /// `for x in xs` → [x]; `for key, value in map` → [key, value].
    pub vars: Vec<Ident>,
    pub iter: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub span: Span,
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub body: ArmBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArmBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `'error'`, `42`, `true`, `null`, `Role.admin`
    Value(Expr),
    /// `is User`
    Is(Type),
    /// `_`
    Wildcard(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Try {
    pub span: Span,
    pub body: Block,
    pub catch: Option<(Option<Ident>, Block)>,
    pub finally: Option<Block>,
}

// ---------- expressions ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Name(String),
    /// `$field`
    SelfField(String),
    This,
    Super,
    Number(Number),
    Str(Vec<StrPiece>),
    Bool(bool),
    Null,
    List(Vec<Expr>),
    Object(Vec<ObjEntry>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then: Box<Expr>,
        otherwise: Box<Expr>,
    },
    Is {
        expr: Box<Expr>,
        ty: Type,
    },
    Member {
        object: Box<Expr>,
        name: Ident,
        optional: bool,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// `f(x)`, `FutureBuilder<String>(...)`: `type_args` are the explicit type arguments.
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
        type_args: Vec<Type>,
    },
    /// `...items` inside a list.
    Spread(Box<Expr>),
    /// `value!`: the value is not null.
    NonNull(Box<Expr>),
    /// `value as Type`
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },
    /// `if cond value` inside a list: the item is there only when cond holds.
    ListIf {
        cond: Box<Expr>,
        value: Box<Expr>,
    },
    /// `target` followed by lines `..member = value` / `..method()`.
    Cascade {
        target: Box<Expr>,
        sections: Vec<CascadeSection>,
    },
    /// The object of a cascade section (`..color`): written by the dialect.
    CascadeSelf,
    Lambda(Box<Lambda>),
    If(Box<If>),
    Match(Box<Match>),
}

/// One `..` line of a cascade: `access` starts at `ExprKind::CascadeSelf` (`..style.color`), with an
/// optional assignment (`..color = Colors.red`).
#[derive(Debug, Clone, PartialEq)]
pub struct CascadeSection {
    pub span: Span,
    pub access: Expr,
    pub assign: Option<(AssignOp, Expr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrPiece {
    Text(String),
    Interp(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjEntry {
    /// `role: "user"`, `"julian": user`
    Pair { key: ObjKey, value: Expr },
    /// `{ text }`
    Shorthand(Ident),
    /// `{ ...base }`
    Spread(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjKey {
    Ident(Ident),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    /// `~call()`
    Await,
    /// `^x`: bitwise complement
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Coalesce,
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    In,
    Range,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    /// `~/`
    IntDiv,
    /// `**`, right associative, above the unary minus (`-2 ** 2` is -4)
    Pow,
    BitOr,
    BitXor,
    BitAnd,
    Shl,
    Shr,
    UShr,
}

impl BinaryOp {
    /// Precedence from spec/nuclear-base.md section 6.1 (higher binds tighter).
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOp::Coalesce => 2,
            BinaryOp::Or => 3,
            BinaryOp::And => 4,
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq
            | BinaryOp::In => 6,
            BinaryOp::Range => 7,
            BinaryOp::BitOr => 8,
            BinaryOp::BitXor => 9,
            BinaryOp::BitAnd => 10,
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::UShr => 11,
            BinaryOp::Add | BinaryOp::Sub => 12,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem | BinaryOp::IntDiv => 13,
            BinaryOp::Pow => 15,
        }
    }

    /// Key used in a dialect's `[operators]` table.
    pub fn key(self) -> &'static str {
        match self {
            BinaryOp::Coalesce => "coalesce",
            BinaryOp::Or => "or",
            BinaryOp::And => "and",
            BinaryOp::Eq => "eq",
            BinaryOp::NotEq => "not_eq",
            BinaryOp::Lt => "lt",
            BinaryOp::Gt => "gt",
            BinaryOp::LtEq => "lt_eq",
            BinaryOp::GtEq => "gt_eq",
            BinaryOp::In => "in",
            BinaryOp::Range => "range",
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::Mul => "mul",
            BinaryOp::Div => "div",
            BinaryOp::Rem => "rem",
            BinaryOp::IntDiv => "int_div",
            BinaryOp::Pow => "pow",
            BinaryOp::BitOr => "bit_or",
            BinaryOp::BitXor => "bit_xor",
            BinaryOp::BitAnd => "bit_and",
            BinaryOp::Shl => "shl",
            BinaryOp::Shr => "shr",
            BinaryOp::UShr => "ushr",
        }
    }

    pub const ALL: &'static [BinaryOp] = &[
        BinaryOp::Coalesce,
        BinaryOp::Or,
        BinaryOp::And,
        BinaryOp::Eq,
        BinaryOp::NotEq,
        BinaryOp::Lt,
        BinaryOp::Gt,
        BinaryOp::LtEq,
        BinaryOp::GtEq,
        BinaryOp::In,
        BinaryOp::Range,
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Rem,
        BinaryOp::IntDiv,
        BinaryOp::Pow,
        BinaryOp::BitOr,
        BinaryOp::BitXor,
        BinaryOp::BitAnd,
        BinaryOp::Shl,
        BinaryOp::Shr,
        BinaryOp::UShr,
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub span: Span,
    /// `name = value`
    pub name: Option<Ident>,
    pub value: Expr,
    /// `...list`
    pub spread: bool,
    /// `users.filter(user: User)`: a typed lambda parameter. Only valid when the call is followed by a
    /// block; the parser then turns the arguments into the lambda's parameters.
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub span: Span,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub body: LambdaBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    Expr(Expr),
    Block(Block),
}

// ---------- types ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub span: Span,
    pub kind: TypeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// `string`, `User`, `Promise<string>`, `Colors.Shade`
    Named { name: String, args: Vec<Type> },
    /// `User[]`
    List(Box<Type>),
    /// `<string, User>`
    Map(Box<Type>, Box<Type>),
    /// `string?`
    Nullable(Box<Type>),
    /// `'user' | 'assistant'`
    Union(Vec<Type>),
    /// `'user'`
    StrLit(String),
    /// `42`
    NumLit(String),
    /// `(a: int) => int`
    Func { params: Vec<Param>, ret: Box<Type> },
}
