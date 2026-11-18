//! A port of the parser inside `java.net.URI(String)` from JDK 17.0.21 (RFC 2396 with the JDK's
//! deviations), exposing only what Pekko's `ActorPathExtractor` reads: scheme, user info, host,
//! port, raw path and raw fragment.
//!
//! The structure and the character-class masks mirror `java/net/URI.java` one to one so that the
//! two can be compared side by side; positions are UTF-16 code-unit indices.

use crate::identity::jdk_int::parse_int_range;
use crate::identity::jdk_tables::{ISO_CONTROL_HIGH, SPACE_CHAR_RANGES};
use crate::{jstr, JStr};

/// What can go wrong while parsing, mirroring the two exceptions that can escape `new URI(s)`.
#[derive(Debug, PartialEq, Eq)]
pub enum UriError {
    /// `java.net.URISyntaxException` (reason is only kept for debugging; Pekko swallows it).
    Syntax(&'static str),
    /// `java.lang.NumberFormatException` escaping from the IPv6/IPv4 literal scan, with its message.
    NumberFormat(JStr),
}

type R<T> = Result<T, UriError>;

fn syntax<T>(reason: &'static str) -> R<T> {
    Err(UriError::Syntax(reason))
}

/// Parsed components, as returned by the corresponding `URI` getters.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Uri {
    pub scheme: Option<JStr>,
    /// `getRawUserInfo()`; use [`Uri::user_info`] for the decoded `getUserInfo()`.
    pub raw_user_info: Option<JStr>,
    pub host: Option<JStr>,
    /// `getPort()`, `-1` when undefined.
    pub port: i32,
    /// `getRawPath()`; `None` for opaque URIs.
    pub raw_path: Option<JStr>,
    pub raw_fragment: Option<JStr>,
}

impl Uri {
    /// `new URI(s)`.
    pub fn parse(s: &[u16]) -> Result<Uri, UriError> {
        let mut p = Parser::new(s);
        p.parse()?;
        Ok(Uri {
            scheme: p.scheme,
            raw_user_info: p.user_info,
            host: p.host,
            port: p.port,
            raw_path: p.path,
            raw_fragment: p.fragment,
        })
    }

    /// `getUserInfo()`: the user info with `%XX` escapes decoded as UTF-8.
    pub fn user_info(&self) -> Option<JStr> {
        self.raw_user_info.as_deref().map(decode)
    }
}

// ---- character classes (long masks copied from URI.java) ----

#[derive(Clone, Copy)]
struct Mask(u64, u64);

const fn or(a: Mask, b: Mask) -> Mask {
    Mask(a.0 | b.0, a.1 | b.1)
}

const DIGIT: Mask = Mask(0x3FF000000000000, 0);
const ALPHA: Mask = Mask(0, 0x7FFFFFE00000000 | 0x7FFFFFE);
const ALPHANUM: Mask = or(DIGIT, ALPHA);
const HEX: Mask = Mask(DIGIT.0, 0x7E0000007E);
const MARK: Mask = Mask(0x678200000000, 0x4000000080000000);
const UNRESERVED: Mask = or(ALPHANUM, MARK);
const RESERVED: Mask = Mask(0xAC00985000000000, 0x28000001);
const ESCAPED: Mask = Mask(1, 0);
const URIC: Mask = or(or(RESERVED, UNRESERVED), ESCAPED);
const PCHAR: Mask = or(or(UNRESERVED, ESCAPED), Mask(0x2400185000000000, 0x1));
const PATH: Mask = or(PCHAR, Mask(0x800800000000000, 0));
const DASH: Mask = Mask(0x200000000000, 0);
const DOT: Mask = Mask(0x400000000000, 0);
const USERINFO: Mask = or(or(UNRESERVED, ESCAPED), Mask(0x2C00185000000000, 0));
const REG_NAME: Mask = or(or(UNRESERVED, ESCAPED), Mask(0x2C00185000000000, 0x1));
const SERVER: Mask = or(or(or(USERINFO, ALPHANUM), DASH), Mask(0x400400000000000, 0x28000001));
const SERVER_PERCENT: Mask = or(SERVER, Mask(0x2000000000, 0));
const SCHEME: Mask = or(or(ALPHA, DIGIT), Mask(0x680000000000, 0));
const SCOPE_ID: Mask = or(ALPHANUM, Mask(0x400000000000, 0x80000000));

fn matches(c: u16, m: Mask) -> bool {
    if c == 0 {
        return false;
    }
    if c < 64 {
        return (1u64 << c) & m.0 != 0;
    }
    if c < 128 {
        return (1u64 << (c - 64)) & m.1 != 0;
    }
    false
}

fn is_space_char(c: u16) -> bool {
    SPACE_CHAR_RANGES.iter().any(|&(lo, hi)| c >= lo && c <= hi)
}

fn is_iso_control_high(c: u16) -> bool {
    c >= ISO_CONTROL_HIGH.0 && c <= ISO_CONTROL_HIGH.1
}

// ---- percent-decoding (URI.decode) ----

fn hex_val(c: u16) -> u8 {
    match c as u8 as char {
        '0'..='9' => c as u8 - b'0',
        'a'..='f' => c as u8 - b'a' + 10,
        _ => c as u8 - b'A' + 10,
    }
}

/// `URI.decode(s)`: evaluates `%XX` escapes, decoding runs of octets as UTF-8 and replacing
/// malformed input with U+FFFD. Assumes escapes are well formed (the parser guarantees it).
fn decode(s: &[u16]) -> JStr {
    let n = s.len();
    if !s.contains(&('%' as u16)) {
        return s.to_vec();
    }
    let mut out: JStr = Vec::with_capacity(n);
    let mut i = 0;
    let mut between_brackets = false;
    while i < n {
        let c = s[i];
        if c == '[' as u16 {
            between_brackets = true;
        } else if between_brackets && c == ']' as u16 {
            between_brackets = false;
        }
        if c != '%' as u16 || between_brackets {
            out.push(c);
            i += 1;
            continue;
        }
        let mut bytes = Vec::new();
        loop {
            bytes.push(hex_val(s[i + 1]) << 4 | hex_val(s[i + 2]));
            i += 3;
            if i >= n || s[i] != '%' as u16 {
                break;
            }
        }
        out.extend(decode_utf8_jdk(&bytes));
    }
    out
}

fn not_continuation(b: i32) -> bool {
    b & 0xc0 != 0x80
}

/// `sun.nio.cs.UTF_8.Decoder` with `CodingErrorAction.REPLACE` over a complete input
/// (`decode(.., endOfInput = true)`), including the JDK's own malformed-sequence lengths:
/// that is what `URI.decode` uses, and it differs from Unicode's "maximal subpart" practice
/// (e.g. `ED A0 80` is one U+FFFD here, not three).
fn decode_utf8_jdk(bytes: &[u8]) -> JStr {
    const REPLACEMENT: u16 = 0xFFFD;
    let b: Vec<i32> = bytes.iter().map(|&x| x as i8 as i32).collect();
    let sl = b.len();
    let mut out: JStr = Vec::with_capacity(sl);
    let mut sp = 0;
    while sp < sl {
        let b1 = b[sp];
        let rem = sl - sp;
        // number of bytes consumed by a single replacement
        let consumed: usize;
        if b1 >= 0 {
            out.push(b1 as u16);
            sp += 1;
            continue;
        } else if (b1 >> 5) == -2 && (b1 & 0x1e) != 0 {
            // 2 bytes: [C2..DF] [80..BF]
            if rem < 2 {
                consumed = rem; // underflow at end of input: the rest is malformed
            } else if not_continuation(b[sp + 1]) {
                consumed = 1;
            } else {
                out.push((((b1 & 0x1f) << 6) | (b[sp + 1] & 0x3f)) as u16);
                sp += 2;
                continue;
            }
        } else if (b1 >> 4) == -2 {
            // 3 bytes: [E0][A0..BF][80..BF] | [E1..EF][80..BF][80..BF]
            let malformed3_2 = |b2: i32| (b1 == 0xe0u8 as i8 as i32 && (b2 & 0xe0) == 0x80) || not_continuation(b2);
            if rem < 3 {
                consumed = if rem > 1 && malformed3_2(b[sp + 1]) { 1 } else { rem };
            } else {
                let (b2, b3) = (b[sp + 1], b[sp + 2]);
                if malformed3_2(b2) || not_continuation(b3) {
                    // malformedN(3): looks only at b1, b2
                    consumed = if malformed3_2(b2) { 1 } else { 2 };
                } else {
                    let c = (((b1 & 0x0f) << 12) | ((b2 & 0x3f) << 6) | (b3 & 0x3f)) as u16;
                    if (0xD800..=0xDFFF).contains(&c) {
                        consumed = 3;
                    } else {
                        out.push(c);
                        sp += 3;
                        continue;
                    }
                }
            }
        } else if (b1 >> 3) == -2 {
            // 4 bytes: [F0][90..BF][80..BF][80..BF] | [F1..F3][80..BF]{3} | [F4][80..8F][80..BF]{2}
            let u1 = b1 & 0xff;
            let malformed4_2 = |b2: i32| {
                let b2 = b2 & 0xff;
                (u1 == 0xf0 && !(0x90..=0xbf).contains(&b2)) || (u1 == 0xf4 && (b2 & 0xf0) != 0x80) || (b2 & 0xc0) != 0x80
            };
            if rem < 4 {
                if u1 > 0xf4 || (rem > 1 && malformed4_2(b[sp + 1])) {
                    consumed = 1;
                } else if rem > 2 && not_continuation(b[sp + 2]) {
                    consumed = 2;
                } else {
                    consumed = rem;
                }
            } else {
                let (b2, b3, b4) = (b[sp + 1], b[sp + 2], b[sp + 3]);
                let uc = ((b1 & 0x07) << 18) | ((b2 & 0x3f) << 12) | ((b3 & 0x3f) << 6) | (b4 & 0x3f);
                if not_continuation(b2) || not_continuation(b3) || not_continuation(b4) || !(0x10000..=0x10FFFF).contains(&uc) {
                    // malformedN(4)
                    consumed = if u1 > 0xf4 || (u1 == 0xf0 && !(0x90..=0xbf).contains(&(b2 & 0xff)))
                        || (u1 == 0xf4 && (b2 & 0xf0) != 0x80) || not_continuation(b2)
                    {
                        1
                    } else if not_continuation(b3) {
                        2
                    } else {
                        3
                    };
                } else {
                    let cp = uc as u32 - 0x10000;
                    out.push(0xD800 + (cp >> 10) as u16);
                    out.push(0xDC00 + (cp & 0x3ff) as u16);
                    sp += 4;
                    continue;
                }
            }
        } else {
            consumed = 1; // C0, C1, 80..BF, F8..FF
        }
        out.push(REPLACEMENT);
        sp += consumed;
    }
    out
}

// ---- the parser ----

struct Parser<'a> {
    s: &'a [u16],
    scheme: Option<JStr>,
    user_info: Option<JStr>,
    host: Option<JStr>,
    port: i32,
    path: Option<JStr>,
    fragment: Option<JStr>,
    ipv6_byte_count: i32,
}

impl<'a> Parser<'a> {
    fn new(s: &'a [u16]) -> Self {
        Parser {
            s,
            scheme: None,
            user_info: None,
            host: None,
            port: -1,
            path: None,
            fragment: None,
            ipv6_byte_count: 0,
        }
    }

    fn sub(&self, from: usize, to: usize) -> JStr {
        self.s[from..to].to_vec()
    }

    fn at(&self, start: usize, end: usize, c: char) -> bool {
        start < end && self.s[start] == c as u16
    }

    fn at_str(&self, start: usize, end: usize, lit: &str) -> bool {
        let lit = jstr(lit);
        if lit.len() as isize > end as isize - start as isize {
            return false;
        }
        self.s[start..start + lit.len()] == lit[..]
    }

    /// `scan(start, end, char c)`
    fn scan_char(&self, start: usize, end: usize, c: char) -> usize {
        if self.at(start, end, c) { start + 1 } else { start }
    }

    /// `scan(start, end, err, stop)`: `None` stands for -1.
    fn scan_err_stop(&self, start: usize, end: usize, err: &str, stop: &str) -> Option<usize> {
        let mut p = start;
        while p < end {
            let c = self.s[p];
            if is_in(err, c) {
                return None;
            }
            if is_in(stop, c) {
                break;
            }
            p += 1;
        }
        Some(p)
    }

    /// `scan(start, end, stop)`
    fn scan_stop(&self, start: usize, end: usize, stop: &str) -> usize {
        let mut p = start;
        while p < end && !is_in(stop, self.s[p]) {
            p += 1;
        }
        p
    }

    fn scan_escape(&self, start: usize, n: usize, first: u16) -> R<usize> {
        if first == '%' as u16 {
            if start + 3 <= n && matches(self.s[start + 1], HEX) && matches(self.s[start + 2], HEX) {
                return Ok(start + 3);
            }
            return syntax("Malformed escape pair");
        } else if first > 128 && !is_space_char(first) && !is_iso_control_high(first) {
            return Ok(start + 1);
        }
        Ok(start)
    }

    /// `scan(start, n, lowMask, highMask)`
    fn scan_mask(&self, start: usize, n: usize, m: Mask) -> R<usize> {
        let mut p = start;
        while p < n {
            let c = self.s[p];
            if matches(c, m) {
                p += 1;
                continue;
            }
            if m.0 & ESCAPED.0 != 0 {
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

    fn check_chars(&self, start: usize, end: usize, m: Mask, _what: &'static str) -> R<()> {
        let p = self.scan_mask(start, end, m)?;
        if p < end { syntax("Illegal character") } else { Ok(()) }
    }

    fn check_char(&self, p: usize, m: Mask, what: &'static str) -> R<()> {
        self.check_chars(p, p + 1, m, what)
    }

    // [<scheme>:]<scheme-specific-part>[#<fragment>]
    fn parse(&mut self) -> R<()> {
        let n = self.s.len();
        let mut p;
        let colon = self.scan_err_stop(0, n, "/?#", ":");
        match colon {
            Some(c) if self.at(c, n, ':') => {
                p = c;
                if p == 0 {
                    return syntax("Expected scheme name");
                }
                self.check_char(0, ALPHA, "scheme name")?;
                self.check_chars(1, p, SCHEME, "scheme name")?;
                self.scheme = Some(self.sub(0, p));
                p += 1;
                if self.at(p, n, '/') {
                    p = self.parse_hierarchical(p, n)?;
                } else {
                    let q = self.scan_stop(p, n, "#");
                    if q <= p {
                        return syntax("Expected scheme-specific part");
                    }
                    self.check_chars(p, q, URIC, "opaque part")?;
                    p = q;
                }
            }
            _ => p = self.parse_hierarchical(0, n)?,
        }
        if self.at(p, n, '#') {
            self.check_chars(p + 1, n, URIC, "fragment")?;
            self.fragment = Some(self.sub(p + 1, n));
            p = n;
        }
        if p < n {
            return syntax("end of URI");
        }
        Ok(())
    }

    // [//authority]<path>[?<query>]
    fn parse_hierarchical(&mut self, start: usize, n: usize) -> R<usize> {
        let mut p = start;
        if self.at(p, n, '/') && self.at(p + 1, n, '/') {
            p += 2;
            let q = self.scan_stop(p, n, "/?#");
            if q > p {
                p = self.parse_authority(p, q)?;
            } else if q < n {
                // DEVIATION: allow empty authority before a non-empty path, query or fragment
            } else {
                return syntax("Expected authority");
            }
        }
        let mut q = self.scan_stop(p, n, "?#");
        self.check_chars(p, q, PATH, "path")?;
        self.path = Some(self.sub(p, q));
        p = q;
        if self.at(p, n, '?') {
            p += 1;
            q = self.scan_stop(p, n, "#");
            self.check_chars(p, q, URIC, "query")?;
            p = q;
        }
        Ok(p)
    }

    // authority = server | reg_name
    fn parse_authority(&mut self, start: usize, n: usize) -> R<usize> {
        let p = start;
        let mut q = p;
        let mut ex: Option<UriError> = None;

        let server_chars = if self.scan_stop(p, n, "]") > p {
            self.scan_mask(p, n, SERVER_PERCENT)? == n
        } else {
            self.scan_mask(p, n, SERVER)? == n
        };
        let reg_chars = self.scan_mask(p, n, REG_NAME)? == n;

        if reg_chars && !server_chars {
            return Ok(n); // registry-based authority: host/userInfo/port stay undefined
        }

        // requireServerAuthority is always false for `new URI(String)`
        let skip_parse_exception = reg_chars;
        if server_chars {
            match self.parse_server(p, n, skip_parse_exception) {
                Ok(parsed_to) => {
                    q = parsed_to;
                    if q < n {
                        if skip_parse_exception {
                            self.user_info = None;
                            self.host = None;
                            self.port = -1;
                            q = p;
                        } else {
                            // failExpecting("end of authority", q), caught right below in Java
                            self.user_info = None;
                            self.host = None;
                            self.port = -1;
                            ex = Some(UriError::Syntax("Expected end of authority"));
                            q = p;
                        }
                    }
                }
                Err(UriError::Syntax(r)) => {
                    self.user_info = None;
                    self.host = None;
                    self.port = -1;
                    ex = Some(UriError::Syntax(r));
                    q = p;
                }
                Err(e @ UriError::NumberFormat(_)) => return Err(e),
            }
        }

        if q < n {
            if reg_chars {
                // registry-based authority
            } else if let Some(e) = ex {
                return Err(e);
            } else {
                return syntax("Illegal character in authority");
            }
        }
        Ok(n)
    }

    // [<userinfo>@]<host>[:<port>]
    fn parse_server(&mut self, start: usize, n: usize, skip_parse_exception: bool) -> R<usize> {
        let mut p = start;

        // userinfo
        if let Some(q) = self.scan_err_stop(p, n, "/?#", "@")
            && q >= p
            && self.at(q, n, '@')
        {
            self.check_chars(p, q, USERINFO, "user info")?;
            self.user_info = Some(self.sub(p, q));
            p = q + 1;
        }

        // hostname, IPv4 address, or IPv6 address
        if self.at(p, n, '[') {
            p += 1;
            match self.scan_err_stop(p, n, "/?#", "]") {
                Some(q) if q > p && self.at(q, n, ']') => {
                    let r = self.scan_stop(p, q, "%");
                    if r > p {
                        self.parse_ipv6_reference(p, r)?;
                        if r + 1 == q {
                            return syntax("scope id expected");
                        }
                        self.check_chars(r + 1, q, SCOPE_ID, "scope id")?;
                    } else {
                        self.parse_ipv6_reference(p, q)?;
                    }
                    self.host = Some(self.sub(p - 1, q + 1));
                    p = q + 1;
                }
                _ => return syntax("Expected closing bracket for IPv6 address"),
            }
        } else {
            let q = self.parse_ipv4_address(p, n);
            let q = if q <= p as isize {
                self.parse_hostname(p, n, skip_parse_exception)?
            } else {
                q as usize
            };
            p = q;
        }

        // port
        if self.at(p, n, ':') {
            p += 1;
            let q = self.scan_stop(p, n, "/");
            if q > p {
                self.check_chars(p, q, DIGIT, "port number")?;
                match parse_int_range(self.s, p, q) {
                    Ok(v) => self.port = v,
                    Err(_) => return syntax("Malformed port number"),
                }
                p = q;
            }
        } else if p < n && skip_parse_exception {
            return Ok(p);
        }

        if p < n {
            return syntax("Expected port number");
        }
        Ok(p)
    }

    /// Scans decimal digits whose value fits in a byte.
    fn scan_byte(&self, start: usize, n: usize) -> R<usize> {
        let p = start;
        let q = self.scan_mask(p, n, DIGIT)?;
        if q <= p {
            return Ok(q);
        }
        match parse_int_range(self.s, p, q) {
            Ok(v) if v > 255 => Ok(p),
            Ok(_) => Ok(q),
            Err(m) => Err(UriError::NumberFormat(m)),
        }
    }

    /// `scanIPv4Address`: `Ok(None)` stands for the `-1` return.
    fn scan_ipv4_address(&self, start: usize, n: usize, strict: bool) -> R<Option<usize>> {
        let mut p = start;
        let m = self.scan_mask(p, n, or(DIGIT, DOT))?;
        if m <= p || (strict && m != n) {
            return Ok(None);
        }
        let mut q;
        'scan: {
            q = self.scan_byte(p, m)?;
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_char(p, m, '.');
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_char(p, m, '.');
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_char(p, m, '.');
            if q <= p { break 'scan; }
            p = q;
            q = self.scan_byte(p, m)?;
            if q <= p { break 'scan; }
            if q < m { break 'scan; }
            return Ok(Some(q));
        }
        syntax("Malformed IPv4 address")
    }

    fn take_ipv4_address(&self, start: usize, n: usize) -> R<usize> {
        match self.scan_ipv4_address(start, n, true)? {
            Some(p) if p > start => Ok(p),
            _ => syntax("Expected IPv4 address"),
        }
    }

    /// `parseIPv4Address`: returns -1 on failure (any exception is swallowed).
    fn parse_ipv4_address(&mut self, start: usize, n: usize) -> isize {
        let mut p: isize = match self.scan_ipv4_address(start, n, false) {
            Ok(Some(p)) => p as isize,
            Ok(None) => -1,
            Err(_) => return -1,
        };
        if p > start as isize && (p as usize) < n && self.s[p as usize] != ':' as u16 {
            p = -1;
        }
        if p > start as isize {
            self.host = Some(self.sub(start, p as usize));
        }
        p
    }

    // hostname = domainlabel [ "." ] | 1*( domainlabel "." ) toplabel [ "." ]
    fn parse_hostname(&mut self, start: usize, n: usize, skip_parse_exception: bool) -> R<usize> {
        let mut p = start;
        let mut l: isize = -1;
        loop {
            let q = self.scan_mask(p, n, ALPHANUM)?;
            if q <= p {
                break;
            }
            l = p as isize;
            p = q;
            let q = self.scan_mask(p, n, or(ALPHANUM, DASH))?;
            if q > p {
                if self.s[q - 1] == '-' as u16 {
                    return syntax("Illegal character in hostname");
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
            return syntax("Illegal character in hostname");
        }
        if l < 0 {
            return syntax("Expected hostname");
        }
        let l = l as usize;
        if l > start && !matches(self.s[l], ALPHA) {
            return syntax("Illegal character in hostname");
        }
        self.host = Some(self.sub(start, p));
        Ok(p)
    }

    // IPv6address = hexseq [ ":" IPv4address ] | hexseq [ "::" [ hexpost ] ] | "::" [ hexpost ]
    fn parse_ipv6_reference(&mut self, start: usize, n: usize) -> R<usize> {
        let mut p = start;
        let mut compressed_zeros = false;

        let q = self.scan_hex_seq(p, n)?;
        if let Some(q) = q.filter(|&q| q > p) {
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
        if p < n {
            return syntax("Malformed IPv6 address");
        }
        if self.ipv6_byte_count > 16 {
            return syntax("IPv6 address too long");
        }
        if !compressed_zeros && self.ipv6_byte_count < 16 {
            return syntax("IPv6 address too short");
        }
        if compressed_zeros && self.ipv6_byte_count == 16 {
            return syntax("Malformed IPv6 address");
        }
        Ok(p)
    }

    fn scan_hex_post(&mut self, start: usize, n: usize) -> R<usize> {
        let mut p = start;
        if p == n {
            return Ok(p);
        }
        match self.scan_hex_seq(p, n)? {
            Some(q) if q > p => {
                p = q;
                if self.at(p, n, ':') {
                    p += 1;
                    p = self.take_ipv4_address(p, n)?;
                    self.ipv6_byte_count += 4;
                }
            }
            _ => {
                p = self.take_ipv4_address(p, n)?;
                self.ipv6_byte_count += 4;
            }
        }
        Ok(p)
    }

    /// Scans a hex sequence; `None` stands for -1 ("could not be scanned").
    fn scan_hex_seq(&mut self, start: usize, n: usize) -> R<Option<usize>> {
        let mut p = start;
        let q = self.scan_mask(p, n, HEX)?;
        if q <= p {
            return Ok(None);
        }
        if self.at(q, n, '.') {
            return Ok(None);
        }
        if q > p + 4 {
            return syntax("IPv6 hexadecimal digit sequence too long");
        }
        self.ipv6_byte_count += 2;
        p = q;
        while p < n {
            if !self.at(p, n, ':') {
                break;
            }
            if self.at(p + 1, n, ':') {
                break;
            }
            p += 1;
            let q = self.scan_mask(p, n, HEX)?;
            if q <= p {
                return syntax("Expected digits for an IPv6 address");
            }
            if self.at(q, n, '.') {
                p -= 1;
                break;
            }
            if q > p + 4 {
                return syntax("IPv6 hexadecimal digit sequence too long");
            }
            self.ipv6_byte_count += 2;
            p = q;
        }
        Ok(Some(p))
    }
}

fn is_in(set: &str, c: u16) -> bool {
    c < 128 && set.as_bytes().contains(&(c as u8))
}
