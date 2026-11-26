// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! The `Error` of a throwing hook precedes the new instance, and a subscriber that panics wedges neither restart nor termination.

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use rakka_actor::event::{Channel, ErrorEvent};

struct Probe(Sender<String>);

impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let seen = match msg.downcast_ref::<ErrorEvent>() {
            Some(e) => format!("{}|{:?}", e.cause, e.message),
            None => (*msg.downcast_ref::<&str>().unwrap()).to_owned(),
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
    assert_eq!(
        next(),
        "org.apache.pekko.actor.PreRestartException: pekko://t-hooks/user/c: exception in preRestart(class \
         java.lang.IllegalStateException, class java.lang.String)|Some(\"hook\")"
    );
    assert_eq!(next(), "restarted");
    system.stop(&actor);
    assert!(next().ends_with("Some(\"stop\")"));
    let stopped = system.terminate().wait(Duration::from_secs(10));
    assert!(stopped.is_some(), "termination wedged");
}

struct Silent;

impl Actor for Silent {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::new("java.lang.IllegalStateException", None));
    }
}

/// AC-SUP-018: the `Error` of `logFailure` carries `getMessage`, which may be `null` (`FaultHandling.scala:406-425`).
#[test]
fn the_failure_of_an_exception_without_a_message_is_published_without_one() {
    let system = ActorSystem::create("t-no-message").unwrap();
    let (sender, seen) = channel();
    let probe = system.actor_of(Props::new(move || Probe(sender.clone())));
    system
        .event_stream()
        .subscribe(&probe.unwrap(), Channel::Error);
    system
        .actor_of(Props::new(|| Silent))
        .unwrap()
        .tell("go", None);
    let published = seen.recv_timeout(Duration::from_secs(10)).expect("event");
    assert_eq!(published, "java.lang.IllegalStateException|None");
}
