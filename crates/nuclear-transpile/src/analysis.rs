//! Read-only passes over a function body: which variables are reassigned and whether it awaits.

use nuclear_syntax::ast::*;
use std::collections::HashSet;

/// Calls `f` on every expression inside `e` (including `e`).
pub fn walk_expr<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    f(e);
    match &e.kind {
        ExprKind::Str(pieces) => {
            for p in pieces {
                if let StrPiece::Interp(x) = p {
                    walk_expr(x, f);
                }
            }
        }
        ExprKind::List(items) => items.iter().for_each(|x| walk_expr(x, f)),
        ExprKind::Object(entries) => {
            for en in entries {
                match en {
                    ObjEntry::Pair { value, .. } => walk_expr(value, f),
                    ObjEntry::Spread(x) => walk_expr(x, f),
                    ObjEntry::Shorthand(_) => {}
                }
            }
        }
        ExprKind::Unary { expr, .. } => walk_expr(expr, f),
        ExprKind::Binary { left, right, .. } => {
            walk_expr(left, f);
            walk_expr(right, f);
        }
        ExprKind::Ternary { cond, then, otherwise } => {
            walk_expr(cond, f);
            walk_expr(then, f);
            walk_expr(otherwise, f);
        }
        ExprKind::Is { expr, .. } => walk_expr(expr, f),
        ExprKind::Member { object, .. } => walk_expr(object, f),
        ExprKind::Index { object, index } => {
            walk_expr(object, f);
            walk_expr(index, f);
        }
        ExprKind::Spread(x) | ExprKind::NonNull(x) | ExprKind::Cast { expr: x, .. } => walk_expr(x, f),
        ExprKind::ListIf { cond, value } => {
            walk_expr(cond, f);
            walk_expr(value, f);
        }
        ExprKind::Cascade { target, sections } => {
            walk_expr(target, f);
            for sec in sections {
                walk_expr(&sec.access, f);
                if let Some((_, v)) = &sec.assign {
                    walk_expr(v, f);
                }
            }
        }
        ExprKind::Call { callee, args, .. } => {
            walk_expr(callee, f);
            args.iter().for_each(|a| walk_expr(&a.value, f));
        }
        ExprKind::Lambda(l) => match &l.body {
            LambdaBody::Expr(x) => walk_expr(x, f),
            LambdaBody::Block(b) => b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f)),
        },
        ExprKind::If(i) => walk_if(i, f),
        ExprKind::Match(m) => walk_match(m, f),
        _ => {}
    }
}

fn walk_if<'a>(i: &'a If, f: &mut dyn FnMut(&'a Expr)) {
    for (c, b) in &i.branches {
        walk_expr(c, f);
        b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
    }
    if let Some(b) = &i.otherwise {
        b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
    }
}

fn walk_match<'a>(m: &'a Match, f: &mut dyn FnMut(&'a Expr)) {
    walk_expr(&m.subject, f);
    for arm in &m.arms {
        for p in &arm.patterns {
            if let Pattern::Value(v) = p {
                walk_expr(v, f);
            }
        }
        match &arm.body {
            ArmBody::Expr(x) => walk_expr(x, f),
            ArmBody::Block(b) => b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f)),
        }
    }
}

/// Calls `f` on every expression inside a statement (and nested statements).
pub fn walk_stmt_exprs<'a>(s: &'a Stmt, f: &mut dyn FnMut(&'a Expr)) {
    match s {
        Stmt::Expr(e) => walk_expr(e, f),
        Stmt::Assign(a) => {
            walk_expr(&a.target, f);
            walk_expr(&a.value, f);
        }
        Stmt::If(i) => walk_if(i, f),
        Stmt::For(fr) => {
            walk_expr(&fr.iter, f);
            fr.body.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
        }
        Stmt::While { cond, body, .. } => {
            walk_expr(cond, f);
            body.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
        }
        Stmt::Match(m) => walk_match(m, f),
        Stmt::Try(t) => {
            t.body.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
            if let Some((_, b)) = &t.catch {
                b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
            }
            if let Some(b) = &t.finally {
                b.stmts.iter().for_each(|s| walk_stmt_exprs(s, f));
            }
        }
        Stmt::Return { value: Some(v), .. } | Stmt::Throw { value: v, .. } | Stmt::Yield { value: v, .. } => {
            walk_expr(v, f)
        }
        _ => {}
    }
}

/// Calls `f` on every statement (including nested ones and those inside lambdas).
pub fn walk_stmts<'a>(stmts: &'a [Stmt], f: &mut dyn FnMut(&'a Stmt)) {
    for s in stmts {
        f(s);
        let mut nested: Vec<&'a Block> = Vec::new();
        match s {
            Stmt::If(i) => {
                nested.extend(i.branches.iter().map(|(_, b)| b));
                nested.extend(i.otherwise.iter());
            }
            Stmt::For(fr) => nested.push(&fr.body),
            Stmt::While { body, .. } => nested.push(body),
            Stmt::Try(t) => {
                nested.push(&t.body);
                nested.extend(t.catch.iter().map(|(_, b)| b));
                nested.extend(t.finally.iter());
            }
            Stmt::Match(m) => {
                for arm in &m.arms {
                    if let ArmBody::Block(b) = &arm.body {
                        nested.push(b);
                    }
                }
            }
            _ => {}
        }
        for b in nested {
            walk_stmts(&b.stmts, f);
        }
        // statements inside lambdas
        let mut lambda_blocks: Vec<&'a Block> = Vec::new();
        walk_stmt_exprs(s, &mut |e| {
            if let ExprKind::Lambda(l) = &e.kind
                && let LambdaBody::Block(b) = &l.body
            {
                lambda_blocks.push(b);
            }
        });
        for b in lambda_blocks {
            walk_stmts(&b.stmts, f);
        }
    }
}

/// Names created by an assignment directly in `stmts` (this block) and assigned again later in the
/// same block, nested blocks or lambdas included: they are mutable variables.
pub fn reassigned_after(stmts: &[Stmt]) -> HashSet<String> {
    let mut created: HashSet<String> = HashSet::new();
    let mut out = HashSet::new();
    for (i, s) in stmts.iter().enumerate() {
        // `[a, b] = xs`: every name of the pattern is created here
        if let Stmt::Assign(a) = s
            && a.op == AssignOp::Set
            && matches!(a.target.kind, ExprKind::List(_) | ExprKind::Object(_))
        {
            let later = assigned_in(&stmts[i + 1..]);
            for n in pattern_names(&a.target) {
                if !created.insert(n.clone()) || later.contains(&n) {
                    out.insert(n);
                }
            }
            continue;
        }
        if let Stmt::Assign(a) = s
            && let ExprKind::Name(n) = &a.target.kind
            && a.op == AssignOp::Set
            && created.insert(n.clone())
        {
            if assigned_in(&stmts[i + 1..]).contains(n) {
                out.insert(n.clone());
            }
            continue;
        }
        // `x += 1` on a name created above
        if let Stmt::Assign(a) = s
            && let ExprKind::Name(n) = &a.target.kind
            && created.contains(n)
        {
            out.insert(n.clone());
        }
    }
    out
}

fn assigned_in(stmts: &[Stmt]) -> HashSet<String> {
    let mut out = HashSet::new();
    walk_stmts(stmts, &mut |st| {
        if let Stmt::Assign(a) = st {
            out.extend(pattern_names(&a.target));
        }
    });
    out
}

/// The names a target writes: `x`, or every name of a pattern (`[a, ...rest]`, `{ name, "k": v }`),
/// without `_`.
pub fn pattern_names(target: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    fn go(e: &Expr, out: &mut Vec<String>) {
        match &e.kind {
            ExprKind::Name(n) if n != "_" => out.push(n.clone()),
            ExprKind::Spread(x) => go(x, out),
            ExprKind::List(items) => items.iter().for_each(|x| go(x, out)),
            ExprKind::Object(entries) => {
                for en in entries {
                    match en {
                        ObjEntry::Shorthand(i) => out.push(i.name.clone()),
                        ObjEntry::Pair { value, .. } => go(value, out),
                        ObjEntry::Spread(_) => {}
                    }
                }
            }
            _ => {}
        }
    }
    go(target, &mut out);
    out
}

/// Does the body `yield` (outside nested lambdas)? Then the function is a generator.
pub fn yields_block(stmts: &[Stmt]) -> bool {
    let mut found = false;
    let mut visit = |st: &Stmt| found |= matches!(st, Stmt::Yield { .. });
    // walk_stmts also enters lambdas: walk the nested blocks by hand instead
    fn go(stmts: &[Stmt], f: &mut dyn FnMut(&Stmt)) {
        for s in stmts {
            f(s);
            match s {
                Stmt::If(i) => {
                    i.branches.iter().for_each(|(_, b)| go(&b.stmts, f));
                    if let Some(b) = &i.otherwise {
                        go(&b.stmts, f);
                    }
                }
                Stmt::For(fr) => go(&fr.body.stmts, f),
                Stmt::While { body, .. } => go(&body.stmts, f),
                Stmt::Try(t) => {
                    go(&t.body.stmts, f);
                    if let Some((_, b)) = &t.catch {
                        go(&b.stmts, f);
                    }
                    if let Some(b) = &t.finally {
                        go(&b.stmts, f);
                    }
                }
                Stmt::Match(m) => {
                    for arm in &m.arms {
                        if let ArmBody::Block(b) = &arm.body {
                            go(&b.stmts, f);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    go(stmts, &mut visit);
    found
}

/// Does the body await (`~`) outside nested lambdas?
pub fn awaits_block(stmts: &[Stmt]) -> bool {
    stmts.iter().any(awaits_stmt)
}

fn awaits_stmt(s: &Stmt) -> bool {
    let mut found = false;
    match s {
        Stmt::If(i) => {
            found |= i.branches.iter().any(|(c, b)| awaits_expr(c) || awaits_block(&b.stmts));
            found |= i.otherwise.as_ref().is_some_and(|b| awaits_block(&b.stmts));
        }
        Stmt::For(f) => found |= awaits_expr(&f.iter) || awaits_block(&f.body.stmts),
        Stmt::While { cond, body, .. } => found |= awaits_expr(cond) || awaits_block(&body.stmts),
        Stmt::Try(t) => {
            found |= awaits_block(&t.body.stmts);
            found |= t.catch.as_ref().is_some_and(|(_, b)| awaits_block(&b.stmts));
            found |= t.finally.as_ref().is_some_and(|b| awaits_block(&b.stmts));
        }
        Stmt::Match(m) => {
            found |= awaits_expr(&m.subject);
            found |= m.arms.iter().any(|a| match &a.body {
                ArmBody::Expr(e) => awaits_expr(e),
                ArmBody::Block(b) => awaits_block(&b.stmts),
            });
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } | Stmt::Yield { value: e, .. } => found |= awaits_expr(e),
        Stmt::Return { value: Some(e), .. } => found |= awaits_expr(e),
        Stmt::Assign(a) => found |= awaits_expr(&a.target) || awaits_expr(&a.value),
        _ => {}
    }
    found
}

/// Does the expression await, not counting nested lambdas?
pub fn awaits_expr(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Unary { op: UnaryOp::Await, .. } => true,
        ExprKind::Lambda(_) => false,
        ExprKind::Unary { expr, .. } | ExprKind::Is { expr, .. } => awaits_expr(expr),
        ExprKind::Binary { left, right, .. } => awaits_expr(left) || awaits_expr(right),
        ExprKind::Ternary { cond, then, otherwise } => awaits_expr(cond) || awaits_expr(then) || awaits_expr(otherwise),
        ExprKind::Member { object, .. } => awaits_expr(object),
        ExprKind::Index { object, index } => awaits_expr(object) || awaits_expr(index),
        ExprKind::Spread(x) | ExprKind::NonNull(x) | ExprKind::Cast { expr: x, .. } => awaits_expr(x),
        ExprKind::ListIf { cond, value } => awaits_expr(cond) || awaits_expr(value),
        ExprKind::Cascade { target, sections } => {
            awaits_expr(target)
                || sections
                    .iter()
                    .any(|s| awaits_expr(&s.access) || s.assign.as_ref().is_some_and(|(_, v)| awaits_expr(v)))
        }
        ExprKind::Call { callee, args, .. } => awaits_expr(callee) || args.iter().any(|a| awaits_expr(&a.value)),
        ExprKind::List(items) => items.iter().any(awaits_expr),
        ExprKind::Object(entries) => entries.iter().any(|en| match en {
            ObjEntry::Pair { value, .. } | ObjEntry::Spread(value) => awaits_expr(value),
            ObjEntry::Shorthand(_) => false,
        }),
        ExprKind::Str(pieces) => pieces.iter().any(|p| matches!(p, StrPiece::Interp(x) if awaits_expr(x))),
        ExprKind::If(i) => awaits_stmt(&Stmt::If((**i).clone())),
        ExprKind::Match(m) => awaits_stmt(&Stmt::Match((**m).clone())),
        _ => false,
    }
}

pub fn lambda_awaits(l: &Lambda) -> bool {
    match &l.body {
        LambdaBody::Expr(e) => awaits_expr(e),
        LambdaBody::Block(b) => awaits_block(&b.stmts),
    }
}
