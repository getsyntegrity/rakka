// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `SingleConsumerOnlyUnboundedMailbox` and its `NodeMessageQueue` (`Mailbox.scala:412-434,699-708`, AC-MBX-006).
//!
//! Any number of producers and one consumer. Pekko's queue is Vyukov's lock-free node queue and exists to be faster than the
//! default; this one holds a `Mutex` only for the push or pop itself, so it keeps the observable behaviour and not that speed.
//! Pekko's `NodeMessageQueue` is `UnboundedMessageQueueSemantics` and does not declare `MultipleConsumerSemantics`: that is what this type declares (the capability API of Q-MBX-14, accepted with
//! changes by the Owner on 2026-11-26).

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use crate::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};

/// `SingleConsumerOnlyUnboundedMailbox`: every actor gets its own unbounded queue, read only by that actor.
#[derive(Clone, Copy, Debug, Default)]
pub struct SingleConsumerOnlyUnboundedMailbox;

/// What the type promises and what each queue it creates answers for itself: unbounded, and not for several consumers.
const PRODUCED: ProducedQueue = ProducedQueue::new(
    "org.apache.pekko.dispatch.NodeMessageQueue",
    &[MessageQueueSemantics::UNBOUNDED],
);

impl MailboxType for SingleConsumerOnlyUnboundedMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(NodeMessageQueue::default())
    }

    fn produced_queue(&self) -> ProducedQueue {
        PRODUCED
    }
}

#[derive(Default)]
struct NodeMessageQueue {
    nodes: Mutex<VecDeque<Envelope>>,
}

impl NodeMessageQueue {
    fn envelopes(&self) -> MutexGuard<'_, VecDeque<Envelope>> {
        self.nodes.lock().unwrap()
    }
}

impl MessageQueue for NodeMessageQueue {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::queue_semantics::MessageQueueSemantics;
    use std::sync::{Arc, Barrier};
    use std::thread;

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

    fn new_queue() -> Box<dyn MessageQueue> {
        SingleConsumerOnlyUnboundedMailbox.create(&receiver())
    }

    #[test]
    fn the_queue_declares_what_pekkos_node_message_queue_is_an_unbounded_queue_that_is_not_for_several_consumers()
     {
        let queue = new_queue();
        let answers = [
            MessageQueueSemantics::MESSAGE_QUEUE,
            MessageQueueSemantics::UNBOUNDED,
            MessageQueueSemantics::MULTIPLE_CONSUMER,
            MessageQueueSemantics::BOUNDED,
        ]
        .map(|semantics| queue.satisfies(semantics));
        assert_eq!(answers, [Some(true), Some(true), Some(false), Some(false)]);
        let promised = SingleConsumerOnlyUnboundedMailbox.produced_queue();
        assert!(promised.satisfies(MessageQueueSemantics::UNBOUNDED));
        assert!(!promised.satisfies(MessageQueueSemantics::MULTIPLE_CONSUMER));
        assert_eq!(
            (queue.class_name(), promised.class_name()),
            (
                "org.apache.pekko.dispatch.NodeMessageQueue",
                "org.apache.pekko.dispatch.NodeMessageQueue"
            )
        );
    }

    #[test]
    fn the_queue_is_first_in_first_out_and_counts_what_it_holds() {
        let queue = new_queue();
        assert!(queue.dequeue().is_none() && !queue.has_messages());
        for name in ["a", "b", "c"] {
            assert!(queue.enqueue(&receiver(), envelope(name)));
        }
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (3, true)
        );
        assert_eq!(drain(&*queue), ["a", "b", "c"]);
        assert_eq!(
            (queue.number_of_messages(), queue.has_messages()),
            (0, false)
        );
    }

    #[test]
    fn clean_up_moves_every_envelope_to_dead_letters_in_order() {
        let (queue, dead_letters) = (new_queue(), new_queue());
        for name in ["a", "b", "c"] {
            queue.enqueue(&receiver(), envelope(name));
        }
        queue.clean_up(&receiver(), &*dead_letters);
        assert!(!queue.has_messages());
        assert_eq!(drain(&*dead_letters), ["a", "b", "c"]);
    }

    #[test]
    fn concurrent_producers_lose_nothing_and_keep_each_producers_order_for_the_one_consumer() {
        let queue: Arc<dyn MessageQueue> = Arc::from(new_queue());
        let start = Arc::new(Barrier::new(4));
        let producers: Vec<_> = (0..4)
            .map(|producer| {
                let (queue, start) = (Arc::clone(&queue), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    for n in 0..250 {
                        queue.enqueue(&receiver(), envelope(&format!("{producer}:{n}")));
                    }
                })
            })
            .collect();
        let mut next = [0_usize; 4];
        let mut taken = 0;
        let started = std::time::Instant::now();
        while taken < 1000 {
            assert!(
                started.elapsed() < std::time::Duration::from_secs(10),
                "only {taken} envelopes arrived"
            );
            let Some(envelope) = queue.dequeue() else {
                thread::yield_now();
                continue;
            };
            let text = envelope.message.downcast_ref::<String>().expect("string");
            let (producer, n) = text.split_once(':').expect("producer:n");
            let producer: usize = producer.parse().expect("producer");
            assert_eq!(n.parse::<usize>().expect("n"), next[producer]);
            next[producer] += 1;
            taken += 1;
        }
        producers.into_iter().for_each(|p| p.join().unwrap());
        assert_eq!(next, [250; 4]);
        assert!(!queue.has_messages());
    }
}
