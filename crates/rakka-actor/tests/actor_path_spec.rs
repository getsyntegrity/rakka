// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorPathSpec.scala (Apache-2.0).

use rakka_actor::actor::{ActorPath, Address, validate_path_element};

fn root(a: &Address) -> ActorPath {
    ActorPath::root(a.clone())
}

fn path(a: &Address, names: &[&str]) -> ActorPath {
    names.iter().fold(root(a), |p, n| p.child(n).unwrap())
}

#[test]
fn support_parsing_its_string_rep() {
    let path = root(&Address::local("pekko", "mysys"))
        .child("user")
        .unwrap();
    assert_eq!(ActorPath::from_string(&path.to_string()).unwrap(), path);
}

#[test]
fn support_parsing_remote_paths() {
    let remote = "pekko://my_sys@host:1234/some/ref";
    assert_eq!(ActorPath::from_string(remote).unwrap().to_string(), remote);
}

#[test]
fn support_parsing_remote_akka_paths() {
    let remote = "akka://my_sys@host:1234/some/ref";
    assert_eq!(ActorPath::from_string(remote).unwrap().to_string(), remote);
}

#[test]
fn throw_exception_upon_malformed_paths() {
    for s in ["", "://hallo", "s://dd@:12", "s://dd@h:hd", "a://l:1/b"] {
        let err = ActorPath::from_string(s).unwrap_err();
        assert_eq!(err.java_class(), "MalformedURLException", "{s:?}");
    }
}

#[test]
fn create_correct_to_string() {
    let a = Address::local("pekko", "mysys");
    assert_eq!(root(&a).to_string(), "pekko://mysys/");
    assert_eq!(path(&a, &["user"]).to_string(), "pekko://mysys/user");
    assert_eq!(
        path(&a, &["user", "foo"]).to_string(),
        "pekko://mysys/user/foo"
    );
    assert_eq!(
        path(&a, &["user", "foo", "bar"]).to_string(),
        "pekko://mysys/user/foo/bar"
    );
}

#[test]
fn have_correct_path_elements() {
    let p = path(&Address::local("pekko", "mysys"), &["user", "foo", "bar"]);
    assert_eq!(p.elements(), ["user", "foo", "bar"]);
}

#[test]
fn create_correct_to_string_without_address() {
    let a = Address::local("pekko", "mysys");
    assert_eq!(root(&a).to_string_without_address(), "/");
    assert_eq!(path(&a, &["user"]).to_string_without_address(), "/user");
    assert_eq!(
        path(&a, &["user", "foo"]).to_string_without_address(),
        "/user/foo"
    );
    assert_eq!(
        path(&a, &["user", "foo", "bar"]).to_string_without_address(),
        "/user/foo/bar"
    );
}

#[test]
fn create_correct_to_string_with_address() {
    let local = Address::local("pekko", "mysys");
    let a = Address::new("pekko", "mysys", Some("aaa".into()), Some(7355));
    let b = Address::new("pekko", "mysys", Some("bb".into()), Some(7355));
    let c = Address::new("pekko", "mysys", Some("cccc".into()), Some(7355));
    let r = |names: &[&str]| path(&local, names);
    assert_eq!(r(&[]).to_string_with_address(&a), "pekko://mysys@aaa:7355/");
    assert_eq!(
        r(&["user"]).to_string_with_address(&a),
        "pekko://mysys@aaa:7355/user"
    );
    assert_eq!(
        r(&["user", "foo"]).to_string_with_address(&a),
        "pekko://mysys@aaa:7355/user/foo"
    );

    // root.toStringWithAddress(b) is commented out in the spec.
    assert_eq!(
        r(&["user"]).to_string_with_address(&b),
        "pekko://mysys@bb:7355/user"
    );
    assert_eq!(
        r(&["user", "foo"]).to_string_with_address(&b),
        "pekko://mysys@bb:7355/user/foo"
    );

    assert_eq!(
        r(&[]).to_string_with_address(&c),
        "pekko://mysys@cccc:7355/"
    );
    assert_eq!(
        r(&["user"]).to_string_with_address(&c),
        "pekko://mysys@cccc:7355/user"
    );
    assert_eq!(
        r(&["user", "foo"]).to_string_with_address(&c),
        "pekko://mysys@cccc:7355/user/foo"
    );

    let ra = |names: &[&str]| path(&a, names);
    assert_eq!(
        ra(&[]).to_string_with_address(&b),
        "pekko://mysys@aaa:7355/"
    );
    assert_eq!(
        ra(&["user"]).to_string_with_address(&b),
        "pekko://mysys@aaa:7355/user"
    );
    assert_eq!(
        ra(&["user", "foo"]).to_string_with_address(&b),
        "pekko://mysys@aaa:7355/user/foo"
    );
}

#[test]
fn not_allow_path_separators_in_root_actor_paths_name() {
    let err = ActorPath::root_named(Address::local("pekko", "mysys"), "/user/boom/*").unwrap_err();
    assert_eq!(err.java_class(), "IllegalArgumentException");
    assert!(err.message().contains("is a path separator"));

    ActorPath::from_string("pekko://mysys/user/boom/*").unwrap();
}

#[test]
fn support_parsing_address_from_uri_string() {
    let remote = "pekko://my_sys@host:1234";
    assert_eq!(
        Address::parse(remote).unwrap(),
        Address::new("pekko", "my_sys", Some("host".into()), Some(1234))
    );
}

#[test]
fn support_parsing_akka_address_from_uri_string() {
    let remote = "akka://my_sys@host:1234";
    assert_eq!(
        Address::parse(remote).unwrap(),
        Address::new("akka", "my_sys", Some("host".into()), Some(1234))
    );
}

#[test]
fn detect_valid_and_invalid_chars_in_host_names() {
    let valid = [
        Address::remote("pekko", "sys", "valid", 0),
        Address::remote("pekko", "sys", "is_valid.org", 0),
        Address::remote("pekko", "sys", "fu.is_valid.org", 0),
    ];
    assert!(!valid.iter().any(Address::has_invalid_host_characters));

    let invalid = [
        Address::remote("pekko", "sys", "in_valid", 0),
        Address::remote("pekko", "sys", "invalid._org", 0),
    ];
    assert!(invalid.iter().all(Address::has_invalid_host_characters));

    let err = Address::parse("pekko://sys@in_valid:5001").unwrap_err();
    assert_eq!(err.java_class(), "MalformedURLException");
}

#[test]
fn not_fail_fast_if_the_check_is_called_on_valid_chars_in_host_names() {
    for host in ["localhost", "is_valid.org", "fu.is_valid.org"] {
        Address::remote("pekko", "sys", host, 0)
            .check_host_characters()
            .unwrap();
    }
}

#[test]
fn fail_fast_if_the_check_is_called_when_invalid_chars_are_in_host_names() {
    for host in ["in_valid", "invalid._org"] {
        let err = Address::remote("pekko", "sys", host, 0)
            .check_host_characters()
            .unwrap_err();
        assert_eq!(err.java_class(), "IllegalArgumentException");
    }
}

#[test]
fn validate_path_elements() {
    let err = validate_path_element("", None).unwrap_err();
    assert_eq!(err.java_class(), "InvalidActorNameException");
    assert!(err.message().contains("must not be empty"));
}
