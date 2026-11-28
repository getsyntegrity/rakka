// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala (Apache-2.0).
//
// "throw suitable exceptions for malformed actor names" (:168-197), through `system.actorOf` on a real system. The `null`
// name (:169) cannot be expressed with `&str` (Q-PATH-4).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use rakka_actor::actor::ErrorClass;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::ActorSystem;

struct Empty;

impl Actor for Empty {
    fn receive(&mut self, _context: &mut ActorContext, _message: &AnyMessage) -> bool {
        false
    }
}

fn assert_refused(system: &ActorSystem, name: &str, expected: &str) {
    let error = system
        .actor_of_named(Props::new(|| Empty), name)
        .err()
        .unwrap_or_else(|| panic!("{name:?} was accepted"));
    assert_eq!(error.class(), ErrorClass::InvalidActorName, "{name:?}");
    assert!(error.message().contains(expected), "{name:?}: {error:?}");
}

#[test]
fn throw_suitable_exceptions_for_malformed_actor_names() {
    let system = RunningSystem::create("names");

    assert_refused(&system, "", "empty");
    assert_refused(&system, "$hallo", "not start with `$`");
    for name in ["a%", "%3", "%xx", "%0G", "%gg", "%", "%1t", "a?"] {
        assert_refused(&system, name, "Invalid actor path element");
    }
    assert_refused(&system, "üß", "include only ASCII");

    assert_refused(&system, "he\"llo", "[\"] at position: 2");
    assert_refused(&system, "$hello", "[$] at position: 0");
    assert_refused(&system, "hell>o", "[>] at position: 4");
}
