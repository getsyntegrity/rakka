// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala, actor/src/main/scala/org/apache/pekko/actor/LightArrayRevolverScheduler.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! `system.scheduler`: scheduling of messages, once or with a fixed delay, on the revolving wheel driven by one thread per
//! actor system. Not here yet: `scheduleAtFixedRate`, tasks that are not messages (`Runnable`, `SchedulerTask`, `TaskRunOnClose`), the
//! `pekko.scheduler.*` settings and `shutdown-timeout`, a pluggable implementation and the recovery of the timer thread.
//!
//! Q-SCH-1 (a): the timer thread takes its time from the system's `Clock` (`NanoClock` in production) and waits through a
//! `Waiter` ([`TimerHooks`]). Both seams are internal: replacing the scheduler from outside the crate is AC-SCH-010.

use std::any::Any;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, Weak};
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
    /// Set when the send is repeated.
    repeat: Option<Arc<Repetition>>,
}

impl Delivery {
    fn new<M: Any + Send + Sync>(
        target: &ActorRef,
        message: M,
        sender: Option<&ActorRef>,
        repeat: Option<Arc<Repetition>>,
    ) -> Delivery {
        Delivery {
            target: target.clone(),
            message: Arc::new(message),
            sender: sender.cloned(),
            repeat,
        }
    }
}

/// The `AtomicCancellable` of a send with a fixed delay.
struct Repetition {
    queue: Arc<Mutex<Queue>>,
    /// `delay`, in ticks.
    ticks: i32,
    /// `None` once cancelled. Otherwise the send that waits for its tick, if there is one: a weak reference, so that what
    /// the wheel discards when the scheduler closes is dropped even while a handle is alive.
    waiting: Mutex<Option<Weak<TimerTask<Delivery>>>>,
}

impl Repetition {
    /// `swap(scheduleOnce(delay, this))`: unless it was cancelled or the scheduler closed, `delivery` waits another delay.
    fn schedule_next(&self, delivery: Delivery) {
        let task = TimerTask::new(delivery);
        let mut waiting = self.waiting.lock().unwrap();
        if waiting.is_none() {
            drop(waiting);
            drop_guarded(task);
            return;
        }
        *waiting = Some(Arc::downgrade(&task));
        drop(waiting);
        let entry = Entry {
            ticks: self.ticks,
            task,
        };
        let mut queue = self.queue.lock().unwrap();
        if queue.closed {
            drop(queue);
            drop_guarded(entry);
        } else {
            queue.new.push(entry);
        }
    }

    /// True for the first call, also when the repetition had already ended by itself.
    fn cancel(&self) -> bool {
        let waiting = self.waiting.lock().unwrap().take();
        let Some(waiting) = waiting else {
            return false;
        };
        // the message is dropped here, with no lock held
        drop_guarded(waiting.upgrade().and_then(|task| task.cancel()));
        true
    }
}

/// `ec.execute(task)`: the send happens on the default dispatcher, so a mailbox that blocks its sender does not stop the timer.
/// A dispatcher that has shut down refuses, and the message is dropped.
fn deliver(dispatcher: &Dispatcher, delivery: Delivery) {
    let send = Box::new(move || {
        let Some(repeat) = delivery.repeat.clone() else {
            return delivery.target.tell_any(delivery.message, delivery.sender);
        };
        let (message, sender) = (delivery.message.clone(), delivery.sender.clone());
        delivery.target.tell_any(message, sender);
        // `receiver.isTerminated` ends the repetition, after the send; so does a send that panics
        if delivery.target.is_terminated() {
            drop_guarded(delivery);
        } else {
            repeat.schedule_next(delivery);
        }
    });
    // a refused task is dropped inside `run`, and with it a message whose `Drop` may panic
    drop_guarded(catch_unwind(AssertUnwindSafe(|| dispatcher.run(send))));
}

/// `Cancellable` of a scheduled message. Dropping it does not cancel; clones share the task.
#[derive(Clone)]
pub struct Cancellable(Handle);

#[derive(Clone)]
enum Handle {
    /// Sent at once: `NotCancellable`.
    Sent,
    Once(Arc<TimerTask<Delivery>>),
    Repeated(Arc<Repetition>),
}

impl Cancellable {
    /// True only for the call that cancelled. For a message scheduled once: false once cancelled, once sent and for a delay
    /// of zero. For a repeated message: true for the first call, even when the repetition had already ended by itself or a
    /// send is under way (which still happens).
    pub fn cancel(&self) -> bool {
        match &self.0 {
            Handle::Sent => false,
            // the message is dropped here, with no lock held
            Handle::Once(task) => task.cancel().is_some(),
            Handle::Repeated(repetition) => repetition.cancel(),
        }
    }

    /// True only after a successful [`Cancellable::cancel`].
    pub fn is_cancelled(&self) -> bool {
        match &self.0 {
            Handle::Sent => false,
            Handle::Once(task) => task.is_cancelled(),
            Handle::Repeated(repetition) => repetition.waiting.lock().unwrap().is_none(),
        }
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
        let delivery = Delivery::new(target, message, sender, None);
        let task = self.enqueue(delay, delivery)?;
        Ok(Cancellable(task.map_or(Handle::Sent, Handle::Once)))
    }

    /// `scheduleWithFixedDelay(initialDelay, delay, receiver, message)`: `target ! message` after `initial_delay`, then again
    /// `delay` after each send, both rounded up to the tick; delays that pile up are not made up for. A zero initial delay
    /// sends at once. It ends when cancelled, when the system of this scheduler terminates, when the target is found
    /// terminated after a send (that message is a dead letter) or when a send panics.
    ///
    /// Fails with `IllegalArgumentException` for a `delay` of zero or more than `Int.MaxValue` ticks, and then as
    /// [`Scheduler::schedule_once`] does for `initial_delay`.
    pub fn schedule_with_fixed_delay<M: Any + Send + Sync>(
        &self,
        initial_delay: Duration,
        delay: Duration,
        target: &ActorRef,
        message: M,
        sender: Option<&ActorRef>,
    ) -> Result<Cancellable, ActorError> {
        // `checkPeriod`, then `checkMaxDelay`
        if delay.is_zero() {
            return Err(ActorError::new(
                ErrorClass::IllegalArgument,
                "Task scheduled with [0] seconds delay, which means creating an infinite loop. The expected delay must be greater than 0.",
            ));
        }
        let delay_nanos = rounded_nanos(delay);
        let ticks = i32::try_from(delay_nanos / TICK_NANOS).map_err(|_| too_far(delay_nanos))?;
        let repetition = Arc::new(Repetition {
            queue: self.queue.clone(),
            ticks,
            waiting: Mutex::new(Some(Weak::new())),
        });
        let delivery = Delivery::new(target, message, sender, Some(repetition.clone()));
        self.enqueue(initial_delay, delivery)?;
        Ok(Cancellable(Handle::Repeated(repetition)))
    }

    /// `schedule`: the task that waits for `delay`, or `None` when the delivery was handed to the dispatcher at once.
    /// A `delivery` that is refused is dropped after the lock of the queue is released (parameters outlive locals).
    fn enqueue(
        &self,
        delay: Duration,
        delivery: Delivery,
    ) -> Result<Option<Arc<TimerTask<Delivery>>>, ActorError> {
        let delay_nanos = rounded_nanos(delay);
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
            return Ok(None);
        }
        let Ok(ticks) = i32::try_from(delay_nanos / TICK_NANOS) else {
            return Err(too_far(delay_nanos));
        };
        let repeat = delivery.repeat.clone();
        let task = TimerTask::new(delivery);
        // before the timer thread can see the task, which would overwrite it with the next one
        if let Some(repeat) = repeat {
            *repeat.waiting.lock().unwrap() = Some(Arc::downgrade(&task));
        }
        queue.new.push(Entry {
            ticks,
            task: task.clone(),
        });
        Ok(Some(task))
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

/// `roundUp` of a delay, saturated to the `Long` nanoseconds of Pekko.
fn rounded_nanos(delay: Duration) -> i64 {
    let delay_nanos = i64::try_from(delay.as_nanos()).unwrap_or(i64::MAX);
    round_up(delay_nanos, TICK_NANOS)
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
