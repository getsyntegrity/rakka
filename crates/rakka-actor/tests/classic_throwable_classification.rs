//! Q-SUP-1: deciders classify a `Throwable` by its subtypes. Tree: `g` -> `p` (default strategy) -> `c`, which throws what it is told.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{
    Directive, OneForOneStrategy, SupervisorStrategy, default_decider,
};

type Log = Arc<Mutex<Vec<String>>>;
type Slot = Arc<Mutex<Option<ActorRef>>>;

fn thrown(kind: &str) {
    let class = |superclass: &str| {
        Throwable::new(format!("a.{kind}"), Some(kind)).with_superclasses([superclass])
    };
    match kind {
        "panic" => panic!("boom"),
        "unknown" => throw(Throwable::new("a.unknown", Some(kind))),
        "argument" => throw(class("java.lang.IllegalArgumentException")),
        "killed" => throw(class("org.apache.pekko.actor.ActorKilledException")),
        _ => throw(class("java.lang.Error")),
    }
}

struct Kid(Log);

impl Actor for Kid {
    fn pre_start(&mut self, _c: &mut ActorContext) {
        self.0.lock().unwrap().push("c started".into());
    }
    fn post_stop(&mut self, _c: &mut ActorContext) {
        self.0.lock().unwrap().push("c stopped".into());
    }
    fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
        let kind = msg.downcast_ref::<&str>().expect("a kind");
        if *kind != "ping" {
            thrown(kind);
        }
        self.0.lock().unwrap().push("c ping".into());
        true
    }
}

/// `p`: the default strategy; creates `c`.
struct Parent {
    log: Log,
    kid: Slot,
}

impl Actor for Parent {
    fn pre_start(&mut self, c: &mut ActorContext) {
        let log = self.log.clone();
        let kid = c.actor_of_named(Props::new(move || Kid(log.clone())), "c");
        *self.kid.lock().unwrap() = Some(kid.unwrap());
    }
}

/// `g`: records what reaches its strategy; creates `p`.
struct Top {
    log: Log,
    kid: Slot,
    strategy: SupervisorStrategy,
}

impl Actor for Top {
    fn pre_start(&mut self, c: &mut ActorContext) {
        let (log, kid) = (self.log.clone(), self.kid.clone());
        let parent = Props::new(move || Parent {
            log: log.clone(),
            kid: kid.clone(),
        });
        c.actor_of_named(parent, "p").unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        self.strategy.clone()
    }
}

struct Tree {
    _system: RunningSystem,
    log: Log,
    kid: ActorRef,
}

/// A tree whose `g` applies `decide` and records the class it was asked about.
fn tree(decide: impl Fn(&Throwable) -> Directive + Send + Sync + 'static) -> Tree {
    let system = RunningSystem::create("sys");
    let (log, slot) = (Log::default(), Slot::default());
    let decisions = log.clone();
    let strategy: SupervisorStrategy = OneForOneStrategy::new(-1, move |cause: &Throwable| {
        let directive = decide(cause);
        let line = format!("g {} {directive:?}", cause.class);
        decisions.lock().unwrap().push(line);
        directive
    })
    .into();
    let (top_log, top_slot) = (log.clone(), slot.clone());
    let top = Props::new(move || Top {
        log: top_log.clone(),
        kid: top_slot.clone(),
        strategy: strategy.clone(),
    });
    system.actor_of_named(top, "g").unwrap();
    wait_until("c started", || snapshot_of(&log) == ["c started"]);
    let kid = slot.lock().unwrap().clone().unwrap();
    Tree {
        _system: system,
        log,
        kid,
    }
}

/// Sends `kind` and then `ping`, and waits for the log to reach `expected` (after the first `c started`).
fn throwing(tree: &Tree, kind: &'static str, expected: &[&str]) {
    tree.kid.tell(kind, None);
    tree.kid.tell("ping", None);
    wait_until(kind, || snapshot_of(&tree.log).len() > expected.len());
    assert_eq!(snapshot_of(&tree.log)[1..], *expected, "{kind}");
}

/// A restart under the default strategy: `postStop` of the old instance, `preStart` of the new one, and the mailbox goes on.
const RESTARTED: [&str; 3] = ["c stopped", "c started", "c ping"];

#[test]
fn unknown_and_derived_exceptions_and_panics_restart_the_child() {
    for kind in ["unknown", "argument", "panic"] {
        let t = tree(default_decider);
        throwing(&t, kind, &RESTARTED);
    }
}

#[test]
fn a_subtype_of_a_stopping_exception_stops_the_child() {
    let t = tree(default_decider);
    throwing(&t, "killed", &["c stopped"]);
}

/// `p` has no directive for a non-`Exception`, so it fails with it and `g` decides about `p`.
#[test]
fn an_error_escalates_to_the_grandparent() {
    let t = tree(|_| Directive::Resume);
    throwing(&t, "assert", &["g a.assert Resume", "c ping"]);
}

#[test]
fn a_custom_decider_matches_by_superclass() {
    let t = tree(|cause| {
        if cause.is_a("java.lang.Error") {
            Directive::Resume
        } else {
            Directive::Stop
        }
    });
    throwing(&t, "assert", &["g a.assert Resume", "c ping"]);
}
