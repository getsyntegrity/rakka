// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/PekkoException.scala and actor-tests PekkoExceptionSpec.scala (Apache-2.0).
//! `PekkoException` and `ConfigurationException` (AC-UTIL-004), plan
//! docs/superpowers/plans/2026-10-30-ac-util-core.md.

use std::error::Error;
use std::fmt;

use rakka_actor::{ConfigurationException, PekkoException};

#[derive(Debug)]
struct Inner(&'static str);

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for Inner {}

#[test]
fn pekko_exception_is_built_from_a_single_message() {
    // PekkoExceptionSpec.scala:29-36 only checks that a constructor taking one String exists
    // (serialization friendliness); the oracle cases EX-01/EX-02 fix what it produces.
    let e = PekkoException::new("boom");
    assert_eq!(e.message(), "boom");
    assert!(e.cause().is_none());
    assert!(e.source().is_none());
    assert_eq!(e.to_string(), "boom");
    assert_eq!(e.java_class(), "PekkoException");
    // An owned String is accepted as well as a &str.
    assert_eq!(
        PekkoException::new(String::from("owned")).message(),
        "owned"
    );
    // An empty message stays empty (Java prints "org.apache.pekko.PekkoException: ").
    assert_eq!(PekkoException::new("").message(), "");
}

#[test]
fn pekko_exception_keeps_its_cause() {
    // new PekkoException(msg, cause): getMessage is the message only, getCause the cause (EX-03).
    let e = PekkoException::with_cause("boom", Inner("inner"));
    assert_eq!(e.message(), "boom");
    assert_eq!(e.to_string(), "boom");
    assert_eq!(e.cause().map(|c| c.to_string()).as_deref(), Some("inner"));
    assert_eq!(e.source().map(|c| c.to_string()).as_deref(), Some("inner"));
}

#[test]
fn a_cloned_exception_shares_the_cause() {
    // Pekko passes one exception instance around by reference; the clone sees the same cause.
    let e = PekkoException::with_cause("boom", Inner("inner"));
    let c = e.clone();
    let (a, b) = (e.cause().unwrap(), c.cause().unwrap());
    assert!(std::ptr::eq(
        a as *const _ as *const u8,
        b as *const _ as *const u8
    ));
}

#[test]
fn configuration_exception_has_the_same_constructors_and_its_own_class() {
    // ConfigurationException(msg) and (msg, cause) (PekkoException.scala:41-43); oracle cases EX-04..06.
    let plain = ConfigurationException::new("bad");
    assert_eq!(plain.message(), "bad");
    assert_eq!(plain.to_string(), "bad");
    assert!(plain.cause().is_none());
    assert_eq!(plain.java_class(), "ConfigurationException");

    let caused = ConfigurationException::with_cause("bad", Inner("why"));
    assert_eq!(caused.message(), "bad");
    assert_eq!(
        caused.cause().map(|c| c.to_string()).as_deref(),
        Some("why")
    );
    assert_eq!(
        caused.source().map(|c| c.to_string()).as_deref(),
        Some("why")
    );
    let c = caused.clone();
    assert!(std::ptr::eq(
        caused.cause().unwrap() as *const _ as *const u8,
        c.cause().unwrap() as *const _ as *const u8
    ));
}

#[test]
fn exceptions_can_cross_threads() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<PekkoException>();
    assert_send_sync::<ConfigurationException>();
}
