// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! Q-SUP-7: the public API of `Throwable` as a user of the crate sees it. Its fields are private: it is built by `new` or
//! `illegal_state` and the `with_` methods, and read by the accessors.

use rakka_actor::actor::fault::Throwable;

#[test]
fn the_constructors_and_accessors_cover_class_message_and_declared_superclasses() {
    let plain = Throwable::new("com.example.Failure", Some("boom"));
    assert_eq!(plain.class(), "com.example.Failure");
    assert_eq!(plain.message(), Some("boom"));
    assert!(plain.cause().is_none());
    assert!(plain.superclasses().is_empty());
    assert_eq!(
        Throwable::new(String::from("a.Silent"), None).message(),
        None
    );

    let custom = Throwable::new("com.example.Child", None)
        .with_superclasses(["com.example.Parent", "java.lang.Error"]);
    assert_eq!(
        custom.superclasses(),
        ["com.example.Parent", "java.lang.Error"]
    );
    assert!(custom.is_a("com.example.Parent") && custom.is_a("java.lang.Throwable"));
    assert!(!custom.is_a("java.lang.Exception"));
}

#[test]
fn a_cause_chain_is_read_link_by_link_and_survives_a_clone() {
    let root = Throwable::new("a.Root", Some("root")).with_superclasses(["java.lang.Error"]);
    let outer = Throwable::illegal_state("outer")
        .with_cause(Throwable::new("a.Middle", None).with_cause(root));
    let copy = outer.clone();
    drop(outer);

    let links: Vec<&Throwable> = std::iter::successors(Some(&copy), |link| link.cause()).collect();
    let classes: Vec<&str> = links.iter().map(|link| link.class()).collect();
    assert_eq!(
        classes,
        ["java.lang.IllegalStateException", "a.Middle", "a.Root"]
    );
    assert_eq!(links[2].message(), Some("root"));
    assert_eq!(links[2].superclasses(), ["java.lang.Error"]);
}
