// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).
//! `Timeout` (AC-UTIL-001), plan docs/superpowers/plans/2026-10-30-ac-util-core.md.
//!
//! Domain (Owner decision on Q-UTIL-1): negative, zero and positive durations within
//! `FiniteDuration`'s range, `[-(2^63-1), 2^63-1]` ns. `Long.MinValue` ns is rejected. The
//! edge values below were measured on the official Pekko 1.7.1 jars with
//! `scala-library-2.13.18` (jshell probe, `Timeout.apply` and `Timeout.create`), not deduced
//! from this crate.

use std::collections::HashSet;
use std::hash::{BuildHasher, RandomState};
use std::time::Duration;

use rakka_actor::util::{TimeUnit, Timeout};

const LIMIT_MESSAGE: &str =
    "requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)";
const MAX_NANOS: i64 = i64::MAX;
const MIN_NANOS: i64 = -i64::MAX;

/// Each unit with the longest `length` whose absolute value stays within 2^63-1 ns.
const ALL_UNITS: [(TimeUnit, i64, i64); 7] = [
    (TimeUnit::Nanoseconds, 9_223_372_036_854_775_807, 1),
    (TimeUnit::Microseconds, 9_223_372_036_854_775, 1_000),
    (TimeUnit::Milliseconds, 9_223_372_036_854, 1_000_000),
    (TimeUnit::Seconds, 9_223_372_036, 1_000_000_000),
    (TimeUnit::Minutes, 153_722_867, 60_000_000_000),
    (TimeUnit::Hours, 2_562_047, 3_600_000_000_000),
    (TimeUnit::Days, 106_751, 86_400_000_000_000),
];

fn assert_out_of_range<T: std::fmt::Debug>(
    result: Result<T, rakka_actor::actor::IllegalArgumentError>,
    what: &str,
) {
    let err = result.unwrap_err();
    assert_eq!(err.java_class(), "IllegalArgumentException", "{what}");
    assert_eq!(err.message(), LIMIT_MESSAGE, "{what}");
}

#[test]
fn new_accepts_negative_zero_and_positive_nanoseconds() {
    assert_eq!(Timeout::new(-5_000_000).unwrap().as_nanos(), -5_000_000);
    assert_eq!(Timeout::new(0).unwrap().as_nanos(), 0);
    assert_eq!(Timeout::new(100_000_000).unwrap().as_nanos(), 100_000_000);
}

#[test]
fn new_accepts_both_exact_limits_and_rejects_the_values_just_outside() {
    assert_eq!(Timeout::new(MAX_NANOS).unwrap().as_nanos(), MAX_NANOS);
    assert_eq!(Timeout::new(MIN_NANOS).unwrap().as_nanos(), MIN_NANOS);
    // Pekko: Timeout(Long.MinValue, NANOSECONDS) -> IllegalArgumentException (oracle TO-NEG-02).
    assert_out_of_range(Timeout::new(i64::MIN), "Long.MinValue");
}

#[test]
fn of_converts_length_and_unit_for_negative_zero_and_positive_lengths() {
    for (unit, _, unit_nanos) in ALL_UNITS {
        for length in [-3, 0, 3] {
            let timeout = Timeout::of(length, unit).unwrap();
            assert_eq!(timeout.as_nanos(), length * unit_nanos, "{unit:?} {length}");
        }
    }
    assert_eq!(
        Timeout::of(-5, TimeUnit::Milliseconds).unwrap().as_nanos(),
        -5_000_000
    );
}

#[test]
fn of_accepts_both_exact_limits_of_every_unit() {
    for (unit, max_length, unit_nanos) in ALL_UNITS {
        assert_eq!(
            Timeout::of(max_length, unit).unwrap().as_nanos(),
            max_length * unit_nanos,
            "{unit:?} +max"
        );
        assert_eq!(
            Timeout::of(-max_length, unit).unwrap().as_nanos(),
            -max_length * unit_nanos,
            "{unit:?} -max"
        );
    }
    assert_eq!(
        Timeout::of(-9_223_372_036_854_775_807, TimeUnit::Nanoseconds).unwrap(),
        Timeout::new(MIN_NANOS).unwrap()
    );
}

#[test]
fn of_rejects_the_length_just_outside_each_limit_with_the_pekko_message() {
    for (unit, max_length, _) in ALL_UNITS {
        if max_length < i64::MAX {
            assert_out_of_range(
                Timeout::of(max_length + 1, unit),
                &format!("{unit:?} +max+1"),
            );
            assert_out_of_range(
                Timeout::of(-max_length - 1, unit),
                &format!("{unit:?} -max-1"),
            );
        }
    }
    // For NANOSECONDS the length past the bound is Long.MinValue (oracle TO-OVER NANOSECONDS).
    assert_out_of_range(Timeout::of(i64::MIN, TimeUnit::Nanoseconds), "MIN ns");
}

#[test]
fn of_rejects_a_conversion_that_overflows_a_long_in_every_unit() {
    for (unit, _, _) in ALL_UNITS {
        if unit != TimeUnit::Nanoseconds {
            assert_out_of_range(
                Timeout::of(i64::MAX, unit),
                &format!("{unit:?} Long.MaxValue"),
            );
            assert_out_of_range(
                Timeout::of(i64::MIN, unit),
                &format!("{unit:?} Long.MinValue"),
            );
        }
    }
    // Long.MaxValue days (oracle TO-05).
    assert_out_of_range(Timeout::of(i64::MAX, TimeUnit::Days), "TO-05");
}

#[test]
fn equality_and_hash_are_by_value_not_by_unit_including_negatives() {
    // Timeout(1000.millis) == Timeout(1.second) in Pekko (case class over FiniteDuration).
    let seconds = Timeout::of(1, TimeUnit::Seconds).unwrap();
    assert_eq!(Timeout::of(1000, TimeUnit::Milliseconds).unwrap(), seconds);
    assert_ne!(seconds, Timeout::of(2, TimeUnit::Seconds).unwrap());
    // Probed on the jars: Timeout(-1000.millis) equals Timeout(-1.second) and hashes alike.
    let negative_seconds = Timeout::of(-1, TimeUnit::Seconds).unwrap();
    let negative_millis = Timeout::of(-1000, TimeUnit::Milliseconds).unwrap();
    assert_eq!(negative_seconds, negative_millis);
    assert_ne!(negative_seconds, seconds);
    let set: HashSet<Timeout> = [negative_seconds].into_iter().collect();
    assert!(set.contains(&negative_millis));
    // Equal values hash alike, whatever the unit they were built with.
    let state = RandomState::new();
    assert_eq!(
        state.hash_one(negative_seconds),
        state.hash_one(negative_millis)
    );
}

#[test]
fn zero_is_the_zero_timeout_whatever_the_unit() {
    // Timeout.zero (Timeout.scala:39); oracle TO-01 and TO-03.
    assert_eq!(Timeout::ZERO.as_nanos(), 0);
    assert_eq!(Timeout::ZERO, Timeout::new(0).unwrap());
    assert_eq!(Timeout::ZERO, Timeout::of(0, TimeUnit::Days).unwrap());
    assert_eq!(Timeout::ZERO, Timeout::try_from(Duration::ZERO).unwrap());
}

#[test]
fn a_std_duration_converts_with_try_from_and_keeps_its_nanoseconds() {
    // Timeout.create(java.time.Duration) / implicit durationToTimeout, for the non-negative range.
    let timeout = Timeout::try_from(Duration::from_millis(250)).unwrap();
    assert_eq!(timeout.as_nanos(), 250_000_000);
    assert_eq!(timeout, Timeout::of(250, TimeUnit::Milliseconds).unwrap());
    // 2^63-1 ns is exactly 9223372036 s 854775807 ns (oracle TO-09).
    let largest = Timeout::try_from(Duration::new(9_223_372_036, 854_775_807)).unwrap();
    assert_eq!(largest.as_nanos(), MAX_NANOS);
}

#[test]
fn a_std_duration_beyond_the_range_fails_like_scala_duration_converters_in_pekko_2_0_0() {
    // Pekko 2.0.0 `Timeout.create` goes through `scala.jdk.DurationConverters.toScala` (oracle TO-NEG-04,
    // TO-NEG-05): a duration with seconds and nanos that overflows is an IllegalArgumentException naming the
    // Java duration; one without nanos reaches `FiniteDuration(seconds, SECONDS)` and fails its requirement.
    let requirement = "requirement failed: Duration is limited to +-(2^63-1)ns (ca. 292 years)";
    let cannot = |iso: &str| format!("Java duration {iso} cannot be expressed as a Scala duration");
    for (too_long, message) in [
        (
            Duration::new(9_223_372_036, 854_775_808),
            cannot("PT2562047H47M16.854775808S"),
        ),
        (Duration::from_secs(i64::MAX as u64), requirement.to_owned()),
        (Duration::from_secs(u64::MAX), requirement.to_owned()),
        (Duration::MAX, cannot("PT5124095576030431H15.999999999S")),
        (
            Duration::new(9_223_372_037, 500_000_000),
            cannot("PT2562047H47M17.5S"),
        ),
    ] {
        let err = Timeout::try_from(too_long).unwrap_err();
        assert_eq!(err.java_class(), "IllegalArgumentException", "{too_long:?}");
        assert_eq!(err.message(), message, "{too_long:?}");
    }
}

#[test]
fn no_public_path_builds_a_timeout_outside_the_range() {
    // Every constructor funnels through one invariant: |nanos| <= 2^63-1.
    let built = [
        Timeout::new(MAX_NANOS),
        Timeout::new(MIN_NANOS),
        Timeout::of(106_751, TimeUnit::Days),
        Timeout::of(-106_751, TimeUnit::Days),
    ];
    for timeout in built {
        assert_ne!(timeout.unwrap().as_nanos(), i64::MIN);
    }
    assert!(Timeout::new(i64::MIN).is_err());
    assert!(Timeout::of(i64::MIN, TimeUnit::Nanoseconds).is_err());
}
