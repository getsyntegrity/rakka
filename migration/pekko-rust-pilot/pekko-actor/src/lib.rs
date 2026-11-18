//! Rust port of the Apache Pekko 2.0.0 actor model (Classic + Typed), built slice by slice.
//!
//! Slice 1 (this crate so far): `ActorPath.fromString` and the `java.net.URI` behaviour it
//! depends on. Strings are modelled as UTF-16 code-unit vectors (`Vec<u16>`) because the
//! reference runs on the JVM and its observable behaviour (lone surrogates, positions,
//! `Character` classes) is defined on UTF-16.

pub mod actor;
pub mod dispatch;
pub mod error;
pub mod event;
pub mod identity;

// Public paths are stable: `pekko_actor::system::ActorSystem`, `pekko_actor::actor_path::ActorPath`, ...
pub use actor::{fault, props, supervision, system};
pub use dispatch::{Failed, Terminated};
pub use identity::{actor_path, actor_ref, jdk_int, jdk_tables, jdk_uri, names};

/// UTF-16 string, the representation of a Java `String`.
pub type JStr = Vec<u16>;

/// Builds a [`JStr`] from a Rust string.
pub fn jstr(s: &str) -> JStr {
    s.encode_utf16().collect()
}
