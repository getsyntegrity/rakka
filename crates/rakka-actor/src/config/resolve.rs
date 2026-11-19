// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Substitutions, `+=`, self-references and value concatenation.

use std::collections::HashMap;

use super::error::{ConfigError, ConfigErrorKind};
use super::model::{Kind, Obj, Piece, Subst, Val, join_path, merge_obj};

fn text_of(v: &Val) -> Option<String> {
    match &v.kind {
        Kind::Str(s) => Some(s.clone()),
        Kind::Num(n) => Some(n.text().to_owned()),
        Kind::Bool(b) => Some(b.to_string()),
        Kind::Null => Some("null".to_owned()),
        _ => None,
    }
}

fn bad_concat(line: u32) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::WrongType,
        format!("String: {line}: Cannot concatenate object or list with a non-object-or-list"),
    )
}

/// Joins the pieces of a concatenation that holds no substitution at its top level (or already resolved ones).
pub(crate) fn fold_pieces(pieces: &[Piece], line: u32) -> Result<Option<Val>, ConfigError> {
    let vals: Vec<&Val> = pieces
        .iter()
        .filter_map(|p| match p {
            Piece::V(v) => Some(v),
            Piece::Ws(_) => None,
        })
        .collect();
    let Some(first) = vals.first() else {
        return Ok(None);
    };
    if pieces.len() == 1 {
        return Ok(Some((*first).clone()));
    }
    let objs = vals
        .iter()
        .filter(|v| matches!(v.kind, Kind::Obj(_)))
        .count();
    let lists = vals
        .iter()
        .filter(|v| matches!(v.kind, Kind::List(_)))
        .count();
    if objs == vals.len() {
        let mut acc = Obj::new();
        for v in &vals {
            if let Kind::Obj(m) = &v.kind {
                acc = merge_obj(m, &acc);
            }
        }
        return Ok(Some(Val::new(first.line, Kind::Obj(acc))));
    }
    if lists == vals.len() {
        let mut acc = Vec::new();
        for v in &vals {
            if let Kind::List(l) = &v.kind {
                acc.extend(l.iter().cloned());
            }
        }
        return Ok(Some(Val::new(first.line, Kind::List(acc))));
    }
    if objs + lists > 0 {
        return Err(bad_concat(line));
    }
    let mut s = String::new();
    for p in pieces {
        match p {
            Piece::Ws(w) => s.push_str(w),
            Piece::V(v) => s.push_str(&text_of(v).ok_or_else(|| bad_concat(line))?),
        }
    }
    Ok(Some(Val::new(first.line, Kind::Str(s))))
}

/// A parsed concatenation: kept unresolved when it holds a substitution, folded otherwise.
pub(crate) fn combine(pieces: Vec<Piece>, line: u32, _resolving: bool) -> Result<Val, ConfigError> {
    if let [Piece::V(v)] = pieces.as_slice() {
        return Ok(v.clone());
    }
    let has_subst = pieces.iter().any(|p| {
        matches!(
            p,
            Piece::V(Val {
                kind: Kind::Subst(_),
                ..
            })
        )
    });
    if has_subst {
        return Ok(Val::new(line, Kind::Concat(pieces)));
    }
    Ok(fold_pieces(&pieces, line)?.unwrap_or(Val::new(line, Kind::Null)))
}

fn unresolved(line: u32, text: &str) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::UnresolvedSubstitution,
        format!("String: {line}: Could not resolve substitution to a value: {text}"),
    )
}

enum Found {
    Stack(Vec<Val>),
    Value(Val),
    Nothing,
}

struct Resolver<'a> {
    root: &'a Obj,
    memo: HashMap<Vec<String>, Option<Val>>,
    active: Vec<Vec<String>>,
    chain: Vec<String>,
}

/// Resolves every substitution of `root`; keys whose only value is a missing optional substitution disappear.
pub(crate) fn resolve_root(root: &Obj) -> Result<Obj, ConfigError> {
    let mut r = Resolver {
        root,
        memo: HashMap::new(),
        active: Vec::new(),
        chain: Vec::new(),
    };
    let top = Val::new(1, Kind::Obj(root.clone()));
    match r.resolve_stack(&[top], &[], None)? {
        Some(Val {
            kind: Kind::Obj(m), ..
        }) => Ok(m),
        _ => Ok(Obj::new()),
    }
}

impl Resolver<'_> {
    fn resolve_stack(
        &mut self,
        stack: &[Val],
        path: &[String],
        base: Option<Val>,
    ) -> Result<Option<Val>, ConfigError> {
        let tracked = !path.is_empty();
        if tracked {
            self.active.push(path.to_vec());
        }
        let mut acc = base;
        let mut result = Ok(());
        for item in stack.iter().rev() {
            match self.resolve_item(item, path, acc.take()) {
                Ok(next) => acc = next,
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }
        if tracked {
            self.active.pop();
        }
        result.map(|()| acc)
    }

    /// The value after laying `item` over `acc` (the value of the layers below it).
    fn resolve_item(
        &mut self,
        item: &Val,
        path: &[String],
        acc: Option<Val>,
    ) -> Result<Option<Val>, ConfigError> {
        match &item.kind {
            Kind::Null | Kind::Bool(_) | Kind::Num(_) | Kind::Str(_) => Ok(Some(item.clone())),
            Kind::List(elems) => {
                let mut out = Vec::with_capacity(elems.len());
                for e in elems {
                    if let Some(v) = self.resolve_item(e, path, None)? {
                        out.push(v);
                    }
                }
                Ok(Some(Val::new(item.line, Kind::List(out))))
            }
            Kind::Obj(map) => {
                let base_map = match &acc {
                    Some(Val {
                        kind: Kind::Obj(m), ..
                    }) => Some(m),
                    _ => None,
                };
                let mut out = base_map.cloned().unwrap_or_default();
                for (k, stack) in map {
                    let mut p = path.to_vec();
                    p.push(k.clone());
                    let child_base = base_map
                        .and_then(|m| m.get(k))
                        .and_then(|s| s.first())
                        .cloned();
                    if let Some(v) = self.resolve_stack(stack, &p, child_base)? {
                        out.insert(k.clone(), vec![v]);
                    }
                }
                Ok(Some(Val::new(item.line, Kind::Obj(out))))
            }
            Kind::Subst(s) => match self.eval_subst(s, item.line, path, &acc)? {
                Some(v) => Ok(Some(place(acc, v))),
                None => Ok(acc),
            },
            Kind::Concat(pieces) => {
                let mut resolved: Vec<Piece> = Vec::with_capacity(pieces.len());
                for p in pieces {
                    match p {
                        Piece::Ws(w) => resolved.push(Piece::Ws(w.clone())),
                        Piece::V(v) => {
                            let val = match &v.kind {
                                Kind::Subst(s) => self.eval_subst(s, v.line, path, &acc)?,
                                _ => self.resolve_item(v, path, None)?,
                            };
                            if let Some(x) = val {
                                resolved.push(Piece::V(x));
                            }
                        }
                    }
                }
                match fold_pieces(&resolved, item.line)? {
                    Some(v) => Ok(Some(place(acc, v))),
                    None => Ok(acc),
                }
            }
        }
    }

    fn eval_subst(
        &mut self,
        s: &Subst,
        line: u32,
        path: &[String],
        acc: &Option<Val>,
    ) -> Result<Option<Val>, ConfigError> {
        if s.path == path {
            // Self-reference: the value the key had before this layer.
            return match acc {
                Some(v) => Ok(Some(v.clone())),
                None if s.optional => Ok(None),
                None => Err(unresolved(line, &format!("${{{}}}", join_path(&s.path)))),
            };
        }
        match self.lookup(s, line)? {
            Some(v) => Ok(Some(v)),
            None if s.optional => Ok(None),
            None => Err(unresolved(line, &format!("${{{}}}", join_path(&s.path)))),
        }
    }

    fn lookup(&mut self, s: &Subst, line: u32) -> Result<Option<Val>, ConfigError> {
        if let Some(v) = self.memo.get(&s.path) {
            return Ok(v.clone());
        }
        let shown = format!("${{{}}}", join_path(&s.path));
        if self.active.contains(&s.path) {
            let mut cycle = self.chain.clone();
            cycle.push(shown.clone());
            if let Some(first) = self.chain.first() {
                cycle.push(first.clone());
            }
            return Err(unresolved(
                line,
                &format!(
                    "{shown} was part of a cycle of substitutions involving {}",
                    cycle.join(", ")
                ),
            ));
        }
        self.chain.push(shown);
        let found = self.find(&s.path, line);
        let r = match found {
            Ok(Found::Stack(st)) => self.resolve_stack(&st, &s.path, None),
            Ok(Found::Value(v)) => Ok(Some(v)),
            Ok(Found::Nothing) => Ok(std::env::var(s.path.join("."))
                .ok()
                .map(|v| Val::new(0, Kind::Str(v)))),
            Err(e) => Err(e),
        };
        self.chain.pop();
        let r = r?;
        self.memo.insert(s.path.clone(), r.clone());
        Ok(r)
    }

    fn find(&mut self, q: &[String], line: u32) -> Result<Found, ConfigError> {
        let mut cur: Vec<Val> = vec![Val::new(1, Kind::Obj(self.root.clone()))];
        for (i, key) in q.iter().enumerate() {
            let mut next: Vec<Val> = Vec::new();
            for it in &cur {
                match &it.kind {
                    Kind::Obj(m) => {
                        if let Some(st) = m.get(key) {
                            next.extend(st.iter().cloned());
                        }
                    }
                    Kind::Concat(_) | Kind::Subst(_) => {
                        let prefix = &q[..i];
                        if self.active.iter().any(|a| a == prefix) {
                            return Err(unresolved(
                                line,
                                &format!(
                                    "${{{}}} was part of a cycle of substitutions",
                                    join_path(q)
                                ),
                            ));
                        }
                        let whole = self.resolve_stack(&cur, prefix, None)?;
                        return Ok(navigate(whole, &q[i..]));
                    }
                    _ => break,
                }
            }
            if next.is_empty() {
                return Ok(Found::Nothing);
            }
            cur = next;
        }
        Ok(Found::Stack(cur))
    }
}

fn navigate(v: Option<Val>, rest: &[String]) -> Found {
    let mut cur = v;
    for k in rest {
        cur = match cur {
            Some(Val {
                kind: Kind::Obj(m), ..
            }) => m.get(k).and_then(|s| s.first()).cloned(),
            _ => None,
        };
    }
    cur.map_or(Found::Nothing, Found::Value)
}

/// A resolved value laid over the value below it: objects merge, anything else replaces.
fn place(acc: Option<Val>, v: Val) -> Val {
    match (&acc, &v.kind) {
        (
            Some(Val {
                kind: Kind::Obj(lo),
                ..
            }),
            Kind::Obj(hi),
        ) => Val::new(v.line, Kind::Obj(merge_obj(hi, lo))),
        _ => v,
    }
}
