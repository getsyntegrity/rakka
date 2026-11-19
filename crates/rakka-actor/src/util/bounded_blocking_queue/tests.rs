// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/util/BoundedBlockingQueueSpec.scala (Apache-2.0).
//! `BoundedBlockingQueueSpec` of Pekko 2.0.0 (actor-tests/.../util/BoundedBlockingQueueSpec.scala), one
//! test per `in` block. The spec's `Thread.sleep` and "does not return within 100 ms" checks are
//! replaced by waiting for the thread under test to announce (through the listener) that it is about
//! to park, which happens with the lock held, so that no other thread can overtake it.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[derive(Clone, Debug, PartialEq)]
enum Ev {
    Poll,
    Offer(String),
    GetSize,
    Lock(QueueEvent),
}

type Events = Arc<Mutex<Vec<Ev>>>;

fn record(events: &Events, event: Ev) {
    events.lock().expect("events").push(event);
}

/// `TestBackingQueue`: a linked list that records every `poll`, `offer` and `size`.
struct TestBacking {
    events: Events,
    items: VecDeque<Arc<String>>,
}

impl BackingQueue<Arc<String>> for TestBacking {
    fn offer(&mut self, element: Arc<String>) -> bool {
        record(&self.events, Ev::Offer((*element).clone()));
        self.items.push_back(element);
        true
    }

    fn poll(&mut self) -> Option<Arc<String>> {
        record(&self.events, Ev::Poll);
        self.items.pop_front()
    }

    fn peek(&self) -> Option<&Arc<String>> {
        self.items.front()
    }

    fn len(&self) -> usize {
        record(&self.events, Ev::GetSize);
        self.items.len()
    }

    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&Arc<String>) -> bool) -> bool {
        self.items.remove_first_where(matches)
    }

    fn retain_where(&mut self, keep: &mut dyn FnMut(&Arc<String>) -> bool) -> bool {
        self.items.retain_where(keep)
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn snapshot(&self) -> Vec<Arc<String>> {
        self.items.iter().cloned().collect()
    }
}

/// A backing queue that holds at most `limit` elements: a `LinkedBlockingQueue(limit)`.
struct Limited(VecDeque<Arc<String>>, usize);

impl BackingQueue<Arc<String>> for Limited {
    fn offer(&mut self, element: Arc<String>) -> bool {
        self.0.offer(element)
    }
    fn poll(&mut self) -> Option<Arc<String>> {
        self.0.poll()
    }
    fn peek(&self) -> Option<&Arc<String>> {
        self.0.peek()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn remaining_capacity(&self) -> usize {
        self.1 - self.0.len()
    }
    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&Arc<String>) -> bool) -> bool {
        self.0.remove_first_where(matches)
    }
    fn retain_where(&mut self, keep: &mut dyn FnMut(&Arc<String>) -> bool) -> bool {
        self.0.retain_where(keep)
    }
    fn clear(&mut self) {
        self.0.clear();
    }
    fn snapshot(&self) -> Vec<Arc<String>> {
        self.0.snapshot()
    }
}

struct Context {
    queue: Arc<BoundedBlockingQueue<Arc<String>>>,
    events: Events,
}

fn s(text: &str) -> Arc<String> {
    Arc::new(text.to_owned())
}

fn new_queue(max_capacity: i32) -> Context {
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let backing = TestBacking {
        events: events.clone(),
        items: VecDeque::new(),
    };
    let listener_events = events.clone();
    let listener: Listener = Arc::new(move |event| record(&listener_events, Ev::Lock(event)));
    let queue = BoundedBlockingQueue::with_listener(max_capacity, backing, Some(listener))
        .expect("a valid queue");
    Context {
        queue: Arc::new(queue),
        events,
    }
}

impl Context {
    fn events(&self) -> Vec<Ev> {
        self.events.lock().expect("events").clone()
    }

    fn count(&self, wanted: &Ev) -> usize {
        self.events().iter().filter(|e| *e == wanted).count()
    }

    fn has(&self, wanted: &Ev) -> bool {
        self.count(wanted) > 0
    }

    /// Waits (spinning, never sleeping) until `wanted` has been recorded `times` times.
    fn wait_for(&self, wanted: &Ev, times: usize) {
        while self.count(wanted) < times {
            thread::yield_now();
        }
    }

    /// `events should contain inOrder`: the events appear in this order, not necessarily next to each other.
    fn has_in_order(&self, wanted: &[Ev]) -> bool {
        let mut remaining = wanted.iter();
        let mut next = remaining.next();
        for event in self.events() {
            if next == Some(&event) {
                next = remaining.next();
            }
        }
        next.is_none()
    }

    fn backing_offer(&self, text: &str) {
        self.queue.lock().backing.offer(s(text));
    }
}

fn offer(text: &str) -> Ev {
    Ev::Offer(text.to_owned())
}

const AWAIT_NOT_EMPTY: Ev = Ev::Lock(QueueEvent::AwaitNotEmpty);
const AWAIT_NOT_FULL: Ev = Ev::Lock(QueueEvent::AwaitNotFull);
const SIGNAL_NOT_EMPTY: Ev = Ev::Lock(QueueEvent::SignalNotEmpty);
const SIGNAL_NOT_FULL: Ev = Ev::Lock(QueueEvent::SignalNotFull);

fn requirement_failed(result: Result<impl Sized, IllegalArgumentError>) {
    let error = result.err().expect("the constructor must refuse");
    assert_eq!(
        (error.java_class(), error.message()),
        ("IllegalArgumentException", "requirement failed")
    );
}

// ---------------------------------------------------------------- Constructor

#[test]
fn constructor_does_not_accept_a_non_empty_backing_queue() {
    let mut full = VecDeque::new();
    full.push_back(s("Hello"));
    requirement_failed(BoundedBlockingQueue::new(10, full));
}

#[test]
fn constructor_accepts_an_empty_backing_queue() {
    let queue = BoundedBlockingQueue::<Arc<String>>::new(10, VecDeque::new()).expect("valid");
    assert_eq!((queue.len(), queue.max_capacity()), (0, 10));
}

#[test]
fn constructor_gets_a_positive_max_capacity() {
    requirement_failed(BoundedBlockingQueue::<Arc<String>>::new(0, VecDeque::new()));
    requirement_failed(BoundedBlockingQueue::<Arc<String>>::new(
        -3,
        VecDeque::new(),
    ));
}

#[test]
fn constructor_does_not_accept_a_backing_queue_with_lower_capacity() {
    requirement_failed(BoundedBlockingQueue::new(2, Limited(VecDeque::new(), 1)));
    assert!(BoundedBlockingQueue::new(1, Limited(VecDeque::new(), 2)).is_ok());
}

// ---------------------------------------------------------------- remainingCapacity

#[test]
fn remaining_capacity_checks_the_backing_queue_size() {
    let c = new_queue(1);
    c.backing_offer("1");
    assert_eq!(c.queue.remaining_capacity(), 0);
    assert!(c.has(&Ev::GetSize));
}

// ---------------------------------------------------------------- put

#[test]
fn put_calls_the_backing_queue_if_not_full() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("World"));
    assert!(c.has_in_order(&[offer("Hello"), offer("World")]));
}

#[test]
fn put_signals_not_empty_when_an_element_is_inserted() {
    let c = new_queue(1);
    c.queue.put(s("Hello"));
    assert!(c.has(&SIGNAL_NOT_EMPTY));
}

#[test]
fn put_blocks_when_the_queue_is_full() {
    let c = new_queue(1);
    c.queue.offer(s("1"));
    let done = Arc::new(AtomicBool::new(false));
    let producer = {
        let (queue, done) = (c.queue.clone(), done.clone());
        thread::spawn(move || {
            queue.put(s("2"));
            done.store(true, Ordering::SeqCst);
        })
    };
    c.wait_for(&AWAIT_NOT_FULL, 1);
    assert!(!done.load(Ordering::SeqCst));
    assert!(c.has_in_order(&[offer("1"), AWAIT_NOT_FULL]));
    assert!(!c.has(&offer("2")));
    c.queue.take();
    producer.join().expect("no panic");
}

#[test]
fn put_blocks_until_the_backing_queue_has_space() {
    let c = new_queue(1);
    c.queue.offer(s("a"));
    let producer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.put(s("b")))
    };
    c.wait_for(&AWAIT_NOT_FULL, 1);
    c.queue.take();
    producer.join().expect("no panic");
    assert!(c.has_in_order(&[offer("a"), Ev::Poll, offer("b")]));
}

#[test]
fn put_checks_the_backing_queue_size_before_offering() {
    let c = new_queue(1);
    c.queue.offer(s("a"));
    let producer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.put(s("b")))
    };
    c.wait_for(&AWAIT_NOT_FULL, 1);
    // a signal with no room: the producer wakes up, looks at the size, and waits again
    c.queue.signal_not_full();
    c.wait_for(&AWAIT_NOT_FULL, 2);
    assert!(c.has_in_order(&[
        offer("a"),
        AWAIT_NOT_FULL,
        SIGNAL_NOT_FULL,
        Ev::GetSize,
        AWAIT_NOT_FULL
    ]));
    assert!(!c.has(&offer("b")));
    c.queue.take();
    producer.join().expect("no panic");
}

// ---------------------------------------------------------------- take

#[test]
fn take_calls_the_backing_queue_if_not_empty_and_signals_not_full() {
    let c = new_queue(1);
    c.queue.put(s("Hello"));
    assert_eq!(*c.queue.take(), "Hello");
    assert!(c.has_in_order(&[offer("Hello"), Ev::Poll]));
    assert!(c.has(&SIGNAL_NOT_FULL));
}

#[test]
fn take_blocks_when_the_queue_is_empty() {
    let c = new_queue(1);
    let consumer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.take())
    };
    c.wait_for(&AWAIT_NOT_EMPTY, 1);
    assert!(!consumer.is_finished());
    assert!(!c.has(&Ev::Poll));
    c.queue.put(s("a"));
    assert_eq!(*consumer.join().expect("no panic"), "a");
    assert!(c.has_in_order(&[AWAIT_NOT_EMPTY, offer("a"), Ev::Poll]));
}

#[test]
fn take_checks_the_backing_queue_size_before_polling() {
    let c = new_queue(1);
    let consumer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.take())
    };
    c.wait_for(&AWAIT_NOT_EMPTY, 1);
    c.queue.signal_not_empty(); // a signal with nothing to take
    c.wait_for(&AWAIT_NOT_EMPTY, 2);
    assert!(c.has_in_order(&[
        Ev::GetSize,
        AWAIT_NOT_EMPTY,
        SIGNAL_NOT_EMPTY,
        Ev::GetSize,
        AWAIT_NOT_EMPTY
    ]));
    assert!(!c.has(&Ev::Poll));
    c.queue.put(s("x"));
    assert_eq!(*consumer.join().expect("no panic"), "x");
}

// ---------------------------------------------------------------- offer

#[test]
fn offer_does_not_block_if_the_queue_is_full() {
    let c = new_queue(1);
    assert!(c.queue.offer(s("Hello")));
    assert!(!c.queue.offer(s("World")));
    assert!(!c.has(&offer("World")));
}

#[test]
fn offer_calls_the_backing_queue_if_not_full_and_signals_not_empty() {
    let c = new_queue(2);
    assert!(c.queue.offer(s("Hello")));
    assert!(c.queue.offer(s("World")));
    assert!(c.has_in_order(&[offer("Hello"), offer("World")]));
    assert!(c.has(&SIGNAL_NOT_EMPTY));
}

// ---------------------------------------------------------------- offer with timeout

#[test]
fn offer_with_timeout_does_not_block_if_the_queue_is_not_full() {
    let c = new_queue(1);
    assert!(
        c.queue
            .offer_timeout(s("Hello"), Duration::from_millis(100))
    );
    assert!(c.has(&offer("Hello")));
    assert!(c.has(&SIGNAL_NOT_EMPTY));
}

#[test]
fn offer_with_timeout_blocks_for_at_least_the_timeout_if_the_queue_is_full() {
    let c = new_queue(1);
    c.queue.put(s("Hello"));
    let started = Instant::now();
    assert!(
        !c.queue
            .offer_timeout(s("World"), Duration::from_millis(100))
    );
    assert!(started.elapsed() >= Duration::from_millis(100));
    assert!(!c.has(&offer("World")));
}

#[test]
fn offer_with_timeout_blocks_for_less_than_the_timeout_when_the_queue_becomes_not_full() {
    let c = new_queue(1);
    c.queue.put(s("Hello"));
    let producer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.offer_timeout(s("World"), Duration::from_secs(3600)))
    };
    c.wait_for(&AWAIT_NOT_FULL, 1);
    c.queue.take();
    assert!(producer.join().expect("no panic"));
    assert!(c.has_in_order(&[AWAIT_NOT_FULL, SIGNAL_NOT_FULL, offer("World")]));
}

#[test]
fn offer_with_timeout_checks_the_backing_queue_size_before_offering() {
    let c = new_queue(1);
    c.queue.put(s("Hello"));
    let producer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.offer_timeout(s("World"), Duration::from_secs(3600)))
    };
    c.wait_for(&AWAIT_NOT_FULL, 1);
    c.queue.signal_not_full();
    c.wait_for(&AWAIT_NOT_FULL, 2);
    assert!(c.has_in_order(&[
        Ev::GetSize,
        AWAIT_NOT_FULL,
        SIGNAL_NOT_FULL,
        Ev::GetSize,
        AWAIT_NOT_FULL
    ]));
    assert!(!c.has(&offer("World")));
    c.queue.take();
    assert!(producer.join().expect("no panic"));
}

// ---------------------------------------------------------------- poll

#[test]
fn poll_returns_none_when_the_queue_is_empty_and_polls_the_backing_queue() {
    let c = new_queue(1);
    assert!(c.queue.poll().is_none());
    assert!(c.has(&Ev::Poll));
}

#[test]
fn poll_returns_the_first_element_inserted_and_signals_not_full() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("World"));
    assert_eq!(*c.queue.poll().expect("an element"), "Hello");
    assert!(c.has(&SIGNAL_NOT_FULL));
}

// ---------------------------------------------------------------- poll with timeout

#[test]
fn poll_with_timeout_does_not_block_if_the_queue_is_not_empty() {
    let c = new_queue(1);
    c.backing_offer("Hello");
    assert_eq!(
        *c.queue
            .poll_timeout(Duration::from_secs(86_400))
            .expect("an element"),
        "Hello"
    );
    assert!(c.has(&Ev::Poll));
    assert!(c.has(&SIGNAL_NOT_FULL));
}

#[test]
fn poll_with_timeout_blocks_for_at_least_the_timeout_if_the_queue_is_empty() {
    let c = new_queue(1);
    let started = Instant::now();
    assert!(c.queue.poll_timeout(Duration::from_millis(100)).is_none());
    assert!(started.elapsed() >= Duration::from_millis(100));
    assert!(c.has(&AWAIT_NOT_EMPTY));
    assert!(c.has(&Ev::Poll)); // it polls the backing queue immediately
}

#[test]
fn poll_with_timeout_blocks_for_less_than_the_timeout_when_the_queue_becomes_non_empty() {
    let c = new_queue(1);
    let consumer = {
        let queue = c.queue.clone();
        thread::spawn(move || queue.poll_timeout(Duration::from_secs(3600)))
    };
    c.wait_for(&AWAIT_NOT_EMPTY, 1);
    c.queue.put(s("Hello"));
    assert_eq!(
        *consumer.join().expect("no panic").expect("an element"),
        "Hello"
    );
    assert!(c.has_in_order(&[AWAIT_NOT_EMPTY, SIGNAL_NOT_EMPTY, Ev::Poll]));
}

// ---------------------------------------------------------------- remove, contains, clear, peek

#[test]
fn remove_returns_false_when_the_queue_is_empty_or_does_not_contain_the_element() {
    let c = new_queue(1);
    assert!(!c.queue.remove(&s("Hello")));
    c.queue.put(s("H3llo"));
    assert!(!c.queue.remove(&s("Hello")));
}

#[test]
fn remove_returns_true_if_the_element_was_removed_and_signals_not_full() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    assert!(c.queue.remove(&s("Hello")));
    assert!(c.has(&SIGNAL_NOT_FULL));
}

#[test]
fn contains_looks_at_the_backing_queue() {
    let c = new_queue(2);
    assert!(!c.queue.contains(&s("Hello")));
    c.backing_offer("H3llo");
    assert!(!c.queue.contains(&s("Hello")));
    c.backing_offer("Hello");
    assert!(c.queue.contains(&s("Hello")));
}

#[test]
fn clear_removes_all_elements_from_the_backing_queue() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.clear();
    assert_eq!(c.queue.lock().backing.snapshot().len(), 0);
    assert!(c.queue.is_empty());
}

#[test]
fn peek_returns_the_first_element_without_removing_it() {
    let c = new_queue(2);
    c.backing_offer("Hello");
    assert_eq!(*c.queue.peek().expect("an element"), "Hello");
    assert!(c.queue.contains(&s("Hello")));
}

// ---------------------------------------------------------------- drainTo

#[test]
fn drain_to_removes_all_elements_from_the_queue_and_adds_them_to_the_target() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("World"));
    let mut target = Vec::new();
    assert_eq!(c.queue.drain_to(&mut target, usize::MAX), 2);
    assert_eq!(c.queue.remaining_capacity(), 2);
    assert_eq!(
        target.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
        ["Hello", "World"]
    );
}

#[test]
fn drain_to_honours_the_maximum_and_signals_not_full() {
    let c = new_queue(3);
    for e in ["a", "b", "c"] {
        c.queue.put(s(e));
    }
    let mut target = Vec::new();
    assert_eq!(c.queue.drain_to(&mut target, 0), 0);
    assert_eq!(c.queue.drain_to(&mut target, 2), 2);
    assert!(c.has(&SIGNAL_NOT_FULL));
    assert_eq!(c.queue.len(), 1);
}

// ---------------------------------------------------------------- containsAll, removeAll, retainAll

#[test]
fn contains_all_is_true_only_when_every_element_is_there() {
    let c = new_queue(3);
    let elems = [s("Hello"), s("World")];
    for e in &elems {
        c.queue.put(e.clone());
    }
    assert!(!c.queue.contains_all(&[s("Pekko"), s("Hello"), s("World")]));
    assert!(c.queue.contains_all(&elems));
    c.queue.put(s("Pekko"));
    assert!(c.queue.contains_all(&elems));
}

#[test]
fn remove_all_removes_all_elements_from_the_argument_collection() {
    let c = new_queue(3);
    for e in ["Hello", "World", "Pekko"] {
        c.queue.put(s(e));
    }
    assert!(c.queue.remove_all(&[s("Hello"), s("World")]));
    assert_eq!(c.queue.remaining_capacity(), 2);
    assert_eq!(*c.queue.poll().expect("an element"), "Pekko");
}

#[test]
fn remove_all_returns_false_if_no_elements_were_removed() {
    let c = new_queue(1);
    c.queue.put(s("Pekko"));
    assert!(!c.queue.remove_all(&[s("Hello"), s("World")]));
    assert_eq!(*c.queue.poll().expect("an element"), "Pekko");
}

#[test]
fn retain_all_removes_all_elements_not_in_the_argument_collection() {
    let c = new_queue(3);
    let elems = [s("Hello"), s("World")];
    for e in &elems {
        c.queue.put(e.clone());
    }
    c.queue.put(s("Pekko"));
    assert!(c.queue.retain_all(&elems));
    assert_eq!(c.queue.remaining_capacity(), 1);
    let left = c.queue.to_vec();
    assert!(!left.iter().any(|e| e.as_str() == "Pekko"));
    assert_eq!(
        left.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
        ["Hello", "World"]
    );
    assert!(c.has(&SIGNAL_NOT_FULL) && c.has(&SIGNAL_NOT_EMPTY));
}

#[test]
fn retain_all_returns_false_if_no_elements_were_removed() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("World"));
    assert!(!c.queue.retain_all(&[s("Hello"), s("World"), s("Pekko")]));
    assert_eq!(c.queue.len(), 2);
}

// ---------------------------------------------------------------- iterator

#[test]
fn iterator_has_no_elements_if_the_queue_is_empty() {
    let c = new_queue(1);
    assert!(c.queue.iter().next().is_none());
}

#[test]
fn iterator_disallows_remove_before_calling_next() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("Pekko"));
    let mut iter = c.queue.iter();
    let error = iter.remove().expect_err("remove before next");
    assert_eq!(
        (error.java_class(), error.message()),
        ("IllegalStateException", None)
    );
}

#[test]
fn iterator_disallows_two_subsequent_removes() {
    let c = new_queue(2);
    c.queue.put(s("Hello"));
    c.queue.put(s("Pekko"));
    let mut iter = c.queue.iter();
    iter.next();
    iter.remove().expect("one remove after next");
    assert!(iter.remove().is_err());
    assert_eq!(c.queue.len(), 1);
}

#[test]
fn iterator_remove_removes_by_identity_and_signals_not_full() {
    let c = new_queue(3);
    let (first, second) = (s("a"), s("a"));
    c.queue.put(first.clone());
    c.queue.put(second.clone());
    let mut iter = c.queue.iter();
    iter.next();
    iter.next();
    iter.remove().expect("remove");
    let left = c.queue.to_vec();
    assert_eq!(left.len(), 1);
    assert!(Arc::ptr_eq(&left[0], &first));
    assert!(c.has(&SIGNAL_NOT_FULL));
}

#[test]
fn iterator_is_a_snapshot() {
    let c = new_queue(4);
    c.queue.put(s("a"));
    c.queue.put(s("b"));
    let iter = c.queue.iter();
    c.queue.put(s("c"));
    c.queue.poll();
    assert_eq!(iter.map(|e| (*e).clone()).collect::<Vec<_>>(), ["a", "b"]);
}

// ---------------------------------------------------------------- a priority backing queue and concurrency

#[test]
fn a_priority_backing_queue_orders_the_elements() {
    let backing = StablePriorityQueue::new(11, |a: &Arc<String>, b: &Arc<String>| a.cmp(b));
    let queue = BoundedBlockingQueue::new(4, backing).expect("valid");
    for e in ["d", "b", "c", "a"] {
        queue.put(s(e));
    }
    assert!(!queue.offer(s("z")));
    assert_eq!(*queue.poll().expect("an element"), "a");
    assert_eq!(*queue.peek().expect("an element"), "b");
    assert_eq!(queue.len(), 3);
}

#[test]
fn one_producer_and_one_consumer_keep_fifo_through_a_small_queue() {
    let queue = Arc::new(BoundedBlockingQueue::new(3, VecDeque::new()).expect("valid"));
    let count = 2000_u32;
    let producer = {
        let queue = queue.clone();
        thread::spawn(move || {
            for i in 0..count {
                queue.put(Arc::new(i));
            }
        })
    };
    let consumer = {
        let queue = queue.clone();
        thread::spawn(move || {
            (0..count)
                .map(|_| {
                    assert!(queue.len() <= 3);
                    *queue.take()
                })
                .collect::<Vec<_>>()
        })
    };
    producer.join().expect("no panic");
    assert_eq!(
        consumer.join().expect("no panic"),
        (0..count).collect::<Vec<_>>()
    );
    assert!(queue.is_empty());
}

#[test]
fn several_producers_and_consumers_lose_and_duplicate_nothing() {
    let queue = Arc::new(BoundedBlockingQueue::new(5, VecDeque::new()).expect("valid"));
    let (parties, each) = (4_u32, 500_u32);
    let producers: Vec<_> = (0..parties)
        .map(|p| {
            let queue = queue.clone();
            thread::spawn(move || {
                for i in 0..each {
                    queue.put(Arc::new(p * 1_000_000 + i));
                }
            })
        })
        .collect();
    let consumers: Vec<_> = (0..parties)
        .map(|_| {
            let queue = queue.clone();
            thread::spawn(move || (0..each).map(|_| *queue.take()).collect::<Vec<_>>())
        })
        .collect();
    for producer in producers {
        producer.join().expect("no panic");
    }
    let mut all = Vec::new();
    for consumer in consumers {
        let mine = consumer.join().expect("no panic");
        let mut last = [None::<u32>; 4];
        for v in &mine {
            let (p, i) = ((v / 1_000_000) as usize, v % 1_000_000);
            assert!(
                last[p].is_none_or(|previous| i > previous),
                "order of one producer"
            );
            last[p] = Some(i);
        }
        all.extend(mine);
    }
    all.sort_unstable();
    let mut expected: Vec<u32> = (0..parties)
        .flat_map(|p| (0..each).map(move |i| p * 1_000_000 + i))
        .collect();
    expected.sort_unstable();
    assert_eq!(all, expected);
}

// A timeout too large for an `Instant` (Java saturates it to Long.MAX_VALUE nanoseconds) means "no deadline":
// when the call can finish at once it must finish, not panic on the deadline arithmetic.

#[test]
fn poll_with_an_unrepresentable_timeout_returns_the_head_at_once() {
    let context = new_queue(1);
    context.queue.put(s("head"));
    assert_eq!(context.queue.poll_timeout(Duration::MAX), Some(s("head")));
}

#[test]
fn offer_with_an_unrepresentable_timeout_succeeds_at_once_when_there_is_room() {
    let context = new_queue(1);
    assert!(context.queue.offer_timeout(s("a"), Duration::MAX));
    assert_eq!(context.queue.to_vec(), vec![s("a")]);
}
