// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala (Apache-2.0).
//! `WildcardIndex` (AC-UTIL-009): a persistent index from a path (a sequence of elements) to a value,
//! where `*` matches one element, `prefix*` matches the elements that start with the prefix, and a
//! trailing `**` matches every remaining element. It is how deployment paths with wildcards are looked
//! up.
//!
//! Faithful to 2.0.0, quirks included: `isEmpty` ignores the `prefix*` children, so an index holding
//! only `prefix*` entries reports empty and finds nothing; inserting `prefix*` again replaces the
//! subtree of that prefix (the lookup of the existing child uses the element with its `*`, which is
//! never a key). Elements are compared as `&str`; `prefix*` is a `starts_with` (Q-UTIL-8 (c): no
//! `makePattern`, no UTF-16 view of the text).
//!
//! The choice between two `prefix*` that both match an element is the first one in the iteration order of a
//! Scala `HashMap` (AC-UTIL-009 A7, Q-UTIL-21, Q-X-20): `wildcard_suffix_children` is kept in that order
//! (`scala_hash_order`), so it does not depend on the order the entries were inserted.

use std::collections::HashMap;

use super::scala_hash_order::iteration_order;
use crate::actor::IllegalArgumentError;

const DOUBLE_WILDCARD_SUFFIX_MESSAGE: &str = "double wildcard can't be used as a suffix (e.g. /user/actor**), only as a full subPath element (e.g. /user/actor/**)";

/// `WildcardIndex[T]`.
#[derive(Clone, Debug)]
pub struct WildcardIndex<T> {
    wildcard_tree: WildcardTree<T>,
    double_wildcard_tree: WildcardTree<T>,
}

/// `WildcardTree[T]`.
#[derive(Clone, Debug)]
pub struct WildcardTree<T> {
    data: Option<T>,
    children: HashMap<String, WildcardTree<T>>,
    /// The `prefix*` children by prefix, in the iteration order of Scala's `HashMap` (insertion order among equal hashes).
    wildcard_suffix_children: Vec<(String, WildcardTree<T>)>,
}

impl<T> Default for WildcardTree<T> {
    fn default() -> Self {
        Self {
            data: None,
            children: HashMap::new(),
            wildcard_suffix_children: Vec::new(),
        }
    }
}

impl<T> Default for WildcardIndex<T> {
    fn default() -> Self {
        Self {
            wildcard_tree: WildcardTree::default(),
            double_wildcard_tree: WildcardTree::default(),
        }
    }
}

impl<T: Clone> WildcardIndex<T> {
    /// `WildcardIndex[T]()`: the empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// `insert(elems, d)`: an index that also maps the path to `d`. An empty path changes nothing; a
    /// path whose last element is `**` goes in the double wildcard tree, any other in the single one.
    /// A `**` suffix inside an element (`actor**`) is an `IllegalArgumentException`.
    pub fn insert(&self, elems: &[&str], data: T) -> Result<Self, IllegalArgumentError> {
        match elems.last() {
            Some(&"**") => Ok(Self {
                wildcard_tree: self.wildcard_tree.clone(),
                double_wildcard_tree: self.double_wildcard_tree.insert(elems, data)?,
            }),
            Some(_) => Ok(Self {
                wildcard_tree: self.wildcard_tree.insert(elems, data)?,
                double_wildcard_tree: self.double_wildcard_tree.clone(),
            }),
            None => Ok(self.clone()),
        }
    }

    /// `find(elems)`: the value of the most specific entry that matches the path.
    pub fn find<'a>(&self, elems: impl IntoIterator<Item = &'a str>) -> Option<&T> {
        let mut elems = elems.into_iter();
        let found = if self.wildcard_tree.is_empty() {
            if self.double_wildcard_tree.is_empty() {
                None
            } else {
                self.double_wildcard_tree
                    .find_with_terminal_double_wildcard(&mut elems)
            }
        } else {
            // the iterator is consumed by the first search: the second one starts over
            let remaining: Vec<&str> = elems.collect();
            let with_single_wildcard = self
                .wildcard_tree
                .find_with_single_wildcard(&mut remaining.iter().copied());
            if with_single_wildcard.is_none_or(WildcardTree::is_empty) {
                self.double_wildcard_tree
                    .find_with_terminal_double_wildcard(&mut remaining.iter().copied())
            } else {
                with_single_wildcard
            }
        };
        found.and_then(|tree| tree.data.as_ref())
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.wildcard_tree.is_empty() && self.double_wildcard_tree.is_empty()
    }
}

impl<T: Clone> WildcardTree<T> {
    fn is_empty(&self) -> bool {
        self.data.is_none() && self.children.is_empty()
    }

    fn insert(&self, elems: &[&str], data: T) -> Result<Self, IllegalArgumentError> {
        let Some((element, rest)) = elems.split_first() else {
            return Ok(Self {
                data: Some(data),
                ..self.clone()
            });
        };
        let element = *element;
        if element != "**" && element.ends_with("**") {
            return Err(IllegalArgumentError::new(DOUBLE_WILDCARD_SUFFIX_MESSAGE));
        }
        let mut updated = self.clone();
        if element != "*" && element != "**" && element.ends_with('*') {
            // `getOrElse(e, ...)` looks the child up by the element with its `*`; the children are
            // stored under the prefix, so it is never found and the subtree is replaced.
            let child = WildcardTree::default().insert(rest, data)?;
            let prefix = element.strip_suffix('*').unwrap_or(element).to_owned();
            match updated
                .wildcard_suffix_children
                .iter_mut()
                .find(|(key, _)| *key == prefix)
            {
                Some((_, existing)) => *existing = child,
                None => updated.wildcard_suffix_children.push((prefix, child)),
            }
            updated.sort_suffix_children_in_hash_map_order();
        } else {
            let existing = self.children.get(element).cloned().unwrap_or_default();
            updated
                .children
                .insert(element.to_owned(), existing.insert(rest, data)?);
        }
        Ok(updated)
    }

    fn sort_suffix_children_in_hash_map_order(&mut self) {
        let keys: Vec<&str> = self
            .wildcard_suffix_children
            .iter()
            .map(|(prefix, _)| prefix.as_str())
            .collect();
        let order = iteration_order(&keys);
        let mut children: Vec<Option<(String, WildcardTree<T>)>> =
            std::mem::take(&mut self.wildcard_suffix_children)
                .into_iter()
                .map(Some)
                .collect();
        self.wildcard_suffix_children = order
            .into_iter()
            .filter_map(|position| children[position].take())
            .collect();
    }

    /// `findWithSingleWildcard`: `None` stands for the empty tree.
    fn find_with_single_wildcard<'s, 'e>(
        &'s self,
        elems: &mut impl Iterator<Item = &'e str>,
    ) -> Option<&'s WildcardTree<T>> {
        let mut node = self;
        loop {
            let Some(next) = elems.next() else {
                return Some(node);
            };
            node = if let Some(branch) = node.children.get(next) {
                branch
            } else if let Some(branch) = node.children.get("*") {
                branch
            } else if let Some((_, branch)) = node
                .wildcard_suffix_children
                .iter()
                .find(|(prefix, _)| next.starts_with(prefix.as_str()))
            {
                branch
            } else {
                return None;
            };
        }
    }

    /// `findWithTerminalDoubleWildcard`: `None` stands for the empty tree.
    fn find_with_terminal_double_wildcard<'s, 'e>(
        &'s self,
        elems: &mut impl Iterator<Item = &'e str>,
    ) -> Option<&'s WildcardTree<T>> {
        let mut node = self;
        let mut alternative: Option<&WildcardTree<T>> = None;
        loop {
            let Some(next) = elems.next() else {
                return Some(node);
            };
            let new_alternative = node.children.get("**").or(alternative);
            node = if let Some(branch) = node.children.get(next) {
                branch
            } else if let Some(branch) = node.children.get("*") {
                branch
            } else {
                return new_alternative;
            };
            alternative = new_alternative;
        }
    }
}

#[cfg(test)]
mod tests {
    //! `WildcardIndexSpec` of Pekko 2.0.0 (actor-tests/.../util/WildcardIndexSpec.scala).
    use super::*;

    fn empty_index() -> WildcardIndex<i32> {
        WildcardIndex::new()
    }

    fn insert(index: &WildcardIndex<i32>, elems: &[&str], data: i32) -> WildcardIndex<i32> {
        index.insert(elems, data).expect("a valid path")
    }

    fn find(index: &WildcardIndex<i32>, elems: &[&str]) -> Option<i32> {
        index.find(elems.iter().copied()).copied()
    }

    #[test]
    fn allow_to_insert_elements_using_arrays_of_strings() {
        assert!(!insert(&empty_index(), &["a", "b"], 1).is_empty());
        assert!(!insert(&empty_index(), &["a"], 1).is_empty());
        assert!(insert(&empty_index(), &[], 1).is_empty());
    }

    #[test]
    fn allow_to_find_inserted_elements() {
        let tree = insert(
            &insert(&insert(&empty_index(), &["a"], 1), &["a", "b"], 2),
            &["a", "c"],
            3,
        );
        assert_eq!(find(&tree, &["a", "b"]), Some(2));
        assert_eq!(find(&tree, &["a"]), Some(1));
        assert_eq!(find(&tree, &["x"]), None);
        assert_eq!(find(&tree, &[]), None);
    }

    #[test]
    fn match_all_elements_in_the_sub_array_when_it_contains_a_wildcard() {
        let tree1 = insert(&insert(&empty_index(), &["a"], 1), &["a", "*"], 1);
        assert_eq!(find(&tree1, &["z"]), None);
        assert_eq!(find(&tree1, &["a"]), Some(1));
        assert_eq!(find(&tree1, &["a", "b"]), Some(1));
        assert_eq!(find(&tree1, &["a", "x"]), Some(1));

        let tree2 = insert(&insert(&empty_index(), &["a", "*"], 1), &["a", "*", "c"], 2);
        assert_eq!(find(&tree2, &["z"]), None);
        assert_eq!(find(&tree2, &["a", "b"]), Some(1));
        assert_eq!(find(&tree2, &["a", "x"]), Some(1));
        assert_eq!(find(&tree2, &["a", "x", "c"]), Some(2));
        assert_eq!(find(&tree2, &["a", "x", "y"]), None);
    }

    #[test]
    fn match_all_elements_in_the_sub_array_when_it_contains_a_wildcard_suffix() {
        let tree1 = insert(&insert(&empty_index(), &["a*"], 1), &["b", "c*"], 2);
        assert_eq!(find(&tree1, &["z"]), None);
        assert_eq!(find(&tree1, &["a"]), Some(1));
        assert_eq!(find(&tree1, &["aa"]), Some(1));
        assert_eq!(find(&tree1, &["aa", "b"]), None);
        assert_eq!(find(&tree1, &["b", "c"]), Some(2));
        assert_eq!(find(&tree1, &["b", "cc"]), Some(2));
        assert_eq!(find(&tree1, &["b", "x"]), None);

        let tree2 = insert(
            &insert(&empty_index(), &["a", "b*"], 1),
            &["b", "c*", "d"],
            2,
        );
        assert_eq!(find(&tree2, &["z"]), None);
        assert_eq!(find(&tree2, &["a", "b"]), Some(1));
        assert_eq!(find(&tree2, &["a", "bb"]), Some(1));
        assert_eq!(find(&tree2, &["a", "c"]), None);
        assert_eq!(find(&tree2, &["b", "c", "d"]), Some(2));
        assert_eq!(find(&tree2, &["b", "cc", "d"]), Some(2));
        assert_eq!(find(&tree2, &["b", "c", "x"]), None);
        assert_eq!(find(&tree2, &["b", "cc", "x"]), None);
    }

    #[test]
    fn fail_when_a_double_wildcard_is_used_as_a_suffix() {
        let error = empty_index()
            .insert(&["a**"], 1)
            .expect_err("a double wildcard suffix");
        assert_eq!(error.java_class(), "IllegalArgumentException");
        assert_eq!(error.message(), DOUBLE_WILDCARD_SUFFIX_MESSAGE);
        assert!(empty_index().insert(&["a", "b**"], 1).is_err());
    }

    #[test]
    fn never_find_anything_when_empty_index() {
        assert_eq!(find(&empty_index(), &["a"]), None);
        assert_eq!(find(&empty_index(), &["a", "b"]), None);
        assert_eq!(find(&empty_index(), &[]), None);
    }

    #[test]
    fn match_all_remaining_elements_when_it_contains_a_terminal_double_wildcard() {
        let tree1 = insert(&empty_index(), &["a", "**"], 1);
        assert_eq!(find(&tree1, &["z"]), None);
        assert_eq!(find(&tree1, &["a", "b"]), Some(1));
        assert_eq!(find(&tree1, &["a", "x"]), Some(1));
        assert_eq!(find(&tree1, &["a", "x", "y"]), Some(1));

        let tree2 = insert(&empty_index(), &["**"], 1);
        assert_eq!(find(&tree2, &["anything", "I", "want"]), Some(1));
        assert_eq!(find(&tree2, &["anything"]), Some(1));
    }

    #[test]
    fn ignore_non_terminal_double_wildcards() {
        let tree = insert(&empty_index(), &["a", "**", "c"], 1);
        assert_eq!(find(&tree, &["a", "x", "y", "c"]), None);
        assert_eq!(find(&tree, &["a", "x", "y"]), None);
    }

    #[test]
    fn faithful_quirks_of_2_0_0() {
        // only `prefix*` entries: the index reports empty and finds nothing
        let only_suffix = insert(&empty_index(), &["a*"], 1);
        assert!(only_suffix.is_empty());
        assert_eq!(find(&only_suffix, &["aa"]), None);
        // exact elements win over `*`, `*` over `prefix*`; `**` is the fallback
        let tree = insert(
            &insert(&insert(&empty_index(), &["a", "b"], 1), &["a", "*"], 2),
            &["a", "**"],
            3,
        );
        assert_eq!(find(&tree, &["a", "b"]), Some(1));
        assert_eq!(find(&tree, &["a", "c"]), Some(2));
        assert_eq!(find(&tree, &["a", "c", "d"]), Some(3));
        // `prefix*` inserted again replaces the subtree of that prefix
        let replaced = insert(
            &insert(
                &insert(&empty_index(), &["x", "b*", "c"], 1),
                &["x", "b*"],
                2,
            ),
            &["x", "k"],
            0,
        );
        assert_eq!(find(&replaced, &["x", "bb", "c"]), None);
        assert_eq!(find(&replaced, &["x", "bb"]), Some(2));
    }

    #[test]
    fn overlapping_suffix_wildcards_resolve_as_the_scala_hash_map_iterates_them() {
        // winners measured in Pekko 2.0.0 (Q-UTIL-21 probe): the first matching key in the map's iteration order
        let cases: &[(&[&str], &str, &str)] = &[
            (&["p", "pq"], "pqzzz", "p"),
            (&["s", "st"], "stzzz", "st"),
            (&["ab", "abc", "a"], "abczzz", "abc"),
            (&["user", "us", "u"], "userzzz", "u"),
            (&["worker", "work", "wo"], "workerzzz", "work"),
        ];
        for (prefixes, element, winner) in cases {
            for order in [prefixes.to_vec(), prefixes.iter().rev().copied().collect()] {
                let tree = order
                    .iter()
                    .fold(insert(&empty_index(), &["q"], 0), |tree, prefix| {
                        insert(&tree, &[format!("{prefix}*").as_str()], 1)
                    });
                let deepest = insert(&tree, &[format!("{winner}*").as_str(), "x"], 2);
                assert_eq!(
                    find(&deepest, &[element, "x"]),
                    Some(2),
                    "{prefixes:?} inserted as {order:?}"
                );
            }
        }
    }

    #[test]
    fn compare_astral_and_non_ascii_elements_by_prefix() {
        let tree = insert(&insert(&empty_index(), &["x", "\u{1F600}*"], 2), &["é*"], 1);
        assert_eq!(find(&tree, &["éa"]), Some(1));
        assert_eq!(find(&tree, &["x", "\u{1F600}ab"]), Some(2));
        assert_eq!(find(&tree, &["x", "\u{1F601}"]), None);
    }
}
