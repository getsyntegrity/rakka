// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala, actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! `system.scheduler`: one-shot message scheduling on the revolving wheel, driven by one thread per actor system.
//! Not here yet: periodic scheduling, tasks that are not messages (`Runnable`, `SchedulerTask`, `TaskRunOnClose`), the
//! `pekko.scheduler.*` settings and `shutdown-timeout`, a pluggable implementation and the recovery of the timer thread.
//!
//! Q-SCH-1 (a): the timer thread takes its time from the system's `Clock` (`NanoClock` in production) and waits through a
//! `Waiter` ([`TimerHooks`]). Both seams are internal: replacing the scheduler from outside the crate is AC-SCH-010.

use std::any::Any;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::drop_guarded;
use crate::actor::errors::{ActorError, ErrorClass};
use crate::actor::props::AnyMessage;
use crate::actor::timer_wheel::{Entry, TimerTask, Wheel, round_up};
use crate::dispatch::dispatcher::Dispatcher;
use crate::util::clock::Clock;

/// `pekko.scheduler.tick-duration` (10 ms) and `ticks-per-wheel` (512), the `reference.conf` defaults.
const TICK_NANOS: i64 = 10_000_000;
const TICKS_PER_WHEEL: usize = 512;

/// `waitNanos`: how the timer thread waits for its next tick.
pub(crate) trait Waiter: Send + Sync {
    /// Returns after `nanos`, or earlier when [`Scheduler::close`] unparks the timer thread.
    fn wait_nanos(&self, nanos: i64);
}

/// The `Waiter` of production: parks the timer thread.
pub(crate) struct ParkWaiter;

impl Waiter for ParkWaiter {
    fn wait_nanos(&self, nanos: i64) {
        thread::park_timeout(Duration::from_nanos(u64::try_from(nanos).unwrap_or(0)));
    }
}

/// What `LightArrayRevolverScheduler` lets a subclass replace (`clock`, `waitNanos`, `startTick`), and `ticks-per-wheel`.
pub(crate) struct TimerHooks {
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) waiter: Arc<dyn Waiter>,
    pub(crate) start_tick: i32,
    pub(crate) ticks_per_wheel: usize,
}

impl TimerHooks {
    /// The defaults of the wheel around the given time.
    pub(crate) fn new(clock: Arc<dyn Clock>, waiter: Arc<dyn Waiter>) -> TimerHooks {
        TimerHooks {
            clock,
            waiter,
            start_tick: 0,
            ticks_per_wheel: TICKS_PER_WHEEL,
        }
    }
}

/// `receiver ! message`, still to be done.
struct Delivery {
    target: ActorRef,
    message: AnyMessage,
    sender: Option<ActorRef>,
}

/// `ec.execute(task)`: the send happens on the default dispatcher, so a mailbox that blocks its sender does not stop the timer.
/// A dispatcher that has shut down refuses, and the message is dropped.
fn deliver(dispatcher: &Dispatcher, delivery: Delivery) {
    let send = Box::new(move || delivery.target.tell_any(delivery.message, delivery.sender));
    // a refused task is dropped inside `run`, and with it a message whose `Drop` may panic
    drop_guarded(catch_unwind(AssertUnwindSafe(|| dispatcher.run(send))));
}

/// `Cancellable` of a scheduled message. Dropping it does not cancel; clones share the task.
#[derive(Clone)]
pub struct Cancellable(Option<Arc<TimerTask<Delivery>>>); // `None`: sent at once, `NotCancellable`

impl Cancellable {
    /// True only for the call that cancelled: false once cancelled, once sent and for a delay of zero.
    pub fn cancel(&self) -> bool {
        // the message is dropped here, with no lock held
        self.0.as_ref().and_then(|task| task.cancel()).is_some()
    }

    /// True only after a successful [`Cancellable::cancel`].
    pub fn is_cancelled(&self) -> bool {
        self.0.as_ref().is_some_and(|task| task.is_cancelled())
    }
}

impl fmt::Debug for Cancellable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cancellable(is_cancelled: {})", self.is_cancelled())
    }
}

#[derive(Default)]
struct Queue {
    /// Tasks the timer thread has not placed in the wheel yet.
    new: Vec<Entry<Delivery>>,
    closed: bool,
}

/// `Scheduler`: reached through `ActorSystem::scheduler`.
pub struct Scheduler {
    queue: Arc<Mutex<Queue>>,
    dispatcher: Arc<Dispatcher>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl Scheduler {
    pub(crate) fn start(
        system_name: &str,
        dispatcher: Arc<Dispatcher>,
        hooks: TimerHooks,
    ) -> Scheduler {
        let queue = Arc::new(Mutex::new(Queue::default()));
        let (timer_queue, timer_dispatcher) = (queue.clone(), dispatcher.clone());
        let thread = thread::Builder::new()
            .name(format!("{system_name}-scheduler-1"))
            .spawn(move || run_timer(&timer_queue, &timer_dispatcher, &hooks))
            .expect("spawn scheduler thread");
        Scheduler {
            queue,
            dispatcher,
            thread: Mutex::new(Some(thread)),
        }
    }

    /// `scheduleOnce(delay, receiver, message)`: `target ! message` once, after `delay` rounded up to the next tick of 10 ms,
    /// never earlier. A zero delay sends at once. Fails with `IllegalStateException` once the system has terminated, and
    /// with `IllegalArgumentException` for a delay of more than `Int.MaxValue` ticks.
    pub fn schedule_once<M: Any + Send + Sync>(
        &self,
        delay: Duration,
        target: &ActorRef,
        message: M,
        sender: Option<&ActorRef>,
    ) -> Result<Cancellable, ActorError> {
        // declared before the guard of the queue: on an error the message is dropped after the lock is released
        let delivery = Delivery {
            target: target.clone(),
            message: Arc::new(message),
            sender: sender.cloned(),
        };
        let delay_nanos = i64::try_from(delay.as_nanos()).unwrap_or(i64::MAX);
        let delay_nanos = round_up(delay_nanos, TICK_NANOS);
        let mut queue = self.queue.lock().unwrap();
        if queue.closed {
            return Err(ActorError::new(
                ErrorClass::IllegalState,
                "cannot enqueue after timer shutdown",
            ));
        }
        if delay_nanos == 0 {
            drop(queue);
            deliver(&self.dispatcher, delivery);
            return Ok(Cancellable(None));
        }
        let Ok(ticks) = i32::try_from(delay_nanos / TICK_NANOS) else {
            return Err(too_far(delay_nanos));
        };
        let task = TimerTask::new(delivery);
        queue.new.push(Entry {
            ticks,
            task: task.clone(),
        });
        Ok(Cancellable(Some(task)))
    }

    /// `close()`: stops the timer thread and discards what is pending. Called before `whenTerminated` completes.
    pub(crate) fn close(&self) {
        self.queue.lock().unwrap().closed = true;
        let thread = self.thread.lock().unwrap().take();
        if let Some(thread) = thread {
            thread.thread().unpark();
            drop_guarded(thread.join());
        }
    }
}

/// `checkMaxDelay`.
fn too_far(delay_nanos: i64) -> ActorError {
    const NANOS_PER_SECOND: i64 = 1_000_000_000;
    let seconds = delay_nanos / NANOS_PER_SECOND;
    let maximum = TICK_NANOS * i64::from(i32::MAX) / NANOS_PER_SECOND - 1;
    ActorError::new(
        ErrorClass::IllegalArgument,
        format!(
            "Task scheduled with [{seconds}] seconds delay, which is too far in future, maximum delay is [{maximum}] seconds"
        ),
    )
}

/// The timer thread (`nextTick`).
fn run_timer(queue: &Mutex<Queue>, dispatcher: &Dispatcher, hooks: &TimerHooks) {
    let clock = &*hooks.clock;
    let mut wheel = Wheel::new(
        clock.current_time(),
        TICK_NANOS,
        hooks.ticks_per_wheel,
        hooks.start_tick,
    );
    loop {
        let now = clock.current_time();
        let sleep = wheel.sleep_nanos(now);
        let due = if sleep > 0 {
            let new = std::mem::take(&mut queue.lock().unwrap().new);
            wheel.place(now, new)
        } else {
            wheel.advance()
        };
        for delivery in due.iter().filter_map(|task| task.take_to_execute()) {
            deliver(dispatcher, delivery);
        }
        if queue.lock().unwrap().closed {
            break;
        }
        if sleep > 0 {
            hooks.waiter.wait_nanos(sleep);
        }
    }
    // `clearAll`: what is pending is never sent (Pekko only runs `TaskRunOnClose`); a handle can still cancel it
    let new = std::mem::take(&mut queue.lock().unwrap().new);
    for entry in wheel.clear().into_iter().chain(new) {
        drop_guarded(entry);
    }
}
