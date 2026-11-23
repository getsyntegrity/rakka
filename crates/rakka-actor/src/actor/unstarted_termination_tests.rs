// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala, actor/dungeon/Children.scala, actor/dungeon/FaultHandling.scala and actor/ActorCell.scala (Apache-2.0).
//! Twins of oracle cases `MBX-UT-01..07` (Q-MBX-11, slice C4): swapLookup, the state of an unstarted reference, and termination and
//! restart of a parent that has a child that never started. The user dispatcher runs nothing by itself (`RunsOnDemand`), so "the child has not started" is a fact
//! about the executor, and the order of system messages sent to the parent before it creates the child is exact.

use super::async_creation_tests::{Fixture, Log, RunsOnDemand, joined, recorded};
use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::fault::Throwable;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::repointable::RepointableActorRef;
use crate::dispatch::dispatcher::{Executor, Rejected};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, DeadLetter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Logs its lifecycle as `<label>:preStart` / `<label>:postStop`.
struct Life(Log, &'static str);
impl Actor for Life {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.0.lock().unwrap().push(format!("{}:preStart", self.1));
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.0.lock().unwrap().push(format!("{}:postStop", self.1));
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

fn is_started(child: &ActorRef) -> bool {
    child
        .0
        .as_repointable()
        .expect("an asynchronously created child")
        .is_started()
}

/// Logs `c:preStart` and `c:r:<text>` for the strings it receives.
struct Echoes(Log);
impl Actor for Echoes {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.0.lock().unwrap().push("c:preStart".to_owned());
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.0.lock().unwrap().push(format!("c:r:{text}"));
        true
    }
}

/// Logs the name of the system message (or `user message`) of every dead letter it receives.
struct RecordsDeadLetters(Log);
impl Actor for RecordsDeadLetters {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(dead) = msg.downcast_ref::<DeadLetter>() else {
            return false;
        };
        let name = match dead.message.downcast_ref::<SystemMessage>() {
            Some(SystemMessage::Supervise { .. }) => "Supervise",
            Some(_) => "another system message",
            None => "user message",
        };
        self.0.lock().unwrap().push(name.to_owned());
        true
    }
}

/// Subscribes a recorder of dead letters (their kind), started, and returns its log.
fn record_dead_letters(fixture: &Fixture) -> Log {
    let log = Log::default();
    let recorder = fixture
        .system
        .actor_of_named(
            Props::new({
                let log = log.clone();
                move || RecordsDeadLetters(log.clone())
            }),
            "dead-letter-recorder",
        )
        .unwrap();
    let recorder = fixture.started("dead-letter-recorder", recorder);
    fixture
        .system
        .event_stream()
        .subscribe(&recorder, Channel::DeadLetter);
    log
}

/// What the oracle's `stateView` prints for a reference (`underlying` is the unstarted cell until the real one is installed).
fn state_view(child: &ActorRef) -> String {
    let reference = child
        .0
        .as_repointable()
        .expect("an asynchronously created child");
    let started = reference.is_started();
    format!(
        "isStarted={started} underlying={} isTerminated={} hasMessages={} queued={}",
        if started {
            "ActorCell"
        } else {
            "UnstartedCell"
        },
        child.is_terminated(),
        reference.queued_user_messages() > 0,
        reference.queued_user_messages()
    )
}

#[test]
fn a_parent_that_terminates_before_the_supervise_of_its_child_does_not_wait_for_it() {
    // twin of oracle case MBX-UT-03: Terminate enters the parent's system queue before the Supervise that attach_child sends
    let fixture = Fixture::new();
    let dead_letters = record_dead_letters(&fixture);
    let log = Log::default();
    let child = Arc::new(Mutex::new(None::<ActorRef>));
    let parent = fixture.actor("p", {
        let (log, child) = (log.clone(), child.clone());
        move |ctx| {
            ctx.self_ref().stop();
            let created = ctx
                .cell
                .attach_child(
                    Props::new({
                        let log = log.clone();
                        move || Life(log.clone(), "c")
                    }),
                    Some("c"),
                    true,
                )
                .unwrap();
            created.tell("m1".to_owned(), None);
            created.tell("m2".to_owned(), None);
            *child.lock().unwrap() = Some(created);
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    let child = child.lock().unwrap().clone().expect("created in the turn");
    // Pekko 2.0.0 (golden): parentTerminated=true | child: isStarted=false isTerminated=false queued=2
    assert!(
        parent.is_terminated(),
        "the parent must terminate without waiting for a child that never started"
    );
    assert!(!is_started(&child));
    assert!(!child.is_terminated());
    assert_eq!(child.0.as_repointable().unwrap().queued_user_messages(), 2);
    assert!(
        recorded("MBX-UT-03").contains(&format!("child: {}", state_view(&child))),
        "{}",
        state_view(&child)
    );
    assert_eq!(joined(&log), "");
    // Pekko 2.0.0 (golden): deadLetters=[Supervise] (the parent closed its mailbox before the Supervise)
    assert!(recorded("MBX-UT-03").contains("deadLetters=[Supervise]"));
    assert_eq!(joined(&dead_letters), "Supervise");
}

#[test]
fn a_restart_with_only_an_unstarted_child_does_not_wait_for_it() {
    // twin of oracle case MBX-UT-06: Suspend and Recreate enter the parent's system queue before the Supervise of its child
    let fixture = Fixture::new();
    let log = Log::default();
    let child = Arc::new(Mutex::new(None::<ActorRef>));
    let parent = fixture.system.actor_of_named(
        Props::new({
            let (log, child) = (log.clone(), child.clone());
            move || Restartable {
                log: log.clone(),
                child: child.clone(),
            }
        }),
        "p",
    );
    let parent = fixture.started("p", parent.unwrap());
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    // Pekko 2.0.0 (golden): [P:preStart, P:preRestart, P:postStop, P:preStart, c:preStart, c:postStop]
    assert_eq!(
        joined(&log),
        "P:preStart, P:preRestart, P:postStop, P:preStart, c:preStart, c:postStop"
    );
    let child = child.lock().unwrap().clone().expect("created in the turn");
    assert!(child.is_terminated());
    assert!(!parent.is_terminated());
}

/// The parent of `MBX-UT-06`: on `go` it suspends and restarts itself, then creates `c` (so `Supervise` comes after `Recreate`).
struct Restartable {
    log: Log,
    child: Arc<Mutex<Option<ActorRef>>>,
}
impl Actor for Restartable {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:preStart".to_owned());
    }
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        _reason: &Throwable,
        _message: Option<&AnyMessage>,
    ) {
        self.log.lock().unwrap().push("P:preRestart".to_owned());
        for child in ctx.children() {
            ctx.stop(&child);
        }
        self.post_stop(ctx);
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:postStop".to_owned());
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<String>().map(String::as_str) != Some("go") {
            return false;
        }
        ctx.self_ref().send_system(SystemMessage::Suspend);
        ctx.self_ref().send_system(SystemMessage::Recreate {
            cause: Throwable::new("java.lang.IllegalStateException", Some("restart")),
        });
        let log = self.log.clone();
        let created = ctx
            .cell
            .attach_child(Props::new(move || Life(log.clone(), "c")), Some("c"), true)
            .unwrap();
        created.tell("m1".to_owned(), None);
        *self.child.lock().unwrap() = Some(created);
        true
    }
}

// ---- the parent of the cases below ----

/// The parent of `MBX-UT-04..07`. `go-create` creates `c` (with "m1" queued); `go-terminate` stops itself first (so `Terminate` is
/// ahead of the `Supervise` of `c`); `go-restart` suspends and restarts itself first. The first instance may create a started child
/// `k1` in `preStart` (`parentHoldsChild`).
struct Parent {
    log: Log,
    k1_log: Log,
    child: Arc<Mutex<Option<ActorRef>>>,
    first_instance: bool,
    holds_started_child: bool,
}
impl Actor for Parent {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:preStart".to_owned());
        if self.first_instance && self.holds_started_child {
            let k1_log = self.k1_log.clone();
            ctx.cell
                .attach_child(
                    Props::new(move || Life(k1_log.clone(), "k1")),
                    Some("k1"),
                    true,
                )
                .unwrap();
        }
    }
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        _reason: &Throwable,
        _message: Option<&AnyMessage>,
    ) {
        self.log.lock().unwrap().push("P:preRestart".to_owned());
        for child in ctx.children() {
            ctx.stop(&child);
        }
        self.post_stop(ctx);
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:postStop".to_owned());
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(command) = msg.downcast_ref::<String>() else {
            return false;
        };
        match command.as_str() {
            "go-terminate" => ctx.self_ref().stop(),
            "go-restart" => {
                ctx.self_ref().send_system(SystemMessage::Suspend);
                ctx.self_ref().send_system(SystemMessage::Recreate {
                    cause: Throwable::new("java.lang.IllegalStateException", Some("restart")),
                });
            }
            "go-create" => {}
            _ => return false,
        }
        let log = self.log.clone();
        let created = ctx
            .cell
            .attach_child(Props::new(move || Life(log.clone(), "c")), Some("c"), true)
            .unwrap();
        created.tell("m1".to_owned(), None);
        *self.child.lock().unwrap() = Some(created);
        true
    }
}

/// A parent plus the logs of its family, started (its `preStart` turn has run, `k1` included).
struct Family {
    fixture: Fixture,
    parent: ActorRef,
    log: Log,
    k1_log: Log,
    child: Arc<Mutex<Option<ActorRef>>>,
    instances: Arc<AtomicUsize>,
}

impl Family {
    fn started(holds_started_child: bool) -> Family {
        Self::in_fixture(Fixture::new(), holds_started_child)
    }

    fn in_fixture(fixture: Fixture, holds_started_child: bool) -> Family {
        let (log, k1_log) = (Log::default(), Log::default());
        let child = Arc::new(Mutex::new(None::<ActorRef>));
        let instances = Arc::new(AtomicUsize::new(0));
        let parent = fixture
            .system
            .actor_of_named(
                Props::new({
                    let (log, k1_log, child, instances) = (
                        log.clone(),
                        k1_log.clone(),
                        child.clone(),
                        instances.clone(),
                    );
                    move || Parent {
                        log: log.clone(),
                        k1_log: k1_log.clone(),
                        child: child.clone(),
                        first_instance: instances.fetch_add(1, Ordering::SeqCst) == 0,
                        holds_started_child,
                    }
                }),
                "p",
            )
            .unwrap();
        let parent = fixture.started("p", parent);
        Family {
            fixture,
            parent,
            log,
            k1_log,
            child,
            instances,
        }
    }

    fn child(&self) -> ActorRef {
        self.child
            .lock()
            .unwrap()
            .clone()
            .expect("created in the turn")
    }
}

#[test]
fn a_parent_stopped_after_its_child_started_waits_for_it_and_the_child_dies_first() {
    // twin of oracle case MBX-UT-04 (control: Children.stop does mark a child that has started)
    let family = Family::started(false);
    family.parent.tell("go-create".to_owned(), None);
    family.fixture.executor.run_until_idle();
    let child = family.child();
    assert!(is_started(&child), "the parent's Supervise started it");
    family.parent.stop();
    family.fixture.executor.run_until_idle();
    assert!(family.parent.is_terminated() && child.is_terminated());
    // Pekko 2.0.0 (golden): log=[P:preStart, c:preStart, c:r:m1, c:postStop, P:postStop]; the twin's `c` does not log its messages
    assert!(
        recorded("MBX-UT-04")
            .contains("log=[P:preStart, c:preStart, c:r:m1, c:postStop, P:postStop]")
    );
    assert_eq!(
        joined(&family.log),
        "P:preStart, c:preStart, c:postStop, P:postStop"
    );
}

#[test]
fn a_parent_already_waiting_for_a_started_child_ignores_the_supervise_of_a_new_child_and_still_terminates()
 {
    // twin of oracle case MBX-UT-05. The parent's own turn (Terminate, then the Supervise of `c`) runs alone: k1's Terminate stays pending,
    // so the parent is Terminating, waiting for k1, when the Supervise arrives.
    // Kept exactly as Pekko 2.0.0 does it (PROP-C4-1 is the Owner's): `c` is never pointed and nothing reports it.
    let family = Family::started(true);
    let dead_letters = record_dead_letters(&family.fixture);
    family.parent.tell("go-terminate".to_owned(), None);
    family.fixture.executor.run_next();
    let child = family.child();
    assert!(!family.parent.is_terminated(), "waiting for k1");
    assert_eq!(joined(&family.log), "P:preStart");
    let while_waiting = state_view(&child);
    assert!(
        recorded("MBX-UT-05").contains(&format!("whileWaiting: parentTerminated=false log=[P:preStart, k1:preStart, k1:postStop] deadLetters=[] | child: {while_waiting}")),
        "{while_waiting}"
    );
    family.fixture.executor.run_until_idle(); // k1 dies
    assert!(family.parent.is_terminated());
    assert_eq!(joined(&family.k1_log), "k1:preStart, k1:postStop");
    assert_eq!(joined(&family.log), "P:preStart, P:postStop");
    assert_eq!(
        joined(&dead_letters),
        "",
        "the Supervise is ignored without a dead letter"
    );
    let after_release = state_view(&child);
    assert!(
        recorded("MBX-UT-05").contains(&format!("afterRelease: parentTerminated=true log=[P:preStart, k1:preStart, k1:postStop, k1:postStop-released, P:postStop] deadLetters=[] | child: {after_release}")),
        "{after_release}"
    );
}

#[test]
fn a_restart_waiting_for_a_started_child_still_processes_the_supervise_of_an_unstarted_one() {
    // twin of oracle case MBX-UT-07: only the parent's and `c`'s turns run first; k1's Terminate (from preRestart) stays pending
    let family = Family::started(true);
    family.parent.tell("go-restart".to_owned(), None);
    family.fixture.executor.run_next_of("p");
    family.fixture.executor.run_next_of("c");
    assert_eq!(
        joined(&family.log),
        "P:preStart, P:preRestart, P:postStop, c:preStart, c:postStop"
    );
    assert_eq!(family.instances.load(Ordering::SeqCst), 1);
    assert!(
        recorded("MBX-UT-07").contains(
            "whileWaiting: parentAndChild=[P:preStart, P:preRestart, P:postStop, c:preStart, c:postStop]"
        ) && recorded("MBX-UT-07").contains("parentInstances=1")
    );
    family.fixture.executor.run_until_idle(); // k1 dies: the restart completes
    assert_eq!(
        joined(&family.log),
        "P:preStart, P:preRestart, P:postStop, c:preStart, c:postStop, P:preStart"
    );
    assert_eq!(family.instances.load(Ordering::SeqCst), 2);
    assert!(family.child().is_terminated());
    assert!(!family.parent.is_terminated());
    assert!(
        recorded("MBX-UT-07").contains("afterRelease: parentAndChild=[P:preStart, P:preRestart, P:postStop, c:preStart, c:postStop, P:preStart]")
            && recorded("MBX-UT-07").contains("restartAfterK1Gone=true parentTerminated=false")
    );
}

// ---- swapLookup and the state of an unstarted reference ----

#[test]
fn an_unstarted_reference_has_no_children_counts_its_user_messages_and_hands_everything_over_in_order()
 {
    // twin of oracle case MBX-UT-02 (inside the parent's turn: `c` has not been pointed yet)
    let fixture = Fixture::new();
    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let log = Log::default();
    let parent = fixture.actor("p", {
        let (observed, log) = (observed.clone(), log.clone());
        move |ctx| {
            let log_of_child = log.clone();
            let created = ctx
                .cell
                .attach_child(
                    Props::new(move || Echoes(log_of_child.clone())),
                    Some("c"),
                    true,
                )
                .unwrap();
            created.tell("m1".to_owned(), None);
            created.tell("m2".to_owned(), None);
            let reference = created.0.as_repointable().unwrap();
            let mut seen = observed.lock().unwrap();
            seen.push(format!(
                "{} children={} singleChild(k)={}",
                state_view(&created),
                reference.children().len(),
                reference.get_single_child("k").is_none()
            ));
            created.send_system(SystemMessage::Suspend);
            created.send_system(SystemMessage::Resume { cause: None });
            seen.push(state_view(&created));
            created.tell("m3".to_owned(), None);
            seen.push(state_view(&created));
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    let observed = observed.lock().unwrap().clone();
    let golden = recorded("MBX-UT-02");
    // the oracle prints the lookup view (`children=[]`, `Nobody`) right after the state of the unstarted reference
    assert!(
        golden.contains(&format!(
            "unstarted: {} lookup: isStarted=false underlying=UnstartedCell children=[] singleChild(k)=pekko://all-systems/Nobody",
            observed[0].replace(" children=0 singleChild(k)=true", "")
        )),
        "{}",
        observed[0]
    );
    assert!(observed[0].ends_with("children=0 singleChild(k)=true"));
    assert!(
        golden.contains(&format!("suspend+resume queued: {}", observed[1])),
        "{}",
        observed[1]
    );
    assert!(
        golden.contains(&format!("m3: {}", observed[2])),
        "{}",
        observed[2]
    );
    assert!(
        golden.contains("started=true")
            && golden.contains("log=[P:preStart, c:preStart, c:r:m1, c:r:m2, c:r:m3]")
    );
    assert_eq!(joined(&log), "c:preStart, c:r:m1, c:r:m2, c:r:m3");
}

/// An executor that, when the turn of `held` is submitted, accepts it and then holds the submitting thread until released: the
/// equivalent of the oracle's `rej-disp`, so that the real cell runs while `point` has not reached `replaceWith`.
struct HoldsTheSubmitter {
    inner: Arc<RunsOnDemand>,
    held: &'static str,
    already_held: std::sync::atomic::AtomicBool,
    submitted: Mutex<Sender<()>>,
    release: Mutex<Receiver<()>>,
}
impl Executor for HoldsTheSubmitter {
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        let hold = cell.self_ref().path().name() == self.held
            && !self.already_held.swap(true, Ordering::SeqCst);
        self.inner.execute(cell)?;
        if hold {
            self.submitted.lock().unwrap().send(()).unwrap();
            let _ = self.release.lock().unwrap().recv(); // a closed channel releases too
        }
        Ok(())
    }
    fn shutdown(&self) {}
}

/// What the oracle's `lookupView` prints through the reference: its state and what the lookup cell knows (`Nobody` for no child).
fn lookup_view(reference: &RepointableActorRef) -> String {
    let started = reference.is_started();
    format!(
        "isStarted={started} underlying={} children=[{}] singleChild(k)={}",
        if started {
            "ActorCell"
        } else {
            "UnstartedCell"
        },
        reference
            .children()
            .iter()
            .map(|child| child.path().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        reference.get_single_child("k").map_or_else(
            || "pekko://all-systems/Nobody".to_owned(),
            |k| k.path().to_string()
        ),
    )
}

/// Lets the held thread go when the test ends, however it ends (the fixture waits for the system to terminate when it is dropped).
struct ReleaseOnDrop(Sender<()>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

/// Creates `k` in `preStart` and logs it.
struct MakesChildInPreStart(Log);
impl Actor for MakesChildInPreStart {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.0.lock().unwrap().push("c:preStart".to_owned());
        ctx.cell
            .attach_child(
                Props::new(|| super::async_creation_tests::Idle),
                Some("k"),
                true,
            )
            .unwrap();
        self.0.lock().unwrap().push("c:made-k".to_owned());
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(text) = msg.downcast_ref::<String>() {
            self.0.lock().unwrap().push(format!("c:r:{text}"));
        }
        true
    }
}

#[test]
fn a_child_created_in_pre_start_is_found_through_a_reference_that_is_still_unstarted() {
    // twin of oracle case MBX-UT-01 (swapLookup): the thread that points `c` is held after `cell.start()` was accepted by the executor
    let (submitted_tx, submitted) = channel();
    let (release_tx, release) = channel();
    let fixture = Fixture::with_executor("mbx", |inner| {
        Arc::new(HoldsTheSubmitter {
            inner,
            held: "c",
            already_held: false.into(),
            submitted: Mutex::new(submitted_tx),
            release: Mutex::new(release),
        })
    });
    let release_tx = ReleaseOnDrop(release_tx);
    let log = Log::default();
    let child = Arc::new(Mutex::new(None::<ActorRef>));
    let parent = fixture.actor("p", {
        let (log, child) = (log.clone(), child.clone());
        move |ctx| {
            let log = log.clone();
            let created = ctx
                .cell
                .attach_child(
                    Props::new(move || MakesChildInPreStart(log.clone())),
                    Some("c"),
                    true,
                )
                .unwrap();
            created.tell("m1".to_owned(), None);
            *child.lock().unwrap() = Some(created);
        }
    });
    parent.tell("go".to_owned(), None);
    let pointing_thread = {
        let executor = fixture.executor.clone();
        std::thread::spawn(move || executor.run_next_of("p"))
    };
    submitted
        .recv_timeout(Duration::from_secs(5))
        .expect("`c` was submitted");
    fixture.executor.run_next_of("c"); // the real cell starts: preStart creates `k`
    let child = child.lock().unwrap().clone().unwrap();
    let reference = child.0.as_repointable().unwrap();
    let during = lookup_view(reference);
    assert_eq!(joined(&log), "c:preStart, c:made-k");
    release_tx.0.send(()).unwrap();
    pointing_thread.join().unwrap();
    fixture.executor.run_until_idle();
    let after = lookup_view(reference);
    // Pekko 2.0.0 (golden): the same lookup through the real cell before and after `replaceWith`
    let golden = recorded("MBX-UT-01");
    assert!(
        golden.contains(&format!("beforeReplaceWith: {during} viaPath(k)=")),
        "{during}"
    );
    assert!(
        golden.contains(&format!("afterReplaceWith: {after} viaPath(k)=")),
        "{after}"
    );
    assert_eq!(joined(&log), "c:preStart, c:made-k, c:r:m1");
}
