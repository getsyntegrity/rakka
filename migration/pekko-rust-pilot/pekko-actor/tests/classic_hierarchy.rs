//! Slice 3a through the public API: children, name reuse, reference identity, DeathWatch.
//! Expected values were observed from Pekko 2.0.0 (see `cases/scenario-3a.jsonl`, `rust/app-cases/`).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use pekko_actor::actor_ref::ActorRef;
use pekko_actor::error::ErrorClass;
use pekko_actor::props::{Actor, ActorContext, AnyMessage, Props};
use pekko_actor::system::ActorSystem;
use pekko_actor::Terminated;

fn path(r: &ActorRef) -> String {
    String::from_utf16_lossy(&r.path().to_jstring())
}

fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// What a `Probe` saw: strings it received and the `Terminated` messages (actor, existenceConfirmed).
#[derive(Default)]
struct Seen {
    msgs: Vec<String>,
    terminated: Vec<(ActorRef, bool)>,
}
type Shared = Arc<Mutex<Seen>>;

/// Executes `Cmd`s inside its own context and records strings and `Terminated`.
struct Probe(Shared);

enum Cmd {
    Spawn(&'static str, Arc<Mutex<Option<Result<ActorRef, ErrorClass>>>>),
    Watch(ActorRef),
    Unwatch(ActorRef),
    Stop(ActorRef),
    Child(&'static str, Arc<Mutex<Option<Option<ActorRef>>>>),
}

impl Actor for Probe {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(t) = msg.downcast_ref::<Terminated>() {
            self.0.lock().unwrap().terminated.push((t.actor.clone(), t.existence_confirmed));
        } else if let Some(m) = msg.downcast_ref::<String>() {
            self.0.lock().unwrap().msgs.push(m.clone());
        } else if let Some(c) = msg.downcast_ref::<Cmd>() {
            match c {
                Cmd::Spawn(name, out) => {
                    let s = self.0.clone();
                    let r = ctx.actor_of_named(Props::new(move || Probe(s.clone())), name).map_err(|e| e.class);
                    *out.lock().unwrap() = Some(r);
                }
                Cmd::Watch(a) => ctx.watch(a),
                Cmd::Unwatch(a) => ctx.unwatch(a),
                Cmd::Stop(a) => ctx.stop(a),
                Cmd::Child(n, out) => *out.lock().unwrap() = Some(ctx.child(n)),
            }
        } else {
            return false;
        }
        true
    }
}

fn probe(sys: &ActorSystem, name: &str) -> (ActorRef, Shared) {
    let seen = Shared::default();
    let s = seen.clone();
    (sys.actor_of_named(Props::new(move || Probe(s.clone())), name).unwrap(), seen)
}

fn spawn_child(parent: &ActorRef, name: &'static str) -> Result<ActorRef, ErrorClass> {
    let out = Arc::new(Mutex::new(None));
    parent.tell(Cmd::Spawn(name, out.clone()), None);
    wait_until("spawn result", || out.lock().unwrap().is_some());
    out.lock().unwrap().take().unwrap()
}

fn child_of(parent: &ActorRef, name: &'static str) -> Option<ActorRef> {
    let out = Arc::new(Mutex::new(None));
    parent.tell(Cmd::Child(name, out.clone()), None);
    wait_until("child result", || out.lock().unwrap().is_some());
    out.lock().unwrap().take().unwrap()
}

/// A command round trip through the actor's own mailbox: everything told to it before has been processed.
fn barrier(a: &ActorRef) {
    child_of(a, "-barrier-");
}

#[test]
fn sibling_collision_and_same_name_under_two_parents() {
    let sys = ActorSystem::create("t-hier").unwrap();
    let ((p1, _), (p2, _)) = (probe(&sys, "p1"), probe(&sys, "p2"));
    let c = spawn_child(&p1, "c").unwrap();
    assert_eq!(path(&c), "pekko://t-hier/user/p1/c");
    assert!(matches!(spawn_child(&p1, "c"), Err(ErrorClass::InvalidActorName)));
    assert_eq!(path(&spawn_child(&p2, "c").unwrap()), "pekko://t-hier/user/p2/c");
    assert_eq!(child_of(&p1, "c"), Some(c));
    assert_eq!(child_of(&p1, "nope"), None);
    sys.terminate().wait(Duration::from_secs(10)).unwrap();
}

#[test]
fn reused_name_has_equal_path_but_is_a_different_reference() {
    let sys = ActorSystem::create("t-reuse").unwrap();
    let (p, seen) = probe(&sys, "p");
    let old = spawn_child(&p, "c").unwrap();
    // the parent watches the child: its own Terminated confirms that the name was released
    p.tell(Cmd::Watch(old.clone()), None);
    p.tell(Cmd::Stop(old.clone()), None);
    wait_until("Terminated(old) in the parent", || seen.lock().unwrap().terminated.len() == 1);
    let new = spawn_child(&p, "c").unwrap();
    assert_eq!(old.path().elements, new.path().elements);
    assert_ne!(old.path().uid, new.path().uid);
    assert_ne!(old, new, "equality is path + uid");
    assert_eq!(child_of(&p, "c"), Some(new.clone()));
    // a stale reference neither reaches nor stops the new actor
    p.tell(Cmd::Stop(old.clone()), None);
    new.tell("alive".to_string(), None);
    wait_until("message to the new actor", || seen.lock().unwrap().msgs.len() == 1);
    assert!(!new.is_terminated());
    sys.terminate().wait(Duration::from_secs(10)).unwrap();
}

#[test]
fn watching_a_dead_actor_notifies_each_watch_without_confirming_existence() {
    let sys = ActorSystem::create("t-watch").unwrap();
    let ((w0, s0), (w, s), (t, _)) = (probe(&sys, "w0"), probe(&sys, "w"), probe(&sys, "t"));
    w0.tell(Cmd::Watch(t.clone()), None);
    barrier(&w0); // the watch is in place before the actor dies
    sys.stop(&t);
    wait_until("Terminated(t) in w0", || s0.lock().unwrap().terminated.len() == 1);
    assert!(s0.lock().unwrap().terminated[0].1, "watched while alive: existenceConfirmed");
    // Pekko: the first Terminated clears the watch, so the second `watch` of the dead actor gets its own
    // (if both were queued before the first Terminated was delivered, Pekko's `terminatedQueued` would collapse them into one)
    w.tell(Cmd::Watch(t.clone()), None);
    wait_until("first Terminated(t) in w", || s.lock().unwrap().terminated.len() == 1);
    w.tell(Cmd::Watch(t.clone()), None);
    wait_until("second Terminated(t) in w", || s.lock().unwrap().terminated.len() == 2);
    w.tell("barrier".to_string(), None);
    wait_until("barrier", || !s.lock().unwrap().msgs.is_empty());
    let seen = s.lock().unwrap();
    assert_eq!(seen.terminated.len(), 2);
    assert!(seen.terminated.iter().all(|(a, confirmed)| *a == t && !confirmed), "watched after death: existenceConfirmed = false");
    drop(seen);
    sys.terminate().wait(Duration::from_secs(10)).unwrap();
}

#[test]
fn unwatch_before_stop_means_no_terminated() {
    let sys = ActorSystem::create("t-unwatch").unwrap();
    let ((w, s), (w0, s0), (t, _)) = (probe(&sys, "w"), probe(&sys, "w0"), probe(&sys, "t"));
    w.tell(Cmd::Watch(t.clone()), None);
    w0.tell(Cmd::Watch(t.clone()), None);
    w.tell(Cmd::Unwatch(t.clone()), None);
    barrier(&w);
    barrier(&w0);
    sys.stop(&t);
    wait_until("Terminated(t) in w0", || !s0.lock().unwrap().terminated.is_empty());
    w.tell("ping".to_string(), None);
    wait_until("ping", || !s.lock().unwrap().msgs.is_empty());
    assert!(s.lock().unwrap().terminated.is_empty());
    sys.terminate().wait(Duration::from_secs(10)).unwrap();
}

#[test]
fn terminated_names_the_watched_instance_when_the_path_is_reused() {
    let sys = ActorSystem::create("t-ident").unwrap();
    let (p, seen) = probe(&sys, "p");
    let t1 = spawn_child(&p, "t").unwrap();
    p.tell(Cmd::Watch(t1.clone()), None);
    p.tell(Cmd::Stop(t1.clone()), None);
    wait_until("first Terminated", || seen.lock().unwrap().terminated.len() == 1);
    let t2 = spawn_child(&p, "t").unwrap();
    // watching the dead predecessor from the parent: immediate Terminated for t1, t2 stays alive
    p.tell(Cmd::Watch(t1.clone()), None);
    wait_until("second Terminated", || seen.lock().unwrap().terminated.len() == 2);
    {
        let s = seen.lock().unwrap();
        assert_eq!(s.terminated[1].0, t1);
        assert_ne!(s.terminated[1].0, t2);
        assert!(!s.terminated[1].1);
    }
    assert_eq!(child_of(&p, "t"), Some(t2));
    sys.terminate().wait(Duration::from_secs(10)).unwrap();
}
