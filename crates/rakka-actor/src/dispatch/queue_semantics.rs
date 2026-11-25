// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `MessageQueueSemantics`: the marker type `T` of `RequiresMessageQueue[T]` (`Mailbox.scala:533-672,1070`).
//!
//! Pekko names a requirement with a JVM class and resolves it through the `pekko.actor.mailbox.requirements` table, so any user
//! type can be a requirement. The set is open here too: a semantics is identified by the class name its Pekko counterpart has.

use std::fmt;

/// The semantics an actor requires of its message queue (`RequiresMessageQueue[T]`), and the key of the requirements table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageQueueSemantics {
    class_name: &'static str,
}

impl MessageQueueSemantics {
    /// `org.apache.pekko.dispatch.UnboundedMessageQueueSemantics`.
    pub const UNBOUNDED: MessageQueueSemantics =
        MessageQueueSemantics::new("org.apache.pekko.dispatch.UnboundedMessageQueueSemantics");

    /// `org.apache.pekko.dispatch.BoundedMessageQueueSemantics`.
    pub const BOUNDED: MessageQueueSemantics =
        MessageQueueSemantics::new("org.apache.pekko.dispatch.BoundedMessageQueueSemantics");

    /// `org.apache.pekko.dispatch.MultipleConsumerSemantics`: the queue can be read by several consumers (`Mailbox.scala:533`); both built-in
    /// queues have it, and `reference.conf` binds it to `unbounded-queue-based`.
    pub const MULTIPLE_CONSUMER: MessageQueueSemantics =
        MessageQueueSemantics::new("org.apache.pekko.dispatch.MultipleConsumerSemantics");

    /// `org.apache.pekko.dispatch.MessageQueue`: what every queue satisfies. A dispatcher that requires it has no requirement; an
    /// actor that requires it has none of its own to bind, so its mailbox comes from the dispatcher's requirement or fails.
    pub const MESSAGE_QUEUE: MessageQueueSemantics =
        MessageQueueSemantics::new("org.apache.pekko.dispatch.MessageQueue");

    /// A semantics identified by `class_name`, the name of the marker type: how an application defines its own requirement.
    pub const fn new(class_name: &'static str) -> MessageQueueSemantics {
        MessageQueueSemantics { class_name }
    }

    /// The class name that identifies the semantics.
    pub const fn class_name(&self) -> &'static str {
        self.class_name
    }
}

/// How the JVM prints the class of a marker trait (`Class.toString`), as Pekko's messages quote it.
impl fmt::Display for MessageQueueSemantics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "interface {}", self.class_name)
    }
}

/// `ProducesMessageQueue[T]` of a `MailboxType`: the class of the queue it creates and every semantics that queue satisfies.
///
/// Pekko checks `requirement.isAssignableFrom(T)`, so a queue class satisfies its own marker, the markers it extends and
/// the markers they extend. The relation is not derived here: the type lists the full set, which keeps `MessageQueueSemantics`
/// open (an application type names its own markers and any it extends).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducedQueue {
    class_name: &'static str,
    satisfies: &'static [MessageQueueSemantics],
    is_interface: bool,
}

impl ProducedQueue {
    /// A queue class named `class_name` (as `Class.getName` prints it) that satisfies `satisfies`.
    pub const fn new(
        class_name: &'static str,
        satisfies: &'static [MessageQueueSemantics],
    ) -> Self {
        ProducedQueue {
            class_name,
            satisfies,
            is_interface: false,
        }
    }

    /// What a type that makes no promise produces (`classOf[MessageQueue]`): it satisfies nothing but `MessageQueue`.
    pub const PLAIN: ProducedQueue = ProducedQueue {
        class_name: "org.apache.pekko.dispatch.MessageQueue",
        satisfies: &[],
        is_interface: true,
    };

    /// `requirement.isAssignableFrom(queueClass)`: every queue satisfies `MessageQueue`.
    pub fn satisfies(&self, requirement: MessageQueueSemantics) -> bool {
        requirement == MessageQueueSemantics::MESSAGE_QUEUE || self.satisfies.contains(&requirement)
    }
}

/// How the JVM prints the class of the queue (`Class.toString`), as Pekko's messages quote it.
impl fmt::Display for ProducedQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_interface {
            "interface"
        } else {
            "class"
        };
        write!(f, "{kind} {}", self.class_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUDIT_TRAIL: MessageQueueSemantics = MessageQueueSemantics::new("app.AuditTrail");

    #[test]
    fn a_produced_queue_satisfies_the_semantics_it_lists_and_every_queue_satisfies_message_queue() {
        let produced = ProducedQueue::new(
            "app.AuditQueue",
            &[MessageQueueSemantics::UNBOUNDED, AUDIT_TRAIL],
        );
        assert!(produced.satisfies(MessageQueueSemantics::UNBOUNDED));
        assert!(produced.satisfies(AUDIT_TRAIL));
        assert!(produced.satisfies(MessageQueueSemantics::MESSAGE_QUEUE));
        assert!(!produced.satisfies(MessageQueueSemantics::BOUNDED));
        assert!(!ProducedQueue::PLAIN.satisfies(AUDIT_TRAIL));
        assert_eq!(produced.to_string(), "class app.AuditQueue");
        assert_eq!(
            ProducedQueue::PLAIN.to_string(),
            "interface org.apache.pekko.dispatch.MessageQueue"
        );
    }

    #[test]
    fn a_semantics_is_identified_by_its_class_name_alone() {
        let defined_by_the_application = MessageQueueSemantics::new("app.AuditTrail");
        assert_eq!(
            defined_by_the_application,
            MessageQueueSemantics::new("app.AuditTrail")
        );
        assert_ne!(defined_by_the_application, MessageQueueSemantics::UNBOUNDED);
        assert_ne!(
            MessageQueueSemantics::UNBOUNDED,
            MessageQueueSemantics::BOUNDED
        );
        assert_eq!(defined_by_the_application.class_name(), "app.AuditTrail");
    }

    #[test]
    fn it_prints_as_the_jvm_prints_the_class_of_a_marker_trait() {
        assert_eq!(
            MessageQueueSemantics::UNBOUNDED.to_string(),
            "interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics"
        );
    }
}
