// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! A `supervisor_strategy` that panics fails the supervisor itself (`handleFailure`, `handleChildTerminated`) instead of
//! being replaced by the default one. The other paths are the cases `st-*` recorded from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::event::{Channel, ErrorEvent};

/// Resumes its child, a [`Parent`], and reports what that one failed with.
struct Top(Sender<String>, Sender<ActorRef>);

impl Actor for Top {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let born = self.1.clone();
        ctx.actor_of_named(Props::new(move || Parent(born.clone())), "p")
            .unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let seen = self.0.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            seen.send(cause.to_string()).unwrap();
            Directive::Resume
        })
        .into()
    }
}

/// Its strategy panics; it has one child, which fails on any message.
struct Parent(Sender<ActorRef>);

impl Actor for Parent {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let child = ctx.actor_of_named(Props::new(|| Child), "c").unwrap();
        self.0.send(child).unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        throw(Throwable::illegal_state("strategy"));
    }
}

struct Child;

impl Actor for Child {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::illegal_state("boom"));
    }
}

/// Hands the `Error`s published by anything under `top` to the test.
struct Probe(Sender<String>);

impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>()
            && error.log_source.ends_with("/top/p")
        {
            let message = error.message.as_deref().unwrap_or("null");
            self.0.send(format!("{}|{message}", error.cause)).unwrap();
        }
        true
    }
}

fn next<T>(receiver: &Receiver<T>) -> T {
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("sent")
}

/// The running system, the grandchild, and what `top` decided about.
fn start(name: &str) -> (RunningSystem, ActorRef, Receiver<String>) {
    let system = RunningSystem::create(name);
    let (decided, decisions) = channel();
    let (born, child) = channel();
    let top = Props::new(move || Top(decided.clone(), born.clone()));
    system.actor_of_named(top, "top").unwrap();
    let child = next(&child);
    (system, child, decisions)
}

#[test]
fn a_strategy_that_panics_on_a_child_failure_fails_the_supervisor_with_what_it_threw() {
    let (_system, child, decisions) = start("t-strategy-failed");
    child.tell("go", None);
    assert_eq!(
        next(&decisions),
        "java.lang.IllegalStateException: strategy"
    );
}

#[test]
fn a_strategy_that_panics_on_a_child_termination_is_published_and_fails_the_supervisor() {
    let (system, child, decisions) = start("t-strategy-terminated");
    let (sender, errors) = channel();
    let probe = system
        .actor_of(Props::new(move || Probe(sender.clone())))
        .unwrap();
    system.event_stream().subscribe(&probe, Channel::Error);
    system.stop(&child);
    assert_eq!(
        next(&errors),
        "java.lang.IllegalStateException: strategy|handleChildTerminated failed"
    );
    assert_eq!(
        next(&decisions),
        "java.lang.IllegalStateException: strategy"
    );
}
