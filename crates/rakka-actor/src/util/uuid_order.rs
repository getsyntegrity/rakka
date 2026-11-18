// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/UUIDComparator.scala (Apache-2.0).
//! `UUIDComparator` (AC-UTIL-016.14): the order Pekko gives to 128-bit UUIDs.
//!
//! Pekko compares first by version number; two version-1 (time-based) UUIDs by their 60-bit `timestamp()` and, on a
//! tie, by the least significant 64 bits; every other pair by the most and then the least significant 64 bits. All
//! 64-bit comparisons are **unsigned** (`java.util.UUID.compareTo` sign-extends, which Pekko calls wrong).
//!
//! The bit layout (`version()`, `timestamp()`) follows the public Javadoc of `java.util.UUID` and was checked against
//! the oracle (Q-X-18).
//!
//! Differences: [`JavaUuid`] is a plain pair of 64-bit halves (no parsing or generation, which no Pekko consumer of
//! this comparator needs). [`pekko_compare_uuid`] reproduces the raw `Int` Pekko returns (`-3` for a version
//! difference of three, the plain difference of the 32-bit halves); [`compare_uuid`] and `Ord` give only its sign.

use std::cmp::Ordering;

use crate::actor::UnsupportedOperationError;

/// A `java.util.UUID`: the most and least significant 64 bits, as signed `Long`s like Java.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JavaUuid {
    msb: i64,
    lsb: i64,
}

impl JavaUuid {
    /// `new UUID(mostSigBits, leastSigBits)`.
    #[must_use]
    pub fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self {
            msb: most_significant_bits,
            lsb: least_significant_bits,
        }
    }

    /// `getMostSignificantBits`.
    #[must_use]
    pub fn most_significant_bits(&self) -> i64 {
        self.msb
    }

    /// `getLeastSignificantBits`.
    #[must_use]
    pub fn least_significant_bits(&self) -> i64 {
        self.lsb
    }

    /// `version()`: the four version bits, `0..=15`.
    #[must_use]
    pub fn version(&self) -> i32 {
        ((self.msb >> 12) & 0xf) as i32
    }

    /// `timestamp()`: the 60-bit time of a version-1 UUID, built from `time_low`, `time_mid` and `time_hi`.
    /// Any other version throws `UnsupportedOperationException("Not a time-based UUID")`.
    pub fn timestamp(&self) -> Result<i64, UnsupportedOperationError> {
        if self.version() != 1 {
            return Err(UnsupportedOperationError::new("Not a time-based UUID"));
        }
        let msb = self.msb as u64;
        let time_low = msb >> 32;
        let time_mid = (msb >> 16) & 0xFFFF;
        let time_hi = msb & 0x0FFF;
        Ok(((time_hi << 48) | (time_mid << 32) | time_low) as i64)
    }
}

/// `UUIDComparator.compare`: the raw `Int` Pekko returns (only its sign and zero are meaningful).
#[must_use]
pub fn pekko_compare_uuid(a: JavaUuid, b: JavaUuid) -> i32 {
    let version = a.version();
    let diff = version - b.version();
    if diff != 0 {
        return diff;
    }
    if version == 1 {
        // `timestamp()` only fails for a version other than 1, which this branch excludes.
        let by_time = compare_ulongs(
            a.timestamp().unwrap_or_default(),
            b.timestamp().unwrap_or_default(),
        );
        if by_time == 0 {
            compare_ulongs(a.lsb, b.lsb)
        } else {
            by_time
        }
    } else {
        let by_msb = compare_ulongs(a.msb, b.msb);
        if by_msb == 0 {
            compare_ulongs(a.lsb, b.lsb)
        } else {
            by_msb
        }
    }
}

fn compare_ulongs(l1: i64, l2: i64) -> i32 {
    let diff = compare_uints((l1 >> 32) as i32, (l2 >> 32) as i32);
    if diff == 0 {
        compare_uints(l1 as i32, l2 as i32)
    } else {
        diff
    }
}

/// Same sign: plain subtraction (it cannot overflow); different signs: the negative one is the greater.
fn compare_uints(i1: i32, i2: i32) -> i32 {
    if i1 < 0 {
        if i2 < 0 { i1.wrapping_sub(i2) } else { 1 }
    } else if i2 < 0 {
        -1
    } else {
        i1.wrapping_sub(i2)
    }
}

/// The sign of [`pekko_compare_uuid`] as an [`Ordering`].
#[must_use]
pub fn compare_uuid(a: JavaUuid, b: JavaUuid) -> Ordering {
    pekko_compare_uuid(a, b).cmp(&0)
}

impl PartialOrd for JavaUuid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JavaUuid {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_uuid(*self, *other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(msb: u64, lsb: u64) -> JavaUuid {
        JavaUuid::new(msb as i64, lsb as i64)
    }

    #[test]
    fn measured_values_from_the_contract() {
        // compare(v1, v4) = -3: the plain difference of the versions.
        assert_eq!(pekko_compare_uuid(u(0x1000, 0), u(0x4000, 0)), -3);
        // Unsigned: a least significant half with its top bit set is the greater one.
        assert_eq!(
            pekko_compare_uuid(
                u(0x4000_0000_0000_0000, 0x8000_0000_0000_0001),
                u(0x4000_0000_0000_0000, 1)
            ),
            1
        );
        assert_eq!(pekko_compare_uuid(u(0x4000, 1), u(0x4000, 2)), -1);
    }

    #[test]
    fn timestamp_rebuilds_the_sixty_bits() {
        assert_eq!(u(0x0000_0001_0000_1000, 0).timestamp(), Ok(1));
        assert_eq!(u(0x0000_0000_0001_1000, 0).timestamp(), Ok(1 << 32));
        assert_eq!(u(0x0000_0000_0000_1001, 0).timestamp(), Ok(1 << 48));
        assert_eq!(
            u(0xFFFF_FFFF_FFFF_1FFF, 0).timestamp(),
            Ok((1_i64 << 60) - 1)
        );
        let err = u(0x4000, 0).timestamp().unwrap_err();
        assert_eq!(err.java_class(), "UnsupportedOperationException");
        assert_eq!(err.message(), "Not a time-based UUID");
    }

    #[test]
    fn version_one_orders_by_timestamp_not_by_msb() {
        let big_msb_small_time = u(0xFFFF_FFFF_FFFF_1000, 0);
        let small_msb_big_time = u(0x0000_0000_0000_1FFF, 0);
        assert_eq!(
            compare_uuid(big_msb_small_time, small_msb_big_time),
            Ordering::Less
        );
        assert_eq!(
            compare_uuid(
                u(0x1234_5678_9ABC_1DEF, u64::MAX),
                u(0x1234_5678_9ABC_1DEF, 0)
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn ord_sorts_like_pekko() {
        let mut all = vec![
            u(0x4000, u64::MAX),
            u(0x1000, 5),
            u(0x4000, 0),
            u(0x1FFF, 0),
        ];
        all.sort();
        assert_eq!(
            all,
            vec![
                u(0x1000, 5),
                u(0x1FFF, 0),
                u(0x4000, 0),
                u(0x4000, u64::MAX)
            ]
        );
        assert_eq!(u(1, 1).partial_cmp(&u(1, 1)), Some(Ordering::Equal));
    }

    #[test]
    fn accessors_return_the_halves() {
        let x = JavaUuid::new(-2, 7);
        assert_eq!(
            (x.most_significant_bits(), x.least_significant_bits()),
            (-2, 7)
        );
        assert_eq!(x.version(), 15);
    }
}
