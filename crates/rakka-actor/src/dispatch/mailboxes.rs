// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala (Apache-2.0).
//! `Mailboxes`: the registry that turns a mailbox id into its `MailboxType`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::ConfigurationException;
use crate::dispatch::message_queue::{MailboxType, UnboundedMailbox};
use crate::dispatch::queue_semantics::MessageQueueSemantics;

/// The mailbox ids an actor system can resolve (`Mailboxes.lookup`).
#[derive(Clone)]
pub struct Mailboxes {
    registered: BTreeMap<String, Arc<dyn MailboxType>>,
    requirements: BTreeMap<MessageQueueSemantics, String>,
}

impl Mailboxes {
    /// `Mailboxes.DefaultMailboxId`.
    pub const DEFAULT_MAILBOX_ID: &'static str = "pekko.actor.default-mailbox";

    /// `Mailboxes.lookup` resolves this id without configuration (`lookupConfigurator`, `"unbounded"`).
    pub const UNBOUNDED_MAILBOX_ID: &'static str = "unbounded";

    /// `pekko.actor.mailbox.unbounded-queue-based`: the mailbox that `reference.conf` binds to `UnboundedMessageQueueSemantics`.
    pub const UNBOUNDED_QUEUE_BASED_ID: &'static str = "pekko.actor.mailbox.unbounded-queue-based";

    /// The ids every system knows: [`UNBOUNDED_MAILBOX_ID`](Self::UNBOUNDED_MAILBOX_ID) and
    /// [`DEFAULT_MAILBOX_ID`](Self::DEFAULT_MAILBOX_ID), both the unbounded FIFO mailbox.
    pub fn new() -> Mailboxes {
        Mailboxes::default()
    }

    /// Makes `id` resolve to `mailbox_type`; returns the type it replaces. `unbounded` always resolves to `UnboundedMailbox`
    /// (Pekko hardcodes it); the default id does until a type is registered under it, which then also serves every actor created
    /// without a mailbox selection (`getMailboxType` ends in `lookup(DefaultMailboxId)`).
    pub fn register(
        &mut self,
        id: impl Into<String>,
        mailbox_type: impl MailboxType,
    ) -> Option<Arc<dyn MailboxType>> {
        self.registered.insert(id.into(), Arc::new(mailbox_type))
    }

    /// `pekko.actor.mailbox.requirements`: the mailbox id that serves the actors requiring `semantics`; returns the id it replaces.
    pub fn bind_requirement(
        &mut self,
        semantics: MessageQueueSemantics,
        mailbox_id: impl Into<String>,
    ) -> Option<String> {
        self.requirements.insert(semantics, mailbox_id.into())
    }

    /// `Mailboxes.lookupByQueueType`: the mailbox type bound to `semantics`.
    pub fn lookup_by_queue_type(
        &self,
        semantics: MessageQueueSemantics,
    ) -> Result<Arc<dyn MailboxType>, ConfigurationException> {
        match self.requirements.get(&semantics) {
            Some(mailbox_id) => self.lookup(mailbox_id),
            None => Err(ConfigurationException::new(format!(
                "Mailbox Mapping for [{semantics}] not configured"
            ))),
        }
    }

    /// `Mailboxes.lookup(id)`.
    pub fn lookup(&self, id: &str) -> Result<Arc<dyn MailboxType>, ConfigurationException> {
        let overridable = id == Self::DEFAULT_MAILBOX_ID || id == Self::UNBOUNDED_QUEUE_BASED_ID;
        if id == Self::UNBOUNDED_MAILBOX_ID || (overridable && !self.registered.contains_key(id)) {
            return Ok(Arc::new(UnboundedMailbox));
        }
        self.registered.get(id).cloned().ok_or_else(|| {
            ConfigurationException::new(format!("Mailbox Type [{id}] not configured"))
        })
    }
}

impl Default for Mailboxes {
    fn default() -> Mailboxes {
        Mailboxes {
            registered: BTreeMap::new(),
            requirements: BTreeMap::from([(
                MessageQueueSemantics::UNBOUNDED,
                Self::UNBOUNDED_QUEUE_BASED_ID.to_owned(),
            )]),
        }
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
    use crate::dispatch::queue_semantics::MessageQueueSemantics;
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
            .resolve_mailbox(&mailboxes, None)
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
        assert!(
            Props::new(|| Idle)
                .resolve_mailbox(&mailboxes, None)
                .is_ok()
        );
    }

    struct RequiresUnbounded;
    impl Actor for RequiresUnbounded {
        fn required_message_queue() -> Option<MessageQueueSemantics> {
            Some(MessageQueueSemantics::UNBOUNDED)
        }
    }

    const AUDIT_TRAIL: MessageQueueSemantics =
        MessageQueueSemantics::new("app.AuditTrailSemantics");

    struct RequiresAuditTrail;
    impl Actor for RequiresAuditTrail {
        fn required_message_queue() -> Option<MessageQueueSemantics> {
            Some(AUDIT_TRAIL)
        }
    }

    fn capacity_after_resolving(
        props: Props,
        mailboxes: &Mailboxes,
    ) -> Result<usize, ConfigurationException> {
        capacity_under_dispatcher(props, mailboxes, None)
    }

    fn capacity_under_dispatcher(
        props: Props,
        mailboxes: &Mailboxes,
        dispatcher_requirement: Option<MessageQueueSemantics>,
    ) -> Result<usize, ConfigurationException> {
        let (_, dead_letters): (_, ActorRef) = recorder("pekko://sys/deadLetters");
        let resolved = props.resolve_mailbox(mailboxes, dispatcher_requirement)?;
        let queue = resolved.new_message_queue(&dead_letters);
        Ok((0..100)
            .take_while(|number| {
                queue.enqueue(
                    &dead_letters,
                    Envelope {
                        message: Arc::new(*number),
                        sender: None,
                    },
                )
            })
            .count())
    }

    #[test]
    fn an_actor_requiring_a_queue_gets_the_mailbox_bound_to_it_and_not_the_registered_default() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, bounded(3));
        assert_eq!(
            capacity_after_resolving(Props::new(|| RequiresUnbounded), &mailboxes).unwrap(),
            100
        );
        assert_eq!(
            capacity_after_resolving(Props::new(|| Idle), &mailboxes).unwrap(),
            3
        );
    }

    #[test]
    fn rebinding_a_requirement_changes_the_mailbox_of_the_actors_that_require_it_and_of_no_others()
    {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        let replaced = mailboxes.bind_requirement(MessageQueueSemantics::UNBOUNDED, "two");
        assert_eq!(
            replaced.as_deref(),
            Some(Mailboxes::UNBOUNDED_QUEUE_BASED_ID)
        );
        assert_eq!(
            capacity_after_resolving(Props::new(|| RequiresUnbounded), &mailboxes).unwrap(),
            2
        );
        assert_eq!(
            capacity_after_resolving(Props::new(|| Idle), &mailboxes).unwrap(),
            100
        );
    }

    #[test]
    fn an_application_defined_semantics_resolves_through_its_own_binding() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        assert!(mailboxes.bind_requirement(AUDIT_TRAIL, "two").is_none());
        assert_eq!(
            capacity_after_resolving(Props::new(|| RequiresAuditTrail), &mailboxes).unwrap(),
            2
        );
    }

    #[test]
    fn a_requirement_without_a_binding_is_a_configuration_exception_naming_the_semantics() {
        let error = capacity_after_resolving(Props::new(|| RequiresAuditTrail), &Mailboxes::new())
            .unwrap_err();
        assert_eq!(
            error.message(),
            "Mailbox Mapping for [interface app.AuditTrailSemantics] not configured"
        );
    }

    #[test]
    fn a_binding_to_an_unregistered_mailbox_fails_like_any_unknown_id() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.bind_requirement(AUDIT_TRAIL, "missing");
        let error =
            capacity_after_resolving(Props::new(|| RequiresAuditTrail), &mailboxes).unwrap_err();
        assert_eq!(error.message(), "Mailbox Type [missing] not configured");
    }

    #[test]
    fn a_selected_id_decides_the_mailbox_over_the_requirement() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("three", bounded(3));
        let selected = Props::new(|| RequiresAuditTrail).with_mailbox_id("three");
        assert_eq!(capacity_after_resolving(selected, &mailboxes).unwrap(), 3);
    }

    #[test]
    fn the_unbounded_queue_based_id_is_the_unbounded_mailbox_until_a_type_is_registered_under_it() {
        let mut mailboxes = Mailboxes::new();
        let id = Mailboxes::UNBOUNDED_QUEUE_BASED_ID;
        assert_eq!(capacity_of(&*mailboxes.lookup(id).unwrap()), 100);
        mailboxes.register(id, bounded(5));
        assert_eq!(capacity_of(&*mailboxes.lookup(id).unwrap()), 5);
    }

    struct RequiresAnyQueue;
    impl Actor for RequiresAnyQueue {
        fn required_message_queue() -> Option<MessageQueueSemantics> {
            Some(MessageQueueSemantics::MESSAGE_QUEUE)
        }
    }

    /// Mailboxes of capacity 2 and 3 under the ids `two` and `three`, with the unbounded semantics bound to `two`.
    fn mailboxes_with_the_unbounded_semantics_bound_to_capacity_two() -> Mailboxes {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register("two", bounded(2));
        mailboxes.register("three", bounded(3));
        mailboxes.bind_requirement(MessageQueueSemantics::UNBOUNDED, "two");
        mailboxes
    }

    const DISPATCHER_REQUIRES_UNBOUNDED: Option<MessageQueueSemantics> =
        Some(MessageQueueSemantics::UNBOUNDED);

    #[test]
    fn the_requirement_of_the_dispatcher_selects_the_mailbox_of_an_actor_that_requires_nothing() {
        let mut mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, bounded(5));
        let props = || Props::new(|| Idle);
        assert_eq!(
            capacity_under_dispatcher(props(), &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED).unwrap(),
            2
        );
        assert_eq!(
            capacity_under_dispatcher(props(), &mailboxes, None).unwrap(),
            5
        );
    }

    #[test]
    fn the_requirement_of_the_actor_decides_before_the_requirement_of_the_dispatcher() {
        let mut mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        mailboxes.bind_requirement(AUDIT_TRAIL, "three");
        let props = Props::new(|| RequiresAuditTrail);
        assert_eq!(
            capacity_under_dispatcher(props, &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED).unwrap(),
            3
        );
    }

    #[test]
    fn a_selected_id_decides_before_the_requirement_of_the_dispatcher() {
        let mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        let props = Props::new(|| Idle).with_mailbox_id("three");
        assert_eq!(
            capacity_under_dispatcher(props, &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED).unwrap(),
            3
        );
    }

    #[test]
    fn the_requirement_of_the_dispatcher_serves_an_actor_whose_requirement_has_no_binding() {
        let mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        let unbound_actor = || Props::new(|| RequiresAuditTrail);
        assert_eq!(
            capacity_under_dispatcher(unbound_actor(), &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED)
                .unwrap(),
            2
        );
        let error = capacity_under_dispatcher(unbound_actor(), &mailboxes, None).unwrap_err();
        assert_eq!(
            error.message(),
            "Mailbox Mapping for [interface app.AuditTrailSemantics] not configured"
        );
        let any_queue = Props::new(|| RequiresAnyQueue);
        assert_eq!(
            capacity_under_dispatcher(any_queue, &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED)
                .unwrap(),
            2
        );
    }

    #[test]
    fn the_requirement_of_the_dispatcher_also_serves_an_actor_bound_to_an_unregistered_mailbox() {
        let mut mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        mailboxes.bind_requirement(AUDIT_TRAIL, "missing");
        let props = Props::new(|| RequiresAuditTrail);
        assert_eq!(
            capacity_under_dispatcher(props, &mailboxes, DISPATCHER_REQUIRES_UNBOUNDED).unwrap(),
            2
        );
    }

    #[test]
    fn a_requirement_of_the_dispatcher_without_binding_fails_naming_the_semantics_of_the_dispatcher()
     {
        let mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        for props in [Props::new(|| Idle), Props::new(|| RequiresAuditTrail)] {
            let error =
                capacity_under_dispatcher(props, &mailboxes, Some(MessageQueueSemantics::BOUNDED))
                    .unwrap_err();
            assert_eq!(
                error.message(),
                "Mailbox Mapping for [interface org.apache.pekko.dispatch.BoundedMessageQueueSemantics] not configured"
            );
        }
    }

    #[test]
    fn a_dispatcher_requiring_any_queue_has_no_requirement() {
        let mut mailboxes = mailboxes_with_the_unbounded_semantics_bound_to_capacity_two();
        mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, bounded(5));
        let any_queue = Some(MessageQueueSemantics::MESSAGE_QUEUE);
        let plain = capacity_under_dispatcher(Props::new(|| Idle), &mailboxes, any_queue);
        assert_eq!(plain.unwrap(), 5);
        let requiring_any = Props::new(|| RequiresAnyQueue);
        let error = capacity_under_dispatcher(requiring_any, &mailboxes, any_queue).unwrap_err();
        assert_eq!(
            error.message(),
            "Mailbox Mapping for [interface org.apache.pekko.dispatch.MessageQueue] not configured"
        );
    }

    fn deployment_with_mailbox(mailbox: &str) -> crate::actor::Deploy {
        crate::actor::Deploy::new(
            "/a",
            crate::config::Config::empty(),
            crate::actor::RouterConfig::NoRouter,
            crate::actor::Scope::NoScopeGiven,
            "",
            mailbox,
        )
    }

    #[test]
    fn a_deployment_with_a_mailbox_overrides_the_props_and_an_empty_or_missing_one_does_not() {
        let selected = Props::new(|| Idle).with_mailbox_id("props-mailbox");
        let deployed = selected
            .clone()
            .deployed_over(Some(&deployment_with_mailbox("deploy-mailbox")));
        assert_eq!(deployed.mailbox_id(), "deploy-mailbox");
        let empty = selected
            .clone()
            .deployed_over(Some(&deployment_with_mailbox("")));
        assert_eq!(empty.mailbox_id(), "props-mailbox");
        assert_eq!(selected.deployed_over(None).mailbox_id(), "props-mailbox");
    }

    #[test]
    fn props_without_a_selection_resolve_to_the_type_registered_as_the_default() {
        let mut mailboxes = Mailboxes::new();
        mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, bounded(3));
        let (_, dead_letters): (_, ActorRef) = recorder("pekko://sys/deadLetters");
        let queue = Props::new(|| Idle)
            .resolve_mailbox(&mailboxes, None)
            .unwrap()
            .new_message_queue(&dead_letters);
        let accepted = (0..10)
            .filter(|number| {
                queue.enqueue(
                    &dead_letters,
                    Envelope {
                        message: Arc::new(*number),
                        sender: None,
                    },
                )
            })
            .count();
        assert_eq!(accepted, 3);
    }
}
