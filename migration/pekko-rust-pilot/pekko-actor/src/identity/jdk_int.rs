//! `Integer.parseInt` as in JDK 17, including its exact `NumberFormatException` messages.

use crate::identity::jdk_tables::DIGIT_ZEROS;
use crate::{jstr, JStr};

/// `Character.digit(c, 10)` for a UTF-16 code unit.
pub fn digit10(c: u16) -> Option<i32> {
    DIGIT_ZEROS
        .iter()
        .find(|&&zero| c >= zero && c < zero + 10)
        .map(|&zero| (c - zero) as i32)
}

fn for_input_string(s: &[u16]) -> JStr {
    let mut m = jstr("For input string: \"");
    m.extend_from_slice(s);
    m.push('"' as u16);
    m
}

fn for_char_sequence(s: &[u16], begin: usize, end: usize, error_index: usize) -> JStr {
    let mut m = jstr(&format!("Error at index {} in: \"", error_index - begin));
    m.extend_from_slice(&s[begin..end]);
    m.push('"' as u16);
    m
}

/// Accumulates digits negatively like the JDK; `Err(i)` is the index where it gave up.
fn accumulate(s: &[u16], mut i: usize, end: usize, limit: i32) -> Result<i32, usize> {
    let multmin = limit / 10;
    let mut result: i32 = 0;
    while i < end {
        let digit = match digit10(s[i]) {
            Some(d) if result >= multmin => d,
            _ => return Err(i),
        };
        result *= 10;
        if result < limit + digit {
            return Err(i);
        }
        i += 1;
        result -= digit;
    }
    Ok(result)
}

/// `Integer.parseInt(String, 10)` (what `Integer.valueOf(String)` uses).
/// The error is the `NumberFormatException` message.
pub fn parse_int(s: &[u16]) -> Result<i32, JStr> {
    let len = s.len();
    if len == 0 {
        return Err(for_input_string(s));
    }
    let mut negative = false;
    let mut limit = -i32::MAX;
    let mut i = 0;
    if s[0] < '0' as u16 {
        if s[0] == '-' as u16 {
            negative = true;
            limit = i32::MIN;
        } else if s[0] != '+' as u16 {
            return Err(for_input_string(s));
        }
        if len == 1 {
            return Err(for_input_string(s));
        }
        i += 1;
    }
    let r = accumulate(s, i, len, limit).map_err(|_| for_input_string(s))?;
    Ok(if negative { r } else { -r })
}

/// `Integer.parseInt(CharSequence, begin, end, 10)` (what `java.net.URI` uses for ports and bytes).
pub fn parse_int_range(s: &[u16], begin: usize, end: usize) -> Result<i32, JStr> {
    if begin >= end {
        return Err(for_input_string(&[]));
    }
    let mut negative = false;
    let mut limit = -i32::MAX;
    let mut i = begin;
    if s[i] < '0' as u16 {
        if s[i] == '-' as u16 {
            negative = true;
            limit = i32::MIN;
        } else if s[i] != '+' as u16 {
            return Err(for_char_sequence(s, begin, end, i));
        }
        i += 1;
        if i == end {
            return Err(for_char_sequence(s, begin, end, i));
        }
    }
    let r = accumulate(s, i, end, limit).map_err(|at| for_char_sequence(s, begin, end, at))?;
    Ok(if negative { r } else { -r })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(s: &str, begin: usize, end: usize) -> String {
        String::from_utf16(&parse_int_range(&jstr(s), begin, end).unwrap_err()).unwrap()
    }

    #[test]
    fn range_accepts_signs_and_extremes_like_integer_parse_int() {
        assert_eq!(parse_int_range(&jstr("+5"), 0, 2), Ok(5));
        assert_eq!(parse_int_range(&jstr("-5"), 0, 2), Ok(-5));
        assert_eq!(parse_int_range(&jstr("-2147483648"), 0, 11), Ok(i32::MIN));
        assert_eq!(parse_int_range(&jstr("ab-7cd"), 2, 4), Ok(-7));
    }

    #[test]
    fn range_errors_use_the_jdk_messages() {
        assert_eq!(err("", 0, 0), "For input string: \"\"");
        assert_eq!(err("+", 0, 1), "Error at index 1 in: \"+\"");
        assert_eq!(err("-", 0, 1), "Error at index 1 in: \"-\"");
        assert_eq!(err("x", 0, 1), "Error at index 0 in: \"x\"");
        assert_eq!(err("ab12x", 2, 5), "Error at index 2 in: \"12x\"");
        assert_eq!(err("2147483648", 0, 10), "Error at index 9 in: \"2147483648\"");
    }

    #[test]
    fn parse_int_matches_range_on_whole_strings() {
        assert_eq!(parse_int(&jstr("+42")), Ok(42));
        assert_eq!(parse_int(&jstr("-2147483648")), Ok(i32::MIN));
        assert_eq!(String::from_utf16(&parse_int(&jstr("")).unwrap_err()).unwrap(), "For input string: \"\"");
        assert_eq!(String::from_utf16(&parse_int(&jstr("-")).unwrap_err()).unwrap(), "For input string: \"-\"");
        assert_eq!(String::from_utf16(&parse_int(&jstr("1x")).unwrap_err()).unwrap(), "For input string: \"1x\"");
    }
}
