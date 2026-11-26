// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala (Apache-2.0).
//! `context.watchWith` with a user message type whose `Drop` panics: no death-watch path may wedge the actor (Rust only).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, snapshot_of, wait_until};

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, PoisonPill, Props};

/// Equal by name; dropping an armed one panics.
struct Fragile(&'static str, bool);

impl PartialEq for Fragile {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Drop for Fragile {
    fn drop(&mut self) {
        assert!(!self.1, "drop of a termination message");
    }
}

type Log = Arc<Mutex<Vec<String>>>;

struct Watcher(ActorRef, Log, Arc<Gate>);

impl Actor for Watcher {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let command = msg.downcast_ref::<&str>().copied();
        let mut register = |name, armed| ctx.watch_with(&self.0, Fragile(name, armed));
        let seen = match command {
            Some(name @ ("armed" | "calm" | "again")) => {
                register(if name == "again" { "b" } else { "a" }, name != "calm");
                None
            }
            Some("other") => match catch_unwind(AssertUnwindSafe(|| register("b", true))) {
                Ok(()) => Some("accepted"),
                Err(_) => Some("rejected"),
            },
            Some("unwatch") => {
                ctx.unwatch(&self.0);
                None
            }
            Some("block") => {
                self.2.hold_until_released();
                None
            }
            other => Some(other.unwrap_or("delivered")),
        };
        if let Some(seen) = seen {
            self.1.lock().unwrap().push(seen.to_owned());
        }
        true
    }
}

/// Runs `before`, stops the subject while the watcher is blocked with `queued` behind, and returns the watcher and its log.
fn scenario(
    system: &RunningSystem,
    before: &[&'static str],
    queued: &[&'static str],
) -> (ActorRef, Log) {
    let subject = system.actor_of(Props::new(|| Idle)).unwrap();
    let spawn = |log: &Log, gate: &Arc<Gate>| {
        let (subject, log, gate) = (subject.clone(), log.clone(), gate.clone());
        let props = Props::new(move || Watcher(subject.clone(), log.clone(), gate.clone()));
        system.actor_of(props).unwrap()
    };
    let (log, gate, witnessed) = (Log::default(), Arc::<Gate>::default(), Log::default());
    let (watcher, witness) = (spawn(&log, &gate), spawn(&witnessed, &gate));
    for command in before.iter().chain(&["ready", "block"]) {
        watcher.tell(*command, None);
    }
    wait_until("the watcher to block", || gate.entered());
    // the witness watches after the watcher: once it is notified, so was the watcher
    witness.tell("calm", None);
    witness.tell("ready", None);
    wait_until("the witness to watch", || {
        !witnessed.lock().unwrap().is_empty()
    });
    system.stop(&subject);
    wait_until("the witness to be told", || {
        witnessed.lock().unwrap().len() == 2
    });
    for command in queued {
        match *command {
            "poison" => watcher.tell(PoisonPill, None),
            command => watcher.tell(command, None),
        }
    }
    gate.release();
    (watcher, log)
}

struct Idle;
impl Actor for Idle {}

#[test]
fn a_termination_message_whose_drop_panics_does_not_wedge_the_watcher() {
    let system = RunningSystem::create("t-watch-with-drop");

    // the notification is queued: cancelled by `unwatch`, then refused a replacement
    let (_, log) = scenario(&system, &["armed", "other"], &["unwatch", "ping"]);
    wait_until("ping after unwatch", || log.lock().unwrap().len() == 3);
    assert_eq!(snapshot_of(&log), ["rejected", "ready", "ping"]);
    let (_, log) = scenario(&system, &["calm"], &["again", "ping"]);
    wait_until("the first message", || log.lock().unwrap().len() == 3);
    assert_eq!(snapshot_of(&log), ["ready", "ping", "delivered"]);

    // the watcher terminates: with the notification queued, and while still watching
    let (watcher, _) = scenario(&system, &["armed"], &["poison"]);
    wait_until("termination with a queued notification", || {
        watcher.is_terminated()
    });
    let idle = system.actor_of(Props::new(|| Idle)).unwrap();
    let watcher = system.actor_of(Props::new(move || {
        Watcher(idle.clone(), Log::default(), Arc::default())
    }));
    let watcher = watcher.unwrap();
    watcher.tell("armed", None);
    watcher.tell(PoisonPill, None);
    wait_until("termination while watching", || watcher.is_terminated());
}
