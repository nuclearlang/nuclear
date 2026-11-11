//! Code metrics: an approximation of cyclomatic complexity per function, method, constructor, lambda,
//! enum and module, shown as code lenses ("Complexity is 5 Everything is cool!").
//!
//! A port of the VS Code extension *CodeMetrics* (kisstkondoros/codemetrics, MIT) and its core library
//! tsmetrics-core: same settings, same default costs, same messages. Every node of the AST that has a
//! cost becomes an entry with a complexity increment and a description. Entries of functions, methods,
//! constructors, lambdas, accessors, enums and modules are *visible*: they collect the entries found
//! inside them. A function shows the sum of its entries; a module shows the maximum of its members.
//!
//! The Nuclear constructs map to the TypeScript node names of the original settings: `match` is a
//! "Switch statement", each arm a "Case clause" (`_` is the "Default case"), a lambda an
//! "Arrow function", `and` / `or` a "Binary expression", `cond ? a : b` a "Conditional expression".

use crate::text::LineIndex;
use nuclear_diag::Span;
use nuclear_syntax::ast::*;
use serde_json::{Map, Value, json};

/// Settings of the "basics" group: (name, default, description). Same names and texts as the original.
/// `client` = only read by the editor (decorations, colors), the server passes them through.
pub const BASICS: &[(&str, Basic, &str)] = &[
    ("Exclude", Basic::List(&[]), "Array of exclusion patterns (in glob format)"),
    (
        "OverviewRulerModeEnabled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are shown on the overview ruler",
    ),
    (
        "DecorationModeEnabled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are shown as inline decoration",
    ),
    (
        "DecorationTemplate",
        Basic::Text(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{{size}}px' height='{{size}}px' viewbox='0 0 {{size}} {{size}}'><rect width='{{size}}' height='{{size}}' style='fill:{{color}};stroke-width:1;stroke:#fff'/></svg>",
        ),
        "A flag which indicates whether code metrics are shown as inline decoration",
    ),
    ("CodeLensEnabled", Basic::Bool(true), "A flag which indicates whether code metrics are shown as code lens"),
    ("DiagnosticsEnabled", Basic::Bool(false), "A flag which indicates whether code metrics are shown as diagnostics"),
    ("IgnoredFunctionNames", Basic::List(&[]), "A set of function names to ignore from calculations"),
    (
        "MetricsForClassDeclarationsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for class declarations",
    ),
    (
        "MetricsForConstructorsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for constructors",
    ),
    (
        "MetricsForEnumDeclarationsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for enum declarations",
    ),
    (
        "MetricsForFunctionDeclarationsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for function declarations",
    ),
    (
        "MetricsForFunctionExpressionsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for function expressions",
    ),
    (
        "MetricsForMethodDeclarationsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for methods declarations",
    ),
    (
        "MetricsForArrowFunctionsToggled",
        Basic::Bool(true),
        "A flag which indicates whether code metrics are enabled for arrow functions (can also be toggled via a command)",
    ),
    (
        "FileSizeLimitMB",
        Basic::Number(0.5),
        "File size limit in megabytes, above this codemetrics are not going to be computed, use a negative number to allow any size",
    ),
    ("ComplexityColorLow", Basic::Text("#4bb14f"), "Color for the low complexity level"),
    ("ComplexityColorNormal", Basic::Text("#ffc208"), "Color for the normal complexity level"),
    ("ComplexityColorHigh", Basic::Text("#f44034"), "Color for the high complexity level"),
    ("ComplexityColorExtreme", Basic::Text("#ff0000"), "Color for the extreme complexity level"),
    ("ComplexityLevelExtreme", Basic::Number(25.0), "Threshold to determine extreme high complexity level"),
    ("ComplexityLevelHigh", Basic::Number(10.0), "Threshold to determine high complexity level"),
    ("ComplexityLevelNormal", Basic::Number(5.0), "Threshold to determine normal complexity level"),
    ("ComplexityLevelLow", Basic::Number(0.0), "Threshold to determine low complexity level"),
    ("CodeLensHiddenUnder", Basic::Number(3.0), "Threshold to show code lens for code metrics"),
    (
        "ComplexityLevelExtremeDescription",
        Basic::Text("Bloody hell..."),
        "Code lens message for extreme high complexity level",
    ),
    (
        "ComplexityLevelHighDescription",
        Basic::Text("You must be kidding"),
        "Code lens message for high complexity level",
    ),
    (
        "ComplexityLevelNormalDescription",
        Basic::Text("It's time to do something..."),
        "Code lens message for normal complexity level",
    ),
    ("ComplexityLevelLowDescription", Basic::Text("Everything is cool!"), "Code lens message for low complexity level"),
    (
        "ComplexityTemplate",
        Basic::Text("Complexity is {0} {1}"),
        "Code lens message template {0} stands for the actual value {1} is for the complexity level description",
    ),
];

#[derive(Debug, Clone, Copy)]
pub enum Basic {
    Bool(bool),
    Number(f64),
    Text(&'static str),
    List(&'static [&'static str]),
}

impl Basic {
    pub fn to_json(self) -> Value {
        match self {
            Basic::Bool(b) => json!(b),
            Basic::Number(n) => json!(n),
            Basic::Text(t) => json!(t),
            Basic::List(l) => json!(l),
        }
    }
}

/// The "nodeconfiguration" group: (node, default cost, default description, the Nuclear construct).
/// Node names, costs and descriptions are the original ones; only the nodes Nuclear has are listed.
pub const NODES: &[(&str, f64, &str, &str)] = &[
    ("ArrayLiteralExpression", 0.0, "Array literal expression", "list literal `[a, b]`"),
    ("ArrowFunction", 1.0, "Arrow function", "lambda `(x) => ...`"),
    ("BinaryExpression", 1.0, "Binary expression", "`and` / `or`"),
    ("BreakStatement", 1.0, "Break statement", "`break`"),
    ("CallExpression", 0.0, "Call expression", "call `f(x)`"),
    ("CaseClause", 1.0, "Case clause", "arm of a `match`"),
    ("CatchClause", 1.0, "Catch clause", "`catch`"),
    ("ClassDeclaration", 0.0, "Class declaration", "module or dialect kind (`User`, `stateless Home`)"),
    ("ConditionalExpression", 1.0, "Conditional expression", "`cond ? a : b`"),
    ("Constructor", 1.0, "Constructor", "`(params)` line of a module"),
    ("ContinueStatement", 1.0, "Continue statement", "`continue`"),
    ("DefaultClause", 1.0, "Default case", "`_` arm of a `match`"),
    ("ElementAccessExpression", 0.0, "Element access expression", "`list[i]`"),
    ("EnumDeclaration", 1.0, "Enum declaration", "enum"),
    ("ForStatement", 1.0, "For statement", "`for x in xs`"),
    ("FunctionDeclaration", 1.0, "Function declaration", "top-level function"),
    ("GetAccessor", 0.0, "Get accessor", "`get name`"),
    ("IfStatement", 1.0, "If statement", "`if` / `else if` without `else`, `if` inside a list"),
    ("IfWithElseStatement", 1.0, "If with else statement", "`if` / `else if` followed by another branch"),
    ("ImportDeclaration", 0.0, "Import declaration", "`use`"),
    ("InterfaceDeclaration", 0.0, "Interface declaration", "module with signatures only"),
    ("MethodDeclaration", 1.0, "Method declaration", "method of a module"),
    ("MethodSignature", 0.0, "Method signature", "method without body"),
    ("NewExpression", 0.0, "New expression", "`User(...)` (PascalCase call)"),
    ("ObjectLiteralExpression", 0.0, "Object literal expression", "map literal `{ a: 1 }`"),
    ("Parameter", 0.0, "Parameter", "parameter"),
    ("PrefixUnaryExpression", 0.0, "Prefix unary expression", "`not x`, `-x`, `~call()`"),
    ("PropertyAccessExpression", 0.0, "Property access expression", "`a.b`"),
    ("PropertyDeclaration", 0.0, "Property declaration", "field of a module"),
    ("ReturnStatement", 1.0, "Return statement", "`return`"),
    ("SetAccessor", 0.0, "Set accessor", "`set name`"),
    ("StringLiteral", 0.0, "String literal", "text without interpolation"),
    ("SwitchStatement", 1.0, "Switch statement", "`match`"),
    ("TemplateExpression", 0.0, "Template expression", "text with `${...}`"),
    ("ThrowStatement", 1.0, "Throw statement", "`throw`"),
    ("TryStatement", 1.0, "Try statement", "`try`"),
    ("TypeAssertionExpression", 0.0, "Type assertion expression", "`x as T`"),
    ("WhileStatement", 1.0, "While statement", "`while`"),
];

/// Every setting with its default and description, for the editors (`nuclear-lsp --codemetrics-defaults`).
pub fn defaults_json() -> Value {
    let basics: Map<String, Value> =
        BASICS.iter().map(|(k, v, d)| (k.to_string(), json!({ "default": v.to_json(), "description": d }))).collect();
    let nodes: Map<String, Value> = NODES
        .iter()
        .map(|(k, cost, desc, nuclear)| {
            (k.to_string(), json!({ "default": cost, "description": desc, "nuclear": nuclear }))
        })
        .collect();
    json!({ "basics": basics, "nodeconfiguration": nodes })
}

/// Effective settings: the defaults with the editor's `nuclear.codemetrics` object on top.
#[derive(Debug, Clone)]
pub struct Config {
    basics: Map<String, Value>,
    costs: Map<String, Value>,
    /// `Toggle code metrics` command (not a setting: lasts until the server restarts, like the original).
    pub displayed: bool,
    /// `Toggle code lenses for arrow functions` command; `None` = use the setting.
    pub arrow_functions: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            basics: BASICS.iter().map(|(k, v, _)| (k.to_string(), v.to_json())).collect(),
            costs: NODES
                .iter()
                .flat_map(|(k, cost, desc, _)| [(k.to_string(), json!(cost)), (format!("{k}Description"), json!(desc))])
                .collect(),
            displayed: true,
            arrow_functions: None,
        }
    }
}

impl Config {
    /// Applies `{ "basics": {...}, "nodeconfiguration": {...} }` over the defaults. Unknown keys and values of
    /// the wrong type are ignored.
    pub fn apply(&mut self, settings: &Value) {
        let fresh = Config::default();
        self.basics = fresh.basics;
        self.costs = fresh.costs;
        for (group, target) in [("basics", &mut self.basics), ("nodeconfiguration", &mut self.costs)] {
            for (k, v) in settings[group].as_object().into_iter().flatten() {
                if let Some(old) = target.get(k)
                    && same_type(old, v)
                {
                    target.insert(k.clone(), v.clone());
                }
            }
        }
    }

    pub fn flag(&self, name: &str) -> bool {
        if name == "MetricsForArrowFunctionsToggled"
            && let Some(on) = self.arrow_functions
        {
            return on;
        }
        self.basics.get(name).and_then(Value::as_bool).unwrap_or(false)
    }

    pub fn number(&self, name: &str) -> f64 {
        self.basics.get(name).and_then(Value::as_f64).unwrap_or(0.0)
    }

    pub fn text(&self, name: &str) -> &str {
        self.basics.get(name).and_then(Value::as_str).unwrap_or_default()
    }

    fn list(&self, name: &str) -> Vec<&str> {
        self.basics.get(name).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect()
    }

    fn cost(&self, node: &str) -> f64 {
        self.costs.get(node).and_then(Value::as_f64).unwrap_or(0.0)
    }

    fn description(&self, node: &str) -> String {
        self.costs.get(&format!("{node}Description")).and_then(Value::as_str).unwrap_or(node).to_string()
    }

    /// 0 low, 1 normal, 2 high, 3 extreme: the color of the decorations (same boundaries as the original).
    pub fn level(&self, complexity: f64) -> u8 {
        if complexity > self.number("ComplexityLevelExtreme") {
            3
        } else if complexity > self.number("ComplexityLevelHigh") {
            2
        } else if complexity > self.number("ComplexityLevelNormal") {
            1
        } else {
            0
        }
    }

    /// The code lens text: `Complexity is 12 You must be kidding`.
    pub fn message(&self, complexity: f64) -> String {
        let description = if complexity > self.number("ComplexityLevelExtreme") {
            self.text("ComplexityLevelExtremeDescription")
        } else if complexity > self.number("ComplexityLevelHigh") {
            self.text("ComplexityLevelHighDescription")
        } else if complexity > self.number("ComplexityLevelNormal") {
            self.text("ComplexityLevelNormalDescription")
        } else if complexity > self.number("ComplexityLevelLow") {
            self.text("ComplexityLevelLowDescription")
        } else {
            ""
        };
        let template = self.text("ComplexityTemplate");
        let template = if template.trim().is_empty() { "Complexity is {0} {1}" } else { template };
        template.replacen("{0}", &round(complexity), 1).replacen("{1}", description, 1)
    }

    /// Whether metrics are computed for this document at all (exclusion patterns and size limit).
    pub fn applies_to(&self, uri: &str, text: &str) -> bool {
        let limit = self.number("FileSizeLimitMB");
        if limit >= 0.0 && text.len() as f64 > limit * 1024.0 * 1024.0 {
            return false;
        }
        let path = crate::server::uri_to_path(uri).map(|p| p.to_string_lossy().replace('\\', "/"));
        !self.list("Exclude").iter().any(|p| glob(p, uri) || path.as_deref().is_some_and(|path| glob(p, path)))
    }
}

fn same_type(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
            | (Value::Array(_), Value::Array(_))
    )
}

/// `1`, `1.5`, `0.33`: like JavaScript's `Number(x.toFixed(2))`.
pub fn round(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0;
    let s = format!("{r:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ------------------------------------------------------------------ model

/// One entry: a node with its own complexity increment and, for visible entries, the entries inside it.
#[derive(Debug, Clone)]
pub struct Model {
    pub span: Span,
    /// First line of the node's source (at most 70 characters).
    pub text: String,
    /// 1-based, like the original.
    pub line: u32,
    pub column: u32,
    pub complexity: f64,
    pub description: String,
    pub visible: bool,
    /// Modules show the maximum of their members instead of the sum.
    pub max: bool,
    pub children: Vec<Model>,
}

impl Model {
    pub fn collected(&self) -> f64 {
        if self.max {
            self.children.iter().map(Model::collected).fold(0.0, f64::max) + self.complexity
        } else {
            self.children.iter().map(Model::collected).sum::<f64>() + self.complexity
        }
    }

    /// The rows of the details list: the entry itself (its own increment) and every entry inside it with a
    /// complexity above zero (their collected complexity). Each row: label, code and 0-based position.
    pub fn details(&self, src: &str, lines: &LineIndex) -> Vec<Value> {
        std::iter::once((self, self.complexity))
            .chain(self.children.iter().map(|c| (c, c.collected())).filter(|(_, c)| *c > 0.0))
            .map(|(m, complexity)| {
                let (line, character) = lines.position(src, m.span.start);
                json!({
                    "label": format!("{} - Ln {} Col {} {}", round(complexity), m.line, m.column, m.description)
                        .replace(['\r', '\n'], " "),
                    "description": m.text,
                    "line": line,
                    "character": character,
                })
            })
            .collect()
    }
}

/// Metrics of a whole file: the root collector (invisible) with every entry inside.
pub fn compute(src: &str, lines: &LineIndex, file: &File, cfg: &Config) -> Model {
    let mut root = Model {
        span: Span::new(0, src.len()),
        text: String::new(),
        line: 1,
        column: 1,
        complexity: 0.0,
        description: "Collector".into(),
        visible: false,
        max: false,
        children: Vec::new(),
    };
    let w = Walker { src, lines, cfg, ignored: cfg.list("IgnoredFunctionNames") };
    for item in &file.items {
        match item {
            Item::Use(u) => {
                w.node(&mut root, u.span, "ImportDeclaration", None);
            }
            Item::Decl(d) => w.decl(&mut root, d, true),
            Item::Stmt(s) => w.stmt(&mut root, s),
        }
    }
    root
}

/// Visible entries with a complexity of at least `CodeLensHiddenUnder`, in source order.
pub fn shown<'a>(root: &'a Model, cfg: &Config) -> Vec<&'a Model> {
    fn collect<'a>(m: &'a Model, min: f64, out: &mut Vec<&'a Model>) {
        if m.visible && m.collected() >= min {
            out.push(m);
        }
        for c in &m.children {
            collect(c, min, out);
        }
    }
    let mut out = Vec::new();
    collect(root, cfg.number("CodeLensHiddenUnder"), &mut out);
    out
}

struct Walker<'a> {
    src: &'a str,
    lines: &'a LineIndex,
    cfg: &'a Config,
    ignored: Vec<&'a str>,
}

impl Walker<'_> {
    /// Adds an entry for `span` under `parent`. Returns the parent the node's children go to: the new entry
    /// when it is visible (`toggle` = the flag that makes it visible), else `parent` itself.
    fn node<'p>(&self, parent: &'p mut Model, span: Span, node: &str, toggle: Option<&str>) -> &'p mut Model {
        self.node_with(parent, span, node, toggle, false, false)
    }

    fn node_with<'p>(
        &self,
        parent: &'p mut Model,
        span: Span,
        node: &str,
        toggle: Option<&str>,
        max: bool,
        ignore: bool,
    ) -> &'p mut Model {
        let (line, col) = self.lines.position(self.src, span.start);
        let visible = toggle.is_some_and(|t| self.cfg.flag(t));
        parent.children.push(Model {
            span,
            text: first_line(self.src, span),
            line: line + 1,
            column: col + 1,
            complexity: if ignore { 0.0 } else { self.cfg.cost(node) },
            description: self.cfg.description(node),
            visible,
            max,
            children: Vec::new(),
        });
        if visible { parent.children.last_mut().expect("just pushed") } else { parent }
    }

    // ------------------------------------------------------------ declarations

    fn decl(&self, parent: &mut Model, d: &Decl, toplevel: bool) {
        for dec in &d.decorators {
            self.args(parent, dec.args.iter().flatten());
        }
        if d.name.is_pascal() {
            return self.module(parent, d);
        }
        let has = |w: &str| d.words.iter().any(|x| x.name == w);
        let (node, toggle) = if matches!(d.body, Body::None) && !toplevel {
            ("MethodSignature", None)
        } else if has("get") {
            ("GetAccessor", Some("MetricsForFunctionExpressionsToggled"))
        } else if has("set") {
            ("SetAccessor", Some("MetricsForFunctionExpressionsToggled"))
        } else if toplevel {
            ("FunctionDeclaration", Some("MetricsForFunctionDeclarationsToggled"))
        } else {
            ("MethodDeclaration", Some("MetricsForMethodDeclarationsToggled"))
        };
        let inner = self.node(parent, d.span, node, toggle);
        if let Some(p) = &d.params {
            self.params(inner, &p.list);
        }
        match &d.body {
            Body::Block(b) => self.block(inner, b),
            Body::Expr(e) => self.expr(inner, e),
            Body::Members(members) => self.members(inner, members),
            Body::None => {}
        }
    }

    /// PascalCase declaration: module (class), dialect kind, enum or interface.
    fn module(&self, parent: &mut Model, d: &Decl) {
        let members: &[Member] = match &d.body {
            Body::Members(m) => m,
            _ => &[],
        };
        let enum_ = !members.is_empty() && members.iter().all(|m| matches!(m, Member::EnumItem(_)));
        // the transpiler's rule: no constructor parameters, fields without value and methods without body
        let interface = !enum_
            && d.params.is_none()
            && !members.is_empty()
            && !d.words.iter().any(|w| w.name == "abstract")
            && members.iter().all(|m| match m {
                Member::Field(f) => f.default.is_none(),
                Member::Method(md) => matches!(md.body, Body::None),
                _ => false,
            });
        let inner = if enum_ {
            self.node(parent, d.span, "EnumDeclaration", Some("MetricsForEnumDeclarationsToggled"))
        } else if interface {
            self.node(parent, d.span, "InterfaceDeclaration", None)
        } else {
            self.node_with(parent, d.span, "ClassDeclaration", Some("MetricsForClassDeclarationsToggled"), true, false)
        };
        if let Some(p) = &d.params {
            self.params(inner, &p.list);
        }
        match &d.body {
            Body::Members(m) => self.members(inner, m),
            Body::Block(b) => self.block(inner, b),
            Body::Expr(e) => self.expr(inner, e),
            Body::None => {}
        }
    }

    fn members(&self, parent: &mut Model, members: &[Member]) {
        for m in members {
            match m {
                Member::Field(f) => {
                    for dec in &f.decorators {
                        self.args(parent, dec.args.iter().flatten());
                    }
                    let inner = self.node(parent, f.span, "PropertyDeclaration", None);
                    if let Some(v) = &f.default {
                        self.expr(inner, v);
                    }
                }
                Member::Method(d) => self.decl(parent, d, false),
                Member::Ctor(c) => {
                    let inner = self.node(parent, c.span, "Constructor", Some("MetricsForConstructorsToggled"));
                    self.params(inner, &c.params.list);
                    if let Some(b) = &c.body {
                        self.block(inner, b);
                    }
                }
                Member::EnumItem(e) => {
                    for a in &e.args {
                        self.expr(parent, a);
                    }
                }
            }
        }
    }

    fn params(&self, parent: &mut Model, params: &[Param]) {
        for p in params {
            for dec in &p.decorators {
                self.args(parent, dec.args.iter().flatten());
            }
            let inner = self.node(parent, p.span, "Parameter", None);
            if let Some(v) = &p.default {
                self.expr(inner, v);
            }
        }
    }

    // ------------------------------------------------------------ statements

    fn block(&self, parent: &mut Model, b: &Block) {
        for s in &b.stmts {
            self.stmt(parent, s);
        }
    }

    fn stmt(&self, parent: &mut Model, s: &Stmt) {
        match s {
            Stmt::Expr(e) => self.expr(parent, e),
            Stmt::Assign(a) => {
                self.expr(parent, &a.target);
                self.expr(parent, &a.value);
            }
            Stmt::If(i) => self.if_(parent, i, false),
            Stmt::For(f) => {
                let inner = self.node(parent, f.span, "ForStatement", None);
                self.expr(inner, &f.iter);
                self.block(inner, &f.body);
            }
            Stmt::While { span, cond, body } => {
                let inner = self.node(parent, *span, "WhileStatement", None);
                self.expr(inner, cond);
                self.block(inner, body);
            }
            Stmt::Match(m) => self.match_(parent, m, false),
            Stmt::Try(t) => {
                let inner = self.node(parent, t.span, "TryStatement", None);
                self.block(inner, &t.body);
                if let Some((var, body)) = &t.catch {
                    let start = var.as_ref().map_or(body.span.start, |v| v.span.start);
                    let span = Span::new(catch_keyword(self.src, start), body.span.end);
                    let inner = self.node(inner, span, "CatchClause", None);
                    self.block(inner, body);
                }
                if let Some(f) = &t.finally {
                    self.block(inner, f);
                }
            }
            Stmt::Return { span, value } => {
                let inner = self.node(parent, *span, "ReturnStatement", None);
                if let Some(v) = value {
                    self.expr(inner, v);
                }
            }
            Stmt::Throw { span, value } => {
                let inner = self.node(parent, *span, "ThrowStatement", None);
                self.expr(inner, value);
            }
            Stmt::Break(span) => {
                self.node(parent, *span, "BreakStatement", None);
            }
            Stmt::Continue(span) => {
                self.node(parent, *span, "ContinueStatement", None);
            }
            Stmt::Yield { value, .. } => self.expr(parent, value),
        }
    }

    /// `if a ... else if b ... else ...` counts like TypeScript's nested ifs: each branch is an "If with else
    /// statement" when another branch follows it, else an "If statement".
    /// `ignore` (argument of an ignored function) zeroes the first branch, the entry of the whole `if`.
    fn if_(&self, parent: &mut Model, i: &If, ignore: bool) {
        let n = i.branches.len();
        let mut start = i.span.start;
        for (k, (cond, body)) in i.branches.iter().enumerate() {
            let has_else = k + 1 < n || i.otherwise.is_some();
            let node = if has_else { "IfWithElseStatement" } else { "IfStatement" };
            let span = Span::new(start.min(cond.span.start), i.span.end);
            let inner = self.node_with(parent, span, node, None, false, ignore && k == 0);
            self.expr(inner, cond);
            self.block(inner, body);
            if let Some((next, _)) = i.branches.get(k + 1) {
                start = else_keyword(self.src, body.span.end, next.span.start);
            }
        }
        if let Some(b) = &i.otherwise {
            self.block(parent, b);
        }
    }

    fn match_(&self, parent: &mut Model, m: &Match, ignore: bool) {
        let inner = self.node_with(parent, m.span, "SwitchStatement", None, false, ignore);
        self.expr(inner, &m.subject);
        for arm in &m.arms {
            let default = arm.patterns.iter().any(|p| matches!(p, Pattern::Wildcard(_)));
            let arm_model = self.node(inner, arm.span, if default { "DefaultClause" } else { "CaseClause" }, None);
            for p in &arm.patterns {
                if let Pattern::Value(e) = p {
                    self.expr(arm_model, e);
                }
            }
            match &arm.body {
                ArmBody::Expr(e) => self.expr(arm_model, e),
                ArmBody::Block(b) => self.block(arm_model, b),
            }
        }
    }

    // ------------------------------------------------------------ expressions

    fn args<'e>(&self, parent: &mut Model, args: impl Iterator<Item = &'e Arg>) {
        for a in args {
            self.expr(parent, &a.value);
        }
    }

    fn expr(&self, parent: &mut Model, e: &Expr) {
        self.expr_with(parent, e, false);
    }

    /// `ignore`: the expression is an argument of a function listed in `IgnoredFunctionNames`; its own entry
    /// counts 0 (the entries inside it still count, as in the original).
    fn expr_with(&self, parent: &mut Model, e: &Expr, ignore: bool) {
        match &e.kind {
            ExprKind::Name(_)
            | ExprKind::SelfField(_)
            | ExprKind::This
            | ExprKind::Super
            | ExprKind::Number(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::CascadeSelf => {}
            ExprKind::Str(pieces) => {
                let interp = pieces.iter().any(|p| matches!(p, StrPiece::Interp(_)));
                let inner = self.node_with(
                    parent,
                    e.span,
                    if interp { "TemplateExpression" } else { "StringLiteral" },
                    None,
                    false,
                    ignore,
                );
                for p in pieces {
                    if let StrPiece::Interp(x) = p {
                        self.expr(inner, x);
                    }
                }
            }
            ExprKind::List(items) => {
                let inner = self.node_with(parent, e.span, "ArrayLiteralExpression", None, false, ignore);
                for x in items {
                    self.expr(inner, x);
                }
            }
            ExprKind::Object(entries) => {
                let inner = self.node_with(parent, e.span, "ObjectLiteralExpression", None, false, ignore);
                for entry in entries {
                    match entry {
                        ObjEntry::Pair { value, .. } | ObjEntry::Spread(value) => self.expr(inner, value),
                        ObjEntry::Shorthand(_) => {}
                    }
                }
            }
            ExprKind::Unary { expr, .. } => {
                let inner = self.node_with(parent, e.span, "PrefixUnaryExpression", None, false, ignore);
                self.expr(inner, expr);
            }
            ExprKind::Binary { op, left, right } => {
                let inner = if matches!(op, BinaryOp::And | BinaryOp::Or) {
                    self.node_with(parent, e.span, "BinaryExpression", None, false, ignore)
                } else {
                    parent
                };
                self.expr(inner, left);
                self.expr(inner, right);
            }
            ExprKind::Ternary { cond, then, otherwise } => {
                let inner = self.node_with(parent, e.span, "ConditionalExpression", None, false, ignore);
                self.expr(inner, cond);
                self.expr(inner, then);
                self.expr(inner, otherwise);
            }
            ExprKind::Is { expr, .. } | ExprKind::NonNull(expr) | ExprKind::Spread(expr) => self.expr(parent, expr),
            ExprKind::Cast { expr, .. } => {
                let inner = self.node_with(parent, e.span, "TypeAssertionExpression", None, false, ignore);
                self.expr(inner, expr);
            }
            ExprKind::Member { object, .. } => {
                let inner = self.node_with(parent, e.span, "PropertyAccessExpression", None, false, ignore);
                self.expr(inner, object);
            }
            ExprKind::Index { object, index } => {
                let inner = self.node_with(parent, e.span, "ElementAccessExpression", None, false, ignore);
                self.expr(inner, object);
                self.expr(inner, index);
            }
            ExprKind::Call { callee, args, .. } => {
                let construct = matches!(&callee.kind, ExprKind::Name(n) if is_pascal(n));
                let name = if construct { "NewExpression" } else { "CallExpression" };
                let inner = self.node_with(parent, e.span, name, None, false, ignore);
                // like the original: only a plain name (`describe(...)`), not `a.describe(...)`
                let skip = matches!(&callee.kind, ExprKind::Name(n) if self.ignored.contains(&n.as_str()));
                self.expr(inner, callee);
                for a in args {
                    self.expr_with(inner, &a.value, skip);
                }
            }
            ExprKind::ListIf { cond, value } => {
                let inner = self.node_with(parent, e.span, "IfStatement", None, false, ignore);
                self.expr(inner, cond);
                self.expr(inner, value);
            }
            ExprKind::Cascade { target, sections } => {
                self.expr(parent, target);
                for s in sections {
                    self.expr(parent, &s.access);
                    if let Some((_, v)) = &s.assign {
                        self.expr(parent, v);
                    }
                }
            }
            ExprKind::Lambda(l) => {
                let inner = self.node_with(
                    parent,
                    e.span,
                    "ArrowFunction",
                    Some("MetricsForArrowFunctionsToggled"),
                    false,
                    ignore,
                );
                self.params(inner, &l.params);
                match &l.body {
                    LambdaBody::Expr(x) => self.expr(inner, x),
                    LambdaBody::Block(b) => self.block(inner, b),
                }
            }
            ExprKind::If(i) => self.if_(parent, i, ignore),
            ExprKind::Match(m) => self.match_(parent, m, ignore),
        }
    }
}

/// First line of the node's source, trimmed, cut at 70 characters (`...` when cut).
fn first_line(src: &str, span: Span) -> String {
    let text = src.get(span.start..span.end.max(span.start)).unwrap_or_default();
    let line = text.lines().next().unwrap_or_default().trim();
    if line.chars().count() > 70 {
        format!("{}...", line.chars().take(70).collect::<String>().trim())
    } else {
        line.to_string()
    }
}

/// Start of the `catch` keyword before the catch variable or block (the AST keeps no span for it).
fn catch_keyword(src: &str, before: usize) -> usize {
    src.get(..before).and_then(|s| s.rfind("catch")).unwrap_or(before)
}

/// Start of the `else` of `else if` between two branches.
fn else_keyword(src: &str, from: usize, to: usize) -> usize {
    src.get(from..to).and_then(|s| s.find("else")).map_or(to, |i| from + i)
}

/// Minimal glob: `**` (any path), `*` (anything but `/`), `?` (one character), `{a,b}` (alternatives).
pub fn glob(pattern: &str, text: &str) -> bool {
    if let Some(open) = pattern.find('{')
        && let Some(close) = pattern[open..].find('}').map(|i| open + i)
    {
        let (head, tail) = (&pattern[..open], &pattern[close + 1..]);
        return pattern[open + 1..close].split(',').any(|alt| glob(&format!("{head}{alt}{tail}"), text));
    }
    fn go(p: &[char], t: &[char]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some('*') if p.get(1) == Some(&'*') => {
                let rest = if p.get(2) == Some(&'/') { &p[3..] } else { &p[2..] };
                (0..=t.len()).any(|i| go(rest, &t[i..])) || (p.get(2) == Some(&'/') && go(&p[2..], t))
            }
            Some('*') => (0..=t.len()).take_while(|&i| i == 0 || t[i - 1] != '/').any(|i| go(&p[1..], &t[i..])),
            Some('?') => !t.is_empty() && t[0] != '/' && go(&p[1..], &t[1..]),
            Some(c) => t.first() == Some(c) && go(&p[1..], &t[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&p, &t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(src: &str) -> (Model, Config) {
        let cfg = Config::default();
        let parsed = nuclear_syntax::parse(src);
        let lines = LineIndex::new(src);
        (compute(src, &lines, &parsed.file, &cfg), cfg)
    }

    #[test]
    fn rounding_like_javascript() {
        assert_eq!(round(1.0), "1");
        assert_eq!(round(1.5), "1.5");
        assert_eq!(round(1.0 / 3.0), "0.33");
        assert_eq!(round(0.0), "0");
    }

    #[test]
    fn messages_and_levels() {
        let cfg = Config::default();
        assert_eq!(cfg.message(0.0), "Complexity is 0 ");
        assert_eq!(cfg.message(3.0), "Complexity is 3 Everything is cool!");
        assert_eq!(cfg.message(5.0), "Complexity is 5 Everything is cool!");
        assert_eq!(cfg.message(6.0), "Complexity is 6 It's time to do something...");
        assert_eq!(cfg.message(11.0), "Complexity is 11 You must be kidding");
        assert_eq!(cfg.message(26.0), "Complexity is 26 Bloody hell...");
        assert_eq!((cfg.level(5.0), cfg.level(6.0), cfg.level(11.0), cfg.level(26.0)), (0, 1, 2, 3));
    }

    #[test]
    fn function_sums_its_branches() {
        let src = "\
label(n: int) string
  if n < 0 and n > -10
    return 'small'
  else if n == 0
    return 'zero'
  else
    return 'big'
";
        let (root, cfg) = metrics(src);
        let f = &root.children[0];
        assert_eq!(f.description, "Function declaration");
        // function 1 + if-with-else 1 + `and` 1 + return 1 + else-if-with-else 1 + return 1 + return 1
        assert_eq!(f.collected(), 7.0);
        assert_eq!(cfg.message(f.collected()), "Complexity is 7 It's time to do something...");
        let lines = LineIndex::new(src);
        let details = f.details(src, &lines);
        let labels: Vec<&str> = details.iter().map(|d| d["label"].as_str().unwrap()).collect();
        assert_eq!(
            labels,
            [
                "1 - Ln 1 Col 1 Function declaration",
                "1 - Ln 2 Col 3 If with else statement",
                "1 - Ln 2 Col 6 Binary expression",
                "1 - Ln 3 Col 5 Return statement",
                "1 - Ln 4 Col 3 If with else statement",
                "1 - Ln 5 Col 5 Return statement",
                "1 - Ln 7 Col 5 Return statement",
            ]
        );
        assert_eq!(details[1]["description"], "if n < 0 and n > -10");
        assert_eq!((details[1]["line"].as_u64(), details[1]["character"].as_u64()), (Some(1), Some(2)));
    }

    #[test]
    fn module_shows_the_maximum_and_lambdas_collect() {
        let src = "\
Counter
  count: int = 0
  items: int[] = []

  add(n: int)
    if n > 0
      $count += n

  reset()
    items.forEach((x) => x > 0 ? print(1) : print(2))
    for x in items
      while x > 0
        break
";
        let (root, cfg) = metrics(src);
        let module = &root.children[0];
        assert_eq!(module.description, "Class declaration");
        let shown: Vec<(String, String)> =
            shown(&root, &cfg).iter().map(|m| (m.description.clone(), round(m.collected()))).collect();
        // add = 1 + if 1; reset = 1 + lambda (1 + ternary 1) + for 1 + while 1 + break 1 = 6
        assert_eq!(
            shown,
            [("Class declaration".into(), "6".into()), ("Method declaration".into(), "6".into())],
            "add (2) and the lambda (2) are under CodeLensHiddenUnder (3)"
        );
    }

    #[test]
    fn match_counts_switch_cases_and_default() {
        let src = "\
name(n: int) string
  match n
    1 -> 'one'
    2 -> 'two'
    _ -> 'many'
";
        let (root, _) = metrics(src);
        // function 1 + switch 1 + 2 cases + default 1
        assert_eq!(root.children[0].collected(), 5.0);
    }

    #[test]
    fn settings_override_defaults() {
        let mut cfg = Config::default();
        cfg.apply(&json!({
            "basics": { "ComplexityLevelLowDescription": "OK", "CodeLensHiddenUnder": "wrong type" },
            "nodeconfiguration": { "ReturnStatement": 0, "ReturnStatementDescription": "Return" }
        }));
        assert_eq!(cfg.text("ComplexityLevelLowDescription"), "OK");
        assert_eq!(cfg.number("CodeLensHiddenUnder"), 3.0);
        assert_eq!(cfg.cost("ReturnStatement"), 0.0);
        assert_eq!(cfg.description("ReturnStatement"), "Return");
    }

    #[test]
    fn ignored_functions_and_toggles() {
        let src = "\
describe('a', () =>
  if x
    y()
)
";
        let mut cfg = Config::default();
        cfg.apply(&json!({ "basics": { "IgnoredFunctionNames": ["describe"] } }));
        let parsed = nuclear_syntax::parse(src);
        let lines = LineIndex::new(src);
        let root = compute(src, &lines, &parsed.file, &cfg);
        let shown_now = shown(&root, &cfg);
        assert!(shown_now.is_empty(), "lambda = 0 + if 1 is under the threshold");
        cfg.arrow_functions = Some(false);
        let root = compute(src, &lines, &parsed.file, &cfg);
        assert!(!root.children.iter().flat_map(|c| &c.children).any(|m| m.visible));
    }

    #[test]
    fn exclusion_globs() {
        assert!(glob("**/generated/**", "/home/a/generated/x.nuc"));
        assert!(glob("**/*.test.nuc", "/a/b/c.test.nuc"));
        assert!(!glob("*.nuc", "/a/b.nuc"));
        assert!(glob("**/{gen,out}/*.nuc", "/a/out/b.nuc"));
        let mut cfg = Config::default();
        cfg.apply(&json!({ "basics": { "Exclude": ["**/gen/**"] } }));
        assert!(!cfg.applies_to("file:///p/gen/a.nuc", ""));
        assert!(cfg.applies_to("file:///p/lib/a.nuc", ""));
    }
}
