// Parsing behavior ported from OpenJDK 21 java.base/java/net/URI.java (Parser, decode) and the
// malformed-input lengths of java.base/sun/nio/cs/UTF_8.java (Decoder) (GPL-2.0 WITH
// Classpath-exception-2.0), as used by Apache Pekko 1.7.1
// actor/src/main/scala/org/apache/pekko/actor/Address.scala (Apache-2.0).
// Licensing of this derived file pending Owner decision Q-PATH-6.
//! The subset of `java.net.URI` that is observable through Pekko's
//! `AddressFromURIString`, `ActorPathExtractor` and `RelativeActorPath`:
//! the parser (RFC 2396 with the JDK deviations, server authority falling back
//! to a registry authority) and the getters Pekko calls.
//!
//! Indices are positions in a `char` slice. The JDK works on UTF-16 units,
//! but every check treats both halves of a surrogate pair alike (non-ASCII,
//! not a space, not a control), so the outcome is the same.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UriSyntaxError;

type Res<T> = Result<T, UriSyntaxError>;

#[derive(Debug, Default)]
pub(crate) struct Uri {
    pub(crate) scheme: Option<String>,
    raw_user_info: Option<String>,
    pub(crate) host: Option<String>,
    /// `-1` when undefined, like `URI.getPort`.
    pub(crate) port: i32,
    /// `getRawPath`: `None` for opaque URIs.
    pub(crate) raw_path: Option<String>,
    pub(crate) raw_fragment: Option<String>,
}

impl Uri {
    /// `getUserInfo`: `%XX` escapes decoded as UTF-8, malformed bytes become U+FFFD.
    pub(crate) fn user_info(&self) -> Option<String> {
        self.raw_user_info.as_deref().map(decode)
    }

    pub(crate) fn is_absolute(&self) -> bool {
        self.scheme.is_some()
    }
}

/// `new URI(s)`.
pub(crate) fn parse(s: &str) -> Res<Uri> {
    let input: Vec<char> = s.chars().collect();
    let mut parser = Parser {
        input: &input,
        uri: Uri {
            port: -1,
            ..Uri::default()
        },
        ipv6_byte_count: 0,
    };
    parser.parse()?;
    Ok(parser.uri)
}

const fn low_mask(chars: &str) -> u64 {
    let b = chars.as_bytes();
    let mut m = 0u64;
    let mut i = 0;
    while i < b.len() {
        if b[i] < 64 {
            m |= 1 << b[i];
        }
        i += 1;
    }
    m
}

const fn high_mask(chars: &str) -> u64 {
    let b = chars.as_bytes();
    let mut m = 0u64;
    let mut i = 0;
    while i < b.len() {
        if b[i] >= 64 && b[i] < 128 {
            m |= 1 << (b[i] - 64);
        }
        i += 1;
    }
    m
}

#[derive(Clone, Copy)]
struct Mask(u64, u64);

impl Mask {
    const fn of(chars: &str) -> Mask {
        Mask(low_mask(chars), high_mask(chars))
    }

    const fn or(self, o: Mask) -> Mask {
        Mask(self.0 | o.0, self.1 | o.1)
    }

    fn matches(self, c: char) -> bool {
        let c = c as u32;
        match c {
            0 => false,
            1..64 => (1u64 << c) & self.0 != 0,
            64..128 => (1u64 << (c - 64)) & self.1 != 0,
            _ => false,
        }
    }

    fn escaped(self) -> bool {
        self.0 & ESCAPED.0 != 0
    }
}

const DIGIT: Mask = Mask::of("0123456789");
const ALPHA: Mask = Mask::of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");
const ALPHANUM: Mask = DIGIT.or(ALPHA);
const HEX: Mask = DIGIT.or(Mask::of("ABCDEFabcdef"));
const MARK: Mask = Mask::of("-_.!~*'()");
const UNRESERVED: Mask = ALPHANUM.or(MARK);
const RESERVED: Mask = Mask::of(";/?:@&=+$,[]");
/// Bit 0 (NUL) marks "escapes allowed", as in the JDK.
const ESCAPED: Mask = Mask(1, 0);
const URIC: Mask = RESERVED.or(UNRESERVED).or(ESCAPED);
const PCHAR: Mask = UNRESERVED.or(ESCAPED).or(Mask::of(":@&=+$,"));
const PATH: Mask = PCHAR.or(Mask::of(";/"));
const DASH: Mask = Mask::of("-");
const DOT: Mask = Mask::of(".");
const USERINFO: Mask = UNRESERVED.or(ESCAPED).or(Mask::of(";:&=+$,"));
const REG_NAME: Mask = UNRESERVED.or(ESCAPED).or(Mask::of("$,;:@&=+"));
const SERVER: Mask = USERINFO.or(ALPHANUM).or(DASH).or(Mask::of(".:@[]"));
const SERVER_PERCENT: Mask = SERVER.or(Mask::of("%"));
const SCHEME: Mask = ALPHA.or(DIGIT).or(Mask::of("+-."));
const SCOPE_ID: Mask = ALPHANUM.or(Mask::of("_."));

/// `Character.isSpaceChar` (Zs, Zl, Zp) for chars above U+0080.
fn is_space_char(c: char) -> bool {
    matches!(
        c,
        '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

fn is_iso_control(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{1F}' | '\u{7F}'..='\u{9F}')
}

struct Parser<'a> {
    input: &'a [char],
    uri: Uri,
    ipv6_byte_count: u32,
}

impl Parser<'_> {
    fn sub(&self, start: usize, end: usize) -> String {
        self.input[start..end].iter().collect()
    }

    fn at(&self, start: usize, end: usize, c: char) -> bool {
        start < end && self.input[start] == c
    }

    fn at_str(&self, start: usize, end: usize, s: &str) -> bool {
        let s: Vec<char> = s.chars().collect();
        start + s.len() <= end && self.input[start..start + s.len()] == s[..]
    }

    fn scan_char(&self, start: usize, end: usize, c: char) -> usize {
        if self.at(start, end, c) {
            start + 1
        } else {
            start
        }
    }

    /// `scan(start, end, err, stop)`: `None` when an `err` char comes first.
    fn scan_err_stop(&self, start: usize, end: usize, err: &str, stop: &str) -> Option<usize> {
        let mut p = start;
        while p < end {
            let c = self.input[p];
            if err.contains(c) {
                return None;
            }
            if stop.contains(c) {
                break;
            }
            p += 1;
        }
        Some(p)
    }

    fn scan_stop(&self, start: usize, end: usize, stop: &str) -> usize {
        let mut p = start;
        while p < end && !stop.contains(self.input[p]) {
            p += 1;
        }
        p
    }

    fn scan_escape(&self, p: usize, n: usize, c: char) -> Res<usize> {
        if c == '%' {
            if p + 3 <= n && HEX.matches(self.input[p + 1]) && HEX.matches(self.input[p + 2]) {
                return Ok(p + 3);
            }
            return Err(UriSyntaxError);
        }
        if (c as u32) > 128 && !is_space_char(c) && !is_iso_control(c) {
            return Ok(p + 1);
        }
        Ok(p)
    }

    fn scan(&self, start: usize, n: usize, mask: Mask) -> Res<usize> {
        let mut p = start;
        while p < n {
            let c = self.input[p];
            if mask.matches(c) {
                p += 1;
                continue;
            }
            if mask.escaped() {
                let q = self.scan_escape(p, n, c)?;
                if q > p {
                    p = q;
                    continue;
                }
            }
            break;
        }
        Ok(p)
    }

    fn check_chars(&self, start: usize, end: usize, mask: Mask) -> Res<()> {
        if self.scan(start, end, mask)? < end {
            return Err(UriSyntaxError);
        }
        Ok(())
    }

    fn parse(&mut self) -> Res<()> {
        let n = self.input.len();
        let mut p;
        match self.scan_err_stop(0, n, "/?#", ":") {
            Some(colon) if self.at(colon, n, ':') => {
                if colon == 0 {
                    return Err(UriSyntaxError);
                }
                self.check_chars(0, 1, ALPHA)?;
                self.check_chars(1, colon, SCHEME)?;
                self.uri.scheme = Some(self.sub(0, colon));
                p = colon + 1;
                if self.at(p, n, '/') {
                    p = self.parse_hierarchical(p, n)?;
                } else {
                    let q = self.scan_stop(p, n, "#");
                    if q <= p {
                        return Err(UriSyntaxError);
                    }
                    self.check_chars(p, q, URIC)?;
                    p = q;
                }
            }
            _ => p = self.parse_hierarchical(0, n)?,
        }
        if self.at(p, n, '#') {
            self.check_chars(p + 1, n, URIC)?;
            self.uri.raw_fragment = Some(self.sub(p + 1, n));
            p = n;
        }
        if p < n {
            return Err(UriSyntaxError);
        }
        Ok(())
    }

    fn parse_hierarchical(&mut self, start: usize, n: usize) -> Res<usize> {
        let mut p = start;
        if self.at(p, n, '/') && self.at(p + 1, n, '/') {
            p += 2;
            let q = self.scan_stop(p, n, "/?#");
            if q > p {
                p = self.parse_authority(p, q)?;
            } else if q >= n {
                return Err(UriSyntaxError);
            }
        }
        let q = self.scan_stop(p, n, "?#");
        self.check_chars(p, q, PATH)?;
        self.uri.raw_path = Some(self.sub(p, q));
        p = q;
        if self.at(p, n, '?') {
            p += 1;
            let q = self.scan_stop(p, n, "#");
            self.check_chars(p, q, URIC)?;
            p = q;
        }
        Ok(p)
    }

    fn reset_server(&mut self) {
        self.uri.raw_user_info = None;
        self.uri.host = None;
        self.uri.port = -1;
    }

    fn parse_authority(&mut self, p: usize, n: usize) -> Res<usize> {
        let server_mask = if self.scan_stop(p, n, "]") > p {
            SERVER_PERCENT
        } else {
            SERVER
        };
        let server_chars = self.scan(p, n, server_mask)? == n;
        let reg_chars = self.scan(p, n, REG_NAME)? == n;

        if reg_chars && !server_chars {
            return Ok(n);
        }

        // `new URI(String)` never requires a server authority, so a server
        // parse that fails or stops early falls back to a registry authority
        // when every char is a reg_name char.
        let mut q = p;
        if server_chars {
            match self.parse_server(p, n, reg_chars) {
                Ok(end) if end == n => q = end,
                _ => self.reset_server(),
            }
        }
        if q < n && !reg_chars {
            return Err(UriSyntaxError);
        }
        Ok(n)
    }

    fn parse_server(&mut self, start: usize, n: usize, skip_parse_exception: bool) -> Res<usize> {
        let mut p = start;
        if let Some(q) = self.scan_err_stop(p, n, "/?#", "@")
            && self.at(q, n, '@')
        {
            self.check_chars(p, q, USERINFO)?;
            self.uri.raw_user_info = Some(self.sub(p, q));
            p = q + 1;
        }

        if self.at(p, n, '[') {
            p += 1;
            match self.scan_err_stop(p, n, "/?#", "]") {
                Some(q) if q > p && self.at(q, n, ']') => {
                    let r = self.scan_stop(p, q, "%");
                    if r > p {
                        self.parse_ipv6_reference(p, r)?;
                        if r + 1 == q {
                            return Err(UriSyntaxError);
                        }
                        self.check_chars(r + 1, q, SCOPE_ID)?;
                    } else {
                        self.parse_ipv6_reference(p, q)?;
                    }
                    self.uri.host = Some(self.sub(p - 1, q + 1));
                    p = q + 1;
                }
                _ => return Err(UriSyntaxError),
            }
        } else {
            p = match self.parse_ipv4_address(p, n) {
                Some(q) if q > p => q,
                _ => self.parse_hostname(p, n, skip_parse_exception)?,
            };
        }

        if self.at(p, n, ':') {
            p += 1;
            let q = self.scan_stop(p, n, "/");
            if q > p {
                self.check_chars(p, q, DIGIT)?;
                let digits = self.sub(p, q);
                self.uri.port = digits.parse::<i32>().map_err(|_| UriSyntaxError)?;
                p = q;
            }
        } else if p < n && skip_parse_exception {
            return Ok(p);
        }
        if p < n {
            return Err(UriSyntaxError);
        }
        Ok(p)
    }

    fn scan_byte(&self, start: usize, n: usize) -> Res<usize> {
        let p = start;
        let q = self.scan(p, n, DIGIT)?;
        if q <= p {
            return Ok(q);
        }
        let mut i = p;
        while self.scan_char(i, q, '0') > i {
            i += 1;
        }
        let significant = q - i;
        if significant < 3 {
            return Ok(q);
        }
        if significant > 3 {
            return Ok(p);
        }
        let value: u32 = self.sub(p, q).parse().map_err(|_| UriSyntaxError)?;
        Ok(if value > 255 { p } else { q })
    }

    fn scan_ipv4_address(&self, start: usize, n: usize, strict: bool) -> Res<Option<usize>> {
        let mut p = start;
        let m = self.scan(p, n, DIGIT.or(DOT))?;
        if m <= p || (strict && m != n) {
            return Ok(None);
        }
        for i in 0..4 {
            let q = self.scan_byte(p, m)?;
            if q <= p {
                break;
            }
            p = q;
            if i < 3 {
                let q = self.scan_char(p, m, '.');
                if q <= p {
                    break;
                }
                p = q;
            } else if q < m {
                break;
            } else {
                return Ok(Some(q));
            }
        }
        if strict {
            return Err(UriSyntaxError);
        }
        Ok(None)
    }

    fn take_ipv4_address(&self, start: usize, n: usize) -> Res<usize> {
        match self.scan_ipv4_address(start, n, true)? {
            Some(p) if p > start => Ok(p),
            _ => Err(UriSyntaxError),
        }
    }

    fn parse_ipv4_address(&mut self, start: usize, n: usize) -> Option<usize> {
        let p = self.scan_ipv4_address(start, n, false).ok()??;
        if p > start && p < n && self.input[p] != ':' {
            return None;
        }
        if p > start {
            self.uri.host = Some(self.sub(start, p));
        }
        Some(p)
    }

    fn parse_hostname(&mut self, start: usize, n: usize, skip_parse_exception: bool) -> Res<usize> {
        let mut p = start;
        let mut last_label = None;
        loop {
            let q = self.scan(p, n, ALPHANUM)?;
            if q <= p {
                break;
            }
            last_label = Some(p);
            p = q;
            let q = self.scan(p, n, ALPHANUM.or(DASH))?;
            if q > p {
                if self.input[q - 1] == '-' {
                    return Err(UriSyntaxError);
                }
                p = q;
            }
            let q = self.scan_char(p, n, '.');
            if q <= p {
                break;
            }
            p = q;
            if p >= n {
                break;
            }
        }
        if p < n && !self.at(p, n, ':') {
            if skip_parse_exception {
                return Ok(p);
            }
            return Err(UriSyntaxError);
        }
        let l = last_label.ok_or(UriSyntaxError)?;
        if l > start && !ALPHA.matches(self.input[l]) {
            return Err(UriSyntaxError);
        }
        self.uri.host = Some(self.sub(start, p));
        Ok(p)
    }

    fn parse_ipv6_reference(&mut self, start: usize, n: usize) -> Res<usize> {
        let mut p = start;
        let mut compressed_zeros = false;
        if let Some(q) = self.scan_hex_seq(p, n)? {
            p = q;
            if self.at_str(p, n, "::") {
                compressed_zeros = true;
                p = self.scan_hex_post(p + 2, n)?;
            } else if self.at(p, n, ':') {
                p = self.take_ipv4_address(p + 1, n)?;
                self.ipv6_byte_count += 4;
            }
        } else if self.at_str(p, n, "::") {
            compressed_zeros = true;
            p = self.scan_hex_post(p + 2, n)?;
        }
        let malformed = p < n
            || self.ipv6_byte_count > 16
            || (!compressed_zeros && self.ipv6_byte_count < 16)
            || (compressed_zeros && self.ipv6_byte_count == 16);
        if malformed {
            return Err(UriSyntaxError);
        }
        Ok(p)
    }

    fn scan_hex_post(&mut self, start: usize, n: usize) -> Res<usize> {
        let mut p = start;
        if p == n {
            return Ok(p);
        }
        if let Some(q) = self.scan_hex_seq(p, n)? {
            p = q;
            if self.at(p, n, ':') {
                p = self.take_ipv4_address(p + 1, n)?;
                self.ipv6_byte_count += 4;
            }
        } else {
            p = self.take_ipv4_address(p, n)?;
            self.ipv6_byte_count += 4;
        }
        Ok(p)
    }

    fn scan_hex_seq(&mut self, start: usize, n: usize) -> Res<Option<usize>> {
        let mut p = start;
        let q = self.scan(p, n, HEX)?;
        if q <= p || self.at(q, n, '.') {
            return Ok(None);
        }
        if q > p + 4 {
            return Err(UriSyntaxError);
        }
        self.ipv6_byte_count += 2;
        p = q;
        while p < n {
            if !self.at(p, n, ':') || self.at(p + 1, n, ':') {
                break;
            }
            p += 1;
            let q = self.scan(p, n, HEX)?;
            if q <= p {
                return Err(UriSyntaxError);
            }
            if self.at(q, n, '.') {
                p -= 1;
                break;
            }
            if q > p + 4 {
                return Err(UriSyntaxError);
            }
            self.ipv6_byte_count += 2;
            p = q;
        }
        Ok(Some(p))
    }
}

/// `URI.decode`: each run of `%XX` becomes bytes decoded with the JDK UTF-8
/// `CharsetDecoder` set to `REPLACE` (see [`decode_utf8_like_jdk`]).
/// Escapes are well-formed here (the parser checked them).
fn decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len());
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        bytes.clear();
        let mut c = c;
        loop {
            debug_assert_eq!(c, '%');
            let hi = chars.next().and_then(|d| d.to_digit(16)).unwrap_or(0);
            let lo = chars.next().and_then(|d| d.to_digit(16)).unwrap_or(0);
            bytes.push((hi * 16 + lo) as u8);
            match chars.peek() {
                Some('%') => c = chars.next().unwrap_or('%'),
                _ => break,
            }
        }
        decode_utf8_like_jdk(&bytes, &mut out);
    }
    out
}

/// UTF-8 decoding with the malformed-input lengths of the JDK decoder
/// (`sun.nio.cs.UTF_8.Decoder` driven by `CharsetDecoder.decode(in, out,
/// true)`): every malformed sequence becomes one U+FFFD. It differs from
/// `String::from_utf8_lossy` (WHATWG "maximal subpart") in two places: an
/// encoded surrogate (`ED A0..BF xx`) is one malformed sequence of up to three
/// bytes, and a truncated sequence at the end of the run is always one.
fn decode_utf8_like_jdk(bytes: &[u8], out: &mut String) {
    let cont = |b: u8| b & 0xC0 == 0x80;
    let mut i = 0;
    while i < bytes.len() {
        let b1 = bytes[i];
        let rest = &bytes[i..];
        let at = |k: usize| rest.get(k).copied();
        // Number of bytes consumed by one decoded char or one U+FFFD.
        let (decoded, len): (Option<char>, usize) = match b1 {
            0x00..=0x7F => (Some(char::from(b1)), 1),
            0xC2..=0xDF => match at(1) {
                // Truncated at the end of input: malformed for the remaining length.
                None => (None, 1),
                Some(b2) if !cont(b2) => (None, 1),
                Some(b2) => (
                    char::from_u32((u32::from(b1 & 0x1F) << 6) | u32::from(b2 & 0x3F)),
                    2,
                ),
            },
            0xE0..=0xEF => {
                // malformedN(3): the E0 overlong prefix or a bad second byte
                // is 1, a bad third byte is 2.
                let bad_first_two = |b2: u8| (b1 == 0xE0 && b2 & 0xE0 == 0x80) || !cont(b2);
                match (at(1), at(2)) {
                    (Some(b2), _) if bad_first_two(b2) => (None, 1),
                    (Some(_), Some(b3)) if !cont(b3) => (None, 2),
                    (Some(b2), Some(b3)) => {
                        let c = (u32::from(b1 & 0x0F) << 12)
                            | (u32::from(b2 & 0x3F) << 6)
                            | u32::from(b3 & 0x3F);
                        // A surrogate is malformed for its three bytes.
                        (char::from_u32(c), 3)
                    }
                    _ => (None, rest.len()),
                }
            }
            0xF0..=0xF7 => {
                // malformedN(4) and the truncated-input checks
                // (isMalformed4_2 / isMalformed4_3).
                let bad_second = |b2: u8| {
                    b1 > 0xF4
                        || (b1 == 0xF0 && !(0x90..=0xBF).contains(&b2))
                        || (b1 == 0xF4 && b2 & 0xF0 != 0x80)
                        || !cont(b2)
                };
                match (at(1), at(2), at(3)) {
                    _ if b1 > 0xF4 => (None, 1),
                    (Some(b2), _, _) if bad_second(b2) => (None, 1),
                    (Some(_), Some(b3), _) if !cont(b3) => (None, 2),
                    (Some(_), Some(_), Some(b4)) if !cont(b4) => (None, 3),
                    (Some(b2), Some(b3), Some(b4)) => {
                        let c = (u32::from(b1 & 0x07) << 18)
                            | (u32::from(b2 & 0x3F) << 12)
                            | (u32::from(b3 & 0x3F) << 6)
                            | u32::from(b4 & 0x3F);
                        // The lead/second-byte checks above already keep c
                        // within U+10000..=U+10FFFF.
                        (char::from_u32(c), 4)
                    }
                    _ => (None, rest.len()),
                }
            }
            // Continuation bytes, C0/C1 and F8..FF.
            _ => (None, 1),
        };
        out.push(decoded.unwrap_or('\u{FFFD}'));
        i += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> Uri {
        parse(s).unwrap_or_else(|_| panic!("{s} should parse"))
    }

    #[test]
    fn server_authority_exposes_user_info_host_and_port() {
        let u = ok("pekko://a%20b@h:1/p?q#f");
        assert_eq!(u.scheme.as_deref(), Some("pekko"));
        assert_eq!(u.user_info().as_deref(), Some("a b"));
        assert_eq!((u.host.as_deref(), u.port), (Some("h"), 1));
        assert_eq!(u.raw_path.as_deref(), Some("/p"));
        assert_eq!(u.raw_fragment.as_deref(), Some("f"));
    }

    #[test]
    fn invalid_server_falls_back_to_registry_without_host() {
        for s in [
            "pekko://my_sys",
            "pekko://sys@in_valid:1",
            "pekko://sys@256.1.1.1:1",
            "pekko://sys@-h:1",
        ] {
            let u = ok(s);
            assert_eq!(
                (u.host.as_deref(), u.user_info(), u.port),
                (None, None, -1),
                "{s}"
            );
        }
    }

    #[test]
    fn port_beyond_i32_falls_back_to_registry() {
        let u = ok("pekko://sys@host:2147483648");
        assert_eq!((u.host, u.port), (None, -1));
    }

    #[test]
    fn opaque_uri_has_no_path() {
        assert_eq!(ok("pekko:sys").raw_path, None);
    }

    #[test]
    fn syntax_errors() {
        for s in [
            "://hallo",
            "pekko://",
            "pekko://s y",
            "pekko://a%2@h:1",
            "pekko://sys@[::1",
            "pekko://sys@[1::2::3]:1",
            "a b",
            "a%2",
            "foo#1#2",
            "pekko://sys/a# 5",
            "1pekko://sys",
        ] {
            assert!(parse(s).is_err(), "{s:?} should fail");
        }
    }

    #[test]
    fn ipv6_literals_keep_brackets_and_zone() {
        assert_eq!(ok("p://s@[::1]:1").host.as_deref(), Some("[::1]"));
        assert_eq!(
            ok("p://s@[fe80::1%25eth0]:1").host.as_deref(),
            Some("[fe80::1%25eth0]")
        );
        assert_eq!(
            ok("p://s@[::FFFF:1.2.3.4]:1").host.as_deref(),
            Some("[::FFFF:1.2.3.4]")
        );
    }

    #[test]
    fn decode_replaces_malformed_utf8() {
        assert_eq!(decode("a%C3%BC"), "a\u{FC}");
        assert_eq!(decode("a%FF"), "a\u{FFFD}");
    }

    /// Malformed-input lengths of the JDK UTF-8 decoder, as probed on Java 21
    /// through `new URI("pekko://" + escapes + "@h:1").getUserInfo()`.
    #[test]
    fn decode_replaces_malformed_utf8_like_the_jdk_decoder() {
        let r = '\u{FFFD}';
        for (escaped, expected) in [
            ("%ED%A0%80", format!("{r}")),
            ("%ED%A0%80%ED%B0%80", format!("{r}{r}")),
            ("%ED%A0%41", format!("{r}A")),
            ("%ED%A0", format!("{r}")),
            ("%C0%80", format!("{r}{r}")),
            ("%E0%80%80", format!("{r}{r}{r}")),
            ("%E0%80", format!("{r}{r}")),
            ("%E2%82", format!("{r}")),
            ("%E2%82%41", format!("{r}A")),
            ("%F0%9F%98", format!("{r}")),
            ("%F0%9F%41", format!("{r}A")),
            ("%F0%90%80%41", format!("{r}A")),
            ("%F4%90%80%80", format!("{r}{r}{r}{r}")),
            ("%F5%80", format!("{r}{r}")),
            ("%F8%88%80%80%80", format!("{r}{r}{r}{r}{r}")),
            ("%F0%9F%98%80%80", format!("\u{1F600}{r}")),
        ] {
            assert_eq!(decode(escaped), expected, "{escaped}");
        }
    }
}
