---
layer: L3
component: nuclear
read-when: you change the lexer or the parser (indentation, comments, blocks inside parentheses, generic calls)
---

# Lexer and parser

## Indentation

The lexer emits INDENT/DEDENT so the parser works as if there were braces. Inside `()[]{}` indentation is
ignored, except for a lambda block: after `=>` + newline, or after `(params)` (a `(` after `=`, `(`, `[` or
`,`) + a more indented line. The block ends when the indentation returns to its first line; the lexer
synthesizes the `,` if missing. A `.method()` line less indented than the open block closes it and continues
the chain (parser: `chain_after_block`).

## Comments (ADR-0080)

A `//` at the start of a line continues on ` * ` lines (the `*` in the column of the second `/`); the text of
each is what follows `* `. A more indented line of another shape still belongs to the comment but is N0021,
with its fix. Trailing empty lines (a lone ` *`) are dropped. The comment right above a declaration is its
doc (`DocComment`, ADR-0079).

## Punctuation details

- `!` right after a name/`)`/`]` is `Punct::Bang` (non-null); anywhere else it is N0009 (use `not`).
- The `<` of a generic call must touch the name (`call_type_args`); with a space it is a comparison.
- `>>`, `>>>`, `>>=`, `>>>=` are NOT tokens: the parser joins `>` tokens written together (`glued_gt`), so
  `List<List<int>>` still closes two generics. `<<`, `<<=`, `**`, `~/`, `&`, `^`, `%=`, `&=`, `|=`, `^=` are tokens
  (ADR-0086). Precedence: comparison < `|` < `^` < `&` < shifts < `+` < `*` `~/` < unary (`-` `~` `^`) < `**`.
- `r"..."` (an `r` touching a quote) is raw text: no escapes, no interpolation (`raw_string`). Numbers: `0b1010`
  (`NumberKind::Bin`), exponents `1.5e-3` (Float).
- `..` with `nl_before` is a cascade, not a range. The parser does not open cascades inside a section
  (`in_cascade`); a `.` line below a section continues that section (as in Dart).

## AST shape

`Decl.words` (kinds/modifiers before the name), clauses after it, `Decl.generic_bounds` (`<T: Comparable>`),
`Call.type_args`, `ExprKind::{Spread, ListIf, Cascade, CascadeSelf}`, destructuring patterns. `BinaryOp` has
`IntDiv Pow BitOr BitXor BitAnd Shl Shr UShr`, `UnaryOp::BitNot`, `AssignOp::{Rem, BitAnd, BitOr, BitXor, Shl,
Shr, UShr}` (ADR-0086). The LSP walks
the AST itself: changing variants breaks `crates/nuclear-lsp` and `dragonfly/`.

## Habits of other languages (ADR-0088)

`habits::scan` (called by `parse`) masks strings and comments, then looks at each line for forms of TypeScript,
Dart, Python, Java and Go (`{ }` blocks, `await`, `const x =`, `new`, `class A extends B`, the return type first,
`def`, trailing `:`, `elif`, `None`, `#`, `x++`, `import`...) and reports N0022 with a fix when it is exact. `parse`
drops the other diagnostics of those lines. A false positive is a bug: every `.nuc` of the repository and every
`nuclear` block of the docs must give none (checked by hand when the rules change; tests in `habits.rs`).
