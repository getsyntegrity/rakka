// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `UnboundedDequeBasedMailbox`: an unbounded queue that also takes envelopes at its head, which is what a stash needs to put
//! messages back ahead of the ones that arrived meanwhile (`Mailbox.scala:535-549,600-622,851-865`).

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use crate::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};

/// `DequeBasedMessageQueueSemantics`: a queue that also takes envelopes at its head. Reached through
/// [`MessageQueue::as_deque`], because a mailbox holds its queue as a `dyn MessageQueue`.
pub trait DequeBasedMessageQueue: MessageQueue {
    /// Adds the envelope at the head, so it is the next one dequeued, and answers whether it was stored (as `enqueue` does).
    fn enqueue_first(&self, receiver: &ActorRef, envelope: Envelope) -> bool;
}

/// `UnboundedDequeBasedMailbox`: every actor gets its own unbounded deque.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnboundedDequeBasedMailbox;

/// What the type promises and what each queue it creates answers for itself (the capability API of Q-MBX-14, proposed). Pekko's
/// queue is also `DequeBasedMessageQueueSemantics` and `UnboundedDequeBasedMessageQueueSemantics`; those two are left out until
/// Q-MBX-2 is decided, so an actor that requires one of them is rejected here and created in Pekko.
const PRODUCED: ProducedQueue = ProducedQueue::new(
    "org.apache.pekko.dispatch.UnboundedDequeBasedMailbox$MessageQueue",
    &[
        MessageQueueSemantics::UNBOUNDED,
        MessageQueueSemantics::MULTIPLE_CONSUMER,
    ],
);

impl MailboxType for UnboundedDequeBasedMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(UnboundedDequeMessageQueue::default())
    }

    fn produced_queue(&self) -> ProducedQueue {
        PRODUCED
    }
}

#[derive(Default)]
struct UnboundedDequeMessageQueue {
    deque: Mutex<VecDeque<Envelope>>,
}

impl UnboundedDequeMessageQueue {
    fn envelopes(&self) -> MutexGuard<'_, VecDeque<Envelope>> {
        self.deque.lock().unwrap()
    }
}

impl MessageQueue for UnboundedDequeMessageQueue {
    fn enqueue(&self, _receiver: &ActorRef, envelope: Envelope) -> bool {
        self.envelopes().push_back(envelope);
        true
    }

    fn dequeue(&self) -> Option<Envelope> {
        self.envelopes().pop_front()
    }

    fn number_of_messages(&self) -> usize {
        self.envelopes().len()
    }

    fn has_messages(&self) -> bool {
        !self.envelopes().is_empty()
    }

    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(owner, envelope);
        }
    }

    fn satisfies(&self, requirement: MessageQueueSemantics) -> Option<bool> {
        Some(PRODUCED.satisfies(requirement))
    }

    fn class_name(&self) -> &'static str {
        PRODUCED.class_name()
    }

    fn as_deque(&self) -> Option<&dyn DequeBasedMessageQueue> {
        Some(self)
    }
}

impl DequeBasedMessageQueue for UnboundedDequeMessageQueue {
    fn enqueue_first(&self, _receiver: &ActorRef, envelope: Envelope) -> bool {
        self.envelopes().push_front(envelope);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::message_queue::UnboundedMailbox;
    use std::sync::Arc;

    fn receiver() -> ActorRef {
        crate::dispatch::test_support::recorder("pekko://sys/user/receiver").1
    }

    fn envelope(text: &str) -> Envelope {
        Envelope {
            message: Arc::new(text.to_owned()),
            sender: None,
        }
    }

    fn drain(queue: &dyn MessageQueue) -> Vec<String> {
        std::iter::from_fn(|| queue.dequeue())
            .map(|e| e.message.downcast_ref::<String>().expect("string").clone())
            .collect()
    }

    fn deque_queue() -> Box<dyn MessageQueue> {
        UnboundedDequeBasedMailbox.create(&receiver())
    }

    fn put_first(queue: &dyn MessageQueue, text: &str) -> bool {
        queue
            .as_deque()
            .expect("a deque based queue")
            .enqueue_first(&receiver(), envelope(text))
    }

    #[test]
    fn an_envelope_enqueued_first_is_dequeued_before_the_ones_already_queued() {
        let queue = deque_queue();
        queue.enqueue(&receiver(), envelope("a"));
        queue.enqueue(&receiver(), envelope("b"));
        assert!(put_first(&*queue, "c"));
        assert_eq!(queue.number_of_messages(), 3);
        assert_eq!(drain(&*queue), ["c", "a", "b"]);
        assert!(!queue.has_messages());
    }

    #[test]
    fn successive_enqueue_first_reverses_their_order_and_works_on_an_empty_queue() {
        let queue = deque_queue();
        put_first(&*queue, "x");
        put_first(&*queue, "y");
        assert!(queue.has_messages());
        assert_eq!(drain(&*queue), ["y", "x"]);
        assert!(queue.dequeue().is_none());
        assert_eq!(queue.number_of_messages(), 0);
    }

    #[test]
    fn clean_up_moves_the_envelopes_to_dead_letters_from_the_head() {
        let (queue, dead_letters) = (deque_queue(), deque_queue());
        queue.enqueue(&receiver(), envelope("a"));
        put_first(&*queue, "c");
        queue.enqueue(&receiver(), envelope("b"));
        queue.clean_up(&receiver(), &*dead_letters);
        assert!(!queue.has_messages());
        assert_eq!(drain(&*dead_letters), ["c", "a", "b"]);
    }

    #[test]
    fn the_type_promises_and_its_queue_declares_an_unbounded_multiple_consumer_queue() {
        let promised = UnboundedDequeBasedMailbox.produced_queue();
        let created = deque_queue();
        for (semantics, has) in [
            (MessageQueueSemantics::UNBOUNDED, true),
            (MessageQueueSemantics::MULTIPLE_CONSUMER, true),
            (MessageQueueSemantics::MESSAGE_QUEUE, true),
            (MessageQueueSemantics::BOUNDED, false),
        ] {
            assert_eq!(promised.satisfies(semantics), has, "{semantics}");
            assert_eq!(created.satisfies(semantics), Some(has), "{semantics}");
        }
        assert_eq!(
            promised.to_string(),
            format!("class {}", created.class_name())
        );
        assert_eq!(
            created.class_name(),
            "org.apache.pekko.dispatch.UnboundedDequeBasedMailbox$MessageQueue"
        );
    }

    #[test]
    fn a_queue_that_is_not_deque_based_offers_no_deque() {
        assert!(UnboundedMailbox.create().as_deque().is_none());
    }
}
