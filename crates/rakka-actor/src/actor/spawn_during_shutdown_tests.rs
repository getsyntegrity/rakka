// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:300-326 (Apache-2.0).
//! "reliable deny creation of actors while shutting down" (AC-SYS-007), ported 1:1. Integration with real actors, in the crate
//! because the spec accepts only actors that never started (`UnstartedCell`), which the public API does not tell. The one substituted
//! collaborator is `scheduleOnce`: the port's scheduler only sends messages, so a thread terminates the system after 100 ms.

use std::thread;
use std::time::{Duration, Instant};

use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::ActorSystem;
use crate::actor::{ActorRef, ErrorClass};

struct Terminator;

impl Actor for Terminator {
    fn receive(&mut self, ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        ctx.stop(&ctx.self_ref());
        true
    }
}

#[test]
fn creation_of_actors_is_reliably_denied_while_shutting_down() {
    let system = ActorSystem::create("deny-creation").unwrap();
    let uptime = Instant::now();
    let timer = {
        let system = system.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            system.terminate();
        })
    };
    let mut failing = false;
    let mut created: Vec<ActorRef> = Vec::new();
    while !system.when_terminated().is_completed() {
        match system.actor_of(Props::new(|| Terminator)) {
            Ok(actor) => {
                assert!(!failing, "once failing, always failing");
                created.push(actor);
                if created.len().is_multiple_of(1000) {
                    thread::sleep(Duration::from_millis(50));
                }
            }
            Err(error) => {
                assert_eq!(error.class(), ErrorClass::IllegalState);
                failing = true;
            }
        }
        assert!(
            failing || uptime.elapsed() < Duration::from_secs(10),
            "the system did not terminate in time"
        );
    }
    timer.join().unwrap();
    assert!(!created.is_empty(), "something was created before");
    let live: Vec<&ActorRef> = created
        .iter()
        .filter(|actor| {
            !actor.is_terminated()
                && actor
                    .0
                    .as_repointable()
                    .is_none_or(|unstarted| unstarted.is_started())
        })
        .collect();
    assert!(live.is_empty(), "started and not terminated: {live:?}");
}
