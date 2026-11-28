// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:300-326 (Apache-2.0).
//! "reliable deny creation of actors while shutting down" (AC-SYS-007). Integration with real actors, in the crate because the
//! spec accepts only actors that never started (`UnstartedCell`), which the public API does not tell. Two collaborators are added
//! to the spec's script: a thread terminates the system after 100 ms (the port's scheduler only sends messages, no `scheduleOnce`
//! of a block), and a `Blocker` holds the termination until a creation was refused, so that the spec cannot pass without one.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// Its `postStop` waits for the first refusal, at most 5 s.
struct Blocker {
    started: Arc<AtomicBool>,
    refused: Arc<AtomicBool>,
}

impl Actor for Blocker {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.started.store(true, Ordering::SeqCst);
    }

    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.refused.load(Ordering::SeqCst) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

#[test]
fn creation_of_actors_is_reliably_denied_while_shutting_down() {
    let system = ActorSystem::create("deny-creation").unwrap();
    let started = Arc::new(AtomicBool::new(false));
    let refused = Arc::new(AtomicBool::new(false));
    let blocker = {
        let (started, refused) = (started.clone(), refused.clone());
        Props::new(move || Blocker {
            started: started.clone(),
            refused: refused.clone(),
        })
    };
    system.actor_of(blocker).unwrap();
    let uptime = Instant::now();
    while !started.load(Ordering::SeqCst) {
        assert!(uptime.elapsed() < Duration::from_secs(5), "no blocker");
        thread::sleep(Duration::from_millis(1));
    }
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
        assert!(
            uptime.elapsed() < Duration::from_secs(10),
            "the system did not terminate in time"
        );
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
                refused.store(true, Ordering::SeqCst);
            }
        }
    }
    timer.join().unwrap();
    assert!(!created.is_empty(), "something was created before");
    assert!(failing, "a creation was refused while shutting down");
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
