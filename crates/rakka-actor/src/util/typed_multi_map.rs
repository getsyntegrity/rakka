// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/TypedMultiMap.scala (Apache-2.0).
//! `TypedMultiMap` (AC-UTIL-014): an immutable multimap whose value type is a function of the key type.
//!
//! In Scala the dependency is a type constructor (`K[_ <: T]`); here it is the associated type of
//! [`TypedKey`]: a map of `K` holds, for each key, a set of `K::Value`, and inserting a value of any
//! other type does not compile. Every key of one map has the same Rust type `K` (a key type may be an
//! enum when a map needs keys of different shapes, with one value type for the enum).
//!
//! ```compile_fail
//! use rakka_actor::util::typed_multi_map::{TypedKey, TypedMultiMap};
//! #[derive(Clone, PartialEq, Eq, Hash)]
//! struct IntKey(i32);
//! impl TypedKey for IntKey { type Value = i32; }
//! // the value of an `IntKey` is an `i32`, not an `i64`
//! let _ = TypedMultiMap::<IntKey>::empty().inserted(IntKey(1), 42_i64);
//! ```
//!
//! ```compile_fail
//! use rakka_actor::util::typed_multi_map::{TypedKey, TypedMultiMap};
//! #[derive(Clone, PartialEq, Eq, Hash)]
//! struct IntKey(i32);
//! impl TypedKey for IntKey { type Value = i32; }
//! let _ = TypedMultiMap::<IntKey>::empty().removed(&IntKey(1), &42_i64);
//! ```

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// A key type and the type of the values stored under it.
pub trait TypedKey: Clone + Eq + Hash {
    type Value: Clone + Eq + Hash;
}

/// `TypedMultiMap[T, K]`: persistent; every change returns a new map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedMultiMap<K: TypedKey> {
    map: HashMap<K, HashSet<K::Value>>,
}

impl<K: TypedKey> Default for TypedMultiMap<K> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K: TypedKey> TypedMultiMap<K> {
    /// `TypedMultiMap.empty`.
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// `keySet`: the keys that are mapped to value sets (a set may be empty after [`TypedMultiMap::set_all`]).
    pub fn key_set(&self) -> HashSet<K> {
        self.map.keys().cloned().collect()
    }

    /// `inserted(key)(value)`: a map with the value added to the mappings of the key.
    #[must_use]
    pub fn inserted(&self, key: K, value: K::Value) -> Self {
        let mut map = self.map.clone();
        map.entry(key).or_default().insert(value);
        Self { map }
    }

    /// `get(key)`: all mappings of the key (empty if there are none).
    pub fn get(&self, key: &K) -> HashSet<K::Value> {
        self.map.get(key).cloned().unwrap_or_default()
    }

    /// `valueRemoved(value)`: a map with the value removed from every key. A key whose set was exactly
    /// that value goes away; a key with an empty set stays.
    #[must_use]
    pub fn value_removed(&self, value: &K::Value) -> Self {
        let map = self
            .map
            .iter()
            .filter(|(_, set)| !(set.len() == 1 && set.contains(value)))
            .map(|(key, set)| {
                let mut without = set.clone();
                without.remove(value);
                (key.clone(), without)
            })
            .collect();
        Self { map }
    }

    /// `keyRemoved(key)`: a map with every mapping of the key removed.
    #[must_use]
    pub fn key_removed(&self, key: &K) -> Self {
        let mut map = self.map.clone();
        map.remove(key);
        Self { map }
    }

    /// `removed(key)(value)`: a map without that one mapping; the key goes away with its last value.
    #[must_use]
    pub fn removed(&self, key: &K, value: &K::Value) -> Self {
        match self.map.get(key) {
            Some(set) if set.contains(value) => {
                let mut map = self.map.clone();
                let mut without = set.clone();
                without.remove(value);
                if without.is_empty() {
                    map.remove(key);
                } else {
                    map.insert(key.clone(), without);
                }
                Self { map }
            }
            _ => self.clone(),
        }
    }

    /// `setAll(key)(values)`: a map where the key maps to exactly these values (an empty set keeps the key).
    #[must_use]
    pub fn set_all(&self, key: K, values: HashSet<K::Value>) -> Self {
        let mut map = self.map.clone();
        map.insert(key, values);
        Self { map }
    }

    /// `++`: all entries of `other`, overwriting existing ones (it does not merge the value sets).
    #[must_use]
    pub fn plus(&self, other: &Self) -> Self {
        let mut map = self.map.clone();
        map.extend(other.map.iter().map(|(k, v)| (k.clone(), v.clone())));
        Self { map }
    }
}

impl<K: TypedKey> Hash for TypedMultiMap<K> {
    /// Independent of the iteration order: the entries' hashes are added.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut total = 0_u64;
        for (key, values) in &self.map {
            let mut entry = DefaultHasher::new();
            key.hash(&mut entry);
            let mut inner = 0_u64;
            for value in values {
                let mut each = DefaultHasher::new();
                value.hash(&mut each);
                inner = inner.wrapping_add(each.finish());
            }
            inner.hash(&mut entry);
            total = total.wrapping_add(entry.finish());
        }
        state.write_u64(total);
    }
}

#[cfg(test)]
mod tests {
    //! `TypedMultiMapSpec` of Pekko 2.0.0 (actor-tests/.../scala-2/.../TypedMultiMapSpec.scala); the two
    //! "reject invalid" tests are the `compile_fail` examples of the module documentation.
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct Key<T>(T);

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct MyValue<T>(T);

    impl TypedKey for Key<i32> {
        type Value = MyValue<i32>;
    }

    type Map = TypedMultiMap<Key<i32>>;

    fn set(values: &[i32]) -> HashSet<MyValue<i32>> {
        values.iter().map(|v| MyValue(*v)).collect()
    }

    #[test]
    fn retain_and_remove_values_for_the_same_key() {
        let m1 = Map::empty();
        let m2 = m1.inserted(Key(1), MyValue(42));
        assert_eq!(m2.get(&Key(1)), set(&[42]));
        assert_eq!(m2.removed(&Key(1), &MyValue(42)).get(&Key(1)), set(&[]));
        let m3 = m2.inserted(Key(1), MyValue(43));
        assert_eq!(m3.get(&Key(1)), set(&[42, 43]));
        assert_eq!(m3.removed(&Key(1), &MyValue(42)).get(&Key(1)), set(&[43]));
    }

    #[test]
    fn retain_and_remove_values_for_multiple_keys() {
        let m2 = Map::empty()
            .inserted(Key(1), MyValue(42))
            .inserted(Key(2), MyValue(43));
        assert_eq!(m2.get(&Key(1)), set(&[42]));
        assert_eq!(m2.removed(&Key(1), &MyValue(42)).get(&Key(1)), set(&[]));
        assert_eq!(m2.get(&Key(2)), set(&[43]));
        assert_eq!(m2.removed(&Key(1), &MyValue(42)).get(&Key(2)), set(&[43]));
    }

    #[test]
    fn remove_a_value_from_all_keys() {
        let m2 = Map::empty()
            .inserted(Key(1), MyValue(42))
            .inserted(Key(2), MyValue(43))
            .inserted(Key(2), MyValue(42));
        let m3 = m2.value_removed(&MyValue(42));
        assert_eq!(m3.get(&Key(1)), set(&[]));
        assert_eq!(m3.get(&Key(2)), set(&[43]));
        assert_eq!(m3.key_set(), HashSet::from([Key(2)]));
    }

    #[test]
    fn remove_all_values_from_a_key() {
        let m2 = Map::empty()
            .inserted(Key(1), MyValue(42))
            .inserted(Key(2), MyValue(43))
            .inserted(Key(2), MyValue(42));
        let m3 = m2.key_removed(&Key(1));
        assert_eq!(m3.get(&Key(1)), set(&[]));
        assert_eq!(m3.get(&Key(2)), set(&[42, 43]));
        assert_eq!(m3.key_set(), HashSet::from([Key(2)]));
    }

    #[test]
    fn set_all_with_an_empty_set_keeps_the_key_and_plus_overwrites() {
        let with_empty = Map::empty().set_all(Key(1), set(&[]));
        assert_eq!(with_empty.key_set(), HashSet::from([Key(1)]));
        assert_eq!(
            with_empty.value_removed(&MyValue(5)).key_set(),
            HashSet::from([Key(1)])
        );
        let left = Map::empty()
            .inserted(Key(1), MyValue(1))
            .inserted(Key(1), MyValue(2))
            .inserted(Key(2), MyValue(3));
        let right = Map::empty()
            .inserted(Key(1), MyValue(9))
            .inserted(Key(4), MyValue(4));
        let merged = left.plus(&right);
        assert_eq!(merged.get(&Key(1)), set(&[9]));
        assert_eq!(merged.get(&Key(2)), set(&[3]));
        assert_eq!(merged.get(&Key(4)), set(&[4]));
    }

    #[test]
    fn removing_an_absent_mapping_gives_an_equal_map_and_equal_maps_hash_alike() {
        let hash_of = |map: &Map| {
            let mut hasher = DefaultHasher::new();
            map.hash(&mut hasher);
            hasher.finish()
        };
        let map = Map::empty().inserted(Key(1), MyValue(1));
        assert_eq!(map.removed(&Key(1), &MyValue(2)), map);
        assert_eq!(map.removed(&Key(9), &MyValue(1)), map);
        let other_order = Map::empty()
            .inserted(Key(1), MyValue(1))
            .inserted(Key(2), MyValue(2));
        let same = Map::empty()
            .inserted(Key(2), MyValue(2))
            .inserted(Key(1), MyValue(1));
        assert_eq!(hash_of(&other_order), hash_of(&same));
        assert_ne!(hash_of(&other_order), hash_of(&map));
        assert_eq!(Map::default(), Map::empty());
    }
}
