// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Version.scala (Apache-2.0).
//! `Version` (AC-UTIL-005.2): a comparable version of up to three numbers plus a qualifier.

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::actor::jstring::{compare_to_raw, is_digit, parse_int};
use crate::actor::{IllegalArgumentError, NumberFormatError};
use crate::util::hash_code;

/// What `Version(String)` can throw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionError {
    IllegalArgument(IllegalArgumentError),
    NumberFormat(NumberFormatError),
}

impl VersionError {
    /// Simple name of the Java exception class Pekko throws.
    pub fn java_class(&self) -> &'static str {
        match self {
            Self::IllegalArgument(e) => e.java_class(),
            Self::NumberFormat(e) => e.java_class(),
        }
    }

    /// `getMessage`.
    pub fn message(&self) -> &str {
        match self {
            Self::IllegalArgument(e) => e.message(),
            Self::NumberFormat(e) => e.message(),
        }
    }
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for VersionError {}

/// `Version.Undefined`: the number of a part that is absent.
const UNDEFINED: i32 = 0;

/// A comparable version: up to four numbers (`major.minor.patch` plus the dynver commit count) and a
/// qualifier (`M1`, `RC2`, `SNAPSHOT`, the hash of a dynver version, ...).
///
/// Unlike Pekko, which parses lazily on first use, the parse happens in [`Version::parse`] (Q-UTIL-6 and
/// AC-UTIL-005.2: an immutable value). Comparison follows Pekko 2.0.0: a version without qualifier is greater
/// than the same version with one (`1.2.0 > 1.2.0-M1`); it is [`Version::pekko_compare`] and not `Ord`
/// because the differences are `Int` subtractions that wrap, so the order is not total for numbers beyond
/// half the `i32` range.
#[derive(Clone, Debug)]
pub struct Version {
    version: String,
    numbers: [i32; 4],
    rest: String,
}

/// `s.split('.')`: no separator gives the whole string, otherwise the trailing empty parts are dropped.
fn split_segments(s: &str) -> Vec<&str> {
    if !s.contains('.') {
        return vec![s];
    }
    let mut parts: Vec<&str> = s.split('.').collect();
    while parts.last() == Some(&"") {
        parts.pop();
    }
    parts
}

fn to_int(s: &str) -> Result<i32, VersionError> {
    parse_int(s).map_err(VersionError::NumberFormat)
}

/// `parseLastPart`: `2`, `2-SNAPSHOT` or dynver `2+10-1234abcd` into the number and the rest.
fn parse_last_part(s: &str) -> Result<(i32, &str), VersionError> {
    if s.is_empty() {
        return Ok((UNDEFINED, s));
    }
    let separator = match (s.find('-'), s.find('+')) {
        (Some(i), Some(j)) => Some(i.min(j)),
        (i, j) => i.or(j),
    };
    match separator {
        None => Ok((to_int(s)?, "")),
        Some(k) => Ok((to_int(&s[..k])?, &s[k + 1..])),
    }
}

/// `parseDynverPart`: `SNAPSHOT` or dynver `10-1234abcd` into the commit count and the rest.
fn parse_dynver_part(s: &str) -> (i32, &str) {
    let starts_with_digit = s
        .chars()
        .next()
        .is_some_and(|c| u16::try_from(u32::from(c)).is_ok_and(is_digit));
    if !starts_with_digit {
        return (UNDEFINED, s);
    }
    match s.find('-') {
        None => (UNDEFINED, s),
        Some(i) => match parse_int(&s[..i]) {
            Ok(count) => (count, &s[i + 1..]),
            Err(_) => (UNDEFINED, s),
        },
    }
}

/// `parseLastParts`: the last segment into its number, the dynver number and the rest.
fn parse_last_parts(s: &str) -> Result<(i32, i32, &str), VersionError> {
    let (last, rest) = parse_last_part(s)?;
    if rest.is_empty() {
        Ok((last, UNDEFINED, rest))
    } else {
        let (dynver, rest) = parse_dynver_part(rest);
        Ok((last, dynver, rest))
    }
}

impl Version {
    /// `Version(version)`: `1`, `1.2`, `1.2.3`, each with an optional `-qualifier` or dynver `+N-hash`.
    ///
    /// Errors: `IllegalArgumentException` for an empty string and for more than three `.`-separated
    /// numbers, `NumberFormatException` for a segment that is not a number where one is expected.
    pub fn parse(version: &str) -> Result<Self, VersionError> {
        let segments = split_segments(version);
        let (numbers, rest) = match segments.as_slice() {
            [single] => {
                if single.is_empty() {
                    return Err(VersionError::IllegalArgument(IllegalArgumentError::new(
                        "Empty version not supported.",
                    )));
                }
                // A single number, or plain alphanumeric text when it is not one.
                let first_is_digit = single
                    .chars()
                    .next()
                    .is_some_and(|c| u16::try_from(u32::from(c)).is_ok_and(is_digit));
                match first_is_digit.then(|| parse_int(single)) {
                    Some(Ok(major)) => ([major, UNDEFINED, UNDEFINED, UNDEFINED], ""),
                    _ => ([UNDEFINED; 4], *single),
                }
            }
            [major, last] => {
                let (n1, n2, rest) = parse_last_parts(last)?;
                ([to_int(major)?, n1, n2, UNDEFINED], rest)
            }
            [major, minor, last] => {
                let (n1, n2, rest) = parse_last_parts(last)?;
                ([to_int(major)?, to_int(minor)?, n1, n2], rest)
            }
            _ => {
                return Err(VersionError::IllegalArgument(IllegalArgumentError::new(
                    format!("Only 3 digits separated with '.' are supported. [{version}]"),
                )));
            }
        };
        Ok(Self {
            version: version.to_owned(),
            numbers,
            rest: rest.to_owned(),
        })
    }

    /// `Version.Zero`.
    pub fn zero() -> Self {
        Self::parse("0.0.0").expect("0.0.0 is a valid version")
    }

    /// The string this version was built from.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// `compareTo`: negative, zero or positive. The same string compares as 0 without looking at the parts;
    /// otherwise the first of the four numbers that differs gives the (wrapping) difference, and then the
    /// qualifiers: none is greater than any, else `String.compareTo` of the two.
    pub fn pekko_compare(&self, other: &Self) -> i32 {
        if self.version == other.version {
            return 0;
        }
        for (a, b) in self.numbers.iter().zip(&other.numbers) {
            let diff = a.wrapping_sub(*b);
            if diff != 0 {
                return diff;
            }
        }
        match (self.rest.is_empty(), other.rest.is_empty()) {
            (true, false) => 1,
            (false, true) => -1,
            _ => compare_to_raw(&self.rest, &other.rest),
        }
    }

    /// `hashCode`: the four numbers and the qualifier folded with `HashCode`.
    pub fn hash_code(&self) -> i32 {
        let seed = self.numbers.iter().fold(hash_code::SEED, hash_code::hash);
        hash_code::hash(seed, self.rest.as_str())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.pekko_compare(other) == 0
    }
}

impl Eq for Version {}

impl Hash for Version {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn final_is_greater_than_qualified_in_both_orders() {
        // Q-UTIL-6 (Owner): the corrected 2.0.0 behaviour, measured with the 2.0.0 jar (VR-05, VR-11).
        assert_eq!(v("1.2.0").pekko_compare(&v("1.2-M1")), 1);
        assert_eq!(v("1.2-M1").pekko_compare(&v("1.2.0")), -1);
        assert_eq!(v("1.2.0").pekko_compare(&v("1.2.0-RC1")), 1);
        assert_eq!(v("1.2.0-RC1").pekko_compare(&v("1.2.0")), -1);
    }

    #[test]
    fn numbers_then_qualifier() {
        assert_eq!(v("1.2.4").pekko_compare(&v("1.2.3")), 1);
        assert_eq!(v("3.2").pekko_compare(&v("3.2.7")), -7);
        assert_eq!(v("1.2.0-M9").pekko_compare(&v("1.2.0-M10")), 8);
        assert_eq!(v("1.0.10+3-ab").pekko_compare(&v("1.0.10+10-ab")), -7);
        assert_eq!(v("1.2.3-M1"), v("1.2.3-M1"));
        assert_eq!(v("1.2"), v("1.2.0"));
        assert_ne!(v("1.2"), v("1.3"));
    }

    #[test]
    fn differences_wrap_like_int_subtraction() {
        assert_eq!(v("2147483647.0.0").pekko_compare(&v("-1.0.0")), i32::MIN);
    }

    #[test]
    fn parse_errors_carry_the_java_class() {
        let e = Version::parse("").unwrap_err();
        assert_eq!(e.java_class(), "IllegalArgumentException");
        assert_eq!(e.to_string(), "Empty version not supported.");
        let e = Version::parse("1.2.3.4").unwrap_err();
        assert_eq!(
            e.message(),
            "Only 3 digits separated with '.' are supported. [1.2.3.4]"
        );
        let e = Version::parse("1.x").unwrap_err();
        assert_eq!(e.java_class(), "NumberFormatException");
        assert_eq!(e.message(), "For input string: \"x\"");
        assert!(Version::parse(".").is_err());
        assert!(Version::parse("1.").is_ok());
    }

    #[test]
    fn non_numeric_single_segment_is_text() {
        assert_eq!(v("2b").pekko_compare(&v("2a")), 1);
        assert_eq!(v("99999999999").pekko_compare(&v("1")), -1);
        assert_eq!(v("foo").pekko_compare(&v("1.2.3")), -1);
    }

    #[test]
    fn dynver_number_is_the_fourth_part() {
        let parsed = v("1.0.10+3-1234abcd");
        assert_eq!(parsed.numbers, [1, 0, 10, 3]);
        assert_eq!(parsed.rest, "1234abcd");
        // not a number before the first dash: kept as qualifier
        assert_eq!(v("1.0.10+99999999999-abc").rest, "99999999999-abc");
        assert_eq!(v("1.0.10-SNAPSHOT").numbers[3], 0);
    }

    #[test]
    fn equal_versions_hash_equal_and_display_is_the_source() {
        use std::collections::hash_map::DefaultHasher;
        assert_eq!(v("1.2").hash_code(), v("1.2.0").hash_code());
        assert_eq!(Version::zero().hash_code(), v("0").hash_code());
        let hash = |x: &Version| {
            let mut h = DefaultHasher::new();
            x.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(&v("1.2")), hash(&v("1.2.0")));
        assert_eq!(v("1.2.0-M1").to_string(), "1.2.0-M1");
        assert_eq!(v("1.2.0-M1").version(), "1.2.0-M1");
    }
}
