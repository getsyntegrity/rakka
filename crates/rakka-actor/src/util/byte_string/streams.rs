// Derived from Apache Pekko 2.0.0 actor/src/main/java/org/apache/pekko/io/UnsynchronizedByteArrayInputStream.java (Apache-2.0).
//! `UnsynchronizedByteArrayInputStream` (AC-UTIL-022) and `ByteString.asInputStream` (AC-UTIL-006.5): an
//! input stream over an array that supports `mark`/`reset` and is not thread safe. `read` returns `-1` at
//! the end of the data, like `java.io.InputStream`; `std::io::Read` is implemented on top for the Rust
//! side (it returns `Ok(0)` there, the `std::io` convention).

use std::io;

use bytes::Bytes;

use super::ByteStringError;

/// `ByteString.asInputStream` is this stream over the bytes of the value.
pub type ByteStringInputStream = UnsynchronizedByteArrayInputStream;

/// An input stream over a byte array with a movable start and a marked position.
#[derive(Clone, Debug)]
pub struct UnsynchronizedByteArrayInputStream {
    data: Bytes,
    end_of_data: usize,
    offset: usize,
    marked_offset: usize,
}

impl UnsynchronizedByteArrayInputStream {
    /// `new UnsynchronizedByteArrayInputStream(data)`: copies `data`.
    #[must_use]
    pub fn new(data: &[u8]) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(data))
    }

    /// The same over bytes that are shared, not copied.
    #[must_use]
    pub fn from_bytes(data: Bytes) -> Self {
        Self {
            end_of_data: data.len(),
            data,
            offset: 0,
            marked_offset: 0,
        }
    }

    /// `new UnsynchronizedByteArrayInputStream(data, offset, length)`: the stream starts at
    /// `min(offset, data.length)` and ends `length` bytes later or at the end of `data`.
    pub fn with_range(data: &[u8], offset: i32, length: i32) -> Result<Self, ByteStringError> {
        let offset = require_non_negative(offset, "offset")?;
        let length = require_non_negative(length, "length")?;
        let start = offset.min(data.len());
        Ok(Self {
            data: Bytes::copy_from_slice(data),
            end_of_data: start.saturating_add(length).min(data.len()),
            offset: start,
            marked_offset: start,
        })
    }

    /// `available()`.
    #[must_use]
    pub fn available(&self) -> i32 {
        i32::try_from(self.end_of_data - self.offset).unwrap_or(i32::MAX)
    }

    /// `markSupported()`: always `true`.
    #[must_use]
    pub fn mark_supported(&self) -> bool {
        true
    }

    /// `mark(readLimit)`: remembers the position; the limit is ignored.
    pub fn mark(&mut self, _read_limit: i32) {
        self.marked_offset = self.offset;
    }

    /// `reset()`: back to the marked position (the start when nothing was marked).
    pub fn reset(&mut self) {
        self.offset = self.marked_offset;
    }

    /// `read()`: the next byte as `0..=255`, or `-1` at the end.
    pub fn read(&mut self) -> i32 {
        if self.offset < self.end_of_data {
            let byte = self.data[self.offset];
            self.offset += 1;
            i32::from(byte)
        } else {
            -1
        }
    }

    /// `read(dest, off, len)`: copies up to `len` bytes into `dest[off..]` and returns how many, `0` when
    /// `len` is `0`, or `-1` at the end. A range outside `dest` fails.
    pub fn read_range(
        &mut self,
        dest: &mut [u8],
        off: i32,
        len: i32,
    ) -> Result<i32, ByteStringError> {
        let range = usize::try_from(off)
            .ok()
            .zip(usize::try_from(len).ok())
            .filter(|&(off, len)| off.checked_add(len).is_some_and(|end| end <= dest.len()))
            .ok_or_else(|| {
                ByteStringError::IndexOutOfBounds(Some(format!(
                    "Range [{off}, {off} + {len}) out of bounds for length {}",
                    dest.len()
                )))
            })?;
        let (off, len) = range;
        if len == 0 {
            return Ok(0);
        }
        let count = len.min(self.end_of_data - self.offset);
        if count == 0 {
            return Ok(-1);
        }
        dest[off..off + count].copy_from_slice(&self.data[self.offset..self.offset + count]);
        self.offset += count;
        Ok(i32::try_from(count).unwrap_or(i32::MAX))
    }

    /// `readAllBytes()`.
    pub fn read_all_bytes(&mut self) -> Vec<u8> {
        let rest = self.data[self.offset..self.end_of_data].to_vec();
        self.offset = self.end_of_data;
        rest
    }

    /// `readNBytes(len)`: up to `len` bytes; a negative `len` fails.
    pub fn read_n_bytes(&mut self, len: i32) -> Result<Vec<u8>, ByteStringError> {
        let len = usize::try_from(len)
            .map_err(|_| ByteStringError::IllegalArgument("len < 0".to_owned()))?;
        let count = len.min(self.end_of_data - self.offset);
        let bytes = self.data[self.offset..self.offset + count].to_vec();
        self.offset += count;
        Ok(bytes)
    }

    /// `readNBytes(dest, off, len)`: like `read_range` but `0` instead of `-1` at the end.
    pub fn read_n_bytes_into(
        &mut self,
        dest: &mut [u8],
        off: i32,
        len: i32,
    ) -> Result<i32, ByteStringError> {
        Ok(self.read_range(dest, off, len)?.max(0))
    }

    /// `skipNBytes(n)`: skips exactly `n` bytes (`n <= 0` skips none); with fewer left it skips them all and
    /// fails with `EOFException`.
    pub fn skip_n_bytes(&mut self, n: i64) -> Result<(), ByteStringError> {
        if n <= 0 {
            return Ok(());
        }
        let available = i64::try_from(self.end_of_data - self.offset).unwrap_or(i64::MAX);
        if n > available {
            self.offset = self.end_of_data;
            return Err(ByteStringError::EndOfFile);
        }
        self.offset += usize::try_from(n).unwrap_or_default();
        Ok(())
    }

    /// `transferTo(out)`: writes everything left to `out` and returns how many bytes.
    pub fn transfer_to(&mut self, out: &mut impl io::Write) -> io::Result<u64> {
        let rest = &self.data[self.offset..self.end_of_data];
        out.write_all(rest)?;
        self.offset = self.end_of_data;
        Ok(rest.len() as u64)
    }

    /// `skip(n)`: skips `min(n, available)` bytes and returns that count; a negative `n` fails.
    pub fn skip(&mut self, n: i64) -> Result<i64, ByteStringError> {
        if n < 0 {
            return Err(ByteStringError::IllegalArgument(
                "Skipping backward is not supported".to_owned(),
            ));
        }
        let skipped = usize::try_from(n)
            .unwrap_or(usize::MAX)
            .min(self.end_of_data - self.offset);
        self.offset += skipped;
        Ok(i64::try_from(skipped).unwrap_or(i64::MAX))
    }
}

fn require_non_negative(value: i32, name: &str) -> Result<usize, ByteStringError> {
    usize::try_from(value)
        .map_err(|_| ByteStringError::IllegalArgument(format!("{name} cannot be negative")))
}

impl io::Read for UnsynchronizedByteArrayInputStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = buf.len().min(self.end_of_data - self.offset);
        buf[..count].copy_from_slice(&self.data[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}
