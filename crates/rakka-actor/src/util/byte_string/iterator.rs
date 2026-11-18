// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! `ByteIterator` (AC-UTIL-006.4, AC-UTIL-016.3): a mutable cursor over bytes with readers for
//! primitives in either byte order. Like Pekko's, a read that runs out of bytes consumes what is left and
//! then fails with `NoSuchElementException("next on empty iterator")`, the message of the empty Scala
//! iterator that `Collections.EmptyImmutableSeq` hands out.

use std::io;

use bytes::{Buf, BufMut, Bytes};

use super::{ByteString, ByteStringError};
use crate::util::swar::ByteOrder;

/// A cursor over a [`ByteString`]; cloning it gives an independent cursor.
#[derive(Clone, Debug, Default)]
pub struct ByteIterator {
    rest: Bytes,
}

macro_rules! get_primitives {
    ($($whole:ident, $range:ident, $get:ident, $ty:ty, $width:expr;)*) => {$(
        /// `get<Primitive>s(array, order)`.
        pub fn $whole(&mut self, destination: &mut [$ty], order: ByteOrder) -> Result<(), ByteStringError> {
            let count = destination.len();
            self.$range(destination, 0, count, order)
        }

        /// `get<Primitive>s(array, start, n, order)`: reads `n` values into `destination[start..]`;
        /// without enough bytes (or room) nothing is read.
        pub fn $range(
            &mut self,
            destination: &mut [$ty],
            start: usize,
            n: usize,
            order: ByteOrder,
        ) -> Result<(), ByteStringError> {
            if start.checked_add(n).is_none_or(|end| end > destination.len()) {
                return Err(ByteStringError::ArrayIndexOutOfBounds(format!(
                    "range [{start}, {start} + {n}) out of bounds for length {}",
                    destination.len()
                )));
            }
            if n.saturating_mul($width) > self.len() {
                return Err(ByteStringError::NoSuchElement("buffer underflow".to_owned()));
            }
            for slot in &mut destination[start..start + n] {
                *slot = self.$get(order)?;
            }
            Ok(())
        }
    )*};
}

impl ByteIterator {
    pub(crate) fn new(rest: Bytes) -> Self {
        Self { rest }
    }

    /// `len`: how many bytes are left.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rest.len()
    }

    /// `isEmpty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    /// `hasNext`.
    #[must_use]
    pub fn has_next(&self) -> bool {
        !self.rest.is_empty()
    }

    /// `head`: the next byte without consuming it, or `None` when exhausted.
    #[must_use]
    pub fn head(&self) -> Option<u8> {
        self.rest.first().copied()
    }

    /// `next()`: consumes one byte.
    pub fn next_byte(&mut self) -> Result<u8, ByteStringError> {
        if self.rest.has_remaining() {
            Ok(self.rest.get_u8())
        } else {
            Err(ByteStringError::exhausted())
        }
    }

    /// The bytes that are left, consumed one at a time as the returned iterator is driven.
    pub fn bytes(&mut self) -> impl Iterator<Item = u8> + '_ {
        std::iter::from_fn(move || self.next_byte().ok())
    }

    /// `size`: counts the bytes left, which consumes the iterator.
    pub fn size(&mut self) -> usize {
        let count = self.rest.len();
        self.rest.clear();
        count
    }

    /// `take(n)`: keeps only the next `n` bytes (`n <= 0` leaves none).
    pub fn take(&mut self, n: i32) -> &mut Self {
        self.rest.truncate(usize::try_from(n).unwrap_or_default());
        self
    }

    /// `drop(n)`: skips `n` bytes (`n <= 0` skips none).
    pub fn drop(&mut self, n: i32) -> &mut Self {
        let skipped = usize::try_from(n).unwrap_or_default().min(self.rest.len());
        self.rest.advance(skipped);
        self
    }

    /// `slice(from, until)`: `drop(from).take(until - from)`.
    pub fn slice(&mut self, from: i32, until: i32) -> &mut Self {
        self.drop(from.max(0));
        self.take(until.saturating_sub(from.max(0)))
    }

    /// `takeWhile(p)`: keeps the longest prefix whose bytes satisfy `predicate`.
    pub fn take_while(&mut self, mut predicate: impl FnMut(u8) -> bool) -> &mut Self {
        let prefix = self.rest.iter().take_while(|&&b| predicate(b)).count();
        self.rest.truncate(prefix);
        self
    }

    /// `dropWhile(p)`: skips the longest prefix whose bytes satisfy `predicate`.
    pub fn drop_while(&mut self, mut predicate: impl FnMut(u8) -> bool) -> &mut Self {
        let prefix = self.rest.iter().take_while(|&&b| predicate(b)).count();
        self.rest.advance(prefix);
        self
    }

    /// `span(p)`: the prefix whose bytes satisfy `predicate`, and the rest.
    #[must_use]
    pub fn span(self, mut predicate: impl FnMut(u8) -> bool) -> (Self, Self) {
        let prefix = self.rest.iter().take_while(|&&b| predicate(b)).count();
        (
            Self::new(self.rest.slice(..prefix)),
            Self::new(self.rest.slice(prefix..)),
        )
    }

    /// `duplicate`: two independent cursors at the same position.
    #[must_use]
    pub fn duplicate(self) -> (Self, Self) {
        (self.clone(), self)
    }

    /// `indexOf(elem, from)`: the position of the first `byte` at or after `from` (a negative `from`
    /// counts as `0`); like Pekko's, it leaves the iterator exhausted.
    pub fn index_of(&mut self, byte: u8, from: i32) -> Option<usize> {
        let start = usize::try_from(from).unwrap_or_default();
        let found = self
            .rest
            .iter()
            .skip(start)
            .position(|&b| b == byte)
            .map(|p| p + start);
        self.rest.clear();
        found
    }

    /// `toByteString`: the bytes left, which consumes the iterator.
    pub fn to_byte_string(&mut self) -> ByteString {
        ByteString::from_bytes(std::mem::take(&mut self.rest))
    }

    /// `getByteString(n)`: up to `n` bytes; asking for more than are left is not an error.
    pub fn get_byte_string(&mut self, n: i32) -> ByteString {
        let count = usize::try_from(n).unwrap_or_default().min(self.rest.len());
        ByteString::from_bytes(self.rest.split_to(count))
    }

    // ------------------------------------------------------------------ primitives

    /// `getByte`.
    pub fn get_byte(&mut self) -> Result<u8, ByteStringError> {
        self.next_byte()
    }

    fn get_ordered<const N: usize>(
        &mut self,
        order: ByteOrder,
    ) -> Result<[u8; N], ByteStringError> {
        if self.rest.len() < N {
            return Err(ByteStringError::exhausted());
        }
        let mut big_endian = [0u8; N];
        for i in 0..N {
            let byte = self.rest.get_u8();
            match order {
                ByteOrder::BigEndian => big_endian[i] = byte,
                ByteOrder::LittleEndian => big_endian[N - 1 - i] = byte,
            }
        }
        Ok(big_endian)
    }

    /// `getShort(order)`. Like `getInt`, `getLong`, `getFloat` and `getDouble`, a read that finds too few
    /// bytes fails without consuming any (Pekko's array-backed iterator does the same; its multi-fragment
    /// one consumes what is left first, a difference of representation, Q-UTIL-12 (c)).
    pub fn get_short(&mut self, order: ByteOrder) -> Result<i16, ByteStringError> {
        self.get_ordered(order).map(i16::from_be_bytes)
    }

    /// `getInt(order)`.
    pub fn get_int(&mut self, order: ByteOrder) -> Result<i32, ByteStringError> {
        self.get_ordered(order).map(i32::from_be_bytes)
    }

    /// `getLong(order)`.
    pub fn get_long(&mut self, order: ByteOrder) -> Result<i64, ByteStringError> {
        self.get_ordered(order).map(i64::from_be_bytes)
    }

    /// `getLongPart(n, order)`: `n` bytes as a `Long`; like Pekko, `n` is not validated (more than eight
    /// bytes shift the first ones out, `n <= 0` reads nothing).
    pub fn get_long_part(&mut self, n: i32, order: ByteOrder) -> Result<i64, ByteStringError> {
        let mut value: u64 = 0;
        for i in 0..n.max(0) {
            let byte = u64::from(self.next_byte()?);
            match order {
                ByteOrder::BigEndian => value = (value << 8) | byte,
                // Pekko shifts an `Int` here: the shift wraps modulo 32 and bit 31 extends the sign.
                ByteOrder::LittleEndian => {
                    value |= i64::from((byte as i32).wrapping_shl(i.wrapping_mul(8) as u32)) as u64;
                }
            }
        }
        Ok(value as i64)
    }

    /// `getFloat(order)`: from the raw bits.
    pub fn get_float(&mut self, order: ByteOrder) -> Result<f32, ByteStringError> {
        self.get_int(order).map(|bits| f32::from_bits(bits as u32))
    }

    /// `getDouble(order)`: from the raw bits.
    pub fn get_double(&mut self, order: ByteOrder) -> Result<f64, ByteStringError> {
        self.get_long(order).map(|bits| f64::from_bits(bits as u64))
    }

    get_primitives! {
        get_shorts, get_shorts_range, get_short, i16, 2;
        get_ints, get_ints_range, get_int, i32, 4;
        get_longs, get_longs_range, get_long, i64, 8;
        get_floats, get_floats_range, get_float, f32, 4;
        get_doubles, get_doubles_range, get_double, f64, 8;
    }

    /// `getBytes(n)`: a new array of the next `n` bytes; with fewer left nothing is consumed.
    pub fn get_bytes(&mut self, n: usize) -> Result<Vec<u8>, ByteStringError> {
        let mut bytes = vec![0; n];
        self.get_bytes_into(&mut bytes, 0, n)?;
        Ok(bytes)
    }

    /// `getBytes(xs, offset, n)`: copies the next `n` bytes into `destination[offset..]`; with fewer left
    /// it fails without copying or consuming.
    pub fn get_bytes_into(
        &mut self,
        destination: &mut [u8],
        offset: usize,
        n: usize,
    ) -> Result<&mut Self, ByteStringError> {
        if n > self.rest.len() {
            return Err(ByteStringError::exhausted());
        }
        let end = offset
            .checked_add(n)
            .filter(|&end| end <= destination.len())
            .ok_or_else(|| {
                ByteStringError::ArrayIndexOutOfBounds(format!(
                    "range [{offset}, {offset} + {n}) out of bounds for length {}",
                    destination.len()
                ))
            })?;
        self.rest.copy_to_slice(&mut destination[offset..end]);
        Ok(self)
    }

    /// `copyToArray(xs, start, len)`: copies `min(len, len(), xs.length - start)` bytes (never fewer than
    /// zero), consumes that many and returns the count.
    pub fn copy_to_array(&mut self, destination: &mut [u8], start: usize, len: i32) -> usize {
        let room = destination.len().saturating_sub(start);
        let count = usize::try_from(len.max(0))
            .unwrap_or_default()
            .min(self.rest.len())
            .min(room);
        if count > 0 {
            self.rest
                .copy_to_slice(&mut destination[start..start + count]);
        }
        count
    }

    /// `copyToBuffer(buffer)`: copies `min(buffer.remaining, len())` bytes, consumes them and returns the count.
    pub fn copy_to_buffer(&mut self, buffer: &mut impl BufMut) -> usize {
        let count = buffer.remaining_mut().min(self.rest.len());
        buffer.put_slice(&self.rest[..count]);
        self.rest.advance(count);
        count
    }

    /// `asInputStream`: reads this iterator through `java.io.InputStream`-style calls.
    #[must_use]
    pub fn into_input_stream(self) -> ByteIteratorInputStream {
        ByteIteratorInputStream { source: self }
    }
}

/// `ByteIterator.asInputStream`.
#[derive(Clone, Debug)]
pub struct ByteIteratorInputStream {
    source: ByteIterator,
}

impl ByteIteratorInputStream {
    /// `read()`: the next byte as `0..=255`, or `-1` at the end.
    pub fn read(&mut self) -> i32 {
        self.source.next_byte().map_or(-1, i32::from)
    }

    /// `read(b, off, len)`: copies up to `len` bytes into `b[off..]`; `0` for an empty request, `-1` at the
    /// end; a range outside `b` fails without a message, as in Pekko.
    pub fn read_range(&mut self, b: &mut [u8], off: i32, len: i32) -> Result<i32, ByteStringError> {
        let (off, len) = usize::try_from(off)
            .ok()
            .zip(usize::try_from(len).ok())
            .filter(|&(off, len)| off.checked_add(len).is_some_and(|end| end <= b.len()))
            .ok_or(ByteStringError::IndexOutOfBounds(None))?;
        if len == 0 {
            return Ok(0);
        }
        if self.source.is_empty() {
            return Ok(-1);
        }
        let count =
            self.source
                .copy_to_array(&mut b[off..], 0, i32::try_from(len).unwrap_or(i32::MAX));
        Ok(i32::try_from(count).unwrap_or(i32::MAX))
    }

    /// `skip(n)`: drops `min(len, n.toInt)` bytes and returns that number (negative when `n` is).
    pub fn skip(&mut self, n: i64) -> i64 {
        let available = i32::try_from(self.source.len()).unwrap_or(i32::MAX);
        let skipped = available.min(n as i32);
        self.source.drop(skipped);
        i64::from(skipped)
    }

    /// `available`: the bytes left.
    #[must_use]
    pub fn available(&self) -> i32 {
        i32::try_from(self.source.len()).unwrap_or(i32::MAX)
    }
}

impl io::Read for ByteIteratorInputStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.source.copy_to_array(buf, 0, i32::MAX))
    }
}
