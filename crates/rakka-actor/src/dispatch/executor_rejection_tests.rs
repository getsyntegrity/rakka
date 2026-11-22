// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala (Apache-2.0).
//! `registerForExecution` when the executor rejects the mailbox (`Dispatcher.scala:146-171`): one retry, then `setAsIdle`, an `Error`
//! on the event stream and the exception thrown to the sender, who reports it too (`Dispatch.scala:129-141`).
//!
//! The executor of the user dispatcher is replaced by one that rejects the next N tasks of the actor named `w`, which is what the
//! oracle does with a custom `ExecutorServiceConfigurator` (cases `MBX-RJ-01..04` of `cases/oracle-only/mailbox.golden`, printed by the
//! Pekko 2.0.0 JVM). The same script runs here and its line is compared with the golden one.

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::dispatcher::{Executor, Rejected, ThreadPool};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, DeadLetter, ErrorEvent};
use crate::util::clock::NanoClock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");

/// Rejects the next `rejections_left` tasks submitted for an actor in `rejected_actors` and counts every submission for them.
struct RejectsNext {
    pool: ThreadPool,
    rejected_actors: Vec<&'static str>,
    rejections_left: AtomicUsize,
    submissions: AtomicUsize,
}

impl RejectsNext {
    fn new(rejected_actors: &[&'static str]) -> Arc<RejectsNext> {
        Arc::new(RejectsNext {
            pool: ThreadPool::start("rejecting", 1),
            rejected_actors: rejected_actors.to_vec(),
            rejections_left: AtomicUsize::new(0),
            submissions: AtomicUsize::new(0),
        })
    }
    fn reject_next(&self, count: usize) {
        self.rejections_left.store(count, Ordering::SeqCst);
    }
    fn rejections_left(&self) -> usize {
        self.rejections_left.load(Ordering::SeqCst)
    }
    fn submissions(&self) -> usize {
        self.submissions.load(Ordering::SeqCst)
    }
}

impl Executor for RejectsNext {
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        if self
            .rejected_actors
            .contains(&cell.self_ref().path().name())
        {
            self.submissions.fetch_add(1, Ordering::SeqCst);
            let rejected = self
                .rejections_left
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                    left.checked_sub(1)
                })
                .is_ok();
            if rejected {
                return Err(Rejected);
            }
        }
        self.pool.execute(cell)
    }
    fn shutdown(&self) {
        self.pool.shutdown();
    }
}

type Log = Arc<Mutex<Vec<String>>>;

/// Logs `r:<text>` for every string it receives.
struct Worker(Log);
impl Actor for Worker {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.0.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

/// Records every event of the channels it is subscribed to, as the oracle's listeners do.
struct Listener {
    errors: Log,
    dead: Log,
    classes: Log,
}
impl Actor for Listener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
            self.errors.lock().unwrap().push(format!(
                "{}|{}|{}",
                error.log_source, error.cause.class, error.message
            ));
            self.classes.lock().unwrap().push(error.log_class.clone());
        } else if let Some(dead) = msg.downcast_ref::<DeadLetter>() {
            let text = dead.message.downcast_ref::<String>().cloned();
            self.dead.lock().unwrap().push(text.unwrap_or_default());
        }
        true
    }
}

struct Fixture {
    system: ActorSystem,
    executor: Arc<RejectsNext>,
    errors: Log,
    dead: Log,
    classes: Log,
    log: Log,
    worker: ActorRef,
    listener: ActorRef,
}

impl Fixture {
    /// A system whose executor can reject tasks of the actors in `rejected_actors`, a listener and an idle actor `w`.
    fn new(rejected_actors: &[&'static str]) -> Fixture {
        let executor = RejectsNext::new(rejected_actors);
        let settings = Settings {
            parallelism: 1,
            ..Settings::default()
        };
        let system = ActorSystem::create_with_executor(
            "mbx",
            settings,
            Arc::new(NanoClock),
            executor.clone(),
        )
        .unwrap();
        let (errors, dead, classes, log): (Log, Log, Log, Log) = Default::default();
        let listener = system
            .actor_of_named(
                Props::new({
                    let (errors, dead, classes) = (errors.clone(), dead.clone(), classes.clone());
                    move || Listener {
                        errors: errors.clone(),
                        dead: dead.clone(),
                        classes: classes.clone(),
                    }
                }),
                "listener",
            )
            .unwrap();
        system.event_stream().subscribe(&listener, Channel::Error);
        system
            .event_stream()
            .subscribe(&listener, Channel::DeadLetter);
        let worker = system
            .actor_of_named(
                Props::new({
                    let log = log.clone();
                    move || Worker(log.clone())
                }),
                "w",
            )
            .unwrap();
        wait_until("the start-up turn to end", || {
            worker.mailbox_view().contains("scheduled=false")
        });
        Fixture {
            system,
            executor,
            errors,
            dead,
            classes,
            log,
            worker,
            listener,
        }
    }

    fn tell(&self, text: &str) {
        self.worker.tell(text.to_owned(), None);
    }
    fn view(&self) -> String {
        self.worker.mailbox_view()
    }
    fn joined(log: &Log) -> String {
        log.lock().unwrap().join(", ")
    }
    fn errors(&self) -> String {
        Self::joined(&self.errors)
    }
    fn log(&self) -> String {
        Self::joined(&self.log)
    }
    fn wait_idle(&self) {
        wait_until("an idle mailbox", || {
            self.view().contains("scheduled=false")
        });
    }
    fn wait_log(&self, count: usize) {
        wait_until("the messages to be processed", || {
            self.log.lock().unwrap().len() == count
        });
    }
    fn wait_errors(&self, count: usize) {
        wait_until("the errors to be published", || {
            self.errors.lock().unwrap().len() == count
        });
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.executor.reject_next(0);
        let _ = self.system.terminate().wait(Duration::from_secs(10));
    }
}

fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn recorded(case: &str) -> &'static str {
    ORACLE_GOLDEN
        .lines()
        .find(|line| line.starts_with(case))
        .and_then(|line| line.split_once(" -> "))
        .map(|(_, observed)| observed)
        .expect("the golden has the case")
}

#[test]
fn one_rejection_is_retried_once_and_the_message_is_delivered() {
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(1);
    fixture.tell("m");
    fixture.wait_log(1);
    fixture.wait_idle();
    let observed = format!(
        "log=[{}] errors=[{}] rejectionsLeft={} {}",
        fixture.log(),
        fixture.errors(),
        fixture.executor.rejections_left(),
        fixture.view()
    );
    assert_eq!(observed, recorded("MBX-RJ-01"));
}

#[test]
fn two_rejections_publish_the_errors_keep_the_message_and_the_next_send_delivers_it() {
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(2);
    fixture.tell("m");
    fixture.wait_errors(2);
    let after_rejection = format!(
        "errors=[{}] {} log=[{}] rejectionsLeft={}",
        fixture.errors(),
        fixture.view(),
        fixture.log(),
        fixture.executor.rejections_left()
    );
    fixture.tell("n");
    fixture.wait_log(2);
    fixture.wait_idle();
    let observed = format!(
        "{after_rejection} | next send: log=[{}] errors={} {}",
        fixture.log(),
        fixture.errors.lock().unwrap().len(),
        fixture.view()
    );
    assert_eq!(observed, recorded("MBX-RJ-02"));
}

#[test]
fn a_system_message_rejected_twice_stays_queued_and_runs_with_the_next_send() {
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(2);
    fixture.worker.send_system(SystemMessage::Suspend);
    fixture.wait_errors(2);
    let after_rejection = format!(
        "errors=[{}] {} systemPending={}",
        fixture.errors(),
        fixture.view(),
        fixture.worker.has_system_messages()
    );
    fixture.tell("n");
    wait_until("the Suspend to be processed", || {
        fixture.view().contains("suspendCount=1")
    });
    fixture.wait_idle();
    let observed = format!(
        "{after_rejection} | next send: {} systemPending={} log=[{}]",
        fixture.view(),
        fixture.worker.has_system_messages(),
        fixture.log()
    );
    assert_eq!(observed, recorded("MBX-RJ-03"));
}

#[test]
fn a_terminate_rejected_twice_terminates_with_the_next_send_and_the_message_is_dead_lettered() {
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(2);
    fixture.system.stop(&fixture.worker);
    fixture.wait_errors(2);
    let after_rejection = format!(
        "errors=[{}] terminated={} systemPending={}",
        fixture.errors(),
        fixture.worker.is_terminated(),
        fixture.worker.has_system_messages()
    );
    fixture.tell("m");
    wait_until("the actor to terminate and the dead letter", || {
        fixture.worker.is_terminated() && fixture.dead.lock().unwrap().len() == 1
    });
    let observed = format!(
        "{after_rejection} | next send: terminated={} deadLetters=[{}] log=[{}]",
        fixture.worker.is_terminated(),
        Fixture::joined(&fixture.dead),
        fixture.log()
    );
    assert_eq!(observed, recorded("MBX-RJ-04"));
}

#[test]
fn a_registration_submits_to_the_executor_at_most_twice() {
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(usize::MAX);
    let before = fixture.executor.submissions();
    fixture.tell("m");
    fixture.wait_errors(2);
    assert_eq!(
        fixture.executor.submissions() - before,
        2,
        "one try and one retry, no third"
    );
    assert!(
        fixture.view().contains("scheduled=false"),
        "the mailbox is idle again"
    );
}

#[test]
fn a_finished_turn_does_not_submit_the_mailbox_again_when_there_is_nothing_left_to_run() {
    let fixture = Fixture::new(&["w"]);
    let before = fixture.executor.submissions();
    fixture.tell("m");
    fixture.wait_log(1);
    fixture.wait_idle();
    assert_eq!(fixture.executor.submissions() - before, 1);
}

#[test]
fn a_report_made_while_reporting_does_not_recurse_and_is_not_published() {
    // The listener's own mailbox is rejected too: publishing the first Error tells it, its scheduling fails twice and would report
    // again from inside the report. `DispatchFailureReporting` sends that nested report to the standard output instead.
    let fixture = Fixture::new(&["w", "listener"]);
    fixture.executor.reject_next(usize::MAX);
    fixture.tell("m");
    let listener = fixture.listener.clone();
    wait_until("the two errors about `w` to be published", || {
        listener.mailbox_view().contains("queued=2")
    });
    std::thread::sleep(Duration::from_millis(50)); // a bounded wait for an absence: nothing else may be published
    assert!(
        listener.mailbox_view().contains("queued=2"),
        "{}",
        listener.mailbox_view()
    );
}

#[test]
fn the_swallowed_send_failure_names_the_class_of_the_actor_that_could_not_be_scheduled() {
    // `Error(e, self.path.toString, clazz(actor), ...)`: the class of the running instance; the dispatcher's own report names the
    // dispatcher. Pekko prints a JVM class name; Rust has the type path of the actor.
    let fixture = Fixture::new(&["w"]);
    fixture.executor.reject_next(2);
    fixture.tell("m");
    fixture.wait_errors(2);
    assert_eq!(
        *fixture.classes.lock().unwrap(),
        [
            "org.apache.pekko.dispatch.Dispatcher".to_owned(),
            std::any::type_name::<Worker>().to_owned()
        ]
    );
}
