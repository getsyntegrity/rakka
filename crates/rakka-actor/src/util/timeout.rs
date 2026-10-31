// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).

use std::time::Duration;

use super::TimeUnit;
use crate::actor::{ArithmeticError, IllegalArgumentError};

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

/// `Timeout.create(java.time.Duration)` for the non-negative range of
/// `std::time::Duration`: fails with `ArithmeticException("long overflow")` when the
/// duration does not fit in `Long` nanoseconds, as `Duration.toNanos` does.
impl TryFrom<Duration> for Timeout {
    type Error = ArithmeticError;

    fn try_from(duration: Duration) -> Result<Timeout, ArithmeticError> {
        i64::try_from(duration.as_nanos())
            .map(Timeout)
            .map_err(|_| ArithmeticError::new("long overflow"))
    }
}
