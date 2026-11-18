// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/WallClock.scala (Apache-2.0).
//! `WallClock` and `AlwaysIncreasingClock` (AC-UTIL-016.16): a wall-clock time source in milliseconds that never
//! repeats and never goes backwards.
//!
//! `AlwaysIncreasingClock` keeps the last value it handed out (Pekko extends `AtomicLong`, initial value 0). A reading
//! takes the current system time `now` and atomically replaces the state `last` with `last + 1` when `now <= last`,
//! with `now` otherwise; the new state is the result. The increment wraps at `Long.MAX_VALUE` like Java.
//!
//! Differences: the time source is injectable ([`AlwaysIncreasingClock::with_time_source`]) so tests drive the system
//! time by hand; Pekko always reads `System.currentTimeMillis()`. `get`/`set` expose the `AtomicLong` state.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// `trait WallClock`: a time source.
pub trait WallClock: Send + Sync {
    /// `currentTimeMillis()`.
    fn current_time_millis(&self) -> i64;
}

/// `System.currentTimeMillis()`: milliseconds since the Unix epoch (negative before it).
fn system_time_millis() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_millis()).map_or(i64::MIN, |m| -m),
    }
}

/// `AlwaysIncreasingClock`: based on the system time but guaranteed to increase on every reading.
pub struct AlwaysIncreasingClock {
    state: AtomicI64,
    time_source: Box<dyn Fn() -> i64 + Send + Sync>,
}

impl AlwaysIncreasingClock {
    /// `new AlwaysIncreasingClock()`: state 0, reading the system time.
    #[must_use]
    pub fn new() -> Self {
        Self::with_time_source(Box::new(system_time_millis))
    }

    /// Like [`AlwaysIncreasingClock::new`] with an explicit time source in place of `System.currentTimeMillis()`.
    #[must_use]
    pub fn with_time_source(time_source: Box<dyn Fn() -> i64 + Send + Sync>) -> Self {
        Self {
            state: AtomicI64::new(0),
            time_source,
        }
    }

    /// `AtomicLong.get`: the last value handed out (0 before the first reading).
    #[must_use]
    pub fn get(&self) -> i64 {
        self.state.load(Ordering::SeqCst)
    }

    /// `AtomicLong.set`.
    pub fn set(&self, value: i64) {
        self.state.store(value, Ordering::SeqCst);
    }
}

impl Default for AlwaysIncreasingClock {
    fn default() -> Self {
        Self::new()
    }
}

impl WallClock for AlwaysIncreasingClock {
    fn current_time_millis(&self) -> i64 {
        let current_system_time = (self.time_source)();
        let mut last = self.state.load(Ordering::SeqCst);
        loop {
            let next = if current_system_time <= last {
                last.wrapping_add(1)
            } else {
                current_system_time
            };
            match self
                .state
                .compare_exchange_weak(last, next, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return next,
                Err(actual) => last = actual,
            }
        }
    }
}

/// `WallClock.AlwaysIncreasingClock`: the shared instance.
pub fn always_increasing_clock() -> &'static AlwaysIncreasingClock {
    static SHARED: OnceLock<AlwaysIncreasingClock> = OnceLock::new();
    SHARED.get_or_init(AlwaysIncreasingClock::new)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicI64;

    use super::*;

    /// A clock whose system time is a hand-driven cell.
    fn driven() -> (Arc<AtomicI64>, AlwaysIncreasingClock) {
        let now = Arc::new(AtomicI64::new(0));
        let source = Arc::clone(&now);
        let clock = AlwaysIncreasingClock::with_time_source(Box::new(move || {
            source.load(Ordering::SeqCst)
        }));
        (now, clock)
    }

    #[test]
    fn a_new_clock_starts_at_zero_and_takes_the_first_system_time() {
        let (now, clock) = driven();
        assert_eq!(clock.get(), 0);
        now.store(1_000, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 1_000);
        assert_eq!(clock.get(), 1_000);
    }

    #[test]
    fn a_stalled_system_time_adds_one_per_reading() {
        let (now, clock) = driven();
        now.store(500, Ordering::SeqCst);
        let r: Vec<i64> = (0..4).map(|_| clock.current_time_millis()).collect();
        assert_eq!(r, vec![500, 501, 502, 503]);
        // The system time catches up with the increments: still strictly increasing.
        now.store(503, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 504);
        now.store(505, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 505);
    }

    #[test]
    fn a_system_time_that_goes_backwards_never_moves_the_clock_back() {
        let (now, clock) = driven();
        now.store(10_000, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 10_000);
        now.store(9_000, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 10_001);
        now.store(-5, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 10_002);
    }

    #[test]
    fn an_equal_reading_is_not_repeated() {
        let (now, clock) = driven();
        now.store(7, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), 7);
        assert_eq!(clock.current_time_millis(), 8);
    }

    #[test]
    fn the_increment_wraps_at_the_top_of_the_long_range() {
        let (now, clock) = driven();
        clock.set(i64::MAX);
        now.store(5, Ordering::SeqCst);
        assert_eq!(clock.current_time_millis(), i64::MIN);
        assert_eq!(clock.current_time_millis(), 5);
    }

    #[test]
    fn concurrent_readings_are_distinct() {
        let (now, clock) = driven();
        now.store(100, Ordering::SeqCst);
        let all: Vec<Vec<i64>> = std::thread::scope(|scope| {
            let hs: Vec<_> = (0..4)
                .map(|_| scope.spawn(|| (0..250).map(|_| clock.current_time_millis()).collect()))
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut flat: Vec<i64> = all.iter().flatten().copied().collect();
        flat.sort_unstable();
        assert_eq!(flat, (100..1_100).collect::<Vec<i64>>());
        assert!(all.iter().all(|row| row.windows(2).all(|w| w[1] > w[0])));
    }

    #[test]
    fn the_real_clock_and_the_shared_instance_follow_the_system_time() {
        let before = system_time_millis();
        let c = AlwaysIncreasingClock::default();
        assert!(c.current_time_millis() >= before);
        let shared = always_increasing_clock();
        assert!(std::ptr::eq(shared, always_increasing_clock()));
        let a = shared.current_time_millis();
        assert!(shared.current_time_millis() > a);
        let dynamic: &dyn WallClock = shared;
        assert!(dynamic.current_time_millis() > a);
    }
}
