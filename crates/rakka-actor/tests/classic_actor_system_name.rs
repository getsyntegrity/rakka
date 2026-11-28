// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:138-165 (Apache-2.0).
//! The three name tests of `ActorSystemSpec` (AC-SYS-001.1), ported 1:1. The property of "reject all invalid names" generates its
//! strings invalid by construction instead of filtering random ones with the production rule.

use std::time::Duration;

use rakka_actor::actor::ErrorClass;
use rakka_actor::actor::system::ActorSystem;

fn assert_rejected(name: &str) {
    let error = ActorSystem::create(name).err();
    let class = error.map(|e| e.class());
    assert_eq!(class, Some(ErrorClass::IllegalArgument), "{name:?}");
}

#[test]
fn rejects_common_invalid_names() {
    for name in [
        "-hallowelt",
        "_hallowelt",
        "hallo welt",
        "hallo*welt",
        "hallo@welt",
        "hallo#welt",
        "hallo$welt",
        "hallo%welt",
        "hallo/welt",
    ] {
        assert_rejected(name);
    }
}

#[test]
fn rejects_all_invalid_names() {
    // never valid anywhere: every ASCII character outside [a-zA-Z0-9_-], and letters, digits and others that are not ASCII
    let forbidden: Vec<char> = ('\0'..='\x7f')
        .filter(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'))
        .chain(['é', 'ß', '١', 'Ａ', '😀', '\u{2028}'])
        .collect();
    assert_rejected("");
    for word in ["", "a", "Z9", "a-b_c", "0123456789"] {
        assert_rejected(&format!("-{word}"));
        assert_rejected(&format!("_{word}"));
        for character in &forbidden {
            assert_rejected(&format!("{character}{word}"));
            assert_rejected(&format!("a{word}{character}{word}"));
            assert_rejected(&format!("a{word}{character}"));
        }
    }
}

#[test]
fn allows_valid_names() {
    let name = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
    let system = ActorSystem::create(name).expect("every valid character");
    assert_eq!(system.name(), name);
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated")
        .expect("terminated normally");
}
