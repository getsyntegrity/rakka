// Derived from Apache Pekko 2.0.0 actor/src/main/resources/reference.conf and actor-typed/src/main/resources/reference.conf (Apache-2.0).
//! Cases of the AC-CFG (parser and loading) oracle (`ConfigHoconOracle.java`): each rebuilds one oracle case with the
//! Rust `Config` API and renders it with the oracle's line format. The `KEY` cases list every leaf key of the
//! embedded `reference.conf` files.

use crate::text_escape::escape_non_ascii_units;
use rakka_actor::config::{ACTOR_REFERENCE_CONF, Config, ConfigError, TYPED_REFERENCE_CONF};
use rakka_actor::util::TimeUnit;

/// Oracle ids Rust does not produce on purpose: `(id, reason)`. Each must exist in the oracle.
#[allow(dead_code)]
pub const PENDING: &[(&str, &str)] = &[];

fn parse(text: &str) -> Result<Config, ConfigError> {
    Config::parse_string(text)
}

fn list<T: ToString>(items: Vec<T>) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn dur(text: &str, unit: TimeUnit) -> Result<String, ConfigError> {
    Ok(parse(&format!("x = \"{text}\""))?
        .get_duration("x", unit)?
        .to_string())
}

fn size(text: &str) -> Result<String, ConfigError> {
    Ok(parse(&format!("x = \"{text}\""))?
        .get_bytes("x")?
        .to_string())
}

fn java_double(v: f64) -> String {
    format!("{v:?}")
}

fn typed_merged() -> Config {
    let actor = Config::parse_string(ACTOR_REFERENCE_CONF).expect("actor reference.conf parses");
    Config::parse_string(TYPED_REFERENCE_CONF)
        .expect("typed reference.conf parses")
        .with_fallback(&actor)
}

pub fn rust_lines() -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut case = |id: String, r: Result<String, ConfigError>| {
        let line = match r {
            Ok(v) => format!("ok {v}"),
            Err(e) => format!("ERR {} [{}]", e.kind().java_name(), e.message()),
        };
        lines.push((
            escape_non_ascii_units(&id),
            escape_non_ascii_units(&format!("{id} -> {line}")),
        ));
    };
    let s = |v: &str| v.to_owned();

    // --- FB: withFallback
    case(
        "FB-01 receiver scalar wins over fallback scalar".into(),
        (|| {
            Ok(parse("a = 1")?
                .with_fallback(&parse("a = 2")?)
                .get_int("a")?
                .to_string())
        })(),
    );
    case(
        "FB-02 objects merge key by key, receiver wins on conflict".into(),
        (|| {
            let c = parse("p { a = 1, b = 2 }")?.with_fallback(&parse("p { b = 9, c = 3 }")?);
            Ok(format!(
                "{},{},{}",
                c.get_int("p.a")?,
                c.get_int("p.b")?,
                c.get_int("p.c")?
            ))
        })(),
    );
    case(
        "FB-03 scalar in receiver replaces object in fallback".into(),
        (|| {
            Ok(s(parse("p = 1")?
                .with_fallback(&parse("p { b = 9 }")?)
                .get_value("p")?
                .value_type()
                .name()))
        })(),
    );
    case(
        "FB-04 object in receiver over scalar in fallback keeps the receiver object".into(),
        (|| {
            Ok(s(parse("p { b = 9 }")?
                .with_fallback(&parse("p = 1")?)
                .get_value("p")?
                .value_type()
                .name()))
        })(),
    );
    case(
        "FB-05 substitution is late-bound: override of referenced key is seen".into(),
        (|| {
            let reference = parse("a.base = 1\na.derived = ${a.base}")?;
            let app = parse("a.base = 2")?;
            Ok(app
                .with_fallback(&reference)
                .resolve()?
                .get_int("a.derived")?
                .to_string())
        })(),
    );
    case(
        "FB-06 resolving the fallback first freezes the substitution".into(),
        (|| {
            let reference = parse("a.base = 1\na.derived = ${a.base}")?.resolve()?;
            let app = parse("a.base = 2")?;
            Ok(app
                .with_fallback(&reference)
                .resolve()?
                .get_int("a.derived")?
                .to_string())
        })(),
    );
    case(
        "FB-07 chain of three: first receiver wins".into(),
        (|| {
            Ok(parse("x = 1")?
                .with_fallback(&parse("x = 2")?)
                .with_fallback(&parse("x = 3")?)
                .get_int("x")?
                .to_string())
        })(),
    );
    case(
        "FB-08 explicit config only (no fallback) does not see reference keys".into(),
        (|| {
            Ok(parse("pekko.loglevel = DEBUG")?
                .has_path("pekko.actor.provider")?
                .to_string())
        })(),
    );
    case(
        "FB-09 hasPath on null value is false".into(),
        (|| Ok(parse("a = null")?.has_path("a")?.to_string()))(),
    );
    case(
        "FB-10 hasPathOrNull on null value is true".into(),
        (|| Ok(parse("a = null")?.has_path_or_null("a")?.to_string()))(),
    );

    // --- SUB: substitutions
    case(
        "SUB-01 optional substitution of a missing path leaves the key absent".into(),
        (|| {
            Ok(parse("x = ${?no.such.path}")?
                .resolve()?
                .has_path("x")?
                .to_string())
        })(),
    );
    case(
        "SUB-02 optional substitution concatenated with an array (reference.conf:78)".into(),
        (|| {
            Ok(list(
                parse("e = ${?pekko.library-extensions} [\"d\"]")?
                    .resolve()?
                    .get_string_list("e")?,
            ))
        })(),
    );
    case(
        "SUB-03 same, with the optional path defined by the application".into(),
        (|| {
            Ok(list(
                parse(
                    "pekko.library-extensions = [\"u\"]\ne = ${?pekko.library-extensions} [\"d\"]",
                )?
                .resolve()?
                .get_string_list("e")?,
            ))
        })(),
    );
    case(
        "SUB-04 required substitution of a missing path fails".into(),
        (|| {
            Ok(parse("x = ${no.such.path}")?
                .resolve()?
                .has_path("x")?
                .to_string())
        })(),
    );
    case(
        "SUB-05 self-reference concatenates strings".into(),
        (|| parse("s = foo\ns = ${s}bar")?.resolve()?.get_string("s"))(),
    );
    case(
        "SUB-06 += appends to an existing list".into(),
        (|| {
            Ok(list(
                parse("a = [1]\na += 2")?.resolve()?.get_int_list("a")?,
            ))
        })(),
    );
    case(
        "SUB-07 += on a missing key creates the list".into(),
        (|| Ok(list(parse("a += 2")?.resolve()?.get_int_list("a")?)))(),
    );
    case(
        "SUB-08 substitution to an unset environment variable, optional".into(),
        (|| {
            Ok(parse("x = ${?RAKKA_ORACLE_UNSET_VARIABLE_42}")?
                .resolve()?
                .has_path("x")?
                .to_string())
        })(),
    );
    case(
        "SUB-09 substitution to an unset environment variable, required".into(),
        (|| {
            Ok(parse("x = ${RAKKA_ORACLE_UNSET_VARIABLE_42}")?
                .resolve()?
                .has_path("x")?
                .to_string())
        })(),
    );
    case(
        "SUB-10 cyclic substitution fails".into(),
        (|| {
            Ok(parse("a = ${b}\nb = ${a}")?
                .resolve()?
                .has_path("a")?
                .to_string())
        })(),
    );
    case(
        "SUB-11 substitution inside a quoted string is not substituted".into(),
        (|| parse("s = \"${a}\"\na = 1")?.resolve()?.get_string("s"))(),
    );
    case(
        "SUB-12 a getter on an unresolved config fails".into(),
        (|| Ok(parse("a = 1\nb = ${a}")?.get_int("b")?.to_string()))(),
    );
    case(
        "SUB-13 substitution of an object merges with a later object".into(),
        (|| {
            Ok(list(
                parse("base { x = 1 }\nd = ${base} { y = 2 }")?
                    .resolve()?
                    .get_config("d")?
                    .root_keys(),
            ))
        })(),
    );
    case(
        "SUB-14 reference.conf:878 shape: key set to another key's list".into(),
        (|| {
            Ok(list(
                parse("w = [\"a\", \"b\"]\nallowed = ${w}")?
                    .resolve()?
                    .get_string_list("allowed")?,
            ))
        })(),
    );

    // --- INC: includes
    case(
        "INC-01 include of a missing file is ignored".into(),
        (|| {
            Ok(parse("include \"no-such-file-oracle\"\nx = 1")?
                .get_int("x")?
                .to_string())
        })(),
    );
    case(
        "INC-02 required include of a missing file fails".into(),
        (|| {
            Ok(parse("include required(\"no-such-file-oracle\")\nx = 1")?
                .get_int("x")?
                .to_string())
        })(),
    );
    case(
        "INC-03 file() include of a missing file is ignored".into(),
        (|| {
            Ok(parse("include file(\"/no/such/file/oracle.conf\")\nx = 1")?
                .get_int("x")?
                .to_string())
        })(),
    );
    case(
        "INC-04 classpath() include of a missing resource is ignored".into(),
        (|| {
            Ok(
                parse("include classpath(\"no-such-resource-oracle\")\nx = 1")?
                    .get_int("x")?
                    .to_string(),
            )
        })(),
    );

    // --- DUR: durations
    for t in [
        "10 ms",
        "10ms",
        "10 milliseconds",
        "1 us",
        "1us",
        "5 ns",
        "1 d",
        "1d",
        "2 minutes",
        "5 minutes",
        "5s",
        "5 s",
        "1 h",
        "100",
        "1.5 s",
        "0.5 ms",
        "-1s",
        "0",
        "1 hour",
        "1 day",
    ] {
        case(format!("DUR-MS {t}"), dur(t, TimeUnit::Milliseconds));
        case(format!("DUR-NS {t}"), dur(t, TimeUnit::Nanoseconds));
    }
    for t in ["5 foo", "", "abc", "1 s s", "infinite", "off"] {
        case(format!("DUR-ERR {t}"), dur(t, TimeUnit::Milliseconds));
    }
    case(
        "DUR-SCI scientific notation is accepted".into(),
        dur("1e3 ms", TimeUnit::Milliseconds),
    );
    case(
        "DUR-UNQUOTED 5 minutes".into(),
        (|| {
            Ok(parse("x = 5 minutes")?
                .get_duration("x", TimeUnit::Milliseconds)?
                .to_string())
        })(),
    );
    case(
        "DUR-NUMBER bare number is milliseconds".into(),
        (|| {
            Ok(parse("x = 100")?
                .get_duration("x", TimeUnit::Milliseconds)?
                .to_string())
        })(),
    );
    case(
        "DUR-ROUND 1 ns in millis truncates toward zero".into(),
        dur("1 ns", TimeUnit::Milliseconds),
    );
    case(
        "DUR-ROUND 999999 ns in millis truncates toward zero".into(),
        dur("999999 ns", TimeUnit::Milliseconds),
    );
    case(
        "DUR-ROUND -1500 us in millis truncates toward zero".into(),
        dur("-1500 us", TimeUnit::Milliseconds),
    );
    case(
        "DUR-LARGE overflow".into(),
        dur("106752 d", TimeUnit::Nanoseconds),
    );
    case(
        "DUR-LARGE 292 years in ns still fits".into(),
        dur("106751 d", TimeUnit::Nanoseconds),
    );

    // --- SIZE: memory sizes
    for t in [
        "128 KiB",
        "128KiB",
        "128 K",
        "128 k",
        "1 MB",
        "1MB",
        "1 kB",
        "1 KB",
        "512",
        "512 B",
        "512 bytes",
        "1.5 KiB",
        "1 GiB",
        "1 MiB",
        "1 g",
        "0",
        "-1 KiB",
        "1 TiB",
        "256 MiB",
    ] {
        case(format!("SIZE {t}"), size(t));
    }
    for t in [
        "unlimited",
        "128 XiB",
        "",
        "KiB",
        "1 KiB KiB",
        "8 EiB",
        "8388608 TiB",
    ] {
        case(format!("SIZE-ERR {t}"), size(t));
    }
    case(
        "SIZE-NUMBER bare number is bytes".into(),
        (|| Ok(parse("x = 4096")?.get_bytes("x")?.to_string()))(),
    );
    case(
        "SIZE-MEMORY getMemorySize".into(),
        (|| Ok(parse("x = 4 MiB")?.get_memory_size("x")?.to_string()))(),
    );

    // --- BOOL: booleans
    for v in [
        "on",
        "off",
        "yes",
        "no",
        "true",
        "false",
        "\"on\"",
        "\"off\"",
        "\"true\"",
        "\"yes\"",
        "\"maybe\"",
        "1",
        "\"ON\"",
    ] {
        case(
            format!("BOOL {v}"),
            (|| Ok(parse(&format!("x = {v}"))?.get_boolean("x")?.to_string()))(),
        );
    }

    // --- TYPE: getters
    case(
        "TYPE-01 missing key".into(),
        (|| Ok(parse("a = 1")?.get_int("b")?.to_string()))(),
    );
    case(
        "TYPE-02 string where int expected".into(),
        (|| Ok(parse("a = abc")?.get_int("a")?.to_string()))(),
    );
    case(
        "TYPE-03 quoted number as int".into(),
        (|| Ok(parse("a = \"7\"")?.get_int("a")?.to_string()))(),
    );
    case(
        "TYPE-04 int where string expected".into(),
        (|| parse("a = 7")?.get_string("a"))(),
    );
    case(
        "TYPE-05 object where string expected".into(),
        (|| parse("a { b = 1 }")?.get_string("a"))(),
    );
    case(
        "TYPE-06 list where string expected".into(),
        (|| parse("a = [1]")?.get_string("a"))(),
    );
    case(
        "TYPE-07 null where string expected".into(),
        (|| parse("a = null")?.get_string("a"))(),
    );
    case(
        "TYPE-08 long beyond int".into(),
        (|| Ok(parse("a = 3000000000")?.get_int("a")?.to_string()))(),
    );
    case(
        "TYPE-09 double as int".into(),
        (|| Ok(parse("a = 1.5")?.get_int("a")?.to_string()))(),
    );
    case(
        "TYPE-10 whole double as int".into(),
        (|| Ok(parse("a = 2.0")?.get_int("a")?.to_string()))(),
    );
    case(
        "TYPE-11 getConfig on scalar".into(),
        (|| {
            parse("a = 1")?
                .get_config("a")
                .map(|_| "<config>".to_owned())
        })(),
    );
    case(
        "TYPE-12 getString of boolean".into(),
        (|| parse("a = on")?.get_string("a"))(),
    );
    case(
        "TYPE-13 getDouble of int".into(),
        (|| Ok(java_double(parse("a = 3")?.get_double("a")?)))(),
    );

    // --- SPECIAL: literals Pekko interprets itself
    case(
        "SPECIAL-01 log-dead-letters = on is a boolean".into(),
        (|| Ok(s(parse("x = on")?.get_value("x")?.value_type().name())))(),
    );
    case(
        "SPECIAL-02 log-dead-letters = 10 is a number".into(),
        (|| Ok(s(parse("x = 10")?.get_value("x")?.value_type().name())))(),
    );
    case(
        "SPECIAL-03 infinite is a string".into(),
        (|| {
            Ok(s(parse("x = infinite")?
                .get_value("x")?
                .value_type()
                .name()))
        })(),
    );
    case(
        "SPECIAL-04 unlimited is a string".into(),
        (|| parse("x = unlimited")?.get_string("x"))(),
    );
    case(
        "SPECIAL-05 off is a string by getString".into(),
        (|| parse("x = off")?.get_string("x"))(),
    );
    case(
        "SPECIAL-06 on is a string by getString".into(),
        (|| parse("x = on")?.get_string("x"))(),
    );
    case(
        "SPECIAL-07 5 minutes unquoted is a string".into(),
        (|| parse("x = 5 minutes")?.get_string("x"))(),
    );
    case(
        "SPECIAL-08 duration getter on infinite fails".into(),
        (|| {
            Ok(parse("x = infinite")?
                .get_duration("x", TimeUnit::Milliseconds)?
                .to_string())
        })(),
    );

    // --- LIST: lists
    case(
        "LIST-01 string list".into(),
        (|| Ok(list(parse("l = [\"a\", \"b\"]")?.get_string_list("l")?)))(),
    );
    case(
        "LIST-02 empty list".into(),
        (|| Ok(list(parse("l = []")?.get_string_list("l")?)))(),
    );
    case(
        "LIST-03 mixed list as string list coerces".into(),
        (|| Ok(list(parse("l = [1, true, x]")?.get_string_list("l")?)))(),
    );
    case(
        "LIST-04 list of objects".into(),
        (|| {
            Ok(parse("l = [{a=1}, {a=2}]")?
                .get_config_list("l")?
                .len()
                .to_string())
        })(),
    );
    case(
        "LIST-05 numeric-keyed object as list".into(),
        (|| {
            Ok(list(
                parse("l { \"0\" = a, \"1\" = b }")?.get_string_list("l")?,
            ))
        })(),
    );
    case(
        "LIST-06 trailing comma allowed".into(),
        (|| Ok(list(parse("l = [1, 2,]")?.get_int_list("l")?)))(),
    );
    case(
        "LIST-07 missing comma between elements on one line fails".into(),
        (|| Ok(list(parse("l = [1 2]")?.get_int_list("l")?)))(),
    );

    // --- REF: reference.conf excerpts
    case(
        "REF-01 library-extensions default (line 78)".into(),
        (|| {
            Ok(list(
            parse("pekko.library-extensions = ${?pekko.library-extensions} [\"org.apache.pekko.serialization.SerializationExtension$\"]")?
                .resolve()?
                .get_string_list("pekko.library-extensions")?,
        ))
        })(),
    );
    case(
        "REF-02 library-extensions with application value first".into(),
        (|| {
            let app = parse("pekko.library-extensions += \"my.Ext\"")?;
            let reference = parse(
                "pekko.library-extensions = ${?pekko.library-extensions} [\"org.apache.pekko.serialization.SerializationExtension$\"]",
            )?;
            Ok(list(
                app.with_fallback(&reference)
                    .resolve()?
                    .get_string_list("pekko.library-extensions")?,
            ))
        })(),
    );
    case(
        "REF-03 max-decompressed-size = unlimited (line 863)".into(),
        (|| {
            parse("pekko.serialization.max-decompressed-size = unlimited")?
                .get_string("pekko.serialization.max-decompressed-size")
        })(),
    );
    case(
        "REF-04 direct-buffer-size = 128 KiB (line 966)".into(),
        (|| {
            Ok(parse("pekko.io.tcp.direct-buffer-size = 128 KiB")?
                .get_bytes("pekko.io.tcp.direct-buffer-size")?
                .to_string())
        })(),
    );
    case(
        "REF-05 log-dead-letters-suspend-duration = 5 minutes (line 68)".into(),
        (|| {
            Ok(
                parse("pekko.log-dead-letters-suspend-duration = 5 minutes")?
                    .get_duration(
                        "pekko.log-dead-letters-suspend-duration",
                        TimeUnit::Milliseconds,
                    )?
                    .to_string(),
            )
        })(),
    );
    case(
        "REF-06 key with dots in quotes is one key".into(),
        (|| Ok(parse("a.\"b.c\" = 1")?.has_path("a.\"b.c\"")?.to_string()))(),
    );
    case(
        "REF-07 unquoted key with hyphens".into(),
        (|| {
            Ok(parse("fork-join-executor { parallelism-min = 8 }")?
                .get_int("fork-join-executor.parallelism-min")?
                .to_string())
        })(),
    );
    case(
        "REF-08 comment styles # and //".into(),
        (|| {
            let a = parse("# c\n// c\na = 1 # trailing\nb = 2 // trailing")?.get_int("a")?;
            let b = parse("b = 2")?.get_int("b")?;
            Ok((a + b).to_string())
        })(),
    );
    case(
        "REF-09 triple-quoted string keeps newlines".into(),
        (|| parse("s = \"\"\"a\nb\"\"\"")?.get_string("s"))(),
    );
    case(
        "REF-10 duplicate keys: later object merges, later scalar wins".into(),
        (|| {
            let c = parse("a { x = 1 }\na { y = 2 }\nb = 1\nb = 2")?;
            Ok(format!(
                "{},{},{}",
                c.get_int("a.x")?,
                c.get_int("a.y")?,
                c.get_int("b")?
            ))
        })(),
    );

    // --- CV: checkValid (AC-CFG-019)
    let cv = |reference: &str, config: &str, restrict: &[&str]| -> Result<String, ConfigError> {
        parse(config)?
            .resolve()?
            .check_valid(&parse(reference)?.resolve()?, restrict)?;
        Ok(s("valid"))
    };
    case(
        "CV-01 a config equal to the reference is valid".into(),
        cv(
            "pekko { a = 1, b = abc }",
            "pekko { a = 1, b = abc }",
            &["pekko"],
        ),
    );
    case(
        "CV-02 a key missing from the config".into(),
        cv("pekko { a = 1, b = abc }", "pekko { a = 1 }", &["pekko"]),
    );
    case(
        "CV-03 string where a number is expected is accepted (a string can be converted)".into(),
        cv("pekko { a = 1 }", "pekko { a = abc }", &["pekko"]),
    );
    case(
        "CV-04 object where a number is expected".into(),
        cv("pekko { a = 1 }", "pekko { a { x = 1 } }", &["pekko"]),
    );
    case(
        "CV-05 number where an object is expected".into(),
        cv("pekko { a { x = 1 } }", "pekko { a = 2 }", &["pekko"]),
    );
    case(
        "CV-06 number where a list is expected".into(),
        cv("pekko { a = [1] }", "pekko { a = 2 }", &["pekko"]),
    );
    case(
        "CV-07 numeric-keyed object where a list is expected".into(),
        cv("pekko { a = [1] }", "pekko { a { \"0\" = 1 } }", &["pekko"]),
    );
    case(
        "CV-08 number where a string is expected".into(),
        cv("pekko { a = abc }", "pekko { a = 2 }", &["pekko"]),
    );
    case(
        "CV-09 boolean where a number is expected".into(),
        cv("pekko { a = 1 }", "pekko { a = true }", &["pekko"]),
    );
    case(
        "CV-10 list where a number is expected".into(),
        cv("pekko { a = 1 }", "pekko { a = [1] }", &["pekko"]),
    );
    case(
        "CV-11 several problems are reported together".into(),
        cv(
            "pekko { a = 1, b = 2, c { d = 1 } }",
            "pekko { a { x = 1 }, c = 3 }",
            &["pekko"],
        ),
    );
    case(
        "CV-12 problems outside the restricted path are not checked".into(),
        cv(
            "pekko { a = 1 }\nother { b = 1 }",
            "pekko { a = 1 }",
            &["pekko"],
        ),
    );
    case(
        "CV-13 no restriction checks the whole reference".into(),
        cv("pekko { a = 1 }\nother { b = 1 }", "pekko { a = 1 }", &[]),
    );
    case(
        "CV-14 null where a number is expected is accepted".into(),
        cv("pekko { a = 1 }", "pekko { a = null }", &["pekko"]),
    );
    case(
        "CV-15 extra keys in the config are accepted".into(),
        cv("pekko { a = 1 }", "pekko { a = 1, z = 2 }", &["pekko"]),
    );
    case(
        "CV-16 withoutPath drops a key of the reference".into(),
        (|| {
            let reference = parse("pekko { a = 1, internal-dispatcher = abc }")?
                .resolve()?
                .without_path("pekko.internal-dispatcher")?;
            parse("pekko { a = 1, internal-dispatcher { x = 1 } }")?
                .resolve()?
                .check_valid(&reference, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );
    case(
        "CV-17 the same without withoutPath fails".into(),
        (|| {
            let reference = parse("pekko { a = 1, internal-dispatcher { y = 1 } }")?.resolve()?;
            parse("pekko { a = 1, internal-dispatcher = abc }")?
                .resolve()?
                .check_valid(&reference, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );
    case(
        "CV-18 an unresolved config fails".into(),
        (|| {
            parse("pekko { a = ${b} }")?
                .check_valid(&parse("pekko { a = 1 }")?.resolve()?, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );
    case(
        "CV-19 the real reference.conf of pekko-actor is valid against itself".into(),
        (|| {
            let reference = parse(ACTOR_REFERENCE_CONF)?.resolve()?;
            reference.check_valid(&reference, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );
    case(
        "CV-20 the real reference.conf is valid for an application that overrides some keys".into(),
        (|| {
            let reference = parse(ACTOR_REFERENCE_CONF)?.resolve()?;
            let app = parse(
                "pekko.loglevel = DEBUG\npekko.log-dead-letters = off\npekko.actor.provider = local",
            )?;
            app.with_fallback(&reference)
                .resolve()?
                .check_valid(&reference, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );
    case(
        "CV-21 the real reference.conf fails when a key it defines is an object in the application"
            .into(),
        (|| {
            let reference = parse(ACTOR_REFERENCE_CONF)?.resolve()?;
            let app = parse("pekko.loglevel { x = 1 }")?;
            app.with_fallback(&reference)
                .resolve()?
                .check_valid(&reference, &["pekko"])?;
            Ok(s("valid"))
        })(),
    );

    // --- RW: renderWithRedactions (AC-UTIL-002.5)
    case(
        "RW-01 render of scalars, nested objects, lists, null and empty values".into(),
        (|| {
            Ok(parse("a = 1\nb = hello\nc { d = [1, 2], e = \"x y\" }\nl = []\no {}\nn = null\nf = 1.5\nbool = true\nw = 1.0")?
            .resolve()?
            .render_with_redactions())
        })(),
    );
    case(
        "RW-02 an environment variable value is not shown".into(),
        (|| Ok(parse("x = ${PATH}")?.resolve()?.render_with_redactions()))(),
    );
    case(
        "RW-03 the user name is replaced".into(),
        (|| {
            let user = std::env::var("USER").unwrap_or_default();
            Ok(parse(&format!("x = \"home of {user}\""))?
                .resolve()?
                .render_with_redactions())
        })(),
    );
    case(
        "RW-04 render of an unresolved config".into(),
        (|| Ok(parse("a = 1\nb = ${a}")?.render_with_redactions()))(),
    );
    case(
        "RW-05 render of a merge of two configs".into(),
        (|| {
            Ok(parse("a.x = 1")?
                .with_fallback(&parse("a.y = 2\nb = 3")?)
                .resolve()?
                .render_with_redactions())
        })(),
    );
    case(
        "RW-06 keys that need quotes and escapes in strings".into(),
        (|| {
            Ok(parse("\"a.b\" = 1\n\"c d\" = \"q\\\"uote\\nline\"")?
                .resolve()?
                .render_with_redactions())
        })(),
    );

    // --- KEY: every leaf key of the reference.conf files
    let actor = Config::parse_string(ACTOR_REFERENCE_CONF).and_then(|c| c.resolve());
    let typed = typed_merged().resolve();
    if let (Ok(actor), Ok(typed)) = (actor, typed) {
        let actor_leaves = actor.leaves();
        for (path, v) in &actor_leaves {
            case(
                format!("KEY actor {path}"),
                Ok(format!("{} {}", v.value_type().name(), v.render_concise())),
            );
        }
        let base: std::collections::HashMap<_, _> = actor_leaves
            .iter()
            .map(|(p, v)| (p.clone(), v.render_concise()))
            .collect();
        for (path, v) in typed.leaves() {
            if base.get(&path) != Some(&v.render_concise()) {
                case(
                    format!("KEY actor-typed {path}"),
                    Ok(format!("{} {}", v.value_type().name(), v.render_concise())),
                );
            }
        }
    }

    // --- NUM-RENDER: how a parsed number is rendered (D-BUG-01)
    for t in [
        "1.50",
        "1e-7",
        "12345678.9",
        "-0",
        "010",
        "1e400",
        "1.5e-3",
        "5e18",
        "1e18",
        "1.0",
        "100.0",
        "0.1",
        "1E5",
        "1e2",
        "123456789012345678",
        "-1.50",
        "0.0",
        "1e+2",
        "-0.0",
        "9223372036854775807",
        "1e19",
        "-1e19",
        "9223372036854775808.0",
        "1.7976931348623157e308",
        "4.9e-324",
        "123456.789",
        "0.001",
        "0.0001",
        "1234567.0",
        "12345678.5",
        "-2.5e-5",
        "0.30000000000000004",
    ] {
        case(
            format!("NUM-RENDER {t}"),
            (|| Ok(parse(&format!("x = {t}"))?.get_value("x")?.render_concise()))(),
        );
    }

    // --- NUMLEX: which literals are numbers (D-BUG-02)
    for t in [
        "1.",
        ".5",
        "-.5",
        "1.e5",
        "1e5",
        "1E5",
        "1e",
        "1e+",
        "1e+5",
        "-1",
        "--1",
        "+1",
        "00",
        "007",
        "1.5.2",
        "1.2.3",
        "0x10",
        "1_000",
        "99999999999999999999",
        "-99999999999999999999",
        "9223372036854775807",
        "9223372036854775808",
        "-9223372036854775808",
        "-9223372036854775809",
        "1+2",
        "1 2",
        "1-2",
        "1.5abc",
        "123abc",
        "abc123",
        "-",
        "-a",
        "1e400",
        "1e-400",
        "0.0e0",
        "1..2",
        "1.e",
        "3.14",
        "1E+5",
        "-0",
        "- 1",
        "1 .5",
        "-.",
        "-e5",
        "1.5e",
        "-1.5E-3",
    ] {
        case(
            format!("NUMLEX {t}"),
            Ok((|| -> Result<String, ConfigError> {
                let c = parse(&format!("x = {t}"))?;
                let v = c.get_value("x")?;
                Ok(format!(
                    "{} {} {}",
                    v.value_type().name(),
                    v.render_concise(),
                    c.get_string("x")?
                ))
            })()
            .unwrap_or_else(|e| format!("error {}", e.kind().java_name()))),
        );
    }

    // --- DUR-SAT: durations saturate at Long.MAX_VALUE / MIN_VALUE nanoseconds before the target unit (D-BUG-09)
    for t in [
        "106752 d",
        "106751 d",
        "-106752 d",
        "-106753 d",
        "9223372036854775807 ms",
        "9223372036854775807 s",
        "1e30 s",
        "-1e30 s",
        "1e18 ns",
        "9.3e18 ns",
        "9.3e18 us",
        ".5 s",
        "+5 s",
        "+5.5 s",
        "-.5 s",
        "5. s",
        "NaN ms",
        "Infinity ms",
        "-Infinity ms",
        "+Infinity ms",
        "0x10 ms",
        "1E3 ms",
        "5e-1 s",
        "0.0000005 s",
        "1.5 ns",
        "-1.5 ns",
        "2.9 ns",
        "106751.99 d",
        "92233720368 s",
        "92233720369 s",
        "9223372036854775807",
        "-9223372036854775808 ns",
        "-9223372036854775809 ns",
        "3e-10 d",
        "+ ms",
        "- s",
        "1.2.3 s",
        "1e s",
        "5 +s",
    ] {
        case(format!("DUR-SAT NS {t}"), dur(t, TimeUnit::Nanoseconds));
        case(format!("DUR-SAT MS {t}"), dur(t, TimeUnit::Milliseconds));
        case(format!("DUR-SAT SEC {t}"), dur(t, TimeUnit::Seconds));
        case(format!("DUR-SAT DAY {t}"), dur(t, TimeUnit::Days));
    }

    // --- SIZE-DEC: sizes are exact decimal arithmetic, and out-of-range messages show the exact product (D-BUG-10)
    for t in [
        "8.2 MB",
        "8.2 MiB",
        "0.1 GB",
        "1.5 KiB",
        "-1.5 KiB",
        "-0.5 B",
        "0.9 B",
        "1.1 kB",
        "8388607.9 TiB",
        "8388608 TiB",
        "8 EiB",
        "7.99 EiB",
        "9223372036854775807 YiB",
        "9223372036854775807 B",
        "9223372036854775808 B",
        "-9223372036854775808 B",
        "-9223372036854775809 B",
        "+5 B",
        "+5.5 kB",
        ".5 kB",
        "5. kB",
        "1e3 B",
        "1E3 kB",
        "1e-3 kB",
        "2.5e2 B",
        "1e30 B",
        "1e-30 GB",
        "NaN B",
        "Infinity B",
        "0x10 B",
        "1.2.3 B",
        "1e B",
        "00012 B",
        "0.000000000000000000000000001 YiB",
        "123456789012345678901234567890123456789012345678901e-30 B",
        "1 YiB",
        "0 YiB",
        "8.0 EiB",
        "9.2233720368547758 EB",
        "9.223372036854775807 EB",
        "9.223372036854775808 EB",
        "-9.223372036854775808 EB",
        "-9.223372036854775809 EB",
        "0.5 kB",
        "2.5 MB",
        "1023.9 B",
        "12345678.9 kB",
    ] {
        case(format!("SIZE-DEC {t}"), size(t));
    }
    lines
}
