// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/util/TestClock.scala (Apache-2.0).
//! The `TestClock` of Pekko's specs: time moves one second per tick.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use super::clock::Clock;

/// `TestClock`: `currentTime` is the tick count; `earlierTime` subtracts whole seconds.
#[derive(Clone, Default)]
pub(crate) struct TestClock(Arc<AtomicI64>);

impl TestClock {
    pub(crate) fn tick(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn current_time(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        self.current_time() - duration_nanos / 1_000_000_000
    }
}
