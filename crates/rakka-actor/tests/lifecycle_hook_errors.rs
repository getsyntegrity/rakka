// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! The `Error` of a throwing hook precedes the new instance, and a subscriber that panics wedges neither restart nor termination.

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use rakka_actor::event::{Channel, ErrorEvent, WarningEvent};

struct Probe(Sender<String>);

impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let seen = if let Some(e) = msg.downcast_ref::<ErrorEvent>() {
            format!("{}|{:?}", e.cause, e.message)
        } else if let Some(w) = msg.downcast_ref::<WarningEvent>() {
            format!("warning|{:?}", w.message)
        } else {
            (*msg.downcast_ref::<&str>().unwrap()).to_owned()
        };
        self.0.send(seen).unwrap();
        true
    }
}

/// A mailbox that panics on every enqueue.
struct Panics;

#[rustfmt::skip] // one stub per line
impl MailboxType for Panics {
    fn create(&self, _: &ActorRef) -> Box<dyn MessageQueue> { Box::new(Panics) }
}

#[rustfmt::skip]
impl MessageQueue for Panics {
    fn enqueue(&self, _: &ActorRef, _: Envelope) -> bool { panic!("subscriber mailbox") }
    fn dequeue(&self) -> Option<Envelope> { None }
    fn number_of_messages(&self) -> usize { 0 }
    fn has_messages(&self) -> bool { false }
    fn clean_up(&self, _: &ActorRef, _: &dyn MessageQueue) {}
}

/// What the send of an event to a subscriber with that mailbox publishes (oracle case `MBX-TQ-05`).
const SWALLOWED: &str = "java.lang.RuntimeException: subscriber mailbox|Some(\"swallowing exception during message send\")";

/// Fails on any message, and again in `preRestart` and `postStop`.
struct Failing(ActorRef);

impl Actor for Failing {
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        throw(Throwable::illegal_state("hook"));
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.tell("restarted", None);
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        throw(Throwable::illegal_state("stop"));
    }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::illegal_state("boom"));
    }
}

#[test]
fn a_throwing_pre_restart_is_published_and_then_the_restart_goes_on() {
    let system = ActorSystem::create("t-hooks").unwrap();
    let (sender, seen) = channel();
    let probe = system
        .actor_of(Props::new(move || Probe(sender.clone())))
        .unwrap();
    system.event_stream().subscribe(&probe, Channel::Error);
    let broken = Props::new(|| Probe(channel().0)).with_mailbox(Panics);
    let broken = system.actor_of(broken).unwrap();
    system.event_stream().subscribe(&broken, Channel::Error); // after the probe, which still hears
    let failing = Props::new(move || Failing(probe.clone()));
    let actor = system.actor_of_named(failing, "c").unwrap();
    actor.tell("go", None);
    let next = || seen.recv_timeout(Duration::from_secs(10)).expect("event");
    // the guardian's `logFailure` comes before the restart it decides
    assert_eq!(
        next(),
        "java.lang.IllegalStateException: boom|Some(\"boom\")"
    );
    assert_eq!(next(), SWALLOWED);
    assert_eq!(
        next(),
        "org.apache.pekko.actor.PreRestartException: pekko://t-hooks/user/c: exception in preRestart(class \
         java.lang.IllegalStateException, class java.lang.String)|Some(\"hook\")"
    );
    assert_eq!(next(), SWALLOWED);
    assert_eq!(next(), "restarted");
    system.stop(&actor);
    assert!(next().ends_with("Some(\"stop\")"));
    let stopped = system.terminate().wait(Duration::from_secs(10));
    assert!(stopped.is_some(), "termination wedged");
}

/// Throws an exception without a message.
struct Silent;

impl Actor for Silent {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::new("java.lang.IllegalStateException", None));
    }
}

/// Resumes its child, a [`Silent`], and hands it every message.
struct Resumes(Option<ActorRef>);

impl Actor for Resumes {
    fn pre_start(&mut self, context: &mut ActorContext) {
        self.0 = context.actor_of_named(Props::new(|| Silent), "c").ok();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        OneForOneStrategy::new(-1, |_| Directive::Resume).into()
    }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        self.0.as_ref().unwrap().tell("go", None);
        true
    }
}

/// AC-SUP-018: the text of `logFailure`'s events is `getMessage`, which may be `null` (`FaultHandling.scala:406-425`), and a
/// subscriber that panics does not stop the directive (the second failure shows the child was resumed).
#[test]
fn the_failure_of_an_exception_without_a_message_is_published_without_one() {
    let system = ActorSystem::create("t-no-message").unwrap();
    let (sender, seen) = channel();
    let probe = system.actor_of(Props::new(move || Probe(sender.clone())));
    let broken = system.actor_of(Props::new(|| Silent).with_mailbox(Panics));
    let (probe, broken) = (probe.unwrap(), broken.unwrap());
    for channel in [Channel::Error, Channel::Warning] {
        system.event_stream().subscribe(&probe, channel);
        system.event_stream().subscribe(&broken, channel);
    }
    let next = || seen.recv_timeout(Duration::from_secs(10)).expect("event");
    system
        .actor_of(Props::new(|| Silent))
        .unwrap()
        .tell("go", None);
    assert_eq!(next(), "java.lang.IllegalStateException|None");
    assert_eq!(next(), SWALLOWED);
    let parent = system.actor_of(Props::new(|| Resumes(None))).unwrap();
    for _ in 0..2 {
        parent.tell("go", None);
        assert_eq!(next(), "warning|None");
        assert_eq!(next(), SWALLOWED);
    }
}
