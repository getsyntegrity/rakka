// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/Done.scala and NotUsed.scala (Apache-2.0).
//! `Done` and `NotUsed` (AC-UTIL-003), plan docs/superpowers/plans/2026-10-30-ac-util-core.md.

use std::collections::HashSet;

use rakka_actor::{Done, NotUsed};

#[test]
fn done_prints_like_its_case_object() {
    // `case object Done` (Done.scala:27): toString is "Done"; the oracle case DN-01 fixes it.
    assert_eq!(Done.to_string(), "Done");
    assert_eq!(format!("{Done:?}"), "Done");
}

#[test]
fn done_is_a_single_value() {
    let copy = Done;
    assert_eq!(copy, Done);
    let set: HashSet<Done> = [Done, copy].into_iter().collect();
    assert_eq!(set.len(), 1);
}

#[test]
fn not_used_prints_like_its_case_object() {
    // `case object NotUsed` (NotUsed.scala:24): toString is "NotUsed"; the oracle case DN-02 fixes it.
    assert_eq!(NotUsed.to_string(), "NotUsed");
    assert_eq!(format!("{NotUsed:?}"), "NotUsed");
}

#[test]
fn not_used_is_a_single_value() {
    let copy = NotUsed;
    assert_eq!(copy, NotUsed);
    let set: HashSet<NotUsed> = [NotUsed, copy].into_iter().collect();
    assert_eq!(set.len(), 1);
}
