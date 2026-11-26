// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala, actor/dungeon/Dispatch.scala, actor/dungeon/FaultHandling.scala and actor/ActorCell.scala (Apache-2.0).
//! Failure to make the cell of an asynchronously created actor (Q-MBX-11, slice C1): `RepointableActorRef.point(catchFailures = true)`
//! catches what `newCell` throws and starts a replacement cell (`ActorCell.initWithFailure`) whose only message is
//! `Create(Some(ActorInitializationException(self, "failure while creating ActorCell", failure)))`; the actor then fails like any
//! actor that fails to initialise, and its supervisor decides.
//!
//! Twins of oracle cases `MBX-IF-01..03` (lines of `cases/oracle-only/mailbox.golden` printed by the Pekko 2.0.0 JVM, where the failure is a
//! `MailboxType` whose `create` throws). Nothing in Rakka can make `newCell` fail yet, so the tests supply a cell maker that does.

use crate::actor::ActorPath;
use crate::actor::actor_ref::ActorRef;
use crate::actor::async_creation_tests::{
    Fixture, Idle, Log, collect_errors, joined, recorded, wait_until,
};
use crate::actor::cell::ActorCell;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::SystemInner;
use crate::event::{Channel, DeadLetter};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// `newCell` throwing: the oracle's `MailboxType` whose `create` throws `IllegalStateException("mailbox boom")`.
fn cell_that_cannot_be_made(
    _system: &Arc<SystemInner>,
    _props: Props,
    _parent: ActorRef,
    _path: ActorPath,
    _outer: &ActorRef,
) -> Result<Arc<ActorCell>, Throwable> {
    Err(Throwable::new(
        "java.lang.IllegalStateException",
        Some("mailbox boom"),
    ))
}

/// The oracle's `Worker`: counts its `preStart`s and logs `r:<text>` for the strings it receives.
struct Worker {
    starts: Arc<AtomicUsize>,
    log: Log,
}

impl Worker {
    fn props(starts: &Arc<AtomicUsize>, log: &Log) -> Props {
        let (starts, log) = (starts.clone(), log.clone());
        Props::new(move || Worker {
            starts: starts.clone(),
            log: log.clone(),
        })
    }
}

impl Actor for Worker {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.starts.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.log.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

/// The oracle's `RestartingParent`/`RestartingSupervisor`: creates `c` when told `"go"` (with a cell that cannot be made, or not) and
/// restarts it whatever it throws.
struct RestartingParent {
    child_props: Props,
    child: Arc<Mutex<Option<ActorRef>>>,
    cell_can_be_made: bool,
    resumes: bool,
}

impl Actor for RestartingParent {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let directive = if self.resumes {
            Directive::Resume
        } else {
            Directive::Restart
        };
        OneForOneStrategy::new(10, move |_| directive).into()
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<String>().map(String::as_str) != Some("go") {
            return false;
        }
        let child = if self.cell_can_be_made {
            ctx.cell
                .attach_child(self.child_props.clone(), Some("c"), true)
        } else {
            ctx.cell.attach_child_made_by(
                self.child_props.clone(),
                Some("c"),
                true,
                cell_that_cannot_be_made,
            )
        }
        .unwrap();
        *self.child.lock().unwrap() = Some(child);
        true
    }
}

/// The supervisor of an actor `a` whose cell cannot be made, and that actor: what `system.actorOf(...)` hands out in the oracle.
fn actor_whose_cell_cannot_be_made(fixture: &Fixture, props: Props) -> (ActorRef, ActorRef) {
    let child = Arc::new(Mutex::new(None));
    let supervisor = fixture.actor("supervisor", {
        let (child, props) = (child.clone(), props);
        move |ctx| {
            let created = ctx
                .cell
                .attach_child_made_by(props.clone(), Some("a"), true, cell_that_cannot_be_made)
                .unwrap();
            *child.lock().unwrap() = Some(created);
        }
    });
    supervisor.tell("go".to_owned(), None);
    fixture.executor.run_until_idle();
    let created = child.lock().unwrap().clone();
    let created =
        created.expect("attach_child returns the reference even though the cell cannot be made");
    (supervisor, created)
}

fn run_until(fixture: &Fixture, what: &str, condition: impl Fn() -> bool) {
    wait_until(what, || {
        fixture.executor.run_until_idle();
        condition()
    });
}

#[test]
fn a_cell_that_cannot_be_made_is_stopped_by_the_supervisor_and_the_actor_never_starts() {
    // twin of oracle case MBX-IF-01
    let fixture = Fixture::in_system("mbx");
    let (starts, log) = (Arc::new(AtomicUsize::new(0)), Log::default());
    let (supervisor, actor) =
        actor_whose_cell_cannot_be_made(&fixture, Worker::props(&starts, &log));
    run_until(&fixture, "the supervisor to stop the actor", || {
        actor.is_terminated()
    });
    run_until(&fixture, "the name to be free again", || {
        supervisor
            .attach_child_for_tests(Props::new(|| Idle), "a")
            .is_ok()
    });
    let observed = format!(
        "refReturned=true started={} terminated={} nameReusable=created",
        starts.load(Ordering::SeqCst),
        actor.is_terminated(),
    );
    assert_eq!(observed, recorded("MBX-IF-01"));
}

#[test]
fn messages_sent_to_an_actor_whose_cell_cannot_be_made_are_dead_letters() {
    // twin of oracle case MBX-IF-02
    let fixture = Fixture::in_system("mbx");
    let dead = collect_dead_letters(&fixture);
    let (starts, log) = (Arc::new(AtomicUsize::new(0)), Log::default());
    let (_, actor) = actor_whose_cell_cannot_be_made(&fixture, Worker::props(&starts, &log));
    actor.tell("before".to_owned(), None);
    run_until(&fixture, "the supervisor to stop the actor", || {
        actor.is_terminated()
    });
    actor.tell("after".to_owned(), None);
    run_until(&fixture, "both messages to be dead letters", || {
        dead.lock().unwrap().len() == 2
    });
    let letters: BTreeSet<String> = dead.lock().unwrap().iter().cloned().collect();
    let observed = format!(
        "dead=[{}] log=[{}] started={}",
        letters.into_iter().collect::<Vec<_>>().join(", "),
        joined(&log),
        starts.load(Ordering::SeqCst)
    );
    assert_eq!(observed, recorded("MBX-IF-02"));
}

/// What the oracle prints for an actor whose cell cannot be made under a supervisor that answers every failure with `directive`.
fn actor_started_by_a_supervisor_that_answers(resumes: bool) -> String {
    actor_started_by_a_supervisor_that_answers_with(resumes, Worker::props)
}

fn actor_started_by_a_supervisor_that_answers_with(
    resumes: bool,
    worker_props: fn(&Arc<AtomicUsize>, &Log) -> Props,
) -> String {
    let fixture = Fixture::in_system("mbx");
    let errors = collect_errors(&fixture);
    let (starts, log) = (Arc::new(AtomicUsize::new(0)), Log::default());
    let child = Arc::new(Mutex::new(None));
    let parent = {
        let (child_props, child) = (worker_props(&starts, &log), child.clone());
        fixture
            .system
            .actor_of_named(
                Props::new(move || RestartingParent {
                    child_props: child_props.clone(),
                    child: child.clone(),
                    cell_can_be_made: false,
                    resumes,
                }),
                "parent",
            )
            .unwrap()
    };
    parent.tell("go".to_owned(), None);
    run_until(&fixture, "the actor to start", || {
        starts.load(Ordering::SeqCst) == 1
    });
    let child = child.lock().unwrap().clone().unwrap();
    child.tell("m".to_owned(), None);
    run_until(&fixture, "the actor to handle the message", || {
        log.lock().unwrap().len() == 1
    });
    format!(
        "errors=[{}] started={} log=[{}] terminated={}",
        published_errors(&errors).join(", "),
        starts.load(Ordering::SeqCst),
        joined(&log),
        child.is_terminated()
    )
}

#[test]
fn a_supervisor_that_restarts_turns_the_failed_creation_into_a_create_and_the_actor_starts() {
    // twin of oracle case MBX-IF-03
    assert_eq!(
        actor_started_by_a_supervisor_that_answers(false),
        recorded("MBX-IF-03")
    );
}

#[test]
fn a_supervisor_that_resumes_turns_the_failed_creation_into_a_create_and_the_actor_starts() {
    // twin of oracle case MBX-IF-05
    assert_eq!(
        actor_started_by_a_supervisor_that_answers(true),
        recorded("MBX-IF-05")
    );
}

/// `initWithFailure` always uses `new UnboundedMailbox` (`Dispatch.scala:117-125`), so the replacement of a cell that could not be made
/// ignores the mailbox the props selected: a message that a capacity-0 mailbox would refuse is received.
#[test]
fn the_replacement_of_a_cell_that_cannot_be_made_ignores_the_selected_bounded_mailbox() {
    let bounded_props = |starts: &Arc<AtomicUsize>, log: &Log| {
        Worker::props(starts, log).with_mailbox(
            crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox::new(0).unwrap(),
        )
    };
    assert_eq!(
        actor_started_by_a_supervisor_that_answers_with(false, bounded_props),
        recorded("MBX-IF-03")
    );
}

/// An actor that needs a bounded queue (`RequiresMessageQueue[BoundedMessageQueueSemantics]`).
struct NeedsABoundedQueue;

impl Actor for NeedsABoundedQueue {
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }

    fn required_message_queue() -> Option<crate::dispatch::queue_semantics::MessageQueueSemantics> {
        Some(crate::dispatch::queue_semantics::MessageQueueSemantics::BOUNDED)
    }
}

/// A parent that records, for every failure of its child, `class: message` of what its supervisor strategy receives, and stops the child.
struct RecordingParent {
    child_props: Props,
    causes: Arc<Mutex<Vec<String>>>,
}

impl Actor for RecordingParent {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let causes = self.causes.clone();
        OneForOneStrategy::new(10, move |cause| {
            causes.lock().unwrap().push(format!(
                "{}: {}",
                cause.class,
                cause.message.as_deref().unwrap_or("")
            ));
            Directive::Stop
        })
        .into()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<String>().map(String::as_str) != Some("go") {
            return false;
        }
        ctx.cell
            .attach_child_made_by(
                self.child_props.clone(),
                Some("c"),
                true,
                cell_that_cannot_be_made,
            )
            .unwrap();
        true
    }
}

/// Direct witness of the wiring of `Dispatch.init` in `ActorCell::build`: the replacement of a cell that could not be made has the default UNBOUNDED queue, so the re-check of the
/// requirement of the actor (bounded) would fail on it too; what the supervisor receives must stay the failure that made the replacement necessary (`initWithFailure`: the
/// re-check runs only when nothing failed before), never the second one.
#[test]
fn the_failure_that_made_the_replacement_necessary_is_kept_ahead_of_the_unmet_queue_requirement_of_that_replacement()
 {
    let fixture = Fixture::in_system("mbx");
    let causes = Arc::new(Mutex::new(Vec::new()));
    let child_props = Props::new(|| NeedsABoundedQueue).with_mailbox(
        crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox::new(0).unwrap(),
    );
    let parent = fixture
        .system
        .actor_of_named(
            Props::new({
                let causes = causes.clone();
                move || RecordingParent {
                    child_props: child_props.clone(),
                    causes: causes.clone(),
                }
            }),
            "parent",
        )
        .unwrap();
    parent.tell("go".to_owned(), None);
    run_until(&fixture, "the supervisor to receive the failure", || {
        !causes.lock().unwrap().is_empty()
    });
    let received = causes.lock().unwrap().clone();
    assert_eq!(received.len(), 1, "{received:?}");
    assert!(
        received[0].starts_with("org.apache.pekko.actor.ActorInitializationException: "),
        "{received:?}"
    );
    assert!(
        received[0].ends_with("failure while creating ActorCell"),
        "the first failure is what the supervisor receives: {received:?}"
    );
    assert!(
        !received[0].contains("requires mailbox type"),
        "never the unmet requirement of the replacement: {received:?}"
    );
}

/// Subscribes an actor to the dead letters and returns the text of what it collects.
fn collect_dead_letters(fixture: &Fixture) -> Arc<Mutex<Vec<String>>> {
    let dead: Arc<Mutex<Vec<String>>> = Arc::default();
    struct Collects(Arc<Mutex<Vec<String>>>);
    impl Actor for Collects {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if let Some(letter) = msg.downcast_ref::<DeadLetter>()
                && let Some(text) = letter.message.downcast_ref::<String>()
            {
                self.0.lock().unwrap().push(text.clone());
            }
            true
        }
    }
    let collector = fixture
        .system
        .actor_of_named(
            Props::new({
                let dead = dead.clone();
                move || Collects(dead.clone())
            }),
            "dead-letter-collector",
        )
        .unwrap();
    wait_until("the collector to be submitted", || {
        fixture.executor.submissions("dead-letter-collector") >= 1
    });
    fixture
        .system
        .event_stream()
        .subscribe(&collector, Channel::DeadLetter);
    dead
}

/// The oracle's `Rebuilder`: creates `k` in `preStart` and throws on the first start.
struct Rebuilder {
    starts: Arc<AtomicUsize>,
    log: Log,
    children_started: Arc<AtomicUsize>,
}

impl Actor for Rebuilder {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let attempt = self.starts.fetch_add(1, Ordering::SeqCst) + 1;
        let (children_started, log) = (self.children_started.clone(), Log::default());
        ctx.actor_of_named(Worker::props(&children_started, &log), "k")
            .expect("the previous `k` is gone before the actor is created again");
        self.log.lock().unwrap().push(format!("preStart:{attempt}"));
        if attempt == 1 {
            throw(Throwable::illegal_state("first start fails"));
        }
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.log.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

#[test]
fn an_actor_that_fails_to_start_after_creating_a_child_is_restarted_as_a_create_once_its_children_are_gone()
 {
    // twin of oracle case MBX-IF-04
    let fixture = Fixture::in_system("mbx");
    let errors = collect_errors(&fixture);
    let (starts, children_started, log) = (
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Log::default(),
    );
    let child_props = Props::new({
        let (starts, children_started, log) =
            (starts.clone(), children_started.clone(), log.clone());
        move || Rebuilder {
            starts: starts.clone(),
            log: log.clone(),
            children_started: children_started.clone(),
        }
    });
    let child = Arc::new(Mutex::new(None));
    let supervisor = fixture
        .system
        .actor_of_named(
            Props::new({
                let child = child.clone();
                move || RestartingParent {
                    child_props: child_props.clone(),
                    child: child.clone(),
                    cell_can_be_made: true,
                    resumes: false,
                }
            }),
            "sup",
        )
        .unwrap();
    supervisor.tell("go".to_owned(), None);
    run_until(&fixture, "the second start", || {
        joined(&log).contains("preStart:2")
    });
    run_until(&fixture, "both children to have started", || {
        children_started.load(Ordering::SeqCst) == 2
    });
    let child = child.lock().unwrap().clone().unwrap();
    child.tell("m".to_owned(), None);
    run_until(&fixture, "the message to be handled", || {
        joined(&log).contains("r:m")
    });
    let observed = format!(
        "errors=[{}] starts={} childStarts={} log=[{}] terminated={}",
        published_errors(&errors).join(", "),
        starts.load(Ordering::SeqCst),
        children_started.load(Ordering::SeqCst),
        joined(&log),
        child.is_terminated()
    );
    assert_eq!(observed, recorded("MBX-IF-04"));
}

/// `logSource|cause class|message` of every error published so far, without the supervisor's `logFailure` lines, which carry the
/// failure's own class (the oracle's `cellErrors`).
fn published_errors(errors: &Mutex<Vec<crate::event::ErrorEvent>>) -> Vec<String> {
    errors
        .lock()
        .unwrap()
        .iter()
        .filter(|error| error.cause.class != "org.apache.pekko.actor.ActorInitializationException")
        .map(|error| {
            format!(
                "{}|{}|{}",
                error.log_source,
                error.cause.class,
                error.message.as_deref().unwrap()
            )
        })
        .collect()
}
