// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).

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
        text += &format!(".{nanos:09}").trim_end_matches('0').to_owned();
    }
    text + "S"
}
