//! Function aliases: `getOne() = Module.method` declares `getOne` as the same function as `method`.
//!
//! An alias is a function or method with empty parentheses, no return type and a body that is only
//! `Module.method`, where `Module` is declared in this file and has that method. Before emitting, the
//! alias is rewritten as a function that delegates to the target, with the target's parameters,
//! return type, async flag and generator flag:
//!
//! ```text
//! getOne() = Users.find          →   getOne(id: int) User?
//!                                      Users().find(id)
//! ```
//!
//! A static target is called on the module (`Users.find(id)`); any other target on a new instance
//! (`Users()`), so the module's constructor parameters must all have a default (N0112). When the
//! declaration's kind names a function in `alias_via` (`inject`), the instance comes from it instead:
//! `inject<Users>().find(id)`. With `alias_via = "constructor"` the instance is a constructor parameter the
//! framework injects (NestJS): the alias calls `users.find(id)` (a member) and the declaration gets the parameter
//! `users: Users`, once per target module. In a project, `Module` can be declared in another file.

use crate::analysis;
use nuclear_diag::{Diagnostic, codes};
use nuclear_syntax::ast::*;
use std::collections::HashMap;

/// A module: its methods and the constructor parameters that need a value.
#[derive(Clone)]
pub(crate) struct Module {
    methods: HashMap<String, Decl>,
    required: Vec<String>,
}

/// Rewrites every alias of the file in place. `others` are the modules of the other files of the
/// project; `via` gives the `alias_via` function of a declaration's kind.
pub(crate) fn expand(
    file: &mut File,
    others: &HashMap<String, Module>,
    via: &dyn Fn(&Decl) -> Option<String>,
) -> Vec<Diagnostic> {
    let mut modules = others.clone();
    modules.extend(self::modules(file));
    let mut diags = Vec::new();
    for item in &mut file.items {
        let Item::Decl(d) = item else { continue };
        let owner_via = via(d);
        if let Body::Members(members) = &mut d.body {
            let mut injected: Vec<String> = Vec::new();
            for m in members {
                if let Member::Method(md) = m
                    && let Some(module) = rewrite(md, &modules, owner_via.as_deref(), &mut diags)
                    && !injected.contains(&module)
                {
                    injected.push(module);
                }
            }
            if owner_via.as_deref() == Some("constructor") && !injected.is_empty() {
                let params = d.params.get_or_insert_with(|| Params { span: d.name.span, list: Vec::new() });
                for module in injected {
                    let name = field_name(&module);
                    if params.list.iter().any(|p| p.name.name == name) {
                        continue;
                    }
                    let span = d.name.span;
                    params.list.push(Param {
                        span,
                        decorators: Vec::new(),
                        name: Ident { name, span },
                        optional: false,
                        ty: Some(Type { span, kind: TypeKind::Named { name: module, args: Vec::new() } }),
                        default: None,
                        named: false,
                    });
                }
            }
        } else {
            rewrite(d, &modules, None, &mut diags);
        }
    }
    diags
}

pub(crate) fn modules(file: &File) -> HashMap<String, Module> {
    let mut out = HashMap::new();
    for item in &file.items {
        let Item::Decl(d) = item else { continue };
        let Body::Members(members) = &d.body else { continue };
        if !d.name.is_pascal() {
            continue;
        }
        let mut params = d.params.as_ref().map(|p| p.list.clone()).unwrap_or_default();
        let mut methods = HashMap::new();
        for m in members {
            match m {
                Member::Method(md) => {
                    methods.insert(md.name.name.clone(), md.clone());
                }
                Member::Ctor(c) => params = c.params.list.clone(),
                _ => {}
            }
        }
        let required =
            params.iter().filter(|p| p.default.is_none() && !p.optional).map(|p| p.name.name.clone()).collect();
        out.insert(d.name.name.clone(), Module { methods, required });
    }
    out
}

/// `Module.method` written as the whole body of `name()`.
fn target(d: &Decl) -> Option<(Ident, Ident)> {
    let empty_params = d.params.as_ref().is_some_and(|p| p.list.is_empty());
    if !empty_params || d.ret.is_some() || d.is_async || !d.generics.is_empty() {
        return None;
    }
    let Body::Expr(e) = &d.body else { return None };
    let ExprKind::Member { object, name, optional: false } = &e.kind else { return None };
    let ExprKind::Name(module) = &object.kind else { return None };
    Some((Ident { name: module.clone(), span: object.span }, name.clone()))
}

/// `GetTask` → `getTask`: the constructor field of an injected module.
fn field_name(module: &str) -> String {
    let mut c = module.chars();
    c.next().map(|f| f.to_lowercase().chain(c).collect()).unwrap_or_default()
}

/// Rewrites one alias; returns the module it delegates to (for `alias_via = "constructor"`).
fn rewrite(
    d: &mut Decl,
    modules: &HashMap<String, Module>,
    via: Option<&str>,
    diags: &mut Vec<Diagnostic>,
) -> Option<String> {
    let (module, method) = target(d)?;
    let m = modules.get(&module.name)?;
    let t = m.methods.get(&method.name)?;
    let span = match &d.body {
        Body::Expr(e) => e.span,
        _ => d.span,
    };
    let is_static = t.words.iter().any(|w| w.name == "static");
    if !is_static
        && via.is_none()
        && let Some(first) = m.required.first()
    {
        diags.push(Diagnostic::error(
            codes::ALIAS_TARGET,
            span,
            format!(
                "'{}()' is an alias of {}.{}, which needs a new {}, but its constructor parameter '{first}' has no \
                 default; give it a default or write '{}' as a method",
                d.name.name, module.name, method.name, module.name, d.name.name
            ),
        ));
        return None;
    }

    let expr = |kind: ExprKind| Expr { span, kind };
    let object = if is_static {
        expr(ExprKind::Name(module.name.clone()))
    } else if via == Some("constructor") {
        expr(ExprKind::Name(field_name(&module.name)))
    } else if let Some(f) = via {
        let ty = Type { span, kind: TypeKind::Named { name: module.name.clone(), args: Vec::new() } };
        let callee = expr(ExprKind::Name(f.to_string()));
        expr(ExprKind::Call { callee: Box::new(callee), args: Vec::new(), type_args: vec![ty] })
    } else {
        let callee = expr(ExprKind::Name(module.name.clone()));
        expr(ExprKind::Call { callee: Box::new(callee), args: Vec::new(), type_args: Vec::new() })
    };
    let params: Vec<Param> = t.params.as_ref().map(|p| p.list.clone()).unwrap_or_default();
    let args = params
        .iter()
        .map(|p| Arg {
            span,
            // named parameters are passed by name; the rest by position
            name: p.named.then(|| p.name.clone()),
            value: expr(ExprKind::Name(p.name.name.clone())),
            spread: false,
            ty: None,
        })
        .collect();
    let callee = expr(ExprKind::Member {
        object: Box::new(object),
        name: Ident { name: method.name.clone(), span: method.span },
        optional: false,
    });
    let mut call = expr(ExprKind::Call { callee: Box::new(callee), args, type_args: Vec::new() });

    let (is_async, generator) = match &t.body {
        Body::Block(b) => (t.is_async || analysis::awaits_block(&b.stmts), analysis::yields_block(&b.stmts)),
        Body::Expr(e) => (t.is_async || analysis::awaits_expr(e), false),
        _ => (t.is_async, false),
    };
    if is_async && !generator {
        call = expr(ExprKind::Unary { op: UnaryOp::Await, expr: Box::new(call) });
    }
    d.params = Some(Params { span: d.params.as_ref().map_or(span, |p| p.span), list: params });
    d.ret = t.ret.clone();
    d.generics = t.generics.clone();
    d.generic_bounds = t.generic_bounds.clone();
    d.is_async = is_async;
    d.body = if generator {
        // every value of the target generator
        Body::Block(Block { span, stmts: vec![Stmt::Yield { span, value: call, all: true }] })
    } else {
        Body::Expr(call)
    };
    Some(module.name)
}
