// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/SegmentedRecencyList.scala (Apache-2.0).
//! `SegmentedRecencyList` (AC-UTIL-017.4): a variation of `RecencyList` for segmented least recently
//! used eviction policies. Mutable and not thread safe; doubly-linked lists plus a hash map, so every
//! operation is constant time. Every segment has a limit; the lowest is the probationary one and the
//! higher ones protect what was used more than once. With a clock the list also tracks the recency
//! over all the segments (`with_overall_recency`).

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use super::clock::Clock;
use super::double_linked_list::{DoubleLinkedList, LinkTable, Slab};
use super::recency_list::duration_nanos;
use crate::actor::UnsupportedOperationError;

struct Node<A> {
    value: A,
    level: usize,
    timestamp: i64,
}

/// `SegmentedRecencyList[A]`; `C` is the clock for the overall recency (`NoClock` without it).
pub struct SegmentedRecencyList<A, C> {
    limits: Vec<i32>,
    total_limit: i64,
    clock: Option<C>,
    nodes: Slab<Node<A>>,
    segment_links: LinkTable,
    segments: Vec<DoubleLinkedList<usize>>,
    sizes: Vec<i32>,
    overall_links: LinkTable,
    overall_recency: DoubleLinkedList<usize>,
    lookup_node: HashMap<A, usize>,
}

/// The clock type of a list without overall recency.
pub type NoClock = super::clock::NanoClock;

impl<A: Clone + Eq + Hash, C: Clock> SegmentedRecencyList<A, C> {
    /// `SegmentedRecencyList.empty(limits)`: one segment per limit, no overall recency (its `C` is
    /// never used: write [`NoClock`]).
    pub fn empty(limits: &[i32]) -> Self {
        Self::build(limits, None)
    }

    /// `SegmentedRecencyList.withOverallRecency.empty(clock, limits)`.
    pub fn with_overall_recency(clock: C, limits: &[i32]) -> Self {
        Self::build(limits, Some(clock))
    }

    fn build(limits: &[i32], clock: Option<C>) -> Self {
        let levels = limits.len();
        Self {
            limits: limits.to_vec(),
            total_limit: limits.iter().map(|l| i64::from(*l)).sum(),
            clock,
            nodes: Slab::default(),
            segment_links: LinkTable::new(),
            segments: (0..levels).map(|_| DoubleLinkedList::new()).collect(),
            sizes: vec![0; levels],
            overall_links: LinkTable::new(),
            overall_recency: DoubleLinkedList::new(),
            lookup_node: HashMap::new(),
        }
    }

    fn lowest(&self) -> usize {
        0
    }

    fn highest(&self) -> usize {
        self.segments.len() - 1
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.lookup_node.len()
    }

    /// `sizeOf(level)`: the number of values in the segment.
    pub fn size_of(&self, level: usize) -> i32 {
        self.sizes[level]
    }

    /// `update(value)`: adds the value to the lowest segment, or promotes it one segment.
    pub fn update(&mut self, value: A) -> &mut Self {
        if let Some(&node) = self.lookup_node.get(&value) {
            self.promote(node);
        } else {
            self.insert(value);
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

    /// `leastRecent`: the least recent value of the lowest segment.
    pub fn least_recent(&self) -> Option<&A> {
        self.segments[self.lowest()]
            .first()
            .map(|node| &self.nodes.get(*node).value)
    }

    /// `leastToMostRecentOf(level)`: the values of one segment.
    pub fn least_to_most_recent_of(&self, level: usize) -> impl Iterator<Item = &A> {
        self.segments[level]
            .forward_iterator(&self.segment_links)
            .map(|node| &self.nodes.get(node).value)
    }

    /// `removeLeastRecentOverLimit()`: when the total size is over the total limit, demotes the excess
    /// of the protected segments and removes the excess values from the lowest one.
    pub fn remove_least_recent_over_limit(&mut self) -> Vec<A> {
        let size = self.size() as i64;
        if size > self.total_limit {
            self.adjust_protected_levels();
            let excess = (size - self.total_limit) as usize;
            if excess == 1 {
                self.remove_least_recent()
            } else {
                let nodes: Vec<usize> = self.segments[self.lowest()]
                    .forward_iterator(&self.segment_links)
                    .take(excess)
                    .collect();
                nodes
                    .into_iter()
                    .map(|node| self.remove_node(node))
                    .collect()
            }
        } else {
            Vec::new()
        }
    }

    /// `removeLeastRecent()`: the least recent value of the lowest segment.
    pub fn remove_least_recent(&mut self) -> Vec<A> {
        match self.segments[self.lowest()].first().copied() {
            Some(first) => vec![self.remove_node(first)],
            None => Vec::new(),
        }
    }

    /// `removeOverallLeastRecentOutside(duration)`: needs the overall recency
    /// (`UnsupportedOperationException` otherwise).
    pub fn remove_overall_least_recent_outside(
        &mut self,
        duration: Duration,
    ) -> Result<Vec<A>, UnsupportedOperationError> {
        let Some(clock) = &self.clock else {
            return Err(UnsupportedOperationError::new(
                "Overall recency is not enabled",
            ));
        };
        let min = clock.earlier_time(duration_nanos(duration));
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

    /// `updateLimits(newLimits)`: the number of limits must stay the number of segments.
    pub fn update_limits(&mut self, new_limits: &[i32]) {
        self.limits = new_limits.to_vec();
        self.total_limit = new_limits.iter().map(|l| i64::from(*l)).sum();
    }

    fn adjust_protected_levels(&mut self) {
        for level in (self.lowest() + 1..=self.highest()).rev() {
            self.adjust(level);
        }
    }

    fn adjust(&mut self, level: usize) {
        let excess = i64::from(self.sizes[level]) - i64::from(self.limits[level]);
        if excess > 0 {
            let nodes: Vec<usize> = self.segments[level]
                .forward_iterator(&self.segment_links)
                .take(excess as usize)
                .collect();
            for node in nodes {
                self.demote(node);
            }
        }
    }

    fn insert(&mut self, value: A) {
        let node = self.nodes.insert(Node {
            value: value.clone(),
            level: 0,
            timestamp: 0,
        });
        let lowest = self.lowest();
        self.append_to(lowest, node);
        self.lookup_node.insert(value, node);
        if let Some(clock) = &self.clock {
            self.nodes.get_mut(node).timestamp = clock.current_time();
            self.overall_recency.append(&mut self.overall_links, node);
        }
    }

    fn promote(&mut self, node: usize) {
        let level = self.nodes.get(node).level;
        if level == self.highest() {
            self.segments[level].move_to_back(&mut self.segment_links, node);
        } else {
            let new_level = level + 1;
            self.remove_from_current_level(node);
            self.append_to(new_level, node);
            self.adjust(new_level);
        }
        if let Some(clock) = &self.clock {
            self.nodes.get_mut(node).timestamp = clock.current_time();
            self.overall_recency
                .move_to_back(&mut self.overall_links, node);
        }
    }

    /// Only from the higher protected levels.
    fn demote(&mut self, node: usize) {
        self.remove_from_current_level(node);
        let level = self.nodes.get(node).level - 1;
        self.append_to(level, node);
    }

    fn append_to(&mut self, level: usize, node: usize) {
        self.nodes.get_mut(node).level = level;
        self.segments[level].append(&mut self.segment_links, node);
        self.sizes[level] += 1;
    }

    fn remove_from_current_level(&mut self, node: usize) {
        let level = self.nodes.get(node).level;
        self.segments[level].remove(&mut self.segment_links, &node);
        self.sizes[level] -= 1;
    }

    fn remove_node(&mut self, node: usize) -> A {
        self.remove_from_current_level(node);
        if self.clock.is_some() {
            self.overall_recency.remove(&mut self.overall_links, &node);
        }
        let removed = self.nodes.remove(node);
        self.lookup_node.remove(&removed.value);
        removed.value
    }
}

#[cfg(test)]
mod tests {
    //! `SegmentedRecencyListSpec` of Pekko 2.0.0 (actor-tests/.../util/SegmentedRecencyListSpec.scala).
    use super::*;
    use crate::util::test_clock::TestClock;

    type List = SegmentedRecencyList<&'static str, TestClock>;

    fn update(list: &mut List, values: &[&'static str]) {
        for value in values {
            list.update(value);
        }
    }

    fn plain(limits: &[i32]) -> List {
        List::build(limits, None)
    }

    fn check(list: &List, expected: &[&[&'static str]]) {
        assert_eq!(
            list.size(),
            expected.iter().map(|segment| segment.len()).sum::<usize>()
        );
        for (level, segment) in expected.iter().enumerate() {
            assert!(segment.iter().all(|v| list.contains(v)));
            assert_eq!(
                list.least_to_most_recent_of(level)
                    .copied()
                    .collect::<Vec<_>>(),
                *segment
            );
            assert_eq!(list.size_of(level) as usize, segment.len());
        }
    }

    #[test]
    fn track_recency_of_elements_across_2_segments() {
        let mut recency = plain(&[2, 3]);
        check(&recency, &[]);
        update(&mut recency, &["a", "b", "c", "d", "e", "f"]);
        check(&recency, &[&["a", "b", "c", "d", "e", "f"], &[]]);
        assert_eq!(recency.remove_least_recent_over_limit(), ["a"]);
        check(&recency, &[&["b", "c", "d", "e", "f"], &[]]);
        update(&mut recency, &["a", "b", "c", "d"]);
        check(&recency, &[&["e", "f", "a"], &["b", "c", "d"]]);
        assert_eq!(recency.remove_least_recent_over_limit(), ["e"]);
        check(&recency, &[&["f", "a"], &["b", "c", "d"]]);
        update(&mut recency, &["a"]);
        check(&recency, &[&["f", "b"], &["c", "d", "a"]]);
        assert_eq!(recency.remove_least_recent_over_limit(), Vec::<&str>::new());
        check(&recency, &[&["f", "b"], &["c", "d", "a"]]);
        update(&mut recency, &["a", "b", "c", "d", "e", "f", "g"]);
        check(&recency, &[&["a", "e", "b", "g"], &["c", "d", "f"]]);
        assert_eq!(recency.remove_least_recent_over_limit(), ["a", "e"]);
        check(&recency, &[&["b", "g"], &["c", "d", "f"]]);
    }

    #[test]
    fn track_recency_of_elements_across_4_segments() {
        let mut recency = plain(&[4, 3, 2, 1]);
        check(&recency, &[]);
        update(&mut recency, &["a", "b", "c", "d", "e", "f"]);
        check(&recency, &[&["a", "b", "c", "d", "e", "f"], &[], &[], &[]]);
        assert_eq!(recency.remove_least_recent_over_limit(), Vec::<&str>::new());
        update(&mut recency, &["g", "h", "i", "j", "k", "l"]);
        check(
            &recency,
            &[
                &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"],
                &[],
                &[],
                &[],
            ],
        );
        assert_eq!(recency.remove_least_recent_over_limit(), ["a", "b"]);
        check(
            &recency,
            &[
                &["c", "d", "e", "f", "g", "h", "i", "j", "k", "l"],
                &[],
                &[],
                &[],
            ],
        );
        update(&mut recency, &["b", "d", "f", "h", "j", "l"]);
        check(
            &recency,
            &[
                &["c", "e", "g", "i", "k", "b", "d", "f"],
                &["h", "j", "l"],
                &[],
                &[],
            ],
        );
        update(&mut recency, &["h", "i", "j", "k", "l", "m"]);
        check(
            &recency,
            &[
                &["c", "e", "g", "b", "d", "f", "m"],
                &["i", "k", "h"],
                &["j", "l"],
                &[],
            ],
        );
        assert_eq!(recency.remove_least_recent_over_limit(), ["c", "e"]);
        check(
            &recency,
            &[
                &["g", "b", "d", "f", "m"],
                &["i", "k", "h"],
                &["j", "l"],
                &[],
            ],
        );
        update(&mut recency, &["j", "k", "l", "m", "n", "o", "p"]);
        check(
            &recency,
            &[
                &["g", "b", "d", "f", "n", "o", "p"],
                &["i", "h", "m"],
                &["k", "j"],
                &["l"],
            ],
        );
        assert_eq!(recency.remove_least_recent_over_limit(), ["g", "b", "d"]);
        check(
            &recency,
            &[&["f", "n", "o", "p"], &["i", "h", "m"], &["k", "j"], &["l"]],
        );
    }

    #[test]
    fn allow_protected_limits_to_be_updated() {
        let mut recency = plain(&[2, 8]);
        check(&recency, &[]);
        let letters = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"];
        update(&mut recency, &letters);
        check(&recency, &[&letters, &[]]);
        update(&mut recency, &letters);
        check(
            &recency,
            &[
                &["a", "b", "c", "d"],
                &["e", "f", "g", "h", "i", "j", "k", "l"],
            ],
        );
        assert_eq!(recency.remove_least_recent_over_limit(), ["a", "b"]);
        check(
            &recency,
            &[&["c", "d"], &["e", "f", "g", "h", "i", "j", "k", "l"]],
        );
        recency.update_limits(&[1, 4]);
        assert_eq!(
            recency.remove_least_recent_over_limit(),
            ["c", "d", "e", "f", "g"]
        );
        check(&recency, &[&["h"], &["i", "j", "k", "l"]]);
    }

    #[test]
    fn remove_overall_least_recent_elements() {
        let clock = TestClock::default();
        let mut recency = SegmentedRecencyList::with_overall_recency(clock.clone(), &[5, 5]);
        check(&recency, &[]);
        clock.tick();
        update(&mut recency, &["a", "b", "c", "d", "e", "f", "g"]);
        check(&recency, &[&["a", "b", "c", "d", "e", "f", "g"], &[]]);
        clock.tick();
        update(&mut recency, &["a", "b", "c", "d", "e", "f", "g"]);
        check(&recency, &[&["a", "b"], &["c", "d", "e", "f", "g"]]);
        clock.tick();
        update(&mut recency, &["h", "i", "j", "k", "j", "k"]);
        check(
            &recency,
            &[&["a", "b", "h", "i", "c", "d"], &["e", "f", "g", "j", "k"]],
        );
        clock.tick();
        update(&mut recency, &["l", "m", "n", "n", "o"]);
        check(
            &recency,
            &[
                &["a", "b", "h", "i", "c", "d", "l", "m", "e", "o"],
                &["f", "g", "j", "k", "n"],
            ],
        );
        clock.tick();
        assert_eq!(
            recency.remove_overall_least_recent_outside(Duration::from_secs(2)),
            Ok(vec!["a", "b", "c", "d", "e", "f", "g"])
        );
        check(&recency, &[&["h", "i", "l", "m", "o"], &["j", "k", "n"]]);
    }

    #[test]
    fn overall_recency_not_enabled() {
        let mut list: SegmentedRecencyList<&str, NoClock> = SegmentedRecencyList::empty(&[2, 3]);
        list.update("a");
        let error = list
            .remove_overall_least_recent_outside(Duration::from_secs(1))
            .expect_err("no clock");
        assert_eq!(
            (error.java_class(), error.message()),
            (
                "UnsupportedOperationException",
                "Overall recency is not enabled"
            )
        );
        assert_eq!(list.remove_least_recent(), ["a"]);
        list.remove(&"a").update("b").remove(&"b");
        assert_eq!(list.size(), 0);
    }

    #[test]
    fn single_segment_and_the_same_value_removed_twice() {
        let mut list = plain(&[3]);
        update(&mut list, &["a", "b", "c", "d"]);
        assert_eq!(list.remove_least_recent_over_limit(), ["a"]);
        list.update("b");
        assert_eq!(
            list.least_to_most_recent_of(0).copied().collect::<Vec<_>>(),
            ["c", "d", "b"]
        );
        list.remove(&"zz").remove(&"c");
        assert_eq!(list.size(), 2);
    }
}
