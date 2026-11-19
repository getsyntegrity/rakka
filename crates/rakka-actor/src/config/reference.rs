// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! The `reference.conf` files of Pekko 2.0.0, embedded.

/// `pekko-actor` 2.0.0 `reference.conf` (identical to `upstream/pekko/actor/src/main/resources/reference.conf`).
pub const ACTOR_REFERENCE_CONF: &str = include_str!("reference/actor.conf");

/// `pekko-actor-typed` 2.0.0 `reference.conf`; it is not used until `rakka-actor-typed` takes it over.
#[doc(hidden)]
pub const TYPED_REFERENCE_CONF: &str = include_str!("reference/actor-typed.conf");

/// What the generated `version.conf` of Pekko holds, with the version rakka reports (Q-CFG-5 (a): 2.0.0).
pub(crate) const VERSION_CONF: &str = "pekko.version = \"2.0.0\"\n";
