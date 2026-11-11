//! Syntax of the NuclearLang base language: tokens, lexer, AST and parser.

pub mod ast;
pub mod doc;
pub mod fmt;
pub mod habits;
pub mod lexer;
pub mod parser;
pub mod token;

pub use doc::DocComment;
pub use lexer::{Lexed, lex};
pub use parser::{Parsed, parse, parse_type};
