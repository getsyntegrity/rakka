//! Dispatch: the thread-pool dispatcher and the internal system messages.

pub(crate) mod dispatcher;
pub mod mailbox_status;
pub(crate) mod sysmsg;
pub use sysmsg::{Failed, Terminated};
