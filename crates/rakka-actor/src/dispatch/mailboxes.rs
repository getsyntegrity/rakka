// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala (Apache-2.0).
//! `Mailboxes`: the registry that turns a mailbox id into its `MailboxType`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::ConfigurationException;
use crate::dispatch::message_queue::{MailboxType, UnboundedMailbox};

/// The mailbox ids an actor system can resolve (`Mailboxes.lookup`).
#[derive(Clone, Default)]
pub struct Mailboxes {
    registered: BTreeMap<String, Arc<dyn MailboxType>>,
}

impl Mailboxes {
    /// `Mailboxes.DefaultMailboxId`.
    pub const DEFAULT_MAILBOX_ID: &'static str = "pekko.actor.default-mailbox";

    /// `Mailboxes.lookup` resolves this id without configuration (`lookupConfigurator`, `"unbounded"`).
    pub const UNBOUNDED_MAILBOX_ID: &'static str = "unbounded";

    /// The ids every system knows: [`UNBOUNDED_MAILBOX_ID`](Self::UNBOUNDED_MAILBOX_ID) and
    /// [`DEFAULT_MAILBOX_ID`](Self::DEFAULT_MAILBOX_ID), both the unbounded FIFO mailbox.
    pub fn new() -> Mailboxes {
        Mailboxes::default()
    }

    /// Makes `id` resolve to `mailbox_type`; returns the type it replaces. `unbounded` always resolves to `UnboundedMailbox`
    /// (Pekko hardcodes it); the default id does until a type is registered under it, which then applies to explicit selection only.
    pub fn register(
        &mut self,
        id: impl Into<String>,
        mailbox_type: impl MailboxType,
    ) -> Option<Arc<dyn MailboxType>> {
        self.registered.insert(id.into(), Arc::new(mailbox_type))
    }

    /// `Mailboxes.lookup(id)`.
    pub fn lookup(&self, id: &str) -> Result<Arc<dyn MailboxType>, ConfigurationException> {
        if id == Self::UNBOUNDED_MAILBOX_ID
            || (id == Self::DEFAULT_MAILBOX_ID && !self.registered.contains_key(id))
        {
            return Ok(Arc::new(UnboundedMailbox));
        }
        self.registered.get(id).cloned().ok_or_else(|| {
            ConfigurationException::new(format!("Mailbox Type [{id}] not configured"))
        })
    }
}

impl fmt::Debug for Mailboxes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.registered.keys()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::ActorRef;
    use crate::actor::props::{Actor, Props};
    use crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
    use crate::dispatch::message_queue::Envelope;
    use crate::dispatch::test_support::recorder;

    fn bounded(capacity: i32) -> NonBlockingBoundedMailbox {
        NonBlockingBoundedMailbox::new(capacity).expect("a valid capacity")
    }

    /// How many messages the queue of `mailbox_type` takes before it refuses one: tells the registered types apart.
    fn capacity_of(mailbox_type: &dyn MailboxType) -> usize {
        let (_, dead_letters): (_, ActorRef) = recorder("pekko://sys/deadLetters");
        let queue = mailbox_type.create(&dead_letters);
        (0..100)
            .take_while(|number| {
                queue.enqueue(
                    &dead_letters,
                    Envelope {
                        message: Arc::new(*number),
                        sender: None,
                    },
                )
            })
            .count()
    }

    #[test]
    fn a_registered_id_resolves_to_its_type_and_each_id_to_its_own() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        mailboxes.register("three", bounded(3));
        assert_eq!(capacity_of(&*mailboxes.lookup("two").unwrap()), 2);
        assert_eq!(capacity_of(&*mailboxes.lookup("three").unwrap()), 3);
    }

    #[test]
    fn registering_an_id_again_replaces_its_type_and_returns_the_old_one() {
        let mut mailboxes = Mailboxes::new();
        assert!(mailboxes.register("box", bounded(1)).is_none());
        let replaced = mailboxes.register("box", bounded(4)).expect("old type");
        assert_eq!(capacity_of(&*replaced), 1);
        assert_eq!(capacity_of(&*mailboxes.lookup("box").unwrap()), 4);
    }

    #[test]
    fn the_built_in_ids_are_unbounded_and_only_the_default_id_can_be_overridden() {
        let mut mailboxes = Mailboxes::new();
        for id in ["unbounded", "pekko.actor.default-mailbox"] {
            assert_eq!(capacity_of(&*mailboxes.lookup(id).unwrap()), 100, "{id}");
        }
        mailboxes.register(Mailboxes::UNBOUNDED_MAILBOX_ID, bounded(1));
        mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, bounded(3));
        assert_eq!(capacity_of(&*mailboxes.lookup("unbounded").unwrap()), 100);
        assert_eq!(
            capacity_of(&*mailboxes.lookup("pekko.actor.default-mailbox").unwrap()),
            3
        );
    }

    #[test]
    fn an_unregistered_id_is_a_configuration_exception_naming_the_id() {
        let error = Mailboxes::new().lookup("missing").err().expect("error");
        assert_eq!(error.message(), "Mailbox Type [missing] not configured");
    }

    #[test]
    fn debug_lists_the_registered_ids() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        assert_eq!(format!("{mailboxes:?}"), r#"["two"]"#);
    }

    struct Idle;
    impl Actor for Idle {}

    #[test]
    fn props_resolve_their_id_to_its_type_and_leave_other_props_alone() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        let (_, dead_letters): (_, ActorRef) = recorder("pekko://sys/deadLetters");
        let resolved = Props::new(|| Idle)
            .with_mailbox_id("two")
            .resolve_mailbox(&mailboxes)
            .unwrap();
        // the props still name the id they selected, which is what an error or a log reports
        assert_eq!(resolved.mailbox_id(), "two");
        let queue = resolved.new_message_queue(&dead_letters);
        let envelope = || Envelope {
            message: Arc::new(0),
            sender: None,
        };
        assert!(queue.enqueue(&dead_letters, envelope()));
        assert!(queue.enqueue(&dead_letters, envelope()));
        assert!(!queue.enqueue(&dead_letters, envelope()));
        assert!(Props::new(|| Idle).resolve_mailbox(&mailboxes).is_ok());
    }
}
