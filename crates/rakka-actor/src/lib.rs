//! Rust port of Apache Pekko 2.0.0 classic actors (`pekko-actor`).
//!
//! Each unit lands here only after its contract is recorded in
//! `docs/inventory/actor-classic.tsv` and validated by its scenarios.

pub mod actor;
pub mod dispatch;
mod done;
pub mod event;
mod not_used;
mod pekko_exception;
pub mod util;

pub use done::Done;
pub use not_used::NotUsed;
pub use pekko_exception::{ConfigurationException, PekkoException};
