// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
// Original implementation: the HOCON behaviour was measured against Typesafe Config 1.4.9 (Apache-2.0, the library Pekko uses) with the oracle; none of its code was copied.
//! Exact decimal arithmetic for memory sizes.
//!
//! Typesafe Config computes `<number> <unit>` with `BigDecimal`/`BigInteger` (`8.2 MB` is exactly `8200000`, and the
//! out-of-range message shows the exact product). `f64` cannot do that, so this module multiplies a decimal literal by a
//! unit multiplier without losing digits and truncates toward zero, like `BigDecimal.toBigInteger`.

const LIMB_BASE: u64 = 1_000_000_000;

/// A non-negative integer of any size: base-10⁹ limbs, least significant first, without leading zero limbs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BigNatural(Vec<u32>);

impl BigNatural {
    fn from_decimal_digits(digits: &str) -> BigNatural {
        let mut limbs = Vec::new();
        let bytes = digits.as_bytes();
        let mut end = bytes.len();
        while end > 0 {
            let start = end.saturating_sub(9);
            let limb = bytes[start..end]
                .iter()
                .fold(0u32, |acc, d| acc * 10 + u32::from(d - b'0'));
            limbs.push(limb);
            end = start;
        }
        let mut natural = BigNatural(limbs);
        natural.trim();
        natural
    }

    fn from_u128(mut value: u128) -> BigNatural {
        let mut limbs = Vec::new();
        while value > 0 {
            limbs.push(u32::try_from(value % u128::from(LIMB_BASE)).expect("below 10^9"));
            value /= u128::from(LIMB_BASE);
        }
        BigNatural(limbs)
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn times(&self, other: &BigNatural) -> BigNatural {
        let mut limbs = vec![0u64; self.0.len() + other.0.len() + 1];
        for (i, a) in self.0.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in other.0.iter().enumerate() {
                let cell = limbs[i + j] + u64::from(*a) * u64::from(*b) + carry;
                limbs[i + j] = cell % LIMB_BASE;
                carry = cell / LIMB_BASE;
            }
            let mut k = i + other.0.len();
            while carry > 0 {
                let cell = limbs[k] + carry;
                limbs[k] = cell % LIMB_BASE;
                carry = cell / LIMB_BASE;
                k += 1;
            }
        }
        let mut product = BigNatural(
            limbs
                .into_iter()
                .map(|l| u32::try_from(l).expect("below 10^9"))
                .collect(),
        );
        product.trim();
        product
    }

    fn times_ten(&mut self) {
        let mut carry = 0u64;
        for limb in &mut self.0 {
            let cell = u64::from(*limb) * 10 + carry;
            *limb = u32::try_from(cell % LIMB_BASE).expect("below 10^9");
            carry = cell / LIMB_BASE;
        }
        if carry > 0 {
            self.0.push(u32::try_from(carry).expect("below 10^9"));
        }
    }

    /// Integer division by ten, dropping the remainder.
    fn divide_by_ten(&mut self) {
        let mut remainder = 0u64;
        for limb in self.0.iter_mut().rev() {
            let cell = remainder * LIMB_BASE + u64::from(*limb);
            *limb = u32::try_from(cell / 10).expect("below 10^9");
            remainder = cell % 10;
        }
        self.trim();
    }

    pub(crate) fn to_decimal_string(&self) -> String {
        let mut limbs = self.0.iter().rev();
        match limbs.next() {
            None => "0".to_owned(),
            Some(first) => limbs.fold(first.to_string(), |text, limb| format!("{text}{limb:09}")),
        }
    }
}

/// The most digits a literal's exponent may add before the literal is refused (Java would build the number; nothing
/// real needs it, and it keeps the arithmetic bounded).
const MAX_EXPONENT_SHIFT: usize = 10_000;

/// `|literal| * multiplier`, truncated toward zero, and whether the literal is negative; `None` if `literal` is not a
/// `java.math.BigDecimal` literal (`[+-]?(digits[.digits*]|.digits)([eE][+-]?digits)?`) or its exponent is absurd.
pub(crate) fn scaled_product(literal: &str, multiplier: u128) -> Option<(bool, BigNatural)> {
    let negative = literal.starts_with('-');
    let unsigned = literal.strip_prefix(['+', '-']).unwrap_or(literal);
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (&unsigned[..at], Some(&unsigned[at + 1..])),
        None => (unsigned, None),
    };
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if integer.len() + fraction.len() == 0 || !all_digits(integer) || !all_digits(fraction) {
        return None;
    }
    let exponent: i64 = match exponent {
        None => 0,
        Some(e) => {
            let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
            if digits.is_empty() || !all_digits(digits) {
                return None;
            }
            e.parse().ok()?
        }
    };
    // value = digits * 10^-scale
    let scale = i64::try_from(fraction.len()).ok()? - exponent;
    let digits = BigNatural::from_decimal_digits(&format!("{integer}{fraction}"));
    let mut product = digits.times(&BigNatural::from_u128(multiplier));
    if product.is_zero() {
        return Some((negative, product));
    }
    let shift = usize::try_from(scale.unsigned_abs()).ok()?;
    if scale > 0 {
        if shift > product.to_decimal_string().len() {
            return Some((negative, BigNatural(Vec::new())));
        }
        for _ in 0..shift {
            product.divide_by_ten();
        }
    } else {
        if shift > MAX_EXPONENT_SHIFT {
            return None;
        }
        for _ in 0..shift {
            product.times_ten();
        }
    }
    Some((negative, product))
}
