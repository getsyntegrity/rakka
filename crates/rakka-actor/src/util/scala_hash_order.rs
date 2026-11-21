// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala (Apache-2.0).
//! The iteration order of a Scala 2.13 `immutable.HashMap[String, _]` (Q-UTIL-21, Q-X-21), which decides which of
//! several matching `prefix*` entries `WildcardIndex` returns (`collectFirst` over `wildcardSuffixChildren`).
//!
//! Behaviour taken from measurements of the real map (`oracle/run.sh deployer-all`, cases DL-60..), not from any source: the
//! key's `String.hashCode` is scrambled by `improve`; the trie then consumes the scrambled hash five bits at a time from the
//! least significant end; in every node the entries that sit there alone come first, in ascending order of their five bits,
//! followed by the child nodes in ascending order of theirs; keys whose scrambled hashes are fully equal share a collision
//! node that keeps their insertion order. The order of a set of keys does not depend on how they were inserted, except among
//! fully colliding keys.

/// `String.hashCode`: polynomial 31 over the UTF-16 code units.
pub(crate) fn string_hash_code(text: &str) -> i32 {
    text.encode_utf16().fold(0_i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}

/// `improve`: the scrambling that `immutable.HashMap` applies to `key.##` before using its bits.
fn improve(hash_code: i32) -> u32 {
    let mut hash = hash_code.wrapping_add(!(hash_code.wrapping_shl(9)));
    hash ^= ((hash as u32) >> 14) as i32;
    hash = hash.wrapping_add(hash.wrapping_shl(4));
    (hash ^ (((hash as u32) >> 10) as i32)) as u32
}

const BITS_PER_LEVEL: u32 = 5;
const HASH_BITS: u32 = 32;

/// The iteration order of the entries `(position, scrambled hash)` that share the bits below `shift`: lone entries by their
/// next five bits, then the groups of two or more (a child node each) by theirs; at the end of the hash, insertion order.
fn node_order(entries: Vec<(usize, u32)>, shift: u32, out: &mut Vec<usize>) {
    if shift >= HASH_BITS {
        out.extend(entries.into_iter().map(|(position, _)| position));
        return;
    }
    let mut groups: Vec<Vec<(usize, u32)>> = vec![Vec::new(); 1 << BITS_PER_LEVEL];
    for entry in entries {
        groups[((entry.1 >> shift) & 0x1f) as usize].push(entry);
    }
    for group in groups.iter().filter(|group| group.len() == 1) {
        out.push(group[0].0);
    }
    for group in groups.into_iter().filter(|group| group.len() > 1) {
        node_order(group, shift + BITS_PER_LEVEL, out);
    }
}

/// The positions of `keys` in the iteration order of the map that holds them (given in insertion order).
pub(crate) fn iteration_order(keys: &[&str]) -> Vec<usize> {
    let entries = keys
        .iter()
        .enumerate()
        .map(|(position, key)| (position, improve(string_hash_code(key))))
        .collect();
    let mut order = Vec::with_capacity(keys.len());
    node_order(entries, 0, &mut order);
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys in the order the real `HashMap` iterates them (`Q21b` probe against Pekko 2.0.0, JDK 17), each inserted both in
    /// the iteration order and in the reverse one.
    const MEASURED_ORDERS: &[&[&str]] = &[
        &["p", "pq"],
        &["st", "s"],
        &["a", "ab"],
        &["bc", "b"],
        &["abc", "a", "ab"],
        &["u", "us", "user"],
        &["work", "wo", "worker"],
        &["a", "ab", "b"],
    ];

    fn ordered<'a>(keys: &[&'a str]) -> Vec<&'a str> {
        iteration_order(keys)
            .into_iter()
            .map(|index| keys[index])
            .collect()
    }

    #[test]
    fn keys_iterate_in_the_order_measured_in_the_jvm_whatever_the_insertion_order() {
        for measured in MEASURED_ORDERS {
            let reversed: Vec<&str> = measured.iter().rev().copied().collect();
            assert_eq!(
                &ordered(measured),
                measured,
                "inserted in the iteration order"
            );
            assert_eq!(
                &ordered(&reversed),
                measured,
                "inserted in the reverse order"
            );
        }
    }

    #[test]
    fn string_hash_code_is_the_java_polynomial_over_utf16_units() {
        assert_eq!(string_hash_code(""), 0);
        assert_eq!(string_hash_code("a"), 97);
        assert_eq!(string_hash_code("Aa"), 2112);
        assert_eq!(string_hash_code("BB"), 2112);
        assert_eq!(string_hash_code("\u{1F600}"), 0xD83D_i32 * 31 + 0xDE00);
        assert_eq!(string_hash_code("polygenelubricants"), i32::MIN);
    }
}
