//! Utilities of `org.apache.pekko.util`.

pub mod bounded_blocking_queue;
pub mod byte_string;
pub mod clock;
pub mod double_linked_list;
pub mod fast_hash;
pub mod frequency_list;
pub mod frequency_sketch;
mod glob;
pub mod hash_code;
pub mod helpers;
pub mod immutable_int_map;
pub mod index;
pub mod line_numbers;
pub mod message_buffer;
mod pretty_duration;
pub mod recency_list;
mod scala_hash_order;
pub mod segmented_recency_list;
pub mod stable_priority_queue;
pub mod subclassified_index;
pub mod swar;
pub mod switch;
#[cfg(test)]
mod test_clock;
/// Deterministic pseudo-random numbers for the unit tests (xorshift64*), so no test needs a copy of the generator.
#[cfg(test)]
mod test_random {
    pub(crate) struct XorshiftStar(u64);

    impl XorshiftStar {
        pub(crate) fn new(seed: u64) -> Self {
            Self(seed)
        }

        pub(crate) fn next_u64(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        /// Uniform-ish value in `0..bound` from the high bits.
        pub(crate) fn below(&mut self, bound: u64) -> u64 {
            (self.next_u64() >> 33) % bound
        }
    }
}
mod time_unit;
mod timeout;
pub mod token_bucket;
pub mod typed_multi_map;
mod uuid_order;
mod version;
pub mod wall_clock;
pub mod wildcard_index;

pub use glob::glob_matches;
pub use pretty_duration::PekkoDuration;
pub use time_unit::TimeUnit;
pub use timeout::Timeout;
pub use uuid_order::{JavaUuid, compare_uuid, pekko_compare_uuid};
pub use version::{Version, VersionError};
