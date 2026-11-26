// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala (Apache-2.0).
//! The revolving wheel of `LightArrayRevolverScheduler` for one-shot tasks, without its thread: whoever drives it passes the time
//! in, so tests move the clock by hand. The thread and the public `Scheduler` are built on top of this.

use std::sync::{Arc, Mutex};

/// The state of a `TaskHolder`: its task can be taken once, either to run it or to cancel it.
enum TaskState<T> {
    Pending(T),
    Executed,
    Cancelled,
}

/// `TaskHolder`: shared by the wheel (which runs it) and the caller (who may cancel it).
pub(crate) struct TimerTask<T>(Mutex<TaskState<T>>);

impl<T> TimerTask<T> {
    pub(crate) fn new(value: T) -> Arc<TimerTask<T>> {
        Arc::new(TimerTask(Mutex::new(TaskState::Pending(value))))
    }

    /// `extractTask`: replaces a pending state; an executed or cancelled task stays as it is.
    fn extract(&self, replacement: TaskState<T>) -> Option<T> {
        let mut state = self.0.lock().unwrap();
        if !matches!(*state, TaskState::Pending(_)) {
            return None;
        }
        match std::mem::replace(&mut *state, replacement) {
            TaskState::Pending(value) => Some(value),
            _ => None,
        }
    }

    /// `executeTask`: the value, if the task is still pending. The caller uses and drops it without any lock held.
    pub(crate) fn take_to_execute(&self) -> Option<T> {
        self.extract(TaskState::Executed)
    }

    /// `cancel`: `Some` only for the call that cancelled it.
    pub(crate) fn cancel(&self) -> Option<T> {
        self.extract(TaskState::Cancelled)
    }

    /// `isCancelled`: false for a task that ran.
    pub(crate) fn is_cancelled(&self) -> bool {
        matches!(*self.0.lock().unwrap(), TaskState::Cancelled)
    }
}

/// `roundUp`: a positive delay rounded up to a multiple of the tick; kept when that would overflow.
pub(crate) fn round_up(delay_nanos: i64, tick_nanos: i64) -> i64 {
    if delay_nanos <= 0 {
        return delay_nanos;
    }
    ((delay_nanos - 1) / tick_nanos + 1)
        .checked_mul(tick_nanos)
        .unwrap_or(delay_nanos)
}

/// A task on its way into the wheel: how many whole ticks it must wait.
pub(crate) struct Entry<T> {
    pub(crate) ticks: i32,
    pub(crate) task: Arc<TimerTask<T>>,
}

/// The state of the timer thread. One turn of that thread: if [`Wheel::sleep_nanos`] is positive, [`Wheel::place`] the new tasks
/// and sleep that long; otherwise [`Wheel::advance`] one tick. Both return the tasks that are due.
pub(crate) struct Wheel<T> {
    start: i64,
    tick_nanos: i64,
    /// Wraps around, like the `Int` of Pekko; only differences and the low bits matter.
    tick: i32,
    /// Does not wrap around: used for the sleep time.
    total_tick: i64,
    buckets: Vec<Vec<Entry<T>>>,
}

impl<T> Wheel<T> {
    /// `slots` (`ticks-per-wheel`) must be a power of two; `start` is the clock reading the ticks are counted from.
    pub(crate) fn new(start: i64, tick_nanos: i64, slots: usize, start_tick: i32) -> Wheel<T> {
        Wheel {
            start,
            tick_nanos,
            tick: start_tick,
            total_tick: i64::from(start_tick),
            buckets: (0..slots).map(|_| Vec::new()).collect(),
        }
    }

    fn bucket_of(&self, tick: i32) -> usize {
        tick as usize & (self.buckets.len() - 1)
    }

    pub(crate) fn sleep_nanos(&self, now: i64) -> i64 {
        self.start
            .wrapping_add(self.total_tick.wrapping_mul(self.tick_nanos))
            .wrapping_sub(now)
    }

    /// `checkQueue`: puts each new task in its bucket; one that waits less than a tick is due at once.
    pub(crate) fn place(&mut self, now: i64, new: Vec<Entry<T>>) -> Vec<Arc<TimerTask<T>>> {
        let mut due = Vec::new();
        for mut entry in new {
            if entry.ticks == 0 {
                due.push(entry.task);
                continue;
            }
            let since_start = now
                .wrapping_sub(self.start)
                .wrapping_add(i64::from(entry.ticks).wrapping_mul(self.tick_nanos))
                .wrapping_add(self.tick_nanos - 1);
            let future_tick = (since_start / self.tick_nanos) as i32;
            entry.ticks = future_tick.wrapping_sub(self.tick);
            let bucket = self.bucket_of(future_tick);
            self.buckets[bucket].push(entry);
        }
        due
    }

    /// `executeBucket`: the due tasks of the current bucket; those with more rounds to go stay, cancelled ones are discarded.
    pub(crate) fn advance(&mut self) -> Vec<Arc<TimerTask<T>>> {
        let bucket = self.bucket_of(self.tick);
        let rotation = self.buckets.len() as i32;
        let mut due = Vec::new();
        for mut entry in std::mem::take(&mut self.buckets[bucket]) {
            if entry.task.is_cancelled() {
                continue;
            }
            if entry.ticks >= rotation {
                entry.ticks -= rotation;
                self.buckets[bucket].push(entry);
            } else {
                due.push(entry.task);
            }
        }
        self.tick = self.tick.wrapping_add(1);
        self.total_tick = self.total_tick.wrapping_add(1);
        due
    }

    /// `clearAll`: every task still in the wheel, when the scheduler closes.
    pub(crate) fn clear(&mut self) -> Vec<Entry<T>> {
        self.buckets.iter_mut().flat_map(std::mem::take).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK: i64 = 10;

    fn entry(delay: i64, value: u32) -> Entry<u32> {
        Entry {
            ticks: (round_up(delay, TICK) / TICK) as i32,
            task: TimerTask::new(value),
        }
    }

    /// Drives the wheel like the timer thread with a clock that only moves when it sleeps; returns `(time, value)` of what ran.
    fn run(
        wheel: &mut Wheel<u32>,
        mut now: i64,
        until: i64,
        mut new: Vec<Entry<u32>>,
    ) -> Vec<(i64, u32)> {
        let mut ran = Vec::new();
        while now <= until {
            let sleep = wheel.sleep_nanos(now);
            let due = if sleep > 0 {
                wheel.place(now, std::mem::take(&mut new))
            } else {
                wheel.advance()
            };
            ran.extend(
                due.iter()
                    .filter_map(|t| t.take_to_execute())
                    .map(|v| (now, v)),
            );
            now += sleep.max(0);
        }
        ran
    }

    #[test]
    fn delays_round_up_to_the_tick_and_stay_when_rounding_would_overflow() {
        let rounded = [-5, 0, 1, 10, 11, 20].map(|delay| round_up(delay, TICK));
        assert_eq!(rounded, [-5, 0, 10, 10, 20, 20]);
        assert_eq!(round_up(i64::MAX, TICK), i64::MAX);
    }

    #[test]
    fn a_task_runs_at_the_first_tick_at_or_after_its_delay_never_before() {
        // scheduled at time 5, between the ticks 0 and 10
        for (delay, expected) in [(1, 20), (10, 20), (11, 30), (25, 40)] {
            let mut wheel = Wheel::new(0, TICK, 4, 0);
            assert!(wheel.advance().is_empty());
            let ran = run(&mut wheel, 5, 100, vec![entry(delay, 1)]);
            assert_eq!(ran, [(expected, 1)], "delay {delay}");
        }
    }

    #[test]
    fn a_task_beyond_one_rotation_waits_for_the_later_rounds() {
        let mut wheel = Wheel::new(0, TICK, 4, 0);
        let new = vec![entry(130, 1), entry(10, 2), entry(40, 3)];
        assert_eq!(run(&mut wheel, 0, 300, new), [(10, 2), (40, 3), (130, 1)]);
    }

    #[test]
    fn the_tick_counter_may_wrap_around() {
        let start_tick = i32::MAX - 2;
        let now = i64::from(start_tick) * TICK;
        let mut wheel = Wheel::new(0, TICK, 4, start_tick);
        let ran = run(&mut wheel, now, now + 200, vec![entry(100, 1)]);
        assert_eq!(ran, [(now + 100, 1)]);
    }

    #[test]
    fn a_task_shorter_than_a_tick_is_due_as_soon_as_it_is_placed() {
        let mut wheel = Wheel::new(0, TICK, 4, 0);
        let task = TimerTask::new(7);
        assert_eq!(wheel.place(0, vec![Entry { ticks: 0, task }]).len(), 1);
    }

    #[test]
    fn cancelled_tasks_are_discarded_and_clear_hands_back_the_pending_ones() {
        let mut wheel = Wheel::new(0, TICK, 4, 0);
        let (cancelled, kept) = (entry(20, 1), entry(500, 2));
        let (first, second) = (cancelled.task.clone(), kept.task.clone());
        assert!(wheel.place(0, vec![cancelled, kept]).is_empty());
        assert_eq!(first.cancel(), Some(1));
        assert!(run(&mut wheel, 0, 100, Vec::new()).is_empty());
        let left = wheel.clear();
        assert_eq!(left.len(), 1);
        assert!(Arc::ptr_eq(&left[0].task, &second));
        assert!(wheel.clear().is_empty());
    }

    #[test]
    fn only_the_first_cancel_succeeds_and_a_task_that_ran_cannot_be_cancelled() {
        let task = TimerTask::new(1);
        assert!(!task.is_cancelled());
        assert_eq!((task.cancel(), task.is_cancelled()), (Some(1), true));
        assert_eq!((task.cancel(), task.take_to_execute()), (None, None));
        assert!(task.is_cancelled());
        let ran = TimerTask::new(2);
        assert_eq!(ran.take_to_execute(), Some(2));
        assert_eq!((ran.cancel(), ran.take_to_execute()), (None, None));
        assert!(!ran.is_cancelled());
    }
}
