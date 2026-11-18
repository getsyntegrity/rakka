// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! `ByteString` and its companions (AC-UTIL-006, AC-UTIL-022): an immutable, cheaply cloned and trimmed
//! sequence of bytes, a builder, a mutable cursor (`ByteIterator`) and `java.io`-style stream adapters.
//!
//! Q-UTIL-12 (c): `ByteString` is a newtype over `bytes::Bytes`, so there is one contiguous fragment and
//! the three internal representations of Pekko (`ByteString1C`, `ByteString1`, `ByteStrings`) are not
//! observable: `++` copies (Pekko links fragments), `compact` returns an exactly sized copy and
//! `is_compact` is always `true`. Everything a program can see of the sequence (lengths, contents,
//! saturated trimming, equality, search, `toString`) is Pekko's, and the recorded cases in
//! `cases/oracle-util-core.jsonl` (`bytes.tsv`) hold it to that.
//!
//! Index arguments are `i32`, as the `Int` of the Scala API, because their saturation (including
//! `Int.MinValue` and `Int.MaxValue`) and the overflow of `takeRight` are part of the contract.

mod base64;
mod builder;
mod charset;
mod error;
mod iterator;
mod streams;
#[cfg(test)]
mod tests;

use std::fmt;

use bytes::{Buf, BufMut, Bytes};

pub use builder::ByteStringBuilder;
pub use charset::Charset;
pub use error::ByteStringError;
pub use iterator::{ByteIterator, ByteIteratorInputStream};
pub use streams::{ByteStringInputStream, UnsynchronizedByteArrayInputStream};

/// How many bytes `Display` prints before it gives up with `... and [n] more`.
const DISPLAY_LIMIT: usize = 100;

/// An immutable sequence of bytes. Equality compares contents; cloning and trimming share storage.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct ByteString {
    bytes: Bytes,
}

impl ByteString {
    /// `ByteString.UTF_8`.
    pub const UTF_8: &'static str = "UTF-8";

    /// `ByteString.empty`.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// `ByteString.fromArray(array)`: copies.
    #[must_use]
    pub fn from_array(array: &[u8]) -> Self {
        Self {
            bytes: Bytes::copy_from_slice(array),
        }
    }

    /// `ByteString.fromArray(array, offset, length)`: copies `length` bytes from `offset`; a negative
    /// `offset` counts as `0` and `length` is cut to what the array has, so this never fails.
    #[must_use]
    pub fn from_array_range(array: &[u8], offset: i32, length: i32) -> Self {
        let from = usize::try_from(offset.max(0)).unwrap_or_default();
        let available = i64::try_from(array.len().saturating_sub(from)).unwrap_or(i64::MAX);
        let count = usize::try_from(available.min(i64::from(length)).max(0)).unwrap_or_default();
        if count == 0 {
            Self::empty()
        } else {
            Self::from_array(&array[from..from + count])
        }
    }

    /// `ByteString.fromArrayUnsafe(array)`: takes the vector without copying. Safe here: the vector is
    /// moved, so nobody can mutate it afterwards.
    #[must_use]
    pub fn from_array_unsafe(array: Vec<u8>) -> Self {
        Self {
            bytes: Bytes::from(array),
        }
    }

    /// `ByteString.fromInts(...)`: each value truncated to its low eight bits.
    #[must_use]
    pub fn from_ints(values: &[i32]) -> Self {
        Self::from_array_unsafe(values.iter().map(|&v| v as u8).collect())
    }

    /// `ByteString.fromString(text)`: UTF-8.
    #[must_use]
    pub fn from_string(text: &str) -> Self {
        Self::from_array(text.as_bytes())
    }

    /// `ByteString.fromString(text, charset)` by charset name.
    pub fn from_string_with_charset(text: &str, charset: &str) -> Result<Self, ByteStringError> {
        Ok(Self::from_string_in(text, Charset::named(charset)?))
    }

    /// `ByteString.fromString(text, charset)`.
    #[must_use]
    pub fn from_string_in(text: &str, charset: Charset) -> Self {
        Self::from_array_unsafe(charset.encode(text))
    }

    /// `ByteString.fromByteBuffer(buffer)`: takes every remaining byte of `buffer` and consumes it.
    pub fn from_buf(buffer: &mut impl Buf) -> Self {
        let count = buffer.remaining();
        Self {
            bytes: buffer.copy_to_bytes(count),
        }
    }

    /// Wraps `Bytes` without copying.
    #[must_use]
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self { bytes }
    }

    /// The underlying `Bytes` (a cheap clone).
    #[must_use]
    pub fn to_bytes(&self) -> Bytes {
        self.bytes.clone()
    }

    /// The bytes as a slice (replaces `asByteBuffer`/`asByteBuffers`: there is a single fragment).
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// `toArray`: a copy.
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    /// `length`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// `isEmpty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn length_as_int(&self) -> i32 {
        i32::try_from(self.bytes.len()).unwrap_or(i32::MAX)
    }

    /// The bytes in order.
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.bytes.iter().copied()
    }

    /// `iterator`: a `ByteIterator` over this value.
    #[must_use]
    pub fn iterator(&self) -> ByteIterator {
        ByteIterator::new(self.bytes.clone())
    }

    /// `asInputStream`.
    #[must_use]
    pub fn as_input_stream(&self) -> ByteStringInputStream {
        ByteStringInputStream::from_bytes(self.bytes.clone())
    }

    /// `ByteString.newBuilder`.
    #[must_use]
    pub fn new_builder() -> ByteStringBuilder {
        ByteStringBuilder::new()
    }

    /// `apply(idx)`.
    pub fn apply(&self, index: i32) -> Result<u8, ByteStringError> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.bytes.get(i).copied())
            .ok_or_else(|| ByteStringError::IndexOutOfBounds(Some(index.to_string())))
    }

    /// `head`.
    pub fn head(&self) -> Result<u8, ByteStringError> {
        self.apply(0)
    }

    /// `last`.
    pub fn last(&self) -> Result<u8, ByteStringError> {
        self.apply(self.length_as_int().wrapping_sub(1))
    }

    /// `tail`: everything after the first byte (empty stays empty).
    #[must_use]
    pub fn tail(&self) -> Self {
        self.drop(1)
    }

    /// `init`: everything before the last byte (empty stays empty).
    #[must_use]
    pub fn init(&self) -> Self {
        self.drop_right(1)
    }

    fn sub(&self, from: usize, until: usize) -> Self {
        Self {
            bytes: self.bytes.slice(from..until),
        }
    }

    /// `take(n)`: the first `n` bytes; any `n` is accepted.
    #[must_use]
    pub fn take(&self, n: i32) -> Self {
        match usize::try_from(n) {
            Ok(0) | Err(_) => Self::empty(),
            Ok(n) if n >= self.len() => self.clone(),
            Ok(n) => self.sub(0, n),
        }
    }

    /// `drop(n)`: everything after the first `n` bytes.
    #[must_use]
    pub fn drop(&self, n: i32) -> Self {
        match usize::try_from(n) {
            Ok(0) | Err(_) => self.clone(),
            Ok(n) if n >= self.len() => Self::empty(),
            Ok(n) => self.sub(n, self.len()),
        }
    }

    /// `dropRight(n)`: everything but the last `n` bytes.
    #[must_use]
    pub fn drop_right(&self, n: i32) -> Self {
        match usize::try_from(n) {
            Ok(0) | Err(_) => self.clone(),
            Ok(n) if n >= self.len() => Self::empty(),
            Ok(n) => self.sub(0, self.len() - n),
        }
    }

    /// `takeRight(n)`: `slice(length - n, length)` with the `Int` arithmetic of Scala, so
    /// `take_right(i32::MIN)` overflows to the whole value.
    #[must_use]
    pub fn take_right(&self, n: i32) -> Self {
        let length = self.length_as_int();
        self.slice(length.wrapping_sub(n), length)
    }

    /// `slice(from, until)`: the bytes at `[from, until)` clamped to the value; never fails.
    #[must_use]
    pub fn slice(&self, from: i32, until: i32) -> Self {
        let length = self.length_as_int();
        if from <= 0 && until >= length {
            self.clone()
        } else if from >= length || until <= 0 || from >= until {
            Self::empty()
        } else {
            self.sub(
                usize::try_from(from.max(0)).unwrap_or_default(),
                usize::try_from(until.min(length)).unwrap_or_default(),
            )
        }
    }

    /// `splitAt(n)`.
    #[must_use]
    pub fn split_at(&self, n: i32) -> (Self, Self) {
        (self.take(n), self.drop(n))
    }

    /// `span(p)`: the longest prefix whose bytes satisfy `predicate`, and the rest.
    #[must_use]
    pub fn span(&self, mut predicate: impl FnMut(u8) -> bool) -> (Self, Self) {
        let prefix = self.iter().take_while(|&b| predicate(b)).count();
        (self.sub(0, prefix), self.sub(prefix, self.len()))
    }

    /// `takeWhile(p)`.
    #[must_use]
    pub fn take_while(&self, predicate: impl FnMut(u8) -> bool) -> Self {
        self.span(predicate).0
    }

    /// `dropWhile(p)`.
    #[must_use]
    pub fn drop_while(&self, predicate: impl FnMut(u8) -> bool) -> Self {
        self.span(predicate).1
    }

    /// `++`: the two values in sequence (a side that is empty yields the other without copying).
    #[must_use]
    pub fn concat(&self, other: &Self) -> Self {
        if self.is_empty() {
            other.clone()
        } else if other.is_empty() {
            self.clone()
        } else {
            let mut joined = Vec::with_capacity(self.len() + other.len());
            joined.extend_from_slice(&self.bytes);
            joined.extend_from_slice(&other.bytes);
            Self::from_array_unsafe(joined)
        }
    }

    /// `grouped(size)`: consecutive slices of `size` bytes, the last one possibly shorter.
    pub fn grouped(&self, size: i32) -> Result<Vec<Self>, ByteStringError> {
        let step = usize::try_from(size)
            .ok()
            .filter(|&s| s > 0)
            .ok_or_else(|| {
                ByteStringError::IllegalArgument(format!("size={size} must be positive"))
            })?;
        Ok((0..self.len())
            .step_by(step)
            .map(|from| self.sub(from, (from + step).min(self.len())))
            .collect())
    }

    /// `mapI(f)`: maps every byte through `f` and truncates the result back to a byte.
    #[must_use]
    pub fn map_i(&self, mut f: impl FnMut(u8) -> i32) -> Self {
        Self::from_array_unsafe(self.iter().map(|b| f(b) as u8).collect())
    }

    /// `compact`: an exactly sized copy that does not retain a larger buffer.
    #[must_use]
    pub fn compact(&self) -> Self {
        Self::from_array(&self.bytes)
    }

    /// `isCompact`: always `true` with a single contiguous fragment (Q-UTIL-12 (c)).
    #[must_use]
    pub fn is_compact(&self) -> bool {
        true
    }

    // ------------------------------------------------------------------ search (AC-UTIL-006.7)

    /// `indexOf(elem)`.
    #[must_use]
    pub fn index_of(&self, byte: u8) -> Option<usize> {
        self.index_of_from(byte, 0)
    }

    /// `indexOf(elem, from)`: the first index `>= from` (a negative `from` counts as `0`).
    #[must_use]
    pub fn index_of_from(&self, byte: u8, from: i32) -> Option<usize> {
        self.index_of_range(byte, from, i32::MAX)
    }

    /// `indexOf(elem, from, to)`: the first index in `[from, to)`.
    #[must_use]
    pub fn index_of_range(&self, byte: u8, from: i32, to: i32) -> Option<usize> {
        let start = usize::try_from(from.max(0)).unwrap_or_default();
        let end = usize::try_from(to).unwrap_or_default().min(self.len());
        if start >= end {
            return None;
        }
        self.bytes[start..end]
            .iter()
            .position(|&b| b == byte)
            .map(|p| p + start)
    }

    /// `lastIndexOf(elem)`.
    #[must_use]
    pub fn last_index_of(&self, byte: u8) -> Option<usize> {
        self.last_index_of_end(byte, self.length_as_int().wrapping_sub(1))
    }

    /// `lastIndexOf(elem, end)`: the last index `<= end`.
    #[must_use]
    pub fn last_index_of_end(&self, byte: u8, end: i32) -> Option<usize> {
        let last = end.min(self.length_as_int().wrapping_sub(1));
        let last = usize::try_from(last).ok()?;
        self.bytes[..=last].iter().rposition(|&b| b == byte)
    }

    /// `contains(elem)`.
    #[must_use]
    pub fn contains(&self, byte: u8) -> bool {
        self.index_of(byte).is_some()
    }

    fn matches_at(&self, start: usize, pattern: &[u8]) -> bool {
        self.bytes
            .get(start..start + pattern.len())
            .is_some_and(|window| window == pattern)
    }

    /// `indexOfSlice(slice, from)`.
    #[must_use]
    pub fn index_of_slice(&self, slice: &[u8], from: i32) -> Option<usize> {
        let length = self.length_as_int();
        if slice.is_empty() {
            return if from > length {
                None
            } else {
                usize::try_from(from.max(0)).ok()
            };
        }
        if slice.len() > self.len() {
            return None;
        }
        if slice.len() == 1 {
            return self.index_of_from(slice[0], from);
        }
        let limit = length
            .wrapping_sub(i32::try_from(slice.len()).unwrap_or(i32::MAX))
            .wrapping_add(1);
        let mut start_from = from.max(0);
        loop {
            let start = self.index_of_range(slice[0], start_from, limit)?;
            if self.matches_at(start, slice) {
                return Some(start);
            }
            start_from = i32::try_from(start + 1).ok()?;
        }
    }

    /// `containsSlice(slice)`.
    #[must_use]
    pub fn contains_slice(&self, slice: &[u8]) -> bool {
        self.index_of_slice(slice, 0).is_some()
    }

    /// `lastIndexOfSlice(slice, end)`: the last start index `<= end`.
    #[must_use]
    pub fn last_index_of_slice(&self, slice: &[u8], end: i32) -> Option<usize> {
        let length = self.length_as_int();
        if slice.is_empty() {
            return if end < 0 {
                None
            } else {
                usize::try_from(end.min(length)).ok()
            };
        }
        if slice.len() > self.len() {
            return None;
        }
        let slice_length = i64::try_from(slice.len()).unwrap_or(i64::MAX);
        let effective_end = i64::from(end).min(i64::from(length) - slice_length);
        let mut current_end = effective_end + slice_length - 1;
        if slice.len() == 1 {
            return self.last_index_of_end(slice[0], i32::try_from(effective_end).ok()?);
        }
        loop {
            let end_position = i64::try_from(
                self.last_index_of_end(slice[slice.len() - 1], i32::try_from(current_end).ok()?)?,
            )
            .ok()?;
            if end_position < slice_length - 1 {
                return None;
            }
            let start = usize::try_from(end_position - slice_length + 1).ok()?;
            if self.matches_at(start, slice) {
                return Some(start);
            }
            current_end = end_position - 1;
        }
    }

    /// `startsWith(bytes)`.
    #[must_use]
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.starts_with_at(prefix, 0)
    }

    /// `startsWith(bytes, offset)`; Pekko rejects a negative offset with an index exception, which an
    /// unsigned offset cannot express.
    #[must_use]
    pub fn starts_with_at(&self, prefix: &[u8], offset: usize) -> bool {
        self.len()
            .checked_sub(offset)
            .is_some_and(|rest| rest >= prefix.len())
            && self.matches_at(offset, prefix)
    }

    /// `endsWith(bytes)`.
    #[must_use]
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        self.len()
            .checked_sub(suffix.len())
            .is_some_and(|start| self.matches_at(start, suffix))
    }

    // ------------------------------------------------------------------ conversions (AC-UTIL-006.6)

    /// `utf8String`: malformed input becomes U+FFFD.
    #[must_use]
    pub fn utf8_string(&self) -> String {
        Charset::Utf8.decode(&self.bytes)
    }

    /// `decodeString(charset)` by charset name.
    pub fn decode_string(&self, charset: &str) -> Result<String, ByteStringError> {
        Ok(self.decode_string_in(Charset::named(charset)?))
    }

    /// `decodeString(charset)`.
    #[must_use]
    pub fn decode_string_in(&self, charset: Charset) -> String {
        charset.decode(&self.bytes)
    }

    /// `decodeString(charset, from, until)` (new in 2.0.0): the range is clamped, so an empty or inverted
    /// range gives `""`.
    #[must_use]
    pub fn decode_string_range(&self, charset: Charset, from: i32, until: i32) -> String {
        let start = usize::try_from(from.max(0)).unwrap_or_default();
        let end = usize::try_from(until).unwrap_or_default().min(self.len());
        if start >= end {
            String::new()
        } else {
            charset.decode(&self.bytes[start..end])
        }
    }

    /// `encodeBase64`: basic alphabet with padding.
    #[must_use]
    pub fn encode_base64(&self) -> Self {
        if self.is_empty() {
            self.clone()
        } else {
            Self::from_array_unsafe(base64::encode(&self.bytes))
        }
    }

    /// `decodeBase64`: the bytes this value spells in basic Base64 (padding optional).
    pub fn decode_base64(&self) -> Result<Self, ByteStringError> {
        if self.is_empty() {
            Ok(self.clone())
        } else {
            base64::decode(&self.bytes).map(Self::from_array_unsafe)
        }
    }

    /// `copyToArray(xs, start, len)`: copies `min(len, length, xs.length - start)` bytes (never fewer
    /// than zero) to `destination[start..]` and returns how many.
    pub fn copy_to_array(&self, destination: &mut [u8], start: usize, len: i32) -> usize {
        let room = destination.len().saturating_sub(start);
        let count = usize::try_from(len.max(0))
            .unwrap_or_default()
            .min(self.len())
            .min(room);
        if count > 0 {
            destination[start..start + count].copy_from_slice(&self.bytes[..count]);
        }
        count
    }

    /// `copyToBuffer(buffer)`: copies `min(buffer.remaining, length)` bytes and returns how many.
    pub fn copy_to_buffer(&self, buffer: &mut impl BufMut) -> usize {
        let count = buffer.remaining_mut().min(self.len());
        buffer.put_slice(&self.bytes[..count]);
        count
    }
}

impl fmt::Display for ByteString {
    /// `ByteString(1, 2, -3)`: signed decimal bytes, cut after 100 with `... and [n] more`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ByteString(")?;
        for (i, byte) in self.bytes.iter().take(DISPLAY_LIMIT).enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", *byte as i8)?;
        }
        f.write_str(")")?;
        if self.len() > DISPLAY_LIMIT {
            write!(f, "... and [{}] more", self.len() - DISPLAY_LIMIT)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_array_unsafe(bytes)
    }
}

impl From<&[u8]> for ByteString {
    fn from(bytes: &[u8]) -> Self {
        Self::from_array(bytes)
    }
}

impl From<&str> for ByteString {
    fn from(text: &str) -> Self {
        Self::from_string(text)
    }
}

impl From<Bytes> for ByteString {
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl FromIterator<u8> for ByteString {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        Self::from_array_unsafe(iter.into_iter().collect())
    }
}
