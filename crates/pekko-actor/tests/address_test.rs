// Derived from Apache Pekko 1.7.1 actor-tests/src/test/java/org/apache/pekko/actor/AddressTest.java (Apache-2.0).

use pekko_actor::actor::Address;

// AddressTest.java:26-30
#[test]
fn port_address_accessible() {
    let address = Address::remote("pekko", "MySystem", "localhost", 2525);
    assert_eq!(Some(2525), address.port());
    assert_eq!(Some("localhost"), address.host());
}
