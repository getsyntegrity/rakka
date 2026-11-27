// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:136-171, dispatch/Mailbox.scala:262-272 and actor/ActorSystem.scala:875-880 (Apache-2.0).
//! A user's `MessageQueue` is public SPI and may panic. Expected values: Pekko 2.0.0 (recorded cases come in a follow-up change).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, snapshot_of, wait_until};

use std::sync::atomic::{AtomicBool, Ordering};
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
    Dequeue,
    HasMessages,
    CleanUp,
    /// `dequeue` throws and so does the `has_messages` that follows it.
    DequeueAndHasMessages,
    /// `enqueue` throws for the `Error` events a subscriber receives.
    EnqueueOfErrors,
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

/// A queue with one operation that throws: `enqueue`/`dequeue` of "bad", the `has_messages` after "bad" was dequeued, or `clean_up`.
struct FaultyQueue {
    inner: Box<dyn MessageQueue>,
    fault: Fault,
    armed: AtomicBool,
}

impl MessageQueue for FaultyQueue {
    fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool {
        if self.fault == Fault::Enqueue && is_bad(&envelope)
            || self.fault == Fault::EnqueueOfErrors && envelope.message.is::<ErrorEvent>()
        {
            boom("enqueue");
        }
        self.inner.enqueue(receiver, envelope)
    }
    fn dequeue(&self) -> Option<Envelope> {
        let envelope = self.inner.dequeue();
        if envelope.as_ref().is_some_and(is_bad) {
            match self.fault {
                Fault::Dequeue => boom("dequeue"),
                Fault::HasMessages => self.armed.store(true, Ordering::SeqCst),
                Fault::DequeueAndHasMessages => {
                    self.armed.store(true, Ordering::SeqCst);
                    boom("dequeue");
                }
                _ => {}
            }
        }
        envelope
    }
    fn number_of_messages(&self) -> usize {
        self.inner.number_of_messages()
    }
    fn has_messages(&self) -> bool {
        if self.armed.swap(false, Ordering::SeqCst) {
            boom("hasMessages");
        }
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
            armed: AtomicBool::new(false),
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
            let thread_number: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ']'];
            let message = message.trim_end_matches(thread_number);
            format!("{}|{}|{message}", error.log_source, error.cause)
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

/// The report of an uncaught error whose cause is the throw of `operation`.
fn uncaught(system: &str, operation: &str) -> String {
    format!(
        "org.apache.pekko.actor.ActorSystemImpl({system})|java.lang.IllegalStateException: {operation} boom|\
         Uncaught error from thread [{system}-pekko.actor.default-dispatcher-"
    )
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
        "pekko://tq-enqueue/user/a|java.lang.IllegalStateException: enqueue boom|swallowing exception during message send",
    ]);
}

#[test]
fn a_dequeue_that_throws_is_reported_as_uncaught_and_the_actor_goes_on() {
    let scenario = Scenario::new("tq-dequeue", Fault::Dequeue);
    scenario.tell("block");
    wait_until("the gate", || scenario.gate.entered());
    for text in ["a", "bad", "c"] {
        scenario.tell(text);
    }
    scenario.gate.release();
    scenario.wait_for_log(&["block", "a", "c"]);
    scenario.tell("d");
    scenario.wait_for_log(&["block", "a", "c", "d"]);
    scenario.wait_for_observed(&[&uncaught("tq-dequeue", "dequeue")]);
}

/// Characterisation of the `finally` of `Mailbox.run`: what `has_messages` throws replaces what the turn threw, one report.
#[test]
fn a_turn_and_its_rescheduling_that_both_throw_report_only_the_second() {
    let scenario = Scenario::new("tq-both", Fault::DequeueAndHasMessages);
    scenario.tell("bad");
    scenario.wait_for_observed(&[&uncaught("tq-both", "hasMessages")]);
    scenario.tell("c");
    scenario.wait_for_log(&["c"]);
    assert_eq!(snapshot_of(&scenario.observed).len(), 1);
}

#[test]
fn the_uncaught_report_to_a_subscriber_that_cannot_take_it_reports_that_to_the_others() {
    let scenario = Scenario::new("tq-subscriber", Fault::Dequeue);
    let (ready, gate) = (Log::default(), Arc::new(Gate::default()));
    let props = Props::new({
        let (ready, gate) = (ready.clone(), gate.clone());
        move || Worker(ready.clone(), gate.clone())
    });
    let broken = scenario
        ._system
        .actor_of_named(
            props.with_mailbox(FaultyMailbox(Fault::EnqueueOfErrors)),
            "broken",
        )
        .unwrap();
    broken.tell("ready".to_owned(), None);
    wait_until("the subscriber to be started", || {
        snapshot_of(&ready) == ["ready"]
    });
    scenario
        ._system
        .event_stream()
        .subscribe(&broken, Channel::Error); // after the observer
    scenario.tell("bad");
    scenario.wait_for_observed(&[
        &uncaught("tq-subscriber", "dequeue"),
        "pekko://tq-subscriber/user/broken|java.lang.IllegalStateException: enqueue boom|swallowing exception during message send",
    ]);
}

#[test]
fn a_has_messages_that_throws_after_a_turn_leaves_the_mailbox_idle() {
    let scenario = Scenario::new("tq-has", Fault::HasMessages);
    scenario.tell("bad");
    scenario.wait_for_observed(&[&uncaught("tq-has", "hasMessages")]);
    scenario.tell("c");
    scenario.wait_for_log(&["bad", "c"]);
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
