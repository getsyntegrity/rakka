// A 1,000,000-level ActorPath must be built, rendered, compared, hashed and
// dropped without recursion, on the default test thread stack. Pekko 1.7.1
// handles it (oracle case DEEP-01 in tools/oracle/path-address.jsh).

use std::cmp::Ordering;
use std::hash::{BuildHasher, RandomState};

use rakka_actor::actor::{ActorPath, Address};

const DEPTH: usize = 1_000_000;

fn root() -> ActorPath {
    ActorPath::root(Address::local("pekko", "s"))
}

fn exercise(a: ActorPath, b: ActorPath) {
    let text = a.to_string();
    assert_eq!(text.len(), "pekko://s/".len() + 2 * DEPTH - 1);
    assert_eq!(b.to_string(), text);
    assert_eq!(a.elements().len(), DEPTH);
    assert!(a == b);
    let hasher = RandomState::new();
    assert_eq!(hasher.hash_one(&a), hasher.hash_one(&b));
    assert_eq!(a.pekko_compare(&b), Ordering::Equal);
    let parent = a.parent();
    assert!(a != parent);
    assert_eq!(a.pekko_compare(&parent), Ordering::Less);
    assert_eq!(a.root_path(), root());
    drop(parent);
    drop(a);
    drop(b);
}

#[test]
fn deep_path_from_string_and_child_chain() {
    let from_string = ActorPath::from_string(&format!("pekko://s/{}", "a/".repeat(DEPTH))).unwrap();
    let mut built = root();
    for _ in 0..DEPTH {
        built = built.child("a").unwrap();
    }
    exercise(from_string, built);
}

#[test]
fn deep_path_from_descendant() {
    let names = vec!["a"; DEPTH];
    let descendant = root().descendant(names.iter().copied()).unwrap();
    let from_string = ActorPath::from_string(&format!("pekko://s/{}", "a/".repeat(DEPTH))).unwrap();
    exercise(descendant, from_string);
}

#[test]
fn deep_paths_sharing_a_prefix_drop_independently() {
    let mut base = root();
    for _ in 0..DEPTH {
        base = base.child("a").unwrap();
    }
    let left = base.child("l").unwrap();
    let right = base.child("r").unwrap();
    drop(base);
    drop(left);
    assert_eq!(right.elements().len(), DEPTH + 1);
    drop(right);
}
