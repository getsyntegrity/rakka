// Clean-room implementation from docs/legal/cleanroom/jstring-spec.md (Apache-2.0).
//! Java-compatible string helpers: UTF-16 ordering and `Integer.valueOf` parsing.

use std::cmp::Ordering;

use super::errors::NumberFormatError;

/// Orders two strings like `java.lang.String.compareTo`: UTF-16 code unit by code unit, unsigned.
pub(crate) fn compare_utf16(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// Parses like `Integer.valueOf(String)` in base 10, accepting BMP decimal digits (category Nd).
pub(crate) fn parse_int(s: &str) -> Result<i32, NumberFormatError> {
    let fail = || NumberFormatError::new(format!("For input string: \"{s}\""));
    let units: Vec<u16> = s.encode_utf16().collect();
    let (negative, digits) = match units.split_first() {
        Some((&SIGN_MINUS, rest)) => (true, rest),
        Some((&SIGN_PLUS, rest)) => (false, rest),
        _ => (false, units.as_slice()),
    };
    if digits.is_empty() {
        return Err(fail());
    }
    // Accumulate as a negative number so that i32::MIN fits.
    let mut value: i32 = 0;
    for &unit in digits {
        let digit = decimal_digit_value(unit).ok_or_else(fail)?;
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_sub(digit))
            .ok_or_else(fail)?;
    }
    if negative {
        Ok(value)
    } else {
        value.checked_neg().ok_or_else(fail)
    }
}

/// `Character.isDigit` of a UTF-16 unit: a decimal digit (category Nd) of the Basic Multilingual Plane.
pub(crate) fn is_digit(unit: u16) -> bool {
    decimal_digit_value(unit).is_some()
}

/// `String.compareTo`: the difference of the first two UTF-16 units that differ, else of the lengths.
pub(crate) fn compare_to_raw(a: &str, b: &str) -> i32 {
    let (mut left, mut right) = (a.encode_utf16(), b.encode_utf16());
    loop {
        match (left.next(), right.next()) {
            (Some(x), Some(y)) if x != y => return i32::from(x) - i32::from(y),
            (Some(_), Some(_)) => {}
            (Some(_), None) => return 1 + left.count() as i32,
            (None, Some(_)) => return -1 - right.count() as i32,
            (None, None) => return 0,
        }
    }
}

const SIGN_MINUS: u16 = b'-' as u16;
const SIGN_PLUS: u16 = b'+' as u16;

/// First code point of each run of ten consecutive Nd (decimal digit) characters in the Basic Multilingual Plane.
/// Generated from UnicodeData.txt of Unicode 13.0.0 (https://www.unicode.org/Public/13.0.0/ucd/UnicodeData.txt):
/// lines with general category `Nd` and code point <= U+FFFF.
const DIGIT_SERIES_STARTS: [u16; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// Value 0..=9 of a UTF-16 code unit that is a decimal digit, or `None`.
fn decimal_digit_value(unit: u16) -> Option<i32> {
    DIGIT_SERIES_STARTS
        .iter()
        .find(|&&start| (start..start + 10).contains(&unit))
        .map(|&start| i32::from(unit - start))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(s: &str) -> String {
        parse_int(s).unwrap_err().message().to_string()
    }

    #[test]
    fn compare_to_raw_is_the_unit_or_length_difference() {
        assert_eq!(compare_to_raw("a", "a"), 0);
        assert_eq!(compare_to_raw("a", "B"), 31);
        assert_eq!(compare_to_raw("ab", "abcd"), -2);
        assert_eq!(compare_to_raw("abcd", "ab"), 2);
        assert_eq!(compare_to_raw("", "x"), -1);
        assert_eq!(compare_to_raw("x", ""), 1);
        assert_eq!(compare_to_raw("\u{1F600}", "\u{ffff}"), 0xD83D - 0xFFFF);
    }

    #[test]
    fn is_digit_accepts_decimal_digits_of_any_script() {
        assert!(is_digit(u16::from(b'7')));
        assert!(is_digit(0x0662));
        assert!(is_digit(0xFF15));
        assert!(!is_digit(u16::from(b'x')));
        assert!(!is_digit(u16::from(b'-')));
    }

    #[test]
    fn compare_equal_prefix_and_basic() {
        assert_eq!(compare_utf16("abc", "abc"), Ordering::Equal);
        assert_eq!(compare_utf16("ab", "abc"), Ordering::Less);
        assert_eq!(compare_utf16("abd", "abc"), Ordering::Greater);
        assert_eq!(compare_utf16("", ""), Ordering::Equal);
        assert_eq!(compare_utf16("", "a"), Ordering::Less);
    }

    #[test]
    fn compare_supplementary_is_below_high_bmp() {
        // U+10000 starts with surrogate D800 in UTF-16, below U+FF61.
        assert_eq!(compare_utf16("\u{10000}", "\u{FF61}"), Ordering::Less);
        assert_eq!(compare_utf16("\u{FF61}", "\u{10000}"), Ordering::Greater);
        // Plain BMP below the surrogates keeps code point order.
        assert_eq!(compare_utf16("\u{10000}", "\u{E000}"), Ordering::Less);
        assert_eq!(compare_utf16("\u{D7FF}", "\u{10000}"), Ordering::Less);
        // Two supplementary characters compare by code point.
        assert_eq!(compare_utf16("\u{10000}", "\u{10001}"), Ordering::Less);
        // Prefix with a supplementary character.
        assert_eq!(compare_utf16("a", "a\u{10000}"), Ordering::Less);
    }

    #[test]
    fn parse_plain_and_signed() {
        assert_eq!(parse_int("0").unwrap(), 0);
        assert_eq!(parse_int("123").unwrap(), 123);
        assert_eq!(parse_int("-5").unwrap(), -5);
        assert_eq!(parse_int("+7").unwrap(), 7);
        assert_eq!(parse_int("-0").unwrap(), 0);
        assert_eq!(parse_int("000042").unwrap(), 42);
    }

    #[test]
    fn parse_limits() {
        assert_eq!(parse_int("2147483647").unwrap(), i32::MAX);
        assert_eq!(parse_int("-2147483648").unwrap(), i32::MIN);
        assert_eq!(err("2147483648"), "For input string: \"2147483648\"");
        assert_eq!(err("-2147483649"), "For input string: \"-2147483649\"");
        assert_eq!(
            err("99999999999999999999999"),
            "For input string: \"99999999999999999999999\""
        );
        assert_eq!(
            parse_int("0000000000000000000002147483647").unwrap(),
            i32::MAX
        );
    }

    #[test]
    fn parse_rejects_non_numbers() {
        for s in [
            "", "-", "+", "--1", "+-1", "-+1", "1a", "a1", " 1", "1 ", "%31", "1.0", "0x1",
        ] {
            assert_eq!(err(s), format!("For input string: \"{s}\""), "input {s:?}");
        }
    }

    #[test]
    fn parse_non_ascii_digits() {
        // Oracle: pekko://sys/a#<digit> gives uid 5 for both.
        assert_eq!(parse_int("\u{0665}").unwrap(), 5);
        assert_eq!(parse_int("\u{FF15}").unwrap(), 5);
        assert_eq!(parse_int("\u{FF11}\u{0662}3").unwrap(), 123);
        assert_eq!(parse_int("-\u{FF19}").unwrap(), -9);
    }

    #[test]
    fn parse_supplementary_digit_is_not_a_digit() {
        // U+1D7D8 MATHEMATICAL DOUBLE-STRUCK DIGIT ZERO is Nd but outside the BMP.
        assert!(parse_int("\u{1D7D8}").is_err());
        assert!(parse_int("1\u{1D7D9}").is_err());
    }

    #[test]
    fn parse_digit_series_boundaries() {
        // Just outside series: U+065F, U+066A, U+FF0F, U+FF1A are not digits.
        for c in [
            '\u{065F}', '\u{066A}', '\u{FF0F}', '\u{FF1A}', '\u{002F}', '\u{003A}',
        ] {
            assert!(parse_int(&c.to_string()).is_err(), "{c:?}");
        }
        // A series that starts mid-block: Kayah Li is A900..A909, Myanmar Shan 1090..1099.
        assert_eq!(parse_int("\u{A909}").unwrap(), 9);
        assert_eq!(parse_int("\u{1099}").unwrap(), 9);
        assert_eq!(parse_int("\u{1946}").unwrap(), 0);
    }
}
