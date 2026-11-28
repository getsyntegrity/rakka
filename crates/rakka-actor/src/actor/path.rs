// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala and actor/src/main/scala/org/apache/pekko/actor/Address.scala (Apache-2.0).

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

/// `ActorPathExtractor.unapply`: the address of a URI and its non-root elements, the raw `#fragment` kept on the last one.
pub(crate) fn actor_path_extractor(s: &str) -> Option<(Address, Vec<String>)> {
    let uri = uri::parse(s).ok()?;
    let elements = split(uri.raw_path.as_deref()?, uri.raw_fragment.as_deref());
    Some((
        Address::from_uri(&uri)?,
        elements.into_iter().skip(1).collect(),
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
pub(crate) fn split_name_and_uid(name: &str) -> Result<(&str, i32), NumberFormatError> {
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
        Ok(self.child_with_uid(name, uid)?)
    }

    /// `new ChildActorPath(this, name, uid)` (`private[pekko]`): `name` is taken as it is, with no `#uid` split.
    #[doc(hidden)]
    pub fn child_with_uid(&self, name: &str, uid: i32) -> Result<ActorPath, IllegalArgumentError> {
        if name.contains('/') {
            return Err(IllegalArgumentError::new(format!(
                "/ is a path separator and is not legal in ActorPath names: [{name}]"
            )));
        }
        if name.contains('#') {
            return Err(IllegalArgumentError::new(format!(
                "# is a fragment separator and is not legal in ActorPath names: [{name}]"
            )));
        }
        Ok(self.new_child(name, uid))
    }

    /// A child with an explicit uid and no name checks, unlike [`ActorPath::child_with_uid`] and Pekko's constructor: for names
    /// already validated or split. The path of a selection that matches nothing can still pass a `#` here (Q-REF-8, proposed).
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
mod tests;
