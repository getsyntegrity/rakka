// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/actor/Address.scala (Apache-2.0).

use std::cmp::Ordering;
use std::fmt;

use super::errors::{IllegalArgumentError, MalformedUrlError};
use super::jstring::compare_utf16;
use super::uri::{self, Uri};

/// `protocol://system[@host][:port]`. Host and port are independent: the
/// constructor accepts any port (negatives included), the parser only
/// `0..=i32::MAX`.
///
/// Deliberately not `Ord`: Pekko's `addressOrdering` is not a total order
/// (see [`Address::pekko_compare`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address {
    protocol: String,
    system: String,
    host: Option<String>,
    port: Option<i32>,
}

impl Address {
    /// `new Address(protocol, system)`.
    pub fn local(protocol: impl Into<String>, system: impl Into<String>) -> Address {
        Address::new(protocol, system, None, None)
    }

    /// `Address(protocol, system, host, port)`.
    pub fn remote(
        protocol: impl Into<String>,
        system: impl Into<String>,
        host: impl Into<String>,
        port: i32,
    ) -> Address {
        Address::new(protocol, system, Some(host.into()), Some(port))
    }

    /// The primary constructor (`copy`), with host and port set independently.
    pub fn new(
        protocol: impl Into<String>,
        system: impl Into<String>,
        host: Option<String>,
        port: Option<i32>,
    ) -> Address {
        Address {
            protocol: protocol.into(),
            system: system.into(),
            host,
            port,
        }
    }

    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub fn system(&self) -> &str {
        &self.system
    }

    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    pub fn port(&self) -> Option<i32> {
        self.port
    }

    pub fn has_local_scope(&self) -> bool {
        self.host.is_none()
    }

    pub fn has_global_scope(&self) -> bool {
        self.host.is_some()
    }

    /// `system[@host][:port]`.
    pub fn host_port(&self) -> String {
        self.to_string()[self.protocol.len() + 3..].to_owned()
    }

    /// `AddressFromURIString(s)`: the error message is the input itself.
    pub fn parse(s: &str) -> Result<Address, MalformedUrlError> {
        uri::parse(s)
            .ok()
            .and_then(|u| Address::from_uri(&u))
            .ok_or_else(|| MalformedUrlError::new(s))
    }

    /// `AddressFromURIString.unapply(uri: URI)`.
    pub(crate) fn from_uri(uri: &Uri) -> Option<Address> {
        let scheme = uri.scheme.as_deref()?;
        match (uri.user_info(), uri.host.as_deref()) {
            (None, None) => None,
            (None, Some(host)) => (uri.port == -1).then(|| Address::local(scheme, host)),
            (Some(_), None) => None,
            (Some(user_info), Some(host)) => {
                (uri.port != -1).then(|| Address::remote(scheme, user_info, host, uri.port))
            }
        }
    }

    /// Pekko's `addressOrdering`. Strings compare by UTF-16 code units; a
    /// missing host compares as `""` and a missing port as `0`, but only once
    /// the fields differ, so this is not a total order: `None` and `Some("")`
    /// compare equal without being equal, and that equivalence is not
    /// transitive.
    pub fn pekko_compare(&self, other: &Address) -> Ordering {
        let less = |a: &Address, b: &Address| {
            if a.protocol != b.protocol {
                compare_utf16(&a.protocol, &b.protocol).is_lt()
            } else if a.system != b.system {
                compare_utf16(&a.system, &b.system).is_lt()
            } else if a.host != b.host {
                compare_utf16(a.host().unwrap_or(""), b.host().unwrap_or("")).is_lt()
            } else if a.port != b.port {
                a.port.unwrap_or(0) < b.port.unwrap_or(0)
            } else {
                false
            }
        };
        if less(self, other) {
            Ordering::Less
        } else if less(other, self) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    /// `InvalidHostRegex = "_[^.]*$"`: an `_` after the last `.`.
    #[doc(hidden)]
    pub fn has_invalid_host_characters(&self) -> bool {
        self.host
            .as_deref()
            .is_some_and(|h| h.rsplit('.').next().is_some_and(|last| last.contains('_')))
    }

    #[doc(hidden)]
    pub fn check_host_characters(&self) -> Result<(), IllegalArgumentError> {
        if self.has_invalid_host_characters() {
            return Err(IllegalArgumentError::new(format!(
                "requirement failed: Using invalid host characters '{}' in the Address is not allowed.",
                scala_option(self.host())
            )));
        }
        Ok(())
    }
}

fn scala_option(o: Option<&str>) -> String {
    match o {
        Some(v) => format!("Some({v})"),
        None => "None".to_owned(),
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.protocol, self.system)?;
        if let Some(host) = &self.host {
            write!(f, "@{host}")?;
        }
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Ordering::*;

    fn mysys() -> Address {
        Address::local("pekko", "mysys")
    }

    #[test]
    fn display_and_scopes_follow_the_constructor_domain() {
        // CT-01, CT-02, CT-04, CT-08, CT-09, CT-10, CT-12, CT-13, CT-14..17
        let cases: [(Address, &str, &str, bool); 12] = [
            (Address::local("pekko", "sys"), "pekko://sys", "sys", true),
            (
                Address::remote("pekko", "sys", "h", 1234),
                "pekko://sys@h:1234",
                "sys@h:1234",
                false,
            ),
            (
                Address::remote("pekko", "sys", "h", -5),
                "pekko://sys@h:-5",
                "sys@h:-5",
                false,
            ),
            (
                Address::remote("pekko", "sys", "h", i32::MIN),
                "pekko://sys@h:-2147483648",
                "sys@h:-2147483648",
                false,
            ),
            (
                Address::new("pekko", "sys", None, Some(1)),
                "pekko://sys:1",
                "sys:1",
                true,
            ),
            (
                Address::remote("pekko", "sys", "", 1),
                "pekko://sys@:1",
                "sys@:1",
                false,
            ),
            (
                Address::new("pekko", "mysys", Some(String::new()), None),
                "pekko://mysys@",
                "mysys@",
                false,
            ),
            (
                Address::new("pekko", "mysys", None, Some(7)),
                "pekko://mysys:7",
                "mysys:7",
                true,
            ),
            (Address::local("", "s"), "://s", "s", true),
            (Address::local("p", ""), "p://", "", true),
            (Address::local("p", "s@h:1"), "p://s@h:1", "s@h:1", true),
            (
                Address::remote("p", "s", "h:1", 2),
                "p://s@h:1:2",
                "s@h:1:2",
                false,
            ),
        ];
        for (a, s, hp, local) in cases {
            assert_eq!(a.to_string(), s);
            assert_eq!(a.host_port(), hp);
            assert_eq!(a.has_local_scope(), local, "{s}");
            assert_eq!(a.has_global_scope(), !local, "{s}");
        }
    }

    #[test]
    fn accessors_return_the_fields() {
        let a = Address::remote("pekko", "sys", "h", 70000);
        assert_eq!(
            (a.protocol(), a.system(), a.host(), a.port()),
            ("pekko", "sys", Some("h"), Some(70000))
        );
        let b = Address::local("pekko", "sys");
        assert_eq!((b.host(), b.port()), (None, None));
    }

    #[test]
    fn invalid_host_means_an_underscore_after_the_last_dot() {
        // HC-*
        for (host, invalid) in [
            ("_", true),
            ("a_", true),
            ("_a", true),
            ("a_b.c", false),
            ("a.b_c", true),
            ("a_b.c_d", true),
            ("a_.b", false),
            ("a._", true),
            ("\u{FC}_x", true),
        ] {
            let a = Address::remote("pekko", "sys", host, 1);
            assert_eq!(a.has_invalid_host_characters(), invalid, "{host}");
            let checked = a.check_host_characters();
            if invalid {
                let e = checked.unwrap_err();
                assert_eq!(e.java_class(), "IllegalArgumentException");
                assert_eq!(
                    e.message(),
                    format!(
                        "requirement failed: Using invalid host characters 'Some({host})' in the Address is not allowed."
                    )
                );
            } else {
                checked.unwrap();
            }
        }
        assert!(!mysys().has_invalid_host_characters());
        mysys().check_host_characters().unwrap();
    }

    fn cmp(a: &Address, b: &Address) -> (Ordering, Ordering) {
        (a.pekko_compare(b), b.pekko_compare(a))
    }

    #[test]
    fn pekko_compare_orders_by_protocol_system_host_port() {
        // CMPA-01..07, 14..16
        let r = |p: &str, s: &str, h: &str, port| Address::remote(p, s, h, port);
        assert_eq!(
            cmp(&r("pekko", "s", "h", 1), &r("pekko", "s", "h", 1)),
            (Equal, Equal)
        );
        assert_eq!(
            cmp(&r("akka", "s", "h", 1), &r("pekko", "s", "h", 1)),
            (Less, Greater)
        );
        assert_eq!(
            cmp(&Address::local("pekko", "a"), &Address::local("pekko", "b")),
            (Less, Greater)
        );
        assert_eq!(
            cmp(&r("pekko", "s", "h1", 9), &r("pekko", "s", "h2", 1)),
            (Less, Greater)
        );
        assert_eq!(
            cmp(&r("pekko", "s", "h", 2), &r("pekko", "s", "h", 10)),
            (Less, Greater)
        );
        assert_eq!(
            cmp(&r("pekko", "s", "h", -5), &r("pekko", "s", "h", 0)),
            (Less, Greater)
        );
        assert_eq!(
            cmp(&Address::local("pekko", "s"), &r("pekko", "s", "h", 1)),
            (Less, Greater)
        );
        assert_eq!(
            cmp(
                &Address::local("pekko", "\u{FF61}"),
                &Address::local("pekko", "\u{10000}")
            ),
            (Greater, Less)
        );
        assert_eq!(
            cmp(
                &Address::local("pekko", "Sys"),
                &Address::local("pekko", "sys")
            ),
            (Less, Greater)
        );
        let same_text = (Address::local("p", "s@h:1"), r("p", "s", "h", 1));
        assert_eq!(same_text.0.to_string(), same_text.1.to_string());
        assert_eq!(cmp(&same_text.0, &same_text.1), (Greater, Less));
        assert_ne!(same_text.0, same_text.1);
    }

    #[test]
    fn pekko_compare_is_not_a_total_order() {
        // CMPA-08..13
        let with = |h: Option<&str>, p| Address::new("pekko", "mysys", h.map(String::from), p);
        assert_eq!(cmp(&mysys(), &with(Some(""), None)), (Equal, Equal));
        assert_ne!(mysys(), with(Some(""), None));
        assert_eq!(
            cmp(&with(Some("h"), None), &with(Some("h"), Some(0))),
            (Equal, Equal)
        );
        let a = with(None, Some(1));
        let b = with(Some(""), Some(2));
        let c = with(None, Some(2));
        assert_eq!(cmp(&a, &b), (Equal, Equal));
        assert_eq!(cmp(&b, &c), (Equal, Equal));
        assert_eq!(cmp(&a, &c), (Less, Greater));
        assert_eq!(cmp(&mysys(), &with(Some(""), Some(5))), (Equal, Equal));
    }

    #[test]
    fn equal_addresses_hash_alike() {
        // CT-18
        use std::hash::{BuildHasher, RandomState};
        let s = RandomState::new();
        let a = Address::remote("pekko", "sys", "h", 1);
        let b = Address::new("pekko", "sys", Some("h".into()), Some(1));
        assert_eq!(a, b);
        assert_eq!(s.hash_one(&a), s.hash_one(&b));
    }

    #[test]
    fn parse_accepts_what_java_net_uri_and_pekko_accept() {
        // AD-* accepted: (input, toString, host, port)
        for (input, shown) in [
            ("pekko://my_sys@host:1234", "pekko://my_sys@host:1234"),
            ("pekko://sys", "pekko://sys"),
            ("pekko://sys/", "pekko://sys"),
            ("pekko://sys/user/a", "pekko://sys"),
            ("pekko://sys?x", "pekko://sys"),
            ("pekko://sys#5", "pekko://sys"),
            ("pekko://sys@h:1/user#5", "pekko://sys@h:1"),
            ("pekko://@host:1", "pekko://@host:1"),
            ("pekko://sys@host:0", "pekko://sys@host:0"),
            ("pekko://sys@host:00012", "pekko://sys@host:12"),
            ("pekko://sys@host:70000", "pekko://sys@host:70000"),
            ("pekko://sys@host:2147483647", "pekko://sys@host:2147483647"),
            ("pekko://sys@[::1]:2552", "pekko://sys@[::1]:2552"),
            ("pekko://sys@127.0.0.1:2552", "pekko://sys@127.0.0.1:2552"),
            ("pekko://a:b@h:1", "pekko://a:b@h:1"),
            ("pekko://a%20b@h:1", "pekko://a b@h:1"),
            ("pekko://s\u{FF}s@h:1", "pekko://s\u{FF}s@h:1"),
            ("PEKKO://Sys@HOST:1", "PEKKO://Sys@HOST:1"),
            (
                "pekko://sys@h-1.example.com:1",
                "pekko://sys@h-1.example.com:1",
            ),
            ("pekko://sys@h.:1", "pekko://sys@h.:1"),
            ("pekko://a%C3%BC@h:1", "pekko://a\u{FC}@h:1"),
            ("pekko://a%FF@h:1", "pekko://a\u{FFFD}@h:1"),
            (
                "pekko://sys@[::FFFF:1.2.3.4]:1",
                "pekko://sys@[::FFFF:1.2.3.4]:1",
            ),
            (
                "pekko://sys@[fe80::1%25eth0]:1",
                "pekko://sys@[fe80::1%25eth0]:1",
            ),
        ] {
            let a = Address::parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(a.to_string(), shown, "{input}");
        }
        let local = Address::parse("pekko://sys").unwrap();
        assert_eq!((local.host(), local.port()), (None, None));
        let at = Address::parse("pekko://@host:1").unwrap();
        assert_eq!(
            (at.system(), at.host(), at.port()),
            ("", Some("host"), Some(1))
        );
    }

    #[test]
    fn parse_rejects_with_the_input_as_message() {
        // AD-* rejected, CT-20, CT-21
        for input in [
            "pekko://sys@in_valid:5001",
            "pekko://sys:1234",
            "pekko://sys@host",
            "pekko://sys@host:",
            "pekko://sys@host:-1",
            "pekko://sys@host:2147483648",
            "pekko://sys@host:99999999999",
            "pekko://sys@[::1]",
            "pekko://sys@[::1",
            "pekko://sys@256.1.1.1:1",
            "pekko://my_sys",
            "pekko://a%2@h:1",
            "pekko://sys@h\u{F6}:1",
            "pekko://s\u{FF}s",
            "pekko:sys",
            "pekko:///",
            "pekko://",
            "pekko://s y",
            "/user",
            "sys@h:1",
            "1pekko://sys",
            "pekko://sys@-h:1",
            "pekko://sys@h_1.example.com:1",
            "pekko://sys@host:1234 ",
            "",
            "://hallo",
            "pekko://sys@[1::2::3]:1",
            "pekko://sys@::1:1",
            "pekko://sys@h:-5",
            "pekko://sys@in_valid:1",
        ] {
            let e = Address::parse(input).unwrap_err();
            assert_eq!(
                (e.java_class(), e.message()),
                ("MalformedURLException", input)
            );
        }
    }
}
