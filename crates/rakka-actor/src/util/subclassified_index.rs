// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala (Apache-2.0).
//! `SubclassifiedIndex` and `Subclassification` (AC-UTIL-008): a mutable index that respects the
//! sub-class relation between its keys. Adding a key inherits the values of the most specific known
//! super-class; adding or removing a value applies to every sub-class. It only keeps the tree and
//! returns the changed key/value sets, so that a cache in front of it (the `EventStream`'s) can be
//! updated; values are never traversed.
//!
//! Rust has no inheritance: the hierarchy is described by a [`Subclassification`] the user provides
//! (Q-X-6 (A)). The tree of Pekko is kept in one arena (`nodes`, the root first) and nodes refer to
//! each other by index. The order of the returned changes follows the first change of each key (Pekko
//! returns a Scala `Map` order that depends on the size of the map): do not rely on it.

use std::collections::HashSet;
use std::hash::Hash;

/// Typeclass which describes a classification hierarchy. Observe the contract between `is_equal` and
/// `is_subclass`!
pub trait Subclassification<K> {
    /// True if and only if x and y are of the same class.
    fn is_equal(&self, x: &K, y: &K) -> bool;

    /// True if and only if x is a subclass of y; equal classes must be considered sub-classes!
    fn is_subclass(&self, x: &K, y: &K) -> bool;
}

/// `Changes = Seq[(K, Set[V])]`.
pub type Changes<K, V> = Vec<(K, HashSet<V>)>;

struct Node<K, V> {
    /// `None` for the root.
    key: Option<K>,
    values: HashSet<V>,
    subkeys: Vec<usize>,
}

const ROOT: usize = 0;

/// `SubclassifiedIndex[K, V]`.
pub struct SubclassifiedIndex<K, V, S> {
    classification: S,
    nodes: Vec<Node<K, V>>,
}

impl<K, V, S> SubclassifiedIndex<K, V, S>
where
    K: Clone + Eq + Hash,
    V: Clone + Eq + Hash,
    S: Subclassification<K>,
{
    /// `new SubclassifiedIndex()`.
    pub fn new(classification: S) -> Self {
        Self {
            classification,
            nodes: vec![Node {
                key: None,
                values: HashSet::new(),
                subkeys: Vec::new(),
            }],
        }
    }

    fn key(&self, node: usize) -> &K {
        self.nodes[node]
            .key
            .as_ref()
            .expect("only the root has no key, and it is never asked for it")
    }

    fn new_node(&mut self, key: K, values: HashSet<V>) -> usize {
        self.nodes.push(Node {
            key: Some(key),
            values,
            subkeys: Vec::new(),
        });
        self.nodes.len() - 1
    }

    /// `addKey(key)`: adds the key to the index, inheriting the value set of the most specific known
    /// super-class. Returns the diff that should be added to the cache.
    pub fn add_key(&mut self, key: &K) -> Changes<K, V> {
        let changes = self.inner_add_key(ROOT, key);
        Self::merge_changes_by_key(changes)
    }

    fn inner_add_key(&mut self, node: usize, key: &K) -> Changes<K, V> {
        let mut found = false;
        let mut changes = Vec::new();
        for n in self.nodes[node].subkeys.clone() {
            if self.classification.is_equal(key, self.key(n)) {
                found = true;
            } else if self.classification.is_subclass(key, self.key(n)) {
                found = true;
                changes.extend(self.inner_add_key(n, key));
            }
        }
        if found {
            changes
        } else {
            let values = self.nodes[node].values.clone();
            let created = self.new_node(key.clone(), values.clone());
            let mut changes = self.integrate(node, created);
            changes.push((key.clone(), values));
            changes
        }
    }

    /// `addValue(key, value)`: adds the value to all keys that are sub-classes of the key; an unknown
    /// key is inserted as by `add_key`. Returns the diff that should be added to the cache.
    pub fn add_value(&mut self, key: &K, value: &V) -> Changes<K, V> {
        let changes = self.inner_add_value(ROOT, key, value);
        Self::merge_changes_by_key(changes)
    }

    fn inner_add_value(&mut self, node: usize, key: &K, value: &V) -> Changes<K, V> {
        // a node of the key itself switches to the recursive add-to-set
        if node != ROOT && self.classification.is_equal(key, self.key(node)) {
            return self.add_value_below(node, value);
        }
        let mut found = false;
        let mut changes = Vec::new();
        for n in self.nodes[node].subkeys.clone() {
            if self.classification.is_subclass(key, self.key(n)) {
                found = true;
                changes.extend(self.inner_add_value(n, key, value));
            }
        }
        if found {
            changes
        } else {
            let mut values = self.nodes[node].values.clone();
            values.insert(value.clone());
            let created = self.new_node(key.clone(), values.clone());
            let mut changes = self.integrate(node, created);
            changes.extend(self.inner_add_value(created, key, value));
            changes.push((key.clone(), values));
            changes
        }
    }

    /// `Nonroot.addValue`: the value goes to the node and, recursively, to its sub-classes.
    fn add_value_below(&mut self, node: usize, value: &V) -> Changes<K, V> {
        let mut changes = Vec::new();
        for n in self.nodes[node].subkeys.clone() {
            changes.extend(self.add_value_below(n, value));
        }
        if self.nodes[node].values.insert(value.clone()) {
            changes.push((self.key(node).clone(), HashSet::from([value.clone()])));
        }
        changes
    }

    /// `removeValue(key, value)`: removes the value from all keys that are sub-classes of the key.
    /// Returns the complete changes that should be updated in the cache: the whole tree is searched
    /// for the values of the changed keys, since new nodes might have been created.
    pub fn remove_value_of(&mut self, key: &K, value: &V) -> Changes<K, V> {
        let changes = Self::merge_changes_by_key(self.inner_remove_value(ROOT, key, value));
        changes
            .into_iter()
            .map(|(k, _)| {
                let values = self.find_values(&k);
                (k, values)
            })
            .collect()
    }

    fn inner_remove_value(&mut self, node: usize, key: &K, value: &V) -> Changes<K, V> {
        if node != ROOT && self.classification.is_equal(key, self.key(node)) {
            return self.remove_value_below(node, value);
        }
        let mut found = false;
        let mut changes = Vec::new();
        for n in self.nodes[node].subkeys.clone() {
            if self.classification.is_subclass(key, self.key(n)) {
                found = true;
                changes.extend(self.inner_remove_value(n, key, value));
            }
        }
        if found {
            changes
        } else {
            let values = self.nodes[node].values.clone();
            let created = self.new_node(key.clone(), values);
            let mut changes = self.integrate(node, created);
            changes.extend(self.remove_value_below(created, value));
            changes
        }
    }

    /// `Nonroot.removeValue`: the value leaves the node and, recursively, its sub-classes.
    fn remove_value_below(&mut self, node: usize, value: &V) -> Changes<K, V> {
        let mut changes = Vec::new();
        for n in self.nodes[node].subkeys.clone() {
            changes.extend(self.remove_value_below(n, value));
        }
        if self.nodes[node].values.remove(value) {
            changes.push((self.key(node).clone(), HashSet::from([value.clone()])));
        }
        changes
    }

    /// `removeValue(value)`: removes the value from all keys in the index. Returns the diff that should
    /// be removed from the cache.
    pub fn remove_value(&mut self, value: &V) -> Changes<K, V> {
        let mut changes = Vec::new();
        for n in self.nodes[ROOT].subkeys.clone() {
            changes.extend(self.remove_value_below(n, value));
        }
        Self::merge_changes_by_key(changes)
    }

    /// `findValues(key)`: all values for the key in the index.
    pub fn find_values(&self, key: &K) -> HashSet<V> {
        self.inner_find_values(ROOT, key)
    }

    fn inner_find_values(&self, node: usize, key: &K) -> HashSet<V> {
        if node != ROOT && self.classification.is_equal(key, self.key(node)) {
            return self.nodes[node].values.clone();
        }
        let mut found = HashSet::new();
        for &n in &self.nodes[node].subkeys {
            if self.classification.is_subclass(key, self.key(n)) {
                found.extend(self.inner_find_values(n, key));
            }
        }
        found
    }

    /// `innerFindSubKeys`: the sub-keys of `key` below `node`, excluding some. Insertion ordered.
    fn inner_find_sub_keys(&self, node: usize, key: &K, except: &[usize]) -> Vec<K> {
        let mut found: Vec<K> = Vec::new();
        for &n in &self.nodes[node].subkeys {
            if self.classification.is_equal(key, self.key(n)) {
                continue;
            }
            for below in self.inner_find_sub_keys(n, key, except) {
                if !found.contains(&below) {
                    found.push(below);
                }
            }
            if self.classification.is_subclass(self.key(n), key)
                && !except
                    .iter()
                    .any(|&e| self.classification.is_equal(key, self.key(e)))
                && !found.contains(self.key(n))
            {
                found.push(self.key(n).clone());
            }
        }
        found
    }

    /// `integrate`: adds the new node `n` below `node` and checks all existing nodes for a sub-class
    /// relationship; it also finds sub-keys in other parts of the tree, to compensate for multiple
    /// inheritance.
    fn integrate(&mut self, node: usize, n: usize) -> Changes<K, V> {
        let n_key = self.key(n).clone();
        let (sub_sub, sub): (Vec<usize>, Vec<usize>) = self.nodes[node]
            .subkeys
            .clone()
            .into_iter()
            .partition(|&k| self.classification.is_subclass(self.key(k), &n_key));
        let mut kept = sub;
        kept.push(n);
        self.nodes[node].subkeys = kept;
        if !sub_sub.is_empty() {
            self.nodes[n].subkeys = sub_sub;
        }
        let except = self.nodes[n].subkeys.clone();
        let inherited_values = self.nodes[node].values.clone();
        for k in self.inner_find_sub_keys(ROOT, &n_key, &except) {
            let created = self.new_node(k, inherited_values.clone());
            self.nodes[n].subkeys.push(created);
        }
        self.nodes[n]
            .subkeys
            .iter()
            .map(|&s| (self.key(s).clone(), self.nodes[s].values.clone()))
            .collect()
    }

    /// `mergeChangesByKey`: one entry per key with the union of its sets, in order of first appearance.
    fn merge_changes_by_key(changes: Changes<K, V>) -> Changes<K, V> {
        let mut merged: Changes<K, V> = Vec::new();
        for (key, values) in changes {
            match merged.iter_mut().find(|(k, _)| *k == key) {
                Some((_, present)) => present.extend(values),
                None => merged.push((key, values)),
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    //! The hierarchy is the divisibility of integers (`x` is a sub-class of `y` when `y` divides `x`):
    //! 12 has two super-classes, 4 and 6 (multiple inheritance). No spec of Pekko tests the index
    //! directly (`EventStreamSpec` does through the `EventStream`): the expectations are the values the
    //! 2.0.0 jar printed for the same operations (cases SI-01..SI-06).
    use super::*;

    struct Divisibility;

    impl Subclassification<u32> for Divisibility {
        fn is_equal(&self, x: &u32, y: &u32) -> bool {
            x == y
        }

        fn is_subclass(&self, x: &u32, y: &u32) -> bool {
            x.is_multiple_of(*y)
        }
    }

    type Index = SubclassifiedIndex<u32, &'static str, Divisibility>;

    fn index() -> Index {
        SubclassifiedIndex::new(Divisibility)
    }

    fn sorted(changes: Changes<u32, &'static str>) -> Vec<(u32, Vec<&'static str>)> {
        let mut out: Vec<(u32, Vec<&'static str>)> = changes
            .into_iter()
            .map(|(k, v)| {
                let mut values: Vec<_> = v.into_iter().collect();
                values.sort_unstable();
                (k, values)
            })
            .collect();
        out.sort_unstable();
        out
    }

    fn values(index: &Index, key: u32) -> Vec<&'static str> {
        let mut found: Vec<_> = index.find_values(&key).into_iter().collect();
        found.sort_unstable();
        found
    }

    #[test]
    fn a_key_inherits_the_values_of_its_superclass() {
        let mut index = index();
        assert_eq!(sorted(index.add_value(&2, &"x")), vec![(2, vec!["x"])]);
        assert_eq!(sorted(index.add_key(&4)), vec![(4, vec!["x"])]);
        assert_eq!(values(&index, 4), vec!["x"]);
        assert_eq!(sorted(index.add_key(&3)), vec![(3, vec![])]);
        assert_eq!(values(&index, 3), Vec::<&str>::new());
        assert_eq!(values(&index, 2), vec!["x"]);
        assert_eq!(values(&index, 1), Vec::<&str>::new());
    }

    #[test]
    fn adding_a_value_reaches_every_subclass() {
        let mut index = index();
        for key in [4, 6, 12] {
            assert_eq!(sorted(index.add_key(&key)), vec![(key, vec![])]);
        }
        assert_eq!(
            sorted(index.add_value(&2, &"x")),
            vec![
                (2, vec!["x"]),
                (4, vec!["x"]),
                (6, vec!["x"]),
                (12, vec!["x"])
            ]
        );
        assert_eq!(
            sorted(index.add_value(&3, &"y")),
            vec![(3, vec!["y"]), (6, vec!["y"]), (12, vec!["y"])]
        );
        assert_eq!(values(&index, 6), vec!["x", "y"]);
        assert_eq!(values(&index, 12), vec!["x", "y"]);
        assert_eq!(values(&index, 3), vec!["y"]);
        assert_eq!(values(&index, 2), vec!["x"]);
    }

    #[test]
    fn multiple_inheritance_gathers_the_values_of_both_superclasses() {
        let mut index = index();
        index.add_value(&4, &"a");
        index.add_value(&6, &"b");
        assert_eq!(sorted(index.add_key(&12)), vec![(12, vec!["a", "b"])]);
        assert_eq!(sorted(index.add_value(&12, &"c")), vec![(12, vec!["c"])]);
        assert_eq!(values(&index, 12), vec!["a", "b", "c"]);
        assert_eq!(values(&index, 4), vec!["a"]);
        assert_eq!(values(&index, 6), vec!["b"]);
    }

    #[test]
    fn remove_a_value_by_key_and_from_every_key() {
        let mut index = index();
        index.add_value(&2, &"x");
        index.add_value(&2, &"y");
        index.add_key(&4);
        index.add_key(&6);
        assert_eq!(
            sorted(index.remove_value_of(&2, &"x")),
            vec![(2, vec!["y"]), (4, vec!["y"]), (6, vec!["y"])]
        );
        assert_eq!(values(&index, 4), vec!["y"]);
        assert_eq!(
            sorted(index.remove_value(&"y")),
            vec![(2, vec!["y"]), (4, vec!["y"]), (6, vec!["y"])]
        );
        assert!(values(&index, 2).is_empty() && values(&index, 4).is_empty());
        assert!(index.remove_value_of(&8, &"z").is_empty());
        assert!(values(&index, 8).is_empty());
    }

    #[test]
    fn a_new_key_slots_between_existing_ones() {
        let mut index = index();
        assert_eq!(sorted(index.add_key(&36)), vec![(36, vec![])]);
        assert_eq!(sorted(index.add_key(&2)), vec![(2, vec![]), (36, vec![])]);
        assert_eq!(
            sorted(index.add_value(&2, &"x")),
            vec![(2, vec!["x"]), (36, vec!["x"])]
        );
        assert_eq!(
            sorted(index.add_key(&6)),
            vec![(6, vec!["x"]), (36, vec!["x"])]
        );
        assert_eq!(
            sorted(index.add_key(&12)),
            vec![(12, vec!["x"]), (36, vec!["x"])]
        );
        assert_eq!(
            sorted(index.add_key(&4)),
            vec![(4, vec!["x"]), (12, vec!["x"]), (36, vec!["x"])]
        );
        assert_eq!(values(&index, 36), vec!["x"]);
    }

    #[test]
    fn a_value_added_under_an_unknown_key_creates_it() {
        let mut index = index();
        assert_eq!(sorted(index.add_value(&6, &"a")), vec![(6, vec!["a"])]);
        assert_eq!(values(&index, 6), vec!["a"]);
        assert!(values(&index, 12).is_empty());
        assert_eq!(sorted(index.add_key(&12)), vec![(12, vec!["a"])]);
        assert_eq!(sorted(index.add_value(&12, &"b")), vec![(12, vec!["b"])]);
    }
}
