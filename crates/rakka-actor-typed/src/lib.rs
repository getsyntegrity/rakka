//! Rust port of Apache Pekko 2.0.0 typed actors (`rakka-actor-typed`).
//!
//! Each unit lands here only after its contract is recorded in
//! `docs/inventory/actor-typed.tsv` and validated by its scenarios.

/// The Classic runtime Typed is built on (decision Q-X-8 decides how much of it is reused).
pub use rakka_actor as classic;
