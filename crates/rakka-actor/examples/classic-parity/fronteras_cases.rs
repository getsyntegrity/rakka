// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Timeout.scala (Apache-2.0).
//! Cases of the AC-UTIL-018.2 oracle (`FronterasOracle.java`): `Timeout.create(java.time.Duration)` goes through
//! `scala.jdk.DurationConverters.toScala` in Pekko 2.0.0 (the old `JavaDurationConverters` was removed). The Rust
//! twin is `TryFrom<std::time::Duration> for Timeout`.

use std::time::Duration;

use rakka_actor::util::Timeout;

fn created(duration: Duration) -> String {
    Timeout::try_from(duration).map_or_else(
        |e| format!("ERR {} [{}]", e.java_class(), e.message()),
        |t| format!("ok nanos={}", t.as_nanos()),
    )
}

/// `(case id, line)` pairs in the registration order of `FronterasOracle`; the line is the full oracle line
/// without the `<id> -> ` prefix.
pub fn cases() -> Vec<(String, String)> {
    let max_seconds_with_nanos = Timeout::try_from(Duration::new(9_223_372_036, 854_775_807));
    let max_as_nanos = Timeout::new(i64::MAX);
    let equal_to_max = match (max_seconds_with_nanos, max_as_nanos) {
        (Ok(a), Ok(b)) => format!("ok eq={} nanos={}", a == b, a.as_nanos()),
        _ => "unexpected".to_owned(),
    };
    vec![
        ("FR-DUR-01 create(Duration.ZERO)", created(Duration::ZERO)),
        (
            "FR-DUR-02 create(ofNanos(1))",
            created(Duration::from_nanos(1)),
        ),
        (
            "FR-DUR-03 create(ofSeconds(0, 999999999))",
            created(Duration::new(0, 999_999_999)),
        ),
        (
            "FR-DUR-04 create(ofSeconds(1))",
            created(Duration::from_secs(1)),
        ),
        (
            "FR-DUR-05 create(ofSeconds(86400, 5))",
            created(Duration::new(86_400, 5)),
        ),
        (
            "FR-DUR-06 create(ofSeconds(9223372036))",
            created(Duration::from_secs(9_223_372_036)),
        ),
        (
            "FR-DUR-07 create(ofSeconds(9223372037))",
            created(Duration::from_secs(9_223_372_037)),
        ),
        (
            "FR-DUR-08 create(ofSeconds(9223372035, 999999999))",
            created(Duration::new(9_223_372_035, 999_999_999)),
        ),
        (
            "FR-DUR-09 create(ofSeconds(9223372037, 1))",
            created(Duration::new(9_223_372_037, 1)),
        ),
        (
            "FR-DUR-10 create(ofSeconds(3600, 500000000))",
            created(Duration::new(3_600, 500_000_000)),
        ),
        (
            "FR-DUR-11 create(ofSeconds(Long.MAX_VALUE, 999999999))",
            created(Duration::new(i64::MAX as u64, 999_999_999)),
        ),
        (
            "FR-DUR-12 create(ofSeconds(61, 1))",
            created(Duration::new(61, 1)),
        ),
        (
            "FR-DUR-13 create(ofSeconds(9223372036, 854775807)) eq Timeout(Long.MaxValue ns)",
            equal_to_max,
        ),
    ]
    .into_iter()
    .map(|(id, line)| (id.to_owned(), line))
    .collect()
}
