// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/FrequencySketch.scala (Apache-2.0).
//! `FrequencySketch` and `FastFrequencySketch` (AC-UTIL-017.1): frequency sketches for estimating the
//! popularity of items, to implement the TinyLFU cache admission policy (cluster-sharding passivation
//! in Pekko). Both include the TinyLFU reset operation, which periodically halves every counter.
//!
//! Q-UTIL-14 (a): the hashes are exact, so the counters and sizes are the ones Pekko 2.0.0 prints:
//! `scala.util.hashing.MurmurHash3` (`stringHash`, `mixLast`, `finalizeHash`) and Java's `hashCode`
//! of a string (UTF-16 units). The 2.0.0 overflow validation is included: `capacity` rounded up to a
//! power of two times `widthMultiplier` must not exceed `Int.MaxValue`, and the reset size is capped
//! there too.
//!
//! The sketches are not thread safe. Counters live in `u64` slots, with the Java semantics of `>>>`
//! and of shifts by 64 (which wrap), where the original uses `Long`.

use crate::actor::IllegalArgumentError;

/// `scala.util.hashing.MurmurHash3`: the parts the sketch uses.
pub mod murmur3 {
    const C1: i32 = 0xcc9e_2d51_u32 as i32;
    const C2: i32 = 0x1b87_3593;

    /// `mixLast(hash, data)`.
    #[must_use]
    pub fn mix_last(hash: i32, data: i32) -> i32 {
        let mut k = data.wrapping_mul(C1);
        k = k.rotate_left(15);
        k = k.wrapping_mul(C2);
        hash ^ k
    }

    /// `mix(hash, data)`.
    #[must_use]
    pub fn mix(hash: i32, data: i32) -> i32 {
        let h = mix_last(hash, data).rotate_left(13);
        h.wrapping_mul(5).wrapping_add(0xe654_6b64_u32 as i32)
    }

    fn avalanche(hash: i32) -> i32 {
        let mut h = hash as u32;
        h ^= h >> 16;
        h = h.wrapping_mul(0x85eb_ca6b);
        h ^= h >> 13;
        h = h.wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        h as i32
    }

    /// `finalizeHash(hash, length)`.
    #[must_use]
    pub fn finalize_hash(hash: i32, length: i32) -> i32 {
        avalanche(hash ^ length)
    }

    /// `stringHash(str, seed)`: over the UTF-16 units of the string, two at a time.
    #[must_use]
    pub fn string_hash(text: &str, seed: i32) -> i32 {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut h = seed;
        let mut i = 0;
        while i + 1 < units.len() {
            let data = (i32::from(units[i]) << 16).wrapping_add(i32::from(units[i + 1]));
            h = mix(h, data);
            i += 2;
        }
        if i < units.len() {
            h = mix_last(h, i32::from(units[i]));
        }
        finalize_hash(h, units.len() as i32)
    }
}

/// `FrequencySketch.Bits`: bit tricks on `Int`, with Java's overflow.
pub mod bits {
    /// `isPowerOfTwo(i)`: also true for 0 and `Int.MinValue`.
    #[must_use]
    pub fn is_power_of_two(i: i32) -> bool {
        i & i.wrapping_sub(1) == 0
    }

    /// `powerOfTwoExponent(i)`.
    #[must_use]
    pub fn power_of_two_exponent(i: i32) -> i32 {
        32 - i.wrapping_sub(1).leading_zeros() as i32
    }

    /// `ceilingPowerOfTwo(i)`: `1 << -numberOfLeadingZeros(i - 1)`, where a Java shift uses the low
    /// five bits of the distance, so it is 1 for 0 and 1, and negative past `2^30`.
    #[must_use]
    pub fn ceiling_power_of_two(i: i32) -> i32 {
        let distance = (i.wrapping_sub(1).leading_zeros() as i32).wrapping_neg();
        1_i32.wrapping_shl(distance as u32)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// `FrequencySketch.Hasher[A]`: the hash function of the element type (sealed, like Pekko's).
pub trait Hasher<A: ?Sized>: sealed::Sealed {
    /// The 32-bit hash of `value`.
    fn hash(&self, value: &A) -> i32;
}

/// `Hasher.StringHasher`: `MurmurHash3.stringHash(value, seed)`.
#[derive(Clone, Copy, Debug)]
pub struct StringHasher {
    seed: i32,
}

/// `Hasher.DefaultSeed`.
pub const DEFAULT_SEED: i32 = 135_283_237;

impl StringHasher {
    /// A hasher that mixes `seed` into every hash.
    pub const fn new(seed: i32) -> Self {
        Self { seed }
    }
}

impl Default for StringHasher {
    /// `Hasher.StringHasher`: the implicit hasher, with the default seed.
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

impl sealed::Sealed for StringHasher {}

impl Hasher<str> for StringHasher {
    fn hash(&self, value: &str) -> i32 {
        murmur3::string_hash(value, self.seed)
    }
}

const WIDTH_OVERFLOW_PREFIX: &str = "requirement failed: computed width (";

/// `width = widthMultiplier * ceilingPowerOfTwo(capacity)` computed in `Long`, which must fit in an `Int`,
/// and the reset size, capped at `Int.MaxValue`.
fn sketch_dimensions(
    capacity: i32,
    width_multiplier: i32,
    reset_multiplier: f64,
) -> Result<(i32, i32), IllegalArgumentError> {
    let width_long = i64::from(width_multiplier) * i64::from(bits::ceiling_power_of_two(capacity));
    if width_long > i64::from(i32::MAX) {
        return Err(IllegalArgumentError::new(format!(
            "{WIDTH_OVERFLOW_PREFIX}{width_long}) exceeds Int.MaxValue; reduce capacity or widthMultiplier"
        )));
    }
    let reset_size =
        i64::from(i32::MAX).min((reset_multiplier * f64::from(capacity)) as i64) as i32;
    Ok((width_long as i32, reset_size))
}

fn requirement_failed(message: &str) -> IllegalArgumentError {
    IllegalArgumentError::new(format!("requirement failed: {message}"))
}

/// Rows of a sketch: `Array.fill(depth)` is empty for a depth that is not positive.
fn rows(depth: i32) -> usize {
    usize::try_from(depth).unwrap_or(0)
}

/// A generalised frequency sketch with configurable depth (number of hash functions) and counter size.
///
/// The matrix of counters is a two-dimensional array of 64-bit slots, each holding several counters
/// depending on the counter size. To get pairwise independent hash functions for the given depth it
/// combines two hash functions ("Building a Better Bloom Filter": `g_i(x) = h1(x) + i * h2(x)`).
pub struct FrequencySketch<A: ?Sized, H> {
    depth: i32,
    counter_width: i32,
    slots: i32,
    row_width: usize,
    column_mask: i32,
    slot_shift: i32,
    slot_mask: i32,
    counter_shift: i32,
    counter_mask: u64,
    odd_mask: u64,
    reset_mask: u64,
    reset_size: i32,
    matrix: Vec<Vec<u64>>,
    row_sizes: Vec<i32>,
    updated_size: i32,
    hasher: H,
    element: std::marker::PhantomData<fn(&A)>,
}

impl<A: ?Sized, H: Hasher<A>> FrequencySketch<A, H> {
    /// `FrequencySketch(capacity, widthMultiplier, resetMultiplier, depth, counterBits)(hasher)`: a sketch
    /// for a cache of `capacity` items (rounded up to a power of two). The defaults of Pekko are 4, 10,
    /// 4 and 4 (see [`FrequencySketch::with_capacity`]).
    pub fn new(
        capacity: i32,
        width_multiplier: i32,
        reset_multiplier: f64,
        depth: i32,
        counter_bits: i32,
        hasher: H,
    ) -> Result<Self, IllegalArgumentError> {
        let (width, reset_size) = sketch_dimensions(capacity, width_multiplier, reset_multiplier)?;
        Self::from_parts(depth, width, counter_bits, reset_size, hasher)
    }

    /// `FrequencySketch(capacity)` with every other parameter at its default.
    pub fn with_capacity(capacity: i32, hasher: H) -> Result<Self, IllegalArgumentError> {
        Self::new(capacity, 4, 10.0, 4, 4, hasher)
    }

    /// `new FrequencySketch(depth, width, counterBits, resetSize, hasher)`.
    pub fn from_parts(
        depth: i32,
        width: i32,
        counter_bits: i32,
        reset_size: i32,
        hasher: H,
    ) -> Result<Self, IllegalArgumentError> {
        if !bits::is_power_of_two(width) {
            return Err(requirement_failed("width must be a power of two"));
        }
        if ![2, 4, 8, 16, 32, 64].contains(&counter_bits) {
            return Err(requirement_failed(
                "counterBits must be 2, 4, 8, 16, 32, or 64 bits",
            ));
        }
        let slots = 64 / counter_bits;
        let row_width = usize::try_from(width / slots).unwrap_or(0).max(1);
        let counter_mask = if counter_bits == 64 {
            i64::MAX as u64
        } else {
            (1_u64 << counter_bits) - 1
        };
        let odd_mask = (1..=slots).fold(1_u64, |mask, count| {
            mask | 1_u64.wrapping_shl((count * counter_bits) as u32)
        });
        let counter_reset_mask = counter_mask >> 1;
        let reset_mask = (1..=slots).fold(counter_reset_mask, |mask, count| {
            mask | counter_reset_mask.wrapping_shl((count * counter_bits) as u32)
        });
        Ok(Self {
            depth,
            counter_width: counter_bits,
            slots,
            row_width,
            column_mask: width.wrapping_sub(1),
            slot_shift: bits::power_of_two_exponent(slots),
            slot_mask: slots - 1,
            counter_shift: bits::power_of_two_exponent(counter_bits),
            counter_mask,
            odd_mask,
            reset_mask,
            reset_size,
            matrix: vec![vec![0; row_width]; rows(depth)],
            row_sizes: vec![0; rows(depth)],
            updated_size: 0,
            hasher,
            element: std::marker::PhantomData,
        })
    }

    /// `size`: the current size of the sketch, the number of incremented counters.
    pub fn size(&self) -> i32 {
        self.updated_size
    }

    /// `rehash(hash)`.
    fn rehash(hash: i32) -> i32 {
        murmur3::finalize_hash(murmur3::mix_last(hash, hash), 2)
    }

    fn position(&self, hash: i32) -> (usize, u32) {
        let column = ((hash & self.column_mask) as u32) >> self.slot_shift;
        let slot = ((hash & self.slot_mask) << self.counter_shift) as u32;
        (column as usize, slot)
    }

    fn get_counter(&self, row: usize, hash: i32) -> i32 {
        let (column, slot) = self.position(hash);
        ((self.matrix[row][column] >> slot) & self.counter_mask) as i32
    }

    fn increment_counter(&mut self, row: usize, hash: i32) -> bool {
        let (column, slot) = self.position(hash);
        let mask = self.counter_mask << slot;
        if self.matrix[row][column] & mask != mask {
            self.matrix[row][column] = self.matrix[row][column].wrapping_add(1_u64 << slot);
            self.row_sizes[row] += 1;
            true
        } else {
            false
        }
    }

    /// `frequency(value)`: the estimated frequency, limited by the maximum of the counters. Frequencies
    /// are also periodically halved as an aging mechanism.
    pub fn frequency(&self, value: &A) -> i32 {
        let hash1 = self.hasher.hash(value);
        let hash2 = Self::rehash(hash1);
        let mut min_count = i32::MAX;
        for row in 0..rows(self.depth) {
            let hash = hash1.wrapping_add((row as i32).wrapping_mul(hash2));
            min_count = min_count.min(self.get_counter(row, hash));
        }
        min_count
    }

    /// `increment(value)`: increments the estimated frequency, limited by the maximum of the counters.
    pub fn increment(&mut self, value: &A) {
        let hash1 = self.hasher.hash(value);
        let hash2 = Self::rehash(hash1);
        let mut updated = false;
        for row in 0..rows(self.depth) {
            let hash = hash1.wrapping_add((row as i32).wrapping_mul(hash2));
            updated |= self.increment_counter(row, hash);
        }
        if updated {
            self.updated_size += 1;
            if self.updated_size == self.reset_size {
                self.reset();
            }
        }
    }

    /// The TinyLFU reset operation: halves every counter, adjusting for the truncation of the integer
    /// division by the number of odd counters of each row (each off by 0.5).
    fn reset(&mut self) {
        for row in 0..rows(self.depth) {
            let mut odd = 0_i32;
            for column in 0..self.row_width {
                let slot = self.matrix[row][column];
                odd += (slot & self.odd_mask).count_ones() as i32;
                self.matrix[row][column] = (slot >> 1) & self.reset_mask;
            }
            self.row_sizes[row] = ((self.row_sizes[row] - odd) as u32 >> 1) as i32;
        }
        self.updated_size = self.row_sizes.iter().copied().max().unwrap_or(0);
    }

    /// `toDebugString`: a table with all the counters, for debugging (small) sketches.
    pub fn to_debug_string(&self) -> String {
        debug_string(
            &self.matrix,
            self.row_width,
            self.slots,
            self.counter_width,
            self.counter_mask,
        )
    }
}

/// An element whose `hashCode` is the JVM's (what `FastFrequencySketch` uses).
pub trait JavaHashCode {
    /// The value `hashCode()` would return on the JVM.
    fn java_hash_code(&self) -> i32;
}

impl JavaHashCode for str {
    /// `String.hashCode`: polynomial of 31 over the UTF-16 units.
    fn java_hash_code(&self) -> i32 {
        self.encode_utf16().fold(0_i32, |h, unit| {
            h.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
}

impl JavaHashCode for String {
    fn java_hash_code(&self) -> i32 {
        self.as_str().java_hash_code()
    }
}

impl JavaHashCode for i32 {
    fn java_hash_code(&self) -> i32 {
        *self
    }
}

impl JavaHashCode for i64 {
    fn java_hash_code(&self) -> i32 {
        (*self ^ ((*self as u64) >> 32) as i64) as i32
    }
}

// seeds are large primes between 2^63 and 2^64
const SEEDS: [i64; 4] = [
    0xC3A5_C85C_97CB_3127_u64 as i64,
    0xB492_B66F_BE98_F273_u64 as i64,
    0x9AE1_6A3B_2F90_404F_u64 as i64,
    0xCBF2_9CE4_8422_2325_u64 as i64,
];

const FAST_DEPTH: usize = 4;
const FAST_SLOT_SHIFT: u32 = 4;
const FAST_SLOT_MASK: i32 = 0xF;
const FAST_COUNTER_SHIFT: u32 = 2;
const FAST_COUNTER_MASK: u64 = 0xF;
const FAST_ODD_MASK: u64 = 0x1111_1111_1111_1111;
const FAST_RESET_MASK: u64 = 0x7777_7777_7777_7777;

/// A faster implementation of the frequency sketch (around twice as fast): fixed depth of 4 and
/// counters of 4 bits (0-15), so constants can be used, and its own rehashing of the item hash codes.
/// Inspired by the approach of the Caffeine caching library.
pub struct FastFrequencySketch<A: ?Sized> {
    row_width: usize,
    index_mask: i32,
    reset_size: i32,
    matrix: Vec<Vec<u64>>,
    row_sizes: [i32; FAST_DEPTH],
    updated_size: i32,
    element: std::marker::PhantomData<fn(&A)>,
}

impl<A: JavaHashCode + ?Sized> FastFrequencySketch<A> {
    /// `FastFrequencySketch(capacity, widthMultiplier, resetMultiplier)` (the defaults are 4 and 10).
    pub fn new(
        capacity: i32,
        width_multiplier: i32,
        reset_multiplier: f64,
    ) -> Result<Self, IllegalArgumentError> {
        let (width, reset_size) = sketch_dimensions(capacity, width_multiplier, reset_multiplier)?;
        Self::from_parts(width, reset_size)
    }

    /// `new FastFrequencySketch(width, resetSize)`.
    pub fn from_parts(width: i32, reset_size: i32) -> Result<Self, IllegalArgumentError> {
        if !bits::is_power_of_two(width) {
            return Err(requirement_failed("width must be a power of two"));
        }
        let row_width = ((width as u32) >> FAST_SLOT_SHIFT).max(1) as usize;
        Ok(Self {
            row_width,
            index_mask: width.wrapping_sub(1),
            reset_size,
            matrix: vec![vec![0; row_width]; FAST_DEPTH],
            row_sizes: [0; FAST_DEPTH],
            updated_size: 0,
            element: std::marker::PhantomData,
        })
    }

    /// `size`.
    pub fn size(&self) -> i32 {
        self.updated_size
    }

    /// A low-bias hash function found by Hash Function Prospector (public domain).
    fn rehash(hash: i32) -> i32 {
        let mut x = hash as u32;
        x = ((x >> 15) ^ x).wrapping_mul(0xD168_AAAD);
        x = ((x >> 15) ^ x).wrapping_mul(0xAF72_3597);
        ((x >> 15) ^ x) as i32
    }

    fn index(&self, hash: i32, seed: i64) -> i32 {
        let x = i64::from(hash).wrapping_add(seed).wrapping_mul(seed);
        (x.wrapping_add(((x as u64) >> 32) as i64) as i32) & self.index_mask
    }

    fn get_counter(&self, row: usize, index: i32) -> i32 {
        let column = ((index as u32) >> FAST_SLOT_SHIFT) as usize;
        let slot = ((index & FAST_SLOT_MASK) as u32) << FAST_COUNTER_SHIFT;
        ((self.matrix[row][column] >> slot) & FAST_COUNTER_MASK) as i32
    }

    fn increment_counter(&mut self, row: usize, index: i32) -> bool {
        let column = ((index as u32) >> FAST_SLOT_SHIFT) as usize;
        let slot = ((index & FAST_SLOT_MASK) as u32) << FAST_COUNTER_SHIFT;
        let mask = FAST_COUNTER_MASK << slot;
        if self.matrix[row][column] & mask != mask {
            self.matrix[row][column] = self.matrix[row][column].wrapping_add(1_u64 << slot);
            self.row_sizes[row] += 1;
            true
        } else {
            false
        }
    }

    /// `frequency(value)`.
    pub fn frequency(&self, value: &A) -> i32 {
        let hash = Self::rehash(value.java_hash_code());
        (0..FAST_DEPTH)
            .map(|row| self.get_counter(row, self.index(hash, SEEDS[row])))
            .min()
            .unwrap_or(i32::MAX)
    }

    /// `increment(value)`.
    pub fn increment(&mut self, value: &A) {
        let hash = Self::rehash(value.java_hash_code());
        let mut updated = false;
        for (row, seed) in SEEDS.iter().enumerate() {
            let index = self.index(hash, *seed);
            updated |= self.increment_counter(row, index);
        }
        if updated {
            self.updated_size += 1;
            if self.updated_size == self.reset_size {
                self.reset();
            }
        }
    }

    fn reset(&mut self) {
        for row in 0..FAST_DEPTH {
            let mut odd = 0_i32;
            for column in 0..self.row_width {
                let slot = self.matrix[row][column];
                odd += (slot & FAST_ODD_MASK).count_ones() as i32;
                self.matrix[row][column] = (slot >> 1) & FAST_RESET_MASK;
            }
            self.row_sizes[row] = ((self.row_sizes[row] - odd) as u32 >> 1) as i32;
        }
        self.updated_size = self.row_sizes.iter().copied().max().unwrap_or(0);
    }

    /// `toDebugString`.
    pub fn to_debug_string(&self) -> String {
        debug_string(&self.matrix, self.row_width, 16, 4, FAST_COUNTER_MASK)
    }
}

/// `FrequencySketchUtil.debugString`: a pretty table with all the counters of a (small) sketch.
fn debug_string(
    matrix: &[Vec<u64>],
    row_width: usize,
    slots: i32,
    counter_width: i32,
    counter_max: u64,
) -> String {
    fn digits(n: u64) -> usize {
        (n as f64).log10().floor() as usize + 1
    }
    let index_digits = digits(row_width as u64);
    let counter_digits = 2.max(digits(counter_max));
    let slots = slots as usize;
    let divider = |start: &str, line: &str, separator1: &str, separator: &str, end: &str| {
        let mut text = String::new();
        text.push_str(start);
        text.push_str(&line.repeat(index_digits + 2));
        text.push_str(separator1);
        text.push_str(&line.repeat(counter_digits + 2));
        for _ in 1..slots {
            text.push_str(separator);
            text.push_str(&line.repeat(counter_digits + 2));
        }
        text.push_str(end);
        text.push('\n');
        text
    };
    let mut builder = divider("╔", "═", "╦", "╤", "╗");
    builder.push('║');
    builder.push_str(&" ".repeat(index_digits + 2));
    for slot in 0..slots {
        builder.push_str(if slot == 0 { "║" } else { "│" });
        builder.push_str(&format!(" {slot:>counter_digits$} "));
    }
    builder.push_str("║\n");
    for row in matrix {
        for (column, slot_bits) in row.iter().enumerate() {
            builder.push_str(&if column == 0 {
                divider("╠", "═", "╬", "╪", "╣")
            } else {
                divider("╟", "─", "╫", "┼", "╢")
            });
            builder.push_str(&format!("║ {column:>index_digits$} "));
            let mut shift = 0;
            while shift < 64 {
                let count = (slot_bits >> shift) & counter_max;
                builder.push_str(if shift == 0 { "║" } else { "│" });
                builder.push_str(&format!(" {count:>counter_digits$} "));
                shift += counter_width;
            }
            builder.push_str("║\n");
        }
    }
    builder.push_str(&divider("╚", "═", "╩", "╧", "╝"));
    builder
}

#[cfg(test)]
mod tests {
    //! `FrequencySketchSpec` of Pekko 2.0.0 (actor-tests/.../util/FrequencySketchSpec.scala), with the
    //! `Random`/`ZipfianGenerator` input replaced by a fixed pseudo-random sequence.
    use super::*;

    type Sketch = FrequencySketch<str, StringHasher>;

    fn sketch(capacity: i32) -> Sketch {
        FrequencySketch::with_capacity(capacity, StringHasher::default()).expect("valid")
    }

    fn increment_n(sketch: &mut Sketch, value: &str, times: i32) {
        for _ in 0..times {
            sketch.increment(value);
        }
    }

    #[test]
    fn murmur3_matches_the_values_measured_with_the_2_0_0_jar() {
        assert_eq!(murmur3::string_hash("", DEFAULT_SEED), -1_699_863_197);
        assert_eq!(murmur3::string_hash("a", DEFAULT_SEED), 329_321_386);
        assert_eq!(murmur3::string_hash("foo", DEFAULT_SEED), 537_187_922);
        assert_eq!(
            murmur3::string_hash("\u{1F600}", DEFAULT_SEED),
            -793_018_926
        );
        assert_eq!(murmur3::string_hash("abc", 0), -838_179_561);
        assert_eq!(murmur3::string_hash("hello", -1), -812_357_243);
        assert_eq!(StringHasher::default().hash("foo"), 537_187_922);
    }

    #[test]
    fn bits_follow_java_shifts() {
        assert_eq!(bits::ceiling_power_of_two(536_870_913), 1_073_741_824);
        assert_eq!(bits::ceiling_power_of_two(1_073_741_825), i32::MIN);
        assert_eq!(bits::ceiling_power_of_two(0), 1);
        assert_eq!(bits::ceiling_power_of_two(-1), 1);
        assert_eq!(bits::power_of_two_exponent(1), 0);
        assert_eq!(bits::power_of_two_exponent(0), 32);
        assert!(
            bits::is_power_of_two(0)
                && bits::is_power_of_two(i32::MIN)
                && !bits::is_power_of_two(6)
        );
    }

    #[test]
    fn increment_counters() {
        let mut sketch = sketch(100);
        sketch.increment("foo");
        increment_n(&mut sketch, "bar", 2);
        increment_n(&mut sketch, "baz", 3);
        assert_eq!(sketch.frequency("foo"), 1);
        assert_eq!(sketch.frequency("bar"), 2);
        assert_eq!(sketch.frequency("baz"), 3);
        assert_eq!(sketch.size(), 6); // total number of increments
    }

    #[test]
    fn increment_counters_to_max_value() {
        let mut sketch = sketch(100);
        increment_n(&mut sketch, "foo", 20);
        assert_eq!(sketch.frequency("foo"), 15); // 4 bit counters
        assert_eq!(sketch.size(), 15);
    }

    #[test]
    fn reset_counters_when_reset_size_is_reached() {
        let mut sketch = Sketch::new(100, 4, 10.0, 4, 4, StringHasher::default()).expect("valid");
        for i in 1..=500 {
            sketch.increment(&i.to_string());
        }
        for i in 1..=499 {
            sketch.increment(&i.to_string());
        }
        assert_eq!(sketch.size(), 999);
        let before: Vec<i32> = (1..=499)
            .map(|i| sketch.frequency(&i.to_string()))
            .collect();
        sketch.increment("500"); // the 1000th increment triggers the reset
        assert_eq!(sketch.size(), 500);
        let after: Vec<i32> = (1..=499)
            .map(|i| sketch.frequency(&i.to_string()))
            .collect();
        let halved = before
            .iter()
            .zip(&after)
            .filter(|(f1, f2)| **f2 == **f1 / 2)
            .count();
        assert!(halved >= 499 - 4);
        for i in 501..=999 {
            sketch.increment(&i.to_string());
        }
        assert_eq!(sketch.size(), 999);
        let before: Vec<i32> = (1..=999)
            .map(|i| sketch.frequency(&i.to_string()))
            .collect();
        sketch.increment("1000");
        assert!(sketch.size() > 300 && sketch.size() < 500);
        let after: Vec<i32> = (1..=999)
            .map(|i| sketch.frequency(&i.to_string()))
            .collect();
        let halved = before
            .iter()
            .zip(&after)
            .filter(|(f3, f4)| **f4 == **f3 / 2)
            .count();
        assert!(halved >= 999 - 4);
    }

    #[test]
    fn compare_frequencies_for_more_popular_items_with_reasonable_accuracy() {
        let mut sketch = Sketch::new(100, 4, 10.0, 4, 4, StringHasher::default()).expect("valid");
        for i in 1000..=10_000 {
            sketch.increment(&i.to_string()); // noise
        }
        for i in 1..=5 {
            increment_n(&mut sketch, &i.to_string(), (6 - i) * 2); // 1-5 are the most popular, in order
        }
        for i in 1..=5 {
            assert!(sketch.frequency(&i.to_string()) >= sketch.frequency(&(i + 1).to_string()));
        }
        for i in 6..=10 {
            assert!(sketch.frequency("5") >= sketch.frequency(&i.to_string()));
        }
    }

    /// Zipf-distributed items from a fixed generator: the most popular items keep their order and the
    /// estimates stay close to the exact counts.
    fn zipfian_accuracy(
        mut increment: impl FnMut(&str),
        mut frequency: impl FnMut(&str) -> i32,
    ) -> f64 {
        let items = 1000_u32;
        let weights: Vec<f64> = (1..=items).map(|rank| 1.0 / f64::from(rank)).collect();
        let total: f64 = weights.iter().sum();
        let mut random = crate::util::test_random::XorshiftStar::new(0x9E37_79B9_7F4A_7C15);
        let mut exact = vec![0_i32; items as usize];
        for _ in 0..2000 {
            let mut target = (random.next_u64() >> 11) as f64 / (1_u64 << 53) as f64 * total;
            let rank = weights
                .iter()
                .position(|w| {
                    target -= w;
                    target <= 0.0
                })
                .unwrap_or(items as usize - 1);
            exact[rank] += 1;
            increment(&rank.to_string());
        }
        let close = (0..items as usize)
            .filter(|rank| (frequency(&rank.to_string()) - exact[*rank].min(15)).abs() <= 2)
            .count();
        close as f64 / f64::from(items)
    }

    #[test]
    fn compare_frequencies_for_a_zipfian_distribution_with_reasonable_accuracy() {
        let sketch = std::cell::RefCell::new(
            Sketch::new(1000, 4, 10.0, 4, 4, StringHasher::default()).expect("valid"),
        );
        let accuracy = zipfian_accuracy(
            |v| sketch.borrow_mut().increment(v),
            |v| sketch.borrow().frequency(v),
        );
        assert!(accuracy > 0.95, "accuracy {accuracy}");
        assert!(sketch.borrow().size() <= 1000 / 2 * 5);
    }

    /// The common part of the counter size tests: three items, then the maximum, then the reset.
    fn counter_size(
        bits: i32,
        reset_multiplier: f64,
        max: Option<(i32, i32, i32)>,
        reset_at: i32,
        fill_to: i32,
    ) {
        let mut sketch =
            Sketch::new(100, 4, reset_multiplier, 4, bits, StringHasher::default()).expect("valid");
        sketch.increment("foo");
        increment_n(&mut sketch, "bar", 2);
        increment_n(&mut sketch, "baz", 3);
        assert_eq!(
            (
                sketch.frequency("foo"),
                sketch.frequency("bar"),
                sketch.frequency("baz")
            ),
            (1, 2, 3)
        );
        assert_eq!(sketch.size(), 6);
        let mut incremented = 6;
        if let Some((extra_increments, max_value, size_at_max)) = max {
            increment_n(&mut sketch, "foo", extra_increments);
            assert_eq!(
                sketch.frequency("foo"),
                max_value,
                "{bits} bits: the maximum value"
            );
            assert_eq!(sketch.size(), size_at_max);
            incremented = size_at_max;
        }
        for i in 1..=(fill_to - incremented) {
            sketch.increment(&i.to_string()); // up to the reset size
        }
        assert_eq!(sketch.size(), reset_at - 1);
        sketch.increment("qux"); // triggers the reset
        assert!(
            sketch.size() <= reset_at / 2,
            "{bits} bits: size {}",
            sketch.size()
        );
    }

    #[test]
    fn allow_counter_size_to_be_configured_as_2_bits() {
        counter_size(2, 1.0, Some((10, 3, 6 + 2)), 100, 99);
    }

    #[test]
    fn allow_counter_size_to_be_configured_as_8_bits() {
        counter_size(8, 10.0, Some((1000, 255, 6 + 254)), 1000, 999);
    }

    #[test]
    fn allow_counter_size_to_be_configured_as_16_bits() {
        counter_size(
            16,
            1000.0,
            Some((100_000, 65_535, 6 + 65_534)),
            100_000,
            99_999,
        );
    }

    #[test]
    fn allow_counter_size_to_be_configured_as_32_bits() {
        counter_size(32, 10.0, None, 1000, 999);
    }

    #[test]
    fn allow_counter_size_to_be_configured_as_64_bits() {
        counter_size(64, 10.0, None, 1000, 999);
    }

    #[test]
    fn allow_depth_to_be_configured() {
        // a single counter and hash function: everything collides
        let mut sketch = Sketch::new(1, 1, 10.0, 1, 64, StringHasher::default()).expect("valid");
        sketch.increment("foo");
        sketch.increment("bar");
        assert_eq!(sketch.frequency("foo"), 2);
        assert_eq!(sketch.frequency("bar"), 2);
    }

    #[test]
    fn reject_a_width_that_overflows_int_2_0_0() {
        let error = |capacity, multiplier| {
            Sketch::new(capacity, multiplier, 10.0, 4, 4, StringHasher::default())
                .err()
                .map(|e| (e.java_class(), e.message().to_owned()))
        };
        assert_eq!(
            error(536_870_913, 4),
            Some((
                "IllegalArgumentException",
                "requirement failed: computed width (4294967296) exceeds Int.MaxValue; reduce capacity or widthMultiplier".to_owned()
            ))
        );
        assert!(error(100, i32::MAX).is_some());
        assert_eq!(
            error(536_870_912, 4).map(|e| e.1),
            Some("requirement failed: computed width (2147483648) exceeds Int.MaxValue; reduce capacity or widthMultiplier".to_owned())
        );
        // the old Int product would have wrapped to a non-positive value; the Long product does not
        assert!(4_i32.wrapping_mul(bits::ceiling_power_of_two(536_870_913)) <= 0);
        assert!(FastFrequencySketch::<str>::new(536_870_913, 4, 10.0).is_err());
    }

    #[test]
    fn constructor_requirements() {
        let h = StringHasher::default;
        let message =
            |r: Result<Sketch, IllegalArgumentError>| r.err().map(|e| e.message().to_owned());
        assert_eq!(
            message(Sketch::from_parts(4, 6, 4, 10, h())),
            Some("requirement failed: width must be a power of two".to_owned())
        );
        assert_eq!(
            message(Sketch::from_parts(4, 8, 3, 10, h())),
            Some("requirement failed: counterBits must be 2, 4, 8, 16, 32, or 64 bits".to_owned())
        );
        assert!(Sketch::from_parts(0, 8, 4, 10, h()).is_ok());
        assert!(Sketch::from_parts(4, i32::MIN, 4, 10, h()).is_ok());
        assert!(FastFrequencySketch::<str>::from_parts(6, 10).is_err());
        assert!(FastFrequencySketch::<str>::from_parts(0, 10).is_ok());
    }

    #[test]
    fn the_reset_size_is_capped_and_follows_the_multiplier() {
        let mut sketch = Sketch::new(8, 4, 0.5, 4, 4, StringHasher::default()).expect("valid");
        for item in ["a", "b", "c", "d"] {
            sketch.increment(item);
        }
        assert_eq!(sketch.size(), 1); // the fourth increment reached the reset size 4 and halved the counters
        let huge = Sketch::new(8, 4, 1e30, 4, 4, StringHasher::default()).expect("valid");
        assert_eq!(huge.reset_size, i32::MAX);
        let negative = Sketch::new(8, 4, -3.0, 4, 4, StringHasher::default()).expect("valid");
        assert_eq!(negative.reset_size, -24);
    }

    #[test]
    fn fast_sketch_increment_counters_and_saturate() {
        let mut sketch = FastFrequencySketch::<str>::new(100, 4, 10.0).expect("valid");
        sketch.increment("foo");
        sketch.increment("bar");
        sketch.increment("bar");
        for _ in 0..3 {
            sketch.increment("baz");
        }
        assert_eq!(
            (
                sketch.frequency("foo"),
                sketch.frequency("bar"),
                sketch.frequency("baz"),
                sketch.size()
            ),
            (1, 2, 3, 6)
        );
        for _ in 0..20 {
            sketch.increment("qux");
        }
        assert_eq!(sketch.frequency("qux"), 15);
    }

    #[test]
    fn fast_sketch_resets_and_keeps_the_relative_frequencies() {
        let mut sketch = FastFrequencySketch::<str>::new(100, 4, 10.0).expect("valid");
        for i in 1..=500 {
            sketch.increment(&i.to_string());
        }
        for i in 1..=499 {
            sketch.increment(&i.to_string());
        }
        assert_eq!(sketch.size(), 999);
        sketch.increment("500");
        assert_eq!(sketch.size(), 500);
        let accuracy = {
            let sketch = std::cell::RefCell::new(
                FastFrequencySketch::<str>::new(1000, 4, 10.0).expect("valid"),
            );
            zipfian_accuracy(
                |v| sketch.borrow_mut().increment(v),
                |v| sketch.borrow().frequency(v),
            )
        };
        assert!(accuracy > 0.95, "accuracy {accuracy}");
    }

    #[test]
    fn fast_sketch_hashes_integers_to_themselves() {
        assert_eq!(7_i32.java_hash_code(), 7);
        assert_eq!(0x1_0000_0001_i64.java_hash_code(), 0);
        assert_eq!("abc".java_hash_code(), 96_354);
        assert_eq!("\u{1F600}".to_owned().java_hash_code(), 1_772_899);
        let mut sketch = FastFrequencySketch::<i32>::new(64, 4, 10.0).expect("valid");
        sketch.increment(&i32::MIN);
        sketch.increment(&i32::MAX);
        sketch.increment(&-1);
        assert_eq!(
            (
                sketch.frequency(&i32::MIN),
                sketch.frequency(&i32::MAX),
                sketch.frequency(&-1),
                sketch.frequency(&0)
            ),
            (1, 1, 1, 0)
        );
    }

    #[test]
    fn debug_strings_are_tables_of_counters() {
        let mut sketch = Sketch::new(4, 1, 10.0, 2, 4, StringHasher::default()).expect("valid");
        sketch.increment("a");
        let table = sketch.to_debug_string();
        assert!(table.starts_with('╔') && table.trim_end().ends_with('╝'));
        assert_eq!(table.lines().filter(|l| l.starts_with('║')).count(), 3);
        let fast = FastFrequencySketch::<str>::new(4, 4, 10.0)
            .expect("valid")
            .to_debug_string();
        assert!(fast.contains("╤") && fast.contains(" 15 "));
    }
}
