// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Index.scala (Apache-2.0).
//! `Index` / `ConcurrentMultiMap` (AC-UTIL-007): a concurrent multimap from a key to the values it
//! holds, ordered by a comparator and without duplicates (two values the comparator calls equal are
//! one value).
//!
//! Q-UTIL-10 (a): one `Mutex` over standard collections instead of a `ConcurrentSkipListSet` per key.
//! Every read returns a snapshot taken under the lock, so an iterator or a callback never holds the
//! lock (a `foreach` callback may call back into the index) and never sees a later change. Pekko's
//! iteration is weakly consistent; a snapshot is the strongest case of that. The map is `Sync`, so it
//! is shared with `Arc`. The order of the keys is not specified.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Mutex, MutexGuard, PoisonError};

type ValueOrder<V> = Box<dyn Fn(&V, &V) -> Ordering + Send + Sync>;

/// `Index[K, V]`. `put`/`remove` of one key are serialised; here the whole map is.
pub struct Index<K, V> {
    map_size: usize,
    value_order: ValueOrder<V>,
    container: Mutex<HashMap<K, Vec<V>>>,
}

/// `ConcurrentMultiMap`: a subclass of `Index` with no behaviour of its own.
pub type ConcurrentMultiMap<K, V> = Index<K, V>;

impl<K: Eq + Hash, V> Index<K, V> {
    /// `new Index(mapSize, valueComparator)`: `map_size` is the initial capacity of the map.
    pub fn new(
        map_size: usize,
        value_order: impl Fn(&V, &V) -> Ordering + Send + Sync + 'static,
    ) -> Self {
        Self {
            map_size,
            value_order: Box::new(value_order),
            container: Mutex::new(HashMap::with_capacity(map_size)),
        }
    }

    /// `mapSize`.
    pub fn map_size(&self) -> usize {
        self.map_size
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<K, Vec<V>>> {
        self.container
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Where `value` is, or would go, in the ordered values of a key.
    fn position(&self, values: &[V], value: &V) -> Result<usize, usize> {
        values.binary_search_by(|present| (self.value_order)(present, value))
    }

    /// `put(key, value)`: true if the value was not associated with the key before.
    pub fn put(&self, key: K, value: V) -> bool {
        let mut container = self.lock();
        let values = container.entry(key).or_default();
        match self.position(values, &value) {
            Ok(_) => false,
            Err(at) => {
                values.insert(at, value);
                true
            }
        }
    }

    /// `remove(key, value)`: true if the value was associated with the key (the key goes away when it
    /// was the last value).
    pub fn remove(&self, key: &K, value: &V) -> bool {
        let mut container = self.lock();
        let Some(values) = container.get_mut(key) else {
            return false;
        };
        let Ok(at) = self.position(values, value) else {
            return false;
        };
        values.remove(at);
        if values.is_empty() {
            container.remove(key);
        }
        true
    }

    /// `remove(key)`: every value of the key, in order, or `None` if the key was not associated.
    pub fn remove_key(&self, key: &K) -> Option<Vec<V>> {
        self.lock().remove(key)
    }

    /// `removeValue(value)`: removes the value from every key and drops the keys left empty.
    pub fn remove_value(&self, value: &V) {
        let mut container = self.lock();
        container.retain(|_, values| {
            if let Ok(at) = self.position(values, value) {
                values.remove(at);
            }
            !values.is_empty()
        });
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// `clear()`: removes every key and every value.
    pub fn clear(&self) {
        self.lock().clear();
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Index<K, V> {
    /// `findValue(key)(f)`: the first value, in order, for which `f` holds. `f` runs on a snapshot,
    /// outside the lock.
    pub fn find_value(&self, key: &K, f: impl Fn(&V) -> bool) -> Option<V> {
        self.value_iterator(key).find(|value| f(value))
    }

    /// `valueIterator(key)`: the values of the key, in order, as they were when it was called; empty
    /// if the key is not associated.
    pub fn value_iterator(&self, key: &K) -> std::vec::IntoIter<V> {
        self.lock()
            .get(key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
    }

    /// `foreach(fun)`: applies `fun` to every key and each of its values (in order per key). It runs on
    /// a snapshot, outside the lock.
    pub fn foreach(&self, mut fun: impl FnMut(&K, &V)) {
        let snapshot: Vec<(K, Vec<V>)> = self
            .lock()
            .iter()
            .map(|(key, values)| (key.clone(), values.clone()))
            .collect();
        for (key, values) in &snapshot {
            for value in values {
                fun(key, value);
            }
        }
    }

    /// `keys`: a snapshot of the keys.
    pub fn keys(&self) -> Vec<K> {
        self.lock().keys().cloned().collect()
    }
}

impl<K: Eq + Hash, V: Clone + Eq + Hash> Index<K, V> {
    /// `values`: the union of the values of every key.
    pub fn values(&self) -> HashSet<V> {
        self.lock().values().flatten().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    //! `IndexSpec` of Pekko 2.0.0 (actor-tests/.../util/IndexSpec.scala), one test per `in` block.
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    fn empty_index() -> Index<String, i32> {
        Index::new(100, |a: &i32, b: &i32| a.cmp(b))
    }

    fn index_with_values() -> Index<String, i32> {
        let index = empty_index();
        for (key, value) in [
            ("s1", 1),
            ("s1", 2),
            ("s1", 3),
            ("s2", 1),
            ("s2", 2),
            ("s3", 2),
        ] {
            index.put(key.to_string(), value);
        }
        index
    }

    fn values_of(index: &Index<String, i32>, key: &str) -> BTreeSet<i32> {
        index.value_iterator(&key.to_string()).collect()
    }

    #[test]
    fn take_and_return_a_value() {
        let index = empty_index();
        index.put("s1".into(), 1);
        assert_eq!(values_of(&index, "s1"), BTreeSet::from([1]));
    }

    #[test]
    fn take_and_return_several_values() {
        let index = empty_index();
        assert!(index.put("s1".into(), 1));
        assert!(!index.put("s1".into(), 1));
        index.put("s1".into(), 2);
        index.put("s1".into(), 3);
        index.put("s2".into(), 4);
        assert_eq!(values_of(&index, "s1"), BTreeSet::from([1, 2, 3]));
        assert_eq!(values_of(&index, "s2"), BTreeSet::from([4]));
    }

    #[test]
    fn remove_values() {
        let index = empty_index();
        for (key, value) in [("s1", 1), ("s1", 2), ("s2", 1), ("s2", 2)] {
            index.put(key.to_string(), value);
        }
        assert!(index.remove(&"s1".to_string(), &1));
        assert!(!index.remove(&"s1".to_string(), &1));
        assert_eq!(values_of(&index, "s1"), BTreeSet::from([2]));
        let removed = index
            .remove_key(&"s2".to_string())
            .expect("s2 was associated");
        assert_eq!(
            removed.into_iter().collect::<BTreeSet<_>>(),
            BTreeSet::from([1, 2])
        );
        assert_eq!(index.remove_key(&"s2".to_string()), None);
        assert!(values_of(&index, "s2").is_empty());
    }

    #[test]
    fn remove_the_specified_value() {
        let index = index_with_values();
        index.remove_value(&1);
        assert_eq!(values_of(&index, "s1"), BTreeSet::from([2, 3]));
        assert_eq!(values_of(&index, "s2"), BTreeSet::from([2]));
        assert_eq!(values_of(&index, "s3"), BTreeSet::from([2]));
    }

    #[test]
    fn apply_a_function_for_all_key_value_pairs_and_find_every_value() {
        let index = index_with_values();
        let mut value_count = 0;
        index.foreach(|key, value| {
            value_count += 1;
            // the callback may use the index: it does not run under the lock
            assert_eq!(index.find_value(key, |v| v == value), Some(*value));
        });
        assert_eq!(value_count, 6);
    }

    #[test]
    fn be_cleared() {
        let index = index_with_values();
        assert!(!index.is_empty());
        index.clear();
        assert!(index.is_empty());
    }

    #[test]
    fn be_able_to_be_accessed_in_parallel() {
        let (keys, values, threads, tasks_per_thread) = (10_i32, 10_i32, 8_u64, 1250);
        let index = Arc::new(Index::<i32, i32>::new(100, |a, b| a.cmp(b)));
        for key in 0..keys {
            for value in 0..values {
                index.put(key, value);
            }
        }
        let go = Arc::new(std::sync::Barrier::new(threads as usize));
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                let (index, go) = (index.clone(), go.clone());
                std::thread::spawn(move || {
                    let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ (worker + 1);
                    let mut next = |bound: i32| {
                        state ^= state >> 12;
                        state ^= state << 25;
                        state ^= state >> 27;
                        ((state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) % bound as u64) as i32
                    };
                    go.wait();
                    for _ in 0..tasks_per_thread {
                        match next(4) {
                            0 => {
                                index.put(next(keys), next(values));
                            }
                            1 => {
                                index.remove(&next(keys / 2), &next(values));
                            }
                            2 => {
                                index.remove_key(&next(keys / 2));
                            }
                            _ => {
                                let key = next(keys);
                                if key >= keys / 2 {
                                    assert!(index.value_iterator(&key).next().is_some());
                                }
                            }
                        }
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("no worker panicked");
        }
        for key in keys / 2..keys {
            assert_eq!(
                index.value_iterator(&key).collect::<Vec<_>>(),
                (0..values).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn values_follow_the_comparator_and_equal_values_collapse() {
        let index = Index::<&str, i32>::new(4, |a, b| b.cmp(a));
        index.put("a", 1);
        index.put("a", 3);
        index.put("a", 2);
        assert!(!index.put("a", 3));
        assert_eq!(
            index.value_iterator(&"a").collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(index.map_size(), 4);
    }

    #[test]
    fn a_key_disappears_with_its_last_value_and_keys_and_values_are_reported() {
        let index = empty_index();
        index.put("a".into(), 1);
        index.put("b".into(), 2);
        index.put("b".into(), 1);
        assert_eq!(index.keys().len(), 2);
        assert_eq!(index.values(), HashSet::from([1, 2]));
        assert!(index.remove(&"a".to_string(), &1));
        assert_eq!(index.keys(), vec!["b".to_string()]);
        assert!(!index.remove(&"zz".to_string(), &1));
        index.remove_value(&2);
        index.remove_value(&1);
        assert!(index.is_empty());
    }
}
