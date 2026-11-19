// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).

/// `java.util.concurrent.TimeUnit`, as taken by `Timeout(length, unit)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    /// `NANOSECONDS`.
    Nanoseconds,
    /// `MICROSECONDS`.
    Microseconds,
    /// `MILLISECONDS`.
    Milliseconds,
    /// `SECONDS`.
    Seconds,
    /// `MINUTES`.
    Minutes,
    /// `HOURS`.
    Hours,
    /// `DAYS`.
    Days,
}

impl TimeUnit {
    /// Nanoseconds in one of this unit.
    pub(crate) const fn nanos(self) -> i64 {
        match self {
            TimeUnit::Nanoseconds => 1,
            TimeUnit::Microseconds => 1_000,
            TimeUnit::Milliseconds => 1_000_000,
            TimeUnit::Seconds => 1_000_000_000,
            TimeUnit::Minutes => 60_000_000_000,
            TimeUnit::Hours => 3_600_000_000_000,
            TimeUnit::Days => 86_400_000_000_000,
        }
    }
}
