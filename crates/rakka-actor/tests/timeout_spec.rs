// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).
//! `Timeout` (AC-UTIL-001), plan docs/superpowers/plans/2026-10-30-ac-util-core.md.

use std::time::Duration;

use rakka_actor::util::{TimeUnit, Timeout};

#[test]
fn new_wraps_the_duration() {
    let t = Timeout::new(Duration::from_millis(100));
    assert_eq!(t.duration(), Duration::from_millis(100));
}

#[test]
fn equality_and_hash_are_by_value_not_by_unit() {
    // Timeout(1000.millis) == Timeout(1.second) in Pekko (case class over FiniteDuration).
    let a = Timeout::new(Duration::from_millis(1000));
    let b = Timeout::new(Duration::from_secs(1));
    assert_eq!(a, b);
    assert_ne!(a, Timeout::new(Duration::from_secs(2)));
    let set: std::collections::HashSet<Timeout> = [a].into_iter().collect();
    assert!(set.contains(&b));
}

#[test]
fn zero_is_a_zero_duration_timeout() {
    // Timeout.zero (Timeout.scala:39): "will cause most requests to always timeout".
    assert_eq!(Timeout::ZERO.duration(), Duration::ZERO);
    assert_eq!(Timeout::ZERO, Timeout::new(Duration::ZERO));
}

#[test]
fn a_duration_converts_implicitly_into_a_timeout() {
    // implicit durationToTimeout (Timeout.scala:54).
    let t: Timeout = Duration::from_secs(3).into();
    assert_eq!(t, Timeout::new(Duration::from_secs(3)));
    assert_eq!(
        Timeout::from(Duration::from_millis(7)).duration(),
        Duration::from_millis(7)
    );
}

#[test]
fn of_converts_length_and_unit() {
    // Timeout(length, unit) (Timeout.scala:28,44); the oracle cases TO-* fix the same values.
    let nanos = |length, unit| Timeout::of(length, unit).unwrap().duration().as_nanos();
    assert_eq!(nanos(3, TimeUnit::Nanoseconds), 3);
    assert_eq!(nanos(3, TimeUnit::Microseconds), 3_000);
    assert_eq!(nanos(3, TimeUnit::Milliseconds), 3_000_000);
    assert_eq!(nanos(3, TimeUnit::Seconds), 3_000_000_000);
    assert_eq!(nanos(3, TimeUnit::Minutes), 180_000_000_000);
    assert_eq!(nanos(3, TimeUnit::Hours), 10_800_000_000_000);
    assert_eq!(nanos(3, TimeUnit::Days), 259_200_000_000_000);
    assert_eq!(nanos(0, TimeUnit::Days), 0);
    assert_eq!(
        Timeout::of(1500, TimeUnit::Milliseconds).unwrap(),
        Timeout::new(Duration::from_millis(1500))
    );
}

const LIMIT_MESSAGE: &str =
    "requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)";

#[test]
fn of_accepts_the_largest_length_each_unit_allows() {
    // FiniteDuration is bounded by 2^63-1 ns (ca. 292 years); the oracle cases TO-MAX-* fix these edges.
    let max = |unit| match unit {
        TimeUnit::Nanoseconds => 9_223_372_036_854_775_807,
        TimeUnit::Microseconds => 9_223_372_036_854_775,
        TimeUnit::Milliseconds => 9_223_372_036_854,
        TimeUnit::Seconds => 9_223_372_036,
        TimeUnit::Minutes => 153_722_867,
        TimeUnit::Hours => 2_562_047,
        TimeUnit::Days => 106_751,
    };
    for unit in ALL_UNITS {
        assert!(Timeout::of(max(unit), unit).is_ok(), "{unit:?}");
    }
    // 2^63-1 ns is exactly 9223372036 s 854775807 ns.
    assert_eq!(
        Timeout::of(max(TimeUnit::Nanoseconds), TimeUnit::Nanoseconds)
            .unwrap()
            .duration(),
        Duration::new(9_223_372_036, 854_775_807)
    );
}

#[test]
fn of_rejects_lengths_beyond_the_range_with_the_pekko_message() {
    let beyond = |unit| match unit {
        TimeUnit::Nanoseconds => u64::MAX,
        TimeUnit::Microseconds => 9_223_372_036_854_776,
        TimeUnit::Milliseconds => 9_223_372_036_855,
        TimeUnit::Seconds => 9_223_372_037,
        TimeUnit::Minutes => 153_722_868,
        TimeUnit::Hours => 2_562_048,
        TimeUnit::Days => 106_752,
    };
    for unit in ALL_UNITS {
        let err = Timeout::of(beyond(unit), unit).unwrap_err();
        assert_eq!(err.java_class(), "IllegalArgumentException", "{unit:?}");
        assert_eq!(err.message(), LIMIT_MESSAGE, "{unit:?}");
    }
    // Long.MaxValue days, and a length no Scala Long can hold.
    assert!(Timeout::of(9_223_372_036_854_775_807, TimeUnit::Days).is_err());
    assert!(Timeout::of(u64::MAX, TimeUnit::Days).is_err());
}

const ALL_UNITS: [TimeUnit; 7] = [
    TimeUnit::Nanoseconds,
    TimeUnit::Microseconds,
    TimeUnit::Milliseconds,
    TimeUnit::Seconds,
    TimeUnit::Minutes,
    TimeUnit::Hours,
    TimeUnit::Days,
];
