// Derived from Apache Pekko 1.7.1 actor-tests/src/test/scala/org/apache/pekko/actor/RelativeActorPathSpec.scala (Apache-2.0).

use rakka_actor::actor::relative_actor_path_elements;

fn elements(path: &str) -> Vec<String> {
    relative_actor_path_elements(path).unwrap_or_default()
}

#[test]
fn match_single_name() {
    assert_eq!(elements("foo"), ["foo"]);
}

#[test]
fn match_path_separated_names() {
    assert_eq!(elements("foo/bar/baz"), ["foo", "bar", "baz"]);
}

#[test]
fn match_url_encoded_name() {
    // URLEncoder.encode("pekko://ClusterSystem@127.0.0.1:7355", "UTF-8")
    let name = "pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355";
    assert_eq!(elements(name), [name]);
}

#[test]
fn match_path_with_uid_fragment() {
    assert_eq!(elements("foo/bar/baz#1234"), ["foo", "bar", "baz#1234"]);
}
