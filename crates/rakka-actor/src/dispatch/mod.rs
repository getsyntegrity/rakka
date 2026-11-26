//! Dispatch: the thread-pool dispatcher and the internal system messages.

pub mod bounded_node_queue;
pub(crate) mod dead_letter_mailbox;
pub(crate) mod dispatcher;
#[cfg(test)]
mod executor_rejection_tests;
pub(crate) mod mailbox_close;
pub mod mailbox_status;
pub mod mailboxes;
pub mod message_queue;
pub mod queue_semantics;
pub mod single_consumer_mailbox;
pub(crate) mod sysmsg;
pub mod system_queue;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod turn;
pub use sysmsg::{Failed, Terminated};
