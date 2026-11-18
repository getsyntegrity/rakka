//! `Address` and `ActorPath` (Pekko Classic), slice 1: `ActorPath.fromString`.
//!
//! Reference: `org.apache.pekko.actor.ActorPath` / `ActorPathExtractor` / `AddressFromURIString`
//! in pekko-actor 2.0.0.

use crate::identity::jdk_int::parse_int;
use crate::identity::jdk_uri::{Uri, UriError};
use crate::{jstr, JStr};

/// `org.apache.pekko.actor.Address`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    pub protocol: JStr,
    pub system: JStr,
    pub host: Option<JStr>,
    pub port: Option<i32>,
}

impl Address {
    /// `Address.toString`: `protocol://system[@host][:port]`.
    pub fn to_jstring(&self) -> JStr {
        let mut s = self.protocol.clone();
        s.extend(jstr("://"));
        s.extend(&self.system);
        if let Some(h) = &self.host {
            s.push('@' as u16);
            s.extend(h);
        }
        if let Some(p) = self.port {
            s.push(':' as u16);
            s.extend(jstr(&p.to_string()));
        }
        s
    }

    /// `AddressFromURIString.unapply(uri)`.
    fn from_uri(uri: &Uri) -> Option<Address> {
        let scheme = uri.scheme.clone()?;
        let user_info = uri.user_info();
        if user_info.is_none() && uri.host.is_none() {
            return None;
        }
        match user_info {
            None => {
                // case 1: "pekko://system"
                if uri.port != -1 {
                    return None;
                }
                Some(Address { protocol: scheme, system: uri.host.clone()?, host: None, port: None })
            }
            Some(system) => {
                // case 2: "pekko://system@host:port"
                if uri.host.is_none() || uri.port == -1 {
                    return None;
                }
                Some(Address { protocol: scheme, system, host: uri.host.clone(), port: Some(uri.port) })
            }
        }
    }
}

/// An `ActorPath`: a `RootActorPath` (no elements) or a `ChildActorPath` chain.
///
/// Only the last element can carry a uid (`ChildActorPath.uid`); parents are built with the
/// undefined uid `0`, which is what `ActorPath.fromString` produces.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorPath {
    pub address: Address,
    pub elements: Vec<JStr>,
    pub uid: i32,
}

/// What `ActorPath.fromString` can throw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// Fully-qualified Java class name of the exception.
    pub exception: &'static str,
    pub message: JStr,
}

const MALFORMED_URL: &str = "java.net.MalformedURLException";
const NUMBER_FORMAT: &str = "java.lang.NumberFormatException";

impl ActorPath {
    /// `ActorPath.fromString(s)`.
    pub fn from_string(s: &[u16]) -> Result<ActorPath, Failure> {
        let malformed = || {
            let mut message = jstr("cannot parse as ActorPath: ");
            message.extend_from_slice(s);
            Failure { exception: MALFORMED_URL, message }
        };

        // ActorPathExtractor.unapply
        let uri = match Uri::parse(s) {
            Ok(u) => u,
            Err(UriError::Syntax(_)) => return Err(malformed()),
            // Not a URISyntaxException, so Pekko's `catch` does not swallow it.
            Err(UriError::NumberFormat(message)) => {
                return Err(Failure { exception: NUMBER_FORMAT, message })
            }
        };
        let Some(raw_path) = uri.raw_path.as_deref() else { return Err(malformed()) };
        let Some(address) = Address::from_uri(&uri) else { return Err(malformed()) };
        let elems = split(raw_path, uri.raw_fragment.as_deref());

        // RootActorPath(address) / elems.drop(1)
        let mut path = ActorPath { address, elements: Vec::new(), uid: 0 };
        for elem in elems.into_iter().skip(1) {
            if elem.is_empty() {
                continue;
            }
            // RootActorPath./ and ChildActorPath./ -> ActorCell.splitNameAndUid
            let (name, uid) = match elem.iter().position(|&c| c == '#' as u16) {
                None => (elem, 0),
                Some(i) => {
                    let uid = parse_int(&elem[i + 1..])
                        .map_err(|message| Failure { exception: NUMBER_FORMAT, message })?;
                    (elem[..i].to_vec(), uid)
                }
            };
            path.elements.push(name);
            path.uid = uid;
        }
        Ok(path)
    }

    /// `RootActorPath(address)`.
    pub fn root(address: Address) -> ActorPath {
        ActorPath { address, elements: Vec::new(), uid: 0 }
    }

    /// `path / name` with an explicit uid (`ChildActorPath(parent, name, uid)`).
    pub fn child(&self, name: &str, uid: i32) -> ActorPath {
        let mut elements = self.elements.clone();
        elements.push(jstr(name));
        ActorPath { address: self.address.clone(), elements, uid }
    }

    /// `ActorPath.parent`: the root is its own parent. The parent's own uid is not tracked.
    pub fn parent(&self) -> ActorPath {
        let mut elements = self.elements.clone();
        elements.pop();
        ActorPath { address: self.address.clone(), elements, uid: 0 }
    }

    /// `ActorPath.name` as a Rust string (lossy for lone surrogates).
    pub fn name_str(&self) -> String {
        String::from_utf16_lossy(&self.name())
    }

    /// `ActorPath.name`: `"/"` for the root.
    pub fn name(&self) -> JStr {
        self.elements.last().cloned().unwrap_or_else(|| jstr("/"))
    }

    /// `ActorPath.elements`: the root's elements are `List("")`.
    pub fn elements(&self) -> Vec<JStr> {
        if self.elements.is_empty() { vec![Vec::new()] } else { self.elements.clone() }
    }

    /// `ActorPath.toString`: `address/e1/e2...` (the root's name is `/`).
    pub fn to_jstring(&self) -> JStr {
        let mut s = self.address.to_jstring();
        s.extend(self.to_string_without_address());
        s
    }

    /// `ActorPath.toStringWithoutAddress`: `elements.mkString("/", "/", "")`.
    pub fn to_string_without_address(&self) -> JStr {
        let mut s = jstr("/");
        for (i, e) in self.elements.iter().enumerate() {
            if i > 0 {
                s.push('/' as u16);
            }
            s.extend(e);
        }
        s
    }

    /// `ActorPath.toSerializationFormat`: like `toString`, plus `#uid` when the uid is defined.
    pub fn to_serialization_format(&self) -> JStr {
        let mut s = self.to_jstring();
        if self.uid != 0 {
            s.push('#' as u16);
            s.extend(jstr(&self.uid.to_string()));
        }
        s
    }
}

/// `PathUtils.split(s, fragment)`: segments of `s` between `/`, with `#fragment` glued to the last.
fn split(s: &[u16], fragment: Option<&[u16]>) -> Vec<JStr> {
    let slash = '/' as u16;
    let mut acc: Vec<JStr> = Vec::new();
    let mut pos = s.len();
    loop {
        let from = s[..pos].iter().rposition(|&c| c == slash);
        let start = from.map_or(0, |f| f + 1);
        let mut sub = s[start..pos].to_vec();
        if let (Some(frag), true) = (fragment, acc.is_empty()) {
            sub.push('#' as u16);
            sub.extend_from_slice(frag);
        }
        acc.insert(0, sub);
        match from {
            None => return acc,
            Some(f) => pos = f,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<ActorPath, Failure> {
        ActorPath::from_string(&jstr(s))
    }

    fn text(s: &[u16]) -> String {
        String::from_utf16_lossy(s)
    }

    // Expected values below were observed from Pekko 2.0.0 (see cases/ and rust/app-cases/).

    #[test]
    fn remote_path_with_uid() {
        let p = parse("pekko://sys@host:2552/user/a/b#42").unwrap();
        assert_eq!(text(&p.to_jstring()), "pekko://sys@host:2552/user/a/b");
        assert_eq!(text(&p.to_serialization_format()), "pekko://sys@host:2552/user/a/b#42");
        assert_eq!(text(&p.to_string_without_address()), "/user/a/b");
        assert_eq!((text(&p.name()), p.uid), ("b".to_string(), 42));
        assert_eq!(p.address.port, Some(2552));
    }

    #[test]
    fn local_root_has_single_empty_element_and_slash_name() {
        let p = parse("pekko://sys").unwrap();
        assert_eq!(p.elements(), vec![Vec::<u16>::new()]);
        assert_eq!(text(&p.name()), "/");
        assert_eq!(text(&p.to_jstring()), "pekko://sys/");
        assert_eq!(p.address.host, None);
    }

    #[test]
    fn empty_path_elements_are_skipped() {
        let p = parse("pekko://sys/user//a///b/").unwrap();
        assert_eq!(text(&p.to_jstring()), "pekko://sys/user/a/b");
    }

    #[test]
    fn host_without_port_is_malformed() {
        let f = parse("pekko://sys@host/user").unwrap_err();
        assert_eq!(f.exception, "java.net.MalformedURLException");
        assert_eq!(text(&f.message), "cannot parse as ActorPath: pekko://sys@host/user");
    }

    #[test]
    fn bad_uid_is_number_format_exception() {
        let f = parse("pekko://sys/user/a#x").unwrap_err();
        assert_eq!(f.exception, "java.lang.NumberFormatException");
        assert_eq!(text(&f.message), "For input string: \"x\"");
    }

    #[test]
    fn number_format_exception_escapes_from_ipv6_literal_scan() {
        let f = parse("pekko://sys@[::1.2.3.99999999999]/x").unwrap_err();
        assert_eq!(f.exception, "java.lang.NumberFormatException");
        assert_eq!(text(&f.message), "Error at index 9 in: \"99999999999\"");
    }

    #[test]
    fn jdk_utf8_decoder_replaces_encoded_surrogate_once() {
        // ED A0 80 is one U+FFFD in the JDK (three with Rust's lossy decoding).
        let p = parse("pekko://%ED%A0%80@host:1/x").unwrap();
        assert_eq!(p.address.system, vec![0xFFFD]);
    }
}
