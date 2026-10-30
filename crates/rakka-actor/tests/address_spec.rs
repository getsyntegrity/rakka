// Derived from Apache Pekko 1.7.1 actor-tests/src/test/scala/org/apache/pekko/actor/AddressSpec.scala (Apache-2.0).

use rakka_actor::actor::Address;

#[test]
fn work_correctly_with_multiple_mixed_protocols() {
    let pekko_address1 = Address::remote("pekko", "system", "host1", 1000);
    let pekko_address2 = Address::remote("pekko", "system", "host2", 1000);
    let akka_address1 = Address::remote("akka", "system", "host1", 1000);
    let akka_address2 = Address::remote("akka", "system", "host2", 1000);

    let mut addresses = vec![
        pekko_address1.clone(),
        pekko_address2.clone(),
        akka_address1.clone(),
        akka_address2.clone(),
    ];
    addresses.sort_by(Address::pekko_compare);

    assert_eq!(
        addresses,
        vec![akka_address1, akka_address2, pekko_address1, pekko_address2]
    );
}
