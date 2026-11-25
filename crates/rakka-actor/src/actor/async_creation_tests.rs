// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala, actor/dungeon/Children.scala and actor/ActorCell.scala (Apache-2.0).
//! Asynchronous creation of children (Q-MBX-11, slice B): `actorOf` hands out a `RepointableActorRef` and the parent starts the child while it
//! processes `Supervise(child, async = true)`, in its own turn.
//!
//! The user dispatcher's executor is replaced by one that runs nothing by itself: the test decides when a mailbox turn happens, so
//! "the child has not been started yet" is a fact about the executor (nothing was submitted for it), not about timing.
//! Twins of oracle cases `MBX-UC-01..04` (lines of `cases/oracle-only/mailbox.golden` printed by the Pekko 2.0.0 JVM) run through the
//! real `actorOf`/`attach_child` path here.

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::dispatcher::{Executor, Rejected};
use crate::dispatch::sysmsg::{SystemMessage, Terminated};
use crate::event::{Channel, ErrorEvent};
use crate::util::clock::NanoClock;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");

/// An executor that only records which actors were submitted and runs their turns when the test says so.
#[derive(Default)]
pub(super) struct RunsOnDemand {
    pending: Mutex<VecDeque<Arc<ActorCell>>>,
    submissions: Mutex<HashMap<String, usize>>,
}

impl RunsOnDemand {
    pub(super) fn submissions(&self, actor: &str) -> usize {
        self.submissions
            .lock()
            .unwrap()
            .get(actor)
            .copied()
            .unwrap_or(0)
    }

    /// Runs the oldest pending mailbox turn, if any.
    fn run_next(&self) {
        let next = self.pending.lock().unwrap().pop_front();
        if let Some(cell) = next {
            ActorCell::run_mailbox(&cell);
        }
    }

    /// Runs the oldest pending mailbox turn of the actor called `name`, if any (the other pending turns stay pending).
    pub(super) fn run_next_of(&self, name: &str) {
        let next = {
            let mut pending = self.pending.lock().unwrap();
            let position = pending
                .iter()
                .position(|cell| cell.self_ref().path().name() == name);
            position.and_then(|position| pending.remove(position))
        };
        if let Some(cell) = next {
            ActorCell::run_mailbox(&cell);
        }
    }

    /// Runs every pending mailbox turn, and the ones those turns schedule, on the calling thread.
    pub(super) fn run_until_idle(&self) {
        loop {
            let next = self.pending.lock().unwrap().pop_front();
            match next {
                Some(cell) => ActorCell::run_mailbox(&cell),
                None => return,
            }
        }
    }
}

impl Executor for RunsOnDemand {
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        let name = cell.self_ref().path().name().to_owned();
        *self.submissions.lock().unwrap().entry(name).or_default() += 1;
        self.pending.lock().unwrap().push_back(cell);
        Ok(())
    }
    fn shutdown(&self) {}
}

pub(super) type Log = Arc<Mutex<Vec<String>>>;

/// Runs `script` inside its own turn when it receives `"go"`, and logs `terminated` for every `Terminated` it is sent.
struct Script(Arc<dyn Fn(&mut ActorContext) + Send + Sync>, Log);
impl Actor for Script {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<String>().map(String::as_str) == Some("go") {
            (self.0)(ctx);
            true
        } else if msg.is::<Terminated>() {
            self.1.lock().unwrap().push("terminated".to_owned());
            true
        } else {
            false
        }
    }
}

/// Logs `r:<text>` for the strings it receives (the oracle's `Worker`).
pub(super) struct Worker(Log);
impl Actor for Worker {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.0.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

pub(super) struct Idle;
impl Actor for Idle {
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

pub(super) struct Fixture {
    pub(super) system: ActorSystem,
    pub(super) executor: Arc<RunsOnDemand>,
    /// `terminated` for every `Terminated` a `Script` actor received
    terminated: Log,
}

impl Fixture {
    pub(super) fn new() -> Fixture {
        Self::in_system("ac")
    }

    pub(super) fn in_system(system_name: &str) -> Fixture {
        Self::with_executor(system_name, |on_demand| on_demand)
    }

    /// A fixture whose user dispatcher goes through `wrap` (e.g. to hold the thread that submits a turn); the turns are still run from the
    /// returned fixture's `executor`.
    pub(super) fn with_executor(
        system_name: &str,
        wrap: impl FnOnce(Arc<RunsOnDemand>) -> Arc<dyn Executor>,
    ) -> Fixture {
        let executor = Arc::new(RunsOnDemand::default());
        let system = ActorSystem::create_with_executor(
            system_name,
            Settings::default(),
            Arc::new(NanoClock),
            wrap(executor.clone()),
        )
        .unwrap();
        Fixture {
            system,
            executor,
            terminated: Log::default(),
        }
    }

    pub(super) fn props(&self, script: Arc<dyn Fn(&mut ActorContext) + Send + Sync>) -> Props {
        let terminated = self.terminated.clone();
        Props::new(move || Script(script.clone(), terminated.clone()))
    }

    /// An actor that runs `script` when told `"go"`, already started (its first turn has run).
    pub(super) fn actor(
        &self,
        name: &str,
        script: impl Fn(&mut ActorContext) + Send + Sync + 'static,
    ) -> ActorRef {
        let actor = self
            .system
            .actor_of_named(self.props(Arc::new(script)), name)
            .unwrap();
        self.started(name, actor)
    }

    /// The same with an anonymous name (`$a`, `$b`, ...).
    fn anonymous_actor(
        &self,
        script: impl Fn(&mut ActorContext) + Send + Sync + 'static,
    ) -> ActorRef {
        let actor = self.system.actor_of(self.props(Arc::new(script))).unwrap();
        let name = actor.path().name().to_owned();
        self.started(&name, actor)
    }

    pub(super) fn started(&self, name: &str, actor: ActorRef) -> ActorRef {
        wait_until(&format!("`{name}` to be submitted"), || {
            self.executor.submissions(name) >= 1
        });
        // `point` submits the new cell before it installs it: wait for the installation too, so the reference is never seen unstarted
        wait_until(&format!("`{name}` to be installed"), || {
            actor.0.as_repointable().is_none_or(|r| r.is_started())
        });
        self.executor.run_until_idle();
        actor
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // the user dispatcher runs nothing by itself: keep running its turns until the termination chain has finished
        let terminated = self.system.terminate();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            self.executor.run_until_idle();
            if terminated.wait(Duration::from_millis(2)).is_some() || Instant::now() > deadline {
                break;
            }
        }
    }
}

pub(super) fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(super) fn recorded(case: &str) -> &'static str {
    ORACLE_GOLDEN
        .lines()
        .find(|line| line.starts_with(case))
        .and_then(|line| line.split_once(" -> "))
        .map(|(_, observed)| observed)
        .expect("the golden has the case")
}

pub(super) fn joined(log: &Log) -> String {
    log.lock().unwrap().join(", ")
}

#[test]
fn attach_child_returns_before_the_child_is_submitted_to_the_executor_and_the_parents_turn_submits_it()
 {
    let fixture = Fixture::new();
    let inside_the_turn = Arc::new(Mutex::new(None));
    let parent = fixture.actor("p", {
        let (executor, inside_the_turn) = (fixture.executor.clone(), inside_the_turn.clone());
        move |ctx| {
            ctx.cell
                .attach_child(Props::new(|| Idle), Some("c"), true)
                .unwrap();
            *inside_the_turn.lock().unwrap() = Some(executor.submissions("c"));
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    assert_eq!(
        *inside_the_turn.lock().unwrap(),
        Some(0),
        "the child cell does not exist yet while the parent's handler runs"
    );
    assert_eq!(
        fixture.executor.submissions("c"),
        1,
        "the parent attaches the child when it processes Supervise, right after the handler"
    );
}

#[test]
fn the_child_is_visible_by_name_and_its_name_is_taken_as_soon_as_attach_child_returns() {
    let fixture = Fixture::new();
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let parent = fixture.actor("p", {
        let seen = seen.clone();
        move |ctx| {
            let child = ctx
                .cell
                .attach_child(Props::new(|| Idle), Some("c"), true)
                .unwrap();
            let found = ctx.child("c").expect("a child is found by name right away");
            let duplicate = ctx
                .actor_of_named(Props::new(|| Idle), "c")
                .err()
                .map(|e| e.to_string());
            seen.lock().unwrap().extend([
                format!("same={}", found == child),
                format!("path={}", child.path()),
                format!("duplicate={}", duplicate.unwrap_or_default()),
            ]);
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    assert_eq!(
        *seen.lock().unwrap(),
        [
            "same=true",
            "path=pekko://ac/user/p/c",
            "duplicate=org.apache.pekko.actor.InvalidActorNameException: actor name [c] is not unique!"
        ]
    );
}

#[test]
fn context_actor_of_is_synchronous_the_child_is_submitted_at_once_and_is_not_repointable() {
    // `Children.actorOf` is `makeChild(..., async = false)`: a started `LocalActorRef` (oracle case MBX-AC-01 uses attachChild for a reason)
    let fixture = Fixture::new();
    let inside_the_turn = Arc::new(Mutex::new(None));
    let parent = fixture.actor("p", {
        let (executor, inside_the_turn) = (fixture.executor.clone(), inside_the_turn.clone());
        move |ctx| {
            let child = ctx.actor_of_named(Props::new(|| Idle), "c").unwrap();
            *inside_the_turn.lock().unwrap() = Some((
                executor.submissions("c"),
                child.0.as_repointable().is_some(),
            ));
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    assert_eq!(*inside_the_turn.lock().unwrap(), Some((1, false)));
}

#[test]
fn a_child_created_under_a_suspended_parent_inherits_the_suspension() {
    // twin of oracle case MBX-UC-04
    let fixture = Fixture::new();
    let parent = fixture.actor("p", |_| {});
    parent.send_system(SystemMessage::Suspend);
    fixture.executor.run_until_idle();
    let log = Log::default();
    let child = parent
        .attach_child_for_tests(
            Props::new({
                let log = log.clone();
                move || Worker(log.clone())
            }),
            "c",
        )
        .unwrap();
    fixture.executor.run_until_idle();
    child.tell("u".to_owned(), None);
    fixture.executor.run_until_idle();
    let suspended = format!("{} log=[{}]", child.mailbox_view(), joined(&log));
    child.send_system(SystemMessage::Resume { cause: None });
    fixture.executor.run_until_idle();
    let observed = format!(
        "childUnderSuspendedParent: {suspended} | afterResume: {} log=[{}]",
        child.mailbox_view(),
        joined(&log)
    );
    assert_eq!(observed, recorded("MBX-UC-04"));
}

#[test]
fn only_the_guardians_are_created_synchronously_every_other_actor_is_repointable() {
    let fixture = Fixture::new();
    let user_actor = fixture.actor("p", |_| {});
    assert!(
        user_actor.0.as_repointable().is_some(),
        "actorOf is the asynchronous path"
    );
    for guardian in [
        fixture.system.0.guardian.get().unwrap().self_ref(),
        fixture.system.0.system_guardian.get().unwrap().self_ref(),
    ] {
        assert!(
            guardian.0.as_repointable().is_none(),
            "{} is created by the provider, synchronously",
            guardian.path()
        );
    }
}

/// Subscribes an actor to the error channel and returns what it collects.
pub(super) fn collect_errors(fixture: &Fixture) -> Arc<Mutex<Vec<ErrorEvent>>> {
    let errors: Arc<Mutex<Vec<ErrorEvent>>> = Arc::default();
    struct Collects(Arc<Mutex<Vec<ErrorEvent>>>);
    impl Actor for Collects {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
                self.0.lock().unwrap().push(error.clone());
            }
            true
        }
    }
    let collector = fixture
        .system
        .actor_of_named(
            Props::new({
                let errors = errors.clone();
                move || Collects(errors.clone())
            }),
            "collector",
        )
        .unwrap();
    wait_until("the collector to be submitted", || {
        fixture.executor.submissions("collector") >= 1
    });
    fixture
        .system
        .event_stream()
        .subscribe(&collector, Channel::Error);
    errors
}

#[test]
fn a_supervise_from_a_child_that_was_never_registered_is_reported_as_an_error() {
    let fixture = Fixture::new();
    let errors = collect_errors(&fixture);
    let parent = fixture.actor("p", |_| {});
    let stranger = fixture.system.dead_letters();
    parent.send_system(SystemMessage::Supervise {
        child: stranger.clone(),
        is_async: true,
    });
    fixture.executor.run_until_idle();
    let errors = errors.lock().unwrap();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].log_source, parent.path().to_string());
    assert_eq!(
        errors[0].message,
        format!("received Supervise from unregistered child {stranger:?}, this will not end well")
    );
    assert_eq!(
        errors[0].cause.class, "org.apache.pekko.event.Logging$Error$NoCause$",
        "`Error(source, class, message)` carries `Error.NoCause`"
    );
}

/// `ActorCell.scala:712-714`: `supervise` does nothing while the parent is terminating, so a `Supervise` that reaches it then is neither
/// registered nor reported (without the guard `initChild` finds no reservation and an `Error` is published).
#[test]
fn a_supervise_that_reaches_a_terminating_parent_is_ignored() {
    let fixture = Fixture::new();
    let errors = collect_errors(&fixture);
    let parent = fixture.actor("p", |ctx| {
        ctx.actor_of_named(Props::new(|| Idle), "c").unwrap();
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();

    parent.stop();
    fixture.executor.run_next(); // the parent's turn: it asks `c` to stop and waits for it, so it is terminating now
    let stranger = fixture.system.dead_letters();
    parent.send_system(SystemMessage::Supervise {
        child: stranger,
        is_async: true,
    });
    fixture.executor.run_until_idle(); // `c` stops, then the parent handles `Supervise` and its child's death in one turn

    assert!(
        errors.lock().unwrap().is_empty(),
        "{:?}",
        errors.lock().unwrap()
    );
}

/// Counts how many times it has been started.
struct CountsStarts(Arc<std::sync::atomic::AtomicUsize>);
impl Actor for CountsStarts {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

/// Before its parent's turn runs, an actor made by `attachChild` holds the system messages sent to it (`UnstartedCell`), has no cell
/// to hand out and cannot create children; pointing it twice (a second `Supervise`) does not start it again
/// (`RepointableActorRef.scala:112-135`). The parent is a user actor on the manual executor, so "its turn has not run" is a fact.
#[test]
fn an_unstarted_actor_holds_system_messages_and_is_started_once() {
    let fixture = Fixture::new();
    let starts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let parent = fixture.actor("p", |_| {});
    let child = parent
        .attach_child_for_tests(
            Props::new({
                let starts = starts.clone();
                move || CountsStarts(starts.clone())
            }),
            "child",
        )
        .unwrap();
    let repointable = child.0.as_repointable().expect("made by attachChild");

    assert!(!repointable.is_started());
    assert!(repointable.installed_cell_for_tests().is_none());
    assert!(!child.0.has_system_messages());
    child.send_system(SystemMessage::Suspend);
    assert!(child.0.has_system_messages(), "kept in the UnstartedCell");
    let refused = child
        .attach_child_for_tests(Props::new(|| Idle), "grandchild")
        .unwrap_err();
    assert_eq!(refused.message(), "the actor has not started");

    fixture.executor.run_until_idle(); // the parent handles `Supervise` and points the child
    assert!(repointable.is_started());
    repointable.point().unwrap(); // a second `Supervise` for the same child
    fixture.executor.run_until_idle();
    assert_eq!(starts.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(repointable.installed_cell_for_tests().is_some());
}

/// A reference that has no cell behind it (dead letters) cannot create children.
#[test]
fn a_reference_without_a_cell_cannot_attach_children() {
    let fixture = Fixture::new();
    let refused = fixture
        .system
        .dead_letters()
        .attach_child_for_tests(Props::new(|| Idle), "child")
        .unwrap_err();
    assert_eq!(refused.message(), "no cell behind this reference");
}

#[test]
fn attach_child_inside_a_turn_returns_before_the_child_starts_and_is_found_by_name() {
    // twin of oracle case MBX-AC-01
    use std::sync::atomic::{AtomicUsize, Ordering};
    let fixture = Fixture::in_system("mbx");
    fixture.system.actor_of(Props::new(|| Idle)).unwrap(); // $a, $b: the oracle's listeners
    fixture.system.actor_of(Props::new(|| Idle)).unwrap();
    let starts = Arc::new(AtomicUsize::new(0));
    let inside = Arc::new(Mutex::new(String::new()));
    let parent = fixture.anonymous_actor({
        let (starts, inside) = (starts.clone(), inside.clone());
        move |ctx| {
            let child = ctx
                .cell
                .attach_child(
                    Props::new({
                        let starts = starts.clone();
                        move || CountsStarts(starts.clone())
                    }),
                    Some("c"),
                    true,
                )
                .unwrap();
            let found = ctx.child("c");
            let duplicate = ctx
                .actor_of_named(Props::new(|| Idle), "c")
                .err()
                .map(|e| e.to_string());
            *inside.lock().unwrap() = format!(
                "startedInsideTurn={} childFound={} sameRef={} path={} duplicate={}",
                starts.load(Ordering::SeqCst),
                found.is_some(),
                found.as_ref() == Some(&child),
                child.path(),
                duplicate.unwrap_or_default()
            );
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    let observed = format!(
        "insideTurn: {} | afterTurn: startedActors={}",
        inside.lock().unwrap(),
        starts.load(Ordering::SeqCst)
    );
    assert_eq!(observed, recorded("MBX-AC-01"));
}

#[test]
fn messages_sent_to_a_child_inside_the_creating_turn_wait_for_it_and_a_suspend_goes_first() {
    // twin of oracle case MBX-UC-01, through the real attach_child
    let fixture = Fixture::new();
    let log = Log::default();
    let child = Arc::new(Mutex::new(None::<ActorRef>));
    let parent = fixture.actor("p", {
        let (log, child) = (log.clone(), child.clone());
        move |ctx| {
            let created = ctx
                .cell
                .attach_child(
                    Props::new({
                        let log = log.clone();
                        move || Worker(log.clone())
                    }),
                    Some("c"),
                    true,
                )
                .unwrap();
            created.tell("m1".to_owned(), None);
            created.tell("m2".to_owned(), None);
            created.send_system(SystemMessage::Suspend);
            *child.lock().unwrap() = Some(created);
        }
    });
    parent.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    let child = child.lock().unwrap().clone().unwrap();
    let after_start = format!("{} log=[{}]", child.mailbox_view(), joined(&log));
    child.send_system(SystemMessage::Resume { cause: None });
    fixture.executor.run_until_idle();
    let observed = format!(
        "afterStart: {after_start} | afterResume: {} log=[{}]",
        child.mailbox_view(),
        joined(&log)
    );
    assert_eq!(observed, recorded("MBX-UC-01"));
}

#[test]
fn a_watch_and_a_terminate_sent_to_a_child_inside_the_creating_turn_are_applied_when_it_starts() {
    // twin of oracle cases MBX-UC-02 (watch then stop) and MBX-UC-03 (watch, unwatch, stop)
    for (case, unwatch) in [("MBX-UC-02", false), ("MBX-UC-03", true)] {
        let fixture = Fixture::new();
        let child = Arc::new(Mutex::new(None::<ActorRef>));
        let parent = fixture.actor("p", {
            let child = child.clone();
            move |ctx| {
                let created = ctx
                    .cell
                    .attach_child(Props::new(|| Idle), Some("c"), true)
                    .unwrap();
                ctx.watch(&created);
                if unwatch {
                    ctx.unwatch(&created);
                }
                created.stop();
                *child.lock().unwrap() = Some(created);
            }
        });
        parent.tell("go".to_owned(), None);
        fixture.executor.run_until_idle();
        let child = child.lock().unwrap().clone().unwrap();
        let observed = format!(
            "watcherLog=[{}] childTerminated={}",
            joined(&fixture.terminated),
            child.is_terminated()
        );
        assert_eq!(observed, recorded(case), "{case}");
    }
}

#[test]
fn many_children_that_message_themselves_while_they_start_lose_and_duplicate_nothing() {
    // `self` of a started child is its repointable reference: sending to it from `pre_start`, while the parent is still handing the
    // queued messages over, must neither deadlock nor lose messages (the lock is never taken twice by one thread)
    use std::sync::atomic::{AtomicUsize, Ordering};
    const CHILDREN: usize = 40;
    struct SelfTalker(Arc<AtomicUsize>);
    impl Actor for SelfTalker {
        fn pre_start(&mut self, ctx: &mut ActorContext) {
            ctx.self_ref().tell("from-pre-start".to_owned(), None);
        }
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.is::<String>() {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
            true
        }
    }
    let system = ActorSystem::create("stress").unwrap();
    let received = Arc::new(AtomicUsize::new(0));
    let parent = system
        .actor_of_named(
            Props::new({
                let received = received.clone();
                move || {
                    struct Parent(Arc<AtomicUsize>);
                    impl Actor for Parent {
                        fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
                            if msg.downcast_ref::<String>().map(String::as_str) == Some("go") {
                                for n in 0..CHILDREN {
                                    let counter = self.0.clone();
                                    let child = ctx
                                        .cell
                                        .attach_child(
                                            Props::new(move || SelfTalker(counter.clone())),
                                            Some(&format!("c{n}")),
                                            true,
                                        )
                                        .unwrap();
                                    child.tell("from-parent".to_owned(), None);
                                }
                            }
                            true
                        }
                    }
                    Parent(received.clone())
                }
            }),
            "p",
        )
        .unwrap();
    parent.tell("go".to_owned(), None);
    wait_until("every message to arrive", || {
        received.load(Ordering::SeqCst) == 2 * CHILDREN
    });
    system.terminate().wait(Duration::from_secs(10)).unwrap();
}

/// A terminated actor made by `system.actorOf` must not stay alive through the reference it knows as `self`
/// (Pekko relies on the GC for this cycle; here the cell would hold the repointable reference that holds the cell).
#[test]
fn a_terminated_top_level_actor_releases_its_cell() {
    let fixture = Fixture::new();
    let actor = fixture.actor("leak-check", |_ctx| {});
    let cell = actor
        .0
        .as_repointable()
        .and_then(|repointable| repointable.installed_cell_for_tests())
        .expect("the actor is started");

    actor.stop();
    fixture.executor.run_until_idle();
    fixture.executor.run_until_idle();
    drop(actor);
    // the guardian forgets its child on its own (internal) dispatcher
    wait_until("the terminated cell to be released", || {
        fixture.executor.run_until_idle();
        cell.strong_count() == 0
    });
}
