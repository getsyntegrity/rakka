// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/util/{Timeout,Helpers}.scala, Done.scala, NotUsed.scala and PekkoException.scala (Apache-2.0).
//! Differential test against tools/oracle/util-core.jsh, run on the official Apache Pekko
//! 1.7.1 jars (not on Pekko compiled from upstream/pekko: see the golden header and the plan
//! docs/superpowers/plans/2026-10-30-ac-util-core.md). Each case below rebuilds one oracle case
//! with the Rust API and renders it with the format of the script; the result must equal the
//! golden line byte for byte.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{BuildHasher, RandomState};
use std::time::Duration;

use rakka_actor::actor::IllegalArgumentError;
use rakka_actor::util::helpers::{Requiring, base64, base64_append, to_root_lower_case};
use rakka_actor::util::{TimeUnit, Timeout};
use rakka_actor::{ConfigurationException, Done, NotUsed, PekkoException};

const GOLDEN: &str = include_str!("oracle/util-core.golden");

/// Golden cases that depend on an open Owner question or on a Java-only facade: not produced
/// by Rust on purpose. Each must exist in the golden and must not be produced below.
const PENDING: &[(&str, &str)] = &[
    ("TO-STR-01 Timeout(100, MILLISECONDS)", "Q-UTIL-2"),
    ("TO-STR-02 Timeout.zero", "Q-UTIL-2"),
    ("TO-STR-03 Timeout(1000, MILLISECONDS)", "Q-UTIL-2"),
    ("RQ-NULL 1.requiring(false, null)", "Q-UTIL-3"),
    ("RQ-LAZY msg not evaluated when cond holds", "Q-UTIL-3"),
    ("EX-NULL PekkoException(null)", "Q-UTIL-3"),
    (
        "EX-SUB ConfigurationException isA PekkoException",
        "Q-UTIL-4",
    ),
    (
        "DN-03 Done.done() is Done.getInstance()",
        "Java facade, no Rust counterpart (Q-UTIL-5)",
    ),
];

/// `esc` of the script: every UTF-16 unit outside printable ASCII becomes `\uXXXX`.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for unit in s.encode_utf16() {
        if (0x20..=0x7e).contains(&unit) {
            out.push(char::from(unit as u8));
        } else {
            out.push_str(&format!("\\u{unit:04X}"));
        }
    }
    out
}

/// Units in `TimeUnit.values()` order with the longest `length` that stays within 2^63-1 ns
/// (the oracle prints it as `length=`).
const UNITS: [(&str, TimeUnit, i64); 7] = [
    (
        "NANOSECONDS",
        TimeUnit::Nanoseconds,
        9_223_372_036_854_775_807,
    ),
    (
        "MICROSECONDS",
        TimeUnit::Microseconds,
        9_223_372_036_854_775,
    ),
    ("MILLISECONDS", TimeUnit::Milliseconds, 9_223_372_036_854),
    ("SECONDS", TimeUnit::Seconds, 9_223_372_036),
    ("MINUTES", TimeUnit::Minutes, 153_722_867),
    ("HOURS", TimeUnit::Hours, 2_562_047),
    ("DAYS", TimeUnit::Days, 106_751),
];

fn ok(fields: impl std::fmt::Display) -> String {
    format!("ok {fields}")
}

fn err(e: &IllegalArgumentError) -> String {
    format!("ERR {} [{}]", e.java_class(), e.message())
}

fn nanos(t: Timeout) -> String {
    format!("nanos={}", t.as_nanos())
}

fn of(length: i64, unit: TimeUnit) -> String {
    Timeout::of(length, unit).map_or_else(|e| err(&e), |t| ok(nanos(t)))
}

fn value<T: std::fmt::Display>(r: Result<T, IllegalArgumentError>) -> String {
    r.map_or_else(|e| err(&e), |v| ok(format!("value={v}")))
}

fn exception(class: &str, message: &str, cause: Option<String>) -> String {
    let cause = cause.map_or_else(|| "none".to_owned(), |c| format!("[{c}]"));
    ok(format!("class=[{class}] msg=[{message}] cause={cause}"))
}

fn rust_lines() -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut case = |id: String, line: String| lines.push((esc(&id), esc(&line)));

    // ---- Timeout (AC-UTIL-001)
    case("TO-01 Timeout.zero".into(), ok(nanos(Timeout::ZERO)));
    for (name, unit, max) in UNITS {
        case(format!("TO-02 Timeout(3, {name})"), of(3, unit));
        let at_max = Timeout::of(max, unit)
            .map_or_else(|e| err(&e), |t| ok(format!("length={max} {}", nanos(t))));
        case(format!("TO-MAX {name}"), at_max);
        // One past the bound; for NANOSECONDS that wraps to Long.MinValue, as in the script.
        case(format!("TO-OVER {name}"), of(max.wrapping_add(1), unit));
    }
    case("TO-03 Timeout(0, DAYS)".into(), of(0, TimeUnit::Days));
    case(
        "TO-04 Timeout(1500, MILLISECONDS)".into(),
        of(1500, TimeUnit::Milliseconds),
    );
    case(
        "TO-05 Timeout(Long.MaxValue, DAYS)".into(),
        of(i64::MAX, TimeUnit::Days),
    );
    let (a, b) = (
        Timeout::of(1000, TimeUnit::Milliseconds).unwrap(),
        Timeout::of(1, TimeUnit::Seconds).unwrap(),
    );
    let hasher = RandomState::new();
    case(
        "TO-06 eq Timeout(1000 ms) Timeout(1 s)".into(),
        ok(format!(
            "eq={} hashEq={}",
            a == b,
            hasher.hash_one(a) == hasher.hash_one(b)
        )),
    );
    let (one, two) = (
        Timeout::of(1, TimeUnit::Seconds).unwrap(),
        Timeout::of(2, TimeUnit::Seconds).unwrap(),
    );
    case(
        "TO-07 ne Timeout(1 s) Timeout(2 s)".into(),
        ok(format!("eq={}", one == two)),
    );
    // `Timeout.create(java.time.Duration)` goes through `TryFrom<std::time::Duration>`; the
    // negative cases cannot be a `std::time::Duration`, so they use the signed constructors.
    let created = |duration: Duration| {
        Timeout::try_from(duration).map_or_else(
            |e| format!("ERR {} [{}]", e.java_class(), e.message()),
            |t| ok(nanos(t)),
        )
    };
    case(
        "TO-08 create(ofMillis(250))".into(),
        created(Duration::from_millis(250)),
    );
    case(
        "TO-09 create(ofSeconds(9223372036, 854775807))".into(),
        created(Duration::new(9_223_372_036, 854_775_807)),
    );
    case(
        "TO-NEG-01 Timeout(-5, MILLISECONDS)".into(),
        of(-5, TimeUnit::Milliseconds),
    );
    case(
        "TO-NEG-02 Timeout(Long.MinValue, NANOSECONDS)".into(),
        Timeout::new(i64::MIN).map_or_else(|e| err(&e), |t| ok(nanos(t))),
    );
    case(
        "TO-NEG-03 create(ofMillis(-250))".into(),
        of(-250, TimeUnit::Milliseconds),
    );
    case(
        "TO-NEG-04 create(ofSeconds(9223372036, 854775808))".into(),
        created(Duration::new(9_223_372_036, 854_775_808)),
    );
    case(
        "TO-NEG-05 create(ofSeconds(Long.MAX_VALUE))".into(),
        created(Duration::from_secs(i64::MAX as u64)),
    );

    // ---- Helpers.base64 (AC-UTIL-002)
    for l in [
        0,
        1,
        25,
        26,
        51,
        52,
        61,
        62,
        63,
        64,
        65,
        4095,
        4096,
        1_i64 << 32,
        -1,
        -64,
        i64::MIN,
        i64::MAX,
    ] {
        case(format!("B64 {l}"), ok(format!("[{}]", base64(l))));
    }
    for (label, prefix, l) in [
        ("prefix x, 5", "x", 5),
        ("prefix temp/$, 64", "temp/$", 64),
        ("empty prefix, 0", "", 0),
    ] {
        let mut sb = prefix.to_owned();
        base64_append(l, &mut sb);
        case(format!("B64-SB {label}"), ok(format!("[{sb}]")));
    }

    // ---- Helpers.toRootLowerCase (AC-UTIL-002)
    for s in [
        "Hello WORLD 123",
        "TITLE",
        "",
        "\u{041F}\u{0420}\u{0418}\u{0412}\u{0415}\u{0422}",
        "\u{FF21}\u{FF22}",
        "\u{10400}",
        "\u{00DF}",
        "\u{0130}",
        "\u{0391}\u{03A3}",
        "\u{0391}\u{03A3}\u{0391}",
        "\u{03A3}",
    ] {
        case(
            format!("LC {}", esc(s)),
            ok(format!("[{}]", esc(&to_root_lower_case(s)))),
        );
    }

    // ---- Helpers.Requiring (AC-UTIL-002)
    case(
        "RQ-01 5.requiring(true)".into(),
        value(5.requiring(true, "unused")),
    );
    case(
        "RQ-02 5.requiring(false, msg)".into(),
        value(5.requiring(false, "idle-cpu-level must be between 1 and 10")),
    );
    case(
        "RQ-03 1.requiring(false, 42)".into(),
        value(1.requiring(false, 42)),
    );
    case(
        "RQ-04 1.requiring(false, empty)".into(),
        value(1.requiring(false, "")),
    );
    case(
        "RQ-05 3.requiring(_ > 0)".into(),
        value(3.requiring_with(|n| *n > 0, "must be > 0")),
    );
    case(
        "RQ-06 0.requiring(_ > 0)".into(),
        value(0.requiring_with(|n| *n > 0, "nr-of-selectors must be > 0")),
    );
    case(
        "RQ-07 abc.requiring(len == 3)".into(),
        value("abc".requiring_with(|s| s.len() == 3, "len")),
    );

    // ---- Done, NotUsed (AC-UTIL-003)
    let (done, not_used) = (Done, NotUsed);
    case(
        "DN-01 Done".into(),
        ok(format!("str=[{done}] eqSelf={}", done == Done)),
    );
    case(
        "DN-02 NotUsed".into(),
        ok(format!("str=[{not_used}] eqSelf={}", not_used == NotUsed)),
    );

    // ---- PekkoException, ConfigurationException (AC-UTIL-004)
    let inner = || std::io::Error::other("inner");
    let why = || std::io::Error::other("why");
    let e = PekkoException::new("boom");
    case(
        "EX-01 PekkoException(boom)".into(),
        exception(e.java_class(), e.message(), None),
    );
    let e = PekkoException::new("");
    case(
        "EX-02 PekkoException(empty)".into(),
        exception(e.java_class(), e.message(), None),
    );
    let e = PekkoException::with_cause("boom", inner());
    case(
        "EX-03 PekkoException(boom, cause)".into(),
        exception(
            e.java_class(),
            e.message(),
            e.cause().map(|c| c.to_string()),
        ),
    );
    let e = ConfigurationException::new("bad");
    case(
        "EX-04 ConfigurationException(bad)".into(),
        exception(e.java_class(), e.message(), None),
    );
    let e = ConfigurationException::with_cause("bad", why());
    case(
        "EX-05 ConfigurationException(bad, cause)".into(),
        exception(
            e.java_class(),
            e.message(),
            e.cause().map(|c| c.to_string()),
        ),
    );
    // A Java null cause is the same as no cause.
    let e = PekkoException::new("m");
    case(
        "EX-06 PekkoException(msg, null)".into(),
        exception(e.java_class(), e.message(), None),
    );

    lines
}

fn golden_lines() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in GOLDEN
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (id, rest) = line
            .split_once(" -> ")
            .unwrap_or_else(|| panic!("bad golden line: {line}"));
        assert!(
            map.insert(id.to_owned(), rest.to_owned()).is_none(),
            "duplicate golden id {id}"
        );
    }
    map
}

#[test]
fn rust_matches_the_pekko_oracle() {
    let golden = golden_lines();
    let pending: BTreeSet<&str> = PENDING.iter().map(|(id, _)| *id).collect();
    for id in &pending {
        assert!(
            golden.contains_key(*id),
            "PENDING case {id} is not in the golden"
        );
    }

    let mut produced = BTreeSet::new();
    let mut mismatches = Vec::new();
    for (id, line) in rust_lines() {
        assert!(
            !pending.contains(id.as_str()),
            "{id} is PENDING but Rust produces it"
        );
        assert!(produced.insert(id.clone()), "Rust produced {id} twice");
        match golden.get(&id) {
            None => mismatches.push(format!("{id}: not in the golden\n  rust:   {line}")),
            Some(expected) if *expected != line => {
                mismatches.push(format!("{id}\n  pekko:  {expected}\n  rust:   {line}"));
            }
            Some(_) => {}
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );

    let missing: Vec<&String> = golden
        .keys()
        .filter(|id| !produced.contains(*id) && !pending.contains(id.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "{} golden cases neither produced nor PENDING:\n{missing:#?}",
        missing.len()
    );
}
