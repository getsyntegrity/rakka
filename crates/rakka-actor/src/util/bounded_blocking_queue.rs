// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/BoundedBlockingQueue.scala (Apache-2.0).
//! `BoundedBlockingQueue` (AC-UTIL-012): turns any queue into a blocking queue with a maximum
//! capacity. Used by the bounded mailboxes with priority.
//!
//! Q-UTIL-10 (a): one `Mutex` and two `Condvar`s (`notEmpty`, `notFull`) over a backing queue given as
//! a [`BackingQueue`]. Differences, all consequences of the language:
//! - There is no `null`: Pekko's rejections of a null backing queue or element do not exist.
//! - A thread cannot be interrupted: `lockInterruptibly`/`InterruptedException` have no counterpart.
//! - `drainTo` takes `&mut Vec<E>`, which can be neither the queue nor its backing queue, so those two
//!   `IllegalArgumentException`s cannot happen.
//! - Identity (the iterator's `remove` removes the element it returned, not an equal one) is
//!   [`QueueElement::same_instance`]; for `Arc<T>` it is the address of the allocation (Q-UTIL-10).
//! - Timeouts are `Duration`s (never negative); the wait in a timed call really waits for the time.
//!
//! [`QueueEvent`]s let a test see which lock operations happened, as Pekko's spec does with its
//! instrumented conditions.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::stable_priority_queue::StablePriorityQueue;
use crate::actor::IllegalArgumentError;

/// The queue a [`BoundedBlockingQueue`] wraps (a `java.util.Queue`).
pub trait BackingQueue<E>: Send {
    /// `offer`: false if the queue refuses the element.
    fn offer(&mut self, element: E) -> bool;
    /// `poll`.
    fn poll(&mut self) -> Option<E>;
    /// `peek`.
    fn peek(&self) -> Option<&E>;
    /// `size`.
    fn len(&self) -> usize;
    /// `isEmpty`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// `remainingCapacity` of a `BlockingQueue` backing queue; a plain queue has no limit.
    fn remaining_capacity(&self) -> usize {
        usize::MAX
    }
    /// Removes the first element (in iteration order) for which `matches` holds; whether one went.
    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&E) -> bool) -> bool;
    /// Keeps only the elements for which `keep` holds; whether any went.
    fn retain_where(&mut self, keep: &mut dyn FnMut(&E) -> bool) -> bool;
    /// `clear`.
    fn clear(&mut self);
    /// The elements in iteration order (`toArray`).
    fn snapshot(&self) -> Vec<E>
    where
        E: Clone;
}

impl<E: Send> BackingQueue<E> for VecDeque<E> {
    fn offer(&mut self, element: E) -> bool {
        self.push_back(element);
        true
    }

    fn poll(&mut self) -> Option<E> {
        self.pop_front()
    }

    fn peek(&self) -> Option<&E> {
        self.front()
    }

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&E) -> bool) -> bool {
        match self.iter().position(&mut *matches) {
            Some(at) => self.remove(at).is_some(),
            None => false,
        }
    }

    fn retain_where(&mut self, keep: &mut dyn FnMut(&E) -> bool) -> bool {
        let before = VecDeque::len(self);
        self.retain(|e| keep(e));
        VecDeque::len(self) != before
    }

    fn clear(&mut self) {
        VecDeque::clear(self);
    }

    fn snapshot(&self) -> Vec<E>
    where
        E: Clone,
    {
        self.iter().cloned().collect()
    }
}

impl<E: Send> BackingQueue<E> for StablePriorityQueue<E> {
    fn offer(&mut self, element: E) -> bool {
        StablePriorityQueue::offer(self, element)
    }

    fn poll(&mut self) -> Option<E> {
        StablePriorityQueue::poll(self)
    }

    fn peek(&self) -> Option<&E> {
        StablePriorityQueue::peek(self)
    }

    fn len(&self) -> usize {
        StablePriorityQueue::len(self)
    }

    fn remove_first_where(&mut self, matches: &mut dyn FnMut(&E) -> bool) -> bool {
        let mut removed = false;
        self.retain(|e| {
            if !removed && matches(e) {
                removed = true;
                false
            } else {
                true
            }
        });
        removed
    }

    fn retain_where(&mut self, keep: &mut dyn FnMut(&E) -> bool) -> bool {
        self.retain(|e| keep(e))
    }

    fn clear(&mut self) {
        StablePriorityQueue::clear(self);
    }

    fn snapshot(&self) -> Vec<E>
    where
        E: Clone,
    {
        self.iter().cloned().collect()
    }
}

/// An element with an identity: two handles to the same element are the same instance.
pub trait QueueElement: Clone + Send {
    /// Pekko's `eq` (reference equality).
    fn same_instance(&self, other: &Self) -> bool;
}

impl<T: Send + Sync> QueueElement for Arc<T> {
    fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

/// The lock operations of a [`BoundedBlockingQueue`], for tests to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueEvent {
    AwaitNotEmpty,
    AwaitNotFull,
    SignalNotEmpty,
    SignalNotFull,
}

type Listener = Arc<dyn Fn(QueueEvent) + Send + Sync>;

/// Time left before `deadline`, `None` once it has passed. `Some(None)` means there is no deadline: Java saturates a
/// timeout past `Long.MAX_VALUE` nanoseconds and waits for as long as it takes, and `Instant` cannot hold such a point.
pub(crate) fn remaining_until(deadline: Option<Instant>) -> Option<Option<Duration>> {
    match deadline {
        None => Some(None),
        Some(deadline) => deadline.checked_duration_since(Instant::now()).map(Some),
    }
}

/// Parks on `condvar` for at most `remaining` (for as long as it takes when `None`) and hands the guard back.
pub(crate) fn wait_up_to<'a, T>(
    condvar: &Condvar,
    guard: MutexGuard<'a, T>,
    remaining: Option<Duration>,
) -> MutexGuard<'a, T> {
    match remaining {
        Some(remaining) => {
            condvar
                .wait_timeout(guard, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0
        }
        None => condvar.wait(guard).unwrap_or_else(PoisonError::into_inner),
    }
}

struct Inner<E> {
    backing: Box<dyn BackingQueue<E>>,
}

/// `BoundedBlockingQueue[E]`.
pub struct BoundedBlockingQueue<E> {
    max_capacity: usize,
    inner: Mutex<Inner<E>>,
    not_empty: Condvar,
    not_full: Condvar,
    listener: Option<Listener>,
}

const REQUIREMENT_FAILED: &str = "requirement failed";

impl<E: QueueElement + 'static> BoundedBlockingQueue<E> {
    /// `new BoundedBlockingQueue(maxCapacity, backing)`: the backing queue must be empty, `max_capacity`
    /// positive and the backing queue able to hold that many elements (`IllegalArgumentException`).
    pub fn new(
        max_capacity: i32,
        backing: impl BackingQueue<E> + 'static,
    ) -> Result<Self, IllegalArgumentError> {
        Self::with_listener(max_capacity, backing, None)
    }

    /// As [`BoundedBlockingQueue::new`], reporting every lock operation to `listener` (a test seam).
    #[doc(hidden)]
    pub fn with_listener(
        max_capacity: i32,
        backing: impl BackingQueue<E> + 'static,
        listener: Option<Listener>,
    ) -> Result<Self, IllegalArgumentError> {
        let max_capacity = usize::try_from(max_capacity)
            .ok()
            .filter(|capacity| *capacity > 0)
            .ok_or_else(|| IllegalArgumentError::new(REQUIREMENT_FAILED))?;
        if !backing.is_empty() || backing.remaining_capacity() < max_capacity {
            return Err(IllegalArgumentError::new(REQUIREMENT_FAILED));
        }
        Ok(Self {
            max_capacity,
            inner: Mutex::new(Inner {
                backing: Box::new(backing),
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            listener,
        })
    }

    /// `maxCapacity`.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    fn lock(&self) -> MutexGuard<'_, Inner<E>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn emit(&self, event: QueueEvent) {
        if let Some(listener) = &self.listener {
            listener(event);
        }
    }

    fn signal_not_empty(&self) {
        self.emit(QueueEvent::SignalNotEmpty);
        self.not_empty.notify_one();
    }

    fn signal_not_full(&self) {
        self.emit(QueueEvent::SignalNotFull);
        self.not_full.notify_one();
    }

    fn await_not_empty<'a>(&self, guard: MutexGuard<'a, Inner<E>>) -> MutexGuard<'a, Inner<E>> {
        self.emit(QueueEvent::AwaitNotEmpty);
        self.not_empty
            .wait(guard)
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn await_not_full<'a>(&self, guard: MutexGuard<'a, Inner<E>>) -> MutexGuard<'a, Inner<E>> {
        self.emit(QueueEvent::AwaitNotFull);
        self.not_full
            .wait(guard)
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// `put(e)`: blocks until the queue is not full.
    pub fn put(&self, element: E) {
        let mut guard = self.lock();
        while guard.backing.len() >= self.max_capacity {
            guard = self.await_not_full(guard);
        }
        assert!(
            guard.backing.offer(element),
            "the backing queue refused an element"
        );
        self.signal_not_empty();
    }

    /// `take()`: blocks until the queue is not empty.
    pub fn take(&self) -> E {
        let mut guard = self.lock();
        loop {
            if !guard.backing.is_empty() {
                let element = guard
                    .backing
                    .poll()
                    .expect("the backing queue is not empty");
                self.signal_not_full();
                return element;
            }
            guard = self.await_not_empty(guard);
        }
    }

    /// `offer(e)`: adds the element if there is room right away, false otherwise.
    pub fn offer(&self, element: E) -> bool {
        let mut guard = self.lock();
        if guard.backing.len() == self.max_capacity {
            false
        } else {
            assert!(
                guard.backing.offer(element),
                "the backing queue refused an element"
            );
            self.signal_not_empty();
            true
        }
    }

    /// `offer(e, timeout, unit)`: adds the element, waiting up to `timeout` for room; false on timeout.
    pub fn offer_timeout(&self, element: E, timeout: Duration) -> bool {
        let deadline = Instant::now().checked_add(timeout);
        let mut guard = self.lock();
        loop {
            if guard.backing.len() < self.max_capacity {
                assert!(
                    guard.backing.offer(element),
                    "the backing queue refused an element"
                );
                self.signal_not_empty();
                return true;
            }
            let Some(remaining) = remaining_until(deadline) else {
                return false;
            };
            self.emit(QueueEvent::AwaitNotFull);
            guard = wait_up_to(&self.not_full, guard, remaining);
        }
    }

    /// `poll(timeout, unit)`: removes the head, waiting up to `timeout` for one; `None` on timeout.
    pub fn poll_timeout(&self, timeout: Duration) -> Option<E> {
        let deadline = Instant::now().checked_add(timeout);
        let mut guard = self.lock();
        loop {
            if let Some(element) = guard.backing.poll() {
                self.signal_not_full();
                return Some(element);
            }
            let remaining = remaining_until(deadline)?;
            self.emit(QueueEvent::AwaitNotEmpty);
            guard = wait_up_to(&self.not_empty, guard, remaining);
        }
    }

    /// `poll()`: removes the head right away, `None` if the queue is empty.
    pub fn poll(&self) -> Option<E> {
        let mut guard = self.lock();
        let element = guard.backing.poll()?;
        self.signal_not_full();
        Some(element)
    }

    /// `remainingCapacity()`.
    pub fn remaining_capacity(&self) -> usize {
        self.max_capacity - self.lock().backing.len()
    }

    /// `size()`.
    pub fn len(&self) -> usize {
        self.lock().backing.len()
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.lock().backing.is_empty()
    }

    /// `peek()`: the head, left in place.
    pub fn peek(&self) -> Option<E> {
        self.lock().backing.peek().cloned()
    }

    /// `clear()`.
    pub fn clear(&self) {
        let mut guard = self.lock();
        guard.backing.clear();
        self.emit(QueueEvent::SignalNotFull);
        self.not_full.notify_all();
    }

    /// `drainTo(c, maxElements)`: moves up to `max_elements` elements into `sink`, returns how many.
    pub fn drain_to(&self, sink: &mut Vec<E>, max_elements: usize) -> usize {
        if max_elements == 0 {
            return 0;
        }
        let mut guard = self.lock();
        let mut drained = 0;
        while drained < max_elements {
            let Some(element) = guard.backing.poll() else {
                break;
            };
            sink.push(element);
            drained += 1;
        }
        if drained > 0 {
            self.emit(QueueEvent::SignalNotFull);
            self.not_full.notify_all();
        }
        drained
    }

    /// `toArray()`: the elements in the order of the backing queue.
    pub fn to_vec(&self) -> Vec<E> {
        self.lock().backing.snapshot()
    }

    /// After a bulk removal: wakes a producer if there is room and a consumer if there is anything.
    fn signal_after_bulk_removal(&self, remaining: usize) {
        if remaining < self.max_capacity {
            self.signal_not_full();
        }
        if remaining > 0 {
            self.signal_not_empty();
        }
    }

    /// `iterator()`: a snapshot of the elements, whose `remove` takes out the element last returned.
    pub fn iter(&self) -> QueueIterator<'_, E> {
        QueueIterator {
            queue: self,
            elements: self.to_vec(),
            at: 0,
            last: None,
        }
    }
}

impl<E: QueueElement + PartialEq + 'static> BoundedBlockingQueue<E> {
    /// `remove(e)`: removes one element equal to `element`, right away; whether one went.
    pub fn remove(&self, element: &E) -> bool {
        let mut guard = self.lock();
        if guard.backing.remove_first_where(&mut |e| e == element) {
            self.signal_not_full();
            true
        } else {
            false
        }
    }

    /// `contains(e)`.
    pub fn contains(&self, element: &E) -> bool {
        self.lock().backing.snapshot().contains(element)
    }

    /// `containsAll(c)`.
    pub fn contains_all(&self, elements: &[E]) -> bool {
        let present = self.lock().backing.snapshot();
        elements.iter().all(|e| present.contains(e))
    }

    /// `removeAll(c)`: removes every element equal to one of `elements`; whether any went.
    pub fn remove_all(&self, elements: &[E]) -> bool {
        let mut guard = self.lock();
        if guard.backing.retain_where(&mut |e| !elements.contains(e)) {
            let remaining = guard.backing.len();
            self.signal_after_bulk_removal(remaining);
            true
        } else {
            false
        }
    }

    /// `retainAll(c)`: removes every element not equal to one of `elements`; whether any went.
    pub fn retain_all(&self, elements: &[E]) -> bool {
        let mut guard = self.lock();
        if guard.backing.retain_where(&mut |e| elements.contains(e)) {
            let remaining = guard.backing.len();
            self.signal_after_bulk_removal(remaining);
            true
        } else {
            false
        }
    }
}

/// `java.lang.IllegalStateException` without a message: `remove()` before `next()`, or twice in a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IteratorRemoveError;

impl IteratorRemoveError {
    /// Simple name of the Java exception class Pekko throws.
    pub fn java_class(&self) -> &'static str {
        "IllegalStateException"
    }

    /// `getMessage`: there is none.
    pub fn message(&self) -> Option<&str> {
        None
    }
}

impl std::fmt::Display for IteratorRemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the iterator has no element to remove")
    }
}

impl std::error::Error for IteratorRemoveError {}

/// The iterator of a [`BoundedBlockingQueue`]: a snapshot taken when it was created.
pub struct QueueIterator<'a, E> {
    queue: &'a BoundedBlockingQueue<E>,
    elements: Vec<E>,
    at: usize,
    last: Option<usize>,
}

impl<E: QueueElement + 'static> QueueIterator<'_, E> {
    /// `remove()`: removes from the queue the element `next` returned last (by identity), if it is
    /// still there. `IllegalStateException` before `next`, or twice after one `next`.
    pub fn remove(&mut self) -> Result<(), IteratorRemoveError> {
        let last = self.last.take().ok_or(IteratorRemoveError)?;
        let target = &self.elements[last];
        let mut guard = self.queue.lock();
        if guard
            .backing
            .remove_first_where(&mut |e| e.same_instance(target))
        {
            self.queue.signal_not_full();
        }
        Ok(())
    }
}

impl<E: Clone> Iterator for QueueIterator<'_, E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        let element = self.elements.get(self.at)?.clone();
        self.last = Some(self.at);
        self.at += 1;
        Some(element)
    }
}

#[cfg(test)]
mod tests;
