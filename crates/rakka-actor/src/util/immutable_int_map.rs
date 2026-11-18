// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ImmutableIntMap.scala (Apache-2.0).
//! `ImmutableIntMap` (AC-UTIL-016.4): an immutable `Int -> Int` map for small sizes (under about a
//! thousand entries), kept as one sorted array of keys and values and copied on every change, with no
//! structural sharing.

use std::fmt;
use std::hash::{Hash, Hasher};

/// `ImmutableIntMap`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImmutableIntMap {
    /// Keys and values one after the other, sorted by key.
    kvs: Vec<i32>,
}

impl ImmutableIntMap {
    /// `ImmutableIntMap.empty`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.kvs.len() / 2
    }

    /// Where the key is (`Ok`, index of its pair) or would be inserted (`Err`).
    fn index_for_key(&self, key: i32) -> Result<usize, usize> {
        let (mut low, mut high) = (0, self.size());
        while low < high {
            let middle = low + (high - low) / 2;
            match self.kvs[2 * middle].cmp(&key) {
                std::cmp::Ordering::Equal => return Ok(middle),
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
            }
        }
        Err(low)
    }

    /// `get(key)`: the value, or `Int.MinValue` if the key is absent (so beware of storing `Int.MinValue`).
    pub fn get(&self, key: i32) -> i32 {
        self.index_for_key(key)
            .map_or(i32::MIN, |pair| self.kvs[2 * pair + 1])
    }

    /// `contains(key)`.
    pub fn contains(&self, key: i32) -> bool {
        self.index_for_key(key).is_ok()
    }

    fn inserted(&self, at: usize, key: i32, value: i32) -> Self {
        let mut kvs = Vec::with_capacity(self.kvs.len() + 2);
        kvs.extend_from_slice(&self.kvs[..2 * at]);
        kvs.extend_from_slice(&[key, value]);
        kvs.extend_from_slice(&self.kvs[2 * at..]);
        Self { kvs }
    }

    /// `updateIfAbsent(key, value)`: a map with the key, if it was not there; `value` is only evaluated
    /// for a new key.
    pub fn update_if_absent(&self, key: i32, value: impl FnOnce() -> i32) -> Self {
        match self.index_for_key(key) {
            Ok(_) => self.clone(),
            Err(at) => self.inserted(at, key, value()),
        }
    }

    /// `updated(key, value)`: a map with the key set to the value.
    pub fn updated(&self, key: i32, value: i32) -> Self {
        match self.index_for_key(key) {
            Ok(pair) if self.kvs[2 * pair + 1] == value => self.clone(),
            Ok(pair) => {
                let mut kvs = self.kvs.clone();
                kvs[2 * pair + 1] = value;
                Self { kvs }
            }
            Err(at) => self.inserted(at, key, value),
        }
    }

    /// `remove(key)`: a map without the key.
    pub fn remove(&self, key: i32) -> Self {
        match self.index_for_key(key) {
            Ok(pair) => {
                let mut kvs = self.kvs.clone();
                kvs.drain(2 * pair..2 * pair + 2);
                Self { kvs }
            }
            Err(_) => self.clone(),
        }
    }

    /// `keysIterator`: every key, in ascending order.
    pub fn keys_iterator(&self) -> impl Iterator<Item = i32> + '_ {
        self.kvs.iter().step_by(2).copied()
    }

    /// `hashCode`: `java.util.Arrays.hashCode` of the key/value array (Q-UTIL-14 (b): the number is not
    /// part of what a consumer depends on, but it is what Pekko measures: 1281 for `{10 -> 10}`).
    pub fn hash_code(&self) -> i32 {
        self.kvs.iter().fold(1_i32, |hash, element| {
            hash.wrapping_mul(31).wrapping_add(*element)
        })
    }
}

impl fmt::Display for ImmutableIntMap {
    /// `toString`: `ImmutableIntMap(10 -> 10, 20 -> 20)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ImmutableIntMap(")?;
        for (n, pair) in self.kvs.as_chunks::<2>().0.iter().enumerate() {
            if n > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{} -> {}", pair[0], pair[1])?;
        }
        f.write_str(")")
    }
}

impl Hash for ImmutableIntMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

#[cfg(test)]
mod tests {
    //! `ImmutableIntMapSpec` of Pekko 2.0.0 (actor-tests/.../util/ImmutableIntMapSpec.scala).
    use super::*;
    use std::collections::BTreeMap;

    fn keys(map: &ImmutableIntMap) -> Vec<i32> {
        map.keys_iterator().collect()
    }

    fn values(map: &ImmutableIntMap) -> Vec<i32> {
        map.keys_iterator().map(|key| map.get(key)).collect()
    }

    #[test]
    fn have_no_entries_when_empty() {
        let empty = ImmutableIntMap::empty();
        assert_eq!(empty.size(), 0);
        assert!(keys(&empty).is_empty());
    }

    #[test]
    fn add_and_get_entries() {
        let m1 = ImmutableIntMap::empty().updated(10, 10);
        assert_eq!((keys(&m1), values(&m1)), (vec![10], vec![10]));
        let m2 = m1.updated(20, 20);
        assert_eq!((keys(&m2), values(&m2)), (vec![10, 20], vec![10, 20]));
        let m3 = m1.updated(5, 5);
        assert_eq!((keys(&m3), values(&m3)), (vec![5, 10], vec![5, 10]));
        let m4 = m2.updated(5, 5);
        assert_eq!(keys(&m4), vec![5, 10, 20]);
        let m5 = m4.updated(15, 15);
        assert_eq!(
            (keys(&m5), values(&m5)),
            (vec![5, 10, 15, 20], vec![5, 10, 15, 20])
        );
    }

    #[test]
    fn replace_entries() {
        let m1 = ImmutableIntMap::empty().updated(10, 10).updated(10, 11);
        assert_eq!(values(&m1), vec![11]);
        let m2 = m1
            .updated(20, 20)
            .updated(30, 30)
            .updated(20, 21)
            .updated(30, 31);
        assert_eq!(values(&m2), vec![11, 21, 31]);
    }

    #[test]
    fn update_if_absent_keeps_present_keys_and_adds_new_ones() {
        let m1 = ImmutableIntMap::empty().updated(10, 10).updated(20, 20);
        let mut evaluated = false;
        assert_eq!(
            m1.update_if_absent(10, || {
                evaluated = true;
                99
            }),
            m1
        );
        assert!(!evaluated);
        let m2 = m1.update_if_absent(15, || {
            evaluated = true;
            15
        });
        assert!(evaluated);
        assert_eq!(keys(&m2), vec![10, 15, 20]);
        assert_eq!(ImmutableIntMap::empty().update_if_absent(1, || 2).get(1), 2);
    }

    #[test]
    fn have_a_pretty_to_string() {
        let empty = ImmutableIntMap::empty();
        assert_eq!(empty.to_string(), "ImmutableIntMap()");
        let one = empty.updated(10, 10);
        assert_eq!(one.to_string(), "ImmutableIntMap(10 -> 10)");
        assert_eq!(
            one.updated(20, 20).to_string(),
            "ImmutableIntMap(10 -> 10, 20 -> 20)"
        );
    }

    #[test]
    fn behave_according_to_equals_and_hash_code() {
        let a = ImmutableIntMap::empty().updated(10, 10).updated(20, 20);
        let b = ImmutableIntMap::empty().updated(20, 20).updated(10, 10);
        assert_eq!((a.clone(), a.hash_code()), (b.clone(), b.hash_code()));
        assert_ne!(a, a.updated(20, 21));
        assert_ne!(a.hash_code(), a.updated(20, 21).hash_code());
        assert_ne!(a, a.updated(30, 30));
        assert_ne!(a, a.remove(10));
        assert_eq!(ImmutableIntMap::empty().updated(10, 10).hash_code(), 1281);
        assert_eq!(ImmutableIntMap::empty().hash_code(), 1);
        let mut hashed = std::collections::HashSet::new();
        assert!(hashed.insert(a.clone()) && !hashed.insert(b));
    }

    #[test]
    fn remove_entries() {
        let full = ImmutableIntMap::empty()
            .updated(10, 10)
            .updated(20, 20)
            .updated(30, 30)
            .updated(40, 40);
        assert_eq!(keys(&full.remove(10)), vec![20, 30, 40]);
        assert_eq!(keys(&full.remove(20)), vec![10, 30, 40]);
        assert_eq!(keys(&full.remove(40)), vec![10, 20, 30]);
        assert_eq!(full.remove(5), full);
        let one = ImmutableIntMap::empty().updated(10, 10);
        assert_eq!(one.remove(10), ImmutableIntMap::empty());
        assert_eq!(one.remove(5), one);
    }

    #[test]
    fn get_and_contains() {
        let m = ImmutableIntMap::empty()
            .updated(10, 10)
            .updated(7, i32::MIN);
        assert_eq!((m.get(11), m.get(9), m.get(10)), (i32::MIN, i32::MIN, 10));
        assert!(m.contains(10) && !m.contains(11) && m.contains(7));
    }

    #[test]
    fn behave_like_a_reference_map_on_a_thousand_random_operations() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = |bound: u64| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) % bound
        };
        let mut map = ImmutableIntMap::empty();
        let mut reference = BTreeMap::new();
        for _ in 0..1000 {
            let key = next(100) as i32;
            if next(2) == 0 {
                let value = next(1000) as i32;
                map = map.updated(key, value);
                reference.insert(key, value);
            } else {
                map = map.remove(key);
                reference.remove(&key);
            }
            assert_eq!(map.size(), reference.len());
            assert_eq!(keys(&map), reference.keys().copied().collect::<Vec<_>>());
            assert_eq!(
                values(&map),
                reference.values().copied().collect::<Vec<_>>()
            );
        }
    }
}
