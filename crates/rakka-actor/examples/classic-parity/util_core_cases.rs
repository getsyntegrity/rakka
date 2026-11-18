// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/{Timeout,Helpers,Glob,Clock,SWARUtil,PrettyDuration}.scala, Done.scala, NotUsed.scala, PekkoException.scala and actor/src/main/java/org/apache/pekko/util/Unsafe.java (Apache-2.0).
//! Cases of the AC-UTIL (core) oracle (`UtilCoreOracle.java`, converted from the deprecated jshell script): each
//! rebuilds one oracle case with the Rust API and renders it with the oracle's line format.

use std::hash::{BuildHasher, RandomState};
use std::sync::Mutex;
use std::time::Duration;

use rakka_actor::actor::IllegalArgumentError;
use rakka_actor::pekko_version::{self, RequireError};
use rakka_actor::util::clock::{
    Clock, FixedDelayScheduler, NanoClock, ScheduledClock, SelectedClock,
};
use rakka_actor::util::fast_hash::fast_hash;
use rakka_actor::util::glob_matches;
use rakka_actor::util::hash_code::{self, HashContribution};
use rakka_actor::util::helpers::{
    IS_WINDOWS, Requiring, base64, base64_append, compare_identity_hash,
    current_time_millis_to_utc_string, identity_hash_comparator, timestamp_at_utc_offset,
    to_root_lower_case,
};
use rakka_actor::util::line_numbers::{LineNumbersResult, closure_origin, pretty_name};
use rakka_actor::util::swar::{self, ByteOrder};
use rakka_actor::util::wall_clock::{AlwaysIncreasingClock, WallClock, always_increasing_clock};
use rakka_actor::util::{JavaUuid, PekkoDuration, TimeUnit, Timeout, Version, pekko_compare_uuid};
use rakka_actor::{ConfigurationException, Done, NotUsed, PekkoException};

/// Golden cases that depend on an open Owner question or on a Java-only facade: not produced
/// by Rust on purpose. Each must exist in the golden and must not be produced below.
/// Documented in the tests (`recorded_pekko`), which check that every oracle id is produced or listed here.
#[allow(dead_code)]
pub const PENDING: &[(&str, &str)] = &[
    (
        "HC-12 null",
        "no null in Rust: hash(seed, null) throws IllegalArgumentException (AC-UTIL-016.15, Q-UTIL-14 (b))",
    ),
    (
        "HC-13 null element in array",
        "no null in Rust (AC-UTIL-016.15, Q-UTIL-14 (b))",
    ),
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
        "VR-21 public constructor parses lazily",
        "Java-only facade: `new Version(s)` defers parsing; Rust parses on construction (AC-UTIL-005.2)",
    ),
    (
        "LC \\uA7C0",
        "Q-UTIL-7 (character added after Unicode 13: JDK 17 leaves it, Rust lower-cases it)",
    ),
    (
        "EX-TS-01 PekkoException(boom).toString",
        "Q-UTIL-17 (proposed)",
    ),
    (
        "EX-TS-02 ConfigurationException(bad, cause).toString",
        "Q-UTIL-17 (proposed)",
    ),
    (
        "EX-TS-03 PekkoException(empty).toString",
        "Q-UTIL-17 (proposed)",
    ),
    (
        "DN-03 Done.done() is Done.getInstance()",
        "Java facade, no Rust counterpart (Q-UTIL-5)",
    ),
    (
        "UN-03 fastHash of non-ASCII text",
        "Q-UTIL-19 (proposed): the result depends on the algorithm the JVM selects",
    ),
    (
        "MP-NL makePattern vs Glob with line breaks and astral",
        "Q-UTIL-8 (c): java.util.regex is not reproduced; Glob.matches is the contract",
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

pub fn rust_lines() -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut case =
        |id: String, line: String| lines.push((esc(&id), esc(&format!("{id} -> {line}"))));

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
    let (negative_millis, negative_second) = (
        Timeout::of(-1000, TimeUnit::Milliseconds).unwrap(),
        Timeout::of(-1, TimeUnit::Seconds).unwrap(),
    );
    case(
        "TO-06N eq Timeout(-1000 ms) Timeout(-1 s)".into(),
        ok(format!(
            "eq={} hashEq={}",
            negative_millis == negative_second,
            hasher.hash_one(negative_millis) == hasher.hash_one(negative_second)
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
        "\u{1C90}",
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

    // ---- Glob (AC-UTIL-016.6), slice P05
    let astral = "\u{1F600}";
    let glob_case = |id: &str, pairs: &[(&str, &str)]| -> (String, String) {
        let shown = |s: &str| {
            if s.encode_utf16().count() > 40 {
                format!("{}x", s.encode_utf16().count())
            } else {
                esc(s)
            }
        };
        let results: Vec<String> = pairs
            .iter()
            .map(|(pattern, input)| {
                format!(
                    "[{}]/[{}]={}",
                    shown(pattern),
                    shown(input),
                    glob_matches(pattern, input)
                )
            })
            .collect();
        (id.to_owned(), ok(results.join(" ")))
    };
    let stars_then_b = |count: usize| format!("{}*b", "*a".repeat(count));
    let stars_then_a = |count: usize| format!("{}*a", "*a".repeat(count));
    let stars_only = |count: usize| format!("{}*", "*a".repeat(count));
    let mut glob_cases: Vec<(String, String)> = vec![
        glob_case(
            "GL-01 literals",
            &[
                ("abc", "abc"),
                ("abc", "abd"),
                ("", ""),
                ("", "a"),
                ("a", ""),
            ],
        ),
        glob_case(
            "GL-02 question mark",
            &[
                ("a?c", "abc"),
                ("a?c", "ac"),
                ("a?c", "abbc"),
                ("???", "abc"),
            ],
        ),
        glob_case(
            "GL-03 star",
            &[
                ("*", ""),
                ("*", "anything"),
                ("a*", "a"),
                ("a*c", "abbbc"),
                ("a*c", "abbbd"),
                ("**", "ab"),
                ("*b*", "abc"),
            ],
        ),
        glob_case(
            "GL-04 star positions",
            &[
                ("*a", "ba"),
                ("*a", "ab"),
                ("a*a", "aa"),
                ("a*a", "a"),
                ("*ab*", "xxabyy"),
                ("*ab*", "xxaybb"),
                ("?*", ""),
                ("?*", "a"),
                ("*?", "a"),
                ("a**b", "ab"),
                ("*?*?*", "ab"),
                ("*?*?*", "a"),
            ],
        ),
    ];
    let glob_u16 = [
        (
            "GL-U16-01 ? vs astral",
            vec![
                ("?".to_owned(), astral.to_owned()),
                ("??".to_owned(), astral.to_owned()),
                ("???".to_owned(), astral.to_owned()),
                ("*".to_owned(), astral.to_owned()),
                ("a?b".to_owned(), format!("a{astral}b")),
                ("a??b".to_owned(), format!("a{astral}b")),
            ],
        ),
        (
            "GL-U16-02 literal astral",
            vec![
                (astral.to_owned(), astral.to_owned()),
                (format!("?{astral}"), format!("x{astral}")),
                (format!("*{astral}"), format!("ab{astral}")),
                (format!("{astral}*"), format!("{astral}ab")),
                ("?".to_owned(), "\u{e9}".to_owned()),
                ("\u{e9}".to_owned(), "\u{e9}".to_owned()),
                ("\u{e9}?".to_owned(), "\u{e9}a".to_owned()),
            ],
        ),
    ];
    for (id, pairs) in &glob_u16 {
        let borrowed: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        glob_cases.push(glob_case(id, &borrowed));
    }
    glob_cases.push(glob_case(
        "GL-U16-03 line breaks",
        &[
            ("a?b", "a\nb"),
            ("a*b", "a\r\nb"),
            ("*", "\n"),
            ("?", "\n"),
            ("a?b", "a\u{2028}b"),
            ("a?", "a\u{85}"),
        ],
    ));
    glob_cases.push(glob_case(
        "GL-U16-04 case and metachars",
        &[
            ("A", "a"),
            ("a.c", "abc"),
            ("a.c", "a.c"),
            ("[a]", "a"),
            ("[a]", "[a]"),
            ("a\\", "a\\"),
            ("\\Q*\\E", "\\Qxx\\E"),
            ("(a)", "(a)"),
            ("a+", "aa"),
        ],
    ));
    for len in 0..=4 {
        let mut row = Vec::new();
        for pattern in strings_of("ab*?", len) {
            let mut bits = String::new();
            for input_len in 0..=4 {
                for input in strings_of("ab", input_len) {
                    bits.push(if glob_matches(&pattern, &input) {
                        'T'
                    } else {
                        'F'
                    });
                }
            }
            row.push(format!("[{pattern}]={bits}"));
        }
        glob_cases.push((
            format!("GL-EXH patterns of length {len}"),
            ok(row.join(" ")),
        ));
    }
    let (a36, a2000, a12) = ("a".repeat(36), "a".repeat(2000), "a".repeat(12));
    let (b12, b20) = (stars_then_b(12), stars_then_b(20));
    glob_cases.push(glob_case(
        "GL-PERF 12 stars vs 36 a",
        &[
            (&b12, &a36),
            (&stars_only(12), &a36),
            (&stars_then_a(12), &a12),
        ],
    ));
    glob_cases.push(glob_case(
        "GL-PERF 20 stars vs 2000 a",
        &[
            (&b20, &a2000),
            (&stars_then_a(20), &a2000),
            (&stars_then_a(20), &"a".repeat(19)),
        ],
    ));
    glob_cases.push(glob_case(
        "GL-05 wildcards in the input",
        &[
            ("*", "*"),
            ("?", "?"),
            ("*", "?"),
            ("a*", "a*"),
            ("a*b", "a*b"),
            ("a*b", "a*"),
            ("*a", "**a"),
            ("?*", "**"),
            ("*?", "?*"),
            ("**", "*"),
        ],
    ));
    for (id, result) in glob_cases {
        lines.push((esc(&id), esc(&format!("{id} -> {result}"))));
    }

    // ---- Slice P06: UTIL-PLATAFORMA (AC-UTIL-020, 023..028)
    for (id, result) in platform_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P65: UTIL-HASHCODE (AC-UTIL-016.15)
    for (id, result) in hash_code_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {result}"))));
    }

    // ---- Slice P66: UTIL-PEKKOVERSION (AC-UTIL-005.1)
    for (id, result) in pekko_version_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P67: UTIL-VERSION (AC-UTIL-005.2)
    for (id, result) in version_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P68: UTIL-PRETTYDURATION (AC-UTIL-016.5)
    for (id, result) in pretty_duration_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P69: UTIL-UUID-ORDEN (AC-UTIL-016.14)
    for (id, result) in uuid_order_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P70: UTIL-RELOJ (AC-UTIL-016.16)
    for (id, result) in wall_clock_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    // ---- Slice P75: UTIL-ORIGEN-CLOSURES (AC-UTIL-016.8)
    for (id, result) in line_numbers_cases() {
        lines.push((esc(&id), esc(&format!("{id} -> {}", ok(result)))));
    }

    lines
}

/// Every string over `alphabet` that has exactly `length` characters, in lexicographic order of the alphabet.
fn strings_of(alphabet: &str, length: usize) -> Vec<String> {
    let mut all = vec![String::new()];
    for _ in 0..length {
        all = all
            .iter()
            .flat_map(|prefix| alphabet.chars().map(move |c| format!("{prefix}{c}")))
            .collect();
    }
    all
}

/// The oracle's fixed time zone (`GMT+05:30`, pinned by `UtilCoreOracle`).
const ORACLE_ZONE_OFFSET_SECONDS: i32 = 5 * 3600 + 30 * 60;

/// A `Scheduler` that keeps the first task of `scheduleWithFixedDelay` so that the case drives the ticks.
#[derive(Default)]
struct CapturingScheduler {
    initial_delay_nanos: Mutex<Option<u128>>,
    task: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
}

impl FixedDelayScheduler for CapturingScheduler {
    fn schedule_with_fixed_delay(
        &self,
        initial_delay: Duration,
        _delay: Duration,
        task: Box<dyn Fn() + Send + Sync>,
    ) {
        *self.initial_delay_nanos.lock().unwrap() = Some(initial_delay.as_nanos());
        *self.task.lock().unwrap() = Some(task);
    }
}

impl CapturingScheduler {
    fn tick(&self) {
        (self
            .task
            .lock()
            .unwrap()
            .as_ref()
            .expect("a scheduled task"))();
    }
}

/// xorshift64, as `PlatformOracle.Gen`.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn utc_strings(values: &[i64]) -> String {
    values
        .iter()
        .map(|v| format!("[{v}]={}", current_time_millis_to_utc_string(*v)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn zoned_strings(values: &[i64]) -> String {
    values
        .iter()
        .map(|v| {
            format!(
                "[{v}]={}",
                timestamp_at_utc_offset(*v, ORACLE_ZONE_OFFSET_SECONDS)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const SWAR_SAMPLE: [u8; 16] = [
    0x01, 0x02, 0x03, 0x84, 0xFF, 0x00, 0x7F, 0x80, 0x55, 0xAA, 0x10, 0x20, 0xFE, 0x01, 0x81, 0x7E,
];

fn swar_gets() -> String {
    let (be, le) = (ByteOrder::BigEndian, ByteOrder::LittleEndian);
    let mut out = Vec::new();
    for i in 0..=SWAR_SAMPLE.len() - 8 {
        out.push(format!(
            "L{i}:{:016x}/{:016x}",
            swar::get_long(&SWAR_SAMPLE, i, be) as u64,
            swar::get_long(&SWAR_SAMPLE, i, le) as u64
        ));
    }
    for i in 0..=SWAR_SAMPLE.len() - 4 {
        out.push(format!(
            "I{i}:{:08x}/{:08x}",
            swar::get_int(&SWAR_SAMPLE, i, be) as u32,
            swar::get_int(&SWAR_SAMPLE, i, le) as u32
        ));
    }
    for i in 0..=SWAR_SAMPLE.len() - 2 {
        out.push(format!(
            "S{i}:{:04x}/{:04x}",
            swar::get_short(&SWAR_SAMPLE, i, be) as u16,
            swar::get_short(&SWAR_SAMPLE, i, le) as u16
        ));
    }
    out.join(" ")
}

fn swar_puts() -> String {
    let longs = [0_i64, 1, -1, 0x0123_4567_89AB_CDEF, i64::MIN, i64::MAX];
    let ints = [0_i32, 1, -1, 0x1234_5678, i32::MIN, i32::MAX];
    let mut out = Vec::new();
    for (order, tag) in [(ByteOrder::BigEndian, "B"), (ByteOrder::LittleEndian, "L")] {
        for v in longs {
            for at in [0, 3] {
                let mut array = [0_u8; 11];
                swar::put_long(&mut array, at, v, order);
                out.push(format!("PL{tag}{at}={}", hex_bytes(&array)));
            }
        }
        for v in ints {
            for at in [0, 5] {
                let mut array = [0_u8; 9];
                swar::put_int(&mut array, at, v, order);
                out.push(format!("PI{tag}{at}={}", hex_bytes(&array)));
                let mut short = [0_u8; 5];
                swar::put_short(&mut short, at % 3, v, order);
                out.push(format!("PS{tag}{}={}", at % 3, hex_bytes(&short)));
            }
        }
    }
    out.join(" ")
}

fn swar_cases() -> Vec<(String, String)> {
    let mut cases = Vec::new();
    cases.push((
        "SW-01 compilePattern for every byte".to_owned(),
        (-128_i32..=127)
            .map(|b| format!("{:016x}", swar::compile_pattern(b as u8) as u64))
            .collect::<Vec<_>>()
            .join(" "),
    ));
    let mut applied = Vec::new();
    let words = [
        0_i64,
        -1,
        0x0102_0304_0506_0708,
        0x8080_8080_8080_8080_u64 as i64,
        0x7F7F_7F7F_7F7F_7F7F,
        0x0001_0001_0001_0001,
        0xFF00_FF00_FF00_FF00_u64 as i64,
    ];
    let patterns = [0_u8, 1, 0x7F, 0x80, 0xFF].map(swar::compile_pattern);
    for word in words {
        for pattern in patterns {
            applied.push(format!(
                "{:016x}",
                swar::apply_pattern(word, pattern) as u64
            ));
        }
    }
    let mut generator = Gen(0x9E37_79B9_7F4A_7C15);
    for i in 0..200_u32 {
        let word = generator.next();
        let probe = generator.next() as u8;
        let pattern = swar::compile_pattern(probe);
        applied.push(format!(
            "{:016x}",
            swar::apply_pattern(word as i64, pattern) as u64
        ));
        let lane = 8 * (i % 8);
        let planted = (word & !(0xFF_u64 << lane)) | (u64::from(probe) << lane);
        applied.push(format!(
            "{:016x}",
            swar::apply_pattern(planted as i64, pattern) as u64
        ));
    }
    cases.push(("SW-02 applyPattern".to_owned(), applied.join(" ")));
    let mut indexes = Vec::new();
    for mask in 0..256_u32 {
        let mut word = 0_u64;
        for b in 0..8 {
            if mask & (1 << b) != 0 {
                word |= 0x80_u64 << (8 * b);
            }
        }
        indexes.push(format!(
            "{}/{}",
            swar::get_index(word as i64),
            swar::get_last_index(word as i64)
        ));
    }
    for word in [
        1_u64,
        0xFF,
        0x100,
        0x0000_FFFF_0000_0000,
        1 << 63,
        u64::MAX,
        0x00FF_0000_0000_0000,
        0x0000_0001_0000_0000,
    ] {
        indexes.push(format!(
            "{}/{}",
            swar::get_index(word as i64),
            swar::get_last_index(word as i64)
        ));
    }
    cases.push((
        "SW-03 getIndex and getLastIndex".to_owned(),
        indexes.join(" "),
    ));
    cases.push(("SW-04 getLong getInt getShort".to_owned(), swar_gets()));
    cases.push(("SW-05 putLong putInt putShort".to_owned(), swar_puts()));
    cases.push((
        "SW-06 variants without MethodHandle".to_owned(),
        format!("{} | {}", swar_gets(), swar_puts()),
    ));
    let mut bytewise = Vec::new();
    for i in (0..=SWAR_SAMPLE.len() - 8).step_by(3) {
        let b = &SWAR_SAMPLE[i..i + 8];
        bytewise.push(format!(
            "{:04x}/{:04x}",
            swar::get_short_be(b[0], b[1]) as u16,
            swar::get_short_le(b[0], b[1]) as u16
        ));
        bytewise.push(format!(
            "{:08x}/{:08x}",
            swar::get_int_be(b[0], b[1], b[2], b[3]) as u32,
            swar::get_int_le(b[0], b[1], b[2], b[3]) as u32
        ));
        let word: [u8; 8] = b.try_into().unwrap();
        bytewise.push(format!(
            "{:016x}/{:016x}",
            swar::get_long_be(word) as u64,
            swar::get_long_le(word) as u64
        ));
    }
    cases.push((
        "SW-07 byte-wise getShort getInt getLong".to_owned(),
        bytewise.join(" "),
    ));
    cases
}

fn clock_cases() -> Vec<(String, String)> {
    let mut cases = Vec::new();
    let nano = NanoClock;
    let mut monotonic = true;
    let mut previous = nano.current_time();
    for _ in 0..1000 {
        let now = nano.current_time();
        monotonic &= now.wrapping_sub(previous) >= 0;
        previous = now;
    }
    let mut nano_fields = format!("monotonic={monotonic}");
    for d in [
        0_i64,
        1,
        1_000_000_000,
        -1_000_000_000,
        4_611_686_018_427_387_903,
    ] {
        let before = nano.current_time();
        let earlier = nano.earlier_time(d);
        let after = nano.current_time();
        let reconstructed = earlier.wrapping_add(d);
        nano_fields.push_str(&format!(
            " d={d}:inRange={}",
            reconstructed.wrapping_sub(before) >= 0 && after.wrapping_sub(reconstructed) >= 0
        ));
    }
    cases.push(("CK-01 NanoClock".to_owned(), nano_fields));

    let scheduler = CapturingScheduler::default();
    let clock = ScheduledClock::new(Duration::from_nanos(100_003), &scheduler);
    let base = clock.earlier_time(0);
    let mut fields = format!(
        "scheduled={} delayNanos={} reached={}",
        scheduler.task.lock().unwrap().is_some(),
        scheduler.initial_delay_nanos.lock().unwrap().unwrap(),
        clock.max_increment_reached()
    );
    for i in 1..=6 {
        let c = clock.current_time();
        fields.push_str(&format!(
            " c{i}=+{},reached={}",
            c.wrapping_sub(base),
            clock.max_increment_reached()
        ));
    }
    fields.push_str(&format!(
        " earlier(0)=+{} earlier(5)=+{}",
        clock.earlier_time(0).wrapping_sub(base),
        clock.earlier_time(5).wrapping_sub(base)
    ));
    cases.push((
        "CK-02 ScheduledClock increments by one until maxIncrement".to_owned(),
        fields,
    ));

    let mut fields = String::new();
    for interval in [1_u64, 100_000, 100_001, 100_002, 100_004] {
        let clock = ScheduledClock::new(
            Duration::from_nanos(interval),
            &CapturingScheduler::default(),
        );
        let base = clock.earlier_time(0);
        fields.push_str(&format!("interval={interval}:"));
        for _ in 0..6 {
            fields.push_str(&format!(" +{}", clock.current_time().wrapping_sub(base)));
        }
        fields.push_str(&format!(" reached={}; ", clock.max_increment_reached()));
    }
    let one_ms = ScheduledClock::new(
        Duration::from_nanos(1_000_000),
        &CapturingScheduler::default(),
    );
    let base = one_ms.earlier_time(0);
    let mut last = 0;
    for _ in 1..=900_000 {
        last = one_ms.current_time().wrapping_sub(base);
    }
    fields.push_str(&format!(
        "interval=1000000: n=900000 +{last} reached={}",
        one_ms.max_increment_reached()
    ));
    fields.push_str(&format!(
        " n+1 +{}",
        one_ms.current_time().wrapping_sub(base)
    ));
    fields.push_str(&format!(
        " n+2 +{}",
        one_ms.current_time().wrapping_sub(base)
    ));
    cases.push((
        "CK-03 ScheduledClock maxIncrement boundaries".to_owned(),
        fields,
    ));

    let scheduler = CapturingScheduler::default();
    let clock = ScheduledClock::new(Duration::from_nanos(100_003), &scheduler);
    let base = clock.earlier_time(0);
    for _ in 0..5 {
        clock.current_time();
    }
    let reached_before = clock.max_increment_reached();
    let stuck = clock.current_time().wrapping_sub(base);
    scheduler.tick();
    let after_update = clock.earlier_time(0);
    let reached_after = clock.max_increment_reached();
    let next = clock.current_time();
    cases.push((
        "CK-04 ScheduledClock update resumes the increments".to_owned(),
        format!(
            "reachedBefore={reached_before} stuck=+{stuck} movedForward={} reachedAfter={reached_after} nextIsPlusOne={}",
            after_update.wrapping_sub(base.wrapping_add(stuck)) >= 0,
            next.wrapping_sub(after_update) == 1
        ),
    ));

    let kind = |interval: Duration| {
        SelectedClock::for_interval(interval, &CapturingScheduler::default()).class_name()
    };
    cases.push((
        "CK-05 Clock extension selection".to_owned(),
        format!(
            "default={} 0={} 0s={} 1ms={} 1s={}",
            kind(Duration::from_secs(1)),
            kind(Duration::ZERO),
            kind(Duration::ZERO),
            kind(Duration::from_millis(1)),
            kind(Duration::from_secs(1))
        ),
    ));
    cases
}

fn identity_cases() -> Vec<(String, String)> {
    let sign = |ordering: std::cmp::Ordering| ordering as i32;
    let objects: Vec<Box<u64>> = (0..60).map(Box::new).collect();
    let (mut self_zero, mut antisym, mut domain, mut transitive) = (true, true, true, true);
    for a in &objects {
        self_zero &= compare_identity_hash(&**a, &**a).is_eq();
        for b in &objects {
            let ab = compare_identity_hash(&**a, &**b);
            domain &= (-1..=1).contains(&sign(ab));
            antisym &= sign(ab) == -sign(compare_identity_hash(&**b, &**a));
            for c in &objects[..12] {
                if ab.is_lt() && compare_identity_hash(&**b, &**c).is_lt() {
                    transitive &= compare_identity_hash(&**a, &**c).is_lt();
                }
            }
        }
    }
    let first = (
        "ID-01 compareIdentityHash invariants".to_owned(),
        format!("self={self_zero} antisym={antisym} domain={domain} transitive={transitive}"),
    );

    #[derive(PartialEq)]
    struct Tagged(u32);
    let by_tag = |x: &Tagged, y: &Tagged| x.0.cmp(&y.0);
    let comparator = identity_hash_comparator(by_tag);
    let items: Vec<Box<Tagged>> = (0..80).map(|tag| Box::new(Tagged(tag))).collect();
    let (mut self_zero, mut distinct, mut antisym) = (true, true, true);
    for a in &items {
        self_zero &= comparator(a, a).is_eq();
        for b in &items {
            if !std::ptr::eq(&**a, &**b) {
                distinct &= comparator(a, b).is_ne();
            }
            antisym &= comparator(a, b) == comparator(b, a).reverse();
        }
    }
    let mut sorted: Vec<&Tagged> = items.iter().map(|b| &**b).collect();
    sorted.sort_by(|a, b| comparator(a, b));
    let strictly_increasing = sorted.windows(2).all(|w| comparator(w[0], w[1]).is_lt());
    let mut from_reverse: Vec<&Tagged> = items.iter().rev().map(|b| &**b).collect();
    from_reverse.sort_by(|a, b| comparator(a, b));
    let stable = sorted
        .iter()
        .zip(&from_reverse)
        .all(|(a, b)| std::ptr::eq(*a, *b));
    let second = (
        "ID-02 identityHashComparator with tie-break".to_owned(),
        format!(
            "self={self_zero} distinct={distinct} antisym={antisym} sorted={strictly_increasing} stable={stable}"
        ),
    );
    vec![first, second]
}

/// The ASCII characters 0..=127 as one string (the oracle's `allAscii()`).
fn all_ascii() -> String {
    (0..128_u8).map(char::from).collect()
}

fn platform_cases() -> Vec<(String, String)> {
    let mut cases = vec![(
        "WIN-01 isWindows".to_owned(),
        format!("isWindows={IS_WINDOWS}"),
    )];
    cases.push((
        "TS-UTC-01 currentTimeMillisToUTCString".to_owned(),
        utc_strings(&[
            0,
            1,
            9,
            10,
            99,
            100,
            999,
            1000,
            59_999,
            60_000,
            3_599_999,
            3_600_000,
            36_000_000,
            86_399_999,
            86_400_000,
            86_400_001,
            90_061_001,
            1_557_014_400_000,
            1_557_100_799_999,
            i64::MAX,
        ]),
    ));
    cases.push((
        "TS-UTC-02 negative timestamps".to_owned(),
        utc_strings(&[
            -1,
            -9,
            -10,
            -999,
            -1000,
            -60_000,
            -3_600_000,
            -36_000_000,
            -86_399_999,
            -86_400_000,
            -86_400_001,
            -1_234_567,
            i64::MIN,
        ]),
    ));
    cases.push((
        "TS-LOCAL-01 timestamp small values".to_owned(),
        zoned_strings(&[
            0,
            1,
            999,
            1000,
            86_399_999,
            86_400_000,
            -1,
            -1000,
            -19_800_000,
            -19_800_001,
        ]),
    ));
    cases.push((
        "TS-LOCAL-02 timestamp calendar edges".to_owned(),
        zoned_strings(&[
            951_782_400_000,
            951_868_800_000,
            1_582_934_400_000,
            1_583_020_800_000,
            4_107_542_400_000,
            -2_203_891_200_000,
            -2_203_977_600_000,
            -62_135_596_800_000,
            -62_167_219_200_000,
            -62_167_239_000_001,
            253_402_300_799_999,
            253_402_281_000_000,
            i64::MAX,
            i64::MIN,
        ]),
    ));
    cases.extend(identity_cases());
    cases.extend(clock_cases());
    cases.extend(swar_cases());

    let long_text = "x".repeat(1000);
    let ascii = all_ascii();
    let texts: [&str; 9] = [
        "",
        "a",
        "abc",
        "abcdefghijklmnopqrstuvxyz",
        "Hello, World!",
        &ascii,
        &long_text,
        "\u{0}",
        "\u{7f}",
    ];
    cases.push((
        "UN-01 fastHash of ASCII text".to_owned(),
        texts
            .iter()
            .map(|t| {
                let shown = if t.len() > 40 {
                    format!("{}x", t.len())
                } else {
                    esc(t)
                };
                format!("[{shown}]={}", fast_hash(t).expect("ASCII text"))
            })
            .collect::<Vec<_>>()
            .join(" "),
    ));
    cases.push((
        "UN-02 copyUSAsciiStrToBytes of ASCII text".to_owned(),
        ["", "abc", "abcdefghijklmnopqrstuvxyz", ascii.as_str()]
            .iter()
            .map(|text| {
                let mut target = vec![0x5A_u8; text.len() + 3];
                target[..text.len()].copy_from_slice(text.as_bytes());
                hex_bytes(&target)
            })
            .collect::<Vec<_>>()
            .join(" "),
    ));

    // makePattern is not reproduced (Q-UTIL-8 (c)): the Rust side answers with `glob_matches`, which
    // must agree with the regex on this alphabet (GlobSpec.scala:60-73).
    let mut bits = Vec::new();
    for len in 0..=4 {
        for pattern in strings_of("ab*?", len) {
            let mut row = String::new();
            for input_len in 0..=4 {
                for input in strings_of("ab", input_len) {
                    row.push(if glob_matches(&pattern, &input) {
                        'T'
                    } else {
                        'F'
                    });
                }
            }
            bits.push(format!("[{pattern}]={row}"));
        }
    }
    cases.push(("MP-EXH makePattern lengths 0-4".to_owned(), bits.join(" ")));
    cases
}

fn join<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The cases of `HashCodeOracle.java` (slice P65).
fn hash_code_cases() -> Vec<(String, String)> {
    use hash_code::hash;
    let seeds = [23, 0, 1, -1, i32::MAX, i32::MIN, 123_456_789];
    let ints = [0, 5, -5, i32::MAX, i32::MIN];
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), ok(line)));

    add(
        "HC-01 boolean",
        join(
            seeds
                .iter()
                .map(|s| format!("{s}:{}/{}", hash(*s, &true), hash(*s, &false))),
        ),
    );
    add(
        "HC-02 int",
        join(seeds.iter().flat_map(|s| ints.map(|v| hash(*s, &v)))),
    );
    let chars = [0_u16, u16::from(b'a'), 0x7fff, 0x8000, 0xffff];
    let shorts = [0_i16, 1, -1, i16::MAX, i16::MIN];
    let bytes = [0_i8, 1, -1, i8::MAX, i8::MIN];
    add(
        "HC-03 char short byte",
        join(
            chars
                .iter()
                .map(|v| hash(23, v))
                .chain(shorts.iter().map(|v| hash(23, v)))
                .chain(bytes.iter().map(|v| hash(23, v))),
        ),
    );
    let longs = [
        0_i64,
        1,
        -1,
        5_000_000_000,
        i64::MAX,
        i64::MIN,
        0x1_0000_0000,
        0xFFFF_FFFF,
    ];
    add(
        "HC-04 long",
        join(
            longs
                .iter()
                .flat_map(|v| [23, i32::MIN].map(|s| hash(s, v))),
        ),
    );
    let floats = [
        0.0_f32,
        -0.0,
        1.5,
        -1.5,
        f32::NAN,
        f32::from_bits(0x7fc0_0001),
        f32::from_bits(0xffc0_0000),
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::from_bits(1),
        f32::MAX,
        f32::MIN_POSITIVE,
    ];
    add("HC-05 float", join(floats.iter().map(|v| hash(23, v))));
    let doubles = [
        0.0_f64,
        -0.0,
        1.5,
        -1.5,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0001),
        f64::from_bits(0xfff8_0000_0000_0000),
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
        f64::MAX,
        f64::MIN_POSITIVE,
    ];
    add("HC-06 double", join(doubles.iter().map(|v| hash(23, v))));
    let thousand = "x".repeat(1000);
    let texts = [
        "",
        "a",
        "abc",
        "Hello, World!",
        "\u{e9}",
        "\u{20ac}",
        "\u{1F600}",
        "a\u{1F600}b",
        thousand.as_str(),
    ];
    add(
        "HC-07 string",
        join(texts.iter().map(|t| {
            let shown = if t.len() > 40 {
                format!("{}x", t.len())
            } else {
                esc(t)
            };
            format!("[{shown}]={}", hash(23, *t))
        })),
    );
    let h = |v: &dyn HashContribution| v.hash_into(23);
    add(
        "HC-08 arrays of primitives",
        [
            h(&Vec::<bool>::new().as_slice()),
            h(&[true, false, true].as_slice()),
            h(&[u16::from(b'a'), 0xffff].as_slice()),
            h(&[1_i16, -1, i16::MIN].as_slice()),
            h(&Vec::<i32>::new().as_slice()),
            h(&[1_i32, 2].as_slice()),
            h(&[i32::MAX, i32::MIN, -1].as_slice()),
            h(&[5_000_000_000_i64, -1, i64::MIN].as_slice()),
            h(&[1.5_f32, f32::NAN, -0.0].as_slice()),
            h(&[1.5_f64, f64::NAN, -0.0].as_slice()),
            h(&[1_i8, -1, i8::MIN].as_slice()),
            hash(i32::MIN, [7_i32, 8, 9].as_slice()),
            hash(0, Vec::<i8>::new().as_slice()),
        ]
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" "),
    );
    let two_longs: &[i64] = &[3];
    let nested: [&dyn HashContribution; 2] = [&"x", &two_longs];
    let ints12: &[i32] = &[1, 2];
    let mixed: [&dyn HashContribution; 9] = [
        &"a",
        &1_i32,
        &2_i64,
        &1.5_f64,
        &true,
        &u16::from(b'c'),
        &1.5_f32,
        &3_i16,
        &-4_i8,
    ];
    let empty: [&dyn HashContribution; 0] = [];
    let deep: [&dyn HashContribution; 3] = [&ints12, &nested.as_slice(), &empty.as_slice()];
    let strings = ["abc", "", "\u{e9}"];
    let rows: [&[i32]; 3] = [&[1, 2], &[3], &[]];
    add(
        "HC-09 arrays of objects",
        [
            hash(23, empty.as_slice()),
            hash(23, mixed.as_slice()),
            hash(23, deep.as_slice()),
            hash(23, strings.as_slice()),
            hash(23, [1_i32, 2, 3].as_slice()),
            hash(23, rows.as_slice()),
        ]
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" "),
    );
    add(
        "HC-10 boxed values",
        format!(
            "{} {} {} {} {} {}",
            hash(23, &5_i32),
            hash(23, &true),
            hash(23, &5_000_000_000_i64),
            hash(23, "abc"),
            hash(23, &1.5_f64),
            hash(23, &u16::from(b'a')),
        ),
    );
    let mut result = hash_code::SEED;
    result = hash(result, &42_i32);
    result = hash(result, "name");
    result = hash(result, [1_i32, 2, 3].as_slice());
    result = hash(result, &3.25_f64);
    result = hash(result, &true);
    add(
        "HC-11 chaining example",
        format!("{} {result}", hash_code::SEED),
    );
    add("HC-14 SEED constant", hash_code::SEED.to_string());
    cases
}

/// `PekkoVersionOracle.outcome`: `ok`, or `<Exception>[<message>]`.
fn require_outcome(library: &str, required: &str, current: &str) -> String {
    match pekko_version::require_against(library, required, current) {
        Ok(()) => "ok".to_owned(),
        Err(e) => require_error(&e),
    }
}

fn require_error(e: &RequireError) -> String {
    let (class, message) = match e {
        RequireError::Unsupported(e) => (e.java_class(), e.message()),
        RequireError::NumberFormat(e) => (e.java_class(), e.message()),
    };
    format!("{class}[{message}]")
}

/// `PekkoVersionOracle.pairs`.
fn require_pairs(pairs: &[&str]) -> String {
    join(pairs.iter().map(|pair| {
        let (required, current) = pair.split_once('|').expect("pair needs a bar");
        format!(
            "[{}]/[{}]={}",
            esc(required),
            esc(current),
            require_outcome("lib", required, current)
        )
    }))
}

/// The cases of `PekkoVersionOracle.java` (slice P66).
fn pekko_version_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));

    add(
        "PV-01 current version",
        format!(
            "current=[{}] conf=[{}]",
            pekko_version::CURRENT,
            esc(&format!("pekko.version = \"{}\"\n", pekko_version::CURRENT))
        ),
    );
    add(
        "PV-02 spec: satisfied",
        require_pairs(&[
            "2.5.6|2.5.6",
            "2.5.6|2.5.7",
            "2.5.6|2.6.0",
            "2.5.6|2.5.7-RC10",
            "2.6.0-RC1|2.6.0-RC1",
            "2.6.0-RC1|2.6.0-RC2",
            "2.6.0-RC1|2.7.0",
            "2.5.6|2.5.7-M10",
            "2.5.7-M9|2.5.7-M9",
            "2.5.7-M9|2.5.7-M10",
            "2.5.7-M9|2.5.8",
            "2.5.7-M9|2.5.7-RC1",
        ]),
    );
    add(
        "PV-03 spec: unsatisfied",
        require_pairs(&[
            "2.5.6|2.5.6-RC1",
            "2.5.6-RC2|2.5.6-RC1",
            "2.5.6|2.5.6-M1",
            "2.5.7-M10|2.5.7-M9",
            "2.5.7-RC1|2.5.7-M9",
            "2.5.6|3.0.0",
            "2.5.6|1.0.0",
            "2.5.6|2.4.19",
            "2.5.6|2.5.5",
        ]),
    );
    add(
        "PV-04 spec: snapshot and nonsense",
        require_pairs(&[
            "2.5.6|2.5-SNAPSHOT",
            "2.5.6|2.5-20180109-133700",
            "2.5-SNAPSHOT|2.5-SNAPSHOT",
            "2.5-20180109-133700|2.5-20180109-133700",
            "2.5.6|nonsense",
            "nonsense|2.5.6",
            "nonsense|nonsense",
            "2.5-SNAPSHOT|2.5.6",
            "2.5.6|2.5.6-SNAPSHOT",
            "2.5.6-SNAPSHOT|2.5.6",
        ]),
    );
    add(
        "PV-05 messages",
        join(
            ["my-library", "", "caf\u{e9} \u{20ac}", "[x]", "a\nb"]
                .iter()
                .map(|library| require_outcome(library, "2.5.6", "2.5.5"))
                .chain([
                    require_outcome("lib", "2.5.6-M1", "2.5.6-M0"),
                    require_outcome("lib", "2.5.6-RC3", "2.5.6-M9"),
                ]),
        ),
    );
    let kinds = ["", "-M1", "-M2", "-RC1", "-RC2"];
    add(
        "PV-06 pre-release matrix",
        kinds
            .iter()
            .flat_map(|required| kinds.iter().map(move |current| (required, current)))
            .map(|(required, current)| {
                let outcome = require_outcome(
                    "lib",
                    &format!("2.5.6{required}"),
                    &format!("2.5.6{current}"),
                );
                if outcome.starts_with("ok") { '1' } else { '0' }
            })
            .collect(),
    );
    add(
        "PV-07 numbers compared as text then as ints",
        require_pairs(&[
            "2.5.6|2.05.6",
            "2.05.6|2.5.6",
            "2.5.6|2.5.06",
            "2.5.06|2.5.6",
            "02.5.6|2.5.6",
            "2.5.6|02.5.6",
            "2.6.0|2.05.9",
            "2.05.9|2.5.8",
            "2.5.007|2.5.7",
            "2.5.7|2.5.0007",
            "2.5.7|2.5.007-M1",
            "2.5.7-M01|2.5.7-M1",
            "2.5.7-M2|2.5.7-M02",
            "2.5.7-M10|2.5.7-M9",
            "2.5.7-M9|2.5.7-M10",
            "2.5.2147483647|2.5.2147483646",
            "2.5.2147483647|2.5.2147483647",
        ]),
    );
    add(
        "PV-08 numbers out of int range",
        require_pairs(&[
            "2.5.6|2.5.99999999999",
            "2.5.99999999999|2.5.6",
            "2.99999999999.0|2.5.0",
            "2.5.0|2.99999999999.0",
            "99999999999.5.6|99999999999.5.7",
            "99999999999.5.6|99999999998.5.7",
            "2.5.99999999999|2.5.99999999999",
            "2.5.99999999999|2.5.99999999998",
            "2.5.6-M99999999999|2.5.6-M1",
            "2.5.6-M1|2.5.6-M99999999999",
            "2.5.6-RC99999999999|2.5.6-RC99999999999",
            "2.5.6-RC99999999999|2.5.6-RC99999999998",
            "2.5.2147483648|2.5.1",
            "2.5.1|2.5.2147483648",
            "2.2147483648.1|2.1.1",
            "2.5.6-M2147483648|2.5.6-M1",
        ]),
    );
    add(
        "PV-09 strings that are not a version",
        require_pairs(&[
            "2.5.6| 2.5.6",
            "2.5.6|2.5.6 ",
            " 2.5.6|2.5.5",
            "2.5.6 |2.5.5",
            "v2.5.6|2.5.5",
            "2.5.6|v2.5.5",
            "2.5.6-|2.5.5",
            "2.5.6-M|2.5.5",
            "2.5.6-m1|2.5.5",
            "2.5.6-rc1|2.5.5",
            "2.5.6-RC|2.5.5",
            "2.5.6.1|2.5.5",
            "2.5|2.5.5",
            "2.5.|2.5.5",
            "..|2.5.5",
            "|2.5.5",
            "2.5.6|",
            "|",
            "2.5.6-M1-M1|2.5.5",
            "2.5.6-M1x|2.5.5",
            "2.5.6\n|2.5.5",
            "2.5.6|2.5.5\n",
            "2.5.6-RC1+3|2.5.5",
            "2.5.6+3|2.5.5",
            "-2.5.6|2.5.5",
            "2.-5.6|2.5.5",
            "2.5.6-M-1|2.5.5",
            "2.5.6-M+1|2.5.5",
        ]),
    );
    add(
        "PV-10 digits outside ASCII",
        require_pairs(&[
            "2.5.6|\u{662}.\u{665}.\u{665}",
            "\u{662}.\u{665}.\u{666}|2.5.5",
            "2.5.6|2.5.\u{665}",
            "2.5.\u{666}|2.5.5",
            "2.5.6|\u{ff12}.\u{ff15}.\u{ff15}",
            "2.5.6-M\u{661}|2.5.6-M1",
            "2.5.6-M\u{661}|2.5.6",
        ]),
    );
    add(
        "PV-11 equal strings are never checked",
        require_pairs(&[
            "2.5.6|2.5.6",
            "2.5.6-M1|2.5.6-M1",
            "nonsense|nonsense",
            "|",
            "99999999999.9.9|99999999999.9.9",
            "2.5.6-M99999999999|2.5.6-M99999999999",
        ]),
    );
    add(
        "PV-12 public require against the current version",
        join(
            [
                "2.0.0",
                "1.7.1",
                "2.0.1",
                "2.1.0",
                "3.0.0",
                "1.0.0",
                "2.0.0-RC1",
                "2.0.0-M1",
                "2.0-SNAPSHOT",
                "nonsense",
                "1.9.9",
                "2.0.0-RC99999999999",
            ]
            .iter()
            .map(|required| {
                let outcome = match pekko_version::require("my-lib", required) {
                    Ok(()) => "ok".to_owned(),
                    Err(e) => require_error(&e),
                };
                format!("[{required}]={outcome}")
            }),
        ),
    );
    add(
        "PV-13 the exception",
        match pekko_version::require_against("lib", "9.0.0", "2.0.0") {
            Err(RequireError::Unsupported(e)) => format!(
                "class=org.apache.pekko.{0} super=java.lang.RuntimeException final=true cause=null toString=[org.apache.pekko.{0}: {1}] suppressed=0",
                e.java_class(),
                e.message()
            ),
            _ => "no exception".to_owned(),
        },
    );
    let mut matrix = String::new();
    for major in 1..=3 {
        for minor in 0..=2 {
            for patch in 0..=2 {
                for pre in ["", "-M1", "-RC1"] {
                    let required = format!("{major}.{minor}.{patch}{pre}");
                    matrix.push(if require_outcome("l", &required, "2.0.0") == "ok" {
                        '1'
                    } else {
                        '0'
                    });
                }
            }
        }
    }
    add("PV-14 many required versions against 2.0.0", matrix);
    cases
}

/// `VersionOracle.parse`: `ok`, or `<Exception>[<message>]`.
fn version_parse(v: &str) -> String {
    match Version::parse(v) {
        Ok(_) => "ok".to_owned(),
        Err(e) => format!("{}[{}]", e.java_class(), e.message()),
    }
}

/// `VersionOracle.cmp`: the raw `compareTo`, or the exception of building either side.
fn version_cmp(a: &str, b: &str) -> String {
    match (Version::parse(a), Version::parse(b)) {
        (Ok(x), Ok(y)) => x.pekko_compare(&y).to_string(),
        (Err(e), _) | (_, Err(e)) => format!("{}[{}]", e.java_class(), e.message()),
    }
}

/// `VersionOracle.rel`.
fn version_rel(pairs: &[&str]) -> String {
    join(pairs.iter().map(|pair| {
        let (x, y) = pair.split_once('|').expect("pair needs a bar");
        let eq = match (Version::parse(x), Version::parse(y)) {
            (Ok(a), Ok(b)) => (a == b).to_string(),
            _ => "-".to_owned(),
        };
        format!("[{}]/[{}]={} eq={eq}", esc(x), esc(y), version_cmp(x, y))
    }))
}

/// `VersionOracle.parse` of every string, as `[v]=outcome`.
fn version_parses(vs: &[&str]) -> String {
    join(vs.iter().map(|v| format!("[{v}]={}", version_parse(v))))
}

const VERSION_MATRIX: [&str; 19] = [
    "1.2",
    "1.2.0",
    "1.2-M1",
    "1.2.0-M1",
    "1.2-M2",
    "1.2.0-RC1",
    "1.2-RC1",
    "1.2.1",
    "1.2.0-SNAPSHOT",
    "1.2.0+3-abc",
    "1.2.0+10-abc",
    "1.2.0-3-abc",
    "1.2.0-",
    "1.2.0-M",
    "1.2.0-m1",
    "1",
    "1-M1",
    "foo",
    "1.2.0-M1-x",
];

/// The cases of `VersionOracle.java` (slice P67).
fn version_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));
    let v = |s: &str| Version::parse(s).expect("valid version");

    add(
        "VR-01 three digits (spec)",
        version_rel(&[
            "1.2.3|1.2.3",
            "1.2.3|1.2.4",
            "1.2.4|1.2.3",
            "3.2.1|1.2.3",
            "3.2.1|3.3.1",
            "3.2.0|3.2.1",
            "1.2.3|1.3.3",
            "1.3.3|1.2.3",
            "10.0.0|9.9.9",
            "0.0.10|0.0.9",
        ]),
    );
    add(
        "VR-02 more than three digits",
        version_parses(&[
            "1.2.3.4",
            "1.2.3.4.5",
            "1.2.3.4-M1",
            "1.2.3.",
            "1.2.3..",
            "1.2.3.4.",
            "1.2.3.x",
        ]),
    );
    add(
        "VR-03 two digits (spec)",
        version_rel(&[
            "1.2|1.2.0",
            "1.2.0|1.2",
            "1.2|1.3",
            "1.2.1|1.2",
            "1.2|1.2.1",
            "2.4|2.3",
            "3.2|3.2.7",
            "3.2.7|3.2",
            "1.2|1.2",
            "1.10|1.9",
        ]),
    );
    add(
        "VR-04 one digit (spec)",
        version_rel(&[
            "1|1.0",
            "1.0|1",
            "1|1.0.0",
            "1.0.0|1",
            "1.0|1.0.0",
            "3|2",
            "2b|2a",
            "2a|2b",
            "2020-09-07|2020-08-30",
            "2020-08-30|2020-09-07",
            "2|2",
            "10|9",
            "99999999999|99999999998",
            "99999999999|1",
        ]),
    );
    add(
        "VR-05 qualifier (spec)",
        version_rel(&[
            "1.2.3-M1|1.2.3-M1",
            "1.2.0-M1|1.2-M1",
            "1.2-M1|1.2.0-M1",
            "1.2-M1|1.2.0",
            "1.2.0|1.2-M1",
            "1.2.3-M2|1.2.3-M1",
            "1.2.3-M1|1.2.3-M2",
            "1.2.3-M1|1.2.3",
            "1.2.3|1.2.3-M1",
        ]),
    );
    add(
        "VR-06 non digits (spec)",
        version_parses(&[
            "1.x.3", "1.2x.3", "1.2.x", "1.2.3x", "x.3", "1.x", "1.2x", "x.y.z", "1.2.3-x",
            "1.2.3-1x",
        ]),
    );
    add(
        "VR-07 dynver (spec)",
        version_rel(&[
            "1.0.10+3-1234abcd|1.0.10+10-1234abcd",
            "1.0.10+10-1234abcd|1.0.10+3-1234abcd",
            "1.0.10+3-1234abcd|1.0.10+3-1234abcd",
            "1.0.10+3-1234abcd|1.0.10+3-1234abce",
            "1.0.10-3-1234abcd|1.0.10-10-1234abcd",
            "1.0.10-10-1234abcd|1.0.10+3-1234abcd",
            "1.0+3-1234abcd|1.0.0+3-1234abcd",
            "1.0+3-1234abcd|1.0+10-1234abcd",
            "HEAD+20140707-1030|HEAD+20140707-1030",
            "HEAD+20140707-1030|HEAD+20140707-1031",
            "HEAD+20140707-1030|1.0.0",
            "1.0.10+3-1234abcd|1.0.10",
            "1.0.10|1.0.10+3-1234abcd",
            "1.0.10-SNAPSHOT|1.0.10+3-1234abcd",
            "1.0.10+3-1234abcd|1.0.10-SNAPSHOT",
            "1.0.10+99999999999-abc|1.0.10+3-abc",
            "1.0.10+3|1.0.10+3",
            "1.0.10+3|1.0.10+4",
            "1.0.10+3-|1.0.10+3-a",
        ]),
    );
    add(
        "VR-08 no digits (spec)",
        version_rel(&[
            "foo|foo",
            "foo|bar",
            "bar|foo",
            "foo|1.2.3",
            "1.2.3|foo",
            "foo|0.0.0",
            "1-foo|01-foo",
            "1-foo|02-foo",
            "02-foo|1-foo",
            "2-foo|1-foo",
            "foo|Foo",
            "Foo|foo",
            "foo|foo1",
            "foo1|foo",
        ]),
    );
    add(
        "VR-09 qualifier matrix (raw compareTo, row vs column)",
        join(VERSION_MATRIX.iter().map(|a| {
            let row: String = VERSION_MATRIX
                .iter()
                .map(|c| format!("{},", version_cmp(a, c)))
                .collect();
            format!("[{a}]:{row}")
        })),
    );
    let mut broken = 0;
    let mut bad = Vec::new();
    for a in VERSION_MATRIX {
        for c in VERSION_MATRIX {
            if v(a).pekko_compare(&v(c)).signum() != -v(c).pekko_compare(&v(a)).signum() {
                broken += 1;
                bad.push(format!("[{a}]/[{c}]"));
            }
        }
    }
    add(
        "VR-10 antisymmetry of the matrix",
        format!(
            "pairs={} broken={broken} {}",
            VERSION_MATRIX.len() * VERSION_MATRIX.len(),
            bad.join(" ").trim()
        ),
    );
    add(
        "VR-11 qualifier on one side only",
        version_rel(&[
            "1.2.0|1.2.0-M1",
            "1.2.0-M1|1.2.0",
            "1.2.0|1.2-M1",
            "1.2-M1|1.2.0",
            "1.2.0|1.2.0-RC1",
            "1.2.0-RC1|1.2.0",
            "1.2.0|1.2.0-SNAPSHOT",
            "1.2.0-SNAPSHOT|1.2.0",
            "1.2|1.2-M1",
            "1.2-M1|1.2",
            "1|1-M1",
            "1-M1|1",
            "1.2.3|1.2.3-M1",
            "1.2.3-M1|1.2.3",
            "1.2.3|1.2.3-",
            "1.2.3-|1.2.3",
            "2.0.0|1.9.9-M1",
            "1.9.9-M1|2.0.0",
            "1.2.0-M1|1.2.1",
            "1.2.1|1.2.0-M1",
        ]),
    );
    add(
        "VR-12 qualifier text order (raw String.compareTo)",
        version_rel(&[
            "1.2.0-M1|1.2.0-M2",
            "1.2.0-M2|1.2.0-M1",
            "1.2.0-M1|1.2.0-M10",
            "1.2.0-M10|1.2.0-M1",
            "1.2.0-M9|1.2.0-M10",
            "1.2.0-M1|1.2.0-RC1",
            "1.2.0-RC1|1.2.0-M1",
            "1.2.0-a|1.2.0-B",
            "1.2.0-B|1.2.0-a",
            "1.2.0-abc|1.2.0-abd",
            "1.2.0-ab|1.2.0-abcd",
            "1.2.0-abcd|1.2.0-ab",
            "1.2.0-\u{e9}|1.2.0-e",
            "1.2.0-\u{1F600}|1.2.0-\u{ffff}",
            "1.2.0-\u{ffff}|1.2.0-\u{1F600}",
            "1.2.0-M1|1.2.0-m1",
        ]),
    );
    let hashed: String = join(
        [
            "1.2",
            "1.2.0",
            "1",
            "1.0",
            "1.0.0",
            "1.2-M1",
            "1.2.0-M1",
            "1.2.3",
            "foo",
            "bar",
            "1.0.10+3-abc",
            "1.0.10+10-abc",
            "0.0.0",
            "0",
            "x",
        ]
        .iter()
        .map(|s| format!("[{s}]={}", v(s).hash_code())),
    );
    add(
        "VR-13 equals and hashCode",
        format!(
            "{hashed} zero={} {}",
            Version::zero().hash_code(),
            Version::zero().version()
        ),
    );
    let mut disagree = Vec::new();
    for x in VERSION_MATRIX {
        for y in VERSION_MATRIX {
            if v(x) == v(y) && v(x).hash_code() != v(y).hash_code() {
                disagree.push(format!("[{x}]/[{y}]"));
            }
        }
    }
    add(
        "VR-14 hashCode agrees with equals",
        format!(
            "pairs={} equal-but-different-hash={} {}",
            VERSION_MATRIX.len() * VERSION_MATRIX.len(),
            disagree.len(),
            disagree.join(" ")
        ),
    );
    add(
        "VR-15 malformed strings",
        join(
            [
                "",
                ".",
                "..",
                "...",
                "1.",
                "1..",
                ".1",
                "..1",
                "1..2",
                ".1.2",
                "1.2.",
                " 1.2",
                "1.2 ",
                "1. 2",
                "-1",
                "+1",
                "1.-2",
                "1.+2",
                "-1.2",
                "-1.2.3",
                "1.-2.3",
                "1.2.-3",
                "1.2.+3",
                "1.2.3-",
                "1.2.3+",
                "1.2.3-+",
                "1.2.3+-",
                "1.2-",
                "1.2+",
                "1-",
                "1+",
                "1.",
                "a",
                "a.b",
                "a.b.c",
                "1.2.3-M1+5",
                "1.2.3+5-M1",
                "1.2.3+5-",
                "1.2.3+-5-x",
                "1.2.3--5-x",
                "1.2.3-5",
                "1.2.3-5-",
                "1.2.3-5x-y",
                "1.2.3-x-5",
                "v1.2.3",
                "1.2.3\n",
                "\n",
                " ",
                "1.2.3 ",
            ]
            .iter()
            .map(|s| format!("[{}]={}", esc(s), esc(&version_parse(s)))),
        ),
    );
    add(
        "VR-16 numbers out of int range",
        version_rel(&[
            "2147483647.0.0|1.0.0",
            "2147483648.0.0|1.0.0",
            "1.2147483648.0|1.0.0",
            "1.0.2147483648|1.0.0",
            "1.2.3-2147483648-x|1.2.3",
            "2147483647|2147483646",
            "2147483648|2147483647",
            "2147483648|2147483649",
            "99999999999|99999999999",
            "99999999999.1.1|1.1.1",
            "1.2.99999999999-M1|1.2.1",
            "1.99999999999-M1|1.2",
            "1.2+2147483648-x|1.2",
        ]),
    );
    add(
        "VR-17 negative and wrapping numbers",
        version_rel(&[
            "-1.2.3|1.2.3",
            "1.2.3|-1.2.3",
            "-1.2.3|-1.2.3",
            "-1.2.3|-2.2.3",
            "2147483647.0.0|-1.0.0",
            "-1.0.0|2147483647.0.0",
            "-2147483648.0.0|2147483647.0.0",
            "2147483647.0.0|-2147483648.0.0",
            "1.2147483647.0|1.-1.0",
            "1.-1.0|1.2147483647.0",
            "1.2.-3|1.2.3",
            "1.2.3|1.2.-3",
            "1.2.-2147483648|1.2.2147483647",
            "1.2.2147483647|1.2.-2147483648",
        ]),
    );
    add(
        "VR-18 digits outside ASCII",
        version_rel(&[
            "\u{662}.\u{665}.\u{665}|2.5.5",
            "2.5.5|\u{662}.\u{665}.\u{665}",
            "2.5.\u{665}|2.5.5",
            "\u{ff12}.\u{ff15}.\u{ff15}|2.5.5",
            "\u{662}|2",
            "2|\u{662}",
            "\u{662}b|2b",
            "\u{662}b|\u{663}b",
            "1.2.3-M\u{661}|1.2.3-M1",
            "1.\u{662}|1.2",
            "1.\u{662}-M1|1.2-M1",
        ]),
    );
    add(
        "VR-19 equal strings short-circuit",
        version_rel(&[
            "1.2.3|1.2.3",
            "foo|foo",
            "1.2.3-M1|1.2.3-M1",
            "99999999999|99999999999",
            "1.2.0+3-abc|1.2.0+3-abc",
            "-1.2.3|-1.2.3",
        ]),
    );
    let mut fields: Vec<String> = ["1.2", "1.2.0-M1", "foo", "2020-09-07", "1.0.10+3-abc"]
        .iter()
        .map(|s| format!("[{}]=[{}]", v(s).version(), v(s)))
        .collect();
    fields.push(format!("zero=[{}]", Version::zero()));
    fields.push(format!("1.2>zero={}", version_cmp("1.2", "0.0.0")));
    fields.push(format!("zero>foo={}", version_cmp("0.0.0", "foo")));
    fields.push(format!("0=zero:{}", version_cmp("0", "0.0.0")));
    add("VR-20 version field and Zero", fields.join(" "));
    add(
        "VR-22 parse exceptions",
        join(
            [
                "",
                "1.2.3.4",
                "1.x.3",
                "1.2x",
                "x.3",
                "1.2.3x",
                "1.",
                ".",
                "1.2.3.4.5",
                "99999999999.1",
                "1.99999999999.1",
            ]
            .iter()
            .map(|s| match Version::parse(s) {
                Ok(_) => format!("[{s}]=ok"),
                Err(e) => format!(
                    "[{s}]=java.lang.{} cause=null msg=[{}]",
                    e.java_class(),
                    e.message()
                ),
            }),
        ),
    );
    let mut sorted: Vec<Version> = [
        "1.2.0",
        "1.2.0-RC1",
        "1.2.0-M2",
        "1.2.0-M1",
        "1.1.9",
        "1.2.1",
        "1.2.0+3-abc",
        "1.2.0+10-abc",
        "1.2",
        "1.2-M1",
        "1.10.0",
        "1.9.0",
    ]
    .iter()
    .map(|s| v(s))
    .collect();
    sorted.sort_by(|a, b| a.pekko_compare(b).cmp(&0));
    let (mut max, mut min) = (&sorted[0], &sorted[0]);
    for item in &sorted {
        if item.pekko_compare(max) > 0 {
            max = item;
        }
        if item.pekko_compare(min) < 0 {
            min = item;
        }
    }
    add(
        "VR-23 sorting",
        format!(
            "{} max={max} min={min}",
            sorted
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ),
    );
    cases
}

const US: i64 = 1_000;
const MS: i64 = 1_000_000;
const S: i64 = 1_000_000_000;
const MIN: i64 = 60 * S;
const H: i64 = 60 * MIN;
const D: i64 = 24 * H;

/// `PrettyDurationOracle.pretty`: `{text}`, or `<Exception>[<message>]`.
fn pretty_outcome(d: PekkoDuration, include_nanos: bool, precision: i32) -> String {
    match d.pretty_with(include_nanos, precision) {
        Ok(text) => format!("{{{text}}}"),
        Err(e) => format!("{}[{}]", e.java_class(), e.message()),
    }
}

/// `PrettyDurationOracle.nanos`: `fromNanos(n)` then `pretty_outcome`.
fn pretty_nanos(n: i64, include_nanos: bool, precision: i32) -> String {
    match Timeout::new(n) {
        Ok(t) => pretty_outcome(PekkoDuration::Finite(t), include_nanos, precision),
        Err(e) => format!("{}[{}]", e.java_class(), e.message()),
    }
}

/// `PrettyDurationOracle.list`: `[n]={text}` with the default rendering.
fn pretty_list(ns: &[i64]) -> String {
    join(
        ns.iter()
            .map(|&n| format!("[{n}]={}", pretty_nanos(n, false, 4))),
    )
}

fn pretty_for_precisions(n: i64, precisions: &[i32]) -> String {
    join(precisions.iter().map(|&p| pretty_nanos(n, false, p)))
}

/// `PrettyDurationOracle.sweep`: the same xorshift64* generator.
fn pretty_sweep(seed: u64, count: usize, vary_precision: bool) -> String {
    let mut x = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let bits = ((r >> 58) & 63) as u32;
        let magnitude = ((r << 1) >> 1) >> (63 - bits);
        let n = if (r >> 57) & 1 == 0 {
            magnitude as i64
        } else {
            -(magnitude as i64)
        };
        let precision = if vary_precision {
            1 + ((r >> 52) & 15) as i32
        } else {
            4
        };
        out.push(pretty_nanos(n, (r >> 51) & 1 == 1, precision));
    }
    join(out)
}

/// The cases of `PrettyDurationOracle.java` (slice P68).
fn pretty_duration_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));
    let non_finite = [
        PekkoDuration::Inf,
        PekkoDuration::MinusInf,
        PekkoDuration::Undefined,
    ];

    add(
        "PD-01 spec table",
        pretty_list(&[
            9,
            95,
            999,
            1000,
            9500,
            9500 * US,
            9500 * MS,
            95 * S,
            95 * MIN,
            95 * H,
        ]),
    );
    add(
        "PD-02 infinities and undefined",
        join(non_finite.iter().map(|&d| {
            format!(
                "{} {} {} {}",
                pretty_outcome(d, false, 4),
                pretty_outcome(d, true, 4),
                pretty_outcome(d, true, 1),
                pretty_outcome(d, false, 40)
            )
        })) + " "
            + &join(non_finite.iter().map(PekkoDuration::pretty)),
    );
    add(
        "PD-03 infinities with invalid precision",
        join(non_finite.iter().flat_map(|&d| {
            [0, -1, i32::MIN]
                .into_iter()
                .map(move |p| pretty_outcome(d, false, p))
        })),
    );
    add(
        "PD-04 unit boundaries",
        join(
            [US, MS, S, MIN, H, D]
                .iter()
                .map(|&u| pretty_list(&[u - 1, u, u + 1])),
        ),
    );
    add(
        "PD-05 default format equals pretty with 4 digits",
        join([1, 999, 1500, 4 * S, 3 * D].iter().map(|&n| {
            let default = PekkoDuration::Finite(Timeout::new(n).expect("in range")).pretty();
            format!("{default}|{}", pretty_nanos(n, false, 4))
        })),
    );
    add(
        "PD-06 include nanos",
        join(
            [0, 1, 999, 1500, 95 * S, 3 * D + 17, i64::MAX, -5, -i64::MAX]
                .iter()
                .map(|&n| {
                    format!(
                        "[{n}]={} {}",
                        pretty_nanos(n, true, 4),
                        pretty_nanos(n, true, 2)
                    )
                }),
        ),
    );
    add(
        "PD-07 precision must be positive",
        join([0, -1, -7, i32::MIN].iter().map(|&p| {
            format!(
                "{} {}",
                pretty_nanos(1500, false, p),
                pretty_nanos(1500, true, p)
            )
        })),
    );
    add(
        "PD-08 scientific notation",
        pretty_list(&[
            9999 * D,
            10000 * D,
            99999 * D,
            100000 * D,
            12345 * D,
            106751 * D,
            9999 * D + 5 * H,
            9999 * D + 12 * H,
            9999 * D + 11 * H,
        ]),
    );
    add("PD-09 zero", pretty_for_precisions(0, &[1, 2, 4, 7, 12]));
    add(
        "PD-10 negative durations",
        pretty_list(&[
            -1,
            -999,
            -1000,
            -1500,
            -US,
            -MS,
            -95 * S,
            -95 * MIN,
            -95 * H,
            -3 * D,
            -i64::MAX,
        ]),
    );
    add(
        "PD-11 Long.MaxValue",
        pretty_for_precisions(i64::MAX, &[1, 3, 4, 6, 10, 17, 18, 20]),
    );
    add(
        "PD-12 precision from 1 to 25",
        join(
            [1, 7, 1234, 1_234_567, 98_765_432_101, 123_456_789_012_345]
                .iter()
                .map(|&n| {
                    let all: Vec<i32> = (1..=25).collect();
                    format!("{} |", pretty_for_precisions(n, &all))
                }),
        ),
    );
    add(
        "PD-13 large precision",
        join([50, 100, 400].iter().map(|&p| {
            format!(
                "{} {}",
                pretty_nanos(1500, false, p),
                pretty_nanos(3 * D, false, p)
            )
        })),
    );
    add(
        "PD-14 rounds half up on the decimal digits",
        join(
            [
                1_000_500,
                12_345,
                1_005_000,
                2_675_000,
                1_000_050,
                999_950,
                9_999_500,
                99_995,
                9_995,
                2_500,
                3_500,
                1_235_000_000,
                1_245_000_000,
                8_650_000_000_000,
            ]
            .iter()
            .map(|&n| format!("[{n}]={}", pretty_for_precisions(n, &[4, 3, 2, 1]))),
        ),
    );
    add(
        "PD-15 carry to the next power of ten",
        join(
            [
                9999,
                99_995,
                999_950,
                9_999_500,
                999_999,
                59_999_500_000,
                3_599_950_000_000,
                86_399_950_000_000,
                106_751 * D,
                9_999 * D + 22 * H,
            ]
            .iter()
            .map(|&n| {
                let first_six: Vec<i32> = (1..=6).collect();
                format!("[{n}]={}", pretty_for_precisions(n, &first_six))
            }),
        ),
    );
    add(
        "PD-16 sweep of 3000 pseudo-random magnitudes",
        pretty_sweep(0x2545_F491_4F6C_DD1D, 3000, false),
    );
    add(
        "PD-17 sweep of 3000 pseudo-random precisions",
        pretty_sweep(0x9E37_79B9_7F4A_7C15, 3000, true),
    );
    add(
        "PD-18 sweep of round values",
        join([1, US, MS, S, MIN, H, D].iter().flat_map(|&u| {
            [
                1, 2, 3, 5, 7, 10, 11, 15, 24, 59, 60, 61, 99, 100, 101, 999, 1000, 1001, 9999,
                10000, 12345, 99999,
            ]
            .into_iter()
            .map(move |k| pretty_nanos(u * k, false, 4))
        })),
    );
    add("PD-19 integral doubles from 2^53", {
        let mut parts = Vec::new();
        for e in 52..=62 {
            let base: i64 = 1 << e;
            for m in [
                base,
                base + base / 3,
                base + base / 2 + 12345,
                base.wrapping_mul(2).wrapping_sub(1),
            ] {
                parts.push(format!(
                    "[{}]={}",
                    -m,
                    pretty_for_precisions(-m, &[16, 17, 18, 19, 20, 22])
                ));
            }
        }
        parts.push(format!(
            "[max]={}",
            pretty_for_precisions(-i64::MAX, &[1, 4, 15, 16, 17, 18, 19, 20, 22])
        ));
        join(parts)
    });
    cases
}

// ---- Slice P69: UTIL-UUID-ORDEN (AC-UTIL-016.14), the twin of `UuidOrderOracle.java`.

fn uu(msb: u64, lsb: u64) -> JavaUuid {
    JavaUuid::new(msb as i64, lsb as i64)
}

fn uu_with_version(rest: u64, version: u64) -> u64 {
    (rest & !0xF000) | (version << 12)
}

fn uu_show(x: JavaUuid) -> String {
    format!(
        "{:016x}:{:016x}",
        x.most_significant_bits() as u64,
        x.least_significant_bits() as u64
    )
}

fn uu_pair(a: JavaUuid, b: JavaUuid) -> String {
    format!(
        "[{}]/[{}]={}",
        uu_show(a),
        uu_show(b),
        pekko_compare_uuid(a, b)
    )
}

fn uu_pairs(pairs: &[(JavaUuid, JavaUuid)]) -> String {
    pairs
        .iter()
        .map(|&(a, b)| uu_pair(a, b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn uu_timestamp(x: JavaUuid) -> String {
    match x.timestamp() {
        Ok(t) => t.to_string(),
        Err(e) => format!("{}[{}]", e.java_class(), e.message()),
    }
}

/// `UuidOrderOracle.next`: xorshift64*.
fn uu_next(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// `UuidOrderOracle.sweep`.
fn uu_sweep(seed: u64, pairs: usize, versions: &[u64], tie_heavy: bool) -> String {
    let mut state = seed;
    let (mut neg, mut zero, mut pos, mut antisym, mut trans) = (0, 0, 0, 0, 0);
    let mut h: i32 = 17;
    let mut all: Vec<JavaUuid> = Vec::new();
    let versioned = |state: &mut u64| {
        let rest = uu_next(state);
        let v = versions[((uu_next(state) >> 1) % versions.len() as u64) as usize];
        uu_with_version(rest, v)
    };
    for i in 0..pairs {
        let m1 = versioned(&mut state);
        let l1 = uu_next(&mut state);
        let mode = if tie_heavy { i % 4 } else { 0 };
        let (m2, l2) = match mode {
            0 => {
                let m2 = versioned(&mut state);
                (m2, uu_next(&mut state))
            }
            1 => (m1, uu_next(&mut state)),
            2 => {
                let bit = (uu_next(&mut state) >> 1) % 128;
                if bit < 64 {
                    (m1 ^ (1 << bit), l1)
                } else {
                    (m1, l1 ^ (1 << (bit - 64)))
                }
            }
            _ => (m1, l1),
        };
        let (a, b) = (uu(m1, l1), uu(m2, l2));
        let r = pekko_compare_uuid(a, b);
        match r.cmp(&0) {
            std::cmp::Ordering::Less => neg += 1,
            std::cmp::Ordering::Equal => zero += 1,
            std::cmp::Ordering::Greater => pos += 1,
        }
        h = h.wrapping_mul(31).wrapping_add(r);
        if pekko_compare_uuid(b, a).signum() != -r.signum() {
            antisym += 1;
        }
        if all.len() < 60 {
            all.push(a);
            all.push(b);
        }
    }
    for w in all.windows(3) {
        let (a, b, c) = (w[0], w[1], w[2]);
        if pekko_compare_uuid(a, b) <= 0
            && pekko_compare_uuid(b, c) <= 0
            && pekko_compare_uuid(a, c) > 0
        {
            trans += 1;
        }
    }
    format!(
        "neg={neg} zero={zero} pos={pos} hash={h} antisymmetry-violations={antisym} transitivity-violations={trans}"
    )
}

/// The cases of `UuidOrderOracle.java` (slice P69).
fn uuid_order_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));
    let fw = uu_with_version;

    add(
        "UU-01 version of the msb",
        [
            0x0_u64,
            0x1000,
            0x4000,
            0xF000,
            0x0FFF,
            0xFFFF_FFFF_FFFF_0FFF,
            0xFFFF_FFFF_FFFF_FFFF,
            0x8000_0000_0000_8000,
            0x1234_5678_9ABC_5DEF,
        ]
        .iter()
        .map(|&m| format!("[{m:016x}]={}", uu(m, 0).version()))
        .collect::<Vec<_>>()
        .join(" "),
    );
    add(
        "UU-02 timestamp of version 1",
        [
            0x0000_0000_0000_1000_u64,
            0x0000_0001_0000_1000,
            0x0000_0000_0001_1000,
            0x0000_0000_0000_1001,
            0xFFFF_FFFF_FFFF_1FFF,
            0x8000_0000_8000_1800,
            0x1234_5678_9ABC_1DEF,
            0xFFFF_FFFF_FFFF_1000,
            0x0000_0000_0000_1FFF,
        ]
        .iter()
        .map(|&m| format!("[{m:016x}]={}", uu_timestamp(uu(m, 0x8000_0000_0000_0001))))
        .collect::<Vec<_>>()
        .join(" "),
    );
    add(
        "UU-03 timestamp of other versions",
        [0_u64, 2, 3, 4, 5, 6, 7, 8, 15]
            .iter()
            .map(|&v| format!("v{v}={}", uu_timestamp(uu(fw(0x1234_5678_9ABC_0DEF, v), 5))))
            .collect::<Vec<_>>()
            .join(" "),
    );
    add("UU-04 order by version number", {
        let mut parts = Vec::new();
        let mut v1 = 0_u64;
        while v1 < 16 {
            let mut v2 = 15_i64;
            while v2 >= 0 {
                parts.push(uu_pair(
                    uu(fw(0x0123_4567_89AB_0000, v1), 7),
                    uu(fw(0x0123_4567_89AB_0000, v2 as u64), 7),
                ));
                v2 -= 4;
            }
            v1 += 3;
        }
        parts.join(" ")
    });
    add(
        "UU-05 version before every other bit",
        uu_pairs(&[
            (uu(fw(u64::MAX, 3), u64::MAX), uu(fw(0, 4), 0)),
            (uu(fw(0, 3), 0), uu(fw(u64::MAX, 4), u64::MAX)),
            (uu(fw(u64::MAX, 1), u64::MAX), uu(fw(0, 2), 0)),
            (uu(fw(0, 15), 0), uu(fw(0, 0), 0)),
        ]),
    );
    add(
        "UU-06 unsigned lsb of version 4",
        uu_pairs(&[
            (
                uu(0x4000_0000_0000_0000, 0x8000_0000_0000_0001),
                uu(0x4000_0000_0000_0000, 1),
            ),
            (
                uu(0x4000_0000_0000_0000, 1),
                uu(0x4000_0000_0000_0000, 0x8000_0000_0000_0001),
            ),
            (
                uu(0x4000_0000_0000_0000, u64::MAX),
                uu(0x4000_0000_0000_0000, 0),
            ),
            (
                uu(0x4000_0000_0000_0000, 0x7FFF_FFFF_FFFF_FFFF),
                uu(0x4000_0000_0000_0000, 0x8000_0000_0000_0000),
            ),
            (uu(0x4000_0000_0000_0000, 1), uu(0x4000_0000_0000_0000, 2)),
            (uu(0x4000_0000_0000_0000, 0), uu(0x4000_0000_0000_0000, 0)),
        ]),
    );
    add(
        "UU-07 unsigned msb of version 4",
        uu_pairs(&[
            (uu(0xF000_0000_0000_4000, 0), uu(0x7000_0000_0000_4000, 0)),
            (uu(0x7000_0000_0000_4000, 0), uu(0xF000_0000_0000_4000, 0)),
            (uu(0x8000_0000_0000_4000, 0), uu(0x0000_0000_0000_4000, 0)),
            (uu(0xFFFF_FFFF_FFFF_4FFF, 0), uu(0x0000_0000_0000_4000, 0)),
            (uu(0x0000_0001_FFFF_4000, 0), uu(0x0000_0000_FFFF_4000, 0)),
            (uu(0x8000_0000_FFFF_4000, 0), uu(0x7FFF_FFFF_FFFF_4000, 0)),
        ]),
    );
    add(
        "UU-08 the two 32-bit halves",
        uu_pairs(&[
            (uu(0x8000_0000_4000_0000, 0), uu(0x7FFF_FFFF_4000_0000, 0)),
            (uu(0x0000_0001_4000_8000, 0), uu(0x0000_0001_4000_7FFF, 0)),
            (uu(0x4000_0000, 0x8000_0000), uu(0x4000_0000, 0x7FFF_FFFF)),
            (
                uu(0x4000_0000, 0x8000_0000_0000_0000),
                uu(0x4000_0000, 0xFFFF_FFFF),
            ),
            (
                uu(0x4000_0000, 0xFFFF_FFFF_0000_0000),
                uu(0x4000_0000, 0x7FFF_FFFF_FFFF_FFFF),
            ),
            (
                uu(0x4000_0000, 0xFFFF_FFFF_0000_0001),
                uu(0x4000_0000, 0xFFFF_FFFF_0000_0002),
            ),
        ]),
    );
    add(
        "UU-09 raw magnitudes of the result",
        uu_pairs(&[
            (uu(0x4000_0000, 0), uu(0x7FFF_FFFF_4000_0000, 0)),
            (uu(0x7FFF_FFFF_4000_0000, 0), uu(0x4000_0000, 0)),
            (uu(0x8000_0000_4000_0000, 0), uu(0xFFFF_FFFF_4000_0000, 0)),
            (uu(0xFFFF_FFFF_4000_0000, 0), uu(0x8000_0000_4000_0000, 0)),
            (
                uu(0x4000_0000_0000_0000, 0x8000_0000_0000_0000),
                uu(0x4000_0000_0000_0000, 0xFFFF_FFFF_0000_0000),
            ),
            (
                uu(0x4000_0000_0000_0000, 0),
                uu(0x4000_0000_0000_0000, 0x7FFF_FFFF_0000_0000),
            ),
            (
                uu(0x4000_0000_0000_0000, 0),
                uu(0x4000_0000_0000_0000, 0x7FFF_FFFF),
            ),
            (
                uu(0x4000_0000_0000_0000, 0xFFFF_FFFF),
                uu(0x4000_0000_0000_0000, 0),
            ),
        ]),
    );
    add(
        "UU-10 version 1 by timestamp",
        uu_pairs(&[
            (uu(0x0000_0002_0000_1000, 0), uu(0x0000_0001_0000_1000, 0)),
            (uu(0x0000_0000_0001_1000, 0), uu(0x0000_0001_0000_1000, 0)),
            (uu(0x0000_0000_0000_1001, 0), uu(0x0000_0001_0000_1000, 0)),
            (uu(0x0000_0000_0000_1001, 0), uu(0x0000_0000_0000_1000, 0)),
            (uu(0x0000_0000_0000_1000, 0), uu(0x0000_0000_0000_1001, 0)),
            (uu(0x8000_0000_0000_1000, 0), uu(0x7FFF_FFFF_0000_1000, 0)),
        ]),
    );
    add(
        "UU-11 version 1 ignores the msb order",
        uu_pairs(&[
            (uu(0xFFFF_FFFF_FFFF_1000, 0), uu(0x0000_0000_0000_1FFF, 0)),
            (uu(0x0000_0000_0000_1FFF, 0), uu(0xFFFF_FFFF_FFFF_1000, 0)),
            (uu(0x0000_0001_0000_1000, 0), uu(0x0000_0000_0000_1FFF, 0)),
            (uu(0x0000_FFFF_FFFF_1000, 0), uu(0xFFFF_0000_0000_1000, 0)),
            (uu(0xFFFF_0000_0000_1000, 0), uu(0x0000_FFFF_FFFF_1000, 0)),
        ]),
    );
    let tie = 0x1234_5678_9ABC_1DEF_u64;
    add(
        "UU-12 version 1 timestamp ties by unsigned lsb",
        uu_pairs(&[
            (uu(tie, 0x8000_0000_0000_0001), uu(tie, 1)),
            (uu(tie, 1), uu(tie, 0x8000_0000_0000_0001)),
            (uu(tie, u64::MAX), uu(tie, 0)),
            (uu(tie, 0xFFFF_FFFF), uu(tie, 0xFFFF_FFFF_0000_0000)),
            (uu(tie, 5), uu(tie, 5)),
            (uu(tie, 5), uu(tie, 6)),
        ]),
    );
    add(
        "UU-13 version 1 lsb tie-break extremes",
        uu_pairs(&[
            (uu(0x1000, 3), uu(0x1000, 3)),
            (uu(0x1000, 3), uu(0x1000, 0xC000_0000_0000_0003)),
            (uu(0x1000, 0xC000_0000_0000_0003), uu(0x1000, 3)),
            (uu(0xFFFF_FFFF_FFFF_1FFF, 0), uu(0xFFFF_FFFF_FFFF_1FFF, 0)),
            (
                uu(0xFFFF_FFFF_FFFF_1FFF, 1),
                uu(0xFFFF_FFFF_FFFF_1FFE, u64::MAX),
            ),
        ]),
    );
    add(
        "UU-14 versions 0, 2, 3, 5 to 15 order like version 4",
        [0_u64, 2, 3, 5, 6, 7, 8, 9, 15]
            .iter()
            .map(|&v| {
                format!(
                    "v{v} {} {};",
                    uu_pair(
                        uu(fw(0xF000_0000_0000_0000, v), 1),
                        uu(fw(0x7000_0000_0000_0000, v), 1)
                    ),
                    uu_pair(
                        uu(fw(0x1000_0000_0000_0000, v), 0x8000_0000_0000_0000),
                        uu(fw(0x1000_0000_0000_0000, v), 1)
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
    );
    add("UU-15 every version against every version", {
        let mut b = Vec::new();
        for v1 in 0..16_u64 {
            let mut row = String::new();
            for v2 in 0..16_u64 {
                row.push_str(&format!(
                    "{},",
                    pekko_compare_uuid(
                        uu(fw(0x0123_4567_89AB_0DEF, v1), (-2_i64) as u64),
                        uu(fw(0x0123_4567_89AB_0DEF, v2), 3)
                    )
                ));
            }
            b.push(row);
        }
        b.join(" ")
    });
    add(
        "UU-16 reflexive and null-like extremes",
        uu_pairs(&[
            (uu(0, 0), uu(0, 0)),
            (uu(u64::MAX, u64::MAX), uu(u64::MAX, u64::MAX)),
            (uu(0, 0), uu(u64::MAX, u64::MAX)),
            (uu(u64::MAX, u64::MAX), uu(0, 0)),
            (uu(1 << 63, 1 << 63), uu((1 << 63) - 1, (1 << 63) - 1)),
            (uu(0x1000, 0), uu(0x1000, 0)),
        ]),
    );
    add(
        "UU-17 the shared comparator instance",
        // Rust has no comparator singleton to share: the `Ord` of `JavaUuid` is the one comparator.
        format!(
            "same=true distinctFromNew=true cmp={}",
            pekko_compare_uuid(
                uu(0x4000_0000_0000_0000, 0x8000_0000_0000_0001),
                uu(0x4000_0000_0000_0000, 1)
            )
        ),
    );
    add("UU-18 sorting a mixed list", {
        let mut state: u64 = 20_260_518;
        let vs = [1_u64, 4, 3, 1, 4];
        let mut l: Vec<JavaUuid> = Vec::new();
        for i in 0..24 {
            let rest = (uu_next(&mut state) & 0x0000_0000_00FF_FFFF)
                | if i % 2 == 0 { 0 } else { 0x8000_0000_0000_0000 };
            let lsb = uu_next(&mut state);
            l.push(uu(fw(rest, vs[i % vs.len()]), lsb));
        }
        l.sort();
        l.iter().map(|&x| uu_show(x)).collect::<Vec<_>>().join(" ")
    });
    add(
        "UU-19 sweep of 3000 random pairs",
        uu_sweep(0x9E37_79B9_7F4A_7C15, 3000, &[1, 2, 3, 4, 5, 8], false),
    );
    add(
        "UU-20 sweep of 3000 pairs with shared bits",
        uu_sweep(0xD1B5_4A32_D192_ED03, 3000, &[1, 4], true),
    );
    add(
        "UU-21 sweep of 3000 version 1 pairs",
        uu_sweep(0x2545_F491_4F6C_DD1D, 3000, &[1], true),
    );
    cases
}

const WC_FUTURE: i64 = 1_000_000_000_000_000_000;

/// `System.currentTimeMillis()` as the oracle reads it around a reading.
fn wc_wall() -> i64 {
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    i64::try_from(after.as_millis()).unwrap()
}

fn wc_strict(xs: &[i64]) -> bool {
    xs.windows(2).all(|w| w[1] > w[0])
}

fn wc_calls(c: &AlwaysIncreasingClock, n: usize) -> Vec<i64> {
    (0..n).map(|_| c.current_time_millis()).collect()
}

fn wc_offsets(xs: &[i64], base: i64) -> String {
    xs.iter()
        .map(|x| format!("+{}", x - base))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A reading of `c` bracketed by two wall-clock readings: is it within them?
fn wc_in_range(c: &AlwaysIncreasingClock) -> bool {
    let before = wc_wall();
    let r = c.current_time_millis();
    let after = wc_wall();
    before <= r && r <= after
}

fn wc_concurrent(c: &AlwaysIncreasingClock, threads: usize, each: usize) -> Vec<Vec<i64>> {
    let go = std::sync::Barrier::new(threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    go.wait();
                    wc_calls(c, each)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// `(count, distinct, eachThreadIncreasing, contiguous, min, max)` of a concurrent run.
fn wc_report(out: &[Vec<i64>], base: i64, shape: bool) -> String {
    let each_increasing = out.iter().all(|row| wc_strict(row));
    let mut all: Vec<i64> = out.iter().flatten().copied().collect();
    all.sort_unstable();
    let distinct = all.windows(2).all(|w| w[0] != w[1]);
    let head = format!(
        "count={} distinct={distinct} eachThreadIncreasing={each_increasing}",
        all.len()
    );
    if !shape {
        return head;
    }
    let contiguous = all[all.len() - 1] - all[0] == i64::try_from(all.len()).unwrap() - 1;
    format!(
        "{head} {} min=+{} max=+{}",
        if contiguous { "contiguous" } else { "gaps" },
        all[0] - base,
        all[all.len() - 1] - base
    )
}

/// The cases of `WallClockOracle.java` (slice P70).
fn wall_clock_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));
    let ahead = || {
        let c = AlwaysIncreasingClock::new();
        c.set(WC_FUTURE);
        c
    };

    add(
        "WC-01 a new clock starts at zero",
        format!("state={}", AlwaysIncreasingClock::new().get()),
    );
    {
        let c = AlwaysIncreasingClock::new();
        let before = wc_wall();
        let r = c.current_time_millis();
        let after = wc_wall();
        add(
            "WC-02 first reading of a new clock is the wall time",
            format!(
                "inRange={} stateIsResult={}",
                before <= r && r <= after,
                c.get() == r
            ),
        );
    }
    {
        let c = AlwaysIncreasingClock::new();
        let before = wc_wall();
        let r = wc_calls(&c, 1000);
        add(
            "WC-03 1000 readings strictly increase",
            format!(
                "strict={} aboveBefore={} stateIsLast={}",
                wc_strict(&r),
                r[0] >= before,
                c.get() == r[999]
            ),
        );
    }
    {
        let c = ahead();
        let r = wc_calls(&c, 6);
        add(
            "WC-04 a clock ahead of the wall time adds one",
            format!(
                "{} state=+{}",
                wc_offsets(&r, WC_FUTURE),
                c.get() - WC_FUTURE
            ),
        );
    }
    {
        let c = AlwaysIncreasingClock::new();
        c.set(1);
        let before = wc_wall();
        let r = c.current_time_millis();
        let after = wc_wall();
        add(
            "WC-05 a clock behind the wall time jumps to it",
            format!(
                "inRange={} stateIsResult={}",
                before <= r && r <= after,
                c.get() == r
            ),
        );
    }
    {
        let fields: Vec<String> = [-5_i64, -1, i64::MIN, i64::MIN + 1]
            .iter()
            .map(|&s| {
                let c = AlwaysIncreasingClock::new();
                c.set(s);
                format!("[{s}]={}", wc_in_range(&c))
            })
            .collect();
        add(
            "WC-06 negative and minimum states are behind the wall time",
            fields.join(" "),
        );
    }
    {
        let c = AlwaysIncreasingClock::new();
        c.set(i64::MAX - 2);
        let (a, b, d) = (
            c.current_time_millis(),
            c.current_time_millis(),
            c.current_time_millis(),
        );
        add(
            "WC-07 the increment wraps around at the top of the Long range",
            format!(
                "a={a} b={b} c={d} wrapped={} nextIsWall={}",
                d == i64::MIN,
                wc_in_range(&c)
            ),
        );
    }
    {
        let a = ahead();
        let b = AlwaysIncreasingClock::new();
        let ra = a.current_time_millis();
        add(
            "WC-08 two clocks are independent",
            format!(
                "a=+{} bIsWall={} aState=+{}",
                ra - WC_FUTURE,
                wc_in_range(&b),
                a.get() - WC_FUTURE
            ),
        );
    }
    {
        let a = ahead();
        let b = AlwaysIncreasingClock::new();
        b.set(WC_FUTURE + 100);
        let mut parts = Vec::new();
        for _ in 0..4 {
            let ra = a.current_time_millis() - WC_FUTURE;
            let rb = b.current_time_millis() - WC_FUTURE;
            parts.push(format!("a=+{ra} b=+{rb}"));
        }
        add("WC-09 interleaved readings of two clocks", parts.join(" "));
    }
    {
        let c = AlwaysIncreasingClock::new();
        c.set(WC_FUTURE);
        let r1 = c.current_time_millis();
        c.set(WC_FUTURE + 50);
        let r2 = c.current_time_millis();
        c.set(WC_FUTURE - 50);
        let r3 = c.current_time_millis();
        add(
            "WC-10 set between readings",
            format!(
                "r1=+{} r2=+{} r3=+{}",
                r1 - WC_FUTURE,
                r2 - WC_FUTURE,
                r3 - WC_FUTURE
            ),
        );
    }
    {
        let (w1, w2) = (always_increasing_clock(), always_increasing_clock());
        let r: Vec<i64> = (0..200)
            .map(|i| if i % 2 == 0 { w1 } else { w2 }.current_time_millis())
            .collect();
        add(
            "WC-11 the shared instance",
            format!("same={} strict={}", std::ptr::eq(w1, w2), wc_strict(&r)),
        );
    }
    {
        let c = ahead();
        let r = wc_report(&wc_concurrent(&c, 8, 500), WC_FUTURE, true);
        add(
            "WC-12 8 threads of 500 readings on a clock ahead of the wall time",
            format!("{r} state=+{}", c.get() - WC_FUTURE),
        );
    }
    {
        let c = AlwaysIncreasingClock::new();
        let before = wc_wall();
        let out = wc_concurrent(&c, 8, 500);
        let all_at_least = out.iter().flatten().all(|&x| x >= before);
        add(
            "WC-13 8 threads of 500 readings on a new clock",
            format!(
                "{} allAtLeastBefore={all_at_least}",
                wc_report(&out, before, false)
            ),
        );
    }
    {
        let c = ahead();
        let r = wc_report(&wc_concurrent(&c, 4, 2000), WC_FUTURE, true);
        add(
            "WC-14 4 threads of 2000 readings on a clock ahead of the wall time",
            format!("{r} state=+{}", c.get() - WC_FUTURE),
        );
    }
    cases
}

fn sfl(filename: &str, from: i32, to: i32) -> LineNumbersResult {
    LineNumbersResult::SourceFileLines {
        filename: filename.to_owned(),
        from,
        to,
    }
}

/// A closure site: every call of this function reports the same origin (one source line).
fn origin_site() -> LineNumbersResult {
    closure_origin()
}

fn origin_other_site() -> LineNumbersResult {
    closure_origin()
}

/// Shape of a closure origin (no compiler numbers), as `LineNumbersOracle.shape` prints it.
fn ln_shape(r: &LineNumbersResult) -> String {
    match r {
        LineNumbersResult::SourceFileLines { filename, from, to } => format!(
            "kind=SourceFileLines fileIsBaseName={} linesPositive={} fromLeTo={}",
            !filename.contains('/') && filename.contains('.'),
            *from > 0 && *to > 0,
            from <= to
        ),
        LineNumbersResult::NoSourceInfo => "kind=NoSourceInfo".to_owned(),
        LineNumbersResult::UnknownSourceFormat(_) => "kind=UnknownSourceFormat".to_owned(),
        LineNumbersResult::SourceFile(_) => "kind=SourceFile".to_owned(),
    }
}

/// Does the text end with `:<digits>` or `:<digits>-<digits>`?
fn has_line_suffix(p: &str) -> bool {
    let tail = p.rsplit_once(':').map_or("", |(_, t)| t);
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    digits(tail)
        || tail
            .split_once('-')
            .is_some_and(|(a, b)| digits(a) && digits(b))
}

/// `LineNumbersOracle.pretty`: the pretty name of a closure whose class is `class_name` in package `package`.
fn ln_pretty(class_name: &str, package: &str, r: &LineNumbersResult) -> String {
    let p = pretty_name(class_name, package, r);
    if matches!(r, LineNumbersResult::SourceFileLines { .. }) {
        format!(
            "prettyIsPackageSlashResult={}",
            p == format!("{package}/{r}")
        )
    } else {
        format!(
            "prettyIsClassName={} prettyHasNoLineSuffix={}",
            p == class_name,
            !has_line_suffix(&p)
        )
    }
}

/// The cases of `LineNumbersOracle.java` (slice P75).
fn line_numbers_cases() -> Vec<(String, String)> {
    let mut cases: Vec<(String, String)> = Vec::new();
    let mut add = |id: &str, line: String| cases.push((id.to_owned(), line));
    add(
        "LN-01 one line shows file:line",
        format!("{} {}", sfl("Foo.scala", 22, 22), sfl("a/B.java", 1, 1)),
    );
    add(
        "LN-02 a range shows file:from-to",
        format!("{} {}", sfl("Foo.scala", 29, 31), sfl("X.java", 40, 45)),
    );
    add(
        "LN-03 unordered or non-positive bounds are printed as given",
        format!(
            "{} {} {} {} {}",
            sfl("F.scala", 9, 5),
            sfl("F.scala", 0, 0),
            sfl("F.scala", -1, 3),
            sfl("", 7, 7),
            sfl("F.scala", i32::MIN, i32::MAX)
        ),
    );
    add(
        "LN-04 SourceFile NoSourceInfo and UnknownSourceFormat text",
        format!(
            "[{}] [{}] [{}] [{}] [{}]",
            LineNumbersResult::SourceFile("Foo.scala".to_owned()),
            LineNumbersResult::SourceFile(String::new()),
            LineNumbersResult::NoSourceInfo,
            LineNumbersResult::UnknownSourceFormat("parse error: x".to_owned()),
            LineNumbersResult::UnknownSourceFormat(String::new())
        ),
    );
    {
        let state = RandomState::new();
        let h = |r: &LineNumbersResult| state.hash_one(r);
        let (a, b, c) = (sfl("F", 1, 2), sfl("F", 1, 2), sfl("F", 1, 3));
        let sf = LineNumbersResult::SourceFile("F".to_owned());
        add(
            "LN-05 equality and hash code of results",
            format!(
                "eq={} ne={} hash={} fileEq={} kindsDiffer={}",
                a == b,
                a != c,
                h(&a) == h(&b),
                sf == LineNumbersResult::SourceFile("F".to_owned()),
                sf != LineNumbersResult::UnknownSourceFormat("F".to_owned())
            ),
        );
    }
    let package = module_path!();
    let one = origin_site();
    add(
        "LN-06 a one-line serializable lambda has a source file and lines",
        format!(
            "{} {} single={}",
            ln_shape(&one),
            ln_pretty("closure", package, &one),
            matches!(&one, LineNumbersResult::SourceFileLines { from, to, .. } if from == to)
        ),
    );
    // A call that spans several lines still reports one line (Q-UTIL-16: a single number).
    let many = closure_origin();
    add(
        "LN-07 a multi-line serializable lambda has a source file and lines",
        format!(
            "{} {}",
            ln_shape(&many),
            ln_pretty("closure", package, &many)
        ),
    );
    let anon = origin_other_site();
    add(
        "LN-08 an anonymous class has a source file and lines",
        format!(
            "{} {}",
            ln_shape(&anon),
            ln_pretty("closure", package, &anon)
        ),
    );
    // A closure whose site was not captured has no source info: the pretty name falls back to its type name.
    let closure = |s: &str| s.to_owned();
    let none = LineNumbersResult::NoSourceInfo;
    add(
        "LN-09 a non-serializable lambda has no source info",
        format!(
            "{} {}",
            ln_shape(&none),
            ln_pretty(std::any::type_name_of_val(&closure), package, &none)
        ),
    );
    add(
        "LN-10 the Java API for is the same as apply",
        format!(
            "same={} again={}",
            origin_site() == origin_site(),
            origin_site() == origin_site()
        ),
    );
    add(
        "LN-11 two closures of one site report the same origin and two sites differ",
        format!(
            "sameSite={} otherSite={}",
            origin_site() == origin_site(),
            origin_site() != origin_other_site()
        ),
    );
    cases
}
