// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! `ByteStringBuilder` (AC-UTIL-006.3): a mutable, unsynchronised accumulator of bytes. Primitives are
//! written in the byte order asked for; `result()` copies what has been written so far and does not
//! invalidate the builder.

use std::io;

use super::{ByteString, ByteStringError};
use crate::util::swar::ByteOrder;

/// Builds a [`ByteString`] by appending bytes, primitives, arrays and other byte strings.
#[derive(Clone, Debug, Default)]
pub struct ByteStringBuilder {
    buffer: Vec<u8>,
}

macro_rules! put_primitives {
    ($($whole:ident, $range:ident, $put:ident, $ty:ty;)*) => {$(
        /// `put<Primitive>s(array, order)`.
        pub fn $whole(&mut self, values: &[$ty], order: ByteOrder) {
            for &value in values {
                self.$put(value, order);
            }
        }

        /// `put<Primitive>s(array, start, len, order)`: reading outside `values` fails like the JVM array access.
        pub fn $range(
            &mut self,
            values: &[$ty],
            start: i32,
            len: i32,
            order: ByteOrder,
        ) -> Result<(), ByteStringError> {
            for index in i64::from(start)..i64::from(start) + i64::from(len) {
                let value = usize::try_from(index)
                    .ok()
                    .and_then(|i| values.get(i).copied())
                    .ok_or_else(|| {
                        ByteStringError::ArrayIndexOutOfBounds(format!(
                            "Index {index} out of bounds for length {}",
                            values.len()
                        ))
                    })?;
                self.$put(value, order);
            }
            Ok(())
        }
    )*};
}

impl ByteStringBuilder {
    /// `new ByteStringBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `putByte(x)` / `+=` / `addOne`.
    pub fn put_byte(&mut self, value: u8) {
        self.buffer.push(value);
    }

    /// `addOne(elem)`.
    pub fn add_one(&mut self, value: u8) {
        self.put_byte(value);
    }

    fn put_ordered<const N: usize>(&mut self, big_endian: [u8; N], order: ByteOrder) {
        match order {
            ByteOrder::BigEndian => self.buffer.extend_from_slice(&big_endian),
            ByteOrder::LittleEndian => self.buffer.extend(big_endian.iter().rev()),
        }
    }

    /// `putShort(x, order)`: the low 16 bits of `value`.
    pub fn put_short(&mut self, value: i32, order: ByteOrder) {
        self.put_ordered((value as i16).to_be_bytes(), order);
    }

    /// `putInt(x, order)`.
    pub fn put_int(&mut self, value: i32, order: ByteOrder) {
        self.put_ordered(value.to_be_bytes(), order);
    }

    /// `putLong(x, order)`.
    pub fn put_long(&mut self, value: i64, order: ByteOrder) {
        self.put_ordered(value.to_be_bytes(), order);
    }

    /// `putLongPart(x, n, order)`: the `n` low bytes of `value`, most significant first for big endian.
    /// Pekko does not validate `n`: shifts wrap modulo 64 like the JVM's, and `n <= 0` writes nothing.
    pub fn put_long_part(&mut self, value: i64, n: i32, order: ByteOrder) {
        let bits = value as u64;
        for i in 0..n.max(0) {
            let shift = match order {
                ByteOrder::BigEndian => n.wrapping_sub(1).wrapping_sub(i).wrapping_mul(8),
                ByteOrder::LittleEndian => i.wrapping_mul(8),
            };
            self.buffer.push(bits.wrapping_shr(shift as u32) as u8);
        }
    }

    /// `putFloat(x, order)`: the raw bits.
    pub fn put_float(&mut self, value: f32, order: ByteOrder) {
        self.put_int(value.to_bits() as i32, order);
    }

    /// `putDouble(x, order)`: the raw bits.
    pub fn put_double(&mut self, value: f64, order: ByteOrder) {
        self.put_long(value.to_bits() as i64, order);
    }

    /// `putBytes(array)`.
    pub fn put_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// `putBytes(array, start, len)`: a range outside `bytes` fails (and writes nothing).
    pub fn put_bytes_range(
        &mut self,
        bytes: &[u8],
        start: i32,
        len: i32,
    ) -> Result<(), ByteStringError> {
        let range = usize::try_from(start).ok().and_then(|from| {
            let count = usize::try_from(len).ok()?;
            bytes.get(from..from.checked_add(count)?)
        });
        let slice = range.ok_or_else(|| {
            ByteStringError::ArrayIndexOutOfBounds(format!(
                "arraycopy: range [{start}, {start} + {len}) out of bounds for byte[{}]",
                bytes.len()
            ))
        })?;
        self.buffer.extend_from_slice(slice);
        Ok(())
    }

    put_primitives! {
        put_shorts, put_shorts_range, put_short_value, i16;
        put_ints, put_ints_range, put_int, i32;
        put_longs, put_longs_range, put_long, i64;
        put_floats, put_floats_range, put_float, f32;
        put_doubles, put_doubles_range, put_double, f64;
    }

    fn put_short_value(&mut self, value: i16, order: ByteOrder) {
        self.put_short(i32::from(value), order);
    }

    /// `append(bs)` / `++=` of a byte string.
    pub fn append(&mut self, bytes: &ByteString) {
        self.buffer.extend_from_slice(bytes.as_slice());
    }

    /// `++=` / `addAll` of any bytes.
    pub fn add_all(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    /// `length`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// `isEmpty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// `nonEmpty`.
    #[must_use]
    pub fn non_empty(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// `sizeHint(len)`: reserves room; a size that is not positive is ignored.
    pub fn size_hint(&mut self, len: i32) {
        self.buffer
            .reserve(usize::try_from(len).unwrap_or_default());
    }

    /// `result()`: what has been written so far. The builder stays usable.
    #[must_use]
    pub fn result(&self) -> ByteString {
        ByteString::from_array(&self.buffer)
    }

    /// `asOutputStream.write(b)`: appends the low eight bits of `value`.
    pub fn write_int(&mut self, value: i32) {
        self.buffer.push(value as u8);
    }
}

impl io::Write for ByteStringBuilder {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
