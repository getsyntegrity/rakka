//! Rust port of Apache Pekko 1.7.1 classic actors (`pekko-actor`).
//!
//! Each unit lands here only after its contract is recorded in
//! `docs/inventory/actor-classic.tsv` and validated by its scenarios.

pub mod actor;
mod done;
mod not_used;
mod pekko_exception;
pub mod util;

pub use done::Done;
pub use not_used::NotUsed;
pub use pekko_exception::{ConfigurationException, PekkoException};
