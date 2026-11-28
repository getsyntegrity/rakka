// Derived from Apache Pekko 2.0.0 actor-tests/src/test/java/org/apache/pekko/util/ConstantFunJavaTest.java (Apache-2.0).
//! AC-UTIL-016.2 A1. rakka has no `ConstantFun`: its identity function is `std::convert::identity` (Q-UTIL-20), so this is a
//! characterisation test, not a RED one. `alwaysReturnFunctionInstance` (A2) has no Rust counterpart and is not ported.

#[test]
fn identity_returns_the_very_value_it_was_given() {
    let value = String::from("value");

    assert!(std::ptr::eq(std::convert::identity(&value), &value));
}
