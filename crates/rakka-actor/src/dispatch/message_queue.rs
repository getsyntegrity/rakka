// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `MessageQueue` and the default `UnboundedMailbox` queue (`Mailbox.scala:376-410,677-696`).
//!
//! A `MessageQueue` holds the user messages of one mailbox: any number of producers, one consumer. The mailbox state
//! (`MailboxStatus`) and the system-message queue are separate pieces. Pekko's `enqueue(receiver, handle)` and
//! `cleanUp(owner, deadLetters)` also receive the actor so that a bounded queue can dead-letter an overflow; this unbounded
//! queue never needs it, so the argument arrives with the bounded queues and the `deadLetterMailbox` (ADR-0001 steps 5 and 7).
//! `MailboxType` and `Mailboxes` are not defined yet (Q-MBX-2 is open).

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::actor::actor_ref::ActorRef;
use crate::actor::props::AnyMessage;

/// `Envelope`: a user message and its sender.
pub struct Envelope {
    pub message: AnyMessage,
    pub sender: Option<ActorRef>,
}

/// `org.apache.pekko.dispatch.MessageQueue`.
pub trait MessageQueue: Send + Sync {
    /// Adds the envelope at the tail.
    fn enqueue(&self, envelope: Envelope);
    /// Removes the head, or `None` when empty.
    fn dequeue(&self) -> Option<Envelope>;
    /// The number of queued envelopes (approximate for concurrent queues in Pekko; exact here).
    fn number_of_messages(&self) -> usize;
    /// Whether anything is queued.
    fn has_messages(&self) -> bool;
    /// Moves every queued envelope, oldest first, to `dead_letters`.
    fn clean_up(&self, dead_letters: &dyn MessageQueue);
}

/// FIFO queue without a bound (the `ConcurrentLinkedQueue` of `UnboundedMailbox`).
#[derive(Default)]
struct UnboundedMessageQueue {
    queue: Mutex<VecDeque<Envelope>>,
}

impl MessageQueue for UnboundedMessageQueue {
    fn enqueue(&self, envelope: Envelope) {
        self.queue.lock().unwrap().push_back(envelope);
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

    fn clean_up(&self, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(envelope);
        }
    }
}

/// `UnboundedMailbox`, the default mailbox: an unbounded FIFO.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnboundedMailbox;

impl UnboundedMailbox {
    /// A new, empty queue for one mailbox.
    pub fn create(&self) -> Box<dyn MessageQueue> {
        Box::new(UnboundedMessageQueue::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

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
            queue.enqueue(envelope(name));
        }
        assert_eq!(drain(&*queue), ["a", "b", "c"]);
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn the_counts_follow_enqueue_and_dequeue() {
        let queue = UnboundedMailbox.create();
        queue.enqueue(envelope("a"));
        queue.enqueue(envelope("b"));
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
        queue.enqueue(envelope("a"));
        queue.enqueue(envelope("b"));
        assert_eq!(text(&queue.dequeue().unwrap()), "a");
        queue.enqueue(envelope("c"));
        assert_eq!(drain(&*queue), ["b", "c"]);
    }

    #[test]
    fn an_envelope_keeps_its_message_and_sender() {
        let queue = UnboundedMailbox.create();
        let message: AnyMessage = Arc::new(7_u32);
        queue.enqueue(Envelope {
            message: Arc::clone(&message),
            sender: None,
        });
        let out = queue.dequeue().unwrap();
        assert!(Arc::ptr_eq(&out.message, &message));
        assert!(out.sender.is_none());
    }

    #[test]
    fn clean_up_moves_every_envelope_to_the_dead_letters_queue_in_order_and_empties_the_queue() {
        let (queue, dead_letters) = (UnboundedMailbox.create(), UnboundedMailbox.create());
        for name in ["a", "b", "c"] {
            queue.enqueue(envelope(name));
        }
        queue.clean_up(&*dead_letters);
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (0, false)
        );
        assert_eq!(drain(&*dead_letters), ["a", "b", "c"]);
    }

    #[test]
    fn clean_up_of_an_empty_queue_leaves_both_empty() {
        let (queue, dead_letters) = (UnboundedMailbox.create(), UnboundedMailbox.create());
        queue.clean_up(&*dead_letters);
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
                        queue.enqueue(envelope(&format!("{producer}:{n}")));
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
