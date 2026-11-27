// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:136-171 and dispatch/AbstractDispatcher.scala:279-285 (Apache-2.0).
//! A user's `MessageQueue` is public SPI and may panic. Expected values: Pekko 2.0.0 (recorded cases come in a follow-up change).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::event::{Channel, DeadLetter, ErrorEvent};

type Log = Arc<Mutex<Vec<String>>>;

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    Enqueue,
    CleanUp,
}

fn boom(what: &str) -> ! {
    throw(Throwable::illegal_state(&format!("{what} boom")))
}

fn is_bad(envelope: &Envelope) -> bool {
    envelope
        .message
        .downcast_ref::<String>()
        .is_some_and(|text| text == "bad")
}

/// A queue with one operation that throws: `enqueue` of "bad", or `clean_up`.
struct FaultyQueue {
    inner: Box<dyn MessageQueue>,
    fault: Fault,
}

impl MessageQueue for FaultyQueue {
    fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool {
        if self.fault == Fault::Enqueue && is_bad(&envelope) {
            boom("enqueue");
        }
        self.inner.enqueue(receiver, envelope)
    }
    fn dequeue(&self) -> Option<Envelope> {
        self.inner.dequeue()
    }
    fn number_of_messages(&self) -> usize {
        self.inner.number_of_messages()
    }
    fn has_messages(&self) -> bool {
        self.inner.has_messages()
    }
    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        if self.fault == Fault::CleanUp {
            boom("cleanUp");
        }
        self.inner.clean_up(owner, dead_letters);
    }
}

struct FaultyMailbox(Fault);
impl MailboxType for FaultyMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(FaultyQueue {
            inner: UnboundedMailbox.create(),
            fault: self.0,
        })
    }
}

/// Logs every string; `block` and `block-then-stop` hold the turn until the gate opens.
struct Worker(Log, Arc<Gate>);
impl Actor for Worker {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(text) = msg.downcast_ref::<String>() {
            if text.starts_with("block") {
                self.1.hold_until_released();
            }
            if text.ends_with("stop") {
                ctx.stop(&ctx.self_ref()); // `Terminate` is queued before this turn takes another message
            }
            self.0.lock().unwrap().push(text.clone());
        }
        true
    }
}

/// `Error`s and dead letters, and the `Terminated` of the actor it is told to watch.
struct Observer(Log);
impl Actor for Observer {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let seen = if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
            let message = error.message.as_deref().unwrap_or("null");
            format!("{}|{}|{message}", error.log_source, error.cause.class)
        } else if let Some(actor) = msg.downcast_ref::<ActorRef>() {
            ctx.watch(actor);
            "watching".to_owned()
        } else if msg.downcast_ref::<Terminated>().is_some() {
            "terminated".to_owned()
        } else if msg.downcast_ref::<DeadLetter>().is_some() {
            "dead letter".to_owned()
        } else {
            return false;
        };
        self.0.lock().unwrap().push(seen);
        true
    }
}

struct Scenario {
    _system: RunningSystem,
    actor: ActorRef,
    observer: ActorRef,
    log: Log,
    observed: Log,
    gate: Arc<Gate>,
}

impl Scenario {
    fn new(name: &str, fault: Fault) -> Scenario {
        let system = RunningSystem::create(name);
        let (log, observed, gate) = (Log::default(), Log::default(), Arc::new(Gate::default()));
        let observer = system
            .actor_of(Props::new({
                let observed = observed.clone();
                move || Observer(observed.clone())
            }))
            .unwrap();
        system.event_stream().subscribe(&observer, Channel::Error);
        system
            .event_stream()
            .subscribe(&observer, Channel::DeadLetter);
        let props = Props::new({
            let (log, gate) = (log.clone(), gate.clone());
            move || Worker(log.clone(), gate.clone())
        });
        let actor = system
            .actor_of_named(props.with_mailbox(FaultyMailbox(fault)), "a")
            .unwrap();
        Scenario {
            _system: system,
            actor,
            observer,
            log,
            observed,
            gate,
        }
    }

    fn tell(&self, text: &str) {
        self.actor.tell(text.to_owned(), None);
    }

    fn wait_for_log(&self, expected: &[&str]) {
        wait_until("the actor's messages", || {
            snapshot_of(&self.log) == expected
        });
    }

    fn wait_for_observed(&self, expected: &[&str]) {
        wait_until("the observer", || {
            snapshot_of(&self.observed).len() >= expected.len()
        });
        assert_eq!(snapshot_of(&self.observed), expected);
    }
}

#[test]
fn an_enqueue_that_throws_does_not_reach_the_caller_of_tell() {
    let scenario = Scenario::new("tq-enqueue", Fault::Enqueue);
    scenario.tell("a");
    scenario.wait_for_log(&["a"]); // the actor is started: the next sends reach its queue on this thread
    scenario.tell("bad");
    scenario.tell("c");
    scenario.wait_for_log(&["a", "c"]);
    scenario.wait_for_observed(&[
        "pekko://tq-enqueue/user/a|java.lang.IllegalStateException|swallowing exception during message send",
    ]);
}

#[test]
fn a_clean_up_that_throws_does_not_stop_the_termination() {
    let scenario = Scenario::new("tq-cleanup", Fault::CleanUp);
    scenario.observer.tell(scenario.actor.clone(), None);
    scenario.wait_for_observed(&["watching"]);
    scenario.tell("block-then-stop");
    wait_until("the gate", || scenario.gate.entered());
    scenario.tell("pending");
    scenario.gate.release();
    // what was queued is lost: no dead letter, no `Error`
    wait_until("Terminated", || {
        snapshot_of(&scenario.observed).contains(&"terminated".to_owned())
    });
    common::settle_for_absence();
    assert_eq!(snapshot_of(&scenario.observed), ["watching", "terminated"]);
    // `RunningSystem` then requires the system to terminate normally
}
