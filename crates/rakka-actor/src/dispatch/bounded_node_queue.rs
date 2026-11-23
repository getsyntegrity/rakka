// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala and actor/src/main/java/org/apache/pekko/dispatch/AbstractBoundedNodeQueue.java (Apache-2.0).
//! `NonBlockingBoundedMailbox` and its queue `BoundedNodeMessageQueue` (`Mailbox.scala:421-452,707-724`, AC-MBX-008).
//!
//! A bounded multiple-producer, single-consumer queue that never blocks the producer: when it is full the envelope is not stored
//! and goes to `deadLetters` as `DeadLetter(message, sender, receiver)`. It has no push timeout (the blocking `BoundedMailbox` is
//! Q-MBX-1, not here).
//!
//! Differences, all consequences of the language:
//! - Pekko's `enqueue(receiver, handle)` receives the receiver on every call and reaches `deadLetters` through its provider. Here the
//!   receiver also arrives on every call, so the queue keeps no reference to its owner (which would be a cycle with the cell), and
//!   `deadLetters` is given when the queue is created.
//! - Pekko's queue is lock-free; this one holds a `Mutex` only for the push or pop itself. The overflow is sent to `deadLetters` after
//!   the lock is released.
//! - A negative capacity is not rejected by an unchecked `IllegalArgumentException` but by a `Result` of the constructor.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::dead_letter_mailbox::tell_dead_letter;
use crate::dispatch::message_queue::{Envelope, MessageQueue};

/// `new NonBlockingBoundedMailbox(capacity)` with a negative capacity (`IllegalArgumentException` in Pekko).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NegativeCapacity;

impl fmt::Display for NegativeCapacity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("The capacity for NonBlockingBoundedMailbox can not be negative")
    }
}

impl std::error::Error for NegativeCapacity {}

/// `NonBlockingBoundedMailbox`: a bounded mailbox whose full queue discards into `deadLetters` without blocking the sender.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonBlockingBoundedMailbox {
    capacity: usize,
}

impl NonBlockingBoundedMailbox {
    /// A mailbox of `capacity` envelopes. Zero is valid: its queue refuses everything.
    pub fn new(capacity: i32) -> Result<Self, NegativeCapacity> {
        usize::try_from(capacity)
            .map(|capacity| NonBlockingBoundedMailbox { capacity })
            .map_err(|_| NegativeCapacity)
    }

    /// The most envelopes the queue keeps.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// A new, empty queue for one mailbox; what it refuses is dead-lettered to `dead_letters` as addressed to the receiver of `enqueue`.
    pub fn create(&self, dead_letters: ActorRef) -> Box<dyn MessageQueue> {
        Box::new(BoundedNodeMessageQueue {
            capacity: self.capacity,
            dead_letters,
            queue: Mutex::new(VecDeque::new()),
        })
    }
}

struct BoundedNodeMessageQueue {
    capacity: usize,
    dead_letters: ActorRef,
    queue: Mutex<VecDeque<Envelope>>,
}

impl BoundedNodeMessageQueue {
    /// Nothing but plain pushes and pops runs under the lock, so a poisoned lock leaves the queue consistent.
    fn locked(&self) -> MutexGuard<'_, VecDeque<Envelope>> {
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl MessageQueue for BoundedNodeMessageQueue {
    fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool {
        let mut queue = self.locked();
        if queue.len() < self.capacity {
            queue.push_back(envelope);
            return true;
        }
        drop(queue); // `deadLetters` may call back into this queue
        tell_dead_letter(&self.dead_letters, receiver, envelope);
        false
    }

    fn dequeue(&self) -> Option<Envelope> {
        self.locked().pop_front()
    }

    fn number_of_messages(&self) -> usize {
        self.locked().len()
    }

    fn has_messages(&self) -> bool {
        !self.locked().is_empty()
    }

    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(owner, envelope);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::message_queue::UnboundedMailbox;
    use crate::dispatch::test_support::{Recorder, recorder};
    use crate::event::DeadLetter;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;

    const ORACLE_GOLDEN: &str =
        include_str!("../../../../cases/oracle-only/nonblocking-bounded-mailbox.golden");

    /// A queue of `capacity` owned by `pekko://sys/user/<owner>`, and the recorder standing in for `deadLetters`.
    struct Fixture {
        queue: Box<dyn MessageQueue>,
        dead_letters: Arc<Recorder>,
        dead_letters_ref: ActorRef,
        owner: ActorRef,
    }

    fn fixture(capacity: i32, owner_name: &str) -> Fixture {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let (_, owner) = recorder(&format!("pekko://sys/user/{owner_name}"));
        let queue = NonBlockingBoundedMailbox::new(capacity)
            .expect("a non-negative capacity")
            .create(dead_letters_ref.clone());
        Fixture {
            queue,
            dead_letters,
            dead_letters_ref,
            owner,
        }
    }

    fn sender(name: &str) -> ActorRef {
        recorder(&format!("pekko://sys/user/{name}")).1
    }

    fn envelope(text: &str, sender: Option<&ActorRef>) -> Envelope {
        Envelope {
            message: Arc::new(text.to_owned()),
            sender: sender.cloned(),
        }
    }

    fn text(envelope: &Envelope) -> String {
        envelope
            .message
            .downcast_ref::<String>()
            .expect("test messages are strings")
            .clone()
    }

    fn drain(queue: &dyn MessageQueue) -> Vec<String> {
        std::iter::from_fn(|| queue.dequeue())
            .map(|e| text(&e))
            .collect()
    }

    fn dead_letters_told(dead_letters: &Recorder) -> Vec<DeadLetter> {
        dead_letters
            .told
            .lock()
            .unwrap()
            .iter()
            .map(|(message, _)| {
                message
                    .downcast_ref::<DeadLetter>()
                    .expect("a DeadLetter")
                    .clone()
            })
            .collect()
    }

    fn dead_texts(dead_letters: &Recorder) -> Vec<String> {
        dead_letters_told(dead_letters)
            .iter()
            .map(|dead| {
                dead.message
                    .downcast_ref::<String>()
                    .expect("string")
                    .clone()
            })
            .collect()
    }

    #[test]
    fn a_negative_capacity_is_rejected_with_the_pekko_message_and_zero_and_the_maximum_pass() {
        for negative in [-1, i32::MIN] {
            let error = NonBlockingBoundedMailbox::new(negative).unwrap_err();
            assert_eq!(
                error.to_string(),
                "The capacity for NonBlockingBoundedMailbox can not be negative"
            );
        }
        assert_eq!(NonBlockingBoundedMailbox::new(0).unwrap().capacity(), 0);
        assert_eq!(
            NonBlockingBoundedMailbox::new(i32::MAX).unwrap().capacity(),
            i32::MAX as usize
        );
    }

    #[test]
    fn a_new_queue_is_empty_and_accepts_up_to_its_capacity_in_fifo_order() {
        let f = fixture(3, "recv");
        assert!(!f.queue.has_messages() && f.queue.dequeue().is_none());
        for name in ["a", "b", "c"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert_eq!(f.queue.number_of_messages(), 3);
        assert_eq!(drain(&*f.queue), ["a", "b", "c"]);
        assert!(dead_letters_told(&f.dead_letters).is_empty());
    }

    #[test]
    fn a_full_queue_refuses_at_once_keeps_the_first_and_dead_letters_the_rest_in_order() {
        let f = fixture(3, "recv");
        for name in ["a", "b", "c", "d", "e"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert_eq!(f.queue.number_of_messages(), 3);
        assert_eq!(dead_texts(&f.dead_letters), ["d", "e"]);
        assert_eq!(drain(&*f.queue), ["a", "b", "c"]);
    }

    #[test]
    fn dequeue_frees_a_slot_and_the_order_stays_first_in_first_out() {
        let f = fixture(2, "recv");
        for name in ["a", "b", "c"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert_eq!(text(&f.queue.dequeue().unwrap()), "a");
        for name in ["d", "e"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert_eq!(drain(&*f.queue), ["b", "d"]);
        assert_eq!(dead_texts(&f.dead_letters), ["c", "e"]);
    }

    #[test]
    fn capacity_zero_is_a_valid_queue_that_refuses_everything() {
        let f = fixture(0, "recv");
        for name in ["a", "b"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert!(!f.queue.has_messages() && f.queue.number_of_messages() == 0);
        assert_eq!(dead_texts(&f.dead_letters), ["a", "b"]);
    }

    #[test]
    fn a_dead_letter_carries_the_message_the_envelope_sender_and_the_owner_as_receiver() {
        let f = fixture(1, "recv");
        let snd = sender("snd");
        f.queue.enqueue(&f.owner, envelope("kept", Some(&snd)));
        f.queue.enqueue(&f.owner, envelope("refused", Some(&snd)));

        let told = f.dead_letters.told.lock().unwrap();
        let dead = told[0].0.downcast_ref::<DeadLetter>().unwrap();
        assert!(dead.sender == snd && dead.recipient == f.owner);
        assert_eq!(told[0].1.as_ref(), Some(&snd), "it is told by the sender");
    }

    #[test]
    fn a_refused_message_without_sender_comes_from_dead_letters() {
        let f = fixture(0, "recv");
        f.queue.enqueue(&f.owner, envelope("orphan", None));

        let dead = &dead_letters_told(&f.dead_letters)[0];
        assert!(dead.sender == f.dead_letters_ref);
    }

    #[test]
    fn a_refused_dead_letter_is_not_dropped_unlike_in_the_closed_mailbox() {
        let f = fixture(0, "recv");
        let snd = sender("snd");
        let inner = DeadLetter {
            message: Arc::new("inner".to_owned()),
            sender: snd.clone(),
            recipient: f.owner.clone(),
        };
        f.queue.enqueue(
            &f.owner,
            Envelope {
                message: Arc::new(inner),
                sender: Some(snd),
            },
        );

        let dead = &dead_letters_told(&f.dead_letters)[0];
        let wrapped = dead.message.downcast_ref::<DeadLetter>().unwrap();
        assert_eq!(wrapped.message.downcast_ref::<String>().unwrap(), "inner");
    }

    #[test]
    fn clean_up_moves_a_full_queue_to_the_target_in_order_and_frees_every_slot() {
        let f = fixture(3, "recv");
        let target = UnboundedMailbox.create();
        for name in ["a", "b", "c", "d"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        f.queue.clean_up(&f.owner, &*target);
        assert!(!f.queue.has_messages());
        assert_eq!(drain(&*target), ["a", "b", "c"]);
        for name in ["e", "f", "g", "h"] {
            f.queue.enqueue(&f.owner, envelope(name, None));
        }
        assert_eq!(drain(&*f.queue), ["e", "f", "g"]);
        assert_eq!(dead_texts(&f.dead_letters), ["d", "h"]);
    }

    #[test]
    fn a_producer_that_panicked_under_the_lock_does_not_stop_the_other_threads() {
        let (_, dead_letters) = recorder("pekko://sys/deadLetters");
        let (_, owner) = recorder("pekko://sys/user/recv");
        let queue = Arc::new(BoundedNodeMessageQueue {
            capacity: 2,
            dead_letters,
            queue: Mutex::new(VecDeque::new()),
        });
        let poisoner = Arc::clone(&queue);
        let panicked = thread::spawn(move || {
            let _guard = poisoner.queue.lock().unwrap();
            panic!("a thread dies holding the queue lock");
        })
        .join();
        assert!(panicked.is_err() && queue.queue.is_poisoned());

        queue.enqueue(&owner, envelope("still", None));

        assert_eq!(drain(&*queue), ["still"]);
        assert!(!queue.has_messages());
    }

    #[test]
    fn a_dead_letters_that_calls_back_into_the_queue_does_not_deadlock_the_producer() {
        let reentered = Arc::new(AtomicBool::new(false));
        let queue_slot: Arc<Mutex<Option<Arc<dyn MessageQueue>>>> = Arc::new(Mutex::new(None));
        let dead_letters = ActorRef(Arc::new(CallsBack {
            path: crate::actor::ActorPath::from_string("pekko://sys/deadLetters").unwrap(),
            queue: Arc::clone(&queue_slot),
            reentered: Arc::clone(&reentered),
        }));
        let (_, owner) = recorder("pekko://sys/user/recv");
        let queue: Arc<dyn MessageQueue> = Arc::from(
            NonBlockingBoundedMailbox::new(0)
                .unwrap()
                .create(dead_letters),
        );
        *queue_slot.lock().unwrap() = Some(Arc::clone(&queue));

        queue.enqueue(&owner, envelope("refused", None));

        assert!(reentered.load(Ordering::SeqCst));
    }

    /// `deadLetters` that, from another thread, asks the queue that dead-lettered to it for its size: it cannot answer while the
    /// producer still holds the queue's lock.
    struct CallsBack {
        path: crate::actor::ActorPath,
        queue: Arc<Mutex<Option<Arc<dyn MessageQueue>>>>,
        reentered: Arc<AtomicBool>,
    }

    impl crate::actor::actor_ref::InternalActorRef for CallsBack {
        fn path(&self) -> &crate::actor::ActorPath {
            &self.path
        }
        fn tell(&self, _msg: crate::actor::props::AnyMessage, _sender: Option<ActorRef>) {
            let queue = self.queue.lock().unwrap().clone().unwrap();
            let (done, answered) = std::sync::mpsc::channel();
            thread::spawn(move || {
                queue.number_of_messages();
                let _ = done.send(());
            });
            let answered_in_time = answered
                .recv_timeout(std::time::Duration::from_secs(2))
                .is_ok();
            self.reentered.store(answered_in_time, Ordering::SeqCst);
        }
        fn send_system(&self, _msg: crate::dispatch::sysmsg::SystemMessage) {}
        fn stop(&self) {}
        fn is_terminated(&self) -> bool {
            false
        }
    }

    fn per_producer_in_order(seen: &[String]) -> bool {
        let mut last = std::collections::HashMap::new();
        seen.iter().all(|item| {
            let (producer, n) = item.split_once(':').unwrap();
            let n: usize = n.parse().unwrap();
            last.insert(producer.to_owned(), n)
                .is_none_or(|previous| previous < n)
        })
    }

    fn run_producers(
        queue: &Arc<dyn MessageQueue>,
        receiver: &ActorRef,
        producers: usize,
        each: usize,
    ) {
        let start = Arc::new(Barrier::new(producers));
        let handles: Vec<_> = (0..producers)
            .map(|producer| {
                let (queue, start, receiver) =
                    (Arc::clone(queue), Arc::clone(&start), receiver.clone());
                thread::spawn(move || {
                    start.wait();
                    for n in 0..each {
                        queue.enqueue(&receiver, envelope(&format!("{producer}:{n}"), None));
                    }
                })
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
    }

    #[test]
    fn concurrent_producers_nobody_consuming_accept_exactly_the_capacity_and_dead_letter_the_rest()
    {
        let f = fixture(10, "recv");
        let (dead_letters, queue) = (
            f.dead_letters.clone(),
            Arc::<dyn MessageQueue>::from(f.queue),
        );
        run_producers(&queue, &f.owner, 4, 50);

        let kept = drain(&*queue);
        let dead = dead_texts(&dead_letters);
        assert_eq!((kept.len(), dead.len()), (10, 190));
        assert!(per_producer_in_order(&kept));
        let all: std::collections::HashSet<_> = kept.iter().chain(&dead).collect();
        assert_eq!(all.len(), 200, "every message exactly once");
    }

    #[test]
    fn concurrent_producers_with_a_consumer_lose_and_duplicate_nothing_and_never_exceed_the_capacity()
     {
        let f = fixture(5, "recv");
        let (dead_letters, queue) = (
            f.dead_letters.clone(),
            Arc::<dyn MessageQueue>::from(f.queue),
        );
        let done = Arc::new(AtomicBool::new(false));
        let max_seen = Arc::new(AtomicUsize::new(0));
        let consumer = {
            let (queue, done, max_seen) =
                (Arc::clone(&queue), Arc::clone(&done), Arc::clone(&max_seen));
            thread::spawn(move || {
                let mut received = Vec::new();
                loop {
                    let last_round = done.load(Ordering::SeqCst);
                    max_seen.fetch_max(queue.number_of_messages(), Ordering::SeqCst);
                    received.extend(std::iter::from_fn(|| queue.dequeue()).map(|e| text(&e)));
                    if last_round {
                        return received;
                    }
                }
            })
        };
        run_producers(&queue, &f.owner, 4, 50);
        done.store(true, Ordering::SeqCst);
        let received = consumer.join().unwrap();

        let dead_count = dead_letters.told.lock().unwrap().len();
        assert_eq!(received.len() + dead_count, 200);
        assert!(per_producer_in_order(&received));
        assert!(max_seen.load(Ordering::SeqCst) <= 5);
        assert!(!queue.has_messages());
    }

    // ---- the same scripts as the Pekko 2.0.0 oracle: each line is compared with `cases/oracle-only/nonblocking-bounded-mailbox.golden` ----

    fn golden_line(id: &str) -> String {
        let prefix = format!("{id} ");
        let line = ORACLE_GOLDEN
            .lines()
            .find(|line| line.starts_with(&prefix))
            .expect("the golden has the case");
        line.split_once(" -> ").unwrap().1.to_owned()
    }

    fn counts(queue: &dyn MessageQueue) -> String {
        format!(
            "n={} has={}",
            queue.number_of_messages(),
            queue.has_messages()
        )
    }

    fn show_dead(fixture: &Fixture) -> String {
        let shown: Vec<String> = dead_letters_told(&fixture.dead_letters)
            .iter()
            .map(|dead| {
                let message = dead
                    .message
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_else(|| {
                        let inner = dead.message.downcast_ref::<DeadLetter>().unwrap();
                        format!(
                            "DeadLetter({})",
                            inner.message.downcast_ref::<String>().unwrap()
                        )
                    });
                format!(
                    "{message}<-{}->{}",
                    dead.sender.path().name(),
                    dead.recipient.path().name()
                )
            })
            .collect();
        format!("[{}]", shown.join(", "))
    }

    fn show(texts: Vec<String>) -> String {
        format!("[{}]", texts.join(", "))
    }

    #[test]
    fn nb_cf_01_matches_the_oracle() {
        let shown: Vec<String> = [-1, i32::MIN, 0, 1, i32::MAX]
            .iter()
            .map(|&capacity| match NonBlockingBoundedMailbox::new(capacity) {
                Ok(mailbox) => format!("{capacity} => ok capacity={}", mailbox.capacity()),
                Err(error) => format!("{capacity} => ERR IllegalArgumentException [{error}]"),
            })
            .collect();
        assert_eq!(shown.join(" | "), golden_line("NB-CF-01"));
    }

    #[test]
    fn nb_qu_01_matches_the_oracle() {
        let f = fixture(3, "recv");
        let snd = sender("snd");
        let mut line = format!("new: {}", counts(&*f.queue));
        for name in ["a", "b", "c", "d", "e"] {
            f.queue.enqueue(&f.owner, envelope(name, Some(&snd)));
            line.push_str(&format!(" | {name}: {}", counts(&*f.queue)));
        }
        let dead = show_dead(&f);
        let drained = show(drain(&*f.queue));
        line.push_str(&format!(
            " | dead={dead} | drain={drained} {}",
            counts(&*f.queue)
        ));
        assert_eq!(line, golden_line("NB-QU-01"));
    }

    #[test]
    fn nb_qu_02_matches_the_oracle() {
        let f = fixture(2, "recv");
        for name in ["a", "b", "c"] {
            f.queue.enqueue(&f.owner, envelope(name, Some(&f.owner)));
        }
        let first = text(&f.queue.dequeue().unwrap());
        for name in ["d", "e"] {
            f.queue.enqueue(&f.owner, envelope(name, Some(&f.owner)));
        }
        let rest = show(drain(&*f.queue));
        let line = format!(
            "first={first} rest={rest} dead={} {}",
            show_dead(&f),
            counts(&*f.queue)
        );
        assert_eq!(line, golden_line("NB-QU-02"));
    }

    #[test]
    fn nb_qu_03_matches_the_oracle() {
        let one = fixture(1, "recv");
        for name in ["a", "b", "c"] {
            one.queue
                .enqueue(&one.owner, envelope(name, Some(&one.owner)));
        }
        let zero = fixture(0, "other");
        for name in ["x", "y"] {
            zero.queue
                .enqueue(&zero.owner, envelope(name, Some(&zero.owner)));
        }
        let line = format!(
            "one: {} drain={} dead={} | zero: {} drain={} dead={}",
            counts(&*one.queue),
            show(drain(&*one.queue)),
            show_dead(&one),
            counts(&*zero.queue),
            show(drain(&*zero.queue)),
            show_dead(&zero),
        );
        assert_eq!(line, golden_line("NB-QU-03"));
    }

    #[test]
    fn nb_qu_04_matches_the_oracle() {
        let f = fixture(1, "recv");
        let snd = sender("snd");
        f.queue.enqueue(&f.owner, envelope("kept", Some(&snd)));
        f.queue.enqueue(&f.owner, envelope("refused", Some(&snd)));
        f.queue.enqueue(&f.owner, envelope("noSender", None));
        f.queue.enqueue(
            &f.owner,
            Envelope {
                message: Arc::new(DeadLetter {
                    message: Arc::new("inner".to_owned()),
                    sender: snd.clone(),
                    recipient: f.owner.clone(),
                }),
                sender: Some(snd),
            },
        );
        let line = format!("dead={} queue={}", show_dead(&f), show(drain(&*f.queue)));
        assert_eq!(line, golden_line("NB-QU-04"));
    }

    #[test]
    fn nb_qu_05_matches_the_oracle() {
        let f = fixture(3, "recv");
        let target = UnboundedMailbox.create();
        for name in ["a", "b", "c", "d"] {
            f.queue.enqueue(&f.owner, envelope(name, Some(&f.owner)));
        }
        f.queue.clean_up(&f.owner, &*target);
        let after = counts(&*f.queue);
        for name in ["e", "f", "g", "h"] {
            f.queue.enqueue(&f.owner, envelope(name, Some(&f.owner)));
        }
        let moved = counts(&*target);
        let order = show(drain(&*target));
        let refilled = counts(&*f.queue);
        let kept = show(drain(&*f.queue));
        let line = format!(
            "source={after} target={moved} order={order} | refilled: {refilled} drain={kept} dead={}",
            show_dead(&f)
        );
        assert_eq!(line, golden_line("NB-QU-05"));
    }
}
