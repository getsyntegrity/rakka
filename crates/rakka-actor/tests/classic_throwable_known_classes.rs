//! Q-SUP-8: the hierarchy of a class the runtime knows is its own; what a user declares over it is ignored. Public API only.

use rakka_actor::actor::fault::Throwable;

const POST_RESTART: &str = "org.apache.pekko.actor.PostRestartException";
const INITIALIZATION: &str = "org.apache.pekko.actor.ActorInitializationException";

#[test]
fn a_known_class_redeclared_keeps_its_hierarchy() {
    let redeclared = Throwable::new(POST_RESTART, None).with_superclasses(["java.lang.Error"]);
    assert!(redeclared.is_a(INITIALIZATION) && redeclared.is_a("java.lang.Exception"));
    assert!(!redeclared.is_a("java.lang.Error"));

    let error = Throwable::new("java.lang.Error", None).with_superclasses(["java.lang.Exception"]);
    assert!(!error.is_a("java.lang.Exception"));

    let runtime =
        Throwable::new("java.lang.RuntimeException", None).with_superclasses(["java.lang.Error"]);
    assert!(runtime.is_a("java.lang.Exception") && !runtime.is_a("java.lang.Error"));

    let reflective = Throwable::new("java.lang.ReflectiveOperationException", None)
        .with_superclasses(["java.lang.Error"]);
    assert!(reflective.is_a("java.lang.Exception") && !reflective.is_a("java.lang.Error"));
}

/// The rule holds wherever a known class stands in a declared chain: from it on, the runtime's own superclasses apply.
#[test]
fn a_known_superclass_of_a_user_class_keeps_its_hierarchy() {
    let subclass =
        Throwable::new("a.Sub", None).with_superclasses([POST_RESTART, "java.lang.Error"]);
    assert!(subclass.is_a(POST_RESTART) && subclass.is_a(INITIALIZATION));
    assert!(!subclass.is_a("java.lang.Error"));
}

#[test]
fn a_class_the_runtime_does_not_know_is_the_users_to_declare() {
    let own = Throwable::new("a.Own", None).with_superclasses(["a.Base", "java.lang.Error"]);
    assert!(own.is_a("a.Base") && own.is_a("java.lang.Error"));
    assert!(!own.is_a("java.lang.Exception"));
}
