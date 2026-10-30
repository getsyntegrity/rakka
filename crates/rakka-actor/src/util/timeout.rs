// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).

use std::time::Duration;

use super::TimeUnit;
use crate::actor::IllegalArgumentError;

/// A `Duration` that says what the duration means (`ask`, `resolveOne`).
///
/// Differences from Pekko's `Timeout(FiniteDuration)`, pending an Owner decision
/// (Q-UTIL-1, Q-UTIL-2 in the AC-UTIL core plan): a negative timeout cannot be
/// built, [`Timeout::new`] does not enforce the 2^63-1 ns bound that
/// [`Timeout::of`] does, and `Debug` does not keep the unit it was built with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Timeout(Duration);

impl Timeout {
    /// `Timeout.zero`: causes most requests to always time out.
    pub const ZERO: Timeout = Timeout(Duration::ZERO);

    /// `Timeout(duration)`.
    pub const fn new(duration: Duration) -> Timeout {
        Timeout(duration)
    }

    /// `Timeout(length, unit)`. Fails like `Duration(length, unit)` when the
    /// result exceeds `FiniteDuration`'s bound of 2^63-1 ns.
    pub fn of(length: u64, unit: TimeUnit) -> Result<Timeout, IllegalArgumentError> {
        // FiniteDuration: |nanos| <= Long.MaxValue (scala.concurrent.duration.FiniteDuration).
        match length.checked_mul(unit.nanos()) {
            Some(nanos) if nanos <= i64::MAX as u64 => {
                Ok(Timeout::new(Duration::from_nanos(nanos)))
            }
            _ => Err(IllegalArgumentError::new(
                "requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)",
            )),
        }
    }

    /// `timeout.duration`.
    pub const fn duration(&self) -> Duration {
        self.0
    }
}

/// The implicit `durationToTimeout`.
impl From<Duration> for Timeout {
    fn from(duration: Duration) -> Timeout {
        Timeout::new(duration)
    }
}
