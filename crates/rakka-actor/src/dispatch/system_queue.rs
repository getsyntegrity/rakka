// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `SystemQueue`: the system-message queue of a mailbox (`DefaultSystemMessageQueue`, `Mailbox.scala:454-527`).
//!
//! Pekko keeps a lock-free list with the newest message at the head and `systemDrain` reverses it, so the oldest message comes
//! out first; after `cleanUp` a sentinel (`NoMessage`) closes the list: `systemEnqueue` no longer stores anything (the message
//! goes to the `deadLetterMailbox`) and `systemDrain` is empty. Here a `Mutex` guards a `Vec` in arrival order, which is that
//! list already reversed (Owner decision Q-MBX-5: `Mutex<Vec>`, no `unsafe`), and the closed state is the sentinel. The
//! `deadLetterMailbox` itself arrives with ADR-0001 step 5: until then the caller decides what to do with a message handed back.

use std::sync::Mutex;

enum State<M> {
    Open(Vec<M>),
    /// the sentinel: `clean_up` has run
    Closed,
}

/// The pending system messages of one mailbox: any number of producers, one consumer.
pub struct SystemQueue<M> {
    state: Mutex<State<M>>,
}

impl<M> Default for SystemQueue<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M> SystemQueue<M> {
    /// An open, empty queue.
    pub fn new() -> Self {
        SystemQueue {
            state: Mutex::new(State::Open(Vec::new())),
        }
    }

    /// `systemEnqueue`: stores the message, or hands it back when the queue is closed so the caller can send it to the
    /// `deadLetterMailbox`.
    pub fn enqueue(&self, message: M) -> Result<(), M> {
        match &mut *self.state.lock().unwrap() {
            State::Open(messages) => {
                messages.push(message);
                Ok(())
            }
            State::Closed => Err(message),
        }
    }

    /// `systemDrain`: everything pending, oldest first; empty once closed.
    pub fn drain(&self) -> Vec<M> {
        match &mut *self.state.lock().unwrap() {
            State::Open(messages) => std::mem::take(messages),
            State::Closed => Vec::new(),
        }
    }

    /// `hasSystemMessages`: false once closed.
    pub fn has_messages(&self) -> bool {
        matches!(&*self.state.lock().unwrap(), State::Open(messages) if !messages.is_empty())
    }

    /// `systemDrain(NoMessage)` as `cleanUp` calls it: closes the queue and returns what was pending, oldest first. Idempotent.
    pub fn clean_up(&self) -> Vec<M> {
        match std::mem::replace(&mut *self.state.lock().unwrap(), State::Closed) {
            State::Open(messages) => messages,
            State::Closed => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn a_new_queue_is_empty() {
        let queue = SystemQueue::<&str>::new();
        assert!(!queue.has_messages());
        assert!(queue.drain().is_empty());
    }

    #[test]
    fn drain_returns_the_oldest_message_first_and_empties_the_queue() {
        let queue = SystemQueue::new();
        for name in ["Suspend", "Resume", "Terminate"] {
            queue.enqueue(name).unwrap();
        }
        assert!(queue.has_messages());
        assert_eq!(queue.drain(), ["Suspend", "Resume", "Terminate"]);
        assert!(!queue.has_messages());
        assert!(queue.drain().is_empty());
    }

    #[test]
    fn messages_enqueued_between_drains_keep_arrival_order() {
        let queue = SystemQueue::new();
        queue.enqueue("a").unwrap();
        queue.enqueue("b").unwrap();
        assert_eq!(queue.drain(), ["a", "b"]);
        queue.enqueue("c").unwrap();
        queue.enqueue("d").unwrap();
        assert_eq!(queue.drain(), ["c", "d"]);
    }

    #[test]
    fn clean_up_returns_what_is_pending_oldest_first_and_closes_the_queue() {
        let queue = SystemQueue::new();
        queue.enqueue("a").unwrap();
        queue.enqueue("b").unwrap();
        assert_eq!(queue.clean_up(), ["a", "b"]);
        assert!(!queue.has_messages());
    }

    #[test]
    fn after_clean_up_enqueue_hands_the_message_back_and_drain_stays_empty() {
        let queue = SystemQueue::new();
        queue.clean_up();
        assert_eq!(queue.enqueue("late"), Err("late"));
        assert!(!queue.has_messages());
        assert!(queue.drain().is_empty());
    }

    #[test]
    fn clean_up_is_idempotent() {
        let queue = SystemQueue::new();
        queue.enqueue("a").unwrap();
        assert_eq!(queue.clean_up(), ["a"]);
        assert!(queue.clean_up().is_empty());
        assert_eq!(queue.enqueue("b"), Err("b"));
    }

    #[test]
    fn concurrent_producers_lose_nothing_and_keep_each_producers_order() {
        let queue = Arc::new(SystemQueue::new());
        let start = Arc::new(Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|producer| {
                let (queue, start) = (Arc::clone(&queue), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    for n in 0..500 {
                        queue.enqueue((producer, n)).unwrap();
                    }
                })
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        let mut next = [0_usize; 4];
        let drained = queue.drain();
        assert_eq!(drained.len(), 2000);
        for (producer, n) in drained {
            assert_eq!(n, next[producer], "producer {producer} out of order");
            next[producer] += 1;
        }
    }

    #[test]
    fn producers_racing_clean_up_account_for_every_message_exactly_once() {
        for _ in 0..100 {
            let queue = Arc::new(SystemQueue::new());
            let start = Arc::new(Barrier::new(5));
            let producers: Vec<_> = (0..4)
                .map(|producer| {
                    let (queue, start) = (Arc::clone(&queue), Arc::clone(&start));
                    thread::spawn(move || {
                        start.wait();
                        (0..50)
                            .filter_map(|n| queue.enqueue((producer, n)).err())
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            let closer = {
                let (queue, start) = (Arc::clone(&queue), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    queue.clean_up()
                })
            };
            let mut seen: Vec<(i32, i32)> = producers
                .into_iter()
                .flat_map(|p| p.join().unwrap())
                .collect();
            seen.extend(closer.join().unwrap());
            assert!(
                !queue.has_messages(),
                "a system message was stranded after the close"
            );
            seen.sort_unstable();
            assert!(
                seen.windows(2).all(|pair| pair[0] != pair[1]),
                "a message was reported twice"
            );
            assert_eq!(
                seen.len(),
                200,
                "every message is either swept by clean_up or handed back"
            );
        }
    }
}
