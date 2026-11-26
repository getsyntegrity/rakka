// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala (Apache-2.0).
//! The message the cell holds as `currentMessage`, with a `Drop` that panics: forgetting it never wedges the actor (Rust only).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};

/// Panics when dropped.
struct Fragile;

impl Drop for Fragile {
    fn drop(&mut self) {
        panic!("drop of a user message");
    }
}

type Log = Arc<Mutex<Vec<String>>>;

struct Failing(Log);

impl Actor for Failing {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<Fragile>() {
            throw(Throwable::illegal_state("fragile"));
        }
        self.0.lock().unwrap().push("ping".into());
        true
    }
}

/// Spawns a `Failing` child and answers its failure with `directive`.
struct Parent(Directive, Log);

impl Actor for Parent {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let log = self.1.clone();
        let child = ctx
            .actor_of(Props::new(move || Failing(log.clone())))
            .unwrap();
        child.tell(Fragile, None);
        child.tell("ping", None);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let directive = self.0;
        OneForOneStrategy::new(-1, move |_| directive).into()
    }
}

#[test]
fn a_failed_message_whose_drop_panics_does_not_wedge_the_actor() {
    let system = RunningSystem::create("t-current-message-drop");
    for directive in [Directive::Restart, Directive::Resume] {
        let log = Log::default();
        let seen = log.clone();
        let props = Props::new(move || Parent(directive, seen.clone()));
        system.actor_of(props).unwrap();
        wait_until("the next message", || !log.lock().unwrap().is_empty());
        assert_eq!(snapshot_of(&log), ["ping"], "{directive:?}");
    }
}
