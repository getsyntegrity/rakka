//! Rust port of Apache Pekko 2.0.0 `pekko-actor-typed`.
//!
//! Slice 4 (not started): `ActorRef<M>`, `Behavior<M>`, `ActorContext<M>`, signals, supervision and
//! adapters, reusing the Classic runtime of `pekko-actor` where that preserves Pekko's behaviour.
//! For now this is only a compilable base.

pub use pekko_actor as classic;
