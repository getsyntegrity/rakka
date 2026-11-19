// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/TokenBucket.scala (Apache-2.0).
//! `TokenBucket` / `NanoTimeTokenBucket` (AC-UTIL-011): token-bucket throttling over a nanosecond
//! clock. Not thread safe, like Pekko's.
//!
//! The time is a `Long` of nanoseconds that is monotonic, may wrap over and has no relation to the wall
//! clock, so every operation on it wraps like the JVM's `Long` (Q-UTIL-10). It comes from a
//! [`TokenBucketClock`], which a test replaces by a manual one.

use crate::actor::IllegalArgumentError;
use crate::util::clock::{Clock, NanoClock};

/// `def currentTime: Long`: the current time in nanoseconds, monotonic and possibly wrapping.
pub trait TokenBucketClock {
    fn current_time(&self) -> i64;
}

impl<F: Fn() -> i64> TokenBucketClock for F {
    fn current_time(&self) -> i64 {
        self()
    }
}

impl TokenBucketClock for NanoClock {
    fn current_time(&self) -> i64 {
        Clock::current_time(self)
    }
}

/// `TokenBucket(capacity, nanosBetweenTokens)`.
pub struct TokenBucket<C> {
    capacity: i64,
    nanos_between_tokens: i64,
    available_tokens: i64,
    last_update: i64,
    clock: C,
}

/// `NanoTimeTokenBucket`: the bucket over `System.nanoTime`.
pub type NanoTimeTokenBucket = TokenBucket<NanoClock>;

impl NanoTimeTokenBucket {
    /// `new NanoTimeTokenBucket(capacity, period)`; call [`TokenBucket::init`] before use.
    pub fn nano_time(capacity: i64, period: i64) -> Result<Self, IllegalArgumentError> {
        Self::new(capacity, period, NanoClock)
    }
}

impl<C: TokenBucketClock> TokenBucket<C> {
    /// `new TokenBucket(capacity, nanosBetweenTokens)` over `clock`. Call [`TokenBucket::init`] before use.
    pub fn new(
        capacity: i64,
        nanos_between_tokens: i64,
        clock: C,
    ) -> Result<Self, IllegalArgumentError> {
        if capacity < 0 {
            return Err(IllegalArgumentError::new(
                "requirement failed: Capacity must be non-negative.",
            ));
        }
        if nanos_between_tokens <= 0 {
            return Err(IllegalArgumentError::new(
                "requirement failed: Time between tokens must be larger than zero nanoseconds.",
            ));
        }
        Ok(Self {
            capacity,
            nanos_between_tokens,
            available_tokens: 0,
            last_update: 0,
            clock,
        })
    }

    /// `init()`: must be called before the bucket can be used; the bucket starts full.
    pub fn init(&mut self) {
        self.available_tokens = self.capacity;
        self.last_update = self.clock.current_time();
    }

    /// `offer(cost)`: call whenever an element should pass through the bucket. Returns the nanoseconds
    /// the element must be delayed to conform with the bucket, zero if it can be emitted immediately.
    /// The caller must not call it again before the returned delay has elapsed (it does not check). The
    /// delay is not checked for overflow: past what a positive `Long` holds the value is undefined.
    /// `cost` can be larger than the capacity.
    pub fn offer(&mut self, cost: i64) -> Result<i64, IllegalArgumentError> {
        if cost < 0 {
            return Err(IllegalArgumentError::new("Cost must be non-negative"));
        }
        let period = self.nanos_between_tokens;
        let now = self.clock.current_time();
        let time_elapsed = now.wrapping_sub(self.last_update);

        let tokens_arrived = if time_elapsed >= period {
            if time_elapsed < period.wrapping_mul(2) {
                // only one tick elapsed
                self.last_update = self.last_update.wrapping_add(period);
                1
            } else {
                // no choice, the slow integer division
                let arrived = time_elapsed / period;
                self.last_update = self.last_update.wrapping_add(arrived.wrapping_mul(period));
                arrived
            }
        } else {
            0
        };

        self.available_tokens = self
            .available_tokens
            .wrapping_add(tokens_arrived)
            .min(self.capacity);

        if cost <= self.available_tokens {
            self.available_tokens -= cost;
            Ok(0)
        } else {
            let remaining_cost = cost.wrapping_sub(self.available_tokens);
            // Tokens always arrive at exact multiples of the token generation period.
            let time_since_token_arrival = now.wrapping_sub(self.last_update);
            let delay = remaining_cost
                .wrapping_mul(period)
                .wrapping_sub(time_since_token_arrival);
            self.available_tokens = 0;
            self.last_update = now.wrapping_add(delay);
            Ok(delay)
        }
    }
}

#[cfg(test)]
mod tests {
    //! `TokenBucketSpec` of Pekko 2.0.0 (actor-tests/.../util/TokenBucketSpec.scala), with a manual clock.
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct Manual(Rc<Cell<i64>>);

    impl TokenBucketClock for Manual {
        fn current_time(&self) -> i64 {
            self.0.get()
        }
    }

    /// `TestBucket`: a bucket and the handle to set its time.
    fn test_bucket(capacity: i64, period: i64) -> (TokenBucket<Manual>, Rc<Cell<i64>>) {
        let time = Rc::new(Cell::new(0));
        let bucket =
            TokenBucket::new(capacity, period, Manual(time.clone())).expect("valid bucket");
        (bucket, time)
    }

    #[test]
    fn start_full() {
        let (mut bucket, _) = test_bucket(10, 1);
        bucket.init();
        for cost in [1, 1, 1, 7] {
            assert_eq!(bucket.offer(cost), Ok(0));
        }
        assert_eq!(bucket.offer(3), Ok(3));
    }

    #[test]
    fn calculate_correctly_with_different_rates_and_capacities() {
        let (mut rate2, _) = test_bucket(10, 2);
        rate2.init();
        assert_eq!(rate2.offer(5), Ok(0));
        assert_eq!(rate2.offer(5), Ok(0));
        assert_eq!(rate2.offer(5), Ok(10));

        let (mut rate3, time) = test_bucket(8, 3);
        rate3.init();
        assert_eq!(rate3.offer(5), Ok(0));
        assert_eq!(rate3.offer(5), Ok(6));
        time.set(6);
        assert_eq!(rate3.offer(3), Ok(9));
    }

    #[test]
    fn allow_sending_elements_larger_than_capacity() {
        let (mut bucket, _) = test_bucket(10, 2);
        bucket.init();
        assert_eq!(bucket.offer(5), Ok(0));
        assert_eq!(bucket.offer(20), Ok(30));
    }

    #[test]
    fn work_if_current_time_is_negative() {
        let (mut bucket, time) = test_bucket(10, 1);
        time.set(-100); // must be set before init()
        bucket.init();
        assert_eq!(bucket.offer(5), Ok(0));
        assert_eq!(bucket.offer(10), Ok(5));
        time.set(time.get() + 10);
        assert_eq!(bucket.offer(5), Ok(0));
    }

    #[test]
    fn work_if_current_time_wraps_over() {
        let (mut bucket, time) = test_bucket(10, 1);
        time.set(i64::MAX - 5);
        bucket.init();
        assert_eq!(bucket.offer(5), Ok(0));
        assert_eq!(bucket.offer(10), Ok(5));
        time.set(time.get().wrapping_add(10));
        assert_eq!(bucket.offer(5), Ok(0));
    }

    #[test]
    fn maintain_equal_time_between_token_renewal_intervals() {
        let (mut bucket, time) = test_bucket(5, 3);
        bucket.init();
        assert_eq!(bucket.offer(10), Ok(15));
        time.set(16);
        // no token in the bucket (consumed at T15) but the next one arrives at T18: wait 2, not 3
        assert_eq!(bucket.offer(1), Ok(2));
        time.set(19);
        assert_eq!(bucket.offer(2), Ok(5));

        let (mut other, time) = test_bucket(10, 3);
        other.init();
        time.set(4);
        assert_eq!(other.offer(6), Ok(0));
        time.set(10);
        assert_eq!(other.offer(7), Ok(2));
    }

    #[test]
    fn work_with_cost_of_zero() {
        let (mut bucket, _) = test_bucket(10, 1);
        bucket.init();
        for _ in 0..3 {
            assert_eq!(bucket.offer(0), Ok(0));
        }
        assert_eq!(bucket.offer(10), Ok(0));
        for _ in 0..3 {
            assert_eq!(bucket.offer(0), Ok(0));
        }
    }

    #[test]
    fn work_with_very_slow_rates() {
        let slow = i64::MAX >> 10;
        let (mut bucket, time) = test_bucket(10, slow);
        bucket.init();
        assert_eq!(bucket.offer(20), Ok(10 * slow));
        time.set(time.get() + 10 * slow);
        time.set(time.get() + 5 * slow); // collect 5 tokens
        assert_eq!(bucket.offer(4), Ok(0));
        assert_eq!(bucket.offer(2), Ok(slow));
    }

    /// The ideal (naive) bucket as the spec writes it, with a fixed pseudo-random sequence in place of
    /// `Random`: offered with perfect timing, the bucket emits exactly when the ideal one does.
    #[test]
    fn behave_exactly_as_the_ideal_token_bucket_if_offer_is_called_with_perfect_timing() {
        let mut random = crate::util::test_random::XorshiftStar::new(0x2545_F491_4F6C_DD1D);
        let mut random_below = |bound: i64| random.below(bound as u64) as i64;
        for capacity in [0_i64, 1, 5, 10] {
            for period in [1_i64, 3, 5] {
                for arrival_period in [1_i64, 3, 5] {
                    for start_time in [i64::MIN, -1, 0, i64::MAX] {
                        for max_cost in [1_i64, 5, 10] {
                            let (mut bucket, time) = test_bucket(capacity, period);
                            time.set(start_time);
                            bucket.init();
                            let mut ideal_bucket = capacity;
                            let mut until_next_tick = period;
                            let mut until_next_element = random_below(arrival_period) + 1;
                            let mut next_emit = 0_i64;
                            let mut delaying = false;
                            for now in 0..=1000_i64 {
                                if until_next_tick == 0 {
                                    until_next_tick = period;
                                    ideal_bucket = (ideal_bucket + 1).min(capacity);
                                }
                                if delaying && ideal_bucket == 0 {
                                    assert_eq!(now, next_emit);
                                    until_next_element = now + random_below(arrival_period);
                                    delaying = false;
                                }
                                if until_next_element == 0 {
                                    let cost = random_below(max_cost + 1);
                                    ideal_bucket -= cost; // can go negative
                                    time.set(start_time.wrapping_add(now));
                                    let delay = bucket.offer(cost).expect("cost is non-negative");
                                    next_emit = now + delay;
                                    if delay == 0 {
                                        assert!(ideal_bucket >= 0);
                                        until_next_element = now + random_below(arrival_period);
                                    } else {
                                        delaying = true;
                                    }
                                }
                                until_next_tick -= 1;
                                until_next_element -= 1;
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reject_invalid_arguments_and_negative_costs() {
        let clock = || 0_i64;
        assert_eq!(
            TokenBucket::new(-1, 1, clock)
                .err()
                .map(|e| e.message().to_owned()),
            Some("requirement failed: Capacity must be non-negative.".to_owned())
        );
        assert_eq!(
            TokenBucket::new(1, 0, clock)
                .err()
                .map(|e| e.message().to_owned()),
            Some(
                "requirement failed: Time between tokens must be larger than zero nanoseconds."
                    .to_owned()
            )
        );
        let mut bucket = TokenBucket::new(3, 1, clock).expect("valid");
        bucket.init();
        let error = bucket.offer(-1).expect_err("negative cost");
        assert_eq!(
            (error.java_class(), error.message()),
            ("IllegalArgumentException", "Cost must be non-negative")
        );
    }

    #[test]
    fn the_nano_time_bucket_starts_full_on_the_real_clock() {
        let mut bucket = NanoTimeTokenBucket::nano_time(5, 1_000_000_000_000).expect("valid");
        bucket.init();
        assert_eq!(bucket.offer(5), Ok(0));
        assert!(bucket.offer(1).expect("non-negative") > 0);
    }
}
