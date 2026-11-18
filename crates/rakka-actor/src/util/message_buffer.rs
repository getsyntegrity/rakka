// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/MessageBuffer.scala (Apache-2.0).
//! `MessageBuffer` and `MessageBufferMap` (AC-UTIL-010): a FIFO of `(message, sender)` pairs for an
//! actor to hold messages in, and a map from an id to such a buffer. Neither is thread safe (by
//! contract in Pekko too).
//!
//! The sender is generic (`R`) until `ActorRef` is wired to the buffer; Pekko's `(null, null)` for the
//! head of an empty buffer is `None`.
//!
//! Difference authorised by the Owner (Q-UTIL-15 (b)): `MessageBufferMap::drop` of an id that was
//! never added returns 0 and sends nothing; Pekko 2.0.0 throws `NullPointerException` there (measured
//! with the 2.0.0 jar, MB-DROP-ABSENT; no spec covers it).

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// What `MessageBufferMap::drop` publishes for every buffered message: `Dropped(message, reason,
/// sender, noSender)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dropped<M, R> {
    pub message: M,
    pub reason: String,
    pub sender: R,
}

/// A non thread safe mutable message buffer that can be used to buffer messages inside actors.
#[derive(Clone, Debug)]
pub struct MessageBuffer<M, R> {
    items: VecDeque<(M, R)>,
}

impl<M, R> Default for MessageBuffer<M, R> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<M, R> MessageBuffer<M, R> {
    /// `MessageBuffer.empty`.
    pub fn empty() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `nonEmpty`.
    pub fn non_empty(&self) -> bool {
        !self.is_empty()
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// `append(message, ref)`: adds one element at the end; returns the buffer.
    pub fn append(&mut self, message: M, sender: R) -> &mut Self {
        self.items.push_back((message, sender));
        self
    }

    /// `dropHead()`: removes the first element (nothing if the buffer is empty).
    pub fn drop_head(&mut self) {
        self.items.pop_front();
    }

    /// `head()`: the first element, `None` where Pekko returns a `(null, null)` pair.
    pub fn head(&self) -> Option<(&M, &R)> {
        self.items
            .front()
            .map(|(message, sender)| (message, sender))
    }

    /// `foreach(f)`: applies `f` to every element, in order.
    pub fn foreach(&self, mut f: impl FnMut(&M, &R)) {
        for (message, sender) in &self.items {
            f(message, sender);
        }
    }

    /// `filterNot(p)`: removes the elements for which `p` holds, keeping the order of the others.
    pub fn filter_not(&mut self, mut p: impl FnMut(&M, &R) -> bool) {
        self.items.retain(|(message, sender)| !p(message, sender));
    }

    /// The elements in order, consuming the buffer.
    fn into_items(self) -> impl Iterator<Item = (M, R)> {
        self.items.into_iter()
    }
}

/// A non thread safe mutable message buffer map: one [`MessageBuffer`] per id.
#[derive(Clone, Debug)]
pub struct MessageBufferMap<I, M, R> {
    buffers: HashMap<I, MessageBuffer<M, R>>,
    /// What `getOrEmpty` returns for an id that is not in the map.
    no_buffer: MessageBuffer<M, R>,
}

impl<I, M, R> Default for MessageBufferMap<I, M, R> {
    fn default() -> Self {
        Self {
            buffers: HashMap::new(),
            no_buffer: MessageBuffer::empty(),
        }
    }
}

impl<I: Eq + Hash, M, R> MessageBufferMap<I, M, R> {
    pub fn new() -> Self {
        Self::default()
    }

    /// `isEmpty`: whether the map has no ids.
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// `nonEmpty`.
    pub fn non_empty(&self) -> bool {
        !self.is_empty()
    }

    /// `size`: how many ids are in the map.
    pub fn size(&self) -> usize {
        self.buffers.len()
    }

    /// `totalSize`: how many elements are in all the buffers.
    pub fn total_size(&self) -> usize {
        self.buffers.values().map(MessageBuffer::size).sum()
    }

    /// `add(id)`: adds the id with an empty buffer if it is not there.
    pub fn add(&mut self, id: I) {
        self.buffers.entry(id).or_default();
    }

    /// `append(id, message, ref)`: appends to the buffer of the id, adding it if needed.
    pub fn append(&mut self, id: I, message: M, sender: R) {
        self.buffers.entry(id).or_default().append(message, sender);
    }

    /// `remove(id)`: removes the buffer of the id.
    pub fn remove(&mut self, id: &I) {
        self.buffers.remove(id);
    }

    /// `drop(id, reason, deadLetters)`: removes the buffer of the id, handing a [`Dropped`] for each
    /// buffered message, in order, to `dead_letters`. Returns how many messages were dropped; 0 for an
    /// id that is not in the map (Q-UTIL-15 (b), see the module documentation).
    pub fn drop(
        &mut self,
        id: &I,
        reason: &str,
        mut dead_letters: impl FnMut(Dropped<M, R>),
    ) -> usize {
        let Some(buffer) = self.buffers.remove(id) else {
            return 0;
        };
        let count = buffer.size();
        for (message, sender) in buffer.into_items() {
            dead_letters(Dropped {
                message,
                reason: reason.to_owned(),
                sender,
            });
        }
        count
    }

    /// `contains(id)`.
    pub fn contains(&self, id: &I) -> bool {
        self.buffers.contains_key(id)
    }

    /// `getOrEmpty(id)`: the buffer of the id, or an empty buffer if the id is not in the map.
    pub fn get_or_empty(&self, id: &I) -> &MessageBuffer<M, R> {
        self.buffers.get(id).unwrap_or(&self.no_buffer)
    }

    /// The buffer of the id to change it (`getOrEmpty(id).dropHead()` in Pekko), if the id is in the map.
    pub fn get_mut(&mut self, id: &I) -> Option<&mut MessageBuffer<M, R>> {
        self.buffers.get_mut(id)
    }

    /// `foreach(f)`: applies `f` to every id and its buffer, in no particular order.
    pub fn foreach(&self, mut f: impl FnMut(&I, &MessageBuffer<M, R>)) {
        for (id, buffer) in &self.buffers {
            f(id, buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    //! `MessageBufferSpec` of Pekko 2.0.0, plus `drop` (which no spec covers).
    use super::*;

    type Buffer = MessageBuffer<&'static str, &'static str>;

    fn order(buffer: &Buffer) -> String {
        let mut out = String::new();
        buffer.foreach(|m, s| out.push_str(&format!("{m}->{s}:")));
        out
    }

    #[test]
    fn answer_empty_correctly() {
        let mut buffer = Buffer::empty();
        assert!(buffer.is_empty() && !buffer.non_empty());
        buffer.append("m1", "s1");
        assert!(!buffer.is_empty() && buffer.non_empty());
    }

    #[test]
    fn append_and_drop() {
        let mut buffer = Buffer::empty();
        assert_eq!(buffer.size(), 0);
        buffer.append("m1", "s1");
        assert_eq!(buffer.size(), 1);
        buffer.append("m2", "s2");
        assert_eq!(buffer.size(), 2);
        assert_eq!(buffer.head(), Some((&"m1", &"s1")));
        assert_eq!(buffer.size(), 2);
        buffer.drop_head();
        assert_eq!(buffer.size(), 1);
        assert_eq!(buffer.head(), Some((&"m2", &"s2")));
        buffer.drop_head();
        assert_eq!(buffer.size(), 0);
        buffer.drop_head();
        assert_eq!(buffer.size(), 0);
        assert_eq!(buffer.head(), None);
    }

    #[test]
    fn process_elements_in_the_right_order() {
        let mut buffer = Buffer::empty();
        buffer
            .append("m1", "s1")
            .append("m2", "s2")
            .append("m3", "s3");
        assert_eq!(order(&buffer), "m1->s1:m2->s2:m3->s3:");
        buffer.drop_head();
        assert_eq!(order(&buffer), "m2->s2:m3->s3:");
    }

    #[test]
    fn filter_not_of_first_and_of_last() {
        let mut buffer = Buffer::empty();
        buffer.append("m1", "s1").append("m2", "s2");
        buffer.filter_not(|m, _| *m == "m1");
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m2"), 1)
        );
        buffer.filter_not(|m, _| *m == "m2");
        assert!(buffer.is_empty());

        buffer.append("m1", "s1").append("m2", "s2");
        buffer.filter_not(|m, _| *m == "m2");
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m1"), 1)
        );
        buffer.filter_not(|m, _| *m == "m1");
        assert!(buffer.is_empty());
    }

    #[test]
    fn filter_not_of_single_entry() {
        let mut buffer = Buffer::empty();
        for (m, s) in [
            ("m1", "s1"),
            ("m2", "s2"),
            ("m3", "s3"),
            ("m4", "s4"),
            ("m5", "s5"),
            ("m6", "s6"),
        ] {
            buffer.append(m, s);
        }
        assert_eq!(buffer.head().map(|h| *h.0), Some("m1"));
        buffer.filter_not(|m, _| *m == "m1");
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m2"), 5)
        );
        buffer.filter_not(|m, _| *m == "m3");
        assert_eq!(buffer.size(), 4);
        buffer.filter_not(|m, _| *m == "m4");
        assert_eq!(buffer.size(), 3);
        buffer.drop_head();
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m5"), 2)
        );
        buffer.filter_not(|m, _| *m == "m6");
        assert_eq!(buffer.size(), 1);
        buffer.append("m7", "s7");
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m5"), 2)
        );
        buffer.filter_not(|m, _| *m == "m5");
        assert_eq!(buffer.size(), 1);
        buffer.filter_not(|m, _| *m == "m7");
        assert_eq!(buffer.size(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn filter_not_of_several_entries() {
        let mut buffer = Buffer::empty();
        for (m, s) in [
            ("m1", "s1"),
            ("m2", "s2"),
            ("m3", "s3"),
            ("m4", "s4"),
            ("m5", "s5"),
            ("m6", "s6"),
        ] {
            buffer.append(m, s);
        }
        buffer.filter_not(|m, _| *m == "m1" || *m == "m2");
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m3"), 4)
        );
        buffer.filter_not(|m, _| *m == "m4" || *m == "m5");
        assert_eq!(buffer.size(), 2);
        buffer.drop_head();
        assert_eq!(
            (buffer.head().map(|h| *h.0), buffer.size()),
            (Some("m6"), 1)
        );
    }

    type Map = MessageBufferMap<&'static str, &'static str, &'static str>;

    #[test]
    fn support_contains_add_append_and_remove() {
        let mut map = Map::new();
        assert!(!map.contains(&"id1"));
        assert!(map.get_or_empty(&"id1").is_empty());
        assert_eq!(map.total_size(), 0);
        map.add("id1");
        assert!(map.contains(&"id1"));
        assert!(map.get_or_empty(&"id1").is_empty());
        assert_eq!(map.total_size(), 0);
        map.append("id1", "m1", "s1");
        assert!(map.contains(&"id1"));
        assert!(!map.get_or_empty(&"id1").is_empty());
        assert_eq!(map.total_size(), 1);
        map.remove(&"id1");
        assert!(!map.contains(&"id1"));
        assert!(map.get_or_empty(&"id1").is_empty());
        assert_eq!(map.total_size(), 0);
        assert_eq!(
            (map.size(), map.is_empty(), map.non_empty()),
            (0, true, false)
        );
    }

    #[test]
    fn handle_multiple_message_buffers() {
        let mut map = Map::new();
        map.append("id1", "m11", "s11");
        map.append("id1", "m12", "s12");
        map.append("id2", "m21", "s21");
        map.append("id2", "m22", "s22");
        assert_eq!(map.total_size(), 4);
        let mut seen = String::new();
        map.get_or_empty(&"id1")
            .foreach(|m, s| seen.push_str(&format!("id1->{m}->{s}:")));
        map.get_or_empty(&"id2")
            .foreach(|m, s| seen.push_str(&format!("id2->{m}->{s}:")));
        assert_eq!(
            seen,
            "id1->m11->s11:id1->m12->s12:id2->m21->s21:id2->m22->s22:"
        );
        let mut ids = Vec::new();
        map.foreach(|id, buffer| ids.push((*id, buffer.size())));
        ids.sort_unstable();
        assert_eq!(ids, vec![("id1", 2), ("id2", 2)]);
        map.get_mut(&"id1").expect("id1 is there").drop_head();
        assert_eq!(map.total_size(), 3);
        assert!(map.get_mut(&"nope").is_none());
    }

    #[test]
    fn drop_publishes_dropped_in_order_and_ignores_an_absent_id() {
        let mut map = Map::new();
        map.append("id1", "m1", "s1");
        map.append("id1", "m2", "s2");
        let mut dead_letters = Vec::new();
        assert_eq!(map.drop(&"id1", "stopped", |d| dead_letters.push(d)), 2);
        assert_eq!(
            dead_letters,
            vec![
                Dropped {
                    message: "m1",
                    reason: "stopped".into(),
                    sender: "s1"
                },
                Dropped {
                    message: "m2",
                    reason: "stopped".into(),
                    sender: "s2"
                },
            ]
        );
        assert!(!map.contains(&"id1"));
        // Q-UTIL-15 (b): an id that was never added drops nothing (Pekko throws NullPointerException)
        assert_eq!(
            map.drop(&"absent", "why", |_| panic!("nothing to publish")),
            0
        );
        map.add("empty");
        assert_eq!(
            map.drop(&"empty", "why", |_| panic!("nothing to publish")),
            0
        );
        assert!(!map.contains(&"empty"));
    }
}
