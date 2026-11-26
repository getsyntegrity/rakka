// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `MessageQueue` and the default `UnboundedMailbox` queue (`Mailbox.scala:376-410,677-696`).
//!
//! A `MessageQueue` holds the user messages of one mailbox: any number of producers, one consumer. The mailbox state
//! (`MailboxStatus`) and the system-message queue are separate pieces. Like Pekko's `enqueue(receiver, handle)` and
//! `cleanUp(owner, deadLetters)`, the operations that can refuse a message receive the actor, so a bounded queue can dead-letter an
//! overflow addressed to it without keeping a reference to its owner (PROP-7B-1, alternative A, approved by the Owner).
//! `MailboxType` is the factory of those queues; the `Mailboxes` registry and `requirements` are not defined yet (Q-MBX-2 is open).

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::actor::actor_ref::ActorRef;
use crate::actor::props::AnyMessage;
use crate::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};

/// `Envelope`: a user message and its sender.
pub struct Envelope {
    pub message: AnyMessage,
    pub sender: Option<ActorRef>,
}

/// `org.apache.pekko.dispatch.MessageQueue`.
pub trait MessageQueue: Send + Sync {
    /// Adds the envelope at the tail and answers whether it was stored. A queue that refuses (a full bounded one) has already
    /// sent the envelope to dead letters as addressed to `receiver`, the actor this queue belongs to (Pekko's `enqueue(receiver, handle)`).
    fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool;
    /// Removes the head, or `None` when empty.
    fn dequeue(&self) -> Option<Envelope>;
    /// The number of queued envelopes (approximate for concurrent queues in Pekko; exact here).
    fn number_of_messages(&self) -> usize;
    /// Whether anything is queued.
    fn has_messages(&self) -> bool;
    /// Moves every queued envelope, oldest first, to `dead_letters` as addressed to `owner` (Pekko's `cleanUp(owner, deadLetters)`).
    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue);
}

/// `org.apache.pekko.dispatch.MailboxType`: what `Props::with_mailbox` selects and the cell asks, once per actor, for the queue of its mailbox
/// (`Mailboxes.scala` `MailboxType.create`). A restart keeps the queue, so `create` is not called again.
///
/// Pekko passes `(owner, system)`; here the type receives the system's `deadLetters`, which is all a queue needs to dead-letter a refusal
/// without holding its owner (PROP-7B-1, alternative A). Queue semantics (`ProducesMessageQueue`, `requirements`) wait for Q-MBX-2.
pub trait MailboxType: Send + Sync + 'static {
    /// A new, empty queue for one mailbox.
    fn create(&self, dead_letters: &ActorRef) -> Box<dyn MessageQueue>;

    /// `ProducesMessageQueue[T]`: the queue class this type promises and the semantics it satisfies. A type that makes no promise
    /// produces a plain `MessageQueue` (`Mailboxes.getProducedMessageQueueType`).
    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::PLAIN
    }
}

/// FIFO queue without a bound (the `ConcurrentLinkedQueue` of `UnboundedMailbox`).
#[derive(Default)]
struct UnboundedMessageQueue {
    queue: Mutex<VecDeque<Envelope>>,
}

impl MessageQueue for UnboundedMessageQueue {
    fn enqueue(&self, _receiver: &ActorRef, envelope: Envelope) -> bool {
        self.queue.lock().unwrap().push_back(envelope);
        true
    }

    fn dequeue(&self) -> Option<Envelope> {
        self.queue.lock().unwrap().pop_front()
    }

    fn number_of_messages(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    fn has_messages(&self) -> bool {
        !self.queue.lock().unwrap().is_empty()
    }

    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(owner, envelope);
        }
    }
}

/// `UnboundedMailbox`, the default mailbox: an unbounded FIFO.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnboundedMailbox;

impl MailboxType for UnboundedMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        UnboundedMailbox::create(self)
    }

    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new(
            "org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue",
            &[MessageQueueSemantics::UNBOUNDED],
        )
    }
}

impl UnboundedMailbox {
    /// A new, empty queue for one mailbox; unlike `MailboxType::create` it needs no `deadLetters` (the cell uses it where it has none).
    /// On the concrete type this inherent method wins over `MailboxType::create`, which it backs.
    pub fn create(&self) -> Box<dyn MessageQueue> {
        Box::new(UnboundedMessageQueue::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn the_unbounded_mailbox_promises_an_unbounded_queue_and_a_type_without_promise_a_plain_one() {
        struct Silent;
        impl MailboxType for Silent {
            fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
                UnboundedMailbox.create()
            }
        }
        let unbounded = UnboundedMailbox.produced_queue();
        assert!(unbounded.satisfies(MessageQueueSemantics::UNBOUNDED));
        assert!(!unbounded.satisfies(MessageQueueSemantics::BOUNDED));
        assert_eq!(
            unbounded.to_string(),
            "class org.apache.pekko.dispatch.UnboundedMailbox$MessageQueue"
        );
        assert_eq!(Silent.produced_queue(), ProducedQueue::PLAIN);
    }

    fn receiver() -> ActorRef {
        crate::dispatch::test_support::recorder("pekko://sys/user/receiver").1
    }

    fn envelope(text: &str) -> Envelope {
        Envelope {
            message: Arc::new(text.to_owned()),
            sender: None,
        }
    }

    fn text(envelope: &Envelope) -> &str {
        envelope
            .message
            .downcast_ref::<String>()
            .expect("test messages are strings")
    }

    fn drain(queue: &dyn MessageQueue) -> Vec<String> {
        std::iter::from_fn(|| queue.dequeue())
            .map(|e| text(&e).to_owned())
            .collect()
    }

    #[test]
    fn a_new_queue_is_empty() {
        let queue = UnboundedMailbox.create();
        assert!(queue.dequeue().is_none());
        assert_eq!(queue.number_of_messages(), 0);
        assert!(!queue.has_messages());
    }

    #[test]
    fn dequeue_returns_envelopes_in_enqueue_order_and_then_nothing() {
        let queue = UnboundedMailbox.create();
        for name in ["a", "b", "c"] {
            queue.enqueue(&receiver(), envelope(name));
        }
        assert_eq!(drain(&*queue), ["a", "b", "c"]);
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn the_counts_follow_enqueue_and_dequeue() {
        let queue = UnboundedMailbox.create();
        queue.enqueue(&receiver(), envelope("a"));
        queue.enqueue(&receiver(), envelope("b"));
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (2, true)
        );
        queue.dequeue();
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (1, true)
        );
        queue.dequeue();
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (0, false)
        );
    }

    #[test]
    fn interleaved_enqueue_and_dequeue_stay_first_in_first_out() {
        let queue = UnboundedMailbox.create();
        queue.enqueue(&receiver(), envelope("a"));
        queue.enqueue(&receiver(), envelope("b"));
        assert_eq!(text(&queue.dequeue().unwrap()), "a");
        queue.enqueue(&receiver(), envelope("c"));
        assert_eq!(drain(&*queue), ["b", "c"]);
    }

    #[test]
    fn an_envelope_keeps_its_message_and_sender() {
        let queue = UnboundedMailbox.create();
        let message: AnyMessage = Arc::new(7_u32);
        queue.enqueue(
            &receiver(),
            Envelope {
                message: Arc::clone(&message),
                sender: None,
            },
        );
        let out = queue.dequeue().unwrap();
        assert!(Arc::ptr_eq(&out.message, &message));
        assert!(out.sender.is_none());
    }

    #[test]
    fn clean_up_moves_every_envelope_to_the_dead_letters_queue_in_order_and_empties_the_queue() {
        let (queue, dead_letters) = (UnboundedMailbox.create(), UnboundedMailbox.create());
        for name in ["a", "b", "c"] {
            queue.enqueue(&receiver(), envelope(name));
        }
        queue.clean_up(&receiver(), &*dead_letters);
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (0, false)
        );
        assert_eq!(drain(&*dead_letters), ["a", "b", "c"]);
    }

    #[test]
    fn clean_up_of_an_empty_queue_leaves_both_empty() {
        let (queue, dead_letters) = (UnboundedMailbox.create(), UnboundedMailbox.create());
        queue.clean_up(&receiver(), &*dead_letters);
        assert!(!queue.has_messages() && !dead_letters.has_messages());
    }

    #[test]
    fn concurrent_producers_lose_nothing_and_keep_each_producers_order() {
        let queue: Arc<dyn MessageQueue> = Arc::from(UnboundedMailbox.create());
        let start = Arc::new(Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|producer| {
                let (queue, start) = (Arc::clone(&queue), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    for n in 0..500 {
                        queue.enqueue(&receiver(), envelope(&format!("{producer}:{n}")));
                    }
                })
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        assert_eq!(queue.number_of_messages(), 2000);
        let mut next = [0_usize; 4];
        for item in drain(&*queue) {
            let (producer, n) = item.split_once(':').unwrap();
            let producer: usize = producer.parse().unwrap();
            assert_eq!(
                n.parse::<usize>().unwrap(),
                next[producer],
                "producer {producer} out of order"
            );
            next[producer] += 1;
        }
        assert_eq!(next, [500; 4]);
    }
}
