// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala (Apache-2.0).
//! `registerForExecution` when the executor rejects the mailbox (`Dispatcher.scala:146-171`): one retry, then `setAsIdle`, an `Error`
//! on the event stream and the exception thrown to the sender, who reports it too (`Dispatch.scala:129-141`).
//!
//! The executor of the user dispatcher is replaced by one that rejects the next N tasks of the actor named `w`, which is what the
//! oracle does with a custom `ExecutorServiceConfigurator` (cases `MBX-RJ-01..04` of `cases/oracle-only/mailbox.golden`, printed by the
//! Pekko 2.0.0 JVM). The same script runs here and its line is compared with the golden one.

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::dispatcher::{Executor, Rejected, ThreadPool};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, DeadLetter, ErrorEvent};
use crate::util::clock::NanoClock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
                error.log_source,
                error.cause.class,
                error.message.as_deref().unwrap_or("null")
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

/// A subscriber that is not an actor: records `logSource|cause class|message` of every `ErrorEvent` it is told.
struct ErrorRecorder {
    path: crate::actor::ActorPath,
    errors: Mutex<Vec<String>>,
}
impl Default for ErrorRecorder {
    fn default() -> Self {
        ErrorRecorder {
            path: crate::actor::ActorPath::from_string("pekko://mbx/system/error-recorder")
                .unwrap(),
            errors: Mutex::default(),
        }
    }
}
impl crate::actor::actor_ref::InternalActorRef for ErrorRecorder {
    fn path(&self) -> &crate::actor::ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        // not the guardian's `logFailure` of `/user`: the oracle's listener is under `/user`, suspended by then, and never sees it
        if let Some(error) = msg
            .downcast_ref::<ErrorEvent>()
            .filter(|error| error.log_source != "pekko://mbx/user")
        {
            self.errors.lock().unwrap().push(format!(
                "{}|{}|{}",
                error.log_source,
                error.cause.class,
                error.message.as_deref().unwrap_or("null")
            ));
        }
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

/// Counts every `preStart` (the oracle's `started`).
struct CountsStarts(Arc<AtomicUsize>);
impl Actor for CountsStarts {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn counting(starts: &Arc<AtomicUsize>) -> Props {
    let starts = starts.clone();
    Props::new(move || CountsStarts(starts.clone()))
}

/// `Dispatcher.attach` rejected twice while the supervisor processes `Supervise(async = true)` (`RepointableActorRef.point`,
/// `RepointableActorRef.scala:112-135`): `cell.start()` is outside the `try` of `catchFailures`, so the rejection is not an
/// initialisation failure of the new actor but a failure of the supervisor, which restarts and stops its children.
#[test]
fn an_attach_rejected_twice_fails_the_supervisor_and_the_new_actor_never_starts() {
    let fixture = Fixture::new(&["a"]);
    let starts = Arc::new(AtomicUsize::new(0));
    let sibling = fixture
        .system
        .actor_of_named(counting(&starts), "sib")
        .unwrap();
    wait_until("the sibling to start", || {
        starts.load(Ordering::SeqCst) == 1
    });
    // The supervisor's restart stops every child of `/user`, listeners included, so the error is recorded by a subscriber that is not
    // an actor under `/user` (the oracle's listener wins that race on another thread; here one thread runs everything in order).
    let recorder = Arc::new(ErrorRecorder::default());
    fixture
        .system
        .event_stream()
        .subscribe(&ActorRef(recorder.clone()), Channel::Error);
    fixture.executor.reject_next(2);
    let rejected = fixture
        .system
        .actor_of_named(counting(&starts), "a")
        .unwrap();
    wait_until("the error and the supervisor's restart", || {
        !recorder.errors.lock().unwrap().is_empty() && sibling.is_terminated()
    });
    std::thread::sleep(Duration::from_millis(500)); // a bounded wait for an absence, as the oracle does
    let observed = format!(
        "errors=[{}] refReturned=true newActorTerminated={} siblingTerminated={} startedActors={} rejectionsLeft={}",
        recorder.errors.lock().unwrap().join(", "),
        rejected.is_terminated(),
        sibling.is_terminated(),
        starts.load(Ordering::SeqCst),
        fixture.executor.rejections_left()
    );
    assert_eq!(observed, recorded("MBX-RJ-05"));
}

struct Boom;
impl Actor for Boom {
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        crate::actor::fault::throw(crate::actor::fault::Throwable::illegal_state("boom"));
    }
}

/// `SupervisorStrategy.publish` is a plain `eventStream.publish` (`FaultHandling.scala:428-431`): when a subscriber cannot be scheduled
/// for a `logFailure` `Error`, the reports of that rejection still reach the subscribers. A Pekko 2.0.0 probe of this scenario (JDK 17,
/// `MailboxOracle$RejectingConfigurator` rejecting twice) printed these three events in this order.
#[test]
fn the_reports_made_while_a_supervisor_publishes_reach_the_subscribers() {
    let fixture = Fixture::new(&["listener"]);
    let boom = fixture.system.actor_of_named(Props::new(|| Boom), "boom");
    wait_until("the listener to be idle", || {
        fixture.listener.mailbox_view().contains("scheduled=false")
    });
    let recorder = Arc::new(ErrorRecorder::default());
    let events = fixture.system.event_stream();
    events.subscribe(&ActorRef(recorder.clone()), Channel::Error);
    fixture.executor.reject_next(2);
    boom.unwrap().tell("go", None);
    wait_until("the supervisor's error", || {
        recorder
            .errors
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.ends_with("|boom"))
    });
    let rejected = "java.util.concurrent.RejectedExecutionException";
    assert_eq!(
        *recorder.errors.lock().unwrap(),
        [
            format!(
                "org.apache.pekko.dispatch.Dispatcher|{rejected}|registerForExecution was rejected twice!"
            ),
            format!(
                "pekko://mbx/user/listener|{rejected}|swallowing exception during message send"
            ),
            "pekko://mbx/user/boom|java.lang.IllegalStateException|boom".to_owned(),
        ]
    );
}

/// The oracle's `SyncSpawner`: creates its child `c` with the synchronous `context.actorOf` (`makeChild(async = false)`) on the
/// rejecting executor. `caught`: it handles what `actorOf` throws; otherwise the exception escapes and fails this actor.
struct SyncSpawner {
    starts: Arc<AtomicUsize>,
    note: Arc<Mutex<Option<String>>>,
    caught: bool,
}
impl Actor for SyncSpawner {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.starts.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let note = match msg.downcast_ref::<&str>().copied() {
            Some("go") => {
                let created = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = ctx.actor_of_named(Props::new(|| Worker(Log::default())), "c");
                }));
                match created {
                    Ok(()) => "thrown=none".to_owned(),
                    Err(thrown) => {
                        let class = thrown
                            .downcast_ref::<crate::actor::fault::Throwable>()
                            .map_or("java.lang.RuntimeException", |t| t.class.as_str())
                            .to_owned();
                        if !self.caught {
                            std::panic::resume_unwind(thrown);
                        }
                        format!("thrown={class}")
                    }
                }
            }
            Some("again") => match ctx.actor_of_named(Props::new(|| Worker(Log::default())), "c") {
                Ok(_) => "again=ok".to_owned(),
                Err(error) => format!("again={}", error.class().java_name()),
            },
            _ => return false,
        };
        *self.note.lock().unwrap() = Some(note);
        true
    }
}

type Note = Arc<Mutex<Option<String>>>;

impl Fixture {
    /// An actor `p` that creates `c` synchronously (the oracle's `SyncSpawner`), already started.
    fn sync_spawner(&self, caught: bool) -> (ActorRef, Arc<AtomicUsize>, Note) {
        let starts = Arc::new(AtomicUsize::new(0));
        let note = Note::default();
        let spawner = self
            .system
            .actor_of_named(
                Props::new({
                    let (starts, note) = (starts.clone(), note.clone());
                    move || SyncSpawner {
                        starts: starts.clone(),
                        note: note.clone(),
                        caught,
                    }
                }),
                "p",
            )
            .unwrap();
        wait_until("the spawner to start", || {
            starts.load(Ordering::SeqCst) == 1
        });
        (spawner, starts, note)
    }
}

fn take_note(note: &Note) -> String {
    wait_until("a note", || note.lock().unwrap().is_some());
    note.lock().unwrap().take().unwrap()
}

/// `Dispatcher.attach` rejected twice under the synchronous `context.actorOf`: `actor.start()` comes after `initChild` and outside the
/// `try` that unreserves the name (`Children.scala:317-350`), so the caller gets the exception and the name stays taken.
#[test]
fn a_synchronous_actor_of_whose_attach_is_rejected_twice_throws_to_the_caller_and_keeps_the_name() {
    let fixture = Fixture::new(&["c"]);
    let (spawner, starts, note) = fixture.sync_spawner(true);
    fixture.executor.reject_next(2);
    spawner.tell("go", None);
    let thrown = take_note(&note);
    std::thread::sleep(Duration::from_millis(500)); // a bounded wait for an absence
    let errors = fixture.errors();
    spawner.tell("again", None);
    let observed = format!(
        "{thrown} errors=[{errors}] rejectionsLeft={} | {} startedActors={} parentTerminated={}",
        fixture.executor.rejections_left(),
        take_note(&note),
        starts.load(Ordering::SeqCst),
        spawner.is_terminated()
    );
    assert_eq!(observed, recorded("MBX-RJ-06"));
}

/// The same rejection escaping the actor fails it: the supervisor restarts it, which stops the child that never started.
#[test]
fn a_synchronous_actor_of_rejection_that_escapes_fails_the_parent_and_restarts_it() {
    let fixture = Fixture::new(&["c"]);
    let (spawner, starts, note) = fixture.sync_spawner(false);
    fixture.executor.reject_next(2);
    spawner.tell("go", None);
    // the parent starts, then the restarted parent starts (the child that never attached is not counted: it may or may not run its Create)
    wait_until("the restart", || starts.load(Ordering::SeqCst) == 2);
    std::thread::sleep(Duration::from_millis(500)); // a bounded wait for an absence
    *note.lock().unwrap() = None;
    spawner.tell("again", None);
    let observed = format!(
        "{} dispatcherErrors=[{}] rejectionsLeft={} startedActors={} parentTerminated={}",
        take_note(&note),
        // the supervisor's `logFailure` line is another contract (AC-SUP-018)
        fixture
            .errors
            .lock()
            .unwrap()
            .iter()
            .filter(|error| error.starts_with("org.apache.pekko.dispatch.Dispatcher|"))
            .cloned()
            .collect::<Vec<_>>()
            .join(", "),
        fixture.executor.rejections_left(),
        starts.load(Ordering::SeqCst),
        spawner.is_terminated()
    );
    assert_eq!(observed, recorded("MBX-RJ-07"));
}

/// Where the supervisor or the lifecycle publishes an `Error`; each case arms two rejections of the listener just before it.
#[derive(Clone, Copy, PartialEq)]
enum Publication {
    PostStop,
    PreRestart,
    ResumeIntoCreate,
    ResumeIntoRestart,
}

struct Hooked {
    publication: Publication,
    executor: Arc<RejectsNext>,
    thrown: Arc<AtomicBool>,
}
impl Hooked {
    fn arm_and_throw(&self, message: &str) {
        self.executor.reject_next(2);
        throw(Throwable::illegal_state(message));
    }
}
impl Actor for Hooked {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        let fails = self.publication == Publication::ResumeIntoCreate;
        if fails && !self.thrown.swap(true, Ordering::SeqCst) {
            self.arm_and_throw("init");
        }
    }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        if self.publication == Publication::PreRestart {
            self.arm_and_throw("hook");
        }
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        let fails = self.publication == Publication::ResumeIntoRestart;
        if fails && !self.thrown.swap(true, Ordering::SeqCst) {
            throw(Throwable::illegal_state("post"));
        }
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        if self.publication == Publication::PostStop {
            self.arm_and_throw("stop");
        }
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        throw(Throwable::illegal_state("boom"));
    }
}

fn hooked(fixture: &Fixture, publication: Publication) -> Props {
    let executor = fixture.executor.clone();
    let thrown = Arc::new(AtomicBool::new(false));
    Props::new(move || Hooked {
        publication,
        executor: executor.clone(),
        thrown: thrown.clone(),
    })
}

/// Supervises one `Hooked` child without `logFailure`: it resumes the two initialization failures and restarts the rest.
struct Resumer {
    child: Props,
    executor: Arc<RejectsNext>,
}
impl Actor for Resumer {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let child = ctx.actor_of_named(self.child.clone(), "child").unwrap();
        child.tell("go", None);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let executor = self.executor.clone();
        OneForOneStrategy::new(-1, move |cause| match cause.class.as_str() {
            "org.apache.pekko.actor.PostRestartException" => {
                executor.reject_next(2); // the next publication is `changing Resume into Restart`
                Directive::Resume
            }
            "org.apache.pekko.actor.ActorInitializationException" => Directive::Resume,
            _ => Directive::Restart,
        })
        .with_logging_enabled(false)
        .into()
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        false
    }
}

fn supervise(fixture: &Fixture, publication: Publication) {
    let child = hooked(fixture, publication);
    let executor = fixture.executor.clone();
    let parent = Props::new(move || Resumer {
        child: child.clone(),
        executor: executor.clone(),
    });
    fixture.system.actor_of_named(parent, "parent").unwrap();
}

/// Runs `provoke` with an idle, rejectable listener and expects the two reports of its rejection and then the `Error` being published,
/// as a Pekko 2.0.0 probe of each scenario printed them (JDK 17, `MailboxOracle$RejectingConfigurator` rejecting twice).
fn assert_rejection_reports_precede(published: &str, provoke: impl FnOnce(&Fixture)) {
    let fixture = Fixture::new(&["listener"]);
    wait_until("the listener to be idle", || {
        fixture.listener.mailbox_view().contains("scheduled=false")
    });
    let recorder = Arc::new(ErrorRecorder::default());
    let events = fixture.system.event_stream();
    events.subscribe(&ActorRef(recorder.clone()), Channel::Error);
    provoke(&fixture);
    wait_until("the published error", || {
        let errors = recorder.errors.lock().unwrap();
        errors.iter().any(|e| e.starts_with(published))
    });
    let errors = recorder.errors.lock().unwrap().clone();
    let rejected = "java.util.concurrent.RejectedExecutionException";
    let reports = [
        format!(
            "org.apache.pekko.dispatch.Dispatcher|{rejected}|registerForExecution was rejected twice!"
        ),
        format!("pekko://mbx/user/listener|{rejected}|swallowing exception during message send"),
    ];
    assert!(
        errors.len() == 3 && errors[..2] == reports && errors[2].starts_with(published),
        "{errors:#?}"
    );
}

#[test]
fn the_reports_made_while_a_post_stop_failure_is_published_reach_the_subscribers() {
    let published = "pekko://mbx/user/child|java.lang.IllegalStateException|stop";
    assert_rejection_reports_precede(published, |fixture| {
        let props = hooked(fixture, Publication::PostStop);
        let child = fixture.system.actor_of_named(props, "child").unwrap();
        fixture.system.stop(&child);
    });
}

#[test]
fn the_reports_made_while_a_pre_restart_failure_is_published_reach_the_subscribers() {
    let published = "pekko://mbx/user/parent/child|org.apache.pekko.actor.PreRestartException|hook";
    assert_rejection_reports_precede(published, |f| supervise(f, Publication::PreRestart));
}

#[test]
fn the_reports_made_while_a_resume_turned_into_create_is_published_reach_the_subscribers() {
    let published = "pekko://mbx/user/parent/child|org.apache.pekko.event.Logging$Error$NoCause$|changing Resume into Create after ";
    assert_rejection_reports_precede(published, |f| supervise(f, Publication::ResumeIntoCreate));
}

#[test]
fn the_reports_made_while_a_resume_turned_into_restart_is_published_reach_the_subscribers() {
    let published = "pekko://mbx/user/parent/child|org.apache.pekko.event.Logging$Error$NoCause$|changing Resume into Restart after ";
    assert_rejection_reports_precede(published, |f| supervise(f, Publication::ResumeIntoRestart));
}

#[test]
fn the_reports_made_while_an_unregistered_child_is_published_reach_the_subscribers() {
    let published = "pekko://mbx/user/w|org.apache.pekko.event.Logging$Error$NoCause$|received Supervise from unregistered child ";
    assert_rejection_reports_precede(published, |fixture| {
        fixture.executor.reject_next(2);
        fixture.worker.send_system(SystemMessage::Supervise {
            child: fixture.system.dead_letters(),
            is_async: false,
        });
    });
}
