// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala (Apache-2.0).
//! Unit tests of `ActorPath`: parsing, construction, navigation, ordering and element validation.

use super::*;
use Ordering::*;

/// Oracle cases whose output depends on reference identity
/// (`same_instance`, test-only), checked here instead of through the
/// recorded `path-address-case` replay, which lists them as `UNIT_TESTED`.
/// The fixture holds Pekko 2.0.0's output for them (JDK 17).
#[test]
fn identity_cases_match_pekko_oracle() {
    let golden = include_str!("../../../tests/fixtures/pekko-2.0.0-identity-cases.txt");
    let line = |id: &str| {
        golden
            .lines()
            .find(|l| l.split_once(" -> ").is_some_and(|(i, _)| i == id))
            .unwrap_or_else(|| panic!("{id} not in golden"))
            .to_owned()
    };
    let render = |p: &ActorPath| {
        let elems: Vec<String> = p.elements().iter().map(|e| format!("[{e}]")).collect();
        format!(
            "{} str=[{p}] ser=[{}] noAddr=[{}] addr=[{}] name=[{}] uid={} elems=({})",
            if p.is_root() {
                "RootActorPath"
            } else {
                "ChildActorPath"
            },
            p.to_serialization_format(),
            p.to_string_without_address(),
            p.address(),
            p.name(),
            p.uid(),
            elems.join(",")
        )
    };
    let r = ActorPath::root(mysys());
    let w = r.with_uid(0).unwrap();
    let c3 = r.child("a#3").unwrap();
    let ch = r.child("a").unwrap().child("b#4").unwrap();
    let cases = [
        (
            "UID-01 root.withUid(0)",
            format!("same={} {}", w.same_instance(&r), render(&w)),
        ),
        (
            "UID-06 child#3.withUid(3) same",
            format!("same={}", c3.with_uid(3).unwrap().same_instance(&c3)),
        ),
        (
            "NAV-01 root",
            format!(
                "{} parentIsSelf={} rootIsSelf={}",
                render(&r),
                r.parent().same_instance(&r),
                r.root_path().same_instance(&r)
            ),
        ),
        (
            "NAV-02 child",
            format!(
                "{} parent=[{}] parentUid={} rootIsSame={} root=[{}]",
                render(&ch),
                ch.parent(),
                ch.parent().uid(),
                ch.root_path().same_instance(&r),
                ch.root_path()
            ),
        ),
    ];
    for (id, out) in cases {
        assert_eq!(format!("{id} -> ok {out}"), line(id));
    }
}

fn mysys() -> Address {
    Address::local("pekko", "mysys")
}

fn remote() -> Address {
    Address::remote("pekko", "mysys", "aaa", 7355)
}

/// (str, ser, noAddr, name, uid, elements)
fn shape(p: &ActorPath) -> (String, String, String, String, i32, Vec<String>) {
    (
        p.to_string(),
        p.to_serialization_format(),
        p.to_string_without_address(),
        p.name().to_owned(),
        p.uid(),
        p.elements().into_iter().map(str::to_owned).collect(),
    )
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn from_string_keeps_raw_names_and_reads_the_uid_from_the_fragment() {
    // FS-04..10, 14..16, 36..39, 42, 44, 48..53
    let cases: &[(&str, &str, &str, i32, &[&str])] = &[
        (
            "pekko://sys/user/",
            "pekko://sys/user",
            "pekko://sys/user",
            0,
            &["user"],
        ),
        (
            "pekko://sys/user//x",
            "pekko://sys/user/x",
            "pekko://sys/user/x",
            0,
            &["user", "x"],
        ),
        (
            "pekko://sys/./a/../b",
            "pekko://sys/./a/../b",
            "pekko://sys/./a/../b",
            0,
            &[".", "a", "..", "b"],
        ),
        (
            "pekko://sys/user/a%20b",
            "pekko://sys/user/a%20b",
            "pekko://sys/user/a%20b",
            0,
            &["user", "a%20b"],
        ),
        (
            "pekko://sys/\u{FC}ser",
            "pekko://sys/\u{FC}ser",
            "pekko://sys/\u{FC}ser",
            0,
            &["\u{FC}ser"],
        ),
        (
            "pekko://sys/user?x#5",
            "pekko://sys/user",
            "pekko://sys/user#5",
            5,
            &["user"],
        ),
        (
            "pekko://sys/a#+5",
            "pekko://sys/a",
            "pekko://sys/a#5",
            5,
            &["a"],
        ),
        (
            "pekko://sys/a#-5",
            "pekko://sys/a",
            "pekko://sys/a#-5",
            -5,
            &["a"],
        ),
        (
            "pekko://sys/a#0",
            "pekko://sys/a",
            "pekko://sys/a",
            0,
            &["a"],
        ),
        (
            "pekko://sys/a#-2147483648",
            "pekko://sys/a",
            "pekko://sys/a#-2147483648",
            i32::MIN,
            &["a"],
        ),
        ("pekko://sys#5", "pekko://sys/", "pekko://sys/", 0, &[""]),
        ("pekko://sys/#5", "pekko://sys/", "pekko://sys/#5", 5, &[""]),
        (
            "pekko://sys/user/#5",
            "pekko://sys/user/",
            "pekko://sys/user/#5",
            5,
            &["user", ""],
        ),
        (
            "pekko://sys/a#\u{665}",
            "pekko://sys/a",
            "pekko://sys/a#5",
            5,
            &["a"],
        ),
        (
            "pekko://sys@[::1]:2552/user",
            "pekko://sys@[::1]:2552/user",
            "pekko://sys@[::1]:2552/user",
            0,
            &["user"],
        ),
        (
            "pekko://a%20b@h:1/u",
            "pekko://a b@h:1/u",
            "pekko://a b@h:1/u",
            0,
            &["u"],
        ),
    ];
    for &(input, str_, ser, uid, elems) in cases {
        let p = ActorPath::from_string(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        let (a, b, _, _, u, e) = shape(&p);
        assert_eq!(
            (a.as_str(), b.as_str(), u, e),
            (str_, ser, uid, s(elems)),
            "{input}"
        );
    }
    assert!(ActorPath::from_string("pekko://sys").unwrap().is_root());
}

#[test]
fn from_string_errors() {
    // FS-11..13, 17..19, 21..23, 25, 26, 32..35, 40, 41, 43, 45..47, 54..56
    for input in [
        "pekko://sys/a%2",
        "pekko://sys/a b",
        "pekko://sys@in_valid:5001/user",
        "pekko://my_sys/user",
        "pekko:sys",
        "pekko:///user",
        "pekko://sys:1234/user",
        "pekko://sys@host/user",
        "/user/a",
        "user/a",
        "pekko://sys@h:99999999999/u",
        "pekko://sys/a#5#6",
        "pekko://sys/a# 5",
    ] {
        let e = ActorPath::from_string(input).unwrap_err();
        assert_eq!(
            (e.java_class(), e.message().to_owned()),
            (
                "MalformedURLException",
                format!("cannot parse as ActorPath: {input}")
            )
        );
    }
    for (input, fragment) in [
        ("pekko://sys/user#5?x", "5?x"),
        ("pekko://sys/a#abc", "abc"),
        ("pekko://sys/a#99999999999", "99999999999"),
        ("pekko://sys/a#2147483648", "2147483648"),
        ("pekko://sys/a#", ""),
        ("pekko://sys/a#+", "+"),
        ("pekko://sys/a#-", "-"),
        ("pekko://sys/a#+-5", "+-5"),
    ] {
        let e = ActorPath::from_string(input).unwrap_err();
        assert!(matches!(e, FromStringError::NumberFormat(_)), "{input}");
        assert_eq!(e.message(), format!("For input string: \"{fragment}\""));
    }
}

#[test]
fn child_splits_name_and_uid() {
    // CH-*
    let user = ActorPath::root(mysys()).child("user").unwrap();
    for (name, leaf, uid) in [
        ("foo", "foo", 0),
        ("foo#7", "foo", 7),
        ("foo#+7", "foo", 7),
        ("foo#-7", "foo", -7),
        ("foo#0", "foo", 0),
        ("#5", "", 5),
        ("", "", 0),
        ("foo#\u{665}", "foo", 5),
        ("a b", "a b", 0),
        ("$a", "$a", 0),
        ("..", "..", 0),
        ("a?b", "a?b", 0),
        ("a%2", "a%2", 0),
    ] {
        let c = user.child(name).unwrap();
        assert_eq!((c.name(), c.uid()), (leaf, uid), "{name}");
        assert_eq!(c.to_string(), format!("pekko://mysys/user/{leaf}"));
    }
    for (name, input) in [
        ("foo#abc", "abc"),
        ("foo#", ""),
        ("foo#99999999999", "99999999999"),
        ("foo#1#2", "1#2"),
        ("foo#+", "+"),
    ] {
        let e = user.child(name).unwrap_err();
        assert!(matches!(e, ChildError::NumberFormat(_)));
        assert_eq!(e.message(), format!("For input string: \"{input}\""));
    }
    for name in ["a/b", "/"] {
        let e = user.child(name).unwrap_err();
        assert_eq!(e.java_class(), "IllegalArgumentException");
        assert_eq!(
            e.message(),
            format!("/ is a path separator and is not legal in ActorPath names: [{name}]")
        );
    }
}

#[test]
fn descendant_skips_empty_names() {
    // DS-*
    let r = ActorPath::root(mysys());
    assert_eq!(
        r.descendant(["a", "", "b"]).unwrap().to_string(),
        "pekko://mysys/a/b"
    );
    assert!(r.descendant([]).unwrap().same_instance(&r));
    assert!(r.descendant(["", ""]).unwrap().is_root());
    assert_eq!(
        r.descendant(["a", "b#5"])
            .unwrap()
            .to_serialization_format(),
        "pekko://mysys/a/b#5"
    );
    assert_eq!(
        r.descendant(["a#5", "b"])
            .unwrap()
            .to_serialization_format(),
        "pekko://mysys/a/b"
    );
    assert_eq!(
        r.descendant(["a/b"]).unwrap_err().java_class(),
        "IllegalArgumentException"
    );
    assert_eq!(
        r.descendant(["a#x"]).unwrap_err().message(),
        "For input string: \"x\""
    );
    let d = r.child("a").unwrap().descendant(["c", "d"]).unwrap();
    assert_eq!(d.elements(), ["a", "c", "d"]);
}

#[test]
fn root_named_rejects_inner_slash_and_hash() {
    // RT-*
    for (name, str_) in [
        ("/", "pekko://mysys/"),
        ("", "pekko://mysys"),
        ("user", "pekko://mysysuser"),
        ("/a", "pekko://mysys/a"),
    ] {
        let r = ActorPath::root_named(mysys(), name).unwrap();
        assert_eq!(
            shape(&r),
            (
                str_.into(),
                str_.into(),
                "/".into(),
                name.into(),
                0,
                s(&[""])
            )
        );
    }
    for name in ["/a/", "//", "a/b"] {
        assert_eq!(
            ActorPath::root_named(mysys(), name).unwrap_err().message(),
            format!(
                "requirement failed: / may only exist at the beginning of the root actors name, it is a path separator and is not legal in ActorPath names: [{name}]"
            )
        );
    }
    for name in ["#", "/a#1", "a#b"] {
        assert_eq!(
            ActorPath::root_named(mysys(), name).unwrap_err().message(),
            format!(
                "requirement failed: # is a fragment separator and is not legal in ActorPath names: [{name}]"
            )
        );
    }
}

#[test]
fn with_uid_validates_the_root_and_replaces_or_drops_the_child_uid() {
    // UID-*
    let r = ActorPath::root(mysys());
    assert!(r.with_uid(0).unwrap().same_instance(&r));
    for uid in [5, -1] {
        let e = r.with_uid(uid).unwrap_err();
        assert_eq!(e.java_class(), "IllegalStateException");
        assert_eq!(
            e.message(),
            format!("RootActorPath must have undefinedUid, [{uid} != 0")
        );
    }
    let c3 = r.child("a#3").unwrap();
    assert_eq!(
        c3.with_uid(9).unwrap().to_serialization_format(),
        "pekko://mysys/a#9"
    );
    assert_eq!(
        c3.with_uid(0).unwrap().to_serialization_format(),
        "pekko://mysys/a"
    );
    assert!(c3.with_uid(3).unwrap().same_instance(&c3));
    assert_eq!(c3.with_uid(i32::MIN).unwrap().uid(), i32::MIN);
}

#[test]
fn parent_root_path_name_and_address_navigate_the_tree() {
    // NAV-*
    let r = ActorPath::root(mysys());
    assert!(r.parent().same_instance(&r) && r.root_path().same_instance(&r));
    assert_eq!((r.name(), r.elements()), ("/", vec![""]));
    let c = r.child("a").unwrap().child("b#4").unwrap();
    assert_eq!(
        (c.parent().to_string(), c.parent().uid()),
        ("pekko://mysys/a".into(), 0)
    );
    assert!(c.root_path().same_instance(&r));
    assert_eq!(c.address(), &mysys());
    assert_eq!(
        ActorPath::root(remote()).child("x").unwrap().address(),
        &remote()
    );
}

#[test]
fn text_with_another_address() {
    // STR-*
    let c7 = ActorPath::root(mysys())
        .child("user")
        .unwrap()
        .child("foo#7")
        .unwrap();
    let other = Address::local("pekko", "other");
    assert_eq!(
        c7.to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/user/foo"
    );
    assert_eq!(
        c7.to_serialization_format_with_address(&remote()),
        "pekko://mysys@aaa:7355/user/foo#7"
    );
    assert_eq!(c7.to_string_with_address(&other), "pekko://other/user/foo");
    assert_eq!(
        c7.to_serialization_format_with_address(&other),
        "pekko://other/user/foo#7"
    );
    let bb = Address::remote("pekko", "mysys", "bb", 7355);
    let rc = ActorPath::root(remote()).child("foo#7").unwrap();
    assert_eq!(
        rc.to_serialization_format_with_address(&bb),
        "pekko://mysys@aaa:7355/foo#7"
    );
    assert_eq!(
        ActorPath::root(mysys()).to_serialization_format_with_address(&remote()),
        "pekko://mysys@aaa:7355/"
    );
    let named = ActorPath::root_named(mysys(), "/x").unwrap();
    assert_eq!(
        named.to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/x"
    );
    assert_eq!(
        named.child("y").unwrap().to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/xy"
    );
}

#[test]
fn ignore_ref_path_never_changes_address_and_drops_uid() {
    // STR-11..18
    let local = Address::local("pekko", "local");
    assert_eq!(ignore_ref_path().to_string(), "pekko://local/ignore");
    assert_eq!(
        ignore_ref_path().to_serialization_format_with_address(&remote()),
        "pekko://local/ignore"
    );
    let built = ActorPath::root(local.clone()).child("ignore#5").unwrap();
    assert_eq!(
        built.to_string_with_address(&remote()),
        "pekko://local/ignore"
    );
    assert_eq!(
        built.to_serialization_format_with_address(&remote()),
        "pekko://local/ignore"
    );
    let below = ActorPath::root(local.clone())
        .descendant(["ignore", "x"])
        .unwrap();
    assert_eq!(
        below.to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/ignore/x"
    );
    assert_eq!(
        ActorPath::root(local).to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/"
    );
    let akka = ActorPath::root(Address::local("akka", "local"))
        .child("ignore")
        .unwrap();
    assert_eq!(
        akka.to_string_with_address(&remote()),
        "pekko://mysys@aaa:7355/ignore"
    );
}

fn cmp(a: &ActorPath, b: &ActorPath) -> (Ordering, Ordering, bool, bool) {
    (a.pekko_compare(b), b.pekko_compare(a), a == b, b == a)
}

fn child_of(parent: &ActorPath, name: &str) -> ActorPath {
    parent.child(name).unwrap()
}

#[test]
fn compare_orders_a_parent_after_its_children_and_siblings_by_name() {
    // CMPP-*
    let r0 = ActorPath::root(mysys());
    let c = child_of;
    assert_eq!(
        cmp(&r0, &ActorPath::root(mysys())),
        (Equal, Equal, true, true)
    );
    assert_eq!(cmp(&r0, &c(&r0, "a")), (Greater, Less, false, false));
    assert_eq!(
        cmp(&c(&r0, "a"), &c(&r0, "b")),
        (Less, Greater, false, false)
    );
    assert_eq!(
        cmp(&c(&c(&r0, "a"), "b"), &c(&r0, "b")),
        (Less, Greater, false, false)
    );
    assert_eq!(
        cmp(&c(&c(&r0, "b"), "a"), &c(&c(&r0, "a"), "b")),
        (Less, Greater, false, false)
    );
    assert_eq!(cmp(&c(&r0, ""), &r0), (Less, Greater, false, false));
}

#[test]
fn paths_that_differ_only_in_uid_are_equal_and_hash_alike() {
    use std::hash::{BuildHasher, RandomState};
    let hasher = RandomState::new();
    let r0 = ActorPath::root(mysys());
    let (a1, a2) = (child_of(&r0, "a#1"), child_of(&r0, "a#2"));
    assert_eq!(cmp(&a1, &a2), (Equal, Equal, true, true));
    assert_eq!(hasher.hash_one(&a1), hasher.hash_one(&a2));
}

#[test]
fn compare_looks_at_the_address_before_the_elements() {
    let r0 = ActorPath::root(mysys());
    let c = child_of;
    let other = ActorPath::root(Address::local("pekko", "other"));
    assert_eq!(
        cmp(&c(&r0, "a"), &c(&other, "a")),
        (Less, Greater, false, false)
    );
    assert_eq!(
        cmp(&c(&r0, "a"), &c(&ActorPath::root(remote()), "a")),
        (Less, Greater, false, false)
    );
}

#[test]
fn compare_uses_the_address_fields_not_its_rendered_text() {
    let c = child_of;
    let t1 = ActorPath::root(Address::local("p", "s@h:1"));
    let t2 = ActorPath::root(Address::remote("p", "s", "h", 1));
    assert_eq!(cmp(&t1, &t2), (Equal, Equal, false, false));
    assert_eq!(
        cmp(&c(&t1, "x"), &c(&t2, "x")),
        (Equal, Equal, false, false)
    );
    let n1 = ActorPath::root_named(Address::local("p", "s"), "/a").unwrap();
    let n2 = ActorPath::root_named(Address::local("p", "s/"), "a").unwrap();
    assert_eq!(cmp(&n1, &n2), (Equal, Equal, false, false));
    assert_eq!(
        cmp(&n1, &c(&ActorPath::root(Address::local("p", "s")), "a")),
        (Greater, Less, false, false)
    );
}

#[test]
fn compare_orders_element_names_by_utf16_code_unit() {
    let r0 = ActorPath::root(mysys());
    assert_eq!(
        cmp(&child_of(&r0, "\u{FF61}"), &child_of(&r0, "\u{10000}")),
        (Greater, Less, false, false)
    );
}

#[test]
fn a_parsed_path_equals_the_same_path_built_by_child() {
    use std::hash::{BuildHasher, RandomState};
    let hasher = RandomState::new();
    let r0 = ActorPath::root(mysys());
    let parsed = ActorPath::from_string("pekko://mysys/user/a#9").unwrap();
    let built = child_of(&child_of(&r0, "user"), "a");
    assert_eq!(cmp(&parsed, &built), (Equal, Equal, true, true));
    assert_eq!(hasher.hash_one(&parsed), hasher.hash_one(&built));
}

#[test]
fn ignore_ref_path_equals_the_path_it_stands_for() {
    let ignored = ActorPath::root(Address::local("pekko", "local"))
        .child("ignore#5")
        .unwrap();
    assert_eq!(
        cmp(&ignore_ref_path(), &ignored),
        (Equal, Equal, true, true)
    );
}

const RULES: &str = ". Actor paths MUST: not start with `$`, include only ASCII letters and can only contain these special characters: -_.*$+:@&=,!~';.";

fn invalid(element: &str, full_path: &str, c: &str, at: usize) -> String {
    format!(
        "Invalid actor path element [{element}]{full_path}, illegal character [{c}] at position: {at}{RULES}"
    )
}

#[test]
fn valid_path_elements() {
    // VAL-02,03,06,07,10..12,24..27, VAL-FP-03
    for e in [
        "abc",
        "ABCxyz019",
        "a$b",
        "a$",
        "a%2Fb",
        "%41",
        "A%aF",
        "-_.*$+:@&=,!~';",
        "-",
        ".",
        "..",
    ] {
        assert!(is_valid_path_element(e), "{e}");
        validate_path_element(e, None).unwrap();
    }
    validate_path_element("ok", Some("/x")).unwrap();
}

#[test]
fn empty_element_message_keeps_the_trailing_space() {
    // VAL-01, VAL-FP-01
    let e = validate_path_element("", None).unwrap_err();
    assert_eq!(e.message(), "Actor path element must not be empty ");
    let e = validate_path_element("", Some("/a/b")).unwrap_err();
    assert_eq!(
        e.message(),
        "Actor path element must not be empty  (in path [/a/b])"
    );
    assert!(!is_valid_path_element(""));
}

#[test]
fn invalid_element_names_char_and_position() {
    // VAL-04,05,08,09,13..21,28..32
    for (e, c, at) in [
        ("$abc", "$", 0),
        ("$", "$", 0),
        ("a/b", "/", 1),
        ("a#b", "#", 1),
        ("a%2", "%", 1),
        ("a%", "%", 1),
        ("%", "%", 0),
        ("%g1", "%", 0),
        ("a%%41", "%", 1),
        ("a b", " ", 1),
        (" ", " ", 0),
        ("\t", "\t", 0),
        ("\u{FC}", "\u{FC}", 0),
        ("a\"b", "\"", 1),
        ("a?b", "?", 1),
        ("a\\b", "\\", 1),
        ("a(b", "(", 1),
        ("a[b", "[", 1),
    ] {
        assert!(!is_valid_path_element(e), "{e}");
        let err = validate_path_element(e, None).unwrap_err();
        assert_eq!(err.java_class(), "InvalidActorNameException");
        assert_eq!(err.message(), invalid(e, "", c, at));
    }
}

#[test]
fn full_path_is_quoted_after_the_element() {
    // VAL-FP-02, VAL-FP-04
    let err = validate_path_element("$a", Some("/user/$a")).unwrap_err();
    assert_eq!(
        err.message(),
        invalid("$a", " (in path [/user/$a])", "$", 0)
    );
    let err = validate_path_element("a b", Some("")).unwrap_err();
    assert_eq!(err.message(), invalid("a b", " (in path [])", " ", 1));
}

#[test]
fn position_counts_utf16_code_units() {
    // VAL-22/23: the position part; the illegal char itself is Q-PATH-2.
    assert!(
        validate_path_element("x\u{1F642}", None)
            .unwrap_err()
            .message()
            .contains(" at position: 1.")
    );
}

#[test]
fn lone_surrogate_in_message_matches_pekko() {
    let err = validate_path_element("x\u{1F642}", None).unwrap_err();
    let units = err.message_utf16();
    let expected: Vec<u16> = "illegal character ["
        .encode_utf16()
        .chain([0xD83D, u16::from(b']')])
        .collect();
    assert!(units.windows(expected.len()).any(|w| w == expected));
}

/// Pekko 2.0.0 runs on JDK 17 (Q-X-16): `URI.scanByte` calls `Integer.parseInt(CharSequence, begin, end, 10)`,
/// so an overflowing IPv4 octet escapes `ActorPath.fromString` as a `NumberFormatException`
/// with the JDK 17 message (`Error at index N in: "digits"`), not as a `MalformedURLException`.
#[test]
fn ipv4_octet_overflow_in_ipv6_host_is_number_format_exception_with_jdk17_message() {
    // Both overflow at the tenth digit, so the JDK reports index 9.
    for (input, digits) in [
        ("pekko://sys@[::1.2.3.99999999999]/x", "99999999999"),
        (
            "pekko://sys@[1:2:3:4:5:6:1.2.3.2147483648]:1/x",
            "2147483648",
        ),
    ] {
        let error = ActorPath::from_string(input).unwrap_err();
        assert_eq!(error.java_class(), "NumberFormatException", "{input}");
        assert_eq!(
            error.message(),
            format!("Error at index 9 in: \"{digits}\""),
            "{input}"
        );
    }
}

#[test]
fn relative_actor_path_elements_follow_java_net_uri() {
    // RP-*
    let cases: [(&str, Option<&[&str]>); 15] = [
        ("", Some(&[""])),
        ("/user/a", Some(&["", "user", "a"])),
        ("pekko://sys/a", None),
        ("foo:bar", None),
        ("a%2", None),
        ("a b", None),
        ("foo?x", Some(&["foo"])),
        ("foo#abc", Some(&["foo#abc"])),
        ("#5", Some(&["#5"])),
        ("../a", Some(&["..", "a"])),
        ("foo//bar", Some(&["foo", "", "bar"])),
        ("foo/", Some(&["foo", ""])),
        ("\u{FC}", Some(&["\u{FC}"])),
        ("a%20b", Some(&["a%20b"])),
        ("foo#1#2", None),
    ];
    for (input, expected) in cases {
        let expected = expected.map(|e| e.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(relative_actor_path_elements(input), expected, "{input:?}");
    }
}
