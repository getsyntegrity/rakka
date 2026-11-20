// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
// Original implementation: the HOCON behaviour was measured against Typesafe Config 1.4.9 (Apache-2.0, the library Pekko uses) with the oracle; none of its code was copied.
//! The value tree shared by the parser, the resolver and the getters.
//!
//! Every key of an object holds a *stack* of values, the highest-priority first (`withFallback`, a later assignment
//! of the same key, `+=`). The stack is what makes `${?x}` late-bound and lets a key refer to its own lower layers.

use std::collections::BTreeMap;

use super::double_text::java_double_to_string;

/// Object: key -> stack of values (highest priority first).
pub(crate) type Obj = BTreeMap<String, Vec<Val>>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Val {
    /// 1-based line of the origin; shown in error messages as `String: <line>`.
    pub line: u32,
    pub kind: Kind,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    Null,
    Bool(bool),
    Num(Num),
    Str(String),
    List(Vec<Val>),
    Obj(Obj),
    /// Unresolved concatenation (it contains a substitution).
    Concat(Vec<Piece>),
    /// Unresolved substitution.
    Subst(Subst),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Piece {
    Ws(String),
    V(Val),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Subst {
    pub path: Vec<String>,
    pub optional: bool,
}

impl Subst {
    pub(crate) fn display(&self) -> String {
        format!(
            "${{{}{}}}",
            if self.optional { "?" } else { "" },
            self.path
                .iter()
                .map(|segment| render_substitution_segment(segment))
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

/// A HOCON number: the original text is kept (it is what `getString` and the render show).
#[derive(Clone, Debug, PartialEq)]
pub struct Number {
    pub(crate) text: String,
    pub(crate) repr: NumRepr,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NumRepr {
    Int(i64),
    Float(f64),
}

pub(crate) type Num = Number;

impl Number {
    /// Parses a number token the way Typesafe Config 1.4.9 does: text that holds a `.`, `e` or `E` must be a valid
    /// Java `Double.parseDouble` literal (`1.`, `1.e5`, `-.5` are; `1e` is not), anything else must fit a `long`
    /// (`99999999999999999999` is not a number, it is text). `None` means the token is plain text.
    pub(crate) fn parse(text: &str) -> Option<Number> {
        let b = text.as_bytes();
        let mut i = 0;
        if i < b.len() && b[i] == b'-' {
            i += 1;
        }
        let int_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        let int_digits = i - int_start;
        let mut is_float = false;
        let mut fraction_digits = 0;
        if i < b.len() && b[i] == b'.' {
            is_float = true;
            i += 1;
            let fraction_start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            fraction_digits = i - fraction_start;
        }
        if int_digits == 0 && fraction_digits == 0 {
            return None;
        }
        if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
            is_float = true;
            i += 1;
            if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
                i += 1;
            }
            let exponent_start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i == exponent_start {
                return None;
            }
        }
        if i != b.len() {
            return None;
        }
        let repr = if is_float {
            NumRepr::Float(text.parse().ok()?)
        } else {
            NumRepr::Int(text.parse::<i64>().ok()?)
        };
        Some(Number {
            text: text.to_owned(),
            repr,
        })
    }

    /// The original text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// What `render` shows for the number: the *value*, not the original text (`1.50` is `1.5`, `010` is `10`).
    ///
    /// Typesafe Config turns a floating-point literal into a `long` when the double is a whole number that a `long`
    /// holds exactly (`1.0` is `1`, `5e18` is `5000000000000000000`); every other double is shown with Java's
    /// `Double.toString` (`1e-7` is `1.0E-7`, `1e400` is `Infinity`).
    pub(crate) fn render(&self) -> String {
        match self.repr {
            NumRepr::Int(v) => v.to_string(),
            NumRepr::Float(f) => {
                let whole = f as i64; // saturating, like Java's `(long) d`
                if whole as f64 == f {
                    whole.to_string()
                } else {
                    java_double_to_string(f)
                }
            }
        }
    }

    /// The value as a double.
    pub fn as_f64(&self) -> f64 {
        match self.repr {
            NumRepr::Int(v) => v as f64,
            NumRepr::Float(f) => f,
        }
    }
}

impl Val {
    pub(crate) fn new(line: u32, kind: Kind) -> Val {
        Val { line, kind }
    }
}

/// Keeps what is reachable in a stack: adjacent objects fuse, a concrete non-object value hides everything under it,
/// and an object hides a non-object value under it (`a { b = 1 }` over `a = 1` is the object).
pub(crate) fn normalize(stack: Vec<Val>) -> Vec<Val> {
    let mut out: Vec<Val> = Vec::with_capacity(stack.len());
    for item in stack {
        match &item.kind {
            Kind::Obj(m) => {
                if let Some(Val {
                    kind: Kind::Obj(last),
                    ..
                }) = out.last_mut()
                {
                    *last = merge_obj(last, m);
                } else {
                    out.push(item);
                }
            }
            Kind::Concat(_) | Kind::Subst(_) => out.push(item),
            _ => {
                if !out.iter().any(|v| matches!(v.kind, Kind::Obj(_))) {
                    out.push(item);
                }
                break;
            }
        }
    }
    out
}

/// `hi.withFallback(lo)` on two objects.
pub(crate) fn merge_obj(hi: &Obj, lo: &Obj) -> Obj {
    let mut result = lo.clone();
    for (k, hs) in hi {
        let mut stack = hs.clone();
        if let Some(low) = result.remove(k) {
            stack.extend(low);
        }
        result.insert(k.clone(), normalize(stack));
    }
    result
}

/// Path expression of `segments`, quoting what needs it (`a."b.c".d`).
pub(crate) fn join_path(segments: &[String]) -> String {
    segments
        .iter()
        .map(|s| render_segment(s))
        .collect::<Vec<_>>()
        .join(".")
}

/// A path element inside `${...}` as Typesafe Config prints it: quoted only when it is empty or has a character that is not
/// a letter, a digit, `-` or `_` (so `1`, `-1`, `007`, `true` and `1e5` stay bare). Letters and digits are the BMP ones, as
/// `Character.isLetterOrDigit(char)` sees UTF-16 units.
fn render_substitution_segment(segment: &str) -> String {
    let bare = |c: char| {
        (c as u32) <= 0xFFFF && (c.is_alphabetic() || c.is_numeric()) || c == '-' || c == '_'
    };
    if !segment.is_empty() && segment.chars().all(bare) {
        segment.to_owned()
    } else {
        quote(segment)
    }
}

fn render_segment(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !s.starts_with(|c: char| c.is_ascii_digit() || c == '-')
        && !matches!(s, "true" | "false" | "null");
    if plain { s.to_owned() } else { quote(s) }
}

/// JSON-style quoting of a string (what `render` shows for a string value).
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
