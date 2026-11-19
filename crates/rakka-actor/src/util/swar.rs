// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/SWARUtil.scala (Apache-2.0).
// SWARUtil.scala was copied by Pekko from the Netty Project (Copyright 2024 The Netty Project, Apache-2.0,
// https://github.com/netty/netty/blob/d28a0fc6598b50fbe8f296831777cf4b653a475f/common/src/main/java/io/netty/util/internal/SWARUtil.java);
// the Netty notice is kept in the repository NOTICE file.
//! `SWARUtil` (AC-UTIL-028): SIMD-within-a-register helpers and multi-byte reads/writes over byte
//! slices. Values keep the signedness of the JVM types (`Long` is `i64`, `Int` is `i32`, `Short` is
//! `i16`, a `Byte` is its bit pattern `u8`).
//!
//! Pekko reads through `MethodHandles.byteArrayViewVarHandle` and falls back to byte-by-byte shifts
//! when the JVM lacks it; the oracle shows both paths agree, and Rust has a single implementation
//! (`from_be_bytes` and friends). Like Pekko, nothing checks bounds on purpose: an index that leaves
//! the slice panics (Pekko throws `IndexOutOfBoundsException`).

/// `java.nio.ByteOrder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    /// Most significant byte first.
    BigEndian,
    /// Least significant byte first.
    LittleEndian,
}

const LOW_BITS: u64 = 0x7F7F_7F7F_7F7F_7F7F;
const BYTE_ONES: u64 = 0x0101_0101_0101_0101;

/// `compilePattern(byteToFind)`: the byte repeated in all eight lanes of a word.
#[must_use]
pub fn compile_pattern(byte_to_find: u8) -> i64 {
    (u64::from(byte_to_find) * BYTE_ONES) as i64
}

/// `applyPattern(word, pattern)`: a word where each byte of `word` that equals the matching byte of
/// `pattern` has its highest bit set.
#[must_use]
pub fn apply_pattern(word: i64, pattern: i64) -> i64 {
    let input = (word ^ pattern) as u64;
    let masked_sum = (input & LOW_BITS).wrapping_add(LOW_BITS);
    !(masked_sum | input | LOW_BITS) as i64
}

/// `getIndex(word)`: index (0 = most significant byte) of the first match flagged by
/// [`apply_pattern`], or 8 if there is none. Big endian only, like Pekko.
#[must_use]
pub fn get_index(word: i64) -> i32 {
    (word.leading_zeros() >> 3) as i32
}

/// `getLastIndex(word)`: index of the last match flagged by [`apply_pattern`], or -1 if there is none.
#[must_use]
pub fn get_last_index(word: i64) -> i32 {
    if word == 0 {
        -1
    } else {
        ((63 - word.trailing_zeros()) >> 3) as i32
    }
}

fn window<const N: usize>(array: &[u8], index: usize) -> [u8; N] {
    array[index..index + N]
        .try_into()
        .expect("a slice of N bytes converts to [u8; N]")
}

/// `getLong(array, index, byteOrder)`.
#[must_use]
pub fn get_long(array: &[u8], index: usize, byte_order: ByteOrder) -> i64 {
    let bytes = window::<8>(array, index);
    match byte_order {
        ByteOrder::BigEndian => i64::from_be_bytes(bytes),
        ByteOrder::LittleEndian => i64::from_le_bytes(bytes),
    }
}

/// `getInt(array, index, byteOrder)`.
#[must_use]
pub fn get_int(array: &[u8], index: usize, byte_order: ByteOrder) -> i32 {
    let bytes = window::<4>(array, index);
    match byte_order {
        ByteOrder::BigEndian => i32::from_be_bytes(bytes),
        ByteOrder::LittleEndian => i32::from_le_bytes(bytes),
    }
}

/// `getShort(array, index, byteOrder)`.
#[must_use]
pub fn get_short(array: &[u8], index: usize, byte_order: ByteOrder) -> i16 {
    let bytes = window::<2>(array, index);
    match byte_order {
        ByteOrder::BigEndian => i16::from_be_bytes(bytes),
        ByteOrder::LittleEndian => i16::from_le_bytes(bytes),
    }
}

/// `putInt(array, index, value, byteOrder)`.
pub fn put_int(array: &mut [u8], index: usize, value: i32, byte_order: ByteOrder) {
    let bytes = match byte_order {
        ByteOrder::BigEndian => value.to_be_bytes(),
        ByteOrder::LittleEndian => value.to_le_bytes(),
    };
    array[index..index + 4].copy_from_slice(&bytes);
}

/// `putShort(array, index, value, byteOrder)`: writes the low 16 bits of `value`.
pub fn put_short(array: &mut [u8], index: usize, value: i32, byte_order: ByteOrder) {
    let low_bits = value as i16;
    let bytes = match byte_order {
        ByteOrder::BigEndian => low_bits.to_be_bytes(),
        ByteOrder::LittleEndian => low_bits.to_le_bytes(),
    };
    array[index..index + 2].copy_from_slice(&bytes);
}

/// `putLong(array, index, value, byteOrder)`.
pub fn put_long(array: &mut [u8], index: usize, value: i64, byte_order: ByteOrder) {
    let bytes = match byte_order {
        ByteOrder::BigEndian => value.to_be_bytes(),
        ByteOrder::LittleEndian => value.to_le_bytes(),
    };
    array[index..index + 8].copy_from_slice(&bytes);
}

/// `getShortBE(b0, b1)`.
#[must_use]
pub fn get_short_be(b0: u8, b1: u8) -> i16 {
    i16::from_be_bytes([b0, b1])
}

/// `getShortLE(b0, b1)`.
#[must_use]
pub fn get_short_le(b0: u8, b1: u8) -> i16 {
    i16::from_le_bytes([b0, b1])
}

/// `getIntBE(b0, b1, b2, b3)`.
#[must_use]
pub fn get_int_be(b0: u8, b1: u8, b2: u8, b3: u8) -> i32 {
    i32::from_be_bytes([b0, b1, b2, b3])
}

/// `getIntLE(b0, b1, b2, b3)`.
#[must_use]
pub fn get_int_le(b0: u8, b1: u8, b2: u8, b3: u8) -> i32 {
    i32::from_le_bytes([b0, b1, b2, b3])
}

/// `getLongBE(b0, ..., b7)`.
#[must_use]
pub fn get_long_be(bytes: [u8; 8]) -> i64 {
    i64::from_be_bytes(bytes)
}

/// `getLongLE(b0, ..., b7)`.
#[must_use]
pub fn get_long_le(bytes: [u8; 8]) -> i64 {
    i64::from_le_bytes(bytes)
}
