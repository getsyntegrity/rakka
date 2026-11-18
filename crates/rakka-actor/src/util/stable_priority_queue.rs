// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/StablePriorityQueue.scala (Apache-2.0).
//! `StablePriorityQueue` and `StablePriorityBlockingQueue` (AC-UTIL-013): priority queues that keep
//! first-in first-out order between elements of equal priority, by numbering every element that goes
//! in and breaking ties with the number.
//!
//! The element with the lowest priority according to the comparator comes out first, like Java's
//! `PriorityQueue`. The order of [`StablePriorityQueue::iter`] is the order of the heap, which is not
//! specified.

use std::cmp::Ordering;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

type Order<E> = Box<dyn Fn(&E, &E) -> Ordering + Send + Sync>;

struct Wrapped<E> {
    element: E,
    seq_num: u64,
}

/// `StablePriorityQueue[E]`: not thread safe.
pub struct StablePriorityQueue<E> {
    order: Order<E>,
    heap: Vec<Wrapped<E>>,
    seq_num: u64,
}

impl<E> StablePriorityQueue<E> {
    /// `new StablePriorityQueue(capacity, cmp)`: `capacity` is the initial capacity.
    pub fn new(capacity: usize, cmp: impl Fn(&E, &E) -> Ordering + Send + Sync + 'static) -> Self {
        Self {
            order: Box::new(cmp),
            heap: Vec::with_capacity(capacity),
            seq_num: 0,
        }
    }

    /// `WrappedElementComparator`: the comparator, then the sequence number.
    fn before(&self, a: usize, b: usize) -> bool {
        let (a, b) = (&self.heap[a], &self.heap[b]);
        match (self.order)(&a.element, &b.element) {
            Ordering::Equal => (a.seq_num.wrapping_sub(b.seq_num) as i64) < 0,
            other => other == Ordering::Less,
        }
    }

    fn sift_up(&mut self, mut at: usize) {
        while at > 0 {
            let parent = (at - 1) / 2;
            if !self.before(at, parent) {
                break;
            }
            self.heap.swap(at, parent);
            at = parent;
        }
    }

    fn sift_down(&mut self, mut at: usize) {
        loop {
            let (left, right) = (2 * at + 1, 2 * at + 2);
            let mut smallest = at;
            if left < self.heap.len() && self.before(left, smallest) {
                smallest = left;
            }
            if right < self.heap.len() && self.before(right, smallest) {
                smallest = right;
            }
            if smallest == at {
                break;
            }
            self.heap.swap(at, smallest);
            at = smallest;
        }
    }

    /// `offer(e)`: adds the element; always true.
    pub fn offer(&mut self, element: E) -> bool {
        self.seq_num += 1;
        self.heap.push(Wrapped {
            element,
            seq_num: self.seq_num,
        });
        self.sift_up(self.heap.len() - 1);
        true
    }

    /// `poll()`: removes and returns the head of the queue.
    pub fn poll(&mut self) -> Option<E> {
        if self.heap.is_empty() {
            return None;
        }
        let last = self.heap.len() - 1;
        self.heap.swap(0, last);
        let head = self.heap.pop().map(|wrapped| wrapped.element);
        self.sift_down(0);
        head
    }

    /// `peek()`: the head of the queue, left in place.
    pub fn peek(&self) -> Option<&E> {
        self.heap.first().map(|wrapped| &wrapped.element)
    }

    /// `size()`.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// `iterator()`: the elements in the order of the heap (not the order they come out in).
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.heap.iter().map(|wrapped| &wrapped.element)
    }

    /// Removes the elements for which `keep` is false (`iterator().remove()` and `removeAll`).
    /// Returns whether any went.
    pub fn retain(&mut self, mut keep: impl FnMut(&E) -> bool) -> bool {
        let before = self.heap.len();
        let kept: Vec<Wrapped<E>> = std::mem::take(&mut self.heap)
            .into_iter()
            .filter(|wrapped| keep(&wrapped.element))
            .collect();
        for wrapped in kept {
            self.heap.push(wrapped);
            self.sift_up(self.heap.len() - 1);
        }
        self.heap.len() != before
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}

/// `StablePriorityBlockingQueue[E]`: the thread safe, blocking variant (it has no capacity).
pub struct StablePriorityBlockingQueue<E> {
    queue: Mutex<StablePriorityQueue<E>>,
    not_empty: Condvar,
}

impl<E> StablePriorityBlockingQueue<E> {
    /// `new StablePriorityBlockingQueue(capacity, cmp)`.
    pub fn new(capacity: usize, cmp: impl Fn(&E, &E) -> Ordering + Send + Sync + 'static) -> Self {
        Self {
            queue: Mutex::new(StablePriorityQueue::new(capacity, cmp)),
            not_empty: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, StablePriorityQueue<E>> {
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `offer(e)`: always true.
    pub fn offer(&self, element: E) -> bool {
        let accepted = self.lock().offer(element);
        self.not_empty.notify_one();
        accepted
    }

    /// `poll()`.
    pub fn poll(&self) -> Option<E> {
        self.lock().poll()
    }

    /// `take()`: blocks until there is an element.
    pub fn take(&self) -> E {
        let mut queue = self.lock();
        loop {
            if let Some(head) = queue.poll() {
                return head;
            }
            queue = self
                .not_empty
                .wait(queue)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// `poll(timeout)`: waits up to `timeout` for an element.
    pub fn poll_timeout(&self, timeout: Duration) -> Option<E> {
        let deadline = Instant::now() + timeout;
        let mut queue = self.lock();
        loop {
            if let Some(head) = queue.poll() {
                return Some(head);
            }
            let remaining = deadline.checked_duration_since(Instant::now())?;
            queue = self
                .not_empty
                .wait_timeout(queue, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }

    /// `size()`.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }
}

#[cfg(test)]
mod tests {
    //! The one assertion of Pekko 2.0.0 about these queues (`StablePriorityDispatcherSpec`) goes through
    //! a mailbox; here the queue alone gets the same input: 200 shuffled integers, small ones with their
    //! own priority and the rest sharing priority 101.
    use super::*;
    use std::sync::Arc;

    fn priority(i: &u32) -> u32 {
        if *i <= 100 { *i } else { 101 }
    }

    fn shuffled(count: u32) -> Vec<u32> {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut messages: Vec<u32> = (1..=count).collect();
        for i in (1..messages.len()).rev() {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            let j = ((state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) % (i as u64 + 1)) as usize;
            messages.swap(i, j);
        }
        messages
    }

    #[test]
    fn order_by_priority_and_keep_fifo_among_equals() {
        let messages = shuffled(200);
        let mut queue =
            StablePriorityQueue::new(11, |a: &u32, b: &u32| priority(a).cmp(&priority(b)));
        for m in &messages {
            assert!(queue.offer(*m));
        }
        let mut out = Vec::new();
        while let Some(m) = queue.poll() {
            out.push(m);
        }
        let low: Vec<u32> = (1..=100).collect();
        let high: Vec<u32> = messages.iter().copied().filter(|m| *m > 100).collect();
        assert_eq!(out, [low, high].concat());
    }

    #[test]
    fn peek_size_iteration_and_retain() {
        let mut queue =
            StablePriorityQueue::new(4, |a: &(u32, char), b: &(u32, char)| a.0.cmp(&b.0));
        assert!(queue.is_empty() && queue.peek().is_none() && queue.poll().is_none());
        for item in [(2, 'x'), (1, 'a'), (2, 'y'), (1, 'b')] {
            queue.offer(item);
        }
        assert_eq!((queue.len(), queue.peek()), (4, Some(&(1, 'a'))));
        assert_eq!(queue.iter().count(), 4);
        assert!(queue.retain(|e| e.1 != 'a'));
        assert!(!queue.retain(|_| true));
        assert_eq!(queue.poll(), Some((1, 'b')));
        assert_eq!(queue.poll(), Some((2, 'x')));
        assert_eq!(queue.poll(), Some((2, 'y')));
        queue.offer((5, 'z'));
        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn the_blocking_queue_orders_the_same_way_and_blocks_take() {
        let queue = Arc::new(StablePriorityBlockingQueue::new(
            11,
            |a: &(u32, &str), b: &(u32, &str)| a.0.cmp(&b.0),
        ));
        assert!(queue.is_empty() && queue.poll().is_none());
        assert_eq!(queue.poll_timeout(Duration::from_millis(1)), None);
        let taker = {
            let queue = queue.clone();
            std::thread::spawn(move || queue.take())
        };
        queue.offer((3, "x1"));
        assert_eq!(taker.join().expect("no panic"), (3, "x1"));
        for item in [(3, "x1"), (1, "y1"), (3, "x2"), (1, "y2"), (2, "z1")] {
            queue.offer(item);
        }
        assert_eq!(queue.len(), 5);
        let order: Vec<_> = (0..5)
            .filter_map(|_| queue.poll_timeout(Duration::from_secs(5)))
            .collect();
        assert_eq!(
            order,
            vec![(1, "y1"), (1, "y2"), (2, "z1"), (3, "x1"), (3, "x2")]
        );
    }
}
