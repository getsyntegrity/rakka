// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala (Apache-2.0).
//! The one-shot `withScheduler` cases of `SchedulerSpec` (AC-SCH-008.3 A1-A5): the test moves the clock and answers every wait.
//! Substituted: the dispatcher (`localEC`, on the timer thread) and the receiver (`TestProbe`); not in the last, real one.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::scheduler::{Scheduler, TimerHooks, Waiter};
use crate::actor::system::{ActorSystem, Settings};
use crate::actor::{ActorPath, Address};
use crate::dispatch::dispatcher::{Dispatcher, Executor, Rejected, Task};
use crate::dispatch::sysmsg::SystemMessage;
use crate::dispatch::turn::TurnPolicy;
use crate::util::clock::Clock;

const WAIT: Duration = Duration::from_secs(5);
/// `step = TickDuration`.
const STEP: Duration = Duration::from_millis(10);

fn nanos(duration: Duration) -> i64 {
    i64::try_from(duration.as_nanos()).unwrap()
}

/// The `clock` and `waitNanos` overrides of `withScheduler`: a wait is reported and lasts until the test answers it with the
/// time that passed. Once the driver is gone, waits return at once.
struct ManualTime {
    now: AtomicI64,
    link: Mutex<(Sender<i64>, Receiver<i64>)>,
}

impl Clock for ManualTime {
    fn current_time(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }

    fn earlier_time(&self, duration_nanos: i64) -> i64 {
        self.current_time().wrapping_sub(duration_nanos)
    }
}

impl Waiter for ManualTime {
    fn wait_nanos(&self, nanos: i64) {
        let link = self.link.lock().unwrap(); // only the timer thread takes it
        link.0.send(nanos).ok();
        if let Ok(passed) = link.1.recv() {
            self.now.fetch_add(passed, Ordering::SeqCst); // wraps around, like the `Long` of Pekko
        }
    }
}

/// `Driver`.
struct Driver {
    wakes: Sender<i64>,
    waits: Receiver<i64>,
}

impl Driver {
    fn new(start: i64) -> (Driver, Arc<ManualTime>) {
        let (waits_tx, waits) = channel();
        let (wakes, wakes_rx) = channel();
        let time = Arc::new(ManualTime {
            now: AtomicI64::new(start),
            link: Mutex::new((waits_tx, wakes_rx)),
        });
        (Driver { wakes, waits }, time)
    }

    /// `driver.close()`: waits return at once, so the timer is never left waiting when the scheduler joins it.
    fn close(&mut self) {
        self.wakes = channel().0;
    }

    fn wake_up(&self, passed: Duration) {
        self.wakes.send(nanos(passed)).unwrap();
    }

    fn next_wait(&self) -> i64 {
        self.waits.recv_timeout(WAIT).expect("the timer waits")
    }

    fn expect_wait(&self, duration: Duration) {
        assert_eq!(self.next_wait(), nanos(duration));
    }

    fn wake_up_expecting(&self, passed: Duration, wait: Duration) {
        self.wake_up(passed);
        self.expect_wait(wait);
    }
}

/// `localEC`.
struct OnTheTimerThread;

impl Executor for OnTheTimerThread {
    fn execute(&self, _cell: Arc<crate::actor::cell::ActorCell>) -> Result<(), Rejected> {
        Err(Rejected)
    }

    fn run(&self, task: Task) -> Result<(), Rejected> {
        task();
        Ok(())
    }

    fn shutdown(&self) {}
}

/// `TestProbe`: a reference that records the texts told to it.
struct ProbeRef(ActorPath, Mutex<Sender<String>>);

impl InternalActorRef for ProbeRef {
    fn path(&self) -> &ActorPath {
        &self.0
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        let text = msg.downcast_ref::<String>().expect("a text").clone();
        self.1.lock().unwrap().send(text).ok();
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

/// `withScheduler`: a scheduler on manual time whose timer has asked for its first wait.
struct Fixture {
    scheduler: Scheduler,
    driver: Driver,
    target: ActorRef,
    received: Receiver<String>,
}

impl Fixture {
    fn new(start: i64, start_tick: i32, ticks_per_wheel: usize) -> Fixture {
        let (driver, time) = Driver::new(start);
        let clock: Arc<dyn Clock> = time.clone();
        let turns = TurnPolicy::new(1, Duration::ZERO);
        let dispatcher = Dispatcher::with_executor(Arc::new(OnTheTimerThread), turns, clock);
        let hooks = TimerHooks {
            start_tick,
            ticks_per_wheel,
            ..TimerHooks::new(time.clone(), time)
        };
        let scheduler = Scheduler::start("manual", dispatcher, hooks);
        let (tx, received) = channel();
        let path = ActorPath::root(Address::local("pekko", "manual")).new_child("probe", 0);
        let target = ActorRef(Arc::new(ProbeRef(path, Mutex::new(tx))));
        driver.next_wait();
        Fixture {
            scheduler,
            driver,
            target,
            received,
        }
    }

    fn schedule(&self, delay: Duration, text: &str) {
        let (target, text) = (&self.target, text.to_string());
        self.scheduler
            .schedule_once(delay, target, text, None)
            .unwrap();
    }

    /// Exactly what reached the probe before the wait the timer is in now.
    fn expect(&self, texts: &[&str]) {
        assert_eq!(self.received.try_iter().collect::<Vec<_>>(), texts);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.driver.close();
        self.scheduler.close();
    }
}

#[test]
fn one_wake_up_executes_the_jobs_of_several_expired_buckets() {
    let fixture = Fixture::new(0, 0, 32);
    for i in 0..4 {
        fixture.schedule(STEP / 2 + STEP * i, "hello");
    }
    fixture.expect(&[]);
    fixture.driver.wake_up_expecting(STEP, STEP);
    fixture
        .driver
        .wake_up_expecting(STEP * 4 + STEP / 2, STEP / 2);
    fixture.expect(&["hello"; 4]);
}

#[test]
fn jobs_are_properly_deferred_even_when_the_timer_thread_oversleeps() {
    let fixture = Fixture::new(0, 0, 32);
    fixture.schedule(STEP * 3, "hello");
    for passed in [STEP * 5, STEP * 2] {
        fixture.driver.wake_up_expecting(passed, STEP);
        fixture.expect(&[]);
    }
    fixture.driver.wake_up_expecting(STEP, STEP);
    fixture.expect(&["hello"]);
}

/// Four jobs half a tick apart from the ticks arrive one per wake-up.
fn one_job_per_wake_up(fixture: &Fixture) {
    for i in 0..4 {
        fixture.schedule(STEP / 2 + STEP * i, "hello");
    }
    fixture.expect(&[]);
    fixture.driver.wake_up_expecting(STEP, STEP);
    for _ in 0..4 {
        fixture.expect(&[]);
        fixture.driver.wake_up_expecting(STEP, STEP);
        fixture.expect(&["hello"]);
    }
    fixture.driver.wake_up_expecting(STEP, STEP);
    fixture.expect(&[]);
}

#[test]
fn wheel_rounds_wrap_around_correctly() {
    one_job_per_wake_up(&Fixture::new(0, 0, 2));
}

#[test]
fn jobs_are_executed_correctly_when_the_clock_wraps_around() {
    one_job_per_wake_up(&Fixture::new(i64::MAX - 200_000_000, 0, 32));
}

#[test]
fn ticks_wrap_around_correctly() {
    const EVENTS: u32 = 40;
    let fixture = Fixture::new(0, i32::MAX - 100, 32);
    let target_ticks = (i32::MAX - 40 + 20) as u32;
    fixture.driver.wake_up_expecting(STEP * target_ticks, STEP);
    for i in 0..EVENTS {
        fixture.schedule(STEP / 2 + STEP * i, &format!("hello-{i}"));
    }
    fixture.expect(&[]);
    fixture.driver.wake_up_expecting(STEP, STEP);
    for i in 0..EVENTS {
        fixture.driver.wake_up_expecting(STEP, STEP);
        fixture.expect(&[&format!("hello-{i}")]);
    }
}

struct Forward(Mutex<Sender<&'static str>>);

impl Actor for Forward {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(text) = msg.downcast_ref::<&'static str>() {
            self.0.lock().unwrap().send(text).ok();
        }
        true
    }
}

/// A system on manual time, terminated when the test ends, also by a failure.
struct ManualSystem(ActorSystem, Driver);

impl Drop for ManualSystem {
    fn drop(&mut self) {
        self.1.close();
        self.0.terminate().wait(WAIT);
    }
}

#[test]
fn the_scheduler_of_a_system_runs_on_the_clock_and_waiter_injected_into_it() {
    let (driver, time) = Driver::new(0);
    let system =
        ActorSystem::create_with_time("injected", Settings::default(), time.clone(), time).unwrap();
    let system = ManualSystem(system, driver);
    let (system, driver) = (&system.0, &system.1);
    let (tx, rx) = channel();
    let props = Props::new(move || Forward(Mutex::new(tx.clone())));
    let target = system.actor_of_named(props, "target").unwrap();
    driver.expect_wait(STEP);
    let scheduler = system.scheduler();
    scheduler
        .schedule_once(STEP * 3, &target, "hello", None)
        .unwrap();
    driver.wake_up_expecting(STEP, STEP);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    driver.wake_up_expecting(STEP * 3, STEP);
    assert_eq!(rx.recv_timeout(WAIT), Ok("hello"));
}
