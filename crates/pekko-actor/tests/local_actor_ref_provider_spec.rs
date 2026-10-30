// Derived from Apache Pekko 1.7.1 actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala (Apache-2.0).
//
// Only the names that reach ActorPath.validatePathElement (:171-197). The
// `null` and `""` cases (:169-170) belong to AC-CELL (dungeon/Children.scala).

use pekko_actor::actor::validate_path_element;

fn message_of(name: &str) -> String {
    let err = validate_path_element(name, None).unwrap_err();
    assert_eq!(err.java_class(), "InvalidActorNameException");
    err.message().to_owned()
}

#[test]
fn throw_suitable_exceptions_for_malformed_actor_names() {
    assert!(message_of("$hallo").contains("not start with `$`"));
    for name in ["a%", "%3", "%xx", "%0G", "%gg", "%", "%1t", "a?"] {
        assert!(
            message_of(name).contains("Invalid actor path element"),
            "{name}"
        );
    }
    assert!(message_of("üß").contains("include only ASCII"));

    assert!(message_of("he\"llo").contains("[\"] at position: 2"));
    assert!(message_of("$hello").contains("[$] at position: 0"));
    assert!(message_of("hell>o").contains("[>] at position: 4"));
}
