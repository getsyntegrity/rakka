// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorRefSpec.scala, actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala, actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala and actor-tests/src/test/scala/org/apache/pekko/actor/Supervisor.scala (Apache-2.0).
//! Supervision assertions of Pekko's specs with real actors, written after the code (parity evidence, not TDD tests).
//! The specs' `withinTimeRange` has no Rust counterpart yet (AC-SUP-006.2); no test here comes near a restart limit.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::supervision::{
    AllForOneStrategy, Directive, OneForOneStrategy, SupervisorStrategy,
};

const EXCEPTION_MESSAGE: &str = "Expected exception; to test fault-tolerance";

fn next<T>(receiver: &Receiver<T>) -> T {
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("sent")
}

/// The next `count` entries, sorted, after which nothing else shows up for a short while.
fn exactly(receiver: &Receiver<String>, count: usize) -> Vec<String> {
    let mut entries: Vec<String> = (0..count).map(|_| next(receiver)).collect();
    let extra = receiver.recv_timeout(Duration::from_millis(150));
    assert!(extra.is_err(), "unexpected {extra:?} after {entries:?}");
    entries.sort();
    entries
}

/// Asks a [`Keeper`] for a child and sends back its reference.
struct Spawn(Props, Sender<ActorRef>);

/// The specs' `Supervisor` and `CountDownActor`: creates the children it is asked for, keeps them over a restart and reports its restarts.
#[derive(Clone)]
struct Keeper {
    name: &'static str,
    strategy: SupervisorStrategy,
    restarted: Sender<String>,
}

impl Keeper {
    fn props(
        name: &'static str,
        strategy: SupervisorStrategy,
        restarted: &Sender<String>,
    ) -> Props {
        let keeper = Keeper {
            name,
            strategy,
            restarted: restarted.clone(),
        };
        Props::new(move || keeper.clone())
    }
}

impl Actor for Keeper {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        self.strategy.clone()
    }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {}
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.restarted.send(self.name.to_string()).unwrap();
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(Spawn(props, born)) = msg.downcast_ref::<Spawn>() else {
            return false;
        };
        born.send(ctx.actor_of(props.clone()).unwrap()).unwrap();
        true
    }
}

fn child_of(parent: &ActorRef, props: Props) -> ActorRef {
    let (born, child) = channel();
    parent.tell(Spawn(props, born), None);
    next(&child)
}

fn one_for_one(max: i32, directive: Directive) -> SupervisorStrategy {
    OneForOneStrategy::new(max, move |_: &Throwable| directive).into()
}

fn all_for_one(max: i32, directive: Directive) -> SupervisorStrategy {
    AllForOneStrategy::new(max, move |_: &Throwable| directive).into()
}

struct Hooked(Sender<String>);

impl Actor for Hooked {
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        self.0.send("pre-restart".into()).unwrap();
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.send("post-restart".into()).unwrap();
    }
}

/// `ActorRefSpec` "restart when Kill:ed" (AC-CELL-012).
#[test]
fn a_killed_actor_under_a_restarting_supervisor_runs_both_restart_hooks() {
    let system = RunningSystem::create("t-kill-restart");
    let (hooks, seen) = channel();
    let (ignored, _restarts) = channel();
    let boss = Keeper::props("boss", one_for_one(2, Directive::Restart), &ignored);
    let boss = system.actor_of(boss).unwrap();
    let child = child_of(&boss, Props::new(move || Hooked(hooks.clone())));

    child.tell(Kill, None);

    assert_eq!(exactly(&seen, 2), ["post-restart", "pre-restart"]);
}

/// `SupervisorSpec.PingPongActor`.
struct PingPong(&'static str, Sender<String>);

impl Actor for PingPong {
    fn post_restart(&mut self, _: &mut ActorContext, reason: &Throwable) {
        let reason = reason.message.clone().unwrap_or_default();
        self.1.send(format!("{}:{reason}", self.0)).unwrap();
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(pong) = msg.downcast_ref::<Sender<()>>() {
            pong.send(()).unwrap();
        } else {
            throw(Throwable::new(
                "java.lang.RuntimeException",
                Some(EXCEPTION_MESSAGE),
            ));
        }
        true
    }
}

fn ping_pong(parent: &ActorRef, name: &'static str, restarted: &Sender<String>) -> ActorRef {
    let restarted = restarted.clone();
    child_of(
        parent,
        Props::new(move || PingPong(name, restarted.clone())),
    )
}

/// ping all, kill the second, see all three restart with its failure, ping all again.
fn call_kill_call(actors: &[ActorRef; 3], restarts: &Receiver<String>) {
    let ping_all = || {
        for actor in actors {
            let (pong, answered) = channel::<()>();
            actor.tell(pong, None);
            next(&answered);
        }
    };
    ping_all();
    actors[1].tell("die", None);
    let expected = ["one", "three", "two"].map(|name| format!("{name}:{EXCEPTION_MESSAGE}"));
    assert_eq!(exactly(restarts, 3), expected);
    ping_all();
}

/// `SupervisorSpec` "(call-kill-call) kill multiple actors AllForOne" (AC-SUP-007).
#[test]
fn all_for_one_restarts_all_three_children_when_one_fails() {
    let system = RunningSystem::create("t-all-for-one");
    let (restarted, restarts) = channel();
    let (ignored, _supervisors) = channel();
    let top = Keeper::props("top", all_for_one(3, Directive::Restart), &ignored);
    let top = system.actor_of(top).unwrap();
    let actors = ["one", "two", "three"].map(|name| ping_pong(&top, name, &restarted));
    call_kill_call(&actors, &restarts);
}

/// `SupervisorSpec` "restart killed actors in nested supervisor hierarchy" (AC-SUP-007): the middle supervisor decides nothing, so it
/// escalates and the top one restarts both of its children.
#[test]
fn all_for_one_restarts_a_nested_hierarchy_when_the_inner_supervisor_escalates() {
    let system = RunningSystem::create("t-all-for-one-nested");
    let (restarted, restarts) = channel();
    let (supervisor_restarted, supervisors) = channel();
    let top = all_for_one(3, Directive::Restart);
    let top = system
        .actor_of(Keeper::props("top", top, &supervisor_restarted))
        .unwrap();
    let one = ping_pong(&top, "one", &restarted);
    let middle = all_for_one(3, Directive::Escalate);
    let middle = child_of(&top, Keeper::props("middle", middle, &supervisor_restarted));
    let two = ping_pong(&middle, "two", &restarted);
    let three = ping_pong(&middle, "three", &restarted);

    call_kill_call(&[one, two, three], &restarts);

    assert_eq!(exactly(&supervisors, 1), ["middle"]);
}

/// `SupervisorHierarchySpec` "restart manager and workers in AllForOne" (AC-SUP-007).
#[test]
fn an_escalating_manager_is_restarted_together_with_its_workers() {
    let system = RunningSystem::create("t-manager-workers");
    let (restarted, restarts) = channel();
    let boss = Keeper::props("boss", one_for_one(-1, Directive::Restart), &restarted);
    let boss = system.actor_of(boss).unwrap();
    let manager = all_for_one(-1, Directive::Escalate);
    let manager = child_of(&boss, Keeper::props("manager", manager, &restarted));
    let workers = ["w1", "w2", "w3"].map(|name| {
        let worker = Keeper::props(name, SupervisorStrategy::default(), &restarted);
        child_of(&manager, worker)
    });

    workers[0].tell(Kill, None);

    assert_eq!(exactly(&restarts, 4), ["manager", "w1", "w2", "w3"]);
}

/// The specs' `testActor`: records the text it is told.
struct Probe(Sender<String>);

impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<&str>() else {
            return false;
        };
        self.0.send(text.to_string()).unwrap();
        true
    }
}

#[derive(Clone, Default)]
struct Creation {
    attempts: Arc<AtomicUsize>,
    pre_starts: Arc<AtomicUsize>,
    post_restarts: Arc<AtomicUsize>,
}

struct Echo(Creation);

impl Actor for Echo {
    fn pre_start(&mut self, _: &mut ActorContext) {
        self.0.pre_starts.fetch_add(1, Ordering::SeqCst);
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.post_restarts.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        ctx.sender().tell_any(msg.clone(), None);
        true
    }
}

struct FailResumer(Creation, Option<ActorRef>);

impl Actor for FailResumer {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let attempts = self.0.attempts.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            if cause.class != "org.apache.pekko.actor.ActorInitializationException" {
                Directive::Escalate
            } else if attempts.load(Ordering::SeqCst).is_multiple_of(2) {
                Directive::Resume
            } else {
                Directive::Restart
            }
        })
        .into()
    }
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let creation = self.0.clone();
        let failing = Props::new(move || {
            if creation.attempts.fetch_add(1, Ordering::SeqCst) + 1 < 6 {
                throw(Throwable::new(
                    "java.lang.IllegalArgumentException",
                    Some("OH NO!"),
                ));
            }
            Echo(creation.clone())
        });
        self.1 = Some(ctx.actor_of_named(failing, "failChild").unwrap());
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        ctx.forward(self.1.as_ref().expect("created"), msg.clone());
        true
    }
}

/// `SupervisorHierarchySpec` "handle failure in creation when supervision strategy returns Resume and Restart" (AC-SUP-011 A1),
/// not 1:1 (Q-CELL-8): no `workingChild`, which the spec's constructor creates, and `failChild` is made in the supervisor's `pre_start`,
/// not in its constructor.
#[test]
fn a_creation_that_fails_five_times_ends_in_one_instance() {
    let system = RunningSystem::create("t-creation-retries");
    let (received, replies) = channel();
    let test_actor = system
        .actor_of(Props::new(move || Probe(received.clone())))
        .unwrap();
    let creation = Creation::default();
    let for_resumer = creation.clone();
    let resumer = system
        .actor_of(Props::new(move || FailResumer(for_resumer.clone(), None)))
        .unwrap();

    resumer.tell("blahonga", Some(&test_actor));

    assert_eq!(next(&replies), "blahonga");
    assert_eq!(creation.attempts.load(Ordering::SeqCst), 6);
    assert_eq!(creation.pre_starts.load(Ordering::SeqCst), 1);
    assert_eq!(creation.post_restarts.load(Ordering::SeqCst), 0);
}
