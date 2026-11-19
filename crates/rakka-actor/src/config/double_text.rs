// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
// Original implementation: the HOCON behaviour was measured against Typesafe Config 1.4.9 (Apache-2.0, the library Pekko uses) with the oracle; none of its code was copied.
//! How Java's `Double.toString` writes a double (Typesafe Config renders every non-integral number with it).

/// `Double.toString` of Java: plain decimal for `1e-3 <= |d| < 1e7`, otherwise `d.dddE<exp>`; always at least one
/// digit after the point. The digits are the shortest that identify the double.
///
/// The digits are the shortest that identify the double (Rust's `{:e}`); when two candidates of that length are exactly
/// equally close to the value, the one with an even last digit is shown (decision Q-CFG-30, semantics of JDK 19+ after
/// JDK-4511638). JDK 17 sometimes prints another string, so a replay against a JDK 17 oracle can differ in the last digit
/// on about 0.001 % of the values; `tools/parity/na_q_cfg_30.py` recognises exactly that signature.
pub(crate) fn java_double_to_string(d: f64) -> String {
    if d.is_nan() {
        return "NaN".to_owned();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if d == 0.0 {
        return if d.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
    }
    // `{:e}` gives the shortest round-trip digits as `d.ddde<exp>`. When that is a single digit, Java shows the exact
    // value rounded to two digits (`Double.MIN_VALUE` is `4.9E-324`, not `5.0E-324`; `1e-7` stays `1.0E-7`).
    let mut sci = format!("{:e}", d.abs());
    if !sci.contains('.') {
        sci = format!("{:.1e}", d.abs());
    }
    let (mantissa, exponent) = sci.split_once('e').expect("`{:e}` always has an exponent");
    let exponent: i32 = exponent.parse().expect("`{:e}` exponent is an integer");
    let mut digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    if let Some(even) = even_digits_of_exact_tie(d.abs(), &digits, exponent) {
        digits = even;
    }
    if digits.len() == 2 && digits.ends_with('0') {
        digits.pop(); // the exact value rounds to one digit: `8.0e-1` is `0.8`
    }
    let sign = if d < 0.0 { "-" } else { "" };
    if (-3..7).contains(&exponent) {
        let point = exponent + 1; // digits before the decimal point (<= 0 means a leading `0.000`)
        let body = if point <= 0 {
            format!(
                "0.{}{}",
                "0".repeat(usize::try_from(-point).expect("non-negative")),
                digits
            )
        } else {
            let point = usize::try_from(point).expect("positive");
            if digits.len() <= point {
                format!("{}{}.0", digits, "0".repeat(point - digits.len()))
            } else {
                format!("{}.{}", &digits[..point], &digits[point..])
            }
        };
        format!("{sign}{body}")
    } else {
        let rest = if digits.len() > 1 { &digits[1..] } else { "0" };
        format!("{sign}{}.{rest}E{exponent}", &digits[..1])
    }
}

/// `value` as `decimal * 10^power10`, exactly, without trailing zeros in `decimal`; `None` when it needs more than 18
/// significant digits (a tie between two candidates needs one digit more than the shortest, and the shortest has 17 at most).
fn exact_decimal_expansion(value: f64) -> Option<(u128, i32)> {
    let bits = value.to_bits();
    let biased_exponent = i32::try_from((bits >> 52) & 0x7ff).expect("11 bits");
    let fraction = bits & ((1 << 52) - 1);
    let (mut mantissa, mut exponent) = if biased_exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), biased_exponent - 1075)
    };
    while mantissa % 2 == 0 {
        mantissa /= 2;
        exponent += 1;
    }
    // value = mantissa * 2^exponent with an odd mantissa. Above 2^74 the digits end in an even digit once the zeros
    // are gone (2^k with k > 22 has no 5 left to pair with), below 2^-32 the exact expansion has more than 18 digits.
    let (mut decimal, mut power10) = if exponent >= 0 {
        (
            u128::from(mantissa) << u32::try_from(exponent).ok().filter(|e| *e <= 74)?,
            0,
        )
    } else {
        let places = u32::try_from(-exponent).ok().filter(|p| *p <= 32)?;
        (u128::from(mantissa) * 5u128.pow(places), exponent)
    };
    while decimal % 10 == 0 {
        decimal /= 10;
        power10 += 1;
    }
    Some((decimal, power10))
}

/// When `value` lies exactly halfway between two decimals of the same length as the shortest digits `shortest`
/// (`10^exponent` is the weight of their first digit), returns the digits of the one with an even last digit, but only
/// if it also identifies `value`. Integer arithmetic only: the exact value is mantissa * 2^exponent.
fn even_digits_of_exact_tie(value: f64, shortest: &str, exponent: i32) -> Option<String> {
    let (exact, power10) = exact_decimal_expansion(value)?;
    let exact_length = exact.to_string().len();
    // the two candidates of the shortest length have one digit less than the exact value
    if shortest.len() < 2 || shortest.len() + 1 != exact_length {
        return None;
    }
    let lower = exact / 10;
    let upper = lower + 1;
    let (below, above) = (exact - lower * 10, upper * 10 - exact);
    if below != above {
        return None;
    }
    let even = if lower % 2 == 0 { lower } else { upper };
    let digits = even.to_string();
    let chosen: u128 = shortest.parse().ok()?;
    let text = format!(
        "{digits}e{}",
        exponent - i32::try_from(digits.len()).ok()? + 1
    );
    let identifies_value = text.parse::<f64>().ok() == Some(value);
    let same_power = power10 + i32::try_from(exact_length).ok()? - 1 == exponent;
    (chosen != even && digits.len() == shortest.len() && same_power && identifies_value)
        .then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::java_double_to_string;

    fn from_signed_bits(bits: i64) -> f64 {
        f64::from_bits(u64::from_ne_bytes(bits.to_ne_bytes()))
    }

    #[test]
    fn layout_follows_the_javadoc_at_the_edges_of_the_decimal_range() {
        let expected = [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (100.0, "100.0"),
            (1234567.0, "1234567.0"),
            (1e7, "1.0E7"),
            (9999999.999999998, "9999999.999999998"),
            (1e-3, "0.001"),
            (0.0009999999999999998, "9.999999999999998E-4"),
            (1.234e-5, "1.234E-5"),
            (f64::MIN_POSITIVE, "2.2250738585072014E-308"),
            (f64::from_bits(1), "4.9E-324"),
            (5e-324, "4.9E-324"),
            (f64::MAX, "1.7976931348623157E308"),
            (2e23, "2.0E23"),
            (-2.5e-5, "-2.5E-5"),
        ];
        for (value, text) in expected {
            assert_eq!(java_double_to_string(value), text, "{value:e}");
        }
    }

    /// Q-CFG-30: the value is exactly halfway between two candidates of the shortest length; JDK 19+ (and JDK 17 here)
    /// show the one with an even last digit, `Display` of Rust the larger one.
    #[test]
    fn an_exact_tie_between_two_shortest_candidates_takes_the_even_last_digit() {
        let expected = [
            (4827998994842175714_i64, "5.804742410468122E14"),
            (4833181399437352365_i64, "1.3306497240451632E15"),
            (4814417810547417288_i64, "7.145950805824312E13"),
            (4820888880500797332_i64, "2.0440246380428462E14"),
            (-4387638258303696571_i64, "-1.9687445024768812E15"),
            (-4386863798435220431_i64, "-2.1623594695959162E15"),
        ];
        for (bits, text) in expected {
            assert_eq!(
                java_double_to_string(from_signed_bits(bits)),
                text,
                "{bits}"
            );
        }
    }

    /// The exact value has one digit more than the shortest string but is not halfway: the closest candidate stays,
    /// even when the other one (with an even last digit) would identify the double too.
    #[test]
    fn a_value_close_to_but_not_at_a_tie_keeps_its_closest_candidate() {
        assert_eq!(
            java_double_to_string((5_815_310_220_806_421_u64 << 5) as f64),
            "1.8608992706580547E17"
        );
        assert_eq!(
            java_double_to_string((8_673_382_687_062_571_u64 << 5) as f64),
            "2.7754824598600227E17"
        );
        // too large for a long, so a config render reaches it: 131490295855716352000 has one digit more than the
        // shortest text and its even neighbour (...636) identifies the double too, but it is not halfway
        assert_eq!(
            java_double_to_string((8_025_530_752_912_375_u64 as f64) * 16384.0),
            "1.3149029585571635E20"
        );
    }

    /// 2^-24 is a tie between ...062 and ...063, but below a power of two the gap is half as wide, so only the odd
    /// candidate identifies the double: the even-digit rule must not replace it.
    #[test]
    fn an_even_tie_candidate_that_does_not_identify_the_double_is_not_used() {
        assert_eq!(
            java_double_to_string(2f64.powi(-24)),
            "5.960464477539063E-8"
        );
    }

    /// The five doubles of the Fork D sample where JDK 17 is wrong and JDK 19+ gives what rakka gives.
    #[test]
    fn doubles_where_jdk_17_is_wrong_show_the_jdk_19_digits() {
        let expected = [
            (-4322011209440565094_i64, "-4.872628193264896E19"),
            (4891210820433502222_i64, "9.8411002603808E18"),
            (4986534514107319191_i64, "2.3851354565708684E25"),
            (4890913006577645348_i64, "9.23117748358592E18"),
            (-4234096347181202089_i64, "-3.5624613975366163E25"),
        ];
        for (bits, text) in expected {
            assert_eq!(
                java_double_to_string(from_signed_bits(bits)),
                text,
                "{bits}"
            );
        }
    }

    /// splitmix64: a fixed seed makes the "random" doubles reproducible (the repo has no property-testing crate).
    struct Bits(u64);

    impl Bits {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }

        /// A finite double: any bit pattern, a subnormal, or a value with few mantissa bits (where exact ties live).
        fn finite_double(&mut self) -> f64 {
            loop {
                let raw = self.next();
                let bits = match raw % 3 {
                    0 => self.next(),
                    1 => self.next() & 0x800f_ffff_ffff_ffff,
                    _ => {
                        let few_bits =
                            self.next() & 0xffff_ffff_ffff & (u64::MAX << (self.next() % 40));
                        (self.next() & 0xfff0_0000_0000_0000) | few_bits
                    }
                };
                let value = f64::from_bits(bits);
                if value.is_finite() {
                    return value;
                }
            }
        }
    }

    /// `-?D+.D+` for `1e-3 <= |x| < 1e7`, `-?D.D+E-?D+` otherwise (Javadoc of `Double.toString`).
    fn has_java_layout(text: &str, value: f64) -> bool {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let magnitude = value.abs();
        if (1e-3..1e7).contains(&magnitude) {
            unsigned.split_once('.').is_some_and(|(whole, fraction)| {
                all_digits(whole) && all_digits(fraction) && !unsigned.contains('E')
            })
        } else {
            unsigned.split_once('.').is_some_and(|(whole, rest)| {
                rest.split_once('E').is_some_and(|(fraction, power)| {
                    whole.len() == 1
                        && all_digits(whole)
                        && all_digits(fraction)
                        && all_digits(power.strip_prefix('-').unwrap_or(power))
                })
            })
        }
    }

    /// Significant digits (no leading or trailing zeros) of a rendered number and the decimal exponent of the first one.
    fn significant_digits(text: &str) -> (String, i32) {
        let unsigned = text.trim_start_matches('-');
        let (mantissa, power) = unsigned.split_once('E').unwrap_or((unsigned, "0"));
        let (whole, fraction) = mantissa.split_once('.').expect("a point");
        let all = format!("{whole}{fraction}");
        let leading = all.len() - all.trim_start_matches('0').len();
        let digits = all.trim_matches('0').to_owned();
        let exponent =
            i32::try_from(whole.len()).expect("short") - 1 - i32::try_from(leading).expect("short")
                + power.parse::<i32>().expect("exponent");
        (digits, exponent)
    }

    #[test]
    fn every_finite_double_renders_in_java_layout_round_trips_and_has_no_shorter_form() {
        let mut bits = Bits(0x005e_ed0f_c0ff_ee00);
        // values whose shortest text has one or two digits take the "shown as two" path, which random bits rarely reach
        let short_texts = [1e23, 5e-324, 2e23, 0.1, 1e22, 3e-5, -7e-9];
        let random = std::iter::repeat_with(|| bits.finite_double()).take(300_000);
        for value in short_texts.into_iter().chain(random) {
            let text = java_double_to_string(value);
            assert!(has_java_layout(&text, value), "{value:e} -> {text}");
            let parsed: f64 = text.parse().expect("Java text parses");
            assert_eq!(parsed.to_bits(), value.to_bits(), "round trip of {text}");
            let (digits, exponent) = significant_digits(&text);
            if digits.len() <= 2 {
                continue; // one digit is shown as two, so there is nothing shorter to compare with
            }
            // dropping the last digit (rounding down and up) must not identify the double any more
            let kept = &digits[..digits.len() - 1];
            let rounded_down: u128 = kept.parse().expect("digits");
            for candidate in [rounded_down, rounded_down + 1] {
                let shorter = format!(
                    "{candidate}e{}",
                    exponent - i32::try_from(kept.len()).expect("short") + 1
                );
                let shorter: f64 = shorter.parse().expect("parses");
                assert_ne!(
                    shorter.to_bits(),
                    value.to_bits(),
                    "{text} has a shorter form"
                );
            }
        }
    }
}
