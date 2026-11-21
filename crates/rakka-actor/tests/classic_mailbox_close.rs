//! Closing a mailbox while senders race it (ADR-0001, Owner decision Q-MBX-8: Rust never strands a message).
//!
//! Pekko 2.0.0 can leave a message in the closed queue, neither processed nor dead-lettered: oracle cases CC-05/CC-05a and the
//! `oracle/run.sh mailbox-stress` experiment (docs/evidence/mailbox-oracle.md). Rust closes that window, so with real actors every
//! message sent while an actor stops is either processed or published as a dead letter, exactly once.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, wait_until};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::event::{Channel, DeadLetter};

struct CountsReceived(Arc<AtomicUsize>);
impl Actor for CountsReceived {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<String>() {
            self.0.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

struct CountsDeadLetters(Arc<AtomicUsize>);
impl Actor for CountsDeadLetters {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<DeadLetter>() {
            Some(dead) if dead.message.is::<String>() => {
                self.0.fetch_add(1, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    }
}

#[test]
fn every_message_sent_while_an_actor_stops_is_processed_or_dead_lettered_exactly_once() {
    const TRIALS: usize = 400;
    const PRODUCERS: usize = 4;
    const SENDS_EACH: usize = 300;
    let sys = RunningSystem::create("t-close-race");
    let (received, dead) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let dead_counter = dead.clone();
    let listener = sys
        .actor_of(Props::new(move || CountsDeadLetters(dead_counter.clone())))
        .unwrap();
    sys.event_stream().subscribe(&listener, Channel::DeadLetter);

    for trial in 0..TRIALS {
        let counter = received.clone();
        let target = sys
            .actor_of_named(
                Props::new(move || CountsReceived(counter.clone())),
                &format!("target-{trial}"),
            )
            .unwrap();
        let start = Arc::new(Barrier::new(PRODUCERS + 1));
        let producers: Vec<_> = (0..PRODUCERS)
            .map(|_| {
                let (target, start) = (target.clone(), start.clone());
                thread::spawn(move || {
                    start.wait();
                    for n in 0..SENDS_EACH {
                        target.tell(format!("m{n}"), None);
                    }
                })
            })
            .collect();
        start.wait();
        sys.stop(&target);
        producers.into_iter().for_each(|p| p.join().unwrap());
        let expected = (trial + 1) * PRODUCERS * SENDS_EACH;
        // a stranded message would never be counted: the wait would time out
        wait_until("every message processed or dead-lettered", || {
            received.load(Ordering::SeqCst) + dead.load(Ordering::SeqCst) >= expected
        });
        assert_eq!(
            received.load(Ordering::SeqCst) + dead.load(Ordering::SeqCst),
            expected,
            "trial {trial}: a message was duplicated"
        );
    }
}
