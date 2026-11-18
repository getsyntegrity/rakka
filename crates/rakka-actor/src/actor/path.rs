// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala and Address.scala (Apache-2.0).

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

use super::address::Address;
use super::errors::{
    ChildError, FromStringError, IllegalArgumentError, IllegalStateError, InvalidActorNameError,
    MalformedUrlError, NumberFormatError,
};
use super::jstring::{compare_utf16, parse_int};
use super::uri;

/// `RelativeActorPath.unapply`: elements of a relative URI, raw (not
/// decoded), with the raw fragment glued to the last one as `#fragment`.
/// `None` for absolute URIs and for strings `java.net.URI` rejects.
pub fn relative_actor_path_elements(s: &str) -> Option<Vec<String>> {
    let uri = uri::parse(s).ok()?;
    if uri.is_absolute() {
        return None;
    }
    Some(split(
        uri.raw_path.as_deref().unwrap_or(""),
        uri.raw_fragment.as_deref(),
    ))
}

/// `PathUtils.split`: every `/`-separated piece, empty ones included, with
/// `#fragment` appended to the last one.
fn split(path: &str, fragment: Option<&str>) -> Vec<String> {
    let mut elements: Vec<String> = path.split('/').map(str::to_owned).collect();
    if let (Some(fragment), Some(last)) = (fragment, elements.last_mut()) {
        last.push('#');
        last.push_str(fragment);
    }
    elements
}

const VALID_SYMBOLS: &str = "-_.*$+:@&=,!~';";

enum Invalid {
    Empty,
    /// UTF-16 index of the first illegal code unit.
    At(usize),
}

/// `findInvalidPathElementCharPosition`, over UTF-16 code units like Java.
fn find_invalid(units: &[u16]) -> Option<Invalid> {
    if units.is_empty() {
        return Some(Invalid::Empty);
    }
    let ascii = |u: u16| u8::try_from(u).ok().filter(u8::is_ascii);
    let is_valid = |u: u16| {
        ascii(u).is_some_and(|c| c.is_ascii_alphanumeric() || VALID_SYMBOLS.as_bytes().contains(&c))
    };
    let is_hex = |u: Option<&u16>| {
        u.and_then(|&u| ascii(u))
            .is_some_and(|c| c.is_ascii_hexdigit())
    };
    if units[0] == u16::from(b'$') {
        return Some(Invalid::At(0));
    }
    let mut pos = 0;
    while pos < units.len() {
        let u = units[pos];
        if is_valid(u) {
            pos += 1;
        } else if u == u16::from(b'%')
            && pos + 2 < units.len()
            && is_hex(units.get(pos + 1))
            && is_hex(units.get(pos + 2))
        {
            pos += 3;
        } else {
            return Some(Invalid::At(pos));
        }
    }
    None
}

/// `ActorPath.validatePathElement`. `full_path` is only used in the message.
///
/// The position in the message is a UTF-16 index. When the illegal code unit
/// is half of a surrogate pair, `message()` shows U+FFFD and `message_utf16()`
/// holds the lone surrogate exactly as Pekko does (Q-PATH-2).
pub fn validate_path_element(
    element: &str,
    full_path: Option<&str>,
) -> Result<(), InvalidActorNameError> {
    let full_path_msg = full_path.map_or(String::new(), |p| format!(" (in path [{p}])"));
    let units: Vec<u16> = element.encode_utf16().collect();
    match find_invalid(&units) {
        None => Ok(()),
        Some(Invalid::Empty) => Err(InvalidActorNameError::new(format!(
            "Actor path element must not be empty {full_path_msg}"
        ))),
        Some(Invalid::At(at)) => {
            // Built from UTF-16 units: when the illegal unit is half of a surrogate pair Pekko's message
            // holds that lone surrogate, which a Rust `String` cannot (see `message_utf16`).
            let head = format!(
                "Invalid actor path element [{element}]{full_path_msg}, illegal character ["
            );
            let tail = format!(
                "] at position: {at}. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: {VALID_SYMBOLS}."
            );
            let mut message: Vec<u16> = head.encode_utf16().collect();
            message.push(units[at]);
            message.extend(tail.encode_utf16());
            Err(InvalidActorNameError::from_utf16(message))
        }
    }
}

/// `ActorPath.isValidPathElement`.
pub fn is_valid_path_element(element: &str) -> bool {
    let units: Vec<u16> = element.encode_utf16().collect();
    find_invalid(&units).is_none()
}

/// `ActorPath`: a `RootActorPath` or a `ChildActorPath` pointing to its
/// parent, shared through `Arc`.
///
/// Equality and hashing ignore the uid. Deliberately not `Ord`: Pekko's
/// `compareTo` is not consistent with equality (see
/// [`ActorPath::pekko_compare`]).
#[derive(Clone)]
pub struct ActorPath(Arc<Node>);

enum Node {
    Root {
        address: Address,
        name: String,
    },
    Child {
        parent: ActorPath,
        name: String,
        uid: i32,
    },
}

/// Dropping a child releases its ancestors in a loop: the default drop glue
/// would recurse once per level and overflow the stack on deep paths (Pekko
/// builds, renders and frees a 1,000,000-level path; oracle case DEEP-01).
impl Drop for Node {
    fn drop(&mut self) {
        let Node::Child { parent, .. } = self else {
            return;
        };
        let mut next = std::mem::replace(parent, drop_placeholder());
        // `Arc::into_inner` yields the node only for the last strong
        // reference; a shared ancestor stays alive and ends the walk.
        while let Some(mut node) = Arc::into_inner(next.0) {
            match &mut node {
                Node::Child { parent, .. } => next = std::mem::replace(parent, drop_placeholder()),
                Node::Root { .. } => return,
            }
            // `node` now points at the placeholder, so its drop is shallow.
        }
    }
}

/// Stand-in parent swapped into a node being dropped; kept alive by the
/// static, so releasing it never recurses.
fn drop_placeholder() -> ActorPath {
    static PLACEHOLDER: LazyLock<ActorPath> =
        LazyLock::new(|| ActorPath::root(Address::local("pekko", "drop-placeholder")));
    PLACEHOLDER.clone()
}

/// `ActorCell.undefinedUid`.
const UNDEFINED_UID: i32 = 0;

/// `ActorCell.splitNameAndUid`: the name up to the first `#`, and the rest
/// read with `Integer.valueOf`.
fn split_name_and_uid(name: &str) -> Result<(&str, i32), NumberFormatError> {
    match name.split_once('#') {
        None => Ok((name, UNDEFINED_UID)),
        Some((name, uid)) => Ok((name, parse_int(uid)?)),
    }
}

impl ActorPath {
    /// `RootActorPath(address)`, named `/`.
    pub fn root(address: Address) -> ActorPath {
        ActorPath(Arc::new(Node::Root {
            address,
            name: "/".to_owned(),
        }))
    }

    /// `RootActorPath(address, name)`: `/` only as first char, no `#`.
    pub fn root_named(address: Address, name: &str) -> Result<ActorPath, IllegalArgumentError> {
        if name.chars().skip(1).any(|c| c == '/') {
            return Err(IllegalArgumentError::new(format!(
                "requirement failed: / may only exist at the beginning of the root actors name, \
                 it is a path separator and is not legal in ActorPath names: [{name}]"
            )));
        }
        if name.contains('#') {
            return Err(IllegalArgumentError::new(format!(
                "requirement failed: # is a fragment separator and is not legal in ActorPath names: [{name}]"
            )));
        }
        Ok(ActorPath(Arc::new(Node::Root {
            address,
            name: name.to_owned(),
        })))
    }

    /// Pekko's `/ child`: `name#uid` sets the uid of the new child.
    pub fn child(&self, name: &str) -> Result<ActorPath, ChildError> {
        let (name, uid) = split_name_and_uid(name)?;
        if name.contains('/') {
            return Err(IllegalArgumentError::new(format!(
                "/ is a path separator and is not legal in ActorPath names: [{name}]"
            ))
            .into());
        }
        Ok(self.new_child(name, uid))
    }

    /// A child with an explicit uid and no name checks (what the runtime does for names it validated itself).
    pub(crate) fn new_child(&self, name: &str, uid: i32) -> ActorPath {
        ActorPath(Arc::new(Node::Child {
            parent: self.clone(),
            name: name.to_owned(),
            uid,
        }))
    }

    /// Pekko's `/ Iterable`: appends every non-empty name.
    pub fn descendant<'a>(
        &self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<ActorPath, ChildError> {
        names
            .into_iter()
            .filter(|n| !n.is_empty())
            .try_fold(self.clone(), |path, name| path.child(name))
    }

    /// `ActorPath.fromString`. A malformed uid in the fragment escapes as
    /// `NumberFormatException`, like in Pekko.
    pub fn from_string(s: &str) -> Result<ActorPath, FromStringError> {
        let malformed = || MalformedUrlError::new(format!("cannot parse as ActorPath: {s}"));
        let uri = uri::parse(s).map_err(|error| match error {
            uri::UriError::Syntax => FromStringError::from(malformed()),
            // Not a URISyntaxException, so Pekko's `catch` does not swallow it.
            uri::UriError::NumberFormat(error) => FromStringError::from(error),
        })?;
        let raw_path = uri.raw_path.as_deref().ok_or_else(malformed)?;
        let address = Address::from_uri(&uri).ok_or_else(malformed)?;
        let elements = split(raw_path, uri.raw_fragment.as_deref());
        let mut path = ActorPath::root(address);
        for element in elements.iter().skip(1).filter(|e| !e.is_empty()) {
            // split() never yields a '/', so only the uid can fail.
            let (name, uid) = split_name_and_uid(element)?;
            path = path.new_child(name, uid);
        }
        Ok(path)
    }

    pub fn address(&self) -> &Address {
        match &*self.root_node().0 {
            Node::Root { address, .. } => address,
            Node::Child { .. } => unreachable!("root_node returns a root"),
        }
    }

    pub fn name(&self) -> &str {
        match &*self.0 {
            Node::Root { name, .. } | Node::Child { name, .. } => name,
        }
    }

    /// The parent; a root is its own parent.
    pub fn parent(&self) -> ActorPath {
        match &*self.0 {
            Node::Root { .. } => self.clone(),
            Node::Child { parent, .. } => parent.clone(),
        }
    }

    /// Pekko's `root`.
    pub fn root_path(&self) -> ActorPath {
        self.root_node().clone()
    }

    fn root_node(&self) -> &ActorPath {
        let mut p = self;
        while let Node::Child { parent, .. } = &*p.0 {
            p = parent;
        }
        p
    }

    pub fn is_root(&self) -> bool {
        matches!(&*self.0, Node::Root { .. })
    }

    /// Names from the root (excluded) to this path; `[""]` for a root.
    pub fn elements(&self) -> Vec<&str> {
        if self.is_root() {
            return vec![""];
        }
        let mut names = Vec::new();
        let mut p = self;
        while let Node::Child { parent, name, .. } = &*p.0 {
            names.push(name.as_str());
            p = parent;
        }
        names.reverse();
        names
    }

    pub fn to_string_without_address(&self) -> String {
        format!("/{}", self.elements().join("/"))
    }

    /// Replaces the address of a local root with `address`; a root with a
    /// host keeps its own. The ignore-ref path never changes.
    pub fn to_string_with_address(&self, address: &Address) -> String {
        if !self.is_root() && is_ignore_ref_path(self) {
            return self.to_string();
        }
        self.render(self.root_string_with_address(address))
    }

    /// `to_string` plus `#uid` when the uid is defined.
    pub fn to_serialization_format(&self) -> String {
        self.append_uid(self.to_string())
    }

    pub fn to_serialization_format_with_address(&self, address: &Address) -> String {
        if !self.is_root() && is_ignore_ref_path(self) {
            return self.to_string();
        }
        self.append_uid(self.to_string_with_address(address))
    }

    fn root_string_with_address(&self, address: &Address) -> String {
        let root = self.root_node();
        let own = root.address();
        if own.host().is_some() {
            format!("{own}{}", root.name())
        } else {
            format!("{address}{}", root.name())
        }
    }

    /// Root text followed by the child names joined with `/`, with no
    /// separator after the root (a root named `/x` gives `.../xy`).
    fn render(&self, root: String) -> String {
        let names = if self.is_root() {
            Vec::new()
        } else {
            self.elements()
        };
        root + &names.join("/")
    }

    fn append_uid(&self, mut s: String) -> String {
        let uid = self.uid();
        if uid != UNDEFINED_UID {
            s.push('#');
            s.push_str(&uid.to_string());
        }
        s
    }

    /// Pekko's `compareTo`: names compared from the leaf up by UTF-16 code
    /// units, a root sorts after a child, two roots compare by their text.
    /// A total preorder that is not consistent with `Eq`: two roots with the
    /// same text and different addresses compare equal.
    pub fn pekko_compare(&self, other: &ActorPath) -> Ordering {
        let (mut left, mut right) = (self, other);
        loop {
            if Arc::ptr_eq(&left.0, &right.0) {
                return Ordering::Equal;
            }
            match (&*left.0, &*right.0) {
                (Node::Root { .. }, Node::Root { .. }) => {
                    return compare_utf16(&left.to_string(), &right.to_string());
                }
                (Node::Root { .. }, Node::Child { .. }) => return Ordering::Greater,
                (Node::Child { .. }, Node::Root { .. }) => return Ordering::Less,
                (
                    Node::Child {
                        parent: lp,
                        name: ln,
                        ..
                    },
                    Node::Child {
                        parent: rp,
                        name: rn,
                        ..
                    },
                ) => match compare_utf16(ln, rn) {
                    Ordering::Equal => (left, right) = (lp, rp),
                    unequal => return unequal,
                },
            }
        }
    }

    /// Uid of this incarnation; `0` (undefined) for roots.
    #[doc(hidden)]
    pub fn uid(&self) -> i32 {
        match &*self.0 {
            Node::Root { .. } => UNDEFINED_UID,
            Node::Child { uid, .. } => *uid,
        }
    }

    /// Same path with another uid; returns this same instance if unchanged.
    #[doc(hidden)]
    pub fn with_uid(&self, uid: i32) -> Result<ActorPath, IllegalStateError> {
        match &*self.0 {
            _ if uid == self.uid() => Ok(self.clone()),
            Node::Root { .. } => Err(IllegalStateError::new(format!(
                "RootActorPath must have undefinedUid, [{uid} != {UNDEFINED_UID}"
            ))),
            Node::Child { parent, name, .. } => Ok(parent.new_child(name, uid)),
        }
    }

    /// Reference identity (Java `==`). Test-only: Pekko exposes no such API.
    #[cfg(test)]
    pub(crate) fn same_instance(&self, other: &ActorPath) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq for ActorPath {
    fn eq(&self, other: &Self) -> bool {
        let (mut left, mut right) = (self, other);
        loop {
            if Arc::ptr_eq(&left.0, &right.0) {
                return true;
            }
            match (&*left.0, &*right.0) {
                (
                    Node::Root {
                        address: la,
                        name: ln,
                    },
                    Node::Root {
                        address: ra,
                        name: rn,
                    },
                ) => return la == ra && ln == rn,
                (
                    Node::Child {
                        parent: lp,
                        name: ln,
                        ..
                    },
                    Node::Child {
                        parent: rp,
                        name: rn,
                        ..
                    },
                ) if ln == rn => (left, right) = (lp, rp),
                _ => return false,
            }
        }
    }
}

impl Eq for ActorPath {}

impl Hash for ActorPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut p = self;
        loop {
            match &*p.0 {
                Node::Root { address, name } => {
                    address.hash(state);
                    name.hash(state);
                    return;
                }
                Node::Child { parent, name, .. } => {
                    name.hash(state);
                    p = parent;
                }
            }
        }
    }
}

impl fmt::Display for ActorPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.root_node();
        f.write_str(&self.render(format!("{}{}", root.address(), root.name())))
    }
}

impl fmt::Debug for ActorPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ActorPath({})", self.to_serialization_format())
    }
}

/// `IgnoreActorRef.path`: `pekko://local/ignore`.
pub(crate) fn ignore_ref_path() -> ActorPath {
    static PATH: LazyLock<ActorPath> = LazyLock::new(|| {
        ActorPath::root(Address::local("pekko", "local")).new_child("ignore", UNDEFINED_UID)
    });
    PATH.clone()
}

/// `IgnoreActorRef.isIgnoreRefPath`: by equality, so the uid is ignored.
fn is_ignore_ref_path(path: &ActorPath) -> bool {
    *path == ignore_ref_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ordering::*;

    /// Oracle cases whose output depends on reference identity
    /// (`same_instance`, test-only), checked here instead of in
    /// tests/oracle_path_address.rs, which lists them as `UNIT_TESTED`.
    #[test]
    fn identity_cases_match_pekko_oracle() {
        let golden = include_str!("../../tests/oracle/path-address.golden");
        let line = |id: &str| {
            golden
                .lines()
                .find(|l| l.split_once(" -> ").is_some_and(|(i, _)| i == id))
                .unwrap_or_else(|| panic!("{id} not in golden"))
                .to_owned()
        };
        let render = |p: &ActorPath| {
            let elems: Vec<String> = p.elements().iter().map(|e| format!("[{e}]")).collect();
            format!(
                "{} str=[{p}] ser=[{}] noAddr=[{}] addr=[{}] name=[{}] uid={} elems=({})",
                if p.is_root() {
                    "RootActorPath"
                } else {
                    "ChildActorPath"
                },
                p.to_serialization_format(),
                p.to_string_without_address(),
                p.address(),
                p.name(),
                p.uid(),
                elems.join(",")
            )
        };
        let r = ActorPath::root(mysys());
        let w = r.with_uid(0).unwrap();
        let c3 = r.child("a#3").unwrap();
        let ch = r.child("a").unwrap().child("b#4").unwrap();
        let cases = [
            (
                "UID-01 root.withUid(0)",
                format!("same={} {}", w.same_instance(&r), render(&w)),
            ),
            (
                "UID-06 child#3.withUid(3) same",
                format!("same={}", c3.with_uid(3).unwrap().same_instance(&c3)),
            ),
            (
                "NAV-01 root",
                format!(
                    "{} parentIsSelf={} rootIsSelf={}",
                    render(&r),
                    r.parent().same_instance(&r),
                    r.root_path().same_instance(&r)
                ),
            ),
            (
                "NAV-02 child",
                format!(
                    "{} parent=[{}] parentUid={} rootIsSame={} root=[{}]",
                    render(&ch),
                    ch.parent(),
                    ch.parent().uid(),
                    ch.root_path().same_instance(&r),
                    ch.root_path()
                ),
            ),
        ];
        for (id, out) in cases {
            assert_eq!(format!("{id} -> ok {out}"), line(id));
        }
    }

    fn mysys() -> Address {
        Address::local("pekko", "mysys")
    }

    fn remote() -> Address {
        Address::remote("pekko", "mysys", "aaa", 7355)
    }

    /// (str, ser, noAddr, name, uid, elements)
    fn shape(p: &ActorPath) -> (String, String, String, String, i32, Vec<String>) {
        (
            p.to_string(),
            p.to_serialization_format(),
            p.to_string_without_address(),
            p.name().to_owned(),
            p.uid(),
            p.elements().into_iter().map(str::to_owned).collect(),
        )
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn from_string_keeps_raw_names_and_reads_the_uid_from_the_fragment() {
        // FS-04..10, 14..16, 36..39, 42, 44, 48..53
        let cases: &[(&str, &str, &str, i32, &[&str])] = &[
            (
                "pekko://sys/user/",
                "pekko://sys/user",
                "pekko://sys/user",
                0,
                &["user"],
            ),
            (
                "pekko://sys/user//x",
                "pekko://sys/user/x",
                "pekko://sys/user/x",
                0,
                &["user", "x"],
            ),
            (
                "pekko://sys/./a/../b",
                "pekko://sys/./a/../b",
                "pekko://sys/./a/../b",
                0,
                &[".", "a", "..", "b"],
            ),
            (
                "pekko://sys/user/a%20b",
                "pekko://sys/user/a%20b",
                "pekko://sys/user/a%20b",
                0,
                &["user", "a%20b"],
            ),
            (
                "pekko://sys/\u{FC}ser",
                "pekko://sys/\u{FC}ser",
                "pekko://sys/\u{FC}ser",
                0,
                &["\u{FC}ser"],
            ),
            (
                "pekko://sys/user?x#5",
                "pekko://sys/user",
                "pekko://sys/user#5",
                5,
                &["user"],
            ),
            (
                "pekko://sys/a#+5",
                "pekko://sys/a",
                "pekko://sys/a#5",
                5,
                &["a"],
            ),
            (
                "pekko://sys/a#-5",
                "pekko://sys/a",
                "pekko://sys/a#-5",
                -5,
                &["a"],
            ),
            (
                "pekko://sys/a#0",
                "pekko://sys/a",
                "pekko://sys/a",
                0,
                &["a"],
            ),
            (
                "pekko://sys/a#-2147483648",
                "pekko://sys/a",
                "pekko://sys/a#-2147483648",
                i32::MIN,
                &["a"],
            ),
            ("pekko://sys#5", "pekko://sys/", "pekko://sys/", 0, &[""]),
            ("pekko://sys/#5", "pekko://sys/", "pekko://sys/#5", 5, &[""]),
            (
                "pekko://sys/user/#5",
                "pekko://sys/user/",
                "pekko://sys/user/#5",
                5,
                &["user", ""],
            ),
            (
                "pekko://sys/a#\u{665}",
                "pekko://sys/a",
                "pekko://sys/a#5",
                5,
                &["a"],
            ),
            (
                "pekko://sys@[::1]:2552/user",
                "pekko://sys@[::1]:2552/user",
                "pekko://sys@[::1]:2552/user",
                0,
                &["user"],
            ),
            (
                "pekko://a%20b@h:1/u",
                "pekko://a b@h:1/u",
                "pekko://a b@h:1/u",
                0,
                &["u"],
            ),
        ];
        for &(input, str_, ser, uid, elems) in cases {
            let p = ActorPath::from_string(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            let (a, b, _, _, u, e) = shape(&p);
            assert_eq!(
                (a.as_str(), b.as_str(), u, e),
                (str_, ser, uid, s(elems)),
                "{input}"
            );
        }
        assert!(ActorPath::from_string("pekko://sys").unwrap().is_root());
    }

    #[test]
    fn from_string_errors() {
        // FS-11..13, 17..19, 21..23, 25, 26, 32..35, 40, 41, 43, 45..47, 54..56
        for input in [
            "pekko://sys/a%2",
            "pekko://sys/a b",
            "pekko://sys@in_valid:5001/user",
            "pekko://my_sys/user",
            "pekko:sys",
            "pekko:///user",
            "pekko://sys:1234/user",
            "pekko://sys@host/user",
            "/user/a",
            "user/a",
            "pekko://sys@h:99999999999/u",
            "pekko://sys/a#5#6",
            "pekko://sys/a# 5",
        ] {
            let e = ActorPath::from_string(input).unwrap_err();
            assert_eq!(
                (e.java_class(), e.message().to_owned()),
                (
                    "MalformedURLException",
                    format!("cannot parse as ActorPath: {input}")
                )
            );
        }
        for (input, fragment) in [
            ("pekko://sys/user#5?x", "5?x"),
            ("pekko://sys/a#abc", "abc"),
            ("pekko://sys/a#99999999999", "99999999999"),
            ("pekko://sys/a#2147483648", "2147483648"),
            ("pekko://sys/a#", ""),
            ("pekko://sys/a#+", "+"),
            ("pekko://sys/a#-", "-"),
            ("pekko://sys/a#+-5", "+-5"),
        ] {
            let e = ActorPath::from_string(input).unwrap_err();
            assert!(matches!(e, FromStringError::NumberFormat(_)), "{input}");
            assert_eq!(e.message(), format!("For input string: \"{fragment}\""));
        }
    }

    #[test]
    fn child_splits_name_and_uid() {
        // CH-*
        let user = ActorPath::root(mysys()).child("user").unwrap();
        for (name, leaf, uid) in [
            ("foo", "foo", 0),
            ("foo#7", "foo", 7),
            ("foo#+7", "foo", 7),
            ("foo#-7", "foo", -7),
            ("foo#0", "foo", 0),
            ("#5", "", 5),
            ("", "", 0),
            ("foo#\u{665}", "foo", 5),
            ("a b", "a b", 0),
            ("$a", "$a", 0),
            ("..", "..", 0),
            ("a?b", "a?b", 0),
            ("a%2", "a%2", 0),
        ] {
            let c = user.child(name).unwrap();
            assert_eq!((c.name(), c.uid()), (leaf, uid), "{name}");
            assert_eq!(c.to_string(), format!("pekko://mysys/user/{leaf}"));
        }
        for (name, input) in [
            ("foo#abc", "abc"),
            ("foo#", ""),
            ("foo#99999999999", "99999999999"),
            ("foo#1#2", "1#2"),
            ("foo#+", "+"),
        ] {
            let e = user.child(name).unwrap_err();
            assert!(matches!(e, ChildError::NumberFormat(_)));
            assert_eq!(e.message(), format!("For input string: \"{input}\""));
        }
        for name in ["a/b", "/"] {
            let e = user.child(name).unwrap_err();
            assert_eq!(e.java_class(), "IllegalArgumentException");
            assert_eq!(
                e.message(),
                format!("/ is a path separator and is not legal in ActorPath names: [{name}]")
            );
        }
    }

    #[test]
    fn descendant_skips_empty_names() {
        // DS-*
        let r = ActorPath::root(mysys());
        assert_eq!(
            r.descendant(["a", "", "b"]).unwrap().to_string(),
            "pekko://mysys/a/b"
        );
        assert!(r.descendant([]).unwrap().same_instance(&r));
        assert!(r.descendant(["", ""]).unwrap().is_root());
        assert_eq!(
            r.descendant(["a", "b#5"])
                .unwrap()
                .to_serialization_format(),
            "pekko://mysys/a/b#5"
        );
        assert_eq!(
            r.descendant(["a#5", "b"])
                .unwrap()
                .to_serialization_format(),
            "pekko://mysys/a/b"
        );
        assert_eq!(
            r.descendant(["a/b"]).unwrap_err().java_class(),
            "IllegalArgumentException"
        );
        assert_eq!(
            r.descendant(["a#x"]).unwrap_err().message(),
            "For input string: \"x\""
        );
        let d = r.child("a").unwrap().descendant(["c", "d"]).unwrap();
        assert_eq!(d.elements(), ["a", "c", "d"]);
    }

    #[test]
    fn root_named_rejects_inner_slash_and_hash() {
        // RT-*
        for (name, str_) in [
            ("/", "pekko://mysys/"),
            ("", "pekko://mysys"),
            ("user", "pekko://mysysuser"),
            ("/a", "pekko://mysys/a"),
        ] {
            let r = ActorPath::root_named(mysys(), name).unwrap();
            assert_eq!(
                shape(&r),
                (
                    str_.into(),
                    str_.into(),
                    "/".into(),
                    name.into(),
                    0,
                    s(&[""])
                )
            );
        }
        for name in ["/a/", "//", "a/b"] {
            assert_eq!(
                ActorPath::root_named(mysys(), name).unwrap_err().message(),
                format!(
                    "requirement failed: / may only exist at the beginning of the root actors name, it is a path separator and is not legal in ActorPath names: [{name}]"
                )
            );
        }
        for name in ["#", "/a#1", "a#b"] {
            assert_eq!(
                ActorPath::root_named(mysys(), name).unwrap_err().message(),
                format!(
                    "requirement failed: # is a fragment separator and is not legal in ActorPath names: [{name}]"
                )
            );
        }
    }

    #[test]
    fn with_uid() {
        // UID-*
        let r = ActorPath::root(mysys());
        assert!(r.with_uid(0).unwrap().same_instance(&r));
        for uid in [5, -1] {
            let e = r.with_uid(uid).unwrap_err();
            assert_eq!(e.java_class(), "IllegalStateException");
            assert_eq!(
                e.message(),
                format!("RootActorPath must have undefinedUid, [{uid} != 0")
            );
        }
        let c3 = r.child("a#3").unwrap();
        assert_eq!(
            c3.with_uid(9).unwrap().to_serialization_format(),
            "pekko://mysys/a#9"
        );
        assert_eq!(
            c3.with_uid(0).unwrap().to_serialization_format(),
            "pekko://mysys/a"
        );
        assert!(c3.with_uid(3).unwrap().same_instance(&c3));
        assert_eq!(c3.with_uid(i32::MIN).unwrap().uid(), i32::MIN);
    }

    #[test]
    fn navigation() {
        // NAV-*
        let r = ActorPath::root(mysys());
        assert!(r.parent().same_instance(&r) && r.root_path().same_instance(&r));
        assert_eq!((r.name(), r.elements()), ("/", vec![""]));
        let c = r.child("a").unwrap().child("b#4").unwrap();
        assert_eq!(
            (c.parent().to_string(), c.parent().uid()),
            ("pekko://mysys/a".into(), 0)
        );
        assert!(c.root_path().same_instance(&r));
        assert_eq!(c.address(), &mysys());
        assert_eq!(
            ActorPath::root(remote()).child("x").unwrap().address(),
            &remote()
        );
    }

    #[test]
    fn text_with_another_address() {
        // STR-*
        let c7 = ActorPath::root(mysys())
            .child("user")
            .unwrap()
            .child("foo#7")
            .unwrap();
        let other = Address::local("pekko", "other");
        assert_eq!(
            c7.to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/user/foo"
        );
        assert_eq!(
            c7.to_serialization_format_with_address(&remote()),
            "pekko://mysys@aaa:7355/user/foo#7"
        );
        assert_eq!(c7.to_string_with_address(&other), "pekko://other/user/foo");
        assert_eq!(
            c7.to_serialization_format_with_address(&other),
            "pekko://other/user/foo#7"
        );
        let bb = Address::remote("pekko", "mysys", "bb", 7355);
        let rc = ActorPath::root(remote()).child("foo#7").unwrap();
        assert_eq!(
            rc.to_serialization_format_with_address(&bb),
            "pekko://mysys@aaa:7355/foo#7"
        );
        assert_eq!(
            ActorPath::root(mysys()).to_serialization_format_with_address(&remote()),
            "pekko://mysys@aaa:7355/"
        );
        let named = ActorPath::root_named(mysys(), "/x").unwrap();
        assert_eq!(
            named.to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/x"
        );
        assert_eq!(
            named.child("y").unwrap().to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/xy"
        );
    }

    #[test]
    fn ignore_ref_path_never_changes_address_and_drops_uid() {
        // STR-11..18
        let local = Address::local("pekko", "local");
        assert_eq!(ignore_ref_path().to_string(), "pekko://local/ignore");
        assert_eq!(
            ignore_ref_path().to_serialization_format_with_address(&remote()),
            "pekko://local/ignore"
        );
        let built = ActorPath::root(local.clone()).child("ignore#5").unwrap();
        assert_eq!(
            built.to_string_with_address(&remote()),
            "pekko://local/ignore"
        );
        assert_eq!(
            built.to_serialization_format_with_address(&remote()),
            "pekko://local/ignore"
        );
        let below = ActorPath::root(local.clone())
            .descendant(["ignore", "x"])
            .unwrap();
        assert_eq!(
            below.to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/ignore/x"
        );
        assert_eq!(
            ActorPath::root(local).to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/"
        );
        let akka = ActorPath::root(Address::local("akka", "local"))
            .child("ignore")
            .unwrap();
        assert_eq!(
            akka.to_string_with_address(&remote()),
            "pekko://mysys@aaa:7355/ignore"
        );
    }

    fn cmp(a: &ActorPath, b: &ActorPath) -> (Ordering, Ordering, bool, bool) {
        (a.pekko_compare(b), b.pekko_compare(a), a == b, b == a)
    }

    #[test]
    fn compare_and_equality() {
        // CMPP-*
        use std::hash::{BuildHasher, RandomState};
        let h = RandomState::new();
        let r0 = ActorPath::root(mysys());
        let c = |p: &ActorPath, n: &str| p.child(n).unwrap();
        assert_eq!(
            cmp(&r0, &ActorPath::root(mysys())),
            (Equal, Equal, true, true)
        );
        assert_eq!(cmp(&r0, &c(&r0, "a")), (Greater, Less, false, false));
        assert_eq!(
            cmp(&c(&r0, "a"), &c(&r0, "b")),
            (Less, Greater, false, false)
        );
        assert_eq!(
            cmp(&c(&c(&r0, "a"), "b"), &c(&r0, "b")),
            (Less, Greater, false, false)
        );
        assert_eq!(
            cmp(&c(&c(&r0, "b"), "a"), &c(&c(&r0, "a"), "b")),
            (Less, Greater, false, false)
        );
        let (a1, a2) = (c(&r0, "a#1"), c(&r0, "a#2"));
        assert_eq!(cmp(&a1, &a2), (Equal, Equal, true, true));
        assert_eq!(h.hash_one(&a1), h.hash_one(&a2));
        let other = ActorPath::root(Address::local("pekko", "other"));
        assert_eq!(
            cmp(&c(&r0, "a"), &c(&other, "a")),
            (Less, Greater, false, false)
        );
        assert_eq!(
            cmp(&c(&r0, "a"), &c(&ActorPath::root(remote()), "a")),
            (Less, Greater, false, false)
        );
        let t1 = ActorPath::root(Address::local("p", "s@h:1"));
        let t2 = ActorPath::root(Address::remote("p", "s", "h", 1));
        assert_eq!(cmp(&t1, &t2), (Equal, Equal, false, false));
        assert_eq!(
            cmp(&c(&t1, "x"), &c(&t2, "x")),
            (Equal, Equal, false, false)
        );
        let n1 = ActorPath::root_named(Address::local("p", "s"), "/a").unwrap();
        let n2 = ActorPath::root_named(Address::local("p", "s/"), "a").unwrap();
        assert_eq!(cmp(&n1, &n2), (Equal, Equal, false, false));
        assert_eq!(
            cmp(&n1, &c(&ActorPath::root(Address::local("p", "s")), "a")),
            (Greater, Less, false, false)
        );
        assert_eq!(
            cmp(&c(&r0, "\u{FF61}"), &c(&r0, "\u{10000}")),
            (Greater, Less, false, false)
        );
        let parsed = ActorPath::from_string("pekko://mysys/user/a#9").unwrap();
        let built = c(&c(&r0, "user"), "a");
        assert_eq!(cmp(&parsed, &built), (Equal, Equal, true, true));
        assert_eq!(h.hash_one(&parsed), h.hash_one(&built));
        let ign = ActorPath::root(Address::local("pekko", "local"))
            .child("ignore#5")
            .unwrap();
        assert_eq!(cmp(&ignore_ref_path(), &ign), (Equal, Equal, true, true));
        assert_eq!(cmp(&c(&r0, ""), &r0), (Less, Greater, false, false));
    }

    const RULES: &str = ". Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.";

    fn invalid(element: &str, full_path: &str, c: &str, at: usize) -> String {
        format!(
            "Invalid actor path element [{element}]{full_path}, illegal character [{c}] at position: {at}{RULES}"
        )
    }

    #[test]
    fn valid_path_elements() {
        // VAL-02,03,06,07,10..12,24..27, VAL-FP-03
        for e in [
            "abc",
            "ABCxyz019",
            "a$b",
            "a$",
            "a%2Fb",
            "%41",
            "A%aF",
            "-_.*$+:@&=,!~';",
            "-",
            ".",
            "..",
        ] {
            assert!(is_valid_path_element(e), "{e}");
            validate_path_element(e, None).unwrap();
        }
        validate_path_element("ok", Some("/x")).unwrap();
    }

    #[test]
    fn empty_element_message_keeps_the_trailing_space() {
        // VAL-01, VAL-FP-01
        let e = validate_path_element("", None).unwrap_err();
        assert_eq!(e.message(), "Actor path element must not be empty ");
        let e = validate_path_element("", Some("/a/b")).unwrap_err();
        assert_eq!(
            e.message(),
            "Actor path element must not be empty  (in path [/a/b])"
        );
        assert!(!is_valid_path_element(""));
    }

    #[test]
    fn invalid_element_names_char_and_position() {
        // VAL-04,05,08,09,13..21,28..32
        for (e, c, at) in [
            ("$abc", "$", 0),
            ("$", "$", 0),
            ("a/b", "/", 1),
            ("a#b", "#", 1),
            ("a%2", "%", 1),
            ("a%", "%", 1),
            ("%", "%", 0),
            ("%g1", "%", 0),
            ("a%%41", "%", 1),
            ("a b", " ", 1),
            (" ", " ", 0),
            ("\t", "\t", 0),
            ("\u{FC}", "\u{FC}", 0),
            ("a\"b", "\"", 1),
            ("a?b", "?", 1),
            ("a\\b", "\\", 1),
            ("a(b", "(", 1),
            ("a[b", "[", 1),
        ] {
            assert!(!is_valid_path_element(e), "{e}");
            let err = validate_path_element(e, None).unwrap_err();
            assert_eq!(err.java_class(), "InvalidActorNameException");
            assert_eq!(err.message(), invalid(e, "", c, at));
        }
    }

    #[test]
    fn full_path_is_quoted_after_the_element() {
        // VAL-FP-02, VAL-FP-04
        let err = validate_path_element("$a", Some("/user/$a")).unwrap_err();
        assert_eq!(
            err.message(),
            invalid("$a", " (in path [/user/$a])", "$", 0)
        );
        let err = validate_path_element("a b", Some("")).unwrap_err();
        assert_eq!(err.message(), invalid("a b", " (in path [])", " ", 1));
    }

    #[test]
    fn position_counts_utf16_code_units() {
        // VAL-22/23: the position part; the illegal char itself is Q-PATH-2.
        assert!(
            validate_path_element("x\u{1F642}", None)
                .unwrap_err()
                .message()
                .contains(" at position: 1.")
        );
    }

    #[test]
    fn lone_surrogate_in_message_matches_pekko() {
        let err = validate_path_element("x\u{1F642}", None).unwrap_err();
        let units = err.message_utf16();
        let expected: Vec<u16> = "illegal character ["
            .encode_utf16()
            .chain([0xD83D, u16::from(b']')])
            .collect();
        assert!(units.windows(expected.len()).any(|w| w == expected));
    }

    /// Pekko 2.0.0 runs on JDK 17 (Q-X-16): `URI.scanByte` calls `Integer.parseInt(CharSequence, begin, end, 10)`,
    /// so an overflowing IPv4 octet escapes `ActorPath.fromString` as a `NumberFormatException`
    /// with the JDK 17 message (`Error at index N in: "digits"`), not as a `MalformedURLException`.
    #[test]
    fn ipv4_octet_overflow_in_ipv6_host_is_number_format_exception_with_jdk17_message() {
        // Both overflow at the tenth digit, so the JDK reports index 9.
        for (input, digits) in [
            ("pekko://sys@[::1.2.3.99999999999]/x", "99999999999"),
            (
                "pekko://sys@[1:2:3:4:5:6:1.2.3.2147483648]:1/x",
                "2147483648",
            ),
        ] {
            let error = ActorPath::from_string(input).unwrap_err();
            assert_eq!(error.java_class(), "NumberFormatException", "{input}");
            assert_eq!(
                error.message(),
                format!("Error at index 9 in: \"{digits}\""),
                "{input}"
            );
        }
    }

    #[test]
    fn relative_actor_path_elements_follow_java_net_uri() {
        // RP-*
        let cases: [(&str, Option<&[&str]>); 15] = [
            ("", Some(&[""])),
            ("/user/a", Some(&["", "user", "a"])),
            ("pekko://sys/a", None),
            ("foo:bar", None),
            ("a%2", None),
            ("a b", None),
            ("foo?x", Some(&["foo"])),
            ("foo#abc", Some(&["foo#abc"])),
            ("#5", Some(&["#5"])),
            ("../a", Some(&["..", "a"])),
            ("foo//bar", Some(&["foo", "", "bar"])),
            ("foo/", Some(&["foo", ""])),
            ("\u{FC}", Some(&["\u{FC}"])),
            ("a%20b", Some(&["a%20b"])),
            ("foo#1#2", None),
        ];
        for (input, expected) in cases {
            let expected = expected.map(|e| e.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            assert_eq!(relative_actor_path_elements(input), expected, "{input:?}");
        }
    }
}
