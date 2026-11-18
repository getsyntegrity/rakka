// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Clock.scala (Apache-2.0).
//! `Clock`, `NanoClock` and `ScheduledClock` (AC-UTIL-027): the internal clock behind recency-based
//! strategies. Times are `System.nanoTime`-like values: an arbitrary origin and wrap-around, so
//! only differences mean something. The extension wiring (`Clock.get(system)`, the
//! `pekko.scheduled-clock-interval` setting and the real scheduler) waits for AC-EXT, AC-CFG and AC-SCH.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// Nanoseconds of `Duration`, saturated to the `Long` range of `FiniteDuration`.
fn saturating_nanos(duration: Duration) -> i64 {
    i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX)
}

/// `System.nanoTime()`: nanoseconds since an arbitrary, fixed origin of this process.
fn nano_time() -> i64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    saturating_nanos(ORIGIN.get_or_init(Instant::now).elapsed())
}

/// `trait Clock extends Extension`.
pub trait Clock: Send + Sync {
    /// `currentTime()`.
    fn current_time(&self) -> i64;
    /// `earlierTime(duration)`: `duration_nanos` (signed, like `FiniteDuration`) before the current time.
    fn earlier_time(&self, duration_nanos: i64) -> i64;
}

/// `NanoClock`: backed by `System.nanoTime`.
#[derive(Debug, Default)]
pub struct NanoClock;

impl Clock for NanoClock {
    fn current_time(&self) -> i64 {
        nano_time()
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        self.current_time().wrapping_sub(duration_nanos)
    }
}

/// The part of `Scheduler` that `ScheduledClock` uses: `scheduleWithFixedDelay(initialDelay, delay)(task)`.
/// AC-SCH implements it for the real scheduler.
pub trait FixedDelayScheduler {
    fn schedule_with_fixed_delay(
        &self,
        initial_delay: Duration,
        delay: Duration,
        task: Box<dyn Fn() + Send + Sync>,
    );
}

/// `ScheduledClock`: reads the time source only when the scheduler ticks. `current_time` never moves
/// backwards and increments the time by one between ticks, until `update_interval - 100 us` increments
/// have been handed out (for very high call rates it then stays put until the next tick).
pub struct ScheduledClock {
    time: AtomicI64,
    updated_time: AtomicI64,
    max_increment: i64,
    max_increment_reached: AtomicBool,
    time_source: Box<dyn Fn() -> i64 + Send + Sync>,
}

impl ScheduledClock {
    /// `new ScheduledClock(updateInterval, scheduler, ec)`: reads `System.nanoTime` and schedules `update`
    /// every `update_interval` with `scheduler`.
    pub fn new(
        update_interval: Duration,
        scheduler: &dyn FixedDelayScheduler,
    ) -> Arc<ScheduledClock> {
        Self::with_time_source(update_interval, scheduler, Box::new(nano_time))
    }

    /// Like [`ScheduledClock::new`] with an explicit time source (tests drive `nanoTime` by hand).
    pub fn with_time_source(
        update_interval: Duration,
        scheduler: &dyn FixedDelayScheduler,
        time_source: Box<dyn Fn() -> i64 + Send + Sync>,
    ) -> Arc<ScheduledClock> {
        let start = time_source();
        let clock = Arc::new(ScheduledClock {
            time: AtomicI64::new(start),
            updated_time: AtomicI64::new(start),
            max_increment: (saturating_nanos(update_interval) - 100_000).max(0),
            max_increment_reached: AtomicBool::new(false),
            time_source,
        });
        let ticking = Arc::clone(&clock);
        scheduler.schedule_with_fixed_delay(
            update_interval,
            update_interval,
            Box::new(move || ticking.update()),
        );
        clock
    }

    /// `maxIncrementReached`.
    pub fn max_increment_reached(&self) -> bool {
        self.max_increment_reached.load(Ordering::Acquire)
    }

    /// The scheduled tick (`update()`): moves to the time source's value unless that is behind (the
    /// difference test also handles the overflow of `Long.MaxValue` to negative).
    pub fn update(&self) {
        loop {
            let current = self.time.load(Ordering::Acquire);
            let now = (self.time_source)();
            let new_time = if now.wrapping_sub(current) >= 0 {
                now
            } else {
                current
            };
            if self
                .time
                .compare_exchange(current, new_time, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.updated_time.store(new_time, Ordering::Release);
                self.max_increment_reached.store(false, Ordering::Release);
                return;
            }
            // a concurrent `current_time()` incremented it: try again
        }
    }
}

impl Clock for ScheduledClock {
    fn current_time(&self) -> i64 {
        if self.max_increment_reached() {
            return self.time.load(Ordering::Acquire);
        }
        let now = self.time.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
        if now.wrapping_sub(self.updated_time.load(Ordering::Acquire)) >= self.max_increment {
            self.max_increment_reached.store(true, Ordering::Release);
        }
        now
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        self.time
            .load(Ordering::Acquire)
            .wrapping_sub(duration_nanos)
    }
}

/// What `Clock.createExtension` builds: a [`ScheduledClock`] when the interval is positive, a
/// [`NanoClock`] otherwise.
pub enum SelectedClock {
    Nano(NanoClock),
    Scheduled(Arc<ScheduledClock>),
}

impl SelectedClock {
    /// `createExtension` for `pekko.scheduled-clock-interval = interval`.
    pub fn for_interval(interval: Duration, scheduler: &dyn FixedDelayScheduler) -> SelectedClock {
        if interval > Duration::ZERO {
            SelectedClock::Scheduled(ScheduledClock::new(interval, scheduler))
        } else {
            SelectedClock::Nano(NanoClock)
        }
    }

    /// The simple class name Pekko would report (`NanoClock` or `ScheduledClock`).
    pub fn class_name(&self) -> &'static str {
        match self {
            SelectedClock::Nano(_) => "NanoClock",
            SelectedClock::Scheduled(_) => "ScheduledClock",
        }
    }
}

impl Clock for SelectedClock {
    fn current_time(&self) -> i64 {
        match self {
            SelectedClock::Nano(clock) => clock.current_time(),
            SelectedClock::Scheduled(clock) => clock.current_time(),
        }
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        match self {
            SelectedClock::Nano(clock) => clock.earlier_time(duration_nanos),
            SelectedClock::Scheduled(clock) => clock.earlier_time(duration_nanos),
        }
    }
}
