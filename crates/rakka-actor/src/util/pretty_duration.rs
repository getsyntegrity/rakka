// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/PrettyDuration.scala (Apache-2.0).
//! `PrettyDuration` (AC-UTIL-016.5): the human readable rendering of a duration used in log messages.

use super::Timeout;
use crate::actor::IllegalArgumentError;

/// `scala.concurrent.duration.Duration`: a finite duration (signed nanoseconds, Q-UTIL-1) or one of the three
/// non-finite values (Q-UTIL-9 (a)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PekkoDuration {
    /// A finite duration.
    Finite(Timeout),
    /// `Duration.Inf`.
    Inf,
    /// `Duration.MinusInf`.
    MinusInf,
    /// `Duration.Undefined`.
    Undefined,
}

impl PekkoDuration {
    /// `duration.pretty`: four significant digits, without the nanoseconds.
    pub fn pretty(&self) -> String {
        self.pretty_with(false, 4)
            .expect("a precision of 4 is valid")
    }

    /// `duration.pretty(includeNanos, precision)`. Fails like `require(precision > 0, ...)`, also for the
    /// non-finite values.
    pub fn pretty_with(
        &self,
        include_nanos: bool,
        precision: i32,
    ) -> Result<String, IllegalArgumentError> {
        if precision <= 0 {
            return Err(IllegalArgumentError::new(
                "requirement failed: precision must be > 0",
            ));
        }
        let precision = precision as usize;
        Ok(match self {
            PekkoDuration::Finite(timeout) => {
                let nanos = timeout.as_nanos();
                let (unit_nanos, abbreviation) = choose_unit(nanos);
                let value = nanos as f64 / unit_nanos as f64;
                let exact = if include_nanos {
                    format!(" ({nanos} ns)")
                } else {
                    String::new()
                };
                format!("{} {abbreviation}{exact}", format_general(value, precision))
            }
            PekkoDuration::MinusInf => "-\u{221e} (minus infinity)".to_owned(),
            PekkoDuration::Inf => "\u{221e} (infinity)".to_owned(),
            PekkoDuration::Undefined => "undefined".to_owned(),
        })
    }
}

/// `chooseUnit` and `abbreviate`: the largest unit whose truncated count is positive, so a non-positive duration
/// is always shown in nanoseconds.
fn choose_unit(nanos: i64) -> (i64, &'static str) {
    const UNITS: [(i64, &str); 6] = [
        (86_400_000_000_000, "d"),
        (3_600_000_000_000, "h"),
        (60_000_000_000, "min"),
        (1_000_000_000, "s"),
        (1_000_000, "ms"),
        (1_000, "\u{3bc}s"),
    ];
    UNITS
        .into_iter()
        .find(|(unit_nanos, _)| nanos >= *unit_nanos)
        .unwrap_or((1, "ns"))
}

/// `String.format(Locale.ROOT, "%.<precision>g", value)` of `java.util.Formatter` for a finite `value` that is zero
/// or has a magnitude of at least 1 (always the case here: the unit is chosen so that the count is at least 1, and
/// a non-positive duration is shown in nanoseconds).
///
/// The digits come from the shortest decimal that identifies the double and are rounded half up to `precision`
/// significant digits (not from the binary expansion, which is what C does). The result is decimal notation while
/// the rounded value is below `10^precision`, otherwise scientific notation with a sign and at least two digits in
/// the exponent. Zero is `0.000...` with `precision - 1` decimals.
fn format_general(value: f64, precision: usize) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let (mut digits, mut exponent) = shortest_digits(value.abs());
    if digits.len() > precision {
        let round_up = digits[precision] >= 5;
        digits.truncate(precision);
        if round_up {
            exponent += increment(&mut digits);
        }
    }
    digits.resize(precision, 0);
    let text: Vec<char> = digits.iter().map(|d| char::from(b'0' + d)).collect();
    let integer_digits = exponent as usize + 1;
    if exponent < precision as i32 {
        let (integer, fraction) = text.split_at(integer_digits.min(precision));
        let fraction: String = fraction.iter().collect();
        let integer: String = integer.iter().collect();
        if fraction.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{fraction}")
        }
    } else {
        let fraction: String = text[1..].iter().collect();
        let point = if fraction.is_empty() { "" } else { "." };
        format!("{sign}{}{point}{fraction}e+{exponent:02}", text[0])
    }
}

/// Decimal digits and decimal exponent (`value = d.ddd * 10^exponent`) that `java.util.Formatter` starts rounding
/// from: the shortest decimal that identifies `value`, except for an integral value of 2^53 or more (a negative
/// duration shown in nanoseconds), whose digits are those of the exact integer with its lowest
/// `dropped_digits(binary exponent)` digits rounded away, as measured with the oracle (PD-19).
fn shortest_digits(value: f64) -> (Vec<u8>, i32) {
    if value == 0.0 {
        return (vec![0], 0);
    }
    if (9_007_199_254_740_992.0..=9_223_372_036_854_775_808.0).contains(&value) {
        return integral_digits(value as u64);
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').expect("exponent marker");
    let digits = mantissa
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|b| b - b'0')
        .collect();
    (digits, exponent.parse().expect("decimal exponent"))
}

/// Adds one unit in the last place to `digits`; a carry out of the first digit leaves `1000...` and returns 1, the
/// amount the exponent grows by.
fn increment(digits: &mut [u8]) -> i32 {
    for digit in digits.iter_mut().rev() {
        if *digit == 9 {
            *digit = 0;
        } else {
            *digit += 1;
            return 0;
        }
    }
    digits[0] = 1;
    1
}

/// Digits of the integer `exact` (at least 2^53, at most 2^63) as Java derives them from a `double`: the lowest
/// digits that the 53 bits of the mantissa cannot tell apart are rounded away, half up.
fn integral_digits(exact: u64) -> (Vec<u8>, i32) {
    let binary_exponent = exact.ilog2();
    let dropped = match binary_exponent {
        ..=57 => 0,
        58..=60 => 1,
        61 | 62 => 2,
        _ => 3,
    };
    let mut digits: Vec<u8> = exact.to_string().bytes().map(|b| b - b'0').collect();
    let exponent = digits.len() as i32 - 1;
    if dropped == 0 {
        return (digits, exponent);
    }
    let kept = digits.len() - dropped;
    let round_up = digits[kept] >= 5;
    digits.truncate(kept);
    let carry = if round_up { increment(&mut digits) } else { 0 };
    (digits, exponent + carry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finite(nanos: i64) -> PekkoDuration {
        PekkoDuration::Finite(Timeout::new(nanos).expect("in range"))
    }

    const MICROS: i64 = 1_000;
    const MILLIS: i64 = 1_000_000;
    const SECONDS: i64 = 1_000_000_000;
    const MINUTES: i64 = 60 * SECONDS;
    const HOURS: i64 = 60 * MINUTES;
    const DAYS: i64 = 24 * HOURS;

    #[test]
    fn prints_the_spec_table_with_four_significant_digits() {
        let table = [
            (9, "9.000 ns"),
            (95, "95.00 ns"),
            (999, "999.0 ns"),
            (1000, "1.000 \u{3bc}s"),
            (9500, "9.500 \u{3bc}s"),
            (9500 * MICROS, "9.500 ms"),
            (9500 * MILLIS, "9.500 s"),
            (95 * SECONDS, "1.583 min"),
            (95 * MINUTES, "1.583 h"),
            (95 * HOURS, "3.958 d"),
        ];
        for (nanos, expected) in table {
            assert_eq!(finite(nanos).pretty(), expected, "{nanos} ns");
        }
    }

    #[test]
    fn prints_the_non_finite_values_whatever_the_options() {
        assert_eq!(PekkoDuration::Inf.pretty(), "\u{221e} (infinity)");
        assert_eq!(
            PekkoDuration::MinusInf.pretty_with(true, 1).unwrap(),
            "-\u{221e} (minus infinity)"
        );
        assert_eq!(
            PekkoDuration::Undefined.pretty_with(true, 9).unwrap(),
            "undefined"
        );
    }

    #[test]
    fn rejects_a_precision_that_is_not_positive_even_for_non_finite_values() {
        for duration in [finite(5), PekkoDuration::Inf, PekkoDuration::Undefined] {
            for precision in [0, -1, i32::MIN] {
                let error = duration.pretty_with(false, precision).unwrap_err();
                assert_eq!(error.java_class(), "IllegalArgumentException");
                assert_eq!(error.message(), "requirement failed: precision must be > 0");
            }
        }
    }

    #[test]
    fn switches_to_scientific_notation_when_the_rounded_value_reaches_ten_to_the_precision() {
        assert_eq!(finite(9999 * DAYS).pretty(), "9999 d");
        assert_eq!(finite(10000 * DAYS).pretty(), "1.000e+04 d");
        assert_eq!(finite(99999 * DAYS).pretty(), "1.000e+05 d");
        assert_eq!(finite(9999 * DAYS + 12 * HOURS).pretty(), "1.000e+04 d");
        assert_eq!(
            finite(999_950).pretty_with(false, 1).unwrap(),
            "1e+03 \u{3bc}s"
        );
        assert_eq!(
            finite(i64::MAX).pretty_with(true, 2).unwrap(),
            "1.1e+05 d (9223372036854775807 ns)"
        );
    }

    #[test]
    fn keeps_the_unit_boundaries_and_the_nanosecond_unit_for_non_positive_values() {
        assert_eq!(finite(999_999).pretty(), "1000 \u{3bc}s");
        assert_eq!(finite(86_399_999_999_999).pretty(), "24.00 h");
        assert_eq!(finite(0).pretty(), "0.000 ns");
        assert_eq!(finite(0).pretty_with(false, 1).unwrap(), "0 ns");
        assert_eq!(finite(-5).pretty(), "-5.000 ns");
        assert_eq!(finite(-95 * SECONDS).pretty(), "-9.500e+10 ns");
    }

    #[test]
    fn rounds_half_up_on_the_shortest_decimal_digits() {
        assert_eq!(finite(1_000_500).pretty(), "1.001 ms");
        assert_eq!(finite(12_345).pretty(), "12.35 \u{3bc}s");
        assert_eq!(finite(2_675_000).pretty_with(false, 3).unwrap(), "2.68 ms");
        assert_eq!(
            finite(9_999).pretty_with(false, 1).unwrap(),
            "1e+01 \u{3bc}s"
        );
    }

    #[test]
    fn pads_with_zeros_beyond_the_digits_of_the_value() {
        assert_eq!(
            finite(1500).pretty_with(false, 8).unwrap(),
            "1.5000000 \u{3bc}s"
        );
        assert_eq!(
            finite(i64::MAX).pretty_with(false, 20).unwrap(),
            "106751.99116730064000 d"
        );
    }

    #[test]
    fn prints_the_digits_of_the_exact_integer_for_a_negative_duration_of_2_pow_53_or_more() {
        // 2^63 is 9223372036854775808; the three lowest digits are rounded away.
        assert_eq!(
            finite(-i64::MAX).pretty_with(false, 19).unwrap(),
            "-9223372036854776000 ns"
        );
        // 2^55: exact integer digits, nothing rounded away.
        let nanos = -(1 << 55);
        assert_eq!(
            finite(nanos).pretty_with(false, 20).unwrap(),
            "-36028797018963968.000 ns"
        );
        // 2^59: one digit rounded away (576460752303423488 -> ...423490).
        assert_eq!(
            finite(-(1 << 59)).pretty_with(false, 18).unwrap(),
            "-576460752303423490 ns"
        );
        // 2^61: two digits rounded away (2305843009213693952 -> ...694000).
        assert_eq!(
            finite(-(1 << 61)).pretty_with(false, 19).unwrap(),
            "-2305843009213694000 ns"
        );
    }

    #[test]
    fn appends_the_exact_nanoseconds_when_asked() {
        assert_eq!(
            finite(1500).pretty_with(true, 4).unwrap(),
            "1.500 \u{3bc}s (1500 ns)"
        );
    }
}
