// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/FrequencyList.scala (Apache-2.0).
//! `FrequencyList` (AC-UTIL-017.2): tracks the frequency of elements to implement least/most frequently
//! used eviction policies. Mutable and not thread safe; a doubly-linked list of doubly-linked lists
//! with a lookup, so every operation is constant time. Elements with the same frequency are kept in
//! order of update (recency within the same frequency). With a clock the overall recency is tracked
//! too, to support time based eviction (`with_overall_recency`). Dynamic aging can be enabled for least
//! frequently used policies, to automatically "age" the whole cache on evictions.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use super::clock::{Clock, NanoClock};
use super::double_linked_list::{DoubleLinkedList, LinkTable, Slab};
use super::recency_list::duration_nanos;
use crate::actor::UnsupportedOperationError;

/// `FrequencyNode`: all the elements that were updated the same number of times (plus the age).
struct FrequencyNode {
    priority: i64,
    nodes: DoubleLinkedList<usize>,
}

struct Node<A> {
    value: A,
    accesses: i64,
    frequency: usize,
    timestamp: i64,
}

/// The clock type of a list without overall recency.
pub type NoClock = NanoClock;

const OVERALL_RECENCY_DISABLED: &str = "Overall recency is not enabled for this FrequencyList";

/// `FrequencyList[A]`.
pub struct FrequencyList<A, C> {
    dynamic_aging: bool,
    clock: Option<C>,
    frequency: DoubleLinkedList<usize>,
    frequency_links: LinkTable,
    frequency_nodes: RefCell<Slab<FrequencyNode>>,
    nodes: Slab<Node<A>>,
    /// Links of the nodes inside their frequency node (`lessRecent`, `moreRecent`).
    node_links: LinkTable,
    overall_recency: DoubleLinkedList<usize>,
    overall_links: LinkTable,
    lookup_node: HashMap<A, usize>,
    age: i64,
}

impl<A: Clone + Eq + Hash, C: Clock> FrequencyList<A, C> {
    /// `FrequencyList.empty(dynamicAging)`: without overall recency (its `C` is never used: write
    /// [`NoClock`]).
    pub fn empty(dynamic_aging: bool) -> Self {
        Self::build(dynamic_aging, None)
    }

    /// `FrequencyList.withOverallRecency.empty(clock, dynamicAging)`.
    pub fn with_overall_recency(clock: C, dynamic_aging: bool) -> Self {
        Self::build(dynamic_aging, Some(clock))
    }

    fn build(dynamic_aging: bool, clock: Option<C>) -> Self {
        Self {
            dynamic_aging,
            clock,
            frequency: DoubleLinkedList::new(),
            frequency_links: LinkTable::new(),
            frequency_nodes: RefCell::new(Slab::default()),
            nodes: Slab::default(),
            node_links: LinkTable::new(),
            overall_recency: DoubleLinkedList::new(),
            overall_links: LinkTable::new(),
            lookup_node: HashMap::new(),
            age: 0,
        }
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.lookup_node.len()
    }

    /// `update(value)`: counts one more access of the value, adding it if it is new.
    pub fn update(&mut self, value: A) -> &mut Self {
        if let Some(&node) = self.lookup_node.get(&value) {
            self.increase_frequency(node);
            if let Some(clock) = &self.clock {
                self.nodes.get_mut(node).timestamp = clock.current_time();
                self.overall_recency
                    .move_to_back(&mut self.overall_links, node);
            }
        } else {
            let node = self.add_initial_frequency(value.clone());
            self.lookup_node.insert(value, node);
            if let Some(clock) = &self.clock {
                self.nodes.get_mut(node).timestamp = clock.current_time();
                self.overall_recency.append(&mut self.overall_links, node);
            }
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

    /// `leastFrequent`: the least recently updated of the least frequent.
    pub fn least_frequent(&self) -> Option<&A> {
        let least = *self.frequency.first()?;
        let first = *self.frequency_nodes.borrow().get(least).nodes.first()?;
        Some(&self.nodes.get(first).value)
    }

    /// `mostFrequent`: the most recently updated of the most frequent.
    pub fn most_frequent(&self) -> Option<&A> {
        let most = *self.frequency.last()?;
        let last = *self.frequency_nodes.borrow().get(most).nodes.last()?;
        Some(&self.nodes.get(last).value)
    }

    /// The nodes from the least to the most frequent (within a frequency, least recent first).
    fn forward_nodes(&self) -> Vec<usize> {
        let frequency_nodes = self.frequency_nodes.borrow();
        self.frequency
            .forward_iterator(&self.frequency_links)
            .flat_map(|f| {
                frequency_nodes
                    .get(f)
                    .nodes
                    .forward_iterator(&self.node_links)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// The nodes from the most to the least frequent (within a frequency, most recent first).
    fn backward_nodes(&self) -> Vec<usize> {
        let frequency_nodes = self.frequency_nodes.borrow();
        self.frequency
            .backward_iterator(&self.frequency_links)
            .flat_map(|f| {
                frequency_nodes
                    .get(f)
                    .nodes
                    .backward_iterator(&self.node_links)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// `leastToMostFrequent`.
    pub fn least_to_most_frequent(&self) -> Vec<&A> {
        self.forward_nodes()
            .into_iter()
            .map(|node| &self.nodes.get(node).value)
            .collect()
    }

    /// `mostToLeastFrequent`.
    pub fn most_to_least_frequent(&self) -> Vec<&A> {
        self.backward_nodes()
            .into_iter()
            .map(|node| &self.nodes.get(node).value)
            .collect()
    }

    /// `removeLeastFrequent(n)`: removes and returns the `n` least frequent values (with dynamic aging
    /// the age follows the last one removed).
    pub fn remove_least_frequent_n(&mut self, n: usize) -> Vec<A> {
        let nodes: Vec<usize> = self.forward_nodes().into_iter().take(n).collect();
        nodes
            .into_iter()
            .map(|node| self.remove_least_frequent_node(node))
            .collect()
    }

    /// `removeLeastFrequent(n, skip)`: as above without ever choosing `skip`.
    pub fn remove_least_frequent_skipping(&mut self, n: usize, skip: Option<&A>) -> Vec<A> {
        let nodes: Vec<usize> = self
            .forward_nodes()
            .into_iter()
            .filter(|node| skip != Some(&self.nodes.get(*node).value))
            .take(n)
            .collect();
        nodes
            .into_iter()
            .map(|node| self.remove_least_frequent_node(node))
            .collect()
    }

    /// `removeLeastFrequent()`: the least frequent value, if any.
    pub fn remove_least_frequent(&mut self) -> Vec<A> {
        let first = self.frequency.first().and_then(|least| {
            self.frequency_nodes
                .borrow()
                .get(*least)
                .nodes
                .first()
                .copied()
        });
        match first {
            Some(node) => vec![self.remove_least_frequent_node(node)],
            None => Vec::new(),
        }
    }

    /// `removeMostFrequent(n)`.
    pub fn remove_most_frequent_n(&mut self, n: usize) -> Vec<A> {
        let nodes: Vec<usize> = self.backward_nodes().into_iter().take(n).collect();
        nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect()
    }

    /// `removeMostFrequent(n, skip)`.
    pub fn remove_most_frequent_skipping(&mut self, n: usize, skip: Option<&A>) -> Vec<A> {
        let nodes: Vec<usize> = self
            .backward_nodes()
            .into_iter()
            .filter(|node| skip != Some(&self.nodes.get(*node).value))
            .take(n)
            .collect();
        nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect()
    }

    /// `removeMostFrequent()`.
    pub fn remove_most_frequent(&mut self) -> Vec<A> {
        let last = self.frequency.last().and_then(|most| {
            self.frequency_nodes
                .borrow()
                .get(*most)
                .nodes
                .last()
                .copied()
        });
        match last {
            Some(node) => vec![self.remove_node(node)],
            None => Vec::new(),
        }
    }

    fn require_overall_recency(&self) -> Result<&C, UnsupportedOperationError> {
        self.clock
            .as_ref()
            .ok_or_else(|| UnsupportedOperationError::new(OVERALL_RECENCY_DISABLED))
    }

    /// `overallLeastToMostRecent`: the values from the least to the most recently updated (empty
    /// without overall recency).
    pub fn overall_least_to_most_recent(&self) -> impl Iterator<Item = &A> {
        self.overall_recency
            .forward_iterator(&self.overall_links)
            .map(|node| &self.nodes.get(node).value)
    }

    /// `overallMostToLeastRecent`.
    pub fn overall_most_to_least_recent(&self) -> impl Iterator<Item = &A> {
        self.overall_recency
            .backward_iterator(&self.overall_links)
            .map(|node| &self.nodes.get(node).value)
    }

    /// `removeOverallLeastRecent(n)`: needs the overall recency (`UnsupportedOperationException` otherwise).
    pub fn remove_overall_least_recent(
        &mut self,
        n: usize,
    ) -> Result<Vec<A>, UnsupportedOperationError> {
        self.require_overall_recency()?;
        let nodes: Vec<usize> = self
            .overall_recency
            .forward_iterator(&self.overall_links)
            .take(n)
            .collect();
        Ok(nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect())
    }

    /// `removeOverallMostRecent(n)`.
    pub fn remove_overall_most_recent(
        &mut self,
        n: usize,
    ) -> Result<Vec<A>, UnsupportedOperationError> {
        self.require_overall_recency()?;
        let nodes: Vec<usize> = self
            .overall_recency
            .backward_iterator(&self.overall_links)
            .take(n)
            .collect();
        Ok(nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect())
    }

    /// `removeOverallLeastRecentOutside(duration)`.
    pub fn remove_overall_least_recent_outside(
        &mut self,
        duration: Duration,
    ) -> Result<Vec<A>, UnsupportedOperationError> {
        let min = self
            .require_overall_recency()?
            .earlier_time(duration_nanos(duration));
        let nodes: Vec<usize> = self
            .overall_recency
            .forward_iterator(&self.overall_links)
            .take_while(|node| self.nodes.get(*node).timestamp < min)
            .collect();
        Ok(nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect())
    }

    /// `removeOverallMostRecentWithin(duration)`.
    pub fn remove_overall_most_recent_within(
        &mut self,
        duration: Duration,
    ) -> Result<Vec<A>, UnsupportedOperationError> {
        let max = self
            .require_overall_recency()?
            .earlier_time(duration_nanos(duration));
        let nodes: Vec<usize> = self
            .overall_recency
            .backward_iterator(&self.overall_links)
            .take_while(|node| self.nodes.get(*node).timestamp > max)
            .collect();
        Ok(nodes
            .into_iter()
            .map(|node| self.remove_node(node))
            .collect())
    }

    fn add_initial_frequency(&mut self, value: A) -> usize {
        let frequency_node = if self.dynamic_aging {
            self.initial_adjusted_frequency_node()
        } else {
            self.least_frequent_node()
        };
        let node = self.nodes.insert(Node {
            value,
            accesses: 1,
            frequency: frequency_node,
            timestamp: 0,
        });
        self.add_to_frequency(node, frequency_node);
        node
    }

    /// `addAsLeastFrequent`: the frequency node of priority 1, at the front.
    fn least_frequent_node(&mut self) -> usize {
        let table = &self.frequency_nodes;
        self.frequency.get_first_or_else_prepend(
            &mut self.frequency_links,
            |f| priority_of(table, *f) == 1,
            || new_frequency_node(table, 1),
        )
    }

    /// `addInitialAdjustedFrequency`: priority 1 plus the age, kept in order.
    fn initial_adjusted_frequency_node(&mut self) -> usize {
        let priority = 1 + self.age;
        let table = &self.frequency_nodes;
        match self.frequency.first().copied() {
            Some(first) if priority_of(table, first) < priority => {
                self.frequency.find_next_or_else_insert(
                    &mut self.frequency_links,
                    &first,
                    |f| priority_of(table, *f) < priority,
                    |f| priority_of(table, *f) == priority,
                    || new_frequency_node(table, priority),
                )
            }
            _ => self.frequency.get_first_or_else_prepend(
                &mut self.frequency_links,
                |f| priority_of(table, *f) == priority,
                || new_frequency_node(table, priority),
            ),
        }
    }

    fn increase_frequency(&mut self, node: usize) {
        self.nodes.get_mut(node).accesses += 1;
        let current = self.nodes.get(node).frequency;
        let accesses = self.nodes.get(node).accesses;
        let table = &self.frequency_nodes;
        let frequency_node = if self.dynamic_aging {
            let priority = accesses + self.age;
            self.frequency.find_next_or_else_insert(
                &mut self.frequency_links,
                &current,
                |f| priority_of(table, *f) < priority,
                |f| priority_of(table, *f) == priority,
                || new_frequency_node(table, priority),
            )
        } else {
            self.frequency.get_next_or_else_insert(
                &mut self.frequency_links,
                &current,
                |f| priority_of(table, *f) == accesses,
                || new_frequency_node(table, accesses),
            )
        };
        self.remove_from_frequency(node);
        self.add_to_frequency(node, frequency_node);
    }

    fn add_to_frequency(&mut self, node: usize, frequency_node: usize) {
        self.nodes.get_mut(node).frequency = frequency_node;
        self.frequency_nodes
            .get_mut()
            .get_mut(frequency_node)
            .nodes
            .append(&mut self.node_links, node);
    }

    fn remove_from_frequency(&mut self, node: usize) {
        let frequency_node = self.nodes.get(node).frequency;
        let table = self.frequency_nodes.get_mut();
        let nodes = &mut table.get_mut(frequency_node).nodes;
        nodes.remove(&mut self.node_links, &node);
        if nodes.is_empty() {
            self.frequency
                .remove(&mut self.frequency_links, &frequency_node);
            table.remove(frequency_node);
        }
    }

    fn remove_node(&mut self, node: usize) -> A {
        self.remove_from_frequency(node);
        if self.clock.is_some() {
            self.overall_recency.remove(&mut self.overall_links, &node);
        }
        let removed = self.nodes.remove(node);
        self.lookup_node.remove(&removed.value);
        removed.value
    }

    fn remove_least_frequent_node(&mut self, node: usize) -> A {
        if self.dynamic_aging {
            self.age = priority_of(&self.frequency_nodes, self.nodes.get(node).frequency);
        }
        self.remove_node(node)
    }
}

fn priority_of(table: &RefCell<Slab<FrequencyNode>>, frequency_node: usize) -> i64 {
    table.borrow().get(frequency_node).priority
}

fn new_frequency_node(table: &RefCell<Slab<FrequencyNode>>, priority: i64) -> usize {
    table.borrow_mut().insert(FrequencyNode {
        priority,
        nodes: DoubleLinkedList::new(),
    })
}

#[cfg(test)]
mod tests {
    //! `FrequencyListSpec` of Pekko 2.0.0 (actor-tests/.../util/FrequencyListSpec.scala).
    use super::*;
    use crate::util::test_clock::TestClock;

    type List<C> = FrequencyList<&'static str, C>;

    fn check<C: Clock>(list: &List<C>, least_to_most_frequent: &[&str]) {
        assert!(least_to_most_frequent.iter().all(|v| list.contains(v)));
        assert_eq!(list.size(), least_to_most_frequent.len());
        assert_eq!(
            list.least_to_most_frequent()
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            least_to_most_frequent
        );
        let mut reversed = least_to_most_frequent.to_vec();
        reversed.reverse();
        assert_eq!(
            list.most_to_least_frequent()
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            reversed
        );
    }

    fn check_recency(list: &List<TestClock>, least_to_most_recent: &[&str]) {
        assert_eq!(list.size(), least_to_most_recent.len());
        assert_eq!(
            list.overall_least_to_most_recent()
                .copied()
                .collect::<Vec<_>>(),
            least_to_most_recent
        );
        let mut reversed = least_to_most_recent.to_vec();
        reversed.reverse();
        assert_eq!(
            list.overall_most_to_least_recent()
                .copied()
                .collect::<Vec<_>>(),
            reversed
        );
    }

    fn update<C: Clock>(list: &mut List<C>, values: &[&'static str]) {
        for value in values {
            list.update(value);
        }
    }

    fn remove<C: Clock>(list: &mut List<C>, values: &[&'static str]) {
        for value in values {
            list.remove(value);
        }
    }

    #[test]
    fn track_frequency_of_elements() {
        let mut frequency = List::<NoClock>::empty(false);
        check(&frequency, &[]);
        update(&mut frequency, &["a"]);
        check(&frequency, &["a"]);
        update(&mut frequency, &["b", "c"]);
        check(&frequency, &["a", "b", "c"]);
        update(&mut frequency, &["a", "c"]);
        check(&frequency, &["b", "a", "c"]);
        update(&mut frequency, &["d", "e", "f", "g"]);
        check(&frequency, &["b", "d", "e", "f", "g", "a", "c"]);
        update(&mut frequency, &["c", "f"]);
        check(&frequency, &["b", "d", "e", "g", "a", "f", "c"]);
        remove(&mut frequency, &["d", "g", "b", "f"]);
        check(&frequency, &["e", "a", "c"]);
        update(&mut frequency, &["e", "h", "i"]);
        check(&frequency, &["h", "i", "a", "e", "c"]);
        assert_eq!(frequency.remove_least_frequent_n(3), ["h", "i", "a"]);
        check(&frequency, &["e", "c"]);
        update(&mut frequency, &["j", "k", "l", "m"]);
        check(&frequency, &["j", "k", "l", "m", "e", "c"]);
        assert_eq!(
            frequency.remove_least_frequent_skipping(1, Some(&"j")),
            ["k"]
        );
        check(&frequency, &["j", "l", "m", "e", "c"]);
        assert_eq!(
            frequency.remove_least_frequent_skipping(2, Some(&"l")),
            ["j", "m"]
        );
        check(&frequency, &["l", "e", "c"]);
        update(&mut frequency, &["n", "o", "p", "e", "o", "l"]);
        check(&frequency, &["n", "p", "o", "l", "c", "e"]);
        assert_eq!(frequency.remove_most_frequent_n(3), ["e", "c", "l"]);
        check(&frequency, &["n", "p", "o"]);
        update(&mut frequency, &["q", "r", "p", "o", "n"]);
        check(&frequency, &["q", "r", "p", "n", "o"]);
        assert_eq!(
            frequency.remove_most_frequent_skipping(1, Some(&"o")),
            ["n"]
        );
        check(&frequency, &["q", "r", "p", "o"]);
        assert_eq!(
            frequency.remove_most_frequent_skipping(2, Some(&"p")),
            ["o", "r"]
        );
        check(&frequency, &["q", "p"]);
    }

    #[test]
    fn track_overall_recency_of_elements_when_enabled() {
        let clock = TestClock::default();
        let mut frequency = List::with_overall_recency(clock.clone(), false);
        check(&frequency, &[]);
        clock.tick();
        update(&mut frequency, &["a"]);
        check(&frequency, &["a"]);
        check_recency(&frequency, &["a"]);
        clock.tick();
        update(&mut frequency, &["b", "c"]);
        check_recency(&frequency, &["a", "b", "c"]);
        clock.tick();
        update(&mut frequency, &["a", "c"]);
        check(&frequency, &["b", "a", "c"]);
        check_recency(&frequency, &["b", "a", "c"]);
        clock.tick();
        update(&mut frequency, &["d", "e", "f"]);
        check(&frequency, &["b", "d", "e", "f", "a", "c"]);
        check_recency(&frequency, &["b", "a", "c", "d", "e", "f"]);
        clock.tick();
        update(&mut frequency, &["c", "f"]);
        check(&frequency, &["b", "d", "e", "a", "f", "c"]);
        check_recency(&frequency, &["b", "a", "d", "e", "c", "f"]);
        clock.tick();
        remove(&mut frequency, &["d", "b", "f"]);
        check(&frequency, &["e", "a", "c"]);
        check_recency(&frequency, &["a", "e", "c"]);
        clock.tick();
        update(&mut frequency, &["e", "h", "i"]);
        check(&frequency, &["h", "i", "a", "e", "c"]);
        check_recency(&frequency, &["a", "c", "e", "h", "i"]);
        clock.tick();
        assert_eq!(frequency.remove_overall_least_recent(1), Ok(vec!["a"]));
        check(&frequency, &["h", "i", "e", "c"]);
        check_recency(&frequency, &["c", "e", "h", "i"]);
        clock.tick();
        update(&mut frequency, &["i", "j", "k"]);
        check(&frequency, &["h", "j", "k", "e", "i", "c"]);
        check_recency(&frequency, &["c", "e", "h", "i", "j", "k"]);
        clock.tick();
        assert_eq!(frequency.remove_overall_most_recent(1), Ok(vec!["k"]));
        check(&frequency, &["h", "j", "e", "i", "c"]);
        check_recency(&frequency, &["c", "e", "h", "i", "j"]);
        clock.tick();
        assert_eq!(
            frequency.remove_overall_least_recent_outside(Duration::from_secs(3)),
            Ok(vec!["c", "e", "h"])
        );
        check(&frequency, &["j", "i"]);
        check_recency(&frequency, &["i", "j"]);
        clock.tick();
        update(&mut frequency, &["l", "m"]);
        check(&frequency, &["j", "l", "m", "i"]);
        check_recency(&frequency, &["i", "j", "l", "m"]);
        clock.tick();
        assert_eq!(
            frequency.remove_overall_most_recent_within(Duration::from_secs(3)),
            Ok(vec!["m", "l"])
        );
        check(&frequency, &["j", "i"]);
        check_recency(&frequency, &["i", "j"]);
        clock.tick();
        update(&mut frequency, &["n", "o", "n"]);
        check(&frequency, &["j", "o", "i", "n"]);
        check_recency(&frequency, &["i", "j", "o", "n"]);
    }

    #[test]
    fn support_dynamic_aging_with_least_frequently_used_eviction() {
        let mut regular = List::<NoClock>::empty(false);
        let mut aging = List::<NoClock>::empty(true);
        check(&regular, &[]);
        check(&aging, &[]);
        for _ in 1..=10 {
            update(&mut regular, &["a", "b", "c"]);
            update(&mut aging, &["a", "b", "c"]);
        }
        check(&regular, &["a", "b", "c"]);
        check(&aging, &["a", "b", "c"]);
        // age = 0
        update(&mut regular, &["x", "y", "z"]);
        check(&regular, &["x", "y", "z", "a", "b", "c"]);
        update(&mut aging, &["x", "y", "z"]);
        check(&aging, &["x", "y", "z", "a", "b", "c"]);
        assert_eq!(regular.remove_least_frequent(), ["x"]);
        check(&regular, &["y", "z", "a", "b", "c"]);
        assert_eq!(aging.remove_least_frequent(), ["x"]);
        check(&aging, &["y", "z", "a", "b", "c"]);
        // age = 1 (from the last removal of "x")
        update(&mut regular, &["x", "y", "z", "z"]);
        check(&regular, &["x", "y", "z", "a", "b", "c"]);
        update(&mut aging, &["x", "y", "z", "z"]);
        check(&aging, &["x", "y", "z", "a", "b", "c"]);
        assert_eq!(regular.remove_least_frequent_n(2), ["x", "y"]);
        check(&regular, &["z", "a", "b", "c"]);
        assert_eq!(aging.remove_least_frequent_n(2), ["x", "y"]);
        check(&aging, &["z", "a", "b", "c"]);
        // age = 3 (from the last removal of "y")
        update(&mut regular, &["x", "y", "z"]);
        check(&regular, &["x", "y", "z", "a", "b", "c"]);
        update(&mut aging, &["x", "y", "z"]);
        check(&aging, &["x", "y", "z", "a", "b", "c"]);
        assert_eq!(regular.remove_least_frequent_n(3), ["x", "y", "z"]);
        check(&regular, &["a", "b", "c"]);
        assert_eq!(aging.remove_least_frequent_n(3), ["x", "y", "z"]);
        check(&aging, &["a", "b", "c"]);
        // age = 7 (from the last removal of "z")
        update(&mut regular, &["x", "y", "y", "z", "z", "z"]);
        check(&regular, &["x", "y", "z", "a", "b", "c"]);
        update(&mut aging, &["x", "y", "y", "z", "z", "z"]);
        check(&aging, &["x", "y", "a", "b", "c", "z"]);
        assert_eq!(regular.remove_least_frequent_n(2), ["x", "y"]);
        check(&regular, &["z", "a", "b", "c"]);
        assert_eq!(aging.remove_least_frequent_n(2), ["x", "y"]);
        check(&aging, &["a", "b", "c", "z"]);
        // age = 9 (from the last removal of "y")
        update(&mut regular, &["x", "y", "z"]);
        check(&regular, &["x", "y", "z", "a", "b", "c"]);
        update(&mut aging, &["x", "y", "z"]);
        check(&aging, &["a", "b", "c", "x", "y", "z"]);
        assert_eq!(regular.remove_least_frequent_n(3), ["x", "y", "z"]);
        check(&regular, &["a", "b", "c"]);
        assert_eq!(aging.remove_least_frequent_n(3), ["a", "b", "c"]);
        check(&aging, &["x", "y", "z"]);
        // age = 10 (from the last removal of "c")
    }

    #[test]
    fn least_and_most_frequent_and_overall_recency_not_enabled() {
        let mut frequency = List::<NoClock>::empty(false);
        assert_eq!(
            (frequency.least_frequent(), frequency.most_frequent()),
            (None, None)
        );
        assert!(
            frequency.remove_least_frequent().is_empty()
                && frequency.remove_most_frequent().is_empty()
        );
        update(&mut frequency, &["a", "b", "a"]);
        assert_eq!(
            (frequency.least_frequent(), frequency.most_frequent()),
            (Some(&"b"), Some(&"a"))
        );
        let error = frequency
            .remove_overall_least_recent(1)
            .expect_err("no clock");
        assert_eq!(
            (error.java_class(), error.message()),
            (
                "UnsupportedOperationException",
                "Overall recency is not enabled for this FrequencyList"
            )
        );
        assert!(frequency.remove_overall_most_recent(1).is_err());
        assert!(
            frequency
                .remove_overall_least_recent_outside(Duration::from_secs(1))
                .is_err()
        );
        assert!(
            frequency
                .remove_overall_most_recent_within(Duration::from_secs(1))
                .is_err()
        );
        assert_eq!(frequency.overall_least_to_most_recent().count(), 0);
        assert_eq!(frequency.remove_most_frequent(), ["a"]);
        assert_eq!(frequency.remove_least_frequent(), ["b"]);
        assert_eq!(frequency.size(), 0);
    }
}
