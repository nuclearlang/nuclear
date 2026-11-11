//! `options.snake_case_names` (Python): the camelCase names of the `.nuc` (variables, functions, methods, fields,
//! parameters, named arguments, members) are written in snake_case, as the target's convention asks. The `.nuc`
//! keeps Nuclear's rule (camelCase, spec/nuclear-base.md §2.4): only the output changes. A name that exists as
//! it is in the dialect's type definitions (`getLogger` of logging) is kept. Strings and map keys are data and
//! are never renamed.

use nuclear_dialect::Dialect;
use nuclear_syntax::ast::*;
use std::collections::HashSet;

/// `getOne` → `get_one`, `parseHTMLText` → `parse_html_text`; PascalCase and names without capitals stay.
pub fn snake(name: &str) -> String {
    if !name.starts_with(|c: char| c.is_lowercase() || c == '_') || !name.chars().any(|c| c.is_uppercase()) {
        return name.to_string();
    }
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev = chars.get(i.wrapping_sub(1)).copied().filter(|_| i > 0);
            let next = chars.get(i + 1).copied();
            let starts_word = prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit())
                || (prev.is_some_and(char::is_uppercase) && next.is_some_and(char::is_lowercase));
            if starts_word && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Renames the file's names when the dialect asks for it.
pub fn apply(file: &mut File, d: &Dialect) {
    if !d.options.snake_case_names {
        return;
    }
    let keep: HashSet<String> = d
        .typedefs
        .declarations()
        .flat_map(|(_, decl)| {
            std::iter::once(decl.name.clone())
                .chain(decl.members.iter().map(|m| m.name.clone()))
                .chain(decl.params.iter().map(|p| p.name.clone()))
                .chain(decl.constructors.iter().flat_map(|c| c.params.iter().map(|p| p.name.clone())))
        })
        .filter(|n| n.chars().any(char::is_uppercase))
        .collect();
    let r = Renamer { keep };
    for item in &mut file.items {
        match item {
            Item::Use(u) => u.names.iter_mut().for_each(|n| r.ident(n)),
            Item::Decl(decl) => r.decl(decl),
            Item::Stmt(s) => r.stmt(s),
        }
    }
}

struct Renamer {
    keep: HashSet<String>,
}

impl Renamer {
    fn name(&self, n: &mut String) {
        if !self.keep.contains(n.as_str()) {
            *n = snake(n);
        }
    }

    fn ident(&self, i: &mut Ident) {
        self.name(&mut i.name);
    }

    fn decl(&self, d: &mut Decl) {
        self.ident(&mut d.name);
        d.decorators.iter_mut().for_each(|x| self.decorator(x));
        if let Some(p) = &mut d.params {
            p.list.iter_mut().for_each(|p| self.param(p));
        }
        match &mut d.body {
            Body::None => {}
            Body::Block(b) => self.block(b),
            Body::Expr(e) => self.expr(e),
            Body::Members(ms) => {
                for m in ms {
                    match m {
                        Member::Field(f) => {
                            self.ident(&mut f.name);
                            f.decorators.iter_mut().for_each(|x| self.decorator(x));
                            if let Some(e) = &mut f.default {
                                self.expr(e);
                            }
                        }
                        Member::Method(md) => self.decl(md),
                        Member::Ctor(c) => {
                            c.params.list.iter_mut().for_each(|p| self.param(p));
                            if let Some(b) = &mut c.body {
                                self.block(b);
                            }
                        }
                        Member::EnumItem(e) => e.args.iter_mut().for_each(|a| self.expr(a)),
                    }
                }
            }
        }
    }

    fn decorator(&self, d: &mut Decorator) {
        for a in d.args.iter_mut().flatten() {
            self.arg(a);
        }
    }

    fn param(&self, p: &mut Param) {
        self.ident(&mut p.name);
        p.decorators.iter_mut().for_each(|x| self.decorator(x));
        if let Some(e) = &mut p.default {
            self.expr(e);
        }
    }

    fn arg(&self, a: &mut Arg) {
        if let Some(n) = &mut a.name {
            self.ident(n);
        }
        self.expr(&mut a.value);
    }

    fn block(&self, b: &mut Block) {
        b.stmts.iter_mut().for_each(|s| self.stmt(s));
    }

    fn if_(&self, i: &mut If) {
        for (c, b) in &mut i.branches {
            self.expr(c);
            self.block(b);
        }
        if let Some(b) = &mut i.otherwise {
            self.block(b);
        }
    }

    fn match_(&self, m: &mut Match) {
        self.expr(&mut m.subject);
        for arm in &mut m.arms {
            for p in &mut arm.patterns {
                if let Pattern::Value(e) = p {
                    self.expr(e);
                }
            }
            match &mut arm.body {
                ArmBody::Expr(e) => self.expr(e),
                ArmBody::Block(b) => self.block(b),
            }
        }
    }

    fn stmt(&self, s: &mut Stmt) {
        match s {
            Stmt::Expr(e) | Stmt::Throw { value: e, .. } | Stmt::Yield { value: e, .. } => self.expr(e),
            Stmt::Assign(a) => {
                self.expr(&mut a.target);
                self.expr(&mut a.value);
            }
            Stmt::If(i) => self.if_(i),
            Stmt::For(f) => {
                f.vars.iter_mut().for_each(|v| self.ident(v));
                self.expr(&mut f.iter);
                self.block(&mut f.body);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::Match(m) => self.match_(m),
            Stmt::Try(t) => {
                self.block(&mut t.body);
                if let Some((v, b)) = &mut t.catch {
                    if let Some(v) = v {
                        self.ident(v);
                    }
                    self.block(b);
                }
                if let Some(b) = &mut t.finally {
                    self.block(b);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(e) = value {
                    self.expr(e);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn expr(&self, e: &mut Expr) {
        match &mut e.kind {
            ExprKind::Name(n) | ExprKind::SelfField(n) => self.name(n),
            ExprKind::Str(pieces) => {
                for p in pieces {
                    if let StrPiece::Interp(x) = p {
                        self.expr(x);
                    }
                }
            }
            ExprKind::List(xs) => xs.iter_mut().for_each(|x| self.expr(x)),
            ExprKind::Object(entries) => {
                for en in entries {
                    match en {
                        // a key is data (a JSON field, a column name): only its value is code
                        ObjEntry::Pair { value, .. } => self.expr(value),
                        ObjEntry::Shorthand(i) => self.ident(i),
                        ObjEntry::Spread(x) => self.expr(x),
                    }
                }
            }
            ExprKind::Unary { expr, .. }
            | ExprKind::Is { expr, .. }
            | ExprKind::Spread(expr)
            | ExprKind::NonNull(expr)
            | ExprKind::Cast { expr, .. } => self.expr(expr),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Ternary { cond, then, otherwise } => {
                self.expr(cond);
                self.expr(then);
                self.expr(otherwise);
            }
            ExprKind::Member { object, name, .. } => {
                self.expr(object);
                self.ident(name);
            }
            ExprKind::Index { object, index } => {
                self.expr(object);
                self.expr(index);
            }
            ExprKind::Call { callee, args, .. } => {
                self.expr(callee);
                args.iter_mut().for_each(|a| self.arg(a));
            }
            ExprKind::ListIf { cond, value } => {
                self.expr(cond);
                self.expr(value);
            }
            ExprKind::Cascade { target, sections } => {
                self.expr(target);
                for s in sections {
                    self.expr(&mut s.access);
                    if let Some((_, v)) = &mut s.assign {
                        self.expr(v);
                    }
                }
            }
            ExprKind::Lambda(l) => {
                l.params.iter_mut().for_each(|p| self.param(p));
                match &mut l.body {
                    LambdaBody::Expr(x) => self.expr(x),
                    LambdaBody::Block(b) => self.block(b),
                }
            }
            ExprKind::If(i) => self.if_(i),
            ExprKind::Match(m) => self.match_(m),
            ExprKind::This
            | ExprKind::Super
            | ExprKind::Number(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::CascadeSelf => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::snake;

    #[test]
    fn camel_to_snake() {
        assert_eq!(snake("getOne"), "get_one");
        assert_eq!(snake("parseHTMLText"), "parse_html_text");
        assert_eq!(snake("toV2"), "to_v2");
        assert_eq!(snake("_privateName"), "_private_name");
        assert_eq!(snake("name"), "name");
        assert_eq!(snake("already_snake"), "already_snake");
        assert_eq!(snake("Character"), "Character");
    }
}
