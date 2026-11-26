// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala and actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! The default dispatcher: an executor that runs mailboxes (by default a fixed pool of threads).
//! (Pekko's default is a fork-join pool with `parallelism-min = 8`; scheduling policy is not observable.)

use crate::actor::cell::ActorCell;
use crate::dispatch::turn::TurnPolicy;
use crate::util::clock::Clock;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

/// `java.util.concurrent.RejectedExecutionException`: the executor did not accept the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rejected;

/// The part of `ExecutorService` the dispatcher uses: run one mailbox turn, or refuse. A seam so that tests can make the executor
/// fail deterministically; the dispatcher's own pool is the only implementation outside tests.
pub(crate) trait Executor: Send + Sync {
    /// `executorService.execute(mailbox)`.
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected>;
    /// `executionContext.execute(runnable)`: a task that is not a mailbox turn. The test doubles run no such task.
    fn run(&self, _task: Task) -> Result<(), Rejected> {
        Err(Rejected)
    }
    /// Lets the workers finish what is queued and exit; later `execute` calls are rejected.
    fn shutdown(&self);
    /// Whether `shutdown` ran, for the tests that check a system cleaned up its threads.
    #[cfg(test)]
    fn is_shut_down(&self) -> bool {
        false
    }
}

pub(crate) type Task = Box<dyn FnOnce() + Send>;

enum Job {
    Mailbox(Arc<ActorCell>),
    Task(Task),
}

/// A fixed pool of threads fed by an unbounded channel. It rejects only once it has been shut down, like a Java pool does.
pub(crate) struct ThreadPool {
    tx: Mutex<Option<Sender<Job>>>,
}

impl ThreadPool {
    pub(crate) fn start(system_name: &str, parallelism: usize) -> ThreadPool {
        let (tx, rx) = channel::<Job>();
        let rx: Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(rx));
        for i in 0..parallelism.max(1) {
            let rx = rx.clone();
            thread::Builder::new()
                .name(format!("{system_name}-pekko.actor.default-dispatcher-{i}"))
                .spawn(move || {
                    loop {
                        // The lock is held only while waiting for the next job, never while running one.
                        let next = rx.lock().unwrap().recv();
                        match next {
                            Ok(Job::Mailbox(cell)) => ActorCell::run_mailbox(&cell),
                            // a task that panics (a `Drop` of the user) must not take the thread with it
                            Ok(Job::Task(task)) => drop(catch_unwind(AssertUnwindSafe(task))),
                            Err(_) => return, // sender dropped: dispatcher shut down
                        }
                    }
                })
                .expect("spawn dispatcher thread");
        }
        ThreadPool {
            tx: Mutex::new(Some(tx)),
        }
    }
}

impl ThreadPool {
    fn submit(&self, job: Job) -> Result<(), Rejected> {
        match self.tx.lock().unwrap().as_ref() {
            Some(tx) => tx.send(job).map_err(|_| Rejected),
            None => Err(Rejected),
        }
    }
}

impl Executor for ThreadPool {
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        self.submit(Job::Mailbox(cell))
    }

    fn run(&self, task: Task) -> Result<(), Rejected> {
        self.submit(Job::Task(task))
    }

    fn shutdown(&self) {
        self.tx.lock().unwrap().take();
    }

    #[cfg(test)]
    fn is_shut_down(&self) -> bool {
        self.tx.lock().unwrap().is_none()
    }
}

pub(crate) struct Dispatcher {
    executor: Arc<dyn Executor>,
    /// `throughput` and `throughput-deadline-time` of this dispatcher: what its mailboxes may do in one turn.
    turn_policy: TurnPolicy,
    /// `System.nanoTime` in Pekko; injected so that tests can move it by hand.
    clock: Arc<dyn Clock>,
}

impl Dispatcher {
    pub fn start_with(
        system_name: &str,
        parallelism: usize,
        turn_policy: TurnPolicy,
        clock: Arc<dyn Clock>,
    ) -> Arc<Dispatcher> {
        Self::with_executor(
            Arc::new(ThreadPool::start(system_name, parallelism)),
            turn_policy,
            clock,
        )
    }

    pub fn with_executor(
        executor: Arc<dyn Executor>,
        turn_policy: TurnPolicy,
        clock: Arc<dyn Clock>,
    ) -> Arc<Dispatcher> {
        Arc::new(Dispatcher {
            executor,
            turn_policy,
            clock,
        })
    }

    pub fn turn_policy(&self) -> TurnPolicy {
        self.turn_policy
    }

    pub fn clock(&self) -> &dyn Clock {
        &*self.clock
    }

    /// `executorService.execute(mbox)`.
    pub fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        self.executor.execute(cell)
    }

    /// Runs a task on the pool.
    pub fn run(&self, task: Task) -> Result<(), Rejected> {
        self.executor.run(task)
    }

    /// Lets the worker threads exit once the queued runs are done.
    pub fn shutdown(&self) {
        self.executor.shutdown();
    }

    #[cfg(test)]
    pub(crate) fn is_shut_down(&self) -> bool {
        self.executor.is_shut_down()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::clock::NanoClock;
    use std::time::Duration;

    #[test]
    fn shutdown_is_idempotent() {
        let d = Dispatcher::start_with(
            "t-dispatch",
            2,
            TurnPolicy::new(5, Duration::ZERO),
            Arc::new(NanoClock),
        );
        d.shutdown();
        d.shutdown();
    }
}

/// The turn limits with real actors and a clock the test moves by hand (twin of oracle case `MBX-SC-02`, which needs 60 ms
/// sleeps against a 30 ms deadline: here each message "takes" 60 ms of fake time and no thread sleeps).
#[cfg(test)]
mod turn_limits_with_real_actors {
    use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
    use crate::actor::system::{ActorSystem, Settings};
    use crate::dispatch::turn::test_clock::ManualClock;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    type Log = Arc<Mutex<Vec<String>>>;

    /// Occupies the only dispatcher thread until released, so everything sent meanwhile is queued behind it.
    struct Gate {
        entered: Sender<()>,
        release: Mutex<Receiver<()>>,
    }
    impl Actor for Gate {
        fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
            self.entered.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            true
        }
    }

    /// What the test sends to learn that an actor is fully installed: only a message taken from the real mailbox proves that the
    /// hand-over of everything sent before has finished.
    const READY: &str = "ready";

    /// Logs `r:<text>` and lets `each` of fake time pass while it "works".
    struct Works {
        log: Log,
        clock: Arc<ManualClock>,
        each: Duration,
        ready: Sender<()>,
    }
    impl Actor for Works {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let Some(text) = msg.downcast_ref::<String>() else {
                return false;
            };
            if text == READY {
                self.ready.send(()).unwrap();
                return true;
            }
            self.log.lock().unwrap().push(format!("r:{text}"));
            self.clock.advance(self.each);
            true
        }
    }

    fn wait_until(condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition() {
            assert!(Instant::now() < deadline, "condition not reached");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// `x1..x3` then `y1..y3` are queued on two actors behind a blocked one, on a single thread; returns the processing order.
    fn interleaving(throughput: usize, deadline: Duration, each: Duration) -> Vec<String> {
        let clock = Arc::new(ManualClock::new(0));
        let settings = Settings {
            parallelism: 1,
            throughput,
            throughput_deadline_time: deadline,
            ..Settings::default()
        };
        let system = ActorSystem::create_with_clock("turns", settings, clock.clone()).unwrap();
        let log: Log = Log::default();
        let (entered_tx, entered_rx) = channel();
        let (release_tx, release_rx) = channel();
        let gate = Mutex::new(Some((entered_tx, release_rx)));
        let blocker = system
            .actor_of(Props::new(move || {
                let (entered, release) = gate.lock().unwrap().take().expect("one instance");
                Gate {
                    entered,
                    release: Mutex::new(release),
                }
            }))
            .unwrap();
        let (ready_tx, ready_rx) = channel();
        let works = |name: &str| {
            let (log, clock, ready) = (log.clone(), clock.clone(), ready_tx.clone());
            system
                .actor_of_named(
                    Props::new(move || Works {
                        log: log.clone(),
                        clock: clock.clone(),
                        each,
                        ready: ready.clone(),
                    }),
                    name,
                )
                .unwrap()
        };
        let (x, y) = (works("x"), works("y"));
        // `actorOf` is asynchronous: the guardian attaches each actor to the dispatcher in its own turn and then hands over what was
        // sent meanwhile. Both must have processed a message from their real mailbox before the only thread is occupied, or the order
        // in which they join its queue depends on that race.
        for actor in [&x, &y] {
            actor.tell(READY.to_owned(), None);
        }
        for _ in 0..2 {
            ready_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("x and y are installed");
        }
        blocker.tell("block".to_owned(), None);
        entered_rx.recv().unwrap();
        for i in 1..=3 {
            x.tell(format!("x{i}"), None);
        }
        for i in 1..=3 {
            y.tell(format!("y{i}"), None);
        }
        release_tx.send(()).unwrap();
        wait_until(|| log.lock().unwrap().len() == 6);
        let order = log.lock().unwrap().clone();
        system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap()
            .expect("terminated normally");
        order
    }

    #[test]
    fn a_deadline_that_every_message_exceeds_alternates_the_actors_one_message_at_a_time() {
        let order = interleaving(100, Duration::from_millis(30), Duration::from_millis(60));
        assert_eq!(order, ["r:x1", "r:y1", "r:x2", "r:y2", "r:x3", "r:y3"]);
    }

    #[test]
    fn without_a_deadline_the_throughput_alone_sets_the_block_size() {
        let order = interleaving(2, Duration::ZERO, Duration::from_secs(3600));
        assert_eq!(order, ["r:x1", "r:x2", "r:y1", "r:y2", "r:x3", "r:y3"]);
    }

    #[test]
    fn a_deadline_that_no_message_reaches_leaves_the_throughput_as_the_limit() {
        let order = interleaving(2, Duration::from_secs(10), Duration::from_millis(1));
        assert_eq!(order, ["r:x1", "r:x2", "r:y1", "r:y2", "r:x3", "r:y3"]);
    }

    /// Occupies the thread like [`Gate`] and logs `r:block` once released (the oracle's `Block` message).
    struct LoggingGate {
        entered: Sender<()>,
        release: Mutex<Receiver<()>>,
        log: Log,
    }
    impl Actor for LoggingGate {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.downcast_ref::<String>().map(String::as_str) != Some("block") {
                self.log.lock().unwrap().push("r:other".to_owned());
                return true;
            }
            self.entered.send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
            self.log.lock().unwrap().push("r:block".to_owned());
            true
        }
    }

    /// Collects the text of every dead letter.
    struct DeadLetterTexts(Log);
    impl Actor for DeadLetterTexts {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let Some(dead) = msg.downcast_ref::<crate::event::DeadLetter>() else {
                return false;
            };
            let text = dead.message.downcast_ref::<String>().cloned();
            self.0.lock().unwrap().push(text.unwrap_or_default());
            true
        }
    }

    /// Twin of oracle case `MBX-SC-03` (the line printed by the Pekko 2.0.0 JVM): an actor that is suspended while its mailbox holds
    /// work ends its turn idle, does not schedule itself again, keeps the work queued, and a later stop sends that work to dead letters.
    #[test]
    fn a_suspended_actor_with_queued_work_does_not_reschedule_itself() {
        use crate::dispatch::sysmsg::SystemMessage;
        let settings = Settings {
            parallelism: 1,
            ..Settings::default()
        };
        let clock = Arc::new(ManualClock::new(0));
        let system = ActorSystem::create_with_clock("sc03", settings, clock).unwrap();
        let (log, dead): (Log, Log) = Default::default();
        let recorder = system
            .actor_of(Props::new({
                let dead = dead.clone();
                move || DeadLetterTexts(dead.clone())
            }))
            .unwrap();
        system
            .event_stream()
            .subscribe(&recorder, crate::event::Channel::DeadLetter);
        let (entered_tx, entered_rx) = channel();
        let (release_tx, release_rx) = channel();
        let gate = Mutex::new(Some((entered_tx, release_rx)));
        let actor = system
            .actor_of(Props::new({
                let log = log.clone();
                move || {
                    let (entered, release) = gate.lock().unwrap().take().expect("one instance");
                    LoggingGate {
                        entered,
                        release: Mutex::new(release),
                        log: log.clone(),
                    }
                }
            }))
            .unwrap();

        actor.tell("block".to_owned(), None);
        entered_rx.recv().unwrap();
        for text in ["u2", "u3", "u4"] {
            actor.tell(text.to_owned(), None);
        }
        actor.send_system(SystemMessage::Suspend);
        release_tx.send(()).unwrap();
        wait_until(|| actor.mailbox_view().contains("scheduled=false"));
        let suspended = format!(
            "suspendedWithWork: {} log=[{}]",
            actor.mailbox_view(),
            log.lock().unwrap().join(", ")
        );
        system.stop(&actor);
        wait_until(|| actor.is_terminated() && dead.lock().unwrap().len() == 3);
        let observed = format!(
            "{suspended} | afterStop deadLetters=[{}]",
            dead.lock().unwrap().join(", ")
        );
        system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap()
            .expect("terminated normally");

        let golden = include_str!("../../../../cases/oracle-only/mailbox.golden");
        let recorded = golden
            .lines()
            .find(|line| line.starts_with("MBX-SC-03 "))
            .and_then(|line| line.split_once(" -> "))
            .map(|(_, observed)| observed)
            .expect("the golden has MBX-SC-03");
        assert_eq!(observed, recorded);
    }
}
