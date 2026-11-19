// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).

use std::time::Duration;

use super::TimeUnit;
use crate::actor::IllegalArgumentError;

/// A duration that says what it means (`ask`, `resolveOne`).
///
/// Holds a `FiniteDuration`: signed nanoseconds in `[-(2^63-1), 2^63-1]`. Negative,
/// zero and positive values are valid (`ask` rejects a non-positive one later, not
/// the `Timeout`). `Long.MinValue` ns is outside the range, as in Pekko. Every
/// constructor enforces that single invariant, so the field is private.
///
/// Difference from Pekko, pending an Owner decision (Q-UTIL-2 in the AC-UTIL core
/// plan): `Debug` does not keep the unit the duration was built with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Timeout(i64);

const LIMIT_MESSAGE: &str =
    "requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)";

impl Timeout {
    /// `Timeout.zero`: causes most requests to always time out.
    pub const ZERO: Timeout = Timeout(0);

    /// `Timeout(duration)` from signed nanoseconds. Fails like `Duration.fromNanos`
    /// when `nanos` is `i64::MIN`, the one `Long` outside `FiniteDuration`'s range.
    pub fn new(nanos: i64) -> Result<Timeout, IllegalArgumentError> {
        if nanos == i64::MIN {
            return Err(IllegalArgumentError::new(LIMIT_MESSAGE));
        }
        Ok(Timeout(nanos))
    }

    /// `Timeout(length, unit)`. Fails like `Duration(length, unit)` when the result
    /// leaves the range (including when the conversion overflows a `Long`).
    pub fn of(length: i64, unit: TimeUnit) -> Result<Timeout, IllegalArgumentError> {
        let nanos = length
            .checked_mul(unit.nanos())
            .ok_or_else(|| IllegalArgumentError::new(LIMIT_MESSAGE))?;
        Timeout::new(nanos)
    }

    /// `timeout.duration.toNanos`.
    pub const fn as_nanos(&self) -> i64 {
        self.0
    }
}

/// `Timeout.create(java.time.Duration)` for the non-negative range of `std::time::Duration`. Pekko 2.0.0 goes
/// through `scala.jdk.DurationConverters.toScala` (decision Q-X-16, JDK 17 oracle):
/// - whole seconds only: `FiniteDuration(seconds, SECONDS)`, which fails its range requirement;
/// - seconds and nanos: `seconds * 1e9 + nanos` must fit a `Long`, else `IllegalArgumentException`
///   "Java duration PT…S cannot be expressed as a Scala duration".
impl TryFrom<Duration> for Timeout {
    type Error = IllegalArgumentError;

    fn try_from(duration: Duration) -> Result<Timeout, IllegalArgumentError> {
        let (seconds, nanos) = (duration.as_secs(), duration.subsec_nanos());
        if nanos == 0 {
            return i64::try_from(seconds)
                .map_err(|_| IllegalArgumentError::new(LIMIT_MESSAGE))
                .and_then(|seconds| Timeout::of(seconds, TimeUnit::Seconds));
        }
        if seconds == 0 {
            return Timeout::new(i64::from(nanos));
        }
        i64::try_from(seconds)
            .ok()
            .and_then(|seconds| seconds.checked_mul(1_000_000_000))
            .and_then(|seconds_as_nanos| seconds_as_nanos.checked_add(i64::from(nanos)))
            .map(Timeout)
            .ok_or_else(|| {
                IllegalArgumentError::new(format!(
                    "Java duration {} cannot be expressed as a Scala duration",
                    java_duration_to_string(seconds, nanos)
                ))
            })
    }
}

/// `java.time.Duration.toString` of a non-negative duration (`PT8H6M12.345S`).
fn java_duration_to_string(seconds: u64, nanos: u32) -> String {
    let (hours, minutes, seconds_of_minute) = (seconds / 3600, seconds % 3600 / 60, seconds % 60);
    let mut text = String::from("PT");
    if hours != 0 {
        text += &format!("{hours}H");
    }
    if minutes != 0 {
        text += &format!("{minutes}M");
    }
    if seconds_of_minute == 0 && nanos == 0 && text.len() > 2 {
        return text;
    }
    text += &seconds_of_minute.to_string();
    if nanos > 0 {
        let fraction = format!(".{nanos:09}");
        text += fraction.trim_end_matches('0');
    }
    text + "S"
}

/// `Timeout.toString` matches Pekko's case class auto-generated form:
/// `Timeout(<length> <unit>)` where `<unit>` is the lowercase plural name of the
/// largest unit that divides the nanosecond value exactly. Zero always displays as
/// `0 days`, matching Scala's `Duration.Zero`.
impl std::fmt::Display for Timeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timeout({})", self.duration_text())
    }
}

impl Timeout {
    fn duration_text(&self) -> String {
        let nanos = self.0;
        if nanos == 0 {
            return "0 days".to_owned();
        }
        let abs = nanos.unsigned_abs();
        let (length, unit_name) = Self::best_unit(abs);
        format!("{length} {unit_name}")
    }

    /// Pick the largest TimeUnit whose nanosecond divisor divides `abs_nanos` exactly.
    fn best_unit(abs_nanos: u64) -> (u64, &'static str) {
        let units: &[(u64, &str)] = &[
            (86_400_000_000_000, "days"),
            (3_600_000_000_000, "hours"),
            (60_000_000_000, "minutes"),
            (1_000_000_000, "seconds"),
            (1_000_000, "milliseconds"),
            (1_000, "microseconds"),
            (1, "nanoseconds"),
        ];
        for &(div, name) in units {
            if abs_nanos.is_multiple_of(div) {
                return (abs_nanos / div, name);
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn timeout_display_positive_milliseconds() {
        let t = Timeout::of(100, TimeUnit::Milliseconds).unwrap();
        assert_eq!(format!("{t}"), "Timeout(100 milliseconds)");
    }

    #[test]
    fn timeout_display_zero_is_days() {
        assert_eq!(format!("{}", Timeout::ZERO), "Timeout(0 days)");
    }

    #[test]
    fn timeout_display_one_second_uses_largest_unit() {
        // Q-UTIL-2 (accepted a): Display picks the largest unit that divides evenly,
        // so 1_000_000_000 ns displays as "1 seconds" rather than "1000 milliseconds".
        let t = Timeout::of(1000, TimeUnit::Milliseconds).unwrap();
        assert_eq!(format!("{t}"), "Timeout(1 seconds)");
    }
}
