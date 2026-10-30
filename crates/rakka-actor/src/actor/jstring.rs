// Reproduces the observable behavior of OpenJDK 21 java.lang.String.compareTo, Integer.valueOf
// (radix 10) and Character.digit (GPL-2.0 WITH Classpath-exception-2.0); the BMP digit table
// lists the values Character.digit reports on Java 21 (Unicode Nd data). Whether this file is
// derived from OpenJDK, and its licensing, is pending Owner decision Q-PATH-6.
//! The bits of `java.lang.String` semantics that are observable through Pekko.

use std::cmp::Ordering;

use super::errors::NumberFormatError;

/// `String.compareTo`: lexicographic order of UTF-16 code units.
pub(crate) fn compare_utf16(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// `Integer.valueOf(s)` (radix 10). Fails with `NumberFormatException`.
///
/// Like Java it reads UTF-16 code units, so only BMP decimal digits count:
/// `Character.digit(char, 10)` accepts any Unicode `Nd` char of the BMP.
pub(crate) fn parse_int(s: &str) -> Result<i32, NumberFormatError> {
    let error = || NumberFormatError::new(format!("For input string: \"{s}\""));
    let units: Vec<u16> = s.encode_utf16().collect();
    let (negative, digits) = match units.split_first() {
        None => return Err(error()),
        Some((&first, rest)) if first == u16::from(b'-') => (true, rest),
        Some((&first, rest)) if first == u16::from(b'+') => (false, rest),
        Some(_) => (false, &units[..]),
    };
    if digits.is_empty() {
        return Err(error());
    }
    let mut value: i64 = 0;
    for &unit in digits {
        let digit = decimal_digit(unit).ok_or_else(error)?;
        value = value * 10 + i64::from(digit);
        if value > i64::from(i32::MAX) + 1 {
            return Err(error());
        }
    }
    let value = if negative { -value } else { value };
    i32::try_from(value).map_err(|_| error())
}

/// Code points of digit zero of every BMP `Nd` run (each run is 0..=9), as
/// reported by `Character.digit` on Java 21.
const BMP_DIGIT_ZEROS: [u16; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

fn decimal_digit(unit: u16) -> Option<u8> {
    BMP_DIGIT_ZEROS
        .iter()
        .find(|&&zero| (zero..zero + 10).contains(&unit))
        .map(|&zero| (unit - zero) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int_accepts_signs_and_unicode_decimal_digits() {
        for (s, v) in [
            ("5", 5),
            ("+5", 5),
            ("-5", -5),
            ("0", 0),
            ("007", 7),
            ("2147483647", i32::MAX),
            ("-2147483648", i32::MIN),
            ("\u{665}", 5),
            ("\u{FF15}", 5),
            ("\u{966}\u{967}", 1),
        ] {
            assert_eq!(parse_int(s), Ok(v), "{s}");
        }
    }

    #[test]
    fn parse_int_rejects_with_the_java_message() {
        for s in [
            "",
            "+",
            "-",
            "+-5",
            "abc",
            "5?x",
            "1#2",
            "2147483648",
            "-2147483649",
            "99999999999",
            " 5",
            "\u{1D7CE}",
            "\u{B2}",
        ] {
            let e = parse_int(s).unwrap_err();
            assert_eq!(e.message(), format!("For input string: \"{s}\""));
        }
    }

    #[test]
    fn compares_by_utf16_code_units_not_utf8_bytes() {
        assert_eq!(compare_utf16("\u{FF61}", "\u{10000}"), Ordering::Greater);
        assert_eq!("\u{FF61}".cmp("\u{10000}"), Ordering::Less);
    }
}
