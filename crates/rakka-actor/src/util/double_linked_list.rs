// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/DoubleLinkedList.scala (Apache-2.0).
//! `DoubleLinkedList` (AC-UTIL-017.5): a mutable, non thread safe double-linked list abstraction with a
//! flexible node type.
//!
//! Pekko's list is told how to read and write the links of its node type by four functions, and the
//! nodes are heap objects pointing at each other. Rust keeps the idea (the list stores no nodes, only
//! its first and last) and gives the access to the links as a [`Links`] value that the caller passes
//! to every call: typically an arena of nodes addressed by index, where one node can sit in several
//! lists through several [`Links`] views over different fields. A node must be linked in at most one
//! list per view, and only nodes that are in the list may be removed or moved.

/// How to read and write the previous/next links of a node of type `N` (`getPrevious`, `getNext`,
/// `setPrevious`, `setNext`).
pub trait Links<N> {
    /// The node before `node`, if any.
    fn previous(&self, node: &N) -> Option<N>;
    /// The node after `node`, if any.
    fn next(&self, node: &N) -> Option<N>;
    /// Sets the node before `node`.
    fn set_previous(&mut self, node: &N, previous: Option<N>);
    /// Sets the node after `node`.
    fn set_next(&mut self, node: &N, next: Option<N>);
}

/// `DoubleLinkedList[Node]`: the first and last node of a list whose links live in a [`Links`].
#[derive(Clone, Debug)]
pub struct DoubleLinkedList<N> {
    first: Option<N>,
    last: Option<N>,
}

impl<N> Default for DoubleLinkedList<N> {
    fn default() -> Self {
        Self {
            first: None,
            last: None,
        }
    }
}

impl<N: Clone + PartialEq> DoubleLinkedList<N> {
    /// An empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }

    /// `getFirst`.
    pub fn first(&self) -> Option<&N> {
        self.first.as_ref()
    }

    /// `getLast`.
    pub fn last(&self) -> Option<&N> {
        self.last.as_ref()
    }

    /// `prepend(node)`: links the node before the first one; returns it.
    pub fn prepend(&mut self, links: &mut impl Links<N>, node: N) -> N {
        let first = self.first.clone();
        self.insert_before(links, node, first)
    }

    /// `append(node)`: links the node after the last one; returns it.
    pub fn append(&mut self, links: &mut impl Links<N>, node: N) -> N {
        let last = self.last.clone();
        self.insert_after(links, last, node)
    }

    /// `remove(node)`: unlinks the node and clears its links.
    pub fn remove(&mut self, links: &mut impl Links<N>, node: &N) {
        self.unlink(links, node);
        links.set_next(node, None);
        links.set_previous(node, None);
    }

    /// `moveToFront(node)`: makes the node the first one; returns it.
    pub fn move_to_front(&mut self, links: &mut impl Links<N>, node: N) -> N {
        if self.first.as_ref() == Some(&node) {
            node
        } else {
            self.unlink(links, &node);
            self.prepend(links, node)
        }
    }

    /// `moveToBack(node)`: makes the node the last one; returns it.
    pub fn move_to_back(&mut self, links: &mut impl Links<N>, node: N) -> N {
        if self.last.as_ref() == Some(&node) {
            node
        } else {
            self.unlink(links, &node);
            self.append(links, node)
        }
    }

    /// `getFirstOrElsePrepend(check, newNode)`: the first node if it passes `check`, else a new node
    /// linked at the front.
    pub fn get_first_or_else_prepend(
        &mut self,
        links: &mut impl Links<N>,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        match self.first.clone() {
            Some(first) if check(&first) => first,
            _ => self.prepend(links, new_node()),
        }
    }

    /// `getLastOrElseAppend(check, newNode)`.
    pub fn get_last_or_else_append(
        &mut self,
        links: &mut impl Links<N>,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        match self.last.clone() {
            Some(last) if check(&last) => last,
            _ => self.append(links, new_node()),
        }
    }

    /// `getNextOrElseInsert(node, check, newNode)`: the node after `node` if it passes `check`, else a
    /// new node linked right after `node`.
    pub fn get_next_or_else_insert(
        &mut self,
        links: &mut impl Links<N>,
        node: &N,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        match links.next(node) {
            Some(next) if check(&next) => next,
            _ => self.insert_after(links, Some(node.clone()), new_node()),
        }
    }

    /// `getPreviousOrElseInsert(node, check, newNode)`.
    pub fn get_previous_or_else_insert(
        &mut self,
        links: &mut impl Links<N>,
        node: &N,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        match links.previous(node) {
            Some(previous) if check(&previous) => previous,
            _ => self.insert_before(links, new_node(), Some(node.clone())),
        }
    }

    /// `findNextOrElseInsert(node, isBefore, check, newNode)`: moves forward from `node` while the next
    /// node satisfies `is_before`, then behaves as `get_next_or_else_insert` from there.
    pub fn find_next_or_else_insert(
        &mut self,
        links: &mut impl Links<N>,
        node: &N,
        is_before: impl Fn(&N) -> bool,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        let at = Self::shift_while(node, |n| links.next(n), is_before);
        self.get_next_or_else_insert(links, &at, check, new_node)
    }

    /// `findPreviousOrElseInsert(node, isAfter, check, newNode)`.
    pub fn find_previous_or_else_insert(
        &mut self,
        links: &mut impl Links<N>,
        node: &N,
        is_after: impl Fn(&N) -> bool,
        check: impl Fn(&N) -> bool,
        new_node: impl FnOnce() -> N,
    ) -> N {
        let at = Self::shift_while(node, |n| links.previous(n), is_after);
        self.get_previous_or_else_insert(links, &at, check, new_node)
    }

    /// `forwardIterator`: the nodes from first to last. Each node is read before it is returned, so the
    /// one just returned may be removed while iterating (the iterator borrows `links`, so collect first
    /// to remove).
    pub fn forward_iterator<'a, L: Links<N>>(&self, links: &'a L) -> impl Iterator<Item = N> + 'a
    where
        N: 'a,
    {
        Self::iterator_from(self.first.clone(), move |n| links.next(n))
    }

    /// `backwardIterator`: the nodes from last to first.
    pub fn backward_iterator<'a, L: Links<N>>(&self, links: &'a L) -> impl Iterator<Item = N> + 'a
    where
        N: 'a,
    {
        Self::iterator_from(self.last.clone(), move |n| links.previous(n))
    }

    fn iterator_from<'a>(
        start: Option<N>,
        shift: impl Fn(&N) -> Option<N> + 'a,
    ) -> impl Iterator<Item = N> + 'a
    where
        N: 'a,
    {
        let mut cursor = start;
        std::iter::from_fn(move || {
            let node = cursor.take()?;
            cursor = shift(&node);
            Some(node)
        })
    }

    fn insert_before(&mut self, links: &mut impl Links<N>, node: N, next: Option<N>) -> N {
        let previous = next.as_ref().and_then(|n| links.previous(n));
        self.link(links, previous, &node, next);
        node
    }

    fn insert_after(&mut self, links: &mut impl Links<N>, previous: Option<N>, node: N) -> N {
        let next = previous.as_ref().and_then(|p| links.next(p));
        self.link(links, previous, &node, next);
        node
    }

    fn link(&mut self, links: &mut impl Links<N>, previous: Option<N>, node: &N, next: Option<N>) {
        links.set_previous(node, previous.clone());
        links.set_next(node, next.clone());
        match previous {
            None => self.first = Some(node.clone()),
            Some(previous) => links.set_next(&previous, Some(node.clone())),
        }
        match next {
            None => self.last = Some(node.clone()),
            Some(next) => links.set_previous(&next, Some(node.clone())),
        }
    }

    fn unlink(&mut self, links: &mut impl Links<N>, node: &N) {
        let previous = links.previous(node);
        let next = links.next(node);
        match &previous {
            None => self.first = next.clone(),
            Some(previous) => links.set_next(previous, next.clone()),
        }
        match &next {
            None => self.last = previous,
            Some(next) => links.set_previous(next, previous),
        }
    }

    fn shift_while(start: &N, shift: impl Fn(&N) -> Option<N>, check: impl Fn(&N) -> bool) -> N {
        let mut current = start.clone();
        while let Some(peek) = shift(&current) {
            if !check(&peek) {
                break;
            }
            current = peek;
        }
        current
    }
}

/// The links of nodes addressed by index: `(previous, next)` per node. The nodes of a list that has
/// its own data (the lists of this module's siblings) keep that data elsewhere, so that a check on
/// the data may run while the links change.
#[derive(Clone, Debug, Default)]
pub struct LinkTable {
    links: Vec<(Option<usize>, Option<usize>)>,
}

impl LinkTable {
    /// An empty link table.
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(&mut self, node: usize) -> &mut (Option<usize>, Option<usize>) {
        if node >= self.links.len() {
            self.links.resize(node + 1, (None, None));
        }
        &mut self.links[node]
    }
}

impl Links<usize> for LinkTable {
    fn previous(&self, node: &usize) -> Option<usize> {
        self.links.get(*node).and_then(|slot| slot.0)
    }

    fn next(&self, node: &usize) -> Option<usize> {
        self.links.get(*node).and_then(|slot| slot.1)
    }

    fn set_previous(&mut self, node: &usize, previous: Option<usize>) {
        self.slot(*node).0 = previous;
    }

    fn set_next(&mut self, node: &usize, next: Option<usize>) {
        self.slot(*node).1 = next;
    }
}

/// Values addressed by a stable index, reusing the indexes of removed values.
#[derive(Clone, Debug)]
pub struct Slab<T> {
    entries: Vec<Option<T>>,
    free: Vec<usize>,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<T> Slab<T> {
    /// Stores `value` in a free slot (or a new one) and returns its index.
    pub fn insert(&mut self, value: T) -> usize {
        match self.free.pop() {
            Some(index) => {
                self.entries[index] = Some(value);
                index
            }
            None => {
                self.entries.push(Some(value));
                self.entries.len() - 1
            }
        }
    }

    /// Removes the value at `index`, which must be there.
    pub fn remove(&mut self, index: usize) -> T {
        self.free.push(index);
        self.entries[index].take().expect("the node is in the slab")
    }

    /// The value at `index`, which must be occupied (panics otherwise).
    pub fn get(&self, index: usize) -> &T {
        self.entries[index]
            .as_ref()
            .expect("the node is in the slab")
    }

    /// Mutable access to the value at `index`, which must be occupied (panics otherwise).
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.entries[index]
            .as_mut()
            .expect("the node is in the slab")
    }
}

#[cfg(test)]
mod tests {
    //! `DoubleLinkedListSpec` of Pekko 2.0.0 (actor-tests/.../util/DoubleLinkedListSpec.scala). As in the
    //! spec, the nodes are created first (`a`, `b`, `c`...), here as indexes into a table of names, and
    //! the links live in a [`LinkTable`].
    use super::*;

    const NAMES: [&str; 8] = ["a", "b", "c", "d", "a1", "a2", "b1", "b2"];
    const NAMES2: [&str; 6] = ["c2", "c1", "a2", "a1", "b1", "d1"];
    const NAMES3: [&str; 6] = ["b1", "b2", "d1", "d2", "c1", "a1"];

    fn check(list: &DoubleLinkedList<usize>, links: &LinkTable, names: &[&str], expected: &[&str]) {
        let name = |n: usize| names[n];
        assert_eq!(list.is_empty(), expected.is_empty());
        assert_eq!(list.first().map(|n| name(*n)), expected.first().copied());
        assert_eq!(list.last().map(|n| name(*n)), expected.last().copied());
        let forward: Vec<_> = list.forward_iterator(links).map(name).collect();
        assert_eq!(forward, expected);
        let mut backward: Vec<_> = list.backward_iterator(links).map(name).collect();
        backward.reverse();
        assert_eq!(backward, expected);
    }

    fn index_of(names: &[&str], name: &str) -> usize {
        names.iter().position(|n| *n == name).expect("a known name")
    }

    fn starts_with<'a>(names: &'a [&'a str], prefix: &'a str) -> impl Fn(&usize) -> bool + 'a {
        move |n| names[*n].starts_with(prefix)
    }

    fn filled(
        names: &[&str],
        order: &[&str],
        at_front: bool,
    ) -> (DoubleLinkedList<usize>, LinkTable) {
        let (mut list, mut links) = (DoubleLinkedList::new(), LinkTable::new());
        for name in order {
            let node = index_of(names, name);
            if at_front {
                list.prepend(&mut links, node);
            } else {
                list.append(&mut links, node);
            }
        }
        (list, links)
    }

    #[test]
    fn start_empty() {
        check(&DoubleLinkedList::new(), &LinkTable::new(), &NAMES, &[]);
    }

    #[test]
    fn prepend_elements_to_front() {
        let (list, links) = filled(&NAMES, &["c", "b", "a"], true);
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn append_elements_to_back() {
        let (list, links) = filled(&NAMES, &["a", "b", "c"], false);
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn remove_elements() {
        let (mut list, mut links) = filled(&NAMES, &["a", "b", "c"], false);
        let (a, b, c) = (0, 1, 2);
        check(&list, &links, &NAMES, &["a", "b", "c"]);
        list.remove(&mut links, &b);
        check(&list, &links, &NAMES, &["a", "c"]);
        list.remove(&mut links, &c);
        check(&list, &links, &NAMES, &["a"]);
        list.remove(&mut links, &a);
        check(&list, &links, &NAMES, &[]);
        assert_eq!((links.previous(&b), links.next(&b)), (None, None));
    }

    #[test]
    fn move_elements_to_front() {
        let (mut list, mut links) = filled(&NAMES, &["a", "b", "c"], true);
        let (a, b, c) = (0, 1, 2);
        check(&list, &links, &NAMES, &["c", "b", "a"]);
        assert_eq!(list.move_to_front(&mut links, c), c);
        check(&list, &links, &NAMES, &["c", "b", "a"]);
        list.move_to_front(&mut links, b);
        check(&list, &links, &NAMES, &["b", "c", "a"]);
        list.move_to_front(&mut links, a);
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn move_elements_to_back() {
        let (mut list, mut links) = filled(&NAMES, &["c", "b", "a"], false);
        let (a, b, c) = (0, 1, 2);
        check(&list, &links, &NAMES, &["c", "b", "a"]);
        assert_eq!(list.move_to_back(&mut links, a), a);
        check(&list, &links, &NAMES, &["c", "b", "a"]);
        list.move_to_back(&mut links, b);
        check(&list, &links, &NAMES, &["c", "a", "b"]);
        list.move_to_back(&mut links, c);
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn get_first_element_or_else_prepend_new_element() {
        let (mut list, mut links) = filled(&NAMES, &["c", "b"], true);
        let (a, b) = (0, 1);
        check(&list, &links, &NAMES, &["b", "c"]);
        assert_eq!(
            list.get_first_or_else_prepend(&mut links, starts_with(&NAMES, "b"), || a),
            b
        );
        check(&list, &links, &NAMES, &["b", "c"]);
        assert_eq!(
            list.get_first_or_else_prepend(&mut links, starts_with(&NAMES, "a"), || a),
            a
        );
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn get_last_element_or_else_append_new_element() {
        let (mut list, mut links) = filled(&NAMES, &["a", "b"], false);
        let (b, c) = (1, 2);
        assert_eq!(
            list.get_last_or_else_append(&mut links, starts_with(&NAMES, "b"), || c),
            b
        );
        check(&list, &links, &NAMES, &["a", "b"]);
        assert_eq!(
            list.get_last_or_else_append(&mut links, starts_with(&NAMES, "c"), || c),
            c
        );
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn get_next_element_or_else_insert_new_element() {
        let (mut list, mut links) = filled(&NAMES, &["c", "a"], true);
        let (a, b, c) = (0, 1, 2);
        check(&list, &links, &NAMES, &["a", "c"]);
        assert_eq!(
            list.get_next_or_else_insert(&mut links, &a, starts_with(&NAMES, "c"), || b),
            c
        );
        check(&list, &links, &NAMES, &["a", "c"]);
        assert_eq!(
            list.get_next_or_else_insert(&mut links, &a, starts_with(&NAMES, "b"), || b),
            b
        );
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn get_previous_element_or_else_insert_new_element() {
        let (mut list, mut links) = filled(&NAMES, &["a", "c"], false);
        let (a, b, c) = (0, 1, 2);
        assert_eq!(
            list.get_previous_or_else_insert(&mut links, &c, starts_with(&NAMES, "a"), || b),
            a
        );
        check(&list, &links, &NAMES, &["a", "c"]);
        assert_eq!(
            list.get_previous_or_else_insert(&mut links, &c, starts_with(&NAMES, "b"), || b),
            b
        );
        check(&list, &links, &NAMES, &["a", "b", "c"]);
    }

    #[test]
    fn find_next_element_or_else_insert_new_element() {
        let names = NAMES2;
        let (mut list, mut links) = filled(&names, &["c2", "c1", "a2", "a1"], true);
        let (c1, b1, d1) = (1, 4, 5);
        let a1 = 3;
        check(&list, &links, &names, &["a1", "a2", "c1", "c2"]);
        let found = list.find_next_or_else_insert(
            &mut links,
            &a1,
            starts_with(&names, "a"),
            starts_with(&names, "c"),
            || b1,
        );
        assert_eq!(found, c1);
        check(&list, &links, &names, &["a1", "a2", "c1", "c2"]);
        let found = list.find_next_or_else_insert(
            &mut links,
            &a1,
            starts_with(&names, "a"),
            starts_with(&names, "b"),
            || b1,
        );
        assert_eq!(found, b1);
        check(&list, &links, &names, &["a1", "a2", "b1", "c1", "c2"]);
        let found = list.find_next_or_else_insert(
            &mut links,
            &c1,
            starts_with(&names, "c"),
            starts_with(&names, "d"),
            || d1,
        );
        assert_eq!(found, d1);
        check(&list, &links, &names, &["a1", "a2", "b1", "c1", "c2", "d1"]);
    }

    #[test]
    fn find_previous_element_or_else_insert_new_element() {
        let names = NAMES3;
        let (mut list, mut links) = filled(&names, &["b1", "b2", "d1", "d2"], false);
        let (b2, d2, c1, a1) = (1, 3, 4, 5);
        check(&list, &links, &names, &["b1", "b2", "d1", "d2"]);
        let found = list.find_previous_or_else_insert(
            &mut links,
            &d2,
            starts_with(&names, "d"),
            starts_with(&names, "b"),
            || c1,
        );
        assert_eq!(found, b2);
        check(&list, &links, &names, &["b1", "b2", "d1", "d2"]);
        let found = list.find_previous_or_else_insert(
            &mut links,
            &d2,
            starts_with(&names, "d"),
            starts_with(&names, "c"),
            || c1,
        );
        assert_eq!(found, c1);
        check(&list, &links, &names, &["b1", "b2", "c1", "d1", "d2"]);
        let found = list.find_previous_or_else_insert(
            &mut links,
            &b2,
            starts_with(&names, "b"),
            starts_with(&names, "a"),
            || a1,
        );
        assert_eq!(found, a1);
        check(&list, &links, &names, &["a1", "b1", "b2", "c1", "d1", "d2"]);
    }

    #[test]
    fn slab_reuses_indexes() {
        let mut slab = Slab::default();
        let (x, y) = (slab.insert("x"), slab.insert("y"));
        assert_eq!((*slab.get(x), *slab.get(y)), ("x", "y"));
        assert_eq!(slab.remove(x), "x");
        assert_eq!(slab.insert("z"), x);
        *slab.get_mut(y) = "w";
        assert_eq!((*slab.get(x), *slab.get(y)), ("z", "w"));
    }
}
