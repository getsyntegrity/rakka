// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! An actor stopped while its restart waits for a child gets `post_stop` on the instance it was replacing, with no current
//! message, before its watchers are told; no new instance is created. The expectations are those of the cases `mr-*`
//! recorded from Pekko 2.0.0. Every wait is bounded.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::dispatch::Terminated;

#[derive(Clone)]
struct Shared {
    events: Sender<String>,
    /// held by the test: the child cannot finish stopping, so the restart waits
    gate: Arc<Mutex<()>>,
    instances: Arc<AtomicU32>,
    /// `pre_restart` stops the child and does not call `post_stop`
    only_stops_children: bool,
}

impl Shared {
    fn report(&self, event: impl Into<String>) {
        self.events.send(event.into()).unwrap();
    }
}

struct Lingering(Shared);

impl Actor for Lingering {
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.0.report("child stopping");
        drop(self.0.gate.lock());
    }
}

struct Restarting(Shared, u32);

impl Actor for Restarting {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let shared = self.0.clone();
        let props = Props::new(move || Lingering(shared.clone()));
        ctx.actor_of(props).unwrap();
    }
    fn pre_restart(&mut self, ctx: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        for child in ctx.children() {
            ctx.stop(&child);
        }
        if !self.0.only_stops_children {
            self.post_stop(ctx);
        }
    }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) {
        self.0.report("restarted");
    }
    fn post_stop(&mut self, ctx: &mut ActorContext) {
        let sender = ctx.sender().path().to_string();
        let instance = self.1;
        self.0
            .report(format!("post_stop of instance {instance} from {sender}"));
    }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::illegal_state("boom"));
    }
}

/// Parent and watcher of the actor under test: "fail" makes it fail with the parent as sender, "stop" stops it.
struct Parent(Shared);

impl Actor for Parent {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let shared = self.0.clone();
        let props = Props::new(move || {
            let instance = shared.instances.fetch_add(1, Ordering::SeqCst) + 1;
            Restarting(shared.clone(), instance)
        });
        let child = ctx.actor_of_named(props, "r").unwrap();
        ctx.watch(&child);
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<Terminated>() {
            self.0.report("terminated");
            return true;
        }
        let child = ctx.child("r").unwrap();
        if msg.downcast_ref::<&str>() == Some(&"fail") {
            child.tell("boom", Some(&ctx.self_ref()));
        } else {
            ctx.stop(&child);
            self.0.report("stop sent");
        }
        true
    }
}

fn expect(events: &Receiver<String>, event: &str) {
    let seen = events.recv_timeout(Duration::from_secs(10));
    assert_eq!(seen.as_deref(), Ok(event));
}

/// Fails the actor, stops it while its restart waits, lets the child go and checks what follows.
fn stop_during_restart(name: &str, only_stops_children: bool) {
    let system = RunningSystem::create(name);
    let (events, seen) = channel();
    let shared = Shared {
        events,
        gate: Arc::default(),
        instances: Arc::default(),
        only_stops_children,
    };
    let gate = shared.gate.clone();
    let held = gate.lock().unwrap();
    let parent = {
        let shared = shared.clone();
        let props = Props::new(move || Parent(shared.clone()));
        system.actor_of_named(props, "p").unwrap()
    };
    parent.tell("fail", None);
    let first = format!("post_stop of instance 1 from pekko://{name}/user/p");
    let mut before = vec!["child stopping".to_owned()];
    if !only_stops_children {
        before.push(first);
    }
    // the two come from different actors: either may be first
    let mut waiting: Vec<String> = (0..before.len())
        .map(|_| seen.recv_timeout(Duration::from_secs(10)).unwrap())
        .collect();
    waiting.sort();
    before.sort();
    assert_eq!(waiting, before);
    parent.tell("stop", None);
    // the `Terminate` is queued ahead of the news that the child died
    expect(&seen, "stop sent");
    drop(held);
    expect(
        &seen,
        &format!("post_stop of instance 1 from pekko://{name}/deadLetters"),
    );
    expect(&seen, "terminated");
    assert_eq!(shared.instances.load(Ordering::SeqCst), 1);
    assert!(seen.try_recv().is_err(), "nothing restarted");
}

#[test]
fn an_actor_stopped_while_it_restarts_stops_the_instance_it_was_replacing() {
    stop_during_restart("t-stop-restart", false);
}

#[test]
fn the_stop_calls_post_stop_even_if_pre_restart_did_not() {
    stop_during_restart("t-stop-restart-kept", true);
}
