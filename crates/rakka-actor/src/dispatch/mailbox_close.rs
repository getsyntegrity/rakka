// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala:80-84, AbstractDispatcher.scala:279-285 and Mailbox.scala:354-368 (Apache-2.0).
//! Enqueue protocol of a user message (Owner decision Q-MBX-8, accepted 2026-11-20).
//!
//! Pekko enqueues into whichever mailbox the sender read and never checks that it is closed, so a message that arrives after
//! `cleanUp` has drained the queue stays there forever (oracle cases CC-05/CC-05a). Rust closes that window: the producer
//! enqueues and then looks at the status again. The closer sets `Closed` *before* it drains the queue, so either its drain sees
//! the producer's message or the producer sees `Closed` and drains the queue itself. Either way the message reaches dead letters.

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::mailbox_status::MailboxStatus;
use crate::dispatch::message_queue::{Envelope, MessageQueue, UnboundedMailbox};

/// What `enqueue_unless_closed` did with a message.
pub(crate) struct Enqueued {
    /// The message is waiting in an open mailbox, so a turn must be scheduled to consume it. It is not when the queue refused it (already
    /// dead-lettered by the queue) or when the mailbox is closed.
    pub(crate) awaits_consumption: bool,
    /// Envelopes the caller must publish as dead letters, oldest first: the message itself when the mailbox was already closed, or
    /// whatever raced with the close.
    pub(crate) dead: Vec<Envelope>,
}

/// Enqueues `envelope` for `receiver` unless the mailbox is closed.
pub(crate) fn enqueue_unless_closed(
    status: &MailboxStatus,
    queue: &dyn MessageQueue,
    receiver: &ActorRef,
    envelope: Envelope,
) -> Enqueued {
    if status.is_closed() {
        return Enqueued {
            awaits_consumption: false,
            dead: vec![envelope],
        };
    }
    let stored = queue.enqueue(receiver, envelope);
    if !status.is_closed() {
        return Enqueued {
            awaits_consumption: stored,
            dead: Vec::new(),
        };
    }
    let swept = UnboundedMailbox.create();
    queue.clean_up(receiver, &*swept);
    Enqueued {
        awaits_consumption: false,
        dead: std::iter::from_fn(|| swept.dequeue()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
    use crate::dispatch::message_queue::UnboundedMailbox;
    use crate::dispatch::test_support::recorder;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn envelope(text: &str) -> Envelope {
        Envelope {
            message: Arc::new(text.to_owned()),
            sender: None,
        }
    }

    fn receiver() -> ActorRef {
        recorder("pekko://sys/user/receiver").1
    }

    fn texts(envelopes: Vec<Envelope>) -> Vec<String> {
        envelopes
            .iter()
            .map(|e| e.message.downcast_ref::<String>().unwrap().clone())
            .collect()
    }

    fn drain(queue: &dyn MessageQueue) -> Vec<String> {
        texts(std::iter::from_fn(|| queue.dequeue()).collect())
    }

    /// Collaborator double (not part of the scenario): a queue that, on its first `enqueue`, lets the closer run to completion
    /// first -- the mailbox becomes closed and `clean_up` empties the queue -- and only then adds the envelope. It is the
    /// interleaving "the producer checked, then the actor closed and was cleaned up, then the producer pushed".
    struct CloserWinsTheRace {
        inner: Box<dyn MessageQueue>,
        status: Arc<MailboxStatus>,
        armed: AtomicBool,
    }

    impl MessageQueue for CloserWinsTheRace {
        fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool {
            if self.armed.swap(false, Ordering::SeqCst) {
                self.status.become_closed();
                self.inner.clean_up(receiver, &*UnboundedMailbox.create());
            }
            self.inner.enqueue(receiver, envelope)
        }
        fn dequeue(&self) -> Option<Envelope> {
            self.inner.dequeue()
        }
        fn number_of_messages(&self) -> usize {
            self.inner.number_of_messages()
        }
        fn has_messages(&self) -> bool {
            self.inner.has_messages()
        }
        fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
            self.inner.clean_up(owner, dead_letters);
        }
    }

    #[test]
    fn an_open_mailbox_keeps_the_message_and_returns_nothing_for_dead_letters() {
        let (status, queue) = (MailboxStatus::new(), UnboundedMailbox.create());
        let enqueued = enqueue_unless_closed(&status, &*queue, &receiver(), envelope("a"));
        assert!(enqueued.dead.is_empty() && enqueued.awaits_consumption);
        assert_eq!(drain(&*queue), ["a"]);
    }

    #[test]
    fn a_suspended_mailbox_still_queues_the_message() {
        let (status, queue) = (MailboxStatus::new(), UnboundedMailbox.create());
        status.suspend();
        let enqueued = enqueue_unless_closed(&status, &*queue, &receiver(), envelope("a"));
        assert!(enqueued.dead.is_empty() && enqueued.awaits_consumption);
        assert_eq!(drain(&*queue), ["a"]);
    }

    #[test]
    fn a_closed_mailbox_hands_the_message_back_for_dead_letters_and_queues_nothing() {
        let (status, queue) = (MailboxStatus::new(), UnboundedMailbox.create());
        status.become_closed();
        let enqueued = enqueue_unless_closed(&status, &*queue, &receiver(), envelope("a"));
        assert!(!enqueued.awaits_consumption);
        assert_eq!(texts(enqueued.dead), ["a"]);
        assert!(!queue.has_messages());
    }

    #[test]
    fn a_message_pushed_after_the_closer_finished_is_not_stranded() {
        let status = Arc::new(MailboxStatus::new());
        let queue = CloserWinsTheRace {
            inner: UnboundedMailbox.create(),
            status: Arc::clone(&status),
            armed: AtomicBool::new(true),
        };
        let enqueued = enqueue_unless_closed(&status, &queue, &receiver(), envelope("late"));
        assert!(!enqueued.awaits_consumption);
        assert_eq!(
            texts(enqueued.dead),
            ["late"],
            "the message must go to dead letters"
        );
        assert!(
            !queue.has_messages(),
            "nothing may stay in the closed mailbox's queue"
        );
    }

    #[test]
    fn producers_racing_the_closer_account_for_every_message_exactly_once() {
        for _ in 0..200 {
            let status = Arc::new(MailboxStatus::new());
            let queue: Arc<dyn MessageQueue> = Arc::from(UnboundedMailbox.create());
            let start = Arc::new(Barrier::new(5));
            let producers: Vec<_> = (0..4)
                .map(|producer| {
                    let (status, queue, start, receiver) = (
                        Arc::clone(&status),
                        Arc::clone(&queue),
                        Arc::clone(&start),
                        receiver(),
                    );
                    thread::spawn(move || {
                        start.wait();
                        (0..50)
                            .flat_map(|n| {
                                texts(
                                    enqueue_unless_closed(
                                        &status,
                                        &*queue,
                                        &receiver,
                                        envelope(&format!("{producer}:{n}")),
                                    )
                                    .dead,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            let closer = {
                let (status, queue, start) =
                    (Arc::clone(&status), Arc::clone(&queue), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    status.become_closed();
                    let swept = UnboundedMailbox.create();
                    queue.clean_up(&receiver(), &*swept);
                    drain(&*swept)
                })
            };
            let mut dead: Vec<String> = producers
                .into_iter()
                .flat_map(|p| p.join().unwrap())
                .collect();
            dead.extend(closer.join().unwrap());
            assert!(
                !queue.has_messages(),
                "a message was stranded in the closed queue"
            );
            dead.sort();
            assert!(
                dead.windows(2).all(|pair| pair[0] != pair[1]),
                "a message was reported twice"
            );
            assert_eq!(
                dead.len(),
                200,
                "with nobody consuming, every message must end in dead letters"
            );
        }
    }

    #[test]
    fn a_message_the_queue_refuses_needs_no_consumption_and_is_not_handed_back() {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let queue = NonBlockingBoundedMailbox::new(0)
            .unwrap()
            .create(dead_letters_ref);
        let status = MailboxStatus::new();

        let enqueued = enqueue_unless_closed(&status, &*queue, &receiver(), envelope("refused"));

        assert!(!enqueued.awaits_consumption);
        assert!(
            enqueued.dead.is_empty(),
            "the queue already dead-lettered it"
        );
        assert_eq!(dead_letters.told.lock().unwrap().len(), 1);
    }
}
