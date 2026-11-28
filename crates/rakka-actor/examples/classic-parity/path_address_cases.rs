// Cases of the AC-PATH oracle (`PathAddressOracle.java`, converted from the deprecated jshell script):
// each rebuilds one oracle case with the Rust API and renders it with the oracle's line format, so
// `path-address-case` prints exactly what Pekko 2.0.0 (JDK 17) printed for the same case id.

use crate::text_escape::escape_non_ascii_units;
use std::hash::{BuildHasher, RandomState};

use rakka_actor::actor::{
    ActorPath, Address, ChildError, FromStringError, IllegalArgumentError, IllegalStateError,
    InvalidActorNameError, MalformedUrlError, is_valid_path_element, relative_actor_path_elements,
    validate_path_element,
};

/// Golden cases that depend on an open Owner question: not produced by Rust on
/// purpose. Each must exist in the golden and must not be produced below.
/// Documented in the tests (`recorded_pekko`), which check that every oracle id is produced or listed here.
#[allow(dead_code)]
pub const PENDING: &[(&str, &str)] = &[
    // Java List.sort (TimSort) with a non-total comparator; how Rust sorts with
    // Address::pekko_compare is undecided.
    (
        "CMPA-17 sort [pekko://a, akka://a@h:2, akka://a@h:1, pekko://a@:0, pekko://a:3]",
        "Q-PATH-1",
    ),
    // Pekko lets the NumberFormatException of an out-of-range embedded IPv4 octet escape from
    // AddressFromURIString.apply; `Address::parse` reports MalformedURLException. Fixing it changes
    // the public signature of `Address::parse`.
    (
        "AD-NFE-01 [pekko://sys@[::1.2.3.99999999999]:1]",
        "Q-PATH-7",
    ),
    (
        "AD-NFE-02 [pekko://sys@[1:2:3:4:5:6:1.2.3.2147483648]:1]",
        "Q-PATH-7",
    ),
];

/// Golden cases that observe reference identity (Java `eq`), which the public
/// Rust API does not expose: checked by the unit test
/// `actor::path::tests::identity_cases_match_pekko_oracle`. Each must exist in
/// the golden and must not be produced below.
#[allow(dead_code)]
pub const UNIT_TESTED: &[&str] = &[
    "UID-01 root.withUid(0)",
    "UID-06 child#3.withUid(3) same",
    "NAV-01 root",
    "NAV-02 child",
];

struct JavaErr(&'static str, String);

macro_rules! java_err {
    ($($t:ty),+) => {$(
        impl From<$t> for JavaErr {
            fn from(e: $t) -> Self {
                JavaErr(e.java_class(), units_to_text(&e.message_utf16()))
            }
        }
    )+};
}
java_err!(
    MalformedUrlError,
    IllegalArgumentError,
    IllegalStateError,
    InvalidActorNameError,
    FromStringError,
    ChildError
);

type R = Result<String, JavaErr>;

/// UTF-16 code units as text, spelling an unpaired surrogate as the `\uXXXX` escape the oracle prints for it
/// (a Rust `String` cannot hold it; `esc` leaves the backslash text unchanged).
fn units_to_text(units: &[u16]) -> String {
    let mut text = String::new();
    for unit in char::decode_utf16(units.iter().copied()) {
        match unit {
            Ok(c) => text.push(c),
            Err(lone) => text.push_str(&format!("\\u{:04X}", lone.unpaired_surrogate())),
        }
    }
    text
}

fn list<S: AsRef<str>>(items: &[S]) -> String {
    let inner: Vec<String> = items.iter().map(|s| format!("[{}]", s.as_ref())).collect();
    format!("({})", inner.join(","))
}

fn opt<T: std::fmt::Display>(o: Option<T>) -> String {
    o.map_or("none".to_owned(), |v| format!("[{v}]"))
}

fn addr(a: &Address) -> String {
    format!(
        "str=[{a}] protocol=[{}] system=[{}] host={} port={} local={} global={} hostPort=[{}]",
        a.protocol(),
        a.system(),
        opt(a.host()),
        opt(a.port()),
        a.has_local_scope(),
        a.has_global_scope(),
        a.host_port()
    )
}

fn path(p: &ActorPath) -> String {
    format!(
        "{} str=[{p}] ser=[{}] noAddr=[{}] addr=[{}] name=[{}] uid={} elems={}",
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
        list(&p.elements())
    )
}

fn sig(o: std::cmp::Ordering) -> i32 {
    o as i32
}

fn hash_eq<T: std::hash::Hash>(a: &T, b: &T) -> bool {
    let s = RandomState::new();
    s.hash_one(a) == s.hash_one(b)
}

fn pair_a(a: &Address, b: &Address) -> String {
    let eq = a == b;
    let mut s = format!(
        "cmp={} cmpRev={} eq={eq}",
        sig(a.pekko_compare(b)),
        sig(b.pekko_compare(a))
    );
    if eq {
        s += &format!(" hashEq={}", hash_eq(a, b));
    }
    s
}

fn pair_p(a: &ActorPath, b: &ActorPath) -> String {
    let eq = a == b;
    let mut s = format!(
        "cmp={} cmpRev={} eq={eq} eqRev={}",
        sig(a.pekko_compare(b)),
        sig(b.pekko_compare(a)),
        b == a
    );
    if eq {
        s += &format!(" hashEq={}", hash_eq(a, b));
    }
    s
}

fn rel(s: &str) -> String {
    match relative_actor_path_elements(s) {
        Some(e) => format!("some {}", list(&e)),
        None => "none".to_owned(),
    }
}

fn val(e: &str, full: Option<&str>) -> R {
    let valid = is_valid_path_element(e);
    Ok(match validate_path_element(e, full) {
        Ok(()) => format!("valid={valid} validate=ok"),
        Err(x) => format!(
            "valid={valid} validate=ERR {} [{}]",
            x.java_class(),
            units_to_text(&x.message_utf16())
        ),
    })
}

fn fs(s: &str) -> Result<ActorPath, FromStringError> {
    ActorPath::from_string(s)
}

fn a(p: &str, s: &str) -> Address {
    Address::local(p, s)
}

fn ar(p: &str, s: &str, h: &str, port: i32) -> Address {
    Address::remote(p, s, h, port)
}

fn copy(base: &Address, host: Option<&str>, port: Option<i32>) -> Address {
    Address::new(base.protocol(), base.system(), host.map(String::from), port)
}

fn root(addr: &Address) -> ActorPath {
    ActorPath::root(addr.clone())
}

/// Pekko's `/` chain, failing like the script would.
fn at(addr: &Address, names: &[&str]) -> Result<ActorPath, ChildError> {
    names.iter().try_fold(root(addr), |p, n| p.child(n))
}

struct Cases(Vec<(String, R)>);

impl Cases {
    fn run(&mut self, id: impl Into<String>, f: impl FnOnce() -> R) {
        self.0.push((id.into(), f()));
    }

    fn val(&mut self, id: impl Into<String>, e: &str, full: Option<&str>) {
        self.run(id, || val(e, full));
    }
}

fn spec_cases(c: &mut Cases, mysys: &Address) {
    c.run("SPEC-APS-27", || {
        let p = at(mysys, &["user"])?;
        let parsed = fs(&p.to_string())?;
        Ok(format!("eq={} {}", parsed == p, path(&parsed)))
    });
    c.run("SPEC-APS-32", || {
        Ok(path(&fs("pekko://my_sys@host:1234/some/ref")?))
    });
    c.run("SPEC-APS-37", || {
        Ok(path(&fs("akka://my_sys@host:1234/some/ref")?))
    });
    for (id, proto) in [("SPEC-APS-42", "pekko"), ("SPEC-APS-47", "akka")] {
        c.run(id, || {
            let parsed = Address::parse(&format!("{proto}://my_sys@host:1234"))?;
            let eq = parsed == ar(proto, "my_sys", "host", 1234);
            Ok(format!("{} eqExpected={eq}", addr(&parsed)))
        });
    }
    for (id, s) in [
        ("SPEC-APS-51", ""),
        ("SPEC-APS-52", "://hallo"),
        ("SPEC-APS-53", "s://dd@:12"),
        ("SPEC-APS-54", "s://dd@h:hd"),
        ("SPEC-APS-55", "a://l:1/b"),
    ] {
        c.run(id, || Ok(path(&fs(s)?)));
    }
    let chains: [&[&str]; 4] = [&[], &["user"], &["user", "foo"], &["user", "foo", "bar"]];
    for (n, names) in (60..).zip(chains) {
        c.run(
            format!("SPEC-APS-{n}"),
            || Ok(at(mysys, names)?.to_string()),
        );
    }
    c.run("SPEC-APS-67", || {
        Ok(list(&at(mysys, &["user", "foo", "bar"])?.elements()))
    });
    for (n, names) in (73..).zip(chains) {
        c.run(format!("SPEC-APS-{n}"), || {
            Ok(at(mysys, names)?.to_string_without_address())
        });
    }
    c.val("SPEC-APS-80", "", None);

    let a_a = copy(mysys, Some("aaa"), Some(7355));
    let a_b = copy(&a_a, Some("bb"), Some(7355));
    let a_c = copy(&a_a, Some("cccc"), Some(7355));
    c.run("SPEC-APS-86", || {
        Ok(format!("{} | {} | {}", addr(&a_a), addr(&a_b), addr(&a_c)))
    });
    let three: [&[&str]; 3] = [&[], &["user"], &["user", "foo"]];
    for (start, base, with) in [
        (90, mysys, &a_a),
        (94, mysys, &a_b),
        (98, mysys, &a_c),
        (103, &a_a, &a_b),
    ] {
        for (n, names) in (start..).zip(three) {
            c.run(format!("SPEC-APS-{n}"), || {
                Ok(at(base, names)?.to_string_with_address(with))
            });
        }
    }
    c.run("SPEC-APS-110", || {
        Ok(path(&ActorPath::root_named(mysys.clone(), "/user/boom/*")?))
    });
    c.run("SPEC-APS-114", || {
        Ok(path(&fs("pekko://mysys/user/boom/*")?))
    });
    for h in [
        "valid",
        "is_valid.org",
        "fu.is_valid.org",
        "in_valid",
        "invalid._org",
    ] {
        c.run(format!("SPEC-APS-119-124[{h}]"), || {
            Ok(format!(
                "invalid={}",
                ar("pekko", "sys", h, 0).has_invalid_host_characters()
            ))
        });
    }
    c.run("SPEC-APS-126", || {
        Ok(addr(&Address::parse("pekko://sys@in_valid:5001")?))
    });
    for h in [
        "localhost",
        "is_valid.org",
        "fu.is_valid.org",
        "in_valid",
        "invalid._org",
    ] {
        c.run(format!("SPEC-APS-133-143[{h}]"), || {
            ar("pekko", "sys", h, 0).check_host_characters()?;
            Ok("checked".to_owned())
        });
    }

    c.run("SPEC-ADS-34", || {
        let mut l = [
            ar("pekko", "system", "host1", 1000),
            ar("pekko", "system", "host2", 1000),
            ar("akka", "system", "host1", 1000),
            ar("akka", "system", "host2", 1000),
        ];
        l.sort_by(Address::pekko_compare);
        let shown: Vec<String> = l.iter().map(Address::to_string).collect();
        Ok(format!("[{}]", shown.join(", ")))
    });

    for (id, s) in [
        ("SPEC-RAP-29", "foo"),
        ("SPEC-RAP-32", "foo/bar/baz"),
        // URLEncoder.encode("pekko://ClusterSystem@127.0.0.1:7355", "UTF-8")
        (
            "SPEC-RAP-35",
            "pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355",
        ),
        ("SPEC-RAP-39", "foo/bar/baz#1234"),
    ] {
        c.run(id, || Ok(rel(s)));
    }

    let larp = [
        "$hallo",
        "a%",
        "%3",
        "%xx",
        "%0G",
        "%gg",
        "%",
        "%1t",
        "a?",
        "\u{fc}\u{df}",
        "he\"llo",
        "$hello",
        "hell>o",
    ];
    for (i, name) in larp.iter().enumerate() {
        c.val(format!("SPEC-LARP-{:02}", i + 1), name, None);
    }

    c.run("SPEC-JAPI-62", || {
        Ok(path(&fs("pekko://Sys@localhost:1234/user/abc")?))
    });
}

fn parser_cases(c: &mut Cases) {
    let fs_in = [
        "pekko://mysys/user",
        "pekko://sys",
        "pekko://sys/",
        "pekko://sys/user/",
        "pekko://sys/user//x",
        "pekko://sys//a",
        "pekko://sys/./a/../b",
        "pekko://sys/$a",
        "pekko://sys/user/a%20b",
        "pekko://sys/a%2Fb",
        "pekko://sys/a%2",
        "pekko://sys/a%zz",
        "pekko://sys/a b",
        "pekko://sys/\u{fc}ser",
        "pekko://sys/user?x=1",
        "pekko://sys/user?x#5",
        "pekko://sys/user#5?x",
        "pekko://sys@in_valid:5001/user",
        "pekko://my_sys/user",
        "pekko://my_sys@host:1/user",
        "pekko:sys",
        "pekko:sys/user",
        "pekko:///user",
        "pekko://@host:1/user",
        "pekko://sys:1234/user",
        "pekko://sys@host/user",
        "pekko://sys@[::1]:2552/user",
        "pekko://sys@127.0.0.1:2552/user",
        "pekko://a%20b@h:1/u",
        "pekko://a:b@h:1/u",
        "PEKKO://Sys@HOST:1/User",
        "/user/a",
        "user/a",
        "pekko://sys@h:70000/u",
        "pekko://sys@h:99999999999/u",
        "pekko://sys/a#5",
        "pekko://sys/a#+5",
        "pekko://sys/a#-5",
        "pekko://sys/a#0",
        "pekko://sys/a#abc",
        "pekko://sys/a#99999999999",
        "pekko://sys/a#2147483647",
        "pekko://sys/a#2147483648",
        "pekko://sys/a#-2147483648",
        "pekko://sys/a#",
        "pekko://sys/a#5#6",
        "pekko://sys/a# 5",
        "pekko://sys#5",
        "pekko://sys/#5",
        "pekko://sys/user/#5",
        "pekko://sys/a/b#7",
        "pekko://sys/a#\u{665}",
        "pekko://sys/a#\u{ff15}",
        "pekko://sys/a#+",
        "pekko://sys/a#-",
        "pekko://sys/a#+-5",
    ];
    for (i, s) in fs_in.iter().enumerate() {
        c.run(format!("FS-{:02} [{s}]", i + 1), || Ok(path(&fs(s)?)));
    }

    let ad_in = [
        "pekko://my_sys@host:1234",
        "pekko://sys",
        "pekko://sys/",
        "pekko://sys/user/a",
        "pekko://sys?x",
        "pekko://sys#5",
        "pekko://sys@h:1/user#5",
        "pekko://sys@in_valid:5001",
        "pekko://sys:1234",
        "pekko://@host:1",
        "pekko://sys@host",
        "pekko://sys@host:",
        "pekko://sys@host:0",
        "pekko://sys@host:00012",
        "pekko://sys@host:-1",
        "pekko://sys@host:70000",
        "pekko://sys@host:2147483647",
        "pekko://sys@host:2147483648",
        "pekko://sys@host:99999999999",
        "pekko://sys@[::1]:2552",
        "pekko://sys@[::1]",
        "pekko://sys@[::1",
        "pekko://sys@127.0.0.1:2552",
        "pekko://sys@256.1.1.1:1",
        "pekko://my_sys",
        "pekko://my_sys@host:1",
        "pekko://a:b@h:1",
        "pekko://a%20b@h:1",
        "pekko://a%2@h:1",
        "pekko://sys@h\u{f6}:1",
        "pekko://s\u{ff}s@h:1",
        "pekko://s\u{ff}s",
        "pekko:sys",
        "pekko:///",
        "pekko://",
        "PEKKO://Sys@HOST:1",
        "pekko://s y",
        "/user",
        "sys@h:1",
        "1pekko://sys",
        "pekko://sys@h-1.example.com:1",
        "pekko://sys@-h:1",
        "pekko://sys@h.:1",
        "pekko://sys@h_1.example.com:1",
        "pekko://sys@host:1234 ",
        "",
        "://hallo",
        "pekko://a%C3%BC@h:1",
        "pekko://a%FF@h:1",
        "pekko://sys@[::FFFF:1.2.3.4]:1",
        "pekko://sys@[fe80::1%25eth0]:1",
        "pekko://sys@[1::2::3]:1",
        "pekko://sys@::1:1",
    ];
    for (i, s) in ad_in.iter().enumerate() {
        c.run(format!("AD-{:02} [{s}]", i + 1), || {
            Ok(addr(&Address::parse(s)?))
        });
    }
    for s in ["pekko://sys@h:1", "pekko://sys:1"] {
        c.run(format!("AD-PARSE [{s}]"), || Ok(addr(&Address::parse(s)?)));
    }

    let utf_in = [
        "%ED%A0%80",
        "%ED%B0%80",
        "%ED%A0%80%ED%B0%80",
        "%ED%A0%41",
        "%ED%A0",
        "%C0%80",
        "%C1%BF",
        "%E0%80%80",
        "%E0%9F%BF",
        "%E0%80",
        "%F0%80%80%80",
        "%F0%8F%BF%BF",
        "%F4%90%80%80",
        "%F5%80%80%80",
        "%E2%82",
        "%E2%82%41",
        "%F0%9F%98",
        "%F0%9F%41",
        "%F0%90%80%41",
        "%80",
        "%BF%80",
        "%C3",
        "%C3%BCx%FF%E2%82%AC",
        "%E2%82%AC%ED%A0%E2%82%AC",
        "%F8%88%80%80%80",
        "%FC%84%80%80%80%80",
        "%FE",
        "%FF",
        "%C2%41",
        "%F0%9F%98%80%80",
    ];
    for (i, e) in utf_in.iter().enumerate() {
        let s = format!("pekko://a{e}b@h:1");
        c.run(format!("UTF-AD-{:02} [{s}]", i + 1), || {
            Ok(addr(&Address::parse(&s)?))
        });
    }
    let utf_fs = [
        "pekko://a%ED%A0%80@h:1/u%ED%A0%80/v%C0%80#5",
        "pekko://a%ED%A0%80/u",
        "pekko://a%E2%82@h%E2:1/u",
        "pekko://a%FF@h:1/u#%FF",
    ];
    for (i, s) in utf_fs.iter().enumerate() {
        c.run(format!("UTF-FS-{:02} [{s}]", i + 1), || Ok(path(&fs(s)?)));
    }

    c.run("DEEP-01 fromString 1e6 levels vs built", || {
        let f = fs(&format!("pekko://s/{}", "a/".repeat(1_000_000)))?;
        let mut b = root(&a("pekko", "s"));
        for _ in 0..1_000_000 {
            b = b.child("a")?;
        }
        let parent = f.parent();
        Ok(format!(
            "elems={} strLen={} eq={} hashEq={} cmp={} vsParent={} eqParent={}",
            f.elements().len(),
            f.to_string().encode_utf16().count(),
            f == b,
            hash_eq(&f, &b),
            sig(f.pekko_compare(&b)),
            sig(f.pekko_compare(&parent)),
            f == parent
        ))
    });

    let rp_in = [
        "",
        "/user/a",
        "pekko://sys/a",
        "foo:bar",
        "a%2",
        "a b",
        "foo?x",
        "foo#abc",
        "#5",
        "../a",
        "foo//bar",
        "foo/",
        "\u{fc}",
        "a%20b",
        "foo#1#2",
    ];
    for (i, s) in rp_in.iter().enumerate() {
        c.run(format!("RP-{:02} [{s}]", i + 1), || Ok(rel(s)));
    }
}

fn address_cases(c: &mut Cases, mysys: &Address) {
    let ct: [(&str, Address); 17] = [
        ("CT-01 new(pekko,sys)", a("pekko", "sys")),
        ("CT-02 new(pekko,sys,h,1234)", ar("pekko", "sys", "h", 1234)),
        ("CT-03 new(pekko,sys,h,0)", ar("pekko", "sys", "h", 0)),
        ("CT-04 new(pekko,sys,h,-5)", ar("pekko", "sys", "h", -5)),
        ("CT-05 new(pekko,sys,h,-1)", ar("pekko", "sys", "h", -1)),
        (
            "CT-06 new(pekko,sys,h,70000)",
            ar("pekko", "sys", "h", 70000),
        ),
        (
            "CT-07 new(pekko,sys,h,MAX)",
            ar("pekko", "sys", "h", i32::MAX),
        ),
        (
            "CT-08 new(pekko,sys,h,MIN)",
            ar("pekko", "sys", "h", i32::MIN),
        ),
        // Java `new Address(p, s, null, 1)`: Option(null) is None.
        (
            "CT-09 new(pekko,sys,null,1)",
            Address::new("pekko", "sys", None, Some(1)),
        ),
        ("CT-10 new(pekko,sys,'',1)", ar("pekko", "sys", "", 1)),
        (
            "CT-11 copy(host=Some(h),port=None)",
            copy(mysys, Some("h"), None),
        ),
        (
            "CT-12 copy(host=Some(''),port=None)",
            copy(mysys, Some(""), None),
        ),
        (
            "CT-13 copy(host=None,port=Some(7))",
            copy(mysys, None, Some(7)),
        ),
        ("CT-14 new('',s)", a("", "s")),
        ("CT-15 new(p,'')", a("p", "")),
        ("CT-16 new(p,'s@h:1')", a("p", "s@h:1")),
        ("CT-17 new(p,s,'h:1',2)", ar("p", "s", "h:1", 2)),
    ];
    for (id, value) in ct {
        c.run(id, || Ok(addr(&value)));
    }
    c.run("CT-18 apply(pekko,sys,h,1) eq new", || {
        Ok(pair_a(
            &ar("pekko", "sys", "h", 1),
            &ar("pekko", "sys", "h", 1),
        ))
    });
    c.run("CT-19 parsed eq constructed", || {
        Ok(pair_a(
            &Address::parse("pekko://sys@h:1")?,
            &ar("pekko", "sys", "h", 1),
        ))
    });
    c.run("CT-20 roundtrip parse(new(p,s,h,-5).toString)", || {
        Ok(addr(&Address::parse(
            &ar("pekko", "sys", "h", -5).to_string(),
        )?))
    });
    c.run(
        "CT-21 roundtrip parse(new(p,s,in_valid,1).toString)",
        || {
            Ok(addr(&Address::parse(
                &ar("pekko", "sys", "in_valid", 1).to_string(),
            )?))
        },
    );

    for h in [
        "_", "a_", "_a", "a_b.c", "a.b_c", "a_b.c_d", "a_.b", "a._", "\u{fc}_x",
    ] {
        c.run(format!("HC [{h}]"), || {
            let x = ar("pekko", "sys", h, 1);
            let inv = format!("invalid={}", x.has_invalid_host_characters());
            x.check_host_characters()?;
            Ok(format!("{inv} checked"))
        });
    }
    c.run("HC host=None", || {
        let x = a("pekko", "sys");
        x.check_host_characters()?;
        Ok(format!(
            "invalid={} checked",
            x.has_invalid_host_characters()
        ))
    });
    c.run("HC host=Some(a_b) port=None", || {
        copy(mysys, Some("a_b"), None).check_host_characters()?;
        Ok("checked".to_owned())
    });

    let h_none_p1 = copy(mysys, None, Some(1));
    let h_empty_p2 = copy(mysys, Some(""), Some(2));
    let h_none_p2 = copy(mysys, None, Some(2));
    let cmpa: [(&str, Address, Address); 16] = [
        (
            "CMPA-01 same fields",
            ar("pekko", "s", "h", 1),
            ar("pekko", "s", "h", 1),
        ),
        (
            "CMPA-02 protocol akka<pekko",
            ar("akka", "s", "h", 1),
            ar("pekko", "s", "h", 1),
        ),
        ("CMPA-03 system", a("pekko", "a"), a("pekko", "b")),
        (
            "CMPA-04 host",
            ar("pekko", "s", "h1", 9),
            ar("pekko", "s", "h2", 1),
        ),
        (
            "CMPA-05 port",
            ar("pekko", "s", "h", 2),
            ar("pekko", "s", "h", 10),
        ),
        (
            "CMPA-06 negative port",
            ar("pekko", "s", "h", -5),
            ar("pekko", "s", "h", 0),
        ),
        (
            "CMPA-07 local vs remote",
            a("pekko", "s"),
            ar("pekko", "s", "h", 1),
        ),
        (
            "CMPA-08 host None vs Some('') (same port None)",
            mysys.clone(),
            copy(mysys, Some(""), None),
        ),
        (
            "CMPA-09 port None vs Some(0) (same host)",
            copy(mysys, Some("h"), None),
            copy(mysys, Some("h"), Some(0)),
        ),
        (
            "CMPA-10 a=(None,1) b=(Some(''),2)",
            h_none_p1.clone(),
            h_empty_p2.clone(),
        ),
        (
            "CMPA-11 b=(Some(''),2) c=(None,2)",
            h_empty_p2,
            h_none_p2.clone(),
        ),
        ("CMPA-12 a=(None,1) c=(None,2)", h_none_p1, h_none_p2),
        (
            "CMPA-13 local(None,None) vs (Some(''),Some(5))",
            mysys.clone(),
            copy(mysys, Some(""), Some(5)),
        ),
        (
            "CMPA-14 UTF-16 order system U+FF61 vs U+10000",
            a("pekko", "\u{ff61}"),
            a("pekko", "\u{10000}"),
        ),
        (
            "CMPA-15 uppercase vs lowercase",
            a("pekko", "Sys"),
            a("pekko", "sys"),
        ),
        (
            "CMPA-16 same toString different fields",
            a("p", "s@h:1"),
            ar("p", "s", "h", 1),
        ),
    ];
    for (id, x, y) in cmpa {
        c.run(id, || Ok(pair_a(&x, &y)));
    }
}

fn path_cases(c: &mut Cases, mysys: &Address) -> Result<(), JavaErr> {
    for n in [
        "/", "", "user", "/a", "/a/", "//", "a/b", "#", "/a#1", "a#b",
    ] {
        c.run(format!("RT [{n}]"), || {
            Ok(path(&ActorPath::root_named(mysys.clone(), n)?))
        });
    }
    c.run("RT new RootActorPath(addr) default name", || {
        Ok(path(&root(mysys)))
    });

    for n in [
        "foo",
        "foo#7",
        "foo#+7",
        "foo#-7",
        "foo#0",
        "foo#abc",
        "foo#",
        "foo#99999999999",
        "foo#1#2",
        "#5",
        "",
        "foo#\u{665}",
        "foo#+",
        "a/b",
        "/",
        "a b",
        "\u{fc}",
        "$a",
        ".",
        "..",
        "a?b",
        "a%2",
    ] {
        c.run(format!("CH [{n}]"), || Ok(path(&at(mysys, &["user", n])?)));
    }
    c.run("CH root/'foo#7'", || Ok(path(&at(mysys, &["foo#7"])?)));
    c.run("CH root/'a/b'", || Ok(path(&at(mysys, &["a/b"])?)));
    c.run("CH java child('x#3')", || {
        Ok(path(&root(mysys).child("x#3")?))
    });

    let ds: [(&str, &[&str]); 7] = [
        ("DS-01 [a,,b]", &["a", "", "b"]),
        ("DS-02 []", &[]),
        ("DS-03 [,]", &["", ""]),
        ("DS-04 [a,b#5]", &["a", "b#5"]),
        ("DS-05 [a#5,b]", &["a#5", "b"]),
        ("DS-06 [a/b]", &["a/b"]),
        ("DS-07 [a#x]", &["a#x"]),
    ];
    for (id, names) in ds {
        c.run(id, || {
            Ok(path(&root(mysys).descendant(names.iter().copied())?))
        });
    }
    c.run("DS-08 child.descendant[c,d]", || {
        Ok(path(&at(mysys, &["a"])?.descendant(["c", "d"])?))
    });

    c.run("UID-02 root.withUid(5)", || {
        Ok(path(&root(mysys).with_uid(5)?))
    });
    c.run("UID-03 root.withUid(-1)", || {
        Ok(path(&root(mysys).with_uid(-1)?))
    });
    c.run("UID-04 child.withUid(9)", || {
        Ok(path(&at(mysys, &["a#3"])?.with_uid(9)?))
    });
    c.run("UID-05 child#3.withUid(0)", || {
        Ok(path(&at(mysys, &["a#3"])?.with_uid(0)?))
    });
    c.run("UID-07 child.withUid(MIN)", || {
        Ok(path(&at(mysys, &["a"])?.with_uid(i32::MIN)?))
    });

    let a_a = copy(mysys, Some("aaa"), Some(7355));
    let a_b = copy(&a_a, Some("bb"), Some(7355));
    c.run("NAV-03 remote child address", || {
        Ok(path(&at(&a_a, &["x"])?))
    });
    c.run("NAV-04 root named elements", || {
        Ok(path(&ActorPath::root_named(mysys.clone(), "/x")?))
    });
    c.run("UID-08 root.withUid(0) value", || {
        let w = root(mysys).with_uid(0)?;
        Ok(format!("equal={} {}", w == root(mysys), path(&w)))
    });
    c.run("UID-09 child#3.withUid(3) value", || {
        let child = at(mysys, &["a#3"])?;
        let w = child.with_uid(3)?;
        Ok(format!("equal={} {}", w == child, path(&w)))
    });
    c.run("NAV-05 parent and root of the root", || {
        let r = root(mysys);
        Ok(format!(
            "parent {} | root {}",
            path(&r.parent()),
            path(&r.root_path())
        ))
    });
    c.run("NAV-06 parent and root of a child", || {
        let child = at(mysys, &["a#7", "b#4"])?;
        Ok(format!(
            "parent {} | grandparent {} | root {}",
            path(&child.parent()),
            path(&child.parent().parent()),
            path(&child.root_path())
        ))
    });

    let c7 = at(mysys, &["user", "foo#7"])?;
    let other = a("pekko", "other");
    let local = a("pekko", "local");
    let named_x = ActorPath::root_named(mysys.clone(), "/x")?;
    // IgnoreActorRef.path (ActorRef.scala:545-546): RootActorPath(Address("pekko", "local")) / "ignore".
    let ign = fs("pekko://local/ignore").map_err(JavaErr::from)?;
    c.run("STR-01 child#7", || Ok(path(&c7)));
    c.run("STR-02 child#7 strWithAddr(remote)", || {
        Ok(c7.to_string_with_address(&a_a))
    });
    c.run("STR-03 child#7 serWithAddr(remote)", || {
        Ok(c7.to_serialization_format_with_address(&a_a))
    });
    c.run("STR-04 child#7 strWithAddr(local other)", || {
        Ok(c7.to_string_with_address(&other))
    });
    c.run("STR-05 child#7 serWithAddr(local other)", || {
        Ok(c7.to_serialization_format_with_address(&other))
    });
    c.run("STR-06 remote child#7 serWithAddr(other remote)", || {
        Ok(at(&a_a, &["foo#7"])?.to_serialization_format_with_address(&a_b))
    });
    c.run("STR-07 root serWithAddr(remote)", || {
        Ok(root(mysys).to_serialization_format_with_address(&a_a))
    });
    c.run("STR-08 root named '/x' strWithAddr(remote)", || {
        Ok(named_x.to_string_with_address(&a_a))
    });
    c.run("STR-09 root named '/x' child strWithAddr(remote)", || {
        Ok(named_x.child("y")?.to_string_with_address(&a_a))
    });
    c.run("STR-10 child#-3 ser", || {
        Ok(at(mysys, &["a#-3"])?.to_serialization_format())
    });
    c.run("STR-11 ignore", || Ok(path(&ign)));
    c.run("STR-12 ignore strWithAddr(remote)", || {
        Ok(ign.to_string_with_address(&a_a))
    });
    c.run("STR-13 ignore serWithAddr(remote)", || {
        Ok(ign.to_serialization_format_with_address(&a_a))
    });
    c.run(
        "STR-14 user-built pekko://local/ignore#5 strWithAddr",
        || Ok(at(&local, &["ignore#5"])?.to_string_with_address(&a_a)),
    );
    c.run(
        "STR-15 user-built pekko://local/ignore#5 serWithAddr",
        || Ok(at(&local, &["ignore#5"])?.to_serialization_format_with_address(&a_a)),
    );
    c.run("STR-16 pekko://local/ignore/x strWithAddr", || {
        Ok(at(&local, &["ignore", "x"])?.to_string_with_address(&a_a))
    });
    c.run("STR-17 root pekko://local/ strWithAddr", || {
        Ok(root(&local).to_string_with_address(&a_a))
    });
    c.run("STR-18 akka://local/ignore strWithAddr", || {
        Ok(at(&a("akka", "local"), &["ignore"])?.to_string_with_address(&a_a))
    });

    let r0 = root(mysys);
    let ch = |names: &[&str]| names.iter().try_fold(r0.clone(), |p, n| p.child(n));
    let ps = |s: &str| a("p", s);
    let cmpp: Vec<(&str, ActorPath, ActorPath)> = vec![
        (
            "CMPP-01 root vs equal root",
            r0.clone(),
            root(&a("pekko", "mysys")),
        ),
        ("CMPP-02 root vs child", r0.clone(), ch(&["a"])?),
        ("CMPP-03 /a vs /b", ch(&["a"])?, ch(&["b"])?),
        ("CMPP-04 /a/b vs /b", ch(&["a", "b"])?, ch(&["b"])?),
        ("CMPP-05 /b/a vs /a/b", ch(&["b", "a"])?, ch(&["a", "b"])?),
        (
            "CMPP-06 /a#1 vs /a#2 (uid ignored)",
            ch(&["a#1"])?,
            ch(&["a#2"])?,
        ),
        (
            "CMPP-07 /a vs /a on other system",
            ch(&["a"])?,
            at(&other, &["a"])?,
        ),
        (
            "CMPP-08 /a local vs /a remote",
            ch(&["a"])?,
            at(&a_a, &["a"])?,
        ),
        (
            "CMPP-09 roots same toString, different Address",
            root(&ps("s@h:1")),
            root(&ar("p", "s", "h", 1)),
        ),
        (
            "CMPP-10 children of roots with same toString",
            at(&ps("s@h:1"), &["x"])?,
            at(&ar("p", "s", "h", 1), &["x"])?,
        ),
        (
            "CMPP-11 root(s,'/a') vs root('s/','a')",
            ActorPath::root_named(ps("s"), "/a")?,
            ActorPath::root_named(ps("s/"), "a")?,
        ),
        (
            "CMPP-12 root(s,'/a') vs child s/a",
            ActorPath::root_named(ps("s"), "/a")?,
            at(&ps("s"), &["a"])?,
        ),
        (
            "CMPP-13 UTF-16 child names U+FF61 vs U+10000",
            ch(&["\u{ff61}"])?,
            ch(&["\u{10000}"])?,
        ),
        (
            "CMPP-14 fromString vs built",
            fs("pekko://mysys/user/a#9")?,
            ch(&["user", "a"])?,
        ),
        (
            "CMPP-15 ignore vs user-built",
            ign.clone(),
            at(&local, &["ignore#5"])?,
        ),
        ("CMPP-16 empty-name child vs root", ch(&[""])?, r0.clone()),
    ];
    for (id, x, y) in cmpp {
        c.run(id, || Ok(pair_p(&x, &y)));
    }
    Ok(())
}

fn validation_cases(c: &mut Cases) {
    let val_in = [
        "",
        "abc",
        "ABCxyz019",
        "$abc",
        "$",
        "a$b",
        "a$",
        "a/b",
        "a#b",
        "a%2Fb",
        "%41",
        "A%aF",
        "a%2",
        "a%",
        "%",
        "%g1",
        "a%%41",
        "a b",
        " ",
        "\t",
        "\u{fc}",
        "\u{1F642}",
        "x\u{1F642}",
        "-_.*$+:@&=,!~';",
        "-",
        ".",
        "..",
        "a\"b",
        "a?b",
        "a\\b",
        "a(b",
        "a[b",
    ];
    for (i, e) in val_in.iter().enumerate() {
        c.val(format!("VAL-{:02} [{e}]", i + 1), e, None);
    }
    c.val("VAL-FP-01 ['' in /a/b]", "", Some("/a/b"));
    c.val("VAL-FP-02 [$a in /user/$a]", "$a", Some("/user/$a"));
    c.val("VAL-FP-03 [ok in /x]", "ok", Some("/x"));
    c.val("VAL-FP-04 [a b in '']", "a b", Some(""));
    // ActorPaths (Java facade) delegates to the same functions (Q-PATH-5 is
    // about the facade's existence, not its behavior).
    c.run("VAL-JAPI-01 ActorPaths.validatePathElement($x)", || {
        validate_path_element("$x", None)?;
        Ok("ok".to_owned())
    });
    c.run("VAL-JAPI-02 ActorPaths.validatePathElement(x, /p)", || {
        validate_path_element("x y", Some("/p"))?;
        Ok("ok".to_owned())
    });
    c.run("VAL-JAPI-03 ActorPaths.isValidPathElement(a)", || {
        Ok(format!("valid={}", is_valid_path_element("a")))
    });
}

pub fn rust_lines() -> Vec<(String, String)> {
    let mysys = a("pekko", "mysys");
    let mut c = Cases(Vec::new());
    spec_cases(&mut c, &mysys);
    parser_cases(&mut c);
    address_cases(&mut c, &mysys);
    if let Err(JavaErr(class, msg)) = path_cases(&mut c, &mysys) {
        panic!("building path cases failed: {class} [{msg}]");
    }
    validation_cases(&mut c);
    c.0.into_iter()
        .map(|(id, r)| {
            let outcome = match r {
                Ok(s) => format!("ok {s}"),
                Err(JavaErr(class, msg)) => format!("ERR {class} [{msg}]"),
            };
            let line = escape_non_ascii_units(&format!("{id} -> {outcome}"));
            (escape_non_ascii_units(&id), line)
        })
        .collect()
}
