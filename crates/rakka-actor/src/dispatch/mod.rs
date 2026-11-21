//! Dispatch: the thread-pool dispatcher and the internal system messages.

pub(crate) mod dispatcher;
pub(crate) mod mailbox_close;
pub mod mailbox_status;
pub mod message_queue;
pub(crate) mod sysmsg;
pub mod system_queue;
pub use sysmsg::{Failed, Terminated};
