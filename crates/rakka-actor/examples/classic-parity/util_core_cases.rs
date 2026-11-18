// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/{Timeout,Helpers,Glob,Clock,SWARUtil}.scala, Done.scala, NotUsed.scala, PekkoException.scala and actor/src/main/java/org/apache/pekko/util/Unsafe.java (Apache-2.0).
//! Cases of the AC-UTIL (core) oracle (`UtilCoreOracle.java`, converted from the deprecated jshell script): each
//! rebuilds one oracle case with the Rust API and renders it with the oracle's line format.

use std::hash::{BuildHasher, RandomState};
use std::sync::Mutex;
use std::time::Duration;

use rakka_actor::actor::IllegalArgumentError;
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
use rakka_actor::util::swar::{self, ByteOrder};
use rakka_actor::util::{TimeUnit, Timeout};
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
