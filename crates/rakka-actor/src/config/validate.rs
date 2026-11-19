// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
// Original implementation measured against Typesafe Config 1.4.9 (Apache-2.0) with the oracle (cases CV-* and RW-*); no code copied.
//! `checkValid` and `renderWithRedactions`.

use super::Config;
use super::error::{ConfigError, ConfigErrorKind};
use super::model::{Kind, Obj, Val, join_path, quote};

fn describe(v: &Val) -> String {
    match &v.kind {
        Kind::Obj(m) => format!(
            "object with keys [{}]",
            m.keys().cloned().collect::<Vec<_>>().join(", ")
        ),
        Kind::List(_) => "list".to_owned(),
        Kind::Str(_) => "string".to_owned(),
        Kind::Num(_) => "number".to_owned(),
        Kind::Bool(_) => "boolean".to_owned(),
        Kind::Null => "null".to_owned(),
        Kind::Concat(_) | Kind::Subst(_) => "unresolved".to_owned(),
    }
}

fn is_resolved(o: &Obj) -> bool {
    o.values().all(|st| {
        st.iter().all(|v| match &v.kind {
            Kind::Concat(_) | Kind::Subst(_) => false,
            Kind::Obj(m) => is_resolved(m),
            Kind::List(l) => l
                .iter()
                .all(|e| !matches!(e.kind, Kind::Concat(_) | Kind::Subst(_))),
            _ => true,
        })
    })
}

fn child<'a>(v: &'a Val, key: &str) -> Option<&'a Val> {
    match &v.kind {
        Kind::Obj(m) => m.get(key).and_then(|s| s.first()),
        _ => None,
    }
}

/// Can a configuration value of this type stand where the reference has `reference`?
fn compatible(reference: &Val, value: &Val) -> bool {
    if reference.kind == Kind::Null || value.kind == Kind::Null {
        return true;
    }
    match &reference.kind {
        Kind::Obj(_) => matches!(value.kind, Kind::Obj(_)),
        Kind::List(_) => matches!(value.kind, Kind::List(_) | Kind::Obj(_)),
        // A string can be read as any type, and any scalar as a string.
        Kind::Str(_) => true,
        _ => {
            matches!(value.kind, Kind::Str(_))
                || std::mem::discriminant(&reference.kind) == std::mem::discriminant(&value.kind)
        }
    }
}

fn check(
    reference: &Val,
    value: Option<&Val>,
    path: &[String],
    origin: &str,
    problems: &mut Vec<String>,
) {
    let shown = join_path(path);
    let Some(value) = value else {
        problems.push(format!(
            "{origin}: {shown}: No setting at '{shown}', expecting: {}",
            describe(reference)
        ));
        return;
    };
    if !compatible(reference, value) {
        problems.push(format!(
            "String: {}: {shown}: Wrong value type at '{shown}', expecting: {} but got: {}",
            value.line,
            describe(reference),
            describe(value)
        ));
        return;
    }
    if let (Kind::Obj(refm), Kind::Obj(_)) = (&reference.kind, &value.kind) {
        for (k, st) in refm {
            if let Some(rv) = st.first() {
                let mut p = path.to_vec();
                p.push(k.clone());
                check(rv, child(value, k), &p, origin, problems);
            }
        }
    }
}

pub(crate) fn check_valid(
    config: &Config,
    reference: &Config,
    restrict: &[&str],
) -> Result<(), ConfigError> {
    if !is_resolved(config.root_obj()) || !is_resolved(reference.root_obj()) {
        return Err(ConfigError::new(
            ConfigErrorKind::NotResolved,
            "need to Config#resolve() each config before using it, see the API docs for Config#resolve()",
        ));
    }
    let root_ref = Val::new(1, Kind::Obj(reference.root_obj().clone()));
    let root_cfg = Val::new(1, Kind::Obj(config.root_obj().clone()));
    let mut problems = Vec::new();
    let starts: Vec<Vec<String>> = if restrict.is_empty() {
        vec![Vec::new()]
    } else {
        restrict
            .iter()
            .map(|p| {
                super::parse::parse_path(p)
                    .map_err(|m| ConfigError::new(ConfigErrorKind::BadPath, m))
            })
            .collect::<Result<_, _>>()?
    };
    for start in starts {
        let mut rv = Some(&root_ref);
        let mut cv = Some(&root_cfg);
        for k in &start {
            rv = rv.and_then(|v| child(v, k));
            cv = cv.and_then(|v| child(v, k));
        }
        let Some(rv) = rv else { continue };
        if start.is_empty() {
            if let Kind::Obj(m) = &rv.kind {
                for (k, st) in m {
                    if let Some(x) = st.first() {
                        check(
                            x,
                            child(&root_cfg, k),
                            std::slice::from_ref(k),
                            config.origin(),
                            &mut problems,
                        );
                    }
                }
            }
        } else {
            check(rv, cv, &start, config.origin(), &mut problems);
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::new(
            ConfigErrorKind::ValidationFailed,
            problems.join(", "),
        ))
    }
}

fn render_value(v: &Val, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    match &v.kind {
        Kind::Obj(m) if m.is_empty() => out.push_str("{}"),
        Kind::Obj(m) => {
            out.push_str("{\n");
            let n = m.len();
            for (i, (k, st)) in m.iter().enumerate() {
                if let Some(x) = st.first() {
                    comment(x, indent + 4, out);
                    out.push_str(&format!("{}{} : ", " ".repeat(indent + 4), quote(k)));
                    render_value(x, indent + 4, out);
                    if i + 1 < n {
                        out.push(',');
                    }
                    out.push('\n');
                }
            }
            out.push_str(&format!("{pad}}}"));
        }
        Kind::List(l) if l.is_empty() => out.push_str("[]"),
        Kind::List(l) => {
            out.push_str("[\n");
            for (i, x) in l.iter().enumerate() {
                comment(x, indent + 4, out);
                out.push_str(&" ".repeat(indent + 4));
                render_value(x, indent + 4, out);
                if i + 1 < l.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{pad}]"));
        }
        Kind::Str(_) if v.line == 0 => out.push_str("\"<env variable>\""),
        Kind::Subst(s) => out.push_str(&s.display()),
        Kind::Concat(pieces) => {
            let parts: Vec<String> = pieces
                .iter()
                .map(|p| match p {
                    super::model::Piece::Ws(w) => w.clone(),
                    super::model::Piece::V(x) => {
                        let mut s = String::new();
                        render_value(x, indent, &mut s);
                        s
                    }
                })
                .collect();
            out.push_str(&parts.concat());
        }
        _ => out.push_str(&super::render_concise(v)),
    }
}

fn comment(v: &Val, indent: usize, out: &mut String) {
    let text = if v.line == 0 {
        "env variables".to_owned()
    } else {
        format!("String: {}", v.line)
    };
    out.push_str(&format!("{}# {text}\n", " ".repeat(indent)));
}

/// The user name of the running process (what Java reads as `user.name`).
fn user_name() -> Option<String> {
    ["USER", "LOGNAME", "USERNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .filter(|u| !u.is_empty())
}

pub(crate) fn render_with_redactions(root: &Obj) -> String {
    let mut out = String::new();
    render_value(&Val::new(1, Kind::Obj(root.clone())), 0, &mut out);
    out.push('\n');
    match user_name() {
        Some(u) => out.replace(&u, "<username>"),
        None => out,
    }
}
