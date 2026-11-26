// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! The default `preRestart` unwatches the children before stopping them; an override that only stops them keeps the watch.
//! The expectations are those of the cases `ur-*` recorded from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, owned_strings, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::dispatch::Terminated;

type Log = Arc<Mutex<Vec<String>>>;

#[derive(Clone, Copy)]
enum Watching {
    Plain,
    With(&'static str),
}

struct Child;
impl Actor for Child {}

/// Creates and watches one child on "watch", waits on the gate on "hold", fails on "boom", records the rest. It does not override `pre_restart`.
#[derive(Clone)]
struct Parent {
    watching: Watching,
    log: Log,
    gate: Arc<Gate>,
}

impl Actor for Parent {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<Terminated>() {
            self.log.lock().unwrap().push("terminated".to_owned());
            return true;
        }
        match msg.downcast_ref::<&str>().copied() {
            Some("watch") => {
                let child = ctx.actor_of(Props::new(|| Child)).unwrap();
                match self.watching {
                    Watching::Plain => ctx.watch(&child),
                    Watching::With(message) => ctx.watch_with(&child, message),
                }
            }
            Some("hold") => self.gate.hold_until_released(),
            Some("boom") => throw(Throwable::illegal_state("boom")),
            Some(other) => self.log.lock().unwrap().push(other.to_owned()),
            None => return false,
        }
        true
    }
}

/// The same parent with an override that stops its children without calling the default.
struct StopsOnly(Parent);

impl Actor for StopsOnly {
    fn pre_restart(&mut self, ctx: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        for child in ctx.children() {
            ctx.stop(&child);
        }
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0.receive(ctx, msg)
    }
}

/// What the parent records for "barrier" (queued behind "boom" before the failure happens) and "ping" (sent after the restart).
fn restarted(system_name: &str, watching: Watching, overrides: bool) -> Vec<String> {
    let system = RunningSystem::create(system_name);
    let (log, gate) = (Log::default(), Arc::<Gate>::default());
    let parent = Parent {
        watching,
        log: log.clone(),
        gate: gate.clone(),
    };
    let props = if overrides {
        Props::new(move || StopsOnly(parent.clone()))
    } else {
        Props::new(move || parent.clone())
    };
    let parent = system.actor_of(props).unwrap();
    parent.tell("watch", None);
    // the parent is held, so "barrier" is in the mailbox before the restart can queue anything about the child
    parent.tell("hold", None);
    parent.tell("boom", None);
    parent.tell("barrier", None);
    gate.release();
    wait_until("the restarted parent", || !snapshot_of(&log).is_empty());
    parent.tell("ping", None);
    wait_until("ping", || snapshot_of(&log).iter().any(|e| e == "ping"));
    snapshot_of(&log)
}

#[test]
fn the_default_pre_restart_cancels_the_watch_of_the_children() {
    let seen = restarted("t-unwatch", Watching::Plain, false);
    assert_eq!(seen, owned_strings(&["barrier", "ping"]));
}

#[test]
fn the_default_pre_restart_cancels_the_watch_with_of_the_children() {
    let seen = restarted("t-unwatch-with", Watching::With("gone"), false);
    assert_eq!(seen, owned_strings(&["barrier", "ping"]));
}

#[test]
fn an_override_that_only_stops_the_children_is_still_told() {
    let seen = restarted("t-keeps-watch", Watching::Plain, true);
    assert_eq!(seen, owned_strings(&["barrier", "terminated", "ping"]));
}
