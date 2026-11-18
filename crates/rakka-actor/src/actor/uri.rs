// Clean-room implementation from docs/legal/cleanroom/uri-spec.md (Apache-2.0).
//! A parser that reproduces what `new java.net.URI(String)` exposes to Pekko: scheme, decoded user
//! information, host, port, raw path and raw fragment.

use super::errors::NumberFormatError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UriError {
    Syntax,
    NumberFormat(NumberFormatError),
}

#[derive(Debug, Default)]
pub(crate) struct Uri {
    pub(crate) scheme: Option<String>,
    raw_user_info: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) port: i32,
    pub(crate) raw_path: Option<String>,
    pub(crate) raw_fragment: Option<String>,
}

impl Uri {
    pub(crate) fn user_info(&self) -> Option<String> {
        self.raw_user_info.as_deref().map(decode_escapes)
    }

    pub(crate) fn is_absolute(&self) -> bool {
        self.scheme.is_some()
    }
}

// ---------------------------------------------------------------------------------------------
// Character classes (RFC 2396 with RFC 2732 brackets) plus Java's "other" characters.
// ---------------------------------------------------------------------------------------------

/// Which component a run of characters belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    UserInfo,
    RegistryName,
    Path,
    /// Query, fragment and the scheme-specific part of opaque URIs.
    Any,
}

fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || "-_.!~*'()".contains(c)
}

/// Non-ASCII, not an ISO control character and not a Unicode space separator (Zs, Zl, Zp).
fn is_other(c: char) -> bool {
    if c.is_ascii() {
        return false;
    }
    let cp = c as u32;
    let control = (0x80..=0x9F).contains(&cp);
    let space = matches!(
        cp,
        0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000
    );
    !control && !space
}

fn allows(class: Class, c: char) -> bool {
    if is_unreserved(c) || is_other(c) {
        return true;
    }
    let extra = match class {
        Class::UserInfo => ";:&=+$,",
        Class::RegistryName => "$,;:@&=+",
        Class::Path => ":@&=+$,;/",
        Class::Any => ";/?:@&=+$,[]",
    };
    extra.contains(c)
}

/// True when every character of `text` is legal in `class` and every `%` starts a valid escape.
fn valid_run(class: Class, text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '%' {
            let ok = bytes.get(i + 1).is_some_and(u8::is_ascii_hexdigit)
                && bytes.get(i + 2).is_some_and(u8::is_ascii_hexdigit);
            if !ok {
                return false;
            }
            chars.next();
            chars.next();
        } else if !allows(class, c) {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------------------------
// Top-level parse
// ---------------------------------------------------------------------------------------------

pub(crate) fn parse(s: &str) -> Result<Uri, UriError> {
    let mut uri = Uri {
        port: -1,
        ..Uri::default()
    };

    // A scheme exists only when ':' comes before any '/', '?' or '#'.
    let mut rest = s;
    if let Some(stop) = s.find(['/', '?', '#', ':'])
        && s.as_bytes()[stop] == b':'
    {
        let scheme = &s[..stop];
        if !valid_scheme(scheme) {
            return Err(UriError::Syntax);
        }
        uri.scheme = Some(scheme.to_owned());
        rest = &s[stop + 1..];
        if rest.is_empty() {
            return Err(UriError::Syntax);
        }
        if !rest.starts_with('/') {
            // Opaque: everything up to '#' is the scheme-specific part.
            let (body, fragment) = split_fragment(rest);
            if !valid_run(Class::Any, body) {
                return Err(UriError::Syntax);
            }
            set_fragment(&mut uri, fragment)?;
            return Ok(uri);
        }
    }

    let (body, fragment) = split_fragment(rest);
    let hierarchical = body;
    let after_authority = if let Some(after_slashes) = hierarchical.strip_prefix("//") {
        let end = after_slashes
            .find(['/', '?'])
            .unwrap_or(after_slashes.len());
        let authority = &after_slashes[..end];
        let remainder = &after_slashes[end..];
        if authority.is_empty() {
            if remainder.is_empty() && fragment.is_none() {
                return Err(UriError::Syntax);
            }
        } else {
            parse_authority(authority, &mut uri)?;
        }
        remainder
    } else {
        hierarchical
    };

    let (path, query) = match after_authority.find('?') {
        Some(q) => (&after_authority[..q], Some(&after_authority[q + 1..])),
        None => (after_authority, None),
    };
    if !valid_run(Class::Path, path) {
        return Err(UriError::Syntax);
    }
    if let Some(query) = query
        && !valid_run(Class::Any, query)
    {
        return Err(UriError::Syntax);
    }
    uri.raw_path = Some(path.to_owned());
    set_fragment(&mut uri, fragment)?;
    Ok(uri)
}

fn split_fragment(s: &str) -> (&str, Option<&str>) {
    match s.find('#') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    }
}

fn set_fragment(uri: &mut Uri, fragment: Option<&str>) -> Result<(), UriError> {
    if let Some(fragment) = fragment {
        if !valid_run(Class::Any, fragment) {
            return Err(UriError::Syntax);
        }
        uri.raw_fragment = Some(fragment.to_owned());
    }
    Ok(())
}

fn valid_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
}

// ---------------------------------------------------------------------------------------------
// Authority
// ---------------------------------------------------------------------------------------------

/// Tries the server-based form first, then falls back to a registry-based name.
fn parse_authority(authority: &str, uri: &mut Uri) -> Result<(), UriError> {
    // The server-based form is only attempted when every character could belong to one.
    let server_shaped = authority
        .chars()
        .all(|c| is_unreserved(c) || is_other(c) || ";:&=+$,@[]%".contains(c));
    if server_shaped && let Some(server) = parse_server(authority)? {
        uri.raw_user_info = server.user_info;
        uri.host = Some(server.host);
        uri.port = server.port;
        return Ok(());
    }
    if valid_run(Class::RegistryName, authority) {
        Ok(())
    } else {
        Err(UriError::Syntax)
    }
}

struct Server {
    user_info: Option<String>,
    host: String,
    port: i32,
}

/// `Ok(None)` means "not a server-based authority" (the caller may still accept it as a registry name).
fn parse_server(authority: &str) -> Result<Option<Server>, UriError> {
    let (user_info, host_port) = match authority.find('@') {
        Some(at) => {
            let user = &authority[..at];
            if !valid_run(Class::UserInfo, user) {
                return Ok(None);
            }
            (Some(user.to_owned()), &authority[at + 1..])
        }
        None => (None, authority),
    };

    let (host, port_text) = if let Some(inner_and_rest) = host_port.strip_prefix('[') {
        let Some(close) = inner_and_rest.find(']') else {
            return Ok(None);
        };
        if !valid_ipv6_reference(&inner_and_rest[..close])? {
            return Ok(None);
        }
        let after = &inner_and_rest[close + 1..];
        let port = if after.is_empty() {
            None
        } else if let Some(p) = after.strip_prefix(':') {
            Some(p)
        } else {
            return Ok(None);
        };
        (&host_port[..close + 2], port)
    } else {
        let (host, port) = match host_port.find(':') {
            Some(colon) => (&host_port[..colon], Some(&host_port[colon + 1..])),
            None => (host_port, None),
        };
        if !(is_ipv4_address(host) || is_hostname(host)) {
            return Ok(None);
        }
        (host, port)
    };

    let port = match port_text {
        None | Some("") => -1,
        Some(digits) => {
            if !digits.bytes().all(|b| b.is_ascii_digit()) {
                return Ok(None);
            }
            digits.parse::<i32>().map_err(|_| UriError::Syntax)?
        }
    };
    Ok(Some(Server {
        user_info,
        host: host.to_owned(),
        port,
    }))
}

// ---------------------------------------------------------------------------------------------
// Hosts
// ---------------------------------------------------------------------------------------------

fn is_ipv4_address(s: &str) -> bool {
    let mut count = 0;
    for part in s.split('.') {
        count += 1;
        let ok = !part.is_empty()
            && part.len() <= 3
            && part.bytes().all(|b| b.is_ascii_digit())
            && part.parse::<u16>().is_ok_and(|v| v <= 255);
        if !ok {
            return false;
        }
    }
    count == 4
}

fn is_hostname(s: &str) -> bool {
    let name = s.strip_suffix('.').unwrap_or(s);
    if name.is_empty() {
        return false;
    }
    let multi_label = name.contains('.');
    let mut labels = name.split('.').peekable();
    while let Some(label) = labels.next() {
        let bytes = label.as_bytes();
        let (Some(&first), Some(&last)) = (bytes.first(), bytes.last()) else {
            return false;
        };
        if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
            return false;
        }
        if !bytes
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return false;
        }
        if multi_label && labels.peek().is_none() && !first.is_ascii_alphabetic() {
            return false;
        }
    }
    true
}

/// Validates the text between `[` and `]`. An IPv4 octet in the trailing position that overflows
/// a 32-bit integer escapes as a `NumberFormat` error.
fn valid_ipv6_reference(inner: &str) -> Result<bool, UriError> {
    let (address, scope) = match inner.find('%') {
        Some(p) => (&inner[..p], Some(&inner[p + 1..])),
        None => (inner, None),
    };
    // The address is examined first: an overflowing embedded octet wins over a bad scope.
    if !valid_ipv6_address(address)? {
        return Ok(false);
    }
    Ok(scope.is_none_or(valid_scope))
}

fn valid_ipv6_address(address: &str) -> Result<bool, UriError> {
    let bytes = address.as_bytes();
    let mut i = 0;
    let mut groups = 0;
    let mut compressed = false;
    if bytes.starts_with(b"::") {
        compressed = true;
        i = 2;
    } else if bytes.first() == Some(&b':') {
        return Ok(false);
    }
    while i < bytes.len() {
        // One token: hex digits, or a dotted-decimal tail.
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'.') {
            i += 1;
        }
        let token = &address[start..i];
        if token.is_empty() {
            return Ok(false);
        }
        if token.contains('.') {
            // A dotted-decimal tail is only recognised after a ':' separator.
            if start == 0 || i != bytes.len() || !embedded_ipv4(token)? {
                return Ok(false);
            }
            groups += 2;
            break;
        }
        if token.len() > 4 {
            return Ok(false);
        }
        groups += 1;
        if i == bytes.len() {
            break;
        }
        // bytes[i] must be ':'
        if bytes[i] != b':' {
            return Ok(false);
        }
        if bytes.get(i + 1) == Some(&b':') {
            if compressed {
                return Ok(false);
            }
            compressed = true;
            i += 2;
        } else {
            i += 1;
            if i == bytes.len() {
                return Ok(false);
            }
        }
    }
    Ok(if compressed { groups < 8 } else { groups == 8 })
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._".contains(c))
}

/// Dotted-decimal tail of an IPv6 reference, octet by octet.
fn embedded_ipv4(token: &str) -> Result<bool, UriError> {
    let mut count = 0;
    for part in token.split('.') {
        count += 1;
        if count > 4 {
            return Ok(false);
        }
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(false);
        }
        let mut value: i32 = 0;
        for (index, digit) in part.bytes().enumerate() {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(i32::from(digit - b'0')))
                .ok_or_else(|| {
                    UriError::NumberFormat(NumberFormatError::new(format!(
                        "Error at index {index} in: \"{part}\""
                    )))
                })?;
        }
        if value > 255 {
            return Ok(false);
        }
    }
    Ok(count == 4)
}

// ---------------------------------------------------------------------------------------------
// User information decoding
// ---------------------------------------------------------------------------------------------

/// Replaces each run of `%XX` escapes by its UTF-8 decoding (malformed input becomes U+FFFD).
fn decode_escapes(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if bytes[i] != b'%' {
            let c = raw[i..].chars().next().unwrap_or('\u{FFFD}');
            out.push(c);
            i += c.len_utf8();
            continue;
        }
        let mut octets = Vec::new();
        while i < raw.len() && bytes[i] == b'%' {
            let hex = &raw[i + 1..i + 3];
            octets.push(u8::from_str_radix(hex, 16).unwrap_or(0));
            i += 3;
        }
        decode_utf8_lossy_into(&octets, &mut out);
    }
    out
}

fn is_continuation(b: u8) -> bool {
    b & 0xC0 == 0x80
}

/// UTF-8 decoding where each malformed prefix yields one U+FFFD (lengths fixed by recorded cases).
fn decode_utf8_lossy_into(octets: &[u8], out: &mut String) {
    let mut i = 0;
    while i < octets.len() {
        let lead = octets[i];
        let next = |n: usize| octets.get(i + n).copied();
        if lead < 0x80 {
            out.push(char::from(lead));
            i += 1;
            continue;
        }
        let (need, lo, hi) = match lead {
            0xC2..=0xDF => (1, 0x80, 0xBF),
            0xE0 => (2, 0xA0, 0xBF),
            0xE1..=0xEC | 0xEE | 0xEF => (2, 0x80, 0xBF),
            0xED => (2, 0x80, 0xBF),
            0xF0 => (3, 0x90, 0xBF),
            0xF1..=0xF3 => (3, 0x80, 0xBF),
            0xF4 => (3, 0x80, 0x8F),
            _ => {
                out.push('\u{FFFD}');
                i += 1;
                continue;
            }
        };
        let Some(second) = next(1).filter(|&b| is_continuation(b)) else {
            out.push('\u{FFFD}');
            i += 1;
            continue;
        };
        if !(lo..=hi).contains(&second) {
            // A surrogate encoding swallows its continuation; other overlong/out-of-range forms do not.
            if lead == 0xED {
                let consumed = if next(2).is_some_and(is_continuation) {
                    3
                } else {
                    2
                };
                out.push('\u{FFFD}');
                i += consumed;
            } else {
                out.push('\u{FFFD}');
                i += 1;
            }
            continue;
        }
        let mut length = 2;
        let mut complete = true;
        while length <= need {
            if next(length).is_some_and(is_continuation) {
                length += 1;
            } else {
                complete = false;
                break;
            }
        }
        if complete {
            if let Ok(text) = std::str::from_utf8(&octets[i..i + length]) {
                out.push_str(text);
            } else {
                out.push('\u{FFFD}');
            }
            i += length;
        } else {
            out.push('\u{FFFD}');
            i += length;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nfe_message(input: &str) -> String {
        match parse(input) {
            Err(UriError::NumberFormat(e)) => e.to_string(),
            other => panic!("expected NumberFormat, got {other:?}"),
        }
    }

    #[test]
    fn remote_authority_fields() {
        let u = parse("pekko://sys@host:2552/user/a%20b?q=1#42").unwrap();
        assert_eq!(u.scheme.as_deref(), Some("pekko"));
        assert_eq!(u.user_info().as_deref(), Some("sys"));
        assert_eq!(u.host.as_deref(), Some("host"));
        assert_eq!(u.port, 2552);
        assert_eq!(u.raw_path.as_deref(), Some("/user/a%20b"));
        assert_eq!(u.raw_fragment.as_deref(), Some("42"));
        assert!(u.is_absolute());
    }

    #[test]
    fn local_authority_and_empty_port() {
        let u = parse("pekko://sys:/x").unwrap();
        assert_eq!((u.host.as_deref(), u.port), (Some("sys"), -1));
        let u = parse("pekko://sys").unwrap();
        assert_eq!(u.raw_path.as_deref(), Some(""));
        assert_eq!(u.raw_fragment, None);
    }

    #[test]
    fn registry_authority_has_no_host() {
        let u = parse("pekko://a_b@c_d/x").unwrap();
        assert_eq!((u.host.clone(), u.user_info(), u.port), (None, None, -1));
        let u = parse("x://256").unwrap();
        assert_eq!(u.host.as_deref(), Some("256"));
    }

    #[test]
    fn hostname_rules() {
        assert!(parse("p://s@a.1/x").unwrap().host.is_none());
        assert_eq!(parse("p://s@1/x").unwrap().host.as_deref(), Some("1"));
        assert_eq!(parse("p://s@a.b./x").unwrap().host.as_deref(), Some("a.b."));
        assert!(parse("p://s@1.2.3.256/x").unwrap().host.is_none());
        assert_eq!(
            parse("p://s@1.2.3.04/x").unwrap().host.as_deref(),
            Some("1.2.3.04")
        );
    }

    #[test]
    fn syntax_errors() {
        for bad in [
            "pekko:",
            ":x",
            "1a://h",
            "//",
            "a b",
            "p://h/a b",
            "p://h/a[b]",
            "p://h#a#b",
            "p://h:99999999999",
            "p://[::1",
            "p://[1::2::3]",
            "p://[1:2:3:4:5:6:7:8:9]",
            "p://h/%zz",
            "p://h/\u{a0}",
            "p://h/\u{85}",
            "p://h/{",
        ] {
            assert_eq!(parse(bad).unwrap_err(), UriError::Syntax, "{bad}");
        }
    }

    #[test]
    fn legal_extras() {
        assert!(parse("pekko:opaque[x]#f").unwrap().raw_path.is_none());
        assert_eq!(
            parse("p://h/x?a[b]#c[d]").unwrap().raw_fragment.as_deref(),
            Some("c[d]")
        );
        assert_eq!(parse("p://h/x#").unwrap().raw_fragment.as_deref(), Some(""));
        assert_eq!(
            parse("p://h/\u{e9}").unwrap().raw_path.as_deref(),
            Some("/\u{e9}")
        );
        assert!(parse("//h/x").unwrap().scheme.is_none());
        assert!(
            parse("p://[fe80::1%eth0]:1/x")
                .unwrap()
                .host
                .unwrap()
                .starts_with('[')
        );
        assert!(parse("p://[::ffff:1.2.3.4]/x").is_ok());
        assert!(parse("p:///x").unwrap().host.is_none());
    }

    #[test]
    fn embedded_ipv4_overflow_is_number_format() {
        assert_eq!(
            nfe_message("p://[::1.2.3.99999999999]"),
            "Error at index 9 in: \"99999999999\""
        );
        assert_eq!(
            nfe_message("p://[::1.2.3.2147483648]"),
            "Error at index 9 in: \"2147483648\""
        );
        assert_eq!(parse("p://[::1.2.3.256]").unwrap_err(), UriError::Syntax);
        assert_eq!(
            parse("p://[1.2.3.99999999999]").unwrap_err(),
            UriError::Syntax
        );
        assert_eq!(
            parse("p://[::1.2.3.99999999999x]").unwrap_err(),
            UriError::Syntax
        );
        assert_eq!(
            parse("p://[::1.2.3.99999999999 ]").unwrap_err(),
            UriError::Syntax
        );
    }

    #[test]
    fn user_info_decoding() {
        let decoded = |s: &str| parse(&format!("p://{s}@h:1")).unwrap().user_info().unwrap();
        assert_eq!(decoded("a%C3%A9b"), "a\u{e9}b");
        assert_eq!(decoded("%E2%82%AC%F0%9F%98%80"), "\u{20ac}\u{1F600}");
        assert_eq!(decoded("a%ED%A0%80b"), "a\u{fffd}b");
        assert_eq!(decoded("%ED%A0b"), "\u{fffd}b");
        assert_eq!(decoded("%C0%80"), "\u{fffd}\u{fffd}");
        assert_eq!(decoded("%E2%82"), "\u{fffd}");
        assert_eq!(decoded("%F0%80%80%80"), "\u{fffd}".repeat(4));
        assert_eq!(decoded("%FF%FE"), "\u{fffd}\u{fffd}");
        assert_eq!(decoded("\u{e9}x"), "\u{e9}x");
    }

    #[test]
    fn deep_path_is_linear() {
        let s = format!("p://h{}", "/a".repeat(1_000_000));
        assert_eq!(parse(&s).unwrap().raw_path.unwrap().len(), 2_000_000);
    }
}
