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

/// A small deterministic generator (xorshift), so that a failure repeats.
struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }

    fn word(&mut self) -> String {
        const VALID: &[u8] = b"abcxyzABCXYZ0189-_";
        (0..self.below(8))
            .map(|_| VALID[self.below(VALID.len())] as char)
            .collect()
    }
}

#[test]
fn rejects_all_invalid_names() {
    // never valid anywhere: neighbours of the accepted ASCII ranges, controls, and letters and digits that are not ASCII
    const FORBIDDEN: [char; 20] = [
        ' ', '*', '@', '#', '$', '%', '/', '.', ':', '[', '`', '{', '\n', '\0', 'é', 'ß', '١',
        'Ａ', '😀', '\u{2028}',
    ];
    let mut random = Random(0x2545_F491_4F6C_DD1D);
    assert_rejected("");
    for _ in 0..200 {
        let leading = ['-', '_'][random.below(2)];
        assert_rejected(&format!("{leading}{}", random.word()));
        let forbidden = FORBIDDEN[random.below(FORBIDDEN.len())];
        let (before, after) = (random.word(), random.word());
        assert_rejected(&format!("{before}{forbidden}{after}"));
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
