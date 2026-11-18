// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/RecencyList.scala (Apache-2.0).
//! `RecencyList` (AC-UTIL-017.3): tracks the recency of elements to implement least/most recently used
//! eviction policies. Mutable and not thread safe; a doubly-linked list plus a hash map, so every
//! operation is constant time.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use super::clock::{Clock, NanoClock};
use super::double_linked_list::{DoubleLinkedList, LinkTable, Slab};

/// Nanoseconds of a duration as a `FiniteDuration` holds them, saturated.
pub(crate) fn duration_nanos(duration: Duration) -> i64 {
    i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX)
}

struct Node<A> {
    value: A,
    timestamp: i64,
}

/// `RecencyList[A]`.
pub struct RecencyList<A, C> {
    clock: C,
    recency: DoubleLinkedList<usize>,
    links: LinkTable,
    nodes: Slab<Node<A>>,
    lookup_node: HashMap<A, usize>,
}

impl<A: Clone + Eq + Hash> RecencyList<A, NanoClock> {
    /// `RecencyList.emptyWithNanoClock`.
    pub fn empty_with_nano_clock() -> Self {
        Self::new(NanoClock)
    }
}

impl<A: Clone + Eq + Hash, C: Clock> RecencyList<A, C> {
    /// `RecencyList(clock)`: an empty list.
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            recency: DoubleLinkedList::new(),
            links: LinkTable::new(),
            nodes: Slab::default(),
            lookup_node: HashMap::new(),
        }
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.lookup_node.len()
    }

    /// `update(value)`: makes the value the most recent, adding it if it is new.
    pub fn update(&mut self, value: A) -> &mut Self {
        let now = self.clock.current_time();
        if let Some(&node) = self.lookup_node.get(&value) {
            self.nodes.get_mut(node).timestamp = now;
            self.recency.move_to_back(&mut self.links, node);
        } else {
            let node = self.nodes.insert(Node {
                value: value.clone(),
                timestamp: now,
            });
            self.recency.append(&mut self.links, node);
            self.lookup_node.insert(value, node);
        }
        self
    }

    /// `remove(value)`.
    pub fn remove(&mut self, value: &A) -> &mut Self {
        if let Some(&node) = self.lookup_node.get(value) {
            self.remove_node(node);
        }
        self
    }

    /// `contains(value)`.
    pub fn contains(&self, value: &A) -> bool {
        self.lookup_node.contains_key(value)
    }

    /// `leastRecent`.
    pub fn least_recent(&self) -> Option<&A> {
        self.recency
            .first()
            .map(|node| &self.nodes.get(*node).value)
    }

    /// `mostRecent`.
    pub fn most_recent(&self) -> Option<&A> {
        self.recency.last().map(|node| &self.nodes.get(*node).value)
    }

    /// `leastToMostRecent`.
    pub fn least_to_most_recent(&self) -> impl Iterator<Item = &A> {
        self.recency
            .forward_iterator(&self.links)
            .map(|node| &self.nodes.get(node).value)
    }

    /// `mostToLeastRecent`.
    pub fn most_to_least_recent(&self) -> impl Iterator<Item = &A> {
        self.recency
            .backward_iterator(&self.links)
            .map(|node| &self.nodes.get(node).value)
    }

    fn remove_nodes(&mut self, nodes: Vec<usize>) -> Vec<A> {
        nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect()
    }

    /// `removeLeastRecent(n)`: removes and returns the `n` least recent values, least recent first.
    pub fn remove_least_recent_n(&mut self, n: usize) -> Vec<A> {
        let nodes: Vec<usize> = self.recency.forward_iterator(&self.links).take(n).collect();
        self.remove_nodes(nodes)
    }

    /// `removeLeastRecent(n, skip)`: the same after skipping the `skip` least recent ones, which stay.
    pub fn remove_least_recent_skipping(&mut self, n: usize, skip: usize) -> Vec<A> {
        let nodes: Vec<usize> = self
            .recency
            .forward_iterator(&self.links)
            .skip(skip)
            .take(n)
            .collect();
        self.remove_nodes(nodes)
    }

    /// `removeLeastRecent()`: the least recent value, if any.
    pub fn remove_least_recent(&mut self) -> Vec<A> {
        match self.recency.first().copied() {
            Some(first) => vec![self.remove_node(first)],
            None => Vec::new(),
        }
    }

    /// `removeMostRecent(n)`: removes and returns the `n` most recent values, most recent first.
    pub fn remove_most_recent_n(&mut self, n: usize) -> Vec<A> {
        let nodes: Vec<usize> = self
            .recency
            .backward_iterator(&self.links)
            .take(n)
            .collect();
        self.remove_nodes(nodes)
    }

    /// `removeMostRecent(n, skip)`.
    pub fn remove_most_recent_skipping(&mut self, n: usize, skip: usize) -> Vec<A> {
        let nodes: Vec<usize> = self
            .recency
            .backward_iterator(&self.links)
            .skip(skip)
            .take(n)
            .collect();
        self.remove_nodes(nodes)
    }

    /// `removeMostRecent()`.
    pub fn remove_most_recent(&mut self) -> Vec<A> {
        match self.recency.last().copied() {
            Some(last) => vec![self.remove_node(last)],
            None => Vec::new(),
        }
    }

    /// `removeLeastRecentOutside(duration)`: removes the least recent values that are older than `duration`.
    pub fn remove_least_recent_outside(&mut self, duration: Duration) -> Vec<A> {
        let min = self.clock.earlier_time(duration_nanos(duration));
        let nodes: Vec<usize> = self
            .recency
            .forward_iterator(&self.links)
            .take_while(|node| self.nodes.get(*node).timestamp < min)
            .collect();
        self.remove_nodes(nodes)
    }

    /// `removeMostRecentWithin(duration)`: removes the most recent values that are within `duration`.
    pub fn remove_most_recent_within(&mut self, duration: Duration) -> Vec<A> {
        let max = self.clock.earlier_time(duration_nanos(duration));
        let nodes: Vec<usize> = self
            .recency
            .backward_iterator(&self.links)
            .take_while(|node| self.nodes.get(*node).timestamp > max)
            .collect();
        self.remove_nodes(nodes)
    }

    fn remove_node(&mut self, node: usize) -> A {
        self.recency.remove(&mut self.links, &node);
        let removed = self.nodes.remove(node);
        self.lookup_node.remove(&removed.value);
        removed.value
    }
}

#[cfg(test)]
mod tests {
    //! `RecencyListSpec` of Pekko 2.0.0 (actor-tests/.../util/RecencyListSpec.scala), with its `TestClock`.
    use super::*;
    use crate::util::test_clock::TestClock;

    fn check(list: &RecencyList<&'static str, TestClock>, least_to_most_recent: &[&str]) {
        assert!(least_to_most_recent.iter().all(|v| list.contains(v)));
        assert_eq!(list.size(), least_to_most_recent.len());
        assert_eq!(
            list.least_to_most_recent().copied().collect::<Vec<_>>(),
            least_to_most_recent
        );
        let mut reversed = least_to_most_recent.to_vec();
        reversed.reverse();
        assert_eq!(
            list.most_to_least_recent().copied().collect::<Vec<_>>(),
            reversed
        );
    }

    fn update(list: &mut RecencyList<&'static str, TestClock>, values: &[&'static str]) {
        for value in values {
            list.update(value);
        }
    }

    #[test]
    fn track_recency_of_elements() {
        let clock = TestClock::default();
        let mut recency = RecencyList::new(clock.clone());
        let seconds = Duration::from_secs(3);

        check(&recency, &[]);
        clock.tick(); // time = 1
        update(&mut recency, &["a"]);
        check(&recency, &["a"]);
        clock.tick();
        update(&mut recency, &["b", "c"]);
        check(&recency, &["a", "b", "c"]);
        clock.tick();
        update(&mut recency, &["a", "c"]);
        check(&recency, &["b", "a", "c"]);
        clock.tick();
        update(&mut recency, &["d", "e", "f", "g"]);
        check(&recency, &["b", "a", "c", "d", "e", "f", "g"]);
        clock.tick();
        update(&mut recency, &["c", "f"]);
        check(&recency, &["b", "a", "d", "e", "g", "c", "f"]);
        clock.tick();
        for value in ["d", "g", "b", "f"] {
            recency.remove(&value);
        }
        check(&recency, &["a", "e", "c"]);
        clock.tick();
        update(&mut recency, &["e", "h", "i", "j"]);
        check(&recency, &["a", "c", "e", "h", "i", "j"]);
        clock.tick();
        assert_eq!(recency.remove_least_recent_n(3), ["a", "c", "e"]);
        check(&recency, &["h", "i", "j"]);
        clock.tick();
        update(&mut recency, &["k", "l", "m", "i"]);
        check(&recency, &["h", "j", "k", "l", "m", "i"]);
        clock.tick();
        assert_eq!(recency.remove_most_recent_n(3), ["i", "m", "l"]);
        check(&recency, &["h", "j", "k"]);
        clock.tick();
        update(&mut recency, &["n", "o"]);
        check(&recency, &["h", "j", "k", "n", "o"]);
        clock.tick();
        assert_eq!(recency.remove_least_recent_outside(seconds), ["h", "j"]);
        check(&recency, &["k", "n", "o"]);
        clock.tick();
        update(&mut recency, &["p", "q", "k", "r"]);
        check(&recency, &["n", "o", "p", "q", "k", "r"]);
        clock.tick();
        assert_eq!(
            recency.remove_most_recent_within(seconds),
            ["r", "k", "q", "p"]
        );
        check(&recency, &["n", "o"]);
        clock.tick();
        update(&mut recency, &["s", "t", "u", "v", "w", "x", "y", "z"]);
        check(
            &recency,
            &["n", "o", "s", "t", "u", "v", "w", "x", "y", "z"],
        );
        clock.tick();
        assert_eq!(recency.remove_least_recent_skipping(3, 3), ["t", "u", "v"]);
        check(&recency, &["n", "o", "s", "w", "x", "y", "z"]);
        clock.tick();
        assert_eq!(recency.remove_most_recent_skipping(3, 2), ["x", "w", "s"]);
        check(&recency, &["n", "o", "y", "z"]);
        clock.tick();
        assert_eq!(recency.remove_least_recent_n(10), ["n", "o", "y", "z"]);
        check(&recency, &[]);
        clock.tick();
        update(&mut recency, &["a", "b", "c"]);
        check(&recency, &["a", "b", "c"]);
        clock.tick();
        assert_eq!(recency.remove_most_recent_n(10), ["c", "b", "a"]);
        check(&recency, &[]);
    }

    #[test]
    fn least_and_most_recent_and_single_removals() {
        let mut recency = RecencyList::new(TestClock::default());
        assert_eq!(
            (recency.least_recent(), recency.most_recent()),
            (None, None)
        );
        assert!(
            recency.remove_least_recent().is_empty() && recency.remove_most_recent().is_empty()
        );
        update(&mut recency, &["a", "b", "c"]);
        assert_eq!(
            (recency.least_recent(), recency.most_recent()),
            (Some(&"a"), Some(&"c"))
        );
        assert_eq!(recency.remove_least_recent(), ["a"]);
        assert_eq!(recency.remove_most_recent(), ["c"]);
        assert_eq!(recency.size(), 1);
        recency.remove(&"absent").remove(&"b");
        assert_eq!(recency.size(), 0);
    }

    #[test]
    fn the_nano_clock_list_orders_by_update() {
        let mut recency = RecencyList::empty_with_nano_clock();
        recency.update("x").update("y").update("x");
        assert_eq!(
            recency.least_to_most_recent().copied().collect::<Vec<_>>(),
            ["y", "x"]
        );
        assert_eq!(
            recency.remove_least_recent_outside(Duration::from_secs(3600)),
            Vec::<&str>::new()
        );
        assert_eq!(
            recency.remove_most_recent_within(Duration::from_secs(3600)),
            ["x", "y"]
        );
    }
}
