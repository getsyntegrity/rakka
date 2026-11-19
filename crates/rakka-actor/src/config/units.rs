// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Durations (`5 s`, `10ms`) and memory sizes (`128 KiB`, `1 MB`).

use super::model::{NumRepr, Number};
use crate::util::TimeUnit;

/// Splits `"<number> <unit>"`: the unit is the trailing run of ASCII letters.
fn split_unit(input: &str) -> (&str, &str) {
    let trimmed = input.trim();
    let unit_len = trimmed
        .chars()
        .rev()
        .take_while(char::is_ascii_alphabetic)
        .count();
    let (num, unit) = trimmed.split_at(trimmed.len() - unit_len);
    (num.trim(), unit)
}

fn duration_unit(unit: &str) -> Option<TimeUnit> {
    Some(match unit {
        "ns" | "nano" | "nanos" | "nanosecond" | "nanoseconds" => TimeUnit::Nanoseconds,
        "us" | "micro" | "micros" | "microsecond" | "microseconds" => TimeUnit::Microseconds,
        "" | "ms" | "milli" | "millis" | "millisecond" | "milliseconds" => TimeUnit::Milliseconds,
        "s" | "second" | "seconds" => TimeUnit::Seconds,
        "m" | "minute" | "minutes" => TimeUnit::Minutes,
        "h" | "hour" | "hours" => TimeUnit::Hours,
        "d" | "day" | "days" => TimeUnit::Days,
        _ => return None,
    })
}

/// `config.getDuration(path, target)` on the text of the value; `Err` is the message after `Invalid value at '<path>': `.
pub(crate) fn parse_duration(input: &str, target: TimeUnit) -> Result<i64, String> {
    let (num, unit_text) = split_unit(input);
    if num.is_empty() {
        return Err(format!("No number in duration value '{input}'"));
    }
    let unit = duration_unit(unit_text).ok_or_else(|| {
        format!("Could not parse time unit '{unit_text}' (try ns, us, ms, s, m, h, d)")
    })?;
    let number =
        duration_number(num).ok_or_else(|| format!("Could not parse duration number '{num}'"))?;
    let unit_ns = unit.nanos();
    // Typesafe Config first turns the value into a `long` count of nanoseconds, saturating at `Long.MAX_VALUE` and
    // `Long.MIN_VALUE`, and only then converts it to the requested unit (so `106752 d` in ms is `9223372036854`).
    let total_ns = match number {
        DurationNumber::Whole(n) => clamp(i128::from(n) * i128::from(unit_ns)),
        DurationNumber::Fractional(f) => (f * unit_ns as f64) as i64, // saturating, NaN is 0, like Java's `(long)`
    };
    Ok(total_ns / target.nanos())
}

/// The number of a duration: `[+-]?[0-9]+` is a `long` (`Long.parseLong`), anything else must be a Java double literal.
enum DurationNumber {
    Whole(i64),
    Fractional(f64),
}

fn duration_number(text: &str) -> Option<DurationNumber> {
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    if !unsigned.is_empty() && unsigned.bytes().all(|b| b.is_ascii_digit()) {
        return text.parse().ok().map(DurationNumber::Whole);
    }
    java_double(text).map(DurationNumber::Fractional)
}

/// `Double.parseDouble` for the characters that can reach it here: an optional sign, `NaN`, `Infinity`, or
/// `digits[.digits][e[+-]digits]` (`5.`, `.5` and `1e3` included).
fn java_double(text: &str) -> Option<f64> {
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    match unsigned {
        "NaN" => return Some(f64::NAN),
        "Infinity" => {
            return Some(if text.starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }
        _ => {}
    }
    let bytes = unsigned.as_bytes();
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut i = digits(0);
    let mut mantissa_digits = i;
    if bytes.get(i) == Some(&b'.') {
        let fraction = digits(i + 1);
        mantissa_digits += fraction;
        i += 1 + fraction;
    }
    if mantissa_digits == 0 {
        return None;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let exponent = digits(i);
        if exponent == 0 {
            return None;
        }
        i += exponent;
    }
    if i != bytes.len() {
        return None;
    }
    text.parse().ok()
}

fn clamp(v: i128) -> i64 {
    i64::try_from(v).unwrap_or(if v < 0 { i64::MIN } else { i64::MAX })
}

const SIZE_UNITS_HINT: &str = "try k, K, kB, KiB, kilobytes, kibibytes";

/// Multiplier of a size unit, as a power of 1000 or 1024.
fn size_multiplier(unit: &str) -> Option<u128> {
    let bin = |n: u32| 1024u128.pow(n);
    let dec = |n: u32| 1000u128.pow(n);
    let table: [(u32, &[&str], &[&str]); 8] = [
        (
            1,
            &["k", "K", "Ki", "KiB", "kibibyte", "kibibytes"],
            &["kB", "kilobyte", "kilobytes"],
        ),
        (
            2,
            &["m", "M", "Mi", "MiB", "mebibyte", "mebibytes"],
            &["MB", "megabyte", "megabytes"],
        ),
        (
            3,
            &["g", "G", "Gi", "GiB", "gibibyte", "gibibytes"],
            &["GB", "gigabyte", "gigabytes"],
        ),
        (
            4,
            &["t", "T", "Ti", "TiB", "tebibyte", "tebibytes"],
            &["TB", "terabyte", "terabytes"],
        ),
        (
            5,
            &["p", "P", "Pi", "PiB", "pebibyte", "pebibytes"],
            &["PB", "petabyte", "petabytes"],
        ),
        (
            6,
            &["e", "E", "Ei", "EiB", "exbibyte", "exbibytes"],
            &["EB", "exabyte", "exabytes"],
        ),
        (
            7,
            &["z", "Z", "Zi", "ZiB", "zebibyte", "zebibytes"],
            &["ZB", "zettabyte", "zettabytes"],
        ),
        (
            8,
            &["y", "Y", "Yi", "YiB", "yobibyte", "yobibytes"],
            &["YB", "yottabyte", "yottabytes"],
        ),
    ];
    if matches!(unit, "" | "b" | "B" | "byte" | "bytes") {
        return Some(1);
    }
    table.iter().find_map(|(n, binary, decimal)| {
        if binary.contains(&unit) {
            Some(bin(*n))
        } else if decimal.contains(&unit) {
            Some(dec(*n))
        } else {
            None
        }
    })
}

/// `config.getBytes(path)` on the text of the value; `Err` is the message after `Invalid value at '<path>': `.
pub(crate) fn parse_size(input: &str) -> Result<i64, String> {
    let (num, unit_text) = split_unit(input);
    if num.is_empty() {
        return Err(format!("No number in size-in-bytes value '{input}'"));
    }
    let mult = size_multiplier(unit_text).ok_or_else(|| {
        format!("Could not parse size-in-bytes unit '{unit_text}' ({SIZE_UNITS_HINT})")
    })?;
    let number = Number::parse(num)
        .ok_or_else(|| format!("Could not parse size-in-bytes number '{num}'"))?;
    let bytes: i128 = match number.repr {
        NumRepr::Int(n) => i128::from(n)
            .checked_mul(i128::try_from(mult).unwrap_or(i128::MAX))
            .ok_or_else(|| {
                format!("size-in-bytes value is out of range for a 64-bit long: '{num}'")
            })?,
        NumRepr::Float(f) => (f * mult as f64) as i128,
    };
    if bytes < 0 {
        return Err(format!(
            "Attempt to construct memory size with negative number: {bytes}"
        ));
    }
    i64::try_from(bytes)
        .map_err(|_| format!("size-in-bytes value is out of range for a 64-bit long: '{bytes}'"))
}
