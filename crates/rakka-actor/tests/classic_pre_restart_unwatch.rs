// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! The default `preRestart` unwatches the children before stopping them; an override that only stops them keeps the watch, and one that
//! calls `default_pre_restart` behaves as the default (`super.preRestart`).
//! The expectations are those of the cases `ur-*` recorded from Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, owned_strings, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props, default_pre_restart};
use rakka_actor::dispatch::Terminated;

type Log = Arc<Mutex<Vec<String>>>;

#[derive(Clone, Copy)]
enum Watching {
    Plain,
    With(&'static str),
}

struct Child(Log);
impl Actor for Child {
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.0.lock().unwrap().push("child-stopped".to_owned());
    }
}

/// Creates and watches one child on "watch", waits on the gate on "hold", fails on "boom", records the rest. It does not override `pre_restart`.
#[derive(Clone)]
struct Parent {
    watching: Watching,
    log: Log,
    child_log: Log,
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
                let child_log = self.child_log.clone();
                let child = ctx
                    .actor_of(Props::new(move || Child(child_log.clone())))
                    .unwrap();
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

/// The same parent with an override that does its own work before and after calling the default, and its own `post_stop`.
struct CallsDefault(Parent);

impl Actor for CallsDefault {
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        reason: &Throwable,
        message: Option<&AnyMessage>,
    ) {
        self.0.log.lock().unwrap().push("before".to_owned());
        default_pre_restart(self, ctx, reason, message);
        self.0.log.lock().unwrap().push("after".to_owned());
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.0.log.lock().unwrap().push("post-stop".to_owned());
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0.receive(ctx, msg)
    }
}

#[derive(Clone, Copy)]
enum PreRestart {
    Default,
    StopsOnly,
    CallsDefault,
}

/// What the parent records up to "ping" (sent after the restart; "barrier" is queued behind "boom" before the failure happens), and
/// what its child records.
fn restarted(
    system_name: &str,
    watching: Watching,
    pre_restart: PreRestart,
) -> (Vec<String>, Vec<String>) {
    let system = RunningSystem::create(system_name);
    let (log, child_log, gate) = (Log::default(), Log::default(), Arc::<Gate>::default());
    let parent = Parent {
        watching,
        log: log.clone(),
        child_log: child_log.clone(),
        gate: gate.clone(),
    };
    let props = match pre_restart {
        PreRestart::Default => Props::new(move || parent.clone()),
        PreRestart::StopsOnly => Props::new(move || StopsOnly(parent.clone())),
        PreRestart::CallsDefault => Props::new(move || CallsDefault(parent.clone())),
    };
    let parent = system.actor_of(props).unwrap();
    parent.tell("watch", None);
    // the parent is held, so "barrier" is in the mailbox before the restart can queue anything about the child
    parent.tell("hold", None);
    parent.tell("boom", None);
    parent.tell("barrier", None);
    gate.release();
    wait_until("the restarted parent", || {
        snapshot_of(&log).iter().any(|e| e == "barrier")
    });
    parent.tell("ping", None);
    wait_until("ping", || snapshot_of(&log).iter().any(|e| e == "ping"));
    (snapshot_of(&log), snapshot_of(&child_log))
}

#[test]
fn the_default_pre_restart_cancels_the_watch_of_the_children() {
    let seen = restarted("t-unwatch", Watching::Plain, PreRestart::Default).0;
    assert_eq!(seen, owned_strings(&["barrier", "ping"]));
}

#[test]
fn the_default_pre_restart_cancels_the_watch_with_of_the_children() {
    let seen = restarted(
        "t-unwatch-with",
        Watching::With("gone"),
        PreRestart::Default,
    )
    .0;
    assert_eq!(seen, owned_strings(&["barrier", "ping"]));
}

#[test]
fn an_override_that_only_stops_the_children_is_still_told() {
    let seen = restarted("t-keeps-watch", Watching::Plain, PreRestart::StopsOnly).0;
    assert_eq!(seen, owned_strings(&["barrier", "terminated", "ping"]));
}

#[test]
fn an_override_that_calls_the_default_stops_the_children_and_runs_its_own_post_stop_in_place() {
    let (seen, child) = restarted("t-calls-default", Watching::Plain, PreRestart::CallsDefault);
    assert_eq!(
        seen,
        owned_strings(&["before", "post-stop", "after", "barrier", "ping"])
    );
    assert_eq!(child, owned_strings(&["child-stopped"]));
}

#[test]
fn an_override_that_calls_the_default_cancels_the_watch_with_of_the_children() {
    let (seen, _) = restarted(
        "t-calls-default-with",
        Watching::With("gone"),
        PreRestart::CallsDefault,
    );
    assert_eq!(
        seen,
        owned_strings(&["before", "post-stop", "after", "barrier", "ping"])
    );
}
