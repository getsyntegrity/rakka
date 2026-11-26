// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! The `Error` of a throwing `preRestart` precedes the new instance (AC-CELL-007.1): one probe hears both, an order no case records.

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::ActorSystem;
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

/// Fails on any message, and again in `preRestart`.
struct Failing(ActorRef);

impl Actor for Failing {
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        throw(Throwable::illegal_state("hook"));
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.tell("restarted", None);
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
    let failing = Props::new(move || Failing(probe.clone()));
    let actor = system.actor_of_named(failing, "c").unwrap();
    actor.tell("go", None);
    let next = || seen.recv_timeout(Duration::from_secs(10)).expect("event");
    assert_eq!(
        next(),
        "org.apache.pekko.actor.PreRestartException: pekko://t-hooks/user/c: exception in preRestart(class \
         java.lang.IllegalStateException, class java.lang.String)|Some(\"hook\")"
    );
    assert_eq!(next(), "restarted");
}
