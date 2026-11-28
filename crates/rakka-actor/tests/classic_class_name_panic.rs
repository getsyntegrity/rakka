// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:748-753 and actor/dungeon/FaultHandling.scala (Apache-2.0).
//! `Actor::class_name` is only the label of reports. Pekko takes it from `getClass`, which runs no user code; in Rust an
//! override can panic, and the actor must live on as without that panic, labelled with its type. Every wait is bounded.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::panic::panic_any;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::event::{Channel, ErrorEvent};

fn expect(events: &Receiver<String>, event: &str) {
    let seen = events.recv_timeout(Duration::from_secs(10));
    assert_eq!(seen.as_deref(), Ok(event));
}

fn terminates(system: &RunningSystem) {
    let terminated = system.terminate().wait(Duration::from_secs(10));
    terminated.expect("system terminated").expect("normally");
}

/// Reports the strings it receives; its `class_name` panics and so does its `post_stop`, to have a report about it.
struct Mislabelled(Sender<String>);

impl Actor for Mislabelled {
    fn class_name(&self) -> &'static str {
        panic!("class_name")
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        throw(Throwable::illegal_state("stop"));
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = msg.downcast_ref::<&str>().unwrap();
        self.0.send((*text).to_owned()).unwrap();
        true
    }
}

/// Reports the class of every published `Error`.
struct ErrorClasses(Sender<String>);

impl Actor for ErrorClasses {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let error = msg.downcast_ref::<ErrorEvent>().unwrap();
        self.0.send(error.log_class.clone()).unwrap();
        true
    }
}

/// The classes of the `Error`s published from now on.
fn error_classes(system: &RunningSystem) -> Receiver<String> {
    let (classes, reported) = channel();
    let listener = Props::new(move || ErrorClasses(classes.clone()));
    let listener = system.actor_of(listener).unwrap();
    system.event_stream().subscribe(&listener, Channel::Error);
    reported
}

#[test]
fn an_actor_whose_class_name_panics_works_and_is_reported_by_its_type() {
    let system = RunningSystem::create("t-class-name");
    let reported = error_classes(&system);
    let (events, seen) = channel();
    let props = Props::new(move || Mislabelled(events.clone()));
    let actor = system.actor_of(props).unwrap();
    actor.tell("m", None);
    expect(&seen, "m");
    system.stop(&actor);
    expect(&reported, std::any::type_name::<Mislabelled>());
    terminates(&system);
}

/// Panics when dropped.
struct Bomb;

impl Drop for Bomb {
    fn drop(&mut self) {
        panic!("drop");
    }
}

struct Echo(Sender<String>);

impl Actor for Echo {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = msg.downcast_ref::<&str>().unwrap();
        self.0.send(format!("child {text}")).unwrap();
        true
    }
}

/// Keeps its child across a restart. The `class_name` of every instance but the first panics, with a [`Bomb`] as payload.
struct MislabelledAfterRestart(Sender<String>, usize);

impl Actor for MislabelledAfterRestart {
    fn class_name(&self) -> &'static str {
        if self.1 > 0 {
            panic_any(Bomb);
        }
        "first"
    }
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let events = self.0.clone();
        let props = Props::new(move || Echo(events.clone()));
        ctx.actor_of_named(props, "c").unwrap();
    }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {}
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.send("restarted".into()).unwrap();
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        throw(Throwable::illegal_state("stop")); // to have a report about the instance
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match *msg.downcast_ref::<&str>().unwrap() {
            "fail" => throw(Throwable::illegal_state("boom")),
            text => {
                self.0.send(text.to_owned()).unwrap();
                ctx.child("c").unwrap().tell(text, None);
            }
        }
        true
    }
}

#[test]
fn a_restart_into_an_instance_whose_class_name_panics_keeps_the_actor_and_its_child() {
    let system = RunningSystem::create("t-class-name-restart");
    let (events, seen) = channel();
    let born = std::sync::atomic::AtomicUsize::new(0);
    let props = Props::new(move || {
        let generation = born.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        MislabelledAfterRestart(events.clone(), generation)
    });
    let actor = system.actor_of(props).unwrap();
    actor.tell("fail", None);
    expect(&seen, "restarted");
    actor.tell("m", None);
    expect(&seen, "m");
    expect(&seen, "child m");
    // subscribed only now, so that the report of the first failure is not among them
    let reported = error_classes(&system);
    system.stop(&actor);
    // the label is not the "first" of the replaced instance any more
    expect(&reported, std::any::type_name::<MislabelledAfterRestart>());
    terminates(&system);
}
