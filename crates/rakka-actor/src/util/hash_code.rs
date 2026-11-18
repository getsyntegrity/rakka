// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/HashCode.scala (Apache-2.0).
//! `HashCode` (AC-UTIL-016.15): the helper that folds values into a 32-bit hash, `37 * seed + contribution`,
//! starting from [`SEED`]. All arithmetic wraps like the JVM's `Int`.
//!
//! The contribution of a value is given by [`HashContribution`]: `Boolean` is 1/0, `Char`, `Short`, `Byte`
//! and `Int` are their integer value, `Long` is `(v ^ (v >>> 32)).toInt`, `Float`/`Double` use the
//! canonical IEEE bits (`floatToIntBits`/`doubleToLongBits`), a string uses Java's `String.hashCode`
//! (polynomial 31 over UTF-16 units), and a slice folds each element in order starting from the seed.
//!
//! Differences (Q-UTIL-14 (b)): Rust has no universal `hashCode`, so the "any object" branch exists
//! only for the types implementing the trait; `hash(seed, null)` (`IllegalArgumentException`) has no
//! counterpart because Rust has no `null`.

/// `HashCode.SEED`.
pub const SEED: i32 = 23;

const PRIME: i32 = 37;

fn first_term(seed: i32) -> i32 {
    PRIME.wrapping_mul(seed)
}

/// A value that `HashCode.hash(seed, value)` can fold into a hash.
pub trait HashContribution {
    /// `HashCode.hash(seed, self)`.
    fn hash_into(&self, seed: i32) -> i32;
}

/// `HashCode.hash(seed, value)`.
#[must_use]
pub fn hash<T: HashContribution + ?Sized>(seed: i32, value: &T) -> i32 {
    value.hash_into(seed)
}

impl HashContribution for bool {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add(i32::from(*self))
    }
}

/// A `Char` is a UTF-16 unit.
impl HashContribution for u16 {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add(i32::from(*self))
    }
}

impl HashContribution for i8 {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add(i32::from(*self))
    }
}

impl HashContribution for i16 {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add(i32::from(*self))
    }
}

impl HashContribution for i32 {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add(*self)
    }
}

impl HashContribution for i64 {
    fn hash_into(&self, seed: i32) -> i32 {
        first_term(seed).wrapping_add((*self ^ ((*self as u64) >> 32) as i64) as i32)
    }
}

impl HashContribution for f32 {
    fn hash_into(&self, seed: i32) -> i32 {
        // floatToIntBits: every NaN collapses to the canonical quiet NaN.
        let bits = if self.is_nan() {
            0x7fc0_0000_u32
        } else {
            self.to_bits()
        };
        (bits as i32).hash_into(seed)
    }
}

impl HashContribution for f64 {
    fn hash_into(&self, seed: i32) -> i32 {
        let bits = if self.is_nan() {
            0x7ff8_0000_0000_0000_u64
        } else {
            self.to_bits()
        };
        (bits as i64).hash_into(seed)
    }
}

/// `String.hashCode` as the object contribution: `s[0]*31^(n-1) + ... + s[n-1]` over UTF-16 units.
impl HashContribution for str {
    fn hash_into(&self, seed: i32) -> i32 {
        let code = self.encode_utf16().fold(0_i32, |h, unit| {
            h.wrapping_mul(31).wrapping_add(i32::from(unit))
        });
        code.hash_into(seed)
    }
}

/// Arrays chain the hash of every element, starting from the seed.
impl<T: HashContribution> HashContribution for [T] {
    fn hash_into(&self, seed: i32) -> i32 {
        self.iter().fold(seed, |acc, item| item.hash_into(acc))
    }
}

impl<T: HashContribution + ?Sized> HashContribution for &T {
    fn hash_into(&self, seed: i32) -> i32 {
        (**self).hash_into(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_values_from_the_contract() {
        assert_eq!(hash(23, &5_i32), 856);
        assert_eq!(hash(23, &true), 852);
        assert_eq!(hash(23, &5_000_000_000_i64), 705_033_556);
        assert_eq!(hash(23, "abc"), 97_205);
        assert_eq!(hash(23, &1.5_f64), 1_073_218_387);
        assert_eq!(hash(23, &[1_i32, 2][..]), 31_526);
    }

    #[test]
    fn nan_payloads_collapse_and_wrapping_holds() {
        assert_eq!(hash(23, &f32::from_bits(0x7fc0_0001)), hash(23, &f32::NAN));
        assert_eq!(
            hash(23, &f64::from_bits(0xfff8_0000_0000_0000)),
            hash(23, &f64::NAN)
        );
        assert_eq!(hash(i32::MIN, &false), i32::MIN.wrapping_mul(37));
        assert_eq!(SEED, 23);
    }
}
