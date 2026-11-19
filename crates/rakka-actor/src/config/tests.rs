// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Unit tests of the configuration module: the paths the recorded oracle cases do not reach, and the Rust-only rows
//! (AC-CFG-001 loading, AC-CFG-003 version check, AC-UTIL-002.4 durations).

use std::io::Write;

use super::*;

fn p(text: &str) -> Config {
    Config::parse_string(text).expect("valid HOCON")
}

fn err_of<T: std::fmt::Debug>(r: Result<T, ConfigError>) -> ConfigError {
    r.expect_err("an error was expected")
}

#[test]
fn parses_the_hocon_value_forms() {
    let c = p(
        "a = 1\nb : 2.5\nc = true\nd = null\ne = \"s\"\nf = [1, 2]\ng { h = x }\nk = 1e3\nm = -4",
    );
    assert_eq!(c.get_int("a").unwrap(), 1);
    assert_eq!(c.get_double("b").unwrap(), 2.5);
    assert!(c.get_boolean("c").unwrap());
    assert!(!c.has_path("d").unwrap());
    assert!(c.has_path_or_null("d").unwrap());
    assert_eq!(c.get_string("e").unwrap(), "s");
    assert_eq!(c.get_int_list("f").unwrap(), vec![1, 2]);
    assert_eq!(c.get_string("g.h").unwrap(), "x");
    assert_eq!(c.get_double("k").unwrap(), 1000.0);
    assert_eq!(c.get_long("m").unwrap(), -4);
    assert_eq!(c.root_keys(), ["a", "b", "c", "d", "e", "f", "g", "k", "m"]);
}

#[test]
fn braced_root_and_commas_and_newline_after_separator() {
    let c = p("{ a = 1, b = 2,\n c =\n 3 }");
    assert!(c.get_int_list("a").is_err());
    assert_eq!(c.get_int("c").unwrap(), 3);
    assert_eq!(c.get_int("b").unwrap(), 2);
}

#[test]
fn escapes_and_triple_quotes() {
    let c = p(
        "a = \"t\\tn\\nq\\\"b\\\\s\\/u\\u0041\\b\\f\\r\"\nb = \"\"\"x \"quoted\" y\"\"\"\nc = \"\"\"a\"\"\"\"",
    );
    assert_eq!(c.get_string("a").unwrap(), "t\tn\nq\"b\\s/uA\u{8}\u{c}\r");
    assert_eq!(c.get_string("b").unwrap(), "x \"quoted\" y");
    assert_eq!(c.get_string("c").unwrap(), "a\"");
}

#[test]
fn parse_errors_name_the_line() {
    for (text, fragment) in [
        ("a = \"open", "Unbalanced quotes"),
        ("a = \"\"\"open", "Unterminated triple-quoted"),
        ("a = \"\\q\"", "not a valid escape"),
        ("a = \"\\u12\"", "Malformed escape"),
        ("[1]", "list at its root"),
        ("a = [1", "List not closed"),
        ("a = {b = 1", "close brace"),
        ("a = 1 }", "Unbalanced close"),
        ("a = $x", "Expecting '{'"),
        ("a = ${x", "Unterminated substitution"),
        ("a = ${}", "empty path element"),
        ("a = ", "Expecting a value"),
        ("a 1", "Expecting a key"),
        (".a = 1", "Empty path element"),
        ("a = ?", "Reserved character"),
        ("= 1", "Empty key or path element"),
        ("a = ]", "Expecting a value"),
        ("a = :", "Expecting a value"),
        ("a = {b = 1} }", "Unbalanced close"),
        ("include \"x\" 5", "Expecting a key"),
    ] {
        let e = err_of(Config::parse_string(text));
        assert_eq!(e.kind(), ConfigErrorKind::Parse, "{text}");
        assert!(e.message().contains(fragment), "{text}: {}", e.message());
        assert!(e.message().starts_with("String: "), "{text}");
    }
}

#[test]
fn path_expressions() {
    let c = p("a.b.\"c.d\" = 1\n\"x y\" = 2\nk . m = 3\n\"\" = 4");
    assert!(c.has_path("a.b.\"c.d\"").unwrap());
    assert_eq!(c.get_int("\"x y\"").unwrap(), 2);
    assert!(c.has_path("\"\"").unwrap());
    for bad in ["", ".a", "a..b", "a.", "\"open"] {
        assert_eq!(
            err_of(c.has_path(bad)).kind(),
            ConfigErrorKind::BadPath,
            "{bad:?}"
        );
    }
    assert_eq!(c.get_value("a.b").unwrap().value_type(), ValueType::Object);
    assert!(!c.has_path("zz.top").unwrap());
}

#[test]
fn substitutions_in_objects_lists_and_with_env() {
    let c = p("a = 1\no { x = ${a} }\nl = [${a}, ${?missing}, 3]\np = ${o}")
        .resolve()
        .unwrap();
    assert_eq!(c.get_int("o.x").unwrap(), 1);
    assert_eq!(c.get_int_list("l").unwrap(), vec![1, 3]);
    assert_eq!(c.get_int("p.x").unwrap(), 1);
    // ${PATH}-like environment fallback, line 0 marks it as coming from the environment
    let c = p("v = ${PATH}").resolve().unwrap();
    assert!(!c.get_string("v").unwrap().is_empty());
}

#[test]
fn self_reference_and_objects_through_substitution() {
    let c = p("a { x = 1 }\na = ${a} { y = 2 }").resolve().unwrap();
    assert_eq!(c.get_int("a.x").unwrap() + c.get_int("a.y").unwrap(), 3);
    let c = p("a { x = 1 }\nb { y = 2 }\na = ${b}").resolve().unwrap();
    assert_eq!(
        (c.has_path("a.x").unwrap(), c.has_path("a.y").unwrap()),
        (true, true)
    );
    let e = err_of(p("a = ${a}").resolve());
    assert_eq!(e.kind(), ConfigErrorKind::UnresolvedSubstitution);
    assert!(
        err_of(p("a { b = ${a} }").resolve())
            .message()
            .contains("cycle")
    );
}

#[test]
fn nested_resolution_through_a_substitution_prefix() {
    let c = p("base { v = 7 }\nx = ${base}\ny = ${x.v}")
        .resolve()
        .unwrap();
    assert_eq!(c.get_int("y").unwrap(), 7);
}

#[test]
fn concatenations() {
    let c = p("a = foo bar\nb = [1] [2]\nc { x = 1 } { y = 2 }\nd = 1 2\ne = true x\nf = null x")
        .resolve()
        .unwrap();
    assert_eq!(c.get_string("a").unwrap(), "foo bar");
    assert_eq!(c.get_int_list("b").unwrap(), vec![1, 2]);
    assert_eq!(c.get_int("c.y").unwrap(), 2);
    assert_eq!(c.get_string("d").unwrap(), "1 2");
    assert_eq!(c.get_string("e").unwrap(), "true x");
    assert_eq!(c.get_string("f").unwrap(), "null x");
    let e = err_of(Config::parse_string("a = [1] x"));
    assert_eq!(e.kind(), ConfigErrorKind::WrongType);
    assert!(
        err_of(p("a = ${b} x\nb = [1]").resolve())
            .message()
            .contains("Cannot concatenate")
    );
}

#[test]
fn includes_find_resources_files_and_report_requireds() {
    let res: Resources<'_> = &[("inc", "i = 1\nj = 2"), ("other.conf", "k = 3")];
    let c = Config::parse_string_with("include \"inc\"\nj = 9\ninclude classpath(\"other\")", res)
        .unwrap();
    assert_eq!(c.get_int("i").unwrap(), 1);
    assert_eq!(c.get_int("j").unwrap(), 9, "what follows the include wins");
    assert_eq!(c.get_int("k").unwrap(), 3);
    let e = err_of(Config::parse_string("include required(\"gone\")"));
    assert_eq!(e.kind(), ConfigErrorKind::Io);
    assert!(e.message().starts_with("gone: gone.conf: java.io.IOException: resource not found on classpath: gone.conf, gone.json"));
    let e = err_of(Config::parse_string(
        "include required(file(\"/no/such/p.conf\"))",
    ));
    assert!(e.message().contains("No such file or directory"), "{e}");
    let dir = std::env::temp_dir().join(format!("rakka-config-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("inc.conf");
    std::fs::File::create(&file)
        .unwrap()
        .write_all(b"from-file = yes")
        .unwrap();
    let c = Config::parse_string(&format!("include file(\"{}\")", file.display())).unwrap();
    assert_eq!(c.get_string("from-file").unwrap(), "yes");
    std::fs::remove_dir_all(&dir).unwrap();
    for bad in [
        "include file(\"x\"",
        "include weird(\"x\")",
        "include required(5)",
    ] {
        assert_eq!(
            err_of(Config::parse_string(bad)).kind(),
            ConfigErrorKind::Parse,
            "{bad}"
        );
    }
    let looping: Resources<'_> = &[("a", "include \"a\"")];
    let e = err_of(Config::parse_string_with("include \"a\"", looping));
    assert!(e.message().contains("more than 50 times"));
    let c = p("include = 5");
    assert_eq!(c.get_int("include").unwrap(), 5, "a key named include");
}

#[test]
fn typed_getters_cover_the_conversions() {
    let c = p(
        "s = \"12\"\nd = 2.9\nbig = 3000000000\nb = yes\nt = 1\nlst = [x]\nobj { a = 1 }\nz = null",
    );
    assert_eq!(c.get_int("s").unwrap(), 12);
    assert_eq!(c.get_int("d").unwrap(), 2);
    assert_eq!(c.get_long("big").unwrap(), 3_000_000_000);
    assert_eq!(c.get_double("s").unwrap(), 12.0);
    assert!(
        c.get_boolean("b").unwrap(),
        "yes unquoted is a string but is read as true"
    );
    assert_eq!(err_of(c.get_int("lst")).kind(), ConfigErrorKind::WrongType);
    assert_eq!(
        err_of(c.get_boolean("t")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(err_of(c.get_int("z")).kind(), ConfigErrorKind::Null);
    assert_eq!(
        err_of(c.get_string("obj")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_string_list("s")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_string_list("lst.zz")).kind(),
        ConfigErrorKind::Missing
    );
    assert_eq!(
        err_of(c.get_config_list("obj")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_int_list("lst")).message(),
        "String: 6: lst has type list of STRING rather than list of NUMBER"
    );
    assert_eq!(err_of(c.get_value("nope")).kind(), ConfigErrorKind::Missing);
    assert_eq!(
        err_of(
            c.get_duration("t", TimeUnit::Seconds)
                .and_then(|_| c.get_duration("lst", TimeUnit::Seconds))
        )
        .kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_bytes("lst")).kind(),
        ConfigErrorKind::WrongType
    );
    let c = p(
        "l = [1, null]\nm = [{a = 1}, 2]\nn { \"1\" = b, \"0\" = a, x = c }\nq = [3000000000]\nr { x = 1 }",
    );
    assert_eq!(
        err_of(c.get_string_list("l")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_config_list("m")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(c.get_string_list("n").unwrap(), ["a", "b"]);
    assert_eq!(
        err_of(c.get_int_list("q")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(
        err_of(c.get_string_list("r")).kind(),
        ConfigErrorKind::WrongType
    );
    assert_eq!(c.get_config_list("m").map(|v| v.len()).unwrap_or(0), 0);
}

#[test]
fn durations_millis_nanos_and_units() {
    let c = p("a = 5 s\nb = 1.5 ms\nc = infinite\nd = -2 minutes\ne = 3 days\nf = \"100 micros\"");
    assert_eq!(
        c.get_millis_duration("a").unwrap().as_nanos(),
        5_000_000_000
    );
    assert_eq!(c.get_nanos_duration("b").unwrap().as_nanos(), 1_500_000);
    assert_eq!(
        c.get_millis_duration("d").unwrap().as_nanos(),
        -120_000_000_000
    );
    assert_eq!(c.get_duration("e", TimeUnit::Hours).unwrap(), 72);
    assert_eq!(c.get_duration("f", TimeUnit::Microseconds).unwrap(), 100);
    assert_eq!(
        err_of(c.get_millis_duration("c")).kind(),
        ConfigErrorKind::BadValue
    );
    let big = p("x = 106751 d");
    assert_eq!(
        big.get_nanos_duration("x").unwrap().as_nanos(),
        9_223_286_400_000_000_000
    );
    let huge = p("x = 9223372036854775807 d");
    assert_eq!(
        err_of(huge.get_millis_duration("x")).kind(),
        ConfigErrorKind::BadValue
    );
}

#[test]
fn sizes() {
    let c = p("a = 1.5 MiB\nb = 2 GB\nc = 3 yottabytes\nd = 1 EB\ne = 9223372036854775807");
    assert_eq!(c.get_bytes("a").unwrap(), 1_572_864);
    assert_eq!(c.get_bytes("b").unwrap(), 2_000_000_000);
    assert_eq!(err_of(c.get_bytes("c")).kind(), ConfigErrorKind::BadValue);
    assert_eq!(c.get_memory_size("d").unwrap(), 1_000_000_000_000_000_000);
    assert_eq!(c.get_bytes("e").unwrap(), i64::MAX);
    assert_eq!(
        err_of(p("x = 9223372036854775807 k").get_bytes("x")).kind(),
        ConfigErrorKind::BadValue
    );
}

#[test]
fn check_valid_edges() {
    let reference = p("pekko { a = 1 }").resolve().unwrap();
    assert!(
        p("pekko { a = 1 }")
            .resolve()
            .unwrap()
            .check_valid(&reference, &["pekko", "nope"])
            .is_ok()
    );
    assert!(
        p("x = 1")
            .resolve()
            .unwrap()
            .check_valid(&reference, &["a..b"])
            .is_err()
    );
    let unresolved_reference = p("pekko { a = ${b} }");
    assert_eq!(
        err_of(reference.check_valid(&unresolved_reference, &[])).kind(),
        ConfigErrorKind::NotResolved
    );
    let with_lists = p("pekko { a = [${b}] }");
    assert_eq!(
        err_of(reference.check_valid(&with_lists, &["pekko"])).kind(),
        ConfigErrorKind::NotResolved
    );
    let nested_unresolved = p("pekko { o { a = ${b} } }");
    assert_eq!(
        err_of(reference.check_valid(&nested_unresolved, &["pekko"])).kind(),
        ConfigErrorKind::NotResolved
    );
    // a null reference accepts anything, and a missing key under a missing object is reported once
    let null_ref = p("pekko { n = null, o { a = 1 } }").resolve().unwrap();
    let e = err_of(
        p("pekko { n = 5 }")
            .resolve()
            .unwrap()
            .check_valid(&null_ref, &["pekko"]),
    );
    assert_eq!(
        e.message(),
        "String: 1: pekko.o: No setting at 'pekko.o', expecting: object with keys [a]"
    );
    // describe covers every reference type
    for (r, c, want) in [
        (
            "pekko { a = true }",
            "pekko { a = 1 }",
            "expecting: boolean but got: number",
        ),
        (
            "pekko { a = [] }",
            "pekko { a = true }",
            "expecting: list but got: boolean",
        ),
    ] {
        let e = err_of(
            p(c).resolve()
                .unwrap()
                .check_valid(&p(r).resolve().unwrap(), &["pekko"]),
        );
        assert!(e.message().contains(want), "{}", e.message());
    }
    // without_path through nested stacks and a missing path
    let w = p("a { b = 1, c = 2 }\na { d = 3 }")
        .without_path("a.b")
        .unwrap();
    assert!(
        !w.has_path("a.b").unwrap() && w.has_path("a.c").unwrap() && w.has_path("a.d").unwrap()
    );
    assert!(w.without_path("zz.y").is_ok());
    assert!(w.without_path("..").is_err());
}

#[test]
fn render_covers_unresolved_values_and_lists() {
    let text = p("a = ${?x} [1]\nb = ${y} z\nc = [{k = 1}, []]").render_with_redactions();
    assert!(text.contains("\"a\" : ${?x} ["));
    assert!(text.contains("\"b\" : ${y} \"z\""));
    assert!(text.contains("\"c\" : ["));
    assert!(text.ends_with("}\n"));
    assert_eq!(Config::empty().render_with_redactions(), "{}\n");
    let v = p("a = [1, \"x\", {b = null}]").get_value("a").unwrap();
    assert_eq!(v.render_concise(), "[1,\"x\",{\"b\":null}]");
}

#[test]
fn with_fallback_keeps_lower_values_alive_for_self_references() {
    let app = p("x += 2");
    let lower = p("x = [1]");
    assert_eq!(
        app.with_fallback(&lower)
            .resolve()
            .unwrap()
            .get_int_list("x")
            .unwrap(),
        vec![1, 2]
    );
    // an unresolved config answers NotResolved only where it is unresolved
    let c = p("a = 1\nb = ${a}\no { c = 2 }");
    assert_eq!(c.get_int("a").unwrap(), 1);
    assert_eq!(c.get_int("o.c").unwrap(), 2);
    assert_eq!(err_of(c.get_int("b")).kind(), ConfigErrorKind::NotResolved);
    let layered = p("o = ${a}").with_fallback(&p("o { z = 1 }"));
    assert_eq!(
        err_of(layered.get_int("o.z")).kind(),
        ConfigErrorKind::NotResolved
    );
    assert!(
        err_of(p("o = ${a}").get_value("o.z"))
            .message()
            .contains("o.z has not been resolved")
    );
}

#[test]
fn default_reference_has_the_pekko_version() {
    let reference = Config::default_reference();
    assert_eq!(reference.get_string("pekko.version").unwrap(), "2.0.0");
    assert_eq!(
        reference
            .get_duration("pekko.log-dead-letters-suspend-duration", TimeUnit::Minutes)
            .unwrap(),
        5
    );
    assert_eq!(
        reference
            .get_bytes("pekko.io.tcp.direct-buffer-size")
            .unwrap(),
        131_072
    );
    assert!(check_pekko_version(&reference).is_ok());
    let other = p("pekko.version = \"1.7.1\"");
    let e = check_pekko_version(&other).unwrap_err();
    assert_eq!(
        e.message(),
        "Pekko JAR version [2.0.0] does not match the provided config version [1.7.1]"
    );
    assert!(check_pekko_version(&Config::empty()).is_err());
    reference.check_valid(&reference, &["pekko"]).unwrap();
}

#[test]
fn load_puts_the_application_file_over_the_reference() {
    let dir = std::env::temp_dir().join(format!("rakka-load-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("application.conf");
    std::fs::write(&file, "pekko.loglevel = DEBUG\nmine = ${pekko.loglevel}").unwrap();
    let c = Config::load(Some(&file)).unwrap();
    assert_eq!(c.get_string("pekko.loglevel").unwrap(), "DEBUG");
    assert_eq!(c.get_string("mine").unwrap(), "DEBUG");
    assert_eq!(c.get_string("pekko.version").unwrap(), "2.0.0");
    assert_eq!(
        err_of(Config::load(Some(&dir.join("missing.conf")))).kind(),
        ConfigErrorKind::Io
    );
    std::fs::write(&file, "a = ").unwrap();
    assert_eq!(
        err_of(Config::load(Some(&file))).kind(),
        ConfigErrorKind::Parse
    );
    std::fs::remove_dir_all(&dir).unwrap();
    // without a path: application.conf of the working directory if there is one, otherwise just the reference
    let c = Config::load(None).unwrap();
    assert!(c.has_path("pekko.actor.provider").unwrap());
}

#[test]
fn leaves_and_numbers() {
    let c = p("a { b = 1, c = [x] }\nd = null");
    let leaves: Vec<String> = c
        .leaves()
        .into_iter()
        .map(|(k, v)| format!("{k}={}", v.render_concise()))
        .collect();
    assert_eq!(leaves, ["a.b=1", "a.c=[\"x\"]", "d=null"]);
    let n = Number::parse("1e2").unwrap();
    assert_eq!((n.text(), n.as_f64()), ("1e2", 100.0));
    assert_eq!(Number::parse("12").unwrap().as_f64(), 12.0);
    // Not numbers for Typesafe Config 1.4.9 (oracle cases `NUMLEX *`): text, not a number. An integer literal must fit a long.
    for text in [
        "",
        "-",
        "-.",
        "1e",
        "1e+",
        "1.2.3",
        "x",
        "1 2",
        "--1",
        "-e5",
        "99999999999999999999",
        "-99999999999999999999",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        assert!(Number::parse(text).is_none(), "{text:?}");
    }
    // Numbers for Typesafe Config: Double.parseDouble accepts a bare trailing point and no integer digits after `-`.
    for text in [
        "1.",
        "1.e5",
        "-.5",
        "9223372036854775807",
        "-9223372036854775808",
        "1e400",
    ] {
        assert!(Number::parse(text).is_some(), "{text:?}");
    }
    assert_eq!(ConfigErrorKind::BadPath.java_name(), "BadPath");
    assert_eq!(
        err_of(p("a = 1").get_int("b")).to_string(),
        "Missing: String: 1: No configuration setting found for key 'b'"
    );
    assert_eq!(ValueType::Boolean.name(), "BOOLEAN");
}

/// The render shows the number's *value*, as Typesafe Config 1.4.9 does (expected strings recorded by the oracle,
/// cases `NUM-RENDER *`): whole doubles that a `long` holds become integers, the rest uses Java's `Double.toString`.
#[test]
fn render_shows_the_value_of_a_number_not_its_text() {
    let expected = [
        ("1.50", "1.5"),
        ("010", "10"),
        ("-0", "0"),
        ("1.0", "1"),
        ("5e18", "5000000000000000000"),
        ("9223372036854775808.0", "9223372036854775807"),
        ("1e19", "1.0E19"),
        ("1e-7", "1.0E-7"),
        ("1.5e-3", "0.0015"),
        ("0.0001", "1.0E-4"),
        ("12345678.9", "1.23456789E7"),
        ("1e400", "Infinity"),
        ("4.9e-324", "4.9E-324"),
        ("0.30000000000000004", "0.30000000000000004"),
    ];
    for (text, rendered) in expected {
        assert_eq!(
            p(&format!("x = {text}"))
                .get_value("x")
                .unwrap()
                .render_concise(),
            rendered,
            "{text}"
        );
    }
}

#[test]
fn a_plus_inside_a_number_like_run_is_a_reserved_character() {
    for text in ["x = 1+2", "x = 1e+"] {
        assert_eq!(
            err_of(Config::parse_string(text)).kind(),
            ConfigErrorKind::Parse,
            "{text}"
        );
    }
    assert_eq!(
        p("x = 1e+5").get_value("x").unwrap().render_concise(),
        "100000"
    );
}
