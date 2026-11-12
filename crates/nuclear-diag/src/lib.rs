//! NuclearLang diagnostics.
//!
//! Every error has a stable code (`N0009`), a position and, when possible, the exact fix.
//! They are rendered as text for people or as JSON for agents (spec/nuclear-base.md, section 9).
//! The list of codes is documented in spec/diagnostics.md (a test checks they match).

use serde_json::json;
use std::fmt::Write;

/// Byte range `[start, end)` in the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}

/// Stable error code. `explanation` is the context given to agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Code {
    pub id: &'static str,
    pub title: &'static str,
    pub explanation: &'static str,
}

pub mod codes {
    use super::Code;

    pub const TAB_INDENT: Code = Code {
        id: "N0002",
        title: "tab in indentation",
        explanation: "NuclearLang indents with exactly 2 spaces; tabs are not allowed.",
    };
    pub const BAD_DEDENT: Code = Code {
        id: "N0004",
        title: "indentation does not match any block",
        explanation: "When a block ends, the line must return to the indentation of an enclosing block.",
    };
    pub const ODD_INDENT: Code = Code {
        id: "N0005",
        title: "indentation is not a multiple of 2",
        explanation: "Each indentation level is exactly 2 spaces.",
    };
    pub const UNEXPECTED_CHAR: Code = Code {
        id: "N0006",
        title: "unexpected character",
        explanation: "This character is not part of NuclearLang syntax.",
    };
    pub const UNTERMINATED_STRING: Code = Code {
        id: "N0007",
        title: "unterminated string",
        explanation: "A \"...\" or '...' string must close on the same line; use \"\"\"...\"\"\" or '''...''' for multi-line text.",
    };
    pub const BRACKET: Code = Code {
        id: "N0008",
        title: "unmatched bracket",
        explanation: "Every ( [ { must be closed by its matching ) ] }.",
    };
    pub const FOREIGN_SYNTAX: Code = Code {
        id: "N0009",
        title: "syntax from another language",
        explanation: "NuclearLang uses `and`, `or`, `not`, `==`, `!=` and has no `;`.",
    };
    pub const BAD_INTERPOLATION: Code = Code {
        id: "N0010",
        title: "invalid interpolation",
        explanation: "Interpolation is written \"${expr}\" with a non-empty expression and a closing }.",
    };
    pub const BAD_ESCAPE: Code = Code {
        id: "N0011",
        title: "invalid escape sequence",
        explanation: "Valid escapes: \\n \\t \\r \\0 \\\\ \\\" \\' \\$ \\u{hex}.",
    };
    pub const BAD_NUMBER: Code = Code {
        id: "N0012",
        title: "invalid number",
        explanation: "Numbers are 42, 1_000, 3.14 or 0xff, with no letters after them.",
    };

    pub const RESERVED_NAME: Code = Code {
        id: "N0001",
        title: "reserved word used as a name",
        explanation: "Reserved words of the base language (if, for, use, match...) cannot be names.",
    };
    pub const UNEXPECTED_TOKEN: Code = Code {
        id: "N0013",
        title: "unexpected token",
        explanation: "The parser expected a different token here; see the message for what was expected.",
    };
    pub const RANGE_NEEDS_PARENS: Code = Code {
        id: "N0014",
        title: "range followed by a method needs parentheses",
        explanation: "Write (1..10).map(...) instead of 1..10.map(...).",
    };
    pub const LAMBDA_NEEDS_BLOCK: Code = Code {
        id: "N0015",
        title: "typed lambda parameter without a block",
        explanation: "list.map(x: T) declares lambda parameters, so the call must be followed by an indented block. For a one-line lambda write list.map((x: T) => expr).",
    };
    pub const NESTED_FUNCTION: Code = Code {
        id: "N0016",
        title: "function declared inside a body",
        explanation: "Functions cannot be declared inside another function. Assign an anonymous function: name = (params) Type, followed by its block.",
    };
    pub const STATEMENT_IN_MODULE: Code = Code {
        id: "N0017",
        title: "statement in a module body",
        explanation: "A module body holds fields (name: Type = value), methods and the constructor. Code that runs on construction goes in the constructor block: (params) followed by an indented block.",
    };
    pub const NESTED_MODULE: Code = Code {
        id: "N0018",
        title: "module declared inside a module",
        explanation: "Modules are declared at the top level of a file.",
    };
    pub const BAD_ASSIGN_TARGET: Code = Code {
        id: "N0019",
        title: "invalid assignment target",
        explanation: "Only names, $fields, members (a.b) and indexes (a[i]) can be assigned.",
    };
    pub const VOID_ONE_LINER: Code = Code {
        id: "N0020",
        title: "one-line function returning void",
        explanation: "A function that returns void must be written with an indented block.",
    };
    pub const COMMENT_LINE: Code = Code {
        id: "N0021",
        title: "comment line without ` * `",
        explanation: "Each line that continues a `//` comment starts with ` * `: the `*` goes right under the second `/` of the `//`.",
    };
    pub const FOREIGN_HABIT: Code = Code {
        id: "N0022",
        title: "habit of another language",
        explanation: "A form of TypeScript, Dart, Python or Java (`{ }` blocks, `await`, `const x =`, `new`, `class`, `def`, `elif`, `None`, `#` comments, the return type first). The message gives the NuclearLang form and, when it is exact, a fix.",
    };
    pub const DUPLICATE_DECL: Code = Code {
        id: "N0102",
        title: "duplicate declaration",
        explanation: "Two declarations in the same file produce the same name.",
    };
    pub const BRANCHING_VALUE: Code = Code {
        id: "N0103",
        title: "if/match inside an expression needs one expression per branch",
        explanation: "Branches of several lines are allowed when the if/match is the value of an assignment or of a return; inside a larger expression every branch must be a single expression.",
    };
    pub const ENUM_SYNTAX: Code = Code {
        id: "N0104",
        title: "invalid enum item",
        explanation: "An enum block only holds items written as name or name(value).",
    };
    pub const OUT_OF_SCOPE: Code = Code {
        id: "N0105",
        title: "name used outside the block that created it",
        explanation: "A variable created inside a block (if, for, while, match, try) exists only in that block. To take a value out of an if or match, use it as a value: x = if cond ...",
    };
    pub const UNDEFINED_NAME: Code = Code {
        id: "N0106",
        title: "undefined name",
        explanation: "The name is not a variable, parameter, member, declaration or import of this file.",
    };
    pub const MIXED_PATTERN: Code = Code {
        id: "N0107",
        title: "destructuring pattern mixes new and existing names",
        explanation: "A pattern either declares new variables or assigns existing ones: [a, b] = xs with a and b both new, or both already declared.",
    };
    pub const UNHANDLED_RESULT: Code = Code {
        id: "N0108",
        title: "result that can fail is not handled",
        explanation: "A call that returns Result<E, T> propagates its error automatically only inside a function that also returns a Result. Elsewhere, handle it with match (Ok(x) -> ... / Err(e) -> ...), keep it in a variable typed Result<T, E>, or declare the function's return type as Result<T, E>.",
    };
    pub const RESULT_ERROR_TYPE: Code = Code {
        id: "N0109",
        title: "propagated error has another type",
        explanation: "An error propagates only to a function whose Result has the same error type. Convert it first: call(x).error(e => Err(MyError(e))).",
    };
    pub const RESULT_NOT_HOISTABLE: Code = Code {
        id: "N0110",
        title: "result cannot propagate from this position",
        explanation: "A Result is unwrapped before its statement runs. Inside a branch that may not run (the right side of and, or, ??, a ternary, an else if or while condition, an if in a list) assign it to a variable first.",
    };
    pub const RESULT_PATTERN: Code = Code {
        id: "N0111",
        title: "invalid use of a Result",
        explanation: "A match on a Result has arms Ok(name) ->, Err(name) -> and _ ->, one pattern per arm. A callback of .ok()/.error() returns Ok(...)/Err(...) in every branch (it transforms) or in none (it only observes). An async function gives its Result with ~: match ~load(id).",
    };
    pub const ALIAS_TARGET: Code = Code {
        id: "N0112",
        title: "alias of a method that cannot be reached",
        explanation: "name() = Module.method makes name the same function as method. A method that is not static is called on a new Module(), so every constructor parameter of Module needs a default.",
    };
    pub const STATEMENTS_AND_MAIN: Code = Code {
        id: "N0113",
        title: "top-level statements in a file with main",
        explanation: "In a target where top-level statements run inside main (Dart), a file cannot have both: move the statements into main(). Top-level variables (`limit = 10`) are allowed.",
    };
    pub const LIBRARY_NAME: Code = Code {
        id: "N0114",
        title: "not a member of Nuclear's standard library",
        explanation: "The members of string, lists, maps, sets, numbers, Bytes, DateTime, Duration, Regex and Match are Nuclear's own (spec/library.md, `nuclear help library`), the same in every dialect: a target's name (push, len, toUpperCase, contains) is replaced by the Nuclear one given in the message.",
    };
    pub const RECEIVER_TYPE: Code = Code {
        id: "N0115",
        title: "the receiver's type is not known",
        explanation: "A member of the standard library is written differently for each base type in this dialect, and the type of the value before the dot is not known: declare it (`items: string[] = ...`, a typed parameter) or build the value from a literal or a typed call.",
    };
    pub const CONTRACT: Code = Code {
        id: "N0307",
        title: "contract not available",
        explanation: "The manifest declares a contract (contracts(name = \"../path\" | \"https://...\")) that cannot be read: build the backend project at that path first, or fetch a url contract with `nuclear contracts update` (it keeps a copy in contracts/<name>/contract.json).",
    };
    pub const FIELD_RULE: Code = Code {
        id: "N0308",
        title: "a field decorator's rule is not met",
        explanation: "A decorator of a field has rules of its platform ([decorators.fields] of platform.toml): the field is optional, its type is a list or not, its type is a declaration with a decorator (an @entity), a sibling field exists (`projectId` for `@manyToOne project`), an argument names a field of the field's type (`inverse`). The message says which rule and the fix.",
    };
    pub const PLATFORM_DECORATOR: Code = Code {
        id: "N0309",
        title: "not a decorator of the platform",
        explanation: "A project of a platform (backend, mobile, web) writes only the decorators its platform declares (`decorators` of platform.toml): one that is not there would be dropped or written as something else by the dialect. The message names the closest one, or what replaced a removed decorator (`@generated` → `@primary(generated = true)`, ADR-0100). Decorators in PascalCase (a converter, a target's own decorator) are not checked.",
    };
    pub const UNKNOWN_MEMBER: Code = Code {
        id: "N0116",
        title: "unknown member of a declaration of the project",
        explanation: "The value's type is a declaration of this project whose members are all known (its fields, methods and constructor parameters, its project bases, and the runtime classes it or its kind inherits when the type definitions list their members), and it has no member with that name. A model generated from a contract changes when the backend changes: rebuild and read the generated file.",
    };
    pub const KIND_PACKAGE: Code = Code {
        id: "N0306",
        title: "kind package without an implementation for the dialect",
        explanation: "A package of extension kinds (kinds/<name>/, ADR-0095) implements some dialects of its platform; enabling it in config.nuc ties the project to those dialects. Build with one of them, or add a folder for this dialect to the package.",
    };
    pub const UNKNOWN_HEADER_WORD: Code = Code {
        id: "N0201",
        title: "unknown header word for the dialect",
        explanation: "Words before a declaration's name must be kinds or modifiers defined by the active dialect (dialect files: [kinds.*], [modifiers.*]).",
    };
    pub const UNKNOWN_CLAUSE: Code = Code {
        id: "N0202",
        title: "unknown clause for the dialect",
        explanation: "Words after a declaration's name (on String, with Flyer) must be clauses defined by the active dialect ([clauses.*]).",
    };
    pub const MISSING_TEMPLATE: Code = Code {
        id: "N0203",
        title: "the dialect does not define a template",
        explanation: "The active dialect (or the dialect it extends) has no [syntax] template for this construct.",
    };
    pub const SHORTCUT_COLLISION: Code = Code {
        id: "N0204",
        title: "shortcut collision",
        explanation: "Two shortcuts, kinds or auto-imports define the same name. Each name must be defined once across a dialect and the dialects it extends; to replace a parent entry, list it in [dialect] replaces.",
    };
    pub const SIMILAR_SHORTCUT: Code = Code {
        id: "N0205",
        title: "shortcut very similar to another",
        explanation: "Two shortcut names differ by one character or only by case, which invites typos.",
    };
    pub const BAD_DIALECT_FILE: Code = Code {
        id: "N0206",
        title: "invalid dialect file",
        explanation: "The dialect file is not valid TOML, has a value of the wrong type, or a template with invalid syntax.",
    };
    pub const DIALECT_NOT_FOUND: Code = Code {
        id: "N0207",
        title: "dialect not found",
        explanation: "No dialect with that name was found in the dialect search path, or `extends` forms a cycle.",
    };
    pub const SINGLE_BASE: Code = Code {
        id: "N0208",
        title: "the target allows a single base",
        explanation: "The target language of this dialect supports extending only one base.",
    };
    pub const KIND_REQUIRES_MEMBER: Code = Code {
        id: "N0209",
        title: "kind requires a member",
        explanation: "This dialect kind requires a member (for example stateless requires build).",
    };
    pub const UNKNOWN_TEMPLATE_VAR: Code = Code {
        id: "N0210",
        title: "unknown template variable",
        explanation: "A dialect template uses a variable that the transpiler does not provide for that template.",
    };
    pub const UNKNOWN_USE_CLAUSE: Code = Code {
        id: "N0211",
        title: "unknown use clause",
        explanation: "Words after a use path (show, hide, deferred) must be listed in the dialect's [use] clauses.",
    };
    pub const SHORTCUT_IS_KEYWORD: Code = Code {
        id: "N0212",
        title: "shortcut named like a base keyword",
        explanation: "Shortcut names cannot be reserved words of the base language.",
    };
    pub const KIND_METHOD_NOT_ALLOWED: Code = Code {
        id: "N0215",
        title: "method not allowed in this kind",
        explanation: "The kind declares the only methods it can have ([kinds.<k>] methods); a usecase has a single exec method.",
    };
    pub const KIND_RETURN_TYPE: Code = Code {
        id: "N0216",
        title: "the kind requires another return type",
        explanation: "Every method of this kind must return the type its dialect names ([kinds.<k>] returns), for example Result<E, T> in a usecase.",
    };
    pub const DECORATOR_ARGS: Code = Code {
        id: "N0217",
        title: "unknown or missing argument of a decorator",
        explanation: "The dialect lists the arguments of the decorator ([decorators.args]): each named argument, each field of an object argument, and the ones that are required (!).",
    };
    pub const KIND_IGNORES: Code = Code {
        id: "N0218",
        title: "the kind does not write that part of the declaration",
        explanation: "The template of the kind never writes the generics (<T>) or the interfaces (> I) of the declaration, so they would be lost: remove them or use a kind (or a plain declaration) that writes them.",
    };
    pub const UNKNOWN_NAMED_ARG: Code = Code {
        id: "N0219",
        title: "named argument the constructor does not have",
        explanation: "The type definitions list the parameters of the constructor; the message names the closest one and all of them. `nuclear defs show <Class> --dialect <d>` prints the constructor.",
    };
    pub const UNKNOWN_CONSTRUCTOR: Code = Code {
        id: "N0220",
        title: "unknown class",
        explanation: "A PascalCase call that is not declared in the project, imported, nor in the dialect's type definitions. `nuclear defs search <text> --dialect <d>` finds the real name.",
    };
    pub const KIND_METHOD_BODY: Code = Code {
        id: "N0221",
        title: "a method of this kind needs a body",
        explanation: "The kind's methods do something ([kinds.<k>] needs_body): an appcomponent method is an alias of a use case (`find() = GetOne.exec`) or has a body. A method without one would be generated empty.",
    };
    pub const NOT_AWAITED: Code = Code {
        id: "N0222",
        title: "an asynchronous call used without `~`",
        explanation: "A member or an index of what an asynchronous call returns (`books.exec(a).length`) reads a value that is not there yet: await the call, `(~books.exec(a)).length`.",
    };
    pub const ARCH_CALLER: Code = Code {
        id: "N0301",
        title: "architecture: this declaration cannot use that one",
        explanation: "The dialect's [architecture] callers says which kinds can use each kind (a usecase only from an appcomponent, a service only from a usecase or an aggregate), and [architecture] names which kinds can use a name (BuildContext only in views).",
    };
    pub const ARCH_PRIVATE: Code = Code {
        id: "N0302",
        title: "architecture: private to its component",
        explanation: "Only the kinds listed in [architecture] public (the appcomponent) are visible from other components; use the component through its appcomponent.",
    };
    pub const PLATFORM_CONTRACT: Code = Code {
        id: "N0305",
        title: "a dialect does not match its platform",
        explanation: "A dialect in dialects/<platform>/ must implement that platform's contract (platform.toml, ADR-0093): the same kinds, and no consumed decorator or plugin the platform does not declare, so the Nuclear of a project never changes with its dialect. A platform plugin the dialect does not bring yet is a warning.",
    };
    pub const PLUGIN_DISABLED: Code = Code {
        id: "N0304",
        title: "a plugin the project has not enabled",
        explanation: "The kind or the name belongs to an optional plugin of the dialect (Genkit, Kafka, LangChain...). A project carries a plugin only when its config.nuc enables it: plugins(name = true). The message gives the exact line.",
    };
    pub const PROJECT: Code = Code {
        id: "N0303",
        title: "invalid project",
        explanation: "The project has no nuclear.toml, the manifest is invalid (config.nuc: only its sections, with named literal values), or two files declare the same name.",
    };
    pub const DEFS_NOT_FOUND: Code = Code {
        id: "N0213",
        title: "type definitions package not found",
        explanation: "A package of [dialect] defs (name@version) or one of its depends is not in the definitions repository (--defs-path, $NUCLEAR_DEFS_PATH, the defs/ folder next to each dialects/ folder), or its files are invalid. Generate it with tools/defs.",
    };
    pub const DEFS_VERSION_CONFLICT: Code = Code {
        id: "N0214",
        title: "two versions of the same type definitions package",
        explanation: "A dialect and the dialects it extends (or the depends of their packages) ask for two versions of the same definitions package. Use one version.",
    };

    pub const ALL: &[Code] = &[
        TAB_INDENT,
        BAD_DEDENT,
        ODD_INDENT,
        UNEXPECTED_CHAR,
        UNTERMINATED_STRING,
        BRACKET,
        FOREIGN_SYNTAX,
        BAD_INTERPOLATION,
        BAD_ESCAPE,
        BAD_NUMBER,
        RESERVED_NAME,
        UNEXPECTED_TOKEN,
        RANGE_NEEDS_PARENS,
        LAMBDA_NEEDS_BLOCK,
        NESTED_FUNCTION,
        STATEMENT_IN_MODULE,
        NESTED_MODULE,
        BAD_ASSIGN_TARGET,
        VOID_ONE_LINER,
        COMMENT_LINE,
        FOREIGN_HABIT,
        DUPLICATE_DECL,
        BRANCHING_VALUE,
        ENUM_SYNTAX,
        OUT_OF_SCOPE,
        UNDEFINED_NAME,
        MIXED_PATTERN,
        UNHANDLED_RESULT,
        RESULT_ERROR_TYPE,
        RESULT_NOT_HOISTABLE,
        RESULT_PATTERN,
        ALIAS_TARGET,
        UNKNOWN_HEADER_WORD,
        UNKNOWN_CLAUSE,
        MISSING_TEMPLATE,
        SHORTCUT_COLLISION,
        SIMILAR_SHORTCUT,
        BAD_DIALECT_FILE,
        DIALECT_NOT_FOUND,
        PLATFORM_CONTRACT,
        CONTRACT,
        FIELD_RULE,
        PLATFORM_DECORATOR,
        UNKNOWN_MEMBER,
        SINGLE_BASE,
        KIND_REQUIRES_MEMBER,
        UNKNOWN_TEMPLATE_VAR,
        UNKNOWN_USE_CLAUSE,
        SHORTCUT_IS_KEYWORD,
        KIND_METHOD_NOT_ALLOWED,
        KIND_RETURN_TYPE,
        KIND_METHOD_BODY,
        NOT_AWAITED,
        DECORATOR_ARGS,
        KIND_IGNORES,
        UNKNOWN_NAMED_ARG,
        UNKNOWN_CONSTRUCTOR,
        ARCH_CALLER,
        ARCH_PRIVATE,
        PROJECT,
        PLUGIN_DISABLED,
        DEFS_NOT_FOUND,
        DEFS_VERSION_CONFLICT,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Exact fix: replace the text at `span` with `replacement` (empty = delete).
#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn error(code: Code, span: Span, message: impl Into<String>) -> Self {
        Diagnostic { code, severity: Severity::Error, message: message.into(), span, fix: None }
    }

    pub fn warning(code: Code, span: Span, message: impl Into<String>) -> Self {
        Diagnostic { code, severity: Severity::Warning, message: message.into(), span, fix: None }
    }

    pub fn with_fix(mut self, span: Span, replacement: impl Into<String>) -> Self {
        self.fix = Some(Fix { span, replacement: replacement.into() });
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// Source file with its line index, to map byte offsets to line/column.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        SourceFile { name: name.into(), text, line_starts }
    }

    /// Line and column (1-based; the column counts characters, not bytes).
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let col = self.text[self.line_starts[line]..offset].chars().count() + 1;
        (line + 1, col)
    }

    /// Text of line `line` (1-based), without the line break.
    pub fn line_text(&self, line: usize) -> &str {
        let start = self.line_starts[line - 1];
        let end = self.line_starts.get(line).copied().unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\n', '\r'])
    }
}

/// Output for people.
pub fn render_text(diags: &[Diagnostic], file: &SourceFile) -> String {
    let mut out = String::new();
    for d in diags {
        let (line, col) = file.line_col(d.span.start);
        let (end_line, end_col) = file.line_col(d.span.end);
        let src_line = file.line_text(line);
        let w = line.to_string().len();
        let width = if end_line == line {
            end_col.saturating_sub(col)
        } else {
            src_line.chars().count().saturating_sub(col - 1)
        }
        .max(1);
        let kind = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let _ = writeln!(out, "{kind}[{}]: {}", d.code.id, d.message);
        let _ = writeln!(out, "{:w$}--> {}:{line}:{col}", "", file.name);
        let _ = writeln!(out, "{:w$} |", "");
        let _ = writeln!(out, "{line} | {src_line}");
        let _ = writeln!(out, "{:w$} | {}{}", "", " ".repeat(col - 1), "^".repeat(width));
        if let Some(fix) = &d.fix {
            let old = &file.text[fix.span.start..fix.span.end];
            let action = if fix.replacement.is_empty() {
                format!("delete `{old}`")
            } else {
                format!("replace `{old}` with `{}`", fix.replacement)
            };
            let _ = writeln!(out, "{:w$} = fix: {action}", "");
        }
        out.push('\n');
    }
    out
}

/// Output for agents (`nuclear check --json`).
pub fn render_json(diags: &[Diagnostic], file: &SourceFile) -> String {
    serde_json::to_string_pretty(&json_items(diags, file)).expect("serialize diagnostics")
}

/// Diagnostics as JSON values, to combine diagnostics of several files in one list.
pub fn json_items(diags: &[Diagnostic], file: &SourceFile) -> Vec<serde_json::Value> {
    diags
        .iter()
        .map(|d| {
            let (line, col) = file.line_col(d.span.start);
            let (end_line, end_col) = file.line_col(d.span.end);
            let fix = d.fix.as_ref().map(|f| {
                let (fl, fc) = file.line_col(f.span.start);
                json!({
                    "line": fl,
                    "col": fc,
                    "replace": &file.text[f.span.start..f.span.end],
                    "with": f.replacement,
                })
            });
            json!({
                "code": d.code.id,
                "severity": match d.severity { Severity::Error => "error", Severity::Warning => "warning" },
                "message": d.message,
                "file": file.name,
                "line": line,
                "col": col,
                "end_line": end_line,
                "end_col": end_col,
                "fix": fix,
                "explanation-context": format!("Line {line}, column {col}: {}", d.code.explanation),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_chars() {
        let f = SourceFile::new("a.nuc", "ab\nñx = 1\n");
        assert_eq!(f.line_col(0), (1, 1));
        assert_eq!(f.line_col(3), (2, 1));
        assert_eq!(f.line_col(5), (2, 2)); // 'ñ' takes 2 bytes
        assert_eq!(f.line_text(2), "ñx = 1");
    }

    #[test]
    fn every_code_is_documented() {
        let doc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/diagnostics.md"))
            .expect("spec/diagnostics.md");
        for code in codes::ALL {
            assert!(doc.contains(code.id), "{} is missing from spec/diagnostics.md", code.id);
        }
    }
}
