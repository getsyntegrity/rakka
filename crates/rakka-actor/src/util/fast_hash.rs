// Derived from Apache Pekko 2.0.0 actor/src/main/java/org/apache/pekko/util/Unsafe.java (Apache-2.0).
//! `Unsafe.fastHash` (AC-UTIL-020): the xorshift128+ hash over the characters of a string. The CAS
//! helpers of the other `Abstract*` classes have no counterpart here: each owner of such a field
//! uses `std::sync::atomic` (Q-UTIL-11), and `copyUSAsciiStrToBytes` for ASCII text is
//! `str::as_bytes`.

/// `Unsafe.fastHash(str)` for ASCII text.
///
/// Returns `None` for text with a non-ASCII character: Pekko's result there depends on the
/// algorithm the JVM selects (Latin-1 sign extension or `?` replacement, and an
/// `ArrayIndexOutOfBoundsException` for characters outside the BMP), which is pending Q-UTIL-19.
#[must_use]
pub fn fast_hash(text: &str) -> Option<i32> {
    if !text.is_ascii() {
        return None;
    }
    let (mut s0, mut s1) = (391_408_i64, 601_258_i64);
    for &byte in text.as_bytes() {
        let mut x = s0 ^ i64::from(byte); // mix the character into the PRNG state
        let mut y = s1;
        // xorshift128+ round; `>>>` is a logical shift
        s0 = y;
        x ^= x << 23;
        y ^= ((y as u64) >> 26) as i64;
        x ^= ((x as u64) >> 17) as i64;
        s1 = x ^ y;
    }
    Some(s0.wrapping_add(s1) as i32)
}
