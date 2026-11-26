// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! A user's `Drop` that panics wedges no actor: the failure is decided, and the actors stop, as without that panic. Pekko has
//! no destructors, so the expected behaviour is the one it has for the same failure. Every wait is bounded.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use rakka_actor::event::Channel;

/// Panics when dropped.
struct Bomb;

impl Drop for Bomb {
    fn drop(&mut self) {
        panic!("drop");
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    /// `preStart` throws and the instance holds a [`Bomb`]
    FailedStartThenDrop,
    /// `preStart` panics with a [`Bomb`] as payload
    BombPayload,
    /// the first instance holds a [`Bomb`]
    FirstInstanceDrop,
    /// every strategy returned captures a [`Bomb`]
    DeciderCapture,
    /// the same, and the decider throws
    ThrowingDeciderCapture,
    None,
}

/// Supervises one [`Child`], forwards strings other than "ping" to it and reports what it decides and receives.
struct Supervisor(Sender<String>, Fault);

impl Actor for Supervisor {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let (events, fault) = (self.0.clone(), self.1);
        let born = Mutex::new(0);
        let props = Props::new(move || {
            let mut born = born.lock().unwrap();
            *born += 1;
            let holds = fault == Fault::FailedStartThenDrop
                || (fault == Fault::FirstInstanceDrop && *born == 1);
            Child(events.clone(), fault, holds.then(|| Bomb))
        });
        let child = ctx.actor_of_named(props, "c").unwrap();
        ctx.watch(&child);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (events, fault) = (self.0.clone(), self.1);
        let held =
            matches!(fault, Fault::DeciderCapture | Fault::ThrowingDeciderCapture).then(|| Bomb);
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            let _captured = &held;
            if fault == Fault::ThrowingDeciderCapture {
                throw(Throwable::illegal_state("decider"));
            }
            let root = cause
                .cause
                .as_ref()
                .map_or(String::new(), |c| c.to_string());
            let decides = format!("decides {} {root}", cause.class);
            events.send(decides).unwrap();
            match cause.class.as_str() {
                "java.lang.IllegalStateException" => Directive::Restart,
                _ => Directive::Stop,
            }
        })
        .into()
    }
    fn post_restart(&mut self, ctx: &mut ActorContext, _: &Throwable) {
        self.pre_start(ctx);
        self.0.send("supervisor restarted".into()).unwrap();
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<&str>() {
            Some(&"ping") => self.0.send("supervisor alive".into()).unwrap(),
            Some(&message) => ctx.child("c").unwrap().tell(message, None),
            None => self.0.send("child terminated".into()).unwrap(),
        }
        true
    }
}

struct Child(Sender<String>, Fault, #[allow(dead_code)] Option<Bomb>);

impl Actor for Child {
    fn pre_start(&mut self, _: &mut ActorContext) {
        match self.1 {
            Fault::FailedStartThenDrop => throw(Throwable::illegal_state("start")),
            Fault::BombPayload => panic_any(Bomb),
            _ => {}
        }
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<&str>() == Some(&"fail") {
            throw(Throwable::illegal_state("boom"));
        }
        self.0.send("child alive".into()).unwrap();
        true
    }
}

const INIT_FAILURE: &str = "decides org.apache.pekko.actor.ActorInitializationException";
const FAILURE: &str = "decides java.lang.IllegalStateException ";

fn expect(events: &Receiver<String>, event: &str) {
    let seen = events.recv_timeout(Duration::from_secs(10));
    assert_eq!(seen.as_deref(), Ok(event));
}

fn start(name: &str, fault: Fault) -> (RunningSystem, ActorRef, Receiver<String>) {
    let system = RunningSystem::create(name);
    let (events, seen) = channel();
    let props = Props::new(move || Supervisor(events.clone(), fault));
    let supervisor = system.actor_of_named(props, "s").unwrap();
    (system, supervisor, seen)
}

fn terminates(system: &RunningSystem) {
    let terminated = system.terminate().wait(Duration::from_secs(10));
    terminated.expect("system terminated").expect("normally");
}

#[test]
fn an_instance_whose_drop_panics_after_a_failed_start_is_still_decided_about() {
    let (system, _, seen) = start("t-drop-start", Fault::FailedStartThenDrop);
    let root = "java.lang.IllegalStateException: start";
    expect(&seen, &format!("{INIT_FAILURE} {root}"));
    expect(&seen, "child terminated");
    terminates(&system);
}

#[test]
fn a_panic_payload_whose_drop_panics_is_still_decided_about() {
    let (system, _, seen) = start("t-drop-payload", Fault::BombPayload);
    expect(&seen, &format!("{INIT_FAILURE} java.lang.RuntimeException"));
    expect(&seen, "child terminated");
    terminates(&system);
}

#[test]
fn a_stopped_actor_whose_drop_panics_terminates() {
    let (system, supervisor, seen) = start("t-drop-stop", Fault::FirstInstanceDrop);
    supervisor.tell("hello", None);
    expect(&seen, "child alive");
    terminates(&system);
}

#[test]
fn a_replaced_instance_whose_drop_panics_does_not_stop_the_restart() {
    let (system, supervisor, seen) = start("t-drop-restart", Fault::FirstInstanceDrop);
    supervisor.tell("fail", None);
    expect(&seen, FAILURE);
    supervisor.tell("hello", None);
    expect(&seen, "child alive");
    terminates(&system);
}

#[test]
fn a_decider_capture_whose_drop_panics_does_not_stop_the_supervisor_after_its_decision() {
    let (system, supervisor, seen) = start("t-drop-decider", Fault::DeciderCapture);
    supervisor.tell("fail", None);
    expect(&seen, FAILURE);
    supervisor.tell("ping", None);
    expect(&seen, "supervisor alive");
    terminates(&system);
}

#[test]
fn a_decider_capture_whose_drop_panics_does_not_stop_a_termination() {
    // no failure: the strategy is only asked for when the child terminates
    let (system, _, _seen) = start("t-drop-termination", Fault::DeciderCapture);
    terminates(&system);
}

#[test]
fn a_throwing_decider_whose_capture_panics_on_drop_still_fails_the_supervisor() {
    let (system, supervisor, seen) = start("t-drop-throwing", Fault::ThrowingDeciderCapture);
    supervisor.tell("fail", None);
    // the supervisor fails with what the decider threw; the guardian restarts it and its new child answers
    expect(&seen, "supervisor restarted");
    supervisor.tell("hello", None);
    expect(&seen, "child alive");
    terminates(&system);
}

/// A mailbox whose enqueue panics with a [`Bomb`] as payload.
struct BombMailbox;

#[rustfmt::skip] // one stub per line
impl MailboxType for BombMailbox {
    fn create(&self, _: &ActorRef) -> Box<dyn MessageQueue> { Box::new(BombMailbox) }
}

#[rustfmt::skip]
impl MessageQueue for BombMailbox {
    fn enqueue(&self, _: &ActorRef, _: Envelope) -> bool { panic_any(Bomb) }
    fn dequeue(&self) -> Option<Envelope> { None }
    fn number_of_messages(&self) -> usize { 0 }
    fn has_messages(&self) -> bool { false }
    fn clean_up(&self, _: &ActorRef, _: &dyn MessageQueue) {}
}

#[test]
fn a_subscriber_panic_payload_whose_drop_panics_does_not_stop_the_decision() {
    let (system, supervisor, seen) = start("t-drop-subscriber", Fault::None);
    let silent = Props::new(|| Supervisor(channel().0, Fault::None)).with_mailbox(BombMailbox);
    let broken = system.actor_of(silent).unwrap();
    system.event_stream().subscribe(&broken, Channel::Error);
    supervisor.tell("fail", None);
    expect(&seen, FAILURE);
    supervisor.tell("hello", None);
    expect(&seen, "child alive");
    terminates(&system);
}

/// Its post_stop reports and then waits until the test lets go.
struct Lingering(Sender<()>, Arc<Mutex<()>>);

impl Actor for Lingering {
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.0.send(()).unwrap();
        drop(self.1.lock());
    }
}

/// Holds a [`Bomb`] and a [`Lingering`] child, so that its restart waits.
struct SlowRestart(Sender<()>, Arc<Mutex<()>>, #[allow(dead_code)] Bomb);

impl Actor for SlowRestart {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let (stopping, gate) = (self.0.clone(), self.1.clone());
        let props = Props::new(move || Lingering(stopping.clone(), gate.clone()));
        ctx.actor_of(props).unwrap();
    }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::illegal_state("boom"));
    }
}

#[test]
fn an_actor_stopped_while_it_restarts_releases_the_replaced_instance_when_it_terminates() {
    let system = RunningSystem::create("t-drop-parked");
    let ((stopping, child_stopping), gate) = (channel(), Arc::new(Mutex::new(())));
    let held = gate.lock().unwrap();
    let props = {
        let gate = gate.clone();
        Props::new(move || SlowRestart(stopping.clone(), gate.clone(), Bomb))
    };
    let actor = system.actor_of(props).unwrap();
    actor.tell("fail", None);
    let waiting = child_stopping.recv_timeout(Duration::from_secs(10));
    waiting.expect("the restart waits for the child");
    system.stop(&actor);
    drop(held);
    terminates(&system);
    // the instance went with the actor: letting go of the last reference runs no user code
    let released = catch_unwind(AssertUnwindSafe(|| drop(actor)));
    assert!(released.is_ok(), "the replaced instance outlived its actor");
}
