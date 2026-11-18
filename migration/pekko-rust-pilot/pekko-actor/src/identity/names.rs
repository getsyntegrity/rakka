//! Actor naming rules: `ActorPath.validatePathElement` and `Helpers.base64`.

use crate::error::{ActorError, ErrorClass};
use crate::jstr;

/// `ActorPath.ValidSymbols`.
const VALID_SYMBOLS: &str = "-_.*$+:@&=,!~';";
const VALID_PATH_CODE: i64 = -1;
const EMPTY_PATH_CODE: i64 = -2;

fn is_valid_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || VALID_SYMBOLS.as_bytes().contains(&b)
}

/// `findInvalidPathElementCharPosition`: `-1` valid, `-2` empty, otherwise the UTF-16 position.
/// Characters outside Latin-1 count as invalid (the JDK encodes them as `?`); positions are kept.
fn find_invalid_position(s: &[u16]) -> i64 {
    if s.is_empty() {
        return EMPTY_PATH_CODE;
    }
    // ISO_8859_1 bytes: chars above 0xFF become '?' (invalid), 0x80..=0xFF are "negative" (invalid).
    let flag = |pos: usize| -> (bool, bool, u8) {
        let c = s[pos];
        if c > 0x7f {
            (false, false, b'?')
        } else {
            let b = c as u8;
            (is_valid_char(b), b.is_ascii_hexdigit(), b)
        }
    };
    let len = s.len();
    let mut pos = if s[0] != '$' as u16 { 0 } else { return 0 };
    loop {
        if pos >= len {
            return VALID_PATH_CODE;
        }
        let (valid, _, b) = flag(pos);
        if valid {
            pos += 1;
        } else if b == b'%' && s[pos] == '%' as u16 && pos + 2 < len && flag(pos + 1).1 && flag(pos + 2).1 {
            pos += 3;
        } else {
            return pos as i64;
        }
    }
}

/// `ActorPath.validatePathElement(element)` (no full path).
pub fn validate_path_element(element: &str) -> Result<(), ActorError> {
    let units = jstr(element);
    match find_invalid_position(&units) {
        VALID_PATH_CODE => Ok(()),
        EMPTY_PATH_CODE => Err(ActorError::new(ErrorClass::InvalidActorName, "Actor path element must not be empty ")),
        at => {
            let at = at as usize;
            let mut message = jstr("Invalid actor path element [");
            message.extend_from_slice(&units);
            message.extend(jstr("], illegal character ["));
            message.push(units[at]);
            message.extend(jstr(&format!(
                "] at position: {at}. Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: {VALID_SYMBOLS}."
            )));
            Err(ActorError { class: ErrorClass::InvalidActorName, message })
        }
    }
}

const BASE64_CHARS: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+~";

/// `Helpers.base64(l, new StringBuilder(prefix))`: the anonymous-actor name generator (`$a`, `$b`, ...).
pub fn base64(mut l: u64, prefix: &str) -> String {
    let mut s = String::from(prefix);
    loop {
        s.push(BASE64_CHARS[(l & 63) as usize] as char);
        l >>= 6;
        if l == 0 {
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(name: &str) -> String {
        String::from_utf16_lossy(&validate_path_element(name).unwrap_err().message)
    }

    #[test]
    fn accepts_symbols_and_escapes() {
        for ok in ["a", "A1", "a-b_c.d*e$f+g:h@i&j=k,l!m~n'o;p", "%41", "a%41b", "-", ".."] {
            assert!(validate_path_element(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn rejects_leading_dollar_and_reports_position() {
        assert!(msg("$a").starts_with("Invalid actor path element [$a], illegal character [$] at position: 0. "));
        assert!(msg("a b").contains("illegal character [ ] at position: 1."));
        assert!(msg("a%4").contains("illegal character [%] at position: 1."));
        assert!(msg("").starts_with("Actor path element must not be empty"));
    }

    #[test]
    fn supplementary_chars_are_reported_per_utf16_unit() {
        let e = validate_path_element("a\u{1F600}").unwrap_err();
        assert!(e.message.windows(1).any(|w| w[0] == 0xD83D));
        assert!(String::from_utf16_lossy(&e.message).contains("at position: 1."));
    }

    #[test]
    fn anonymous_names_follow_helpers_base64() {
        assert_eq!(base64(0, "$"), "$a");
        assert_eq!(base64(25, "$"), "$z");
        assert_eq!(base64(26, "$"), "$A");
        assert_eq!(base64(62, "$"), "$+");
        assert_eq!(base64(63, "$"), "$~");
        assert_eq!(base64(64, "$"), "$ab"); // low 6 bits first, then the next group
    }
}
