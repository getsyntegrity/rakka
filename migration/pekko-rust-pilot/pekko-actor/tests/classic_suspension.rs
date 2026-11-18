//! Slice 3b-4: while an actor that failed on its own waits for its supervisor, its whole subtree is suspended; `Resume` / `Restart` / `Stop` then
//! recover it. Recorded from Pekko 2.0.0 first (`cases/scenario-3b4.jsonl`, `rust/app-cases/`). The supervisor's decision is held pending by a
//! barrier (a `Condvar`), not by a sleep; the only timed step is the bounded wait that proves an absence.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use pekko_actor::actor_ref::ActorRef;
use pekko_actor::fault::{Throwable, throw};
use pekko_actor::props::{Actor, ActorContext, AnyMessage, Props};
use pekko_actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use pekko_actor::system::ActorSystem;

type Log = Arc<Mutex<Vec<String>>>;
type Hold = Arc<(Mutex<bool>, Condvar)>;
type Make = Arc<dyn Fn() -> Box<dyn Actor> + Send + Sync>;

fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn snap(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// proves an absence: `log` still has `len` entries after a bounded wait
fn stays_at(log: &Log, len: usize) -> bool {
    let end = Instant::now() + Duration::from_millis(300);
    while Instant::now() < end {
        if snap(log).len() > len {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    true
}

/// records lifecycle and messages; `boom` makes it fail; `kid` is the name of the child it creates on start (if any)
struct Node {
    log: Log,
    kid: Option<(&'static str, Make)>,
    slot: Arc<Mutex<Option<ActorRef>>>,
}

impl Actor for Node {
    fn pre_start(&mut self, c: &mut ActorContext) {
        self.log.lock().unwrap().push("pre-start".into());
        if let Some((name, make)) = &self.kid
            && self.slot.lock().unwrap().is_none()
        {
            let make = make.clone();
            *self.slot.lock().unwrap() = Some(c.actor_of_named(Props::new(move || BoxActor(make())), name).unwrap());
        }
    }
    fn post_stop(&mut self, _c: &mut ActorContext) {
        self.log.lock().unwrap().push("post-stop".into());
    }
    fn pre_restart(&mut self, c: &mut ActorContext, _r: &Throwable, _m: Option<&AnyMessage>) {
        self.log.lock().unwrap().push("pre-restart".into());
        for k in c.children() {
            c.stop(&k);
        }
        self.post_stop(c);
    }
    fn post_restart(&mut self, c: &mut ActorContext, _r: &Throwable) {
        self.log.lock().unwrap().push("post-restart".into());
        self.pre_start(c);
    }
    fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(m) = msg.downcast_ref::<String>() else { return false };
        self.log.lock().unwrap().push(m.clone());
        if m == "boom" {
            throw(Throwable::illegal_state("boom"));
        }
        true
    }
}

struct BoxActor(Box<dyn Actor>);

impl Actor for BoxActor {
    fn pre_start(&mut self, c: &mut ActorContext) {
        self.0.pre_start(c)
    }
    fn post_stop(&mut self, c: &mut ActorContext) {
        self.0.post_stop(c)
    }
    fn pre_restart(&mut self, c: &mut ActorContext, r: &Throwable, m: Option<&AnyMessage>) {
        self.0.pre_restart(c, r, m)
    }
    fn post_restart(&mut self, c: &mut ActorContext, r: &Throwable) {
        self.0.post_restart(c, r)
    }
    fn receive(&mut self, c: &mut ActorContext, m: &AnyMessage) -> bool {
        self.0.receive(c, m)
    }
}

/// `g`: creates `p`, decides `directive` for `boom` but only after the hold is opened.
struct Top {
    log: Log,
    hold: Hold,
    directive: Directive,
    make_p: Make,
    p_slot: Arc<Mutex<Option<ActorRef>>>,
}

impl Actor for Top {
    fn pre_start(&mut self, c: &mut ActorContext) {
        let make = self.make_p.clone();
        let p = c.actor_of_named(Props::new(move || BoxActor(make())), "p").unwrap();
        c.watch(&p);
        *self.p_slot.lock().unwrap() = Some(p);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (log, hold, directive) = (self.log.clone(), self.hold.clone(), self.directive);
        OneForOneStrategy::new(-1, move |e: &Throwable| {
            log.lock().unwrap().push(format!("decide {e}"));
            let open = hold.0.lock().unwrap();
            drop(hold.1.wait_timeout_while(open, Duration::from_secs(30), |o| !*o).unwrap());
            directive
        }).into()
    }
    fn receive(&mut self, _c: &mut ActorContext, m: &AnyMessage) -> bool {
        if m.is::<pekko_actor::Terminated>() {
            self.log.lock().unwrap().push("terminated p".into());
        }
        true
    }
}

struct World {
    _system: ActorSystem,
    g: Log,
    p: Log,
    c: Log,
    gc: Log,
    c2: Log,
    gc_ref: Option<ActorRef>,
    c2_ref: Option<ActorRef>,
    hold: Hold,
    p_ref: ActorRef,
    c_ref: ActorRef,
}

impl World {
    fn open(&self) {
        *self.hold.0.lock().unwrap() = true;
        self.hold.1.notify_all();
    }
}

/// g -> p (fails on `boom`) -> c (active child) [-> gc (grandchild)] [and a second child c2 of p]
fn world(directive: Directive) -> World {
    world_with(directive, false, false)
}

fn node(log: &Log, kid: Option<(&'static str, Make)>, slot: &Arc<Mutex<Option<ActorRef>>>) -> Make {
    let (log, slot) = (log.clone(), slot.clone());
    Arc::new(move || Box::new(Node { log: log.clone(), kid: kid.clone(), slot: slot.clone() }))
}

fn world_with(directive: Directive, grandchild: bool, second: bool) -> World {
    let system = ActorSystem::create("sys").unwrap();
    let (g, p, c, gc, c2) = (Log::default(), Log::default(), Log::default(), Log::default(), Log::default());
    let hold: Hold = Default::default();
    let (c_slot, gc_slot, c2_slot, p_slot): (Slot, Slot, Slot, Slot) = Default::default();
    let make_gc = node(&gc, None, &Default::default());
    let make_c = node(&c, grandchild.then(|| ("gc", make_gc.clone())), &gc_slot);
    let make_c2 = node(&c2, None, &Default::default());
    // `p` owns c (and c2): a small actor creating both on start
    let make_p: Make = {
        let (p, c_slot, c2_slot) = (p.clone(), c_slot.clone(), c2_slot.clone());
        Arc::new(move || Box::new(Parent { node: Node { log: p.clone(), kid: Some(("c", make_c.clone())), slot: c_slot.clone() }, second: second.then(|| ("c2", make_c2.clone(), c2_slot.clone())) }))
    };
    {
        let (g, hold, make_p, p_slot) = (g.clone(), hold.clone(), make_p.clone(), p_slot.clone());
        system
            .actor_of_named(Props::new(move || Top { log: g.clone(), hold: hold.clone(), directive, make_p: make_p.clone(), p_slot: p_slot.clone() }), "g")
            .unwrap();
    }
    wait_until("children started", || snap(&c) == ["pre-start"] && (!grandchild || snap(&gc) == ["pre-start"]) && (!second || snap(&c2) == ["pre-start"]));
    let get = |s: &Slot| s.lock().unwrap().clone();
    World { _system: system, g, p, c, gc, c2, hold, p_ref: get(&p_slot).unwrap(), c_ref: get(&c_slot).unwrap(), gc_ref: get(&gc_slot), c2_ref: get(&c2_slot) }
}

type Slot = Arc<Mutex<Option<ActorRef>>>;

/// `p`: a `Node` that also creates a second child when asked
struct Parent {
    node: Node,
    second: Option<(&'static str, Make, Slot)>,
}

impl Actor for Parent {
    fn pre_start(&mut self, c: &mut ActorContext) {
        self.node.pre_start(c);
        if let Some((name, make, slot)) = &self.second
            && slot.lock().unwrap().is_none()
        {
            let make = make.clone();
            *slot.lock().unwrap() = Some(c.actor_of_named(Props::new(move || BoxActor(make())), name).unwrap());
        }
    }
    fn post_stop(&mut self, c: &mut ActorContext) {
        self.node.post_stop(c)
    }
    fn pre_restart(&mut self, c: &mut ActorContext, r: &Throwable, m: Option<&AnyMessage>) {
        self.node.pre_restart(c, r, m)
    }
    fn post_restart(&mut self, c: &mut ActorContext, r: &Throwable) {
        self.node.log.lock().unwrap().push("post-restart".into());
        let _ = r;
        self.pre_start(c);
    }
    fn receive(&mut self, c: &mut ActorContext, m: &AnyMessage) -> bool {
        self.node.receive(c, m)
    }
}

fn tell(r: &ActorRef, m: &str) {
    r.tell(m.to_string(), None);
}

/// s01-resume-subtree: while `g` has not decided, `c` (a child of the failed `p`) processes nothing; after `Resume` it processes
/// what arrived meanwhile, in order, and `p` keeps going.
#[test]
fn the_children_of_an_actor_that_failed_are_suspended_until_its_supervisor_decides() {
    let w = world(Directive::Resume);
    tell(&w.c_ref, "m0");
    wait_until("m0", || snap(&w.c).len() == 2);
    tell(&w.p_ref, "boom");
    wait_until("g is deciding", || !snap(&w.g).is_empty());
    tell(&w.c_ref, "x1");
    tell(&w.c_ref, "x2");
    assert!(stays_at(&w.c, 2), "c must not process anything while its parent waits for the decision: {:?}", snap(&w.c));
    w.open();
    wait_until("c recovers", || snap(&w.c).len() == 4);
    tell(&w.p_ref, "pm");
    wait_until("p recovers", || snap(&w.p).len() == 3);
    assert_eq!(snap(&w.c), strs(&["pre-start", "m0", "x1", "x2"]));
    assert_eq!(snap(&w.p), strs(&["pre-start", "boom", "pm"]));
}

fn pause(w: &World) {
    tell(&w.c_ref, "m0");
    wait_until("m0", || snap(&w.c).len() == 2);
    tell(&w.p_ref, "boom");
    wait_until("g is deciding", || !snap(&w.g).is_empty());
}

/// s02-restart-subtree: the child is stopped by the restart without ever processing what arrived during the pause.
#[test]
fn a_restart_stops_the_suspended_child_without_processing_its_mailbox() {
    let w = world(Directive::Restart);
    pause(&w);
    tell(&w.c_ref, "x1");
    tell(&w.c_ref, "x2");
    assert!(stays_at(&w.c, 2));
    w.open();
    wait_until("p restarted", || snap(&w.p).len() == 6);
    wait_until("c stopped", || snap(&w.c).len() == 3);
    assert_eq!(snap(&w.c), strs(&["pre-start", "m0", "post-stop"]));
    assert_eq!(snap(&w.p), strs(&["pre-start", "boom", "pre-restart", "post-stop", "post-restart", "pre-start"]));
}

/// s03-stop-subtree
#[test]
fn a_stop_terminates_the_suspended_subtree() {
    let w = world(Directive::Stop);
    pause(&w);
    tell(&w.c_ref, "x1");
    assert!(stays_at(&w.c, 2));
    w.open();
    wait_until("g saw p terminate", || snap(&w.g).len() == 2);
    wait_until("c stopped", || snap(&w.c).len() == 3);
    assert_eq!(snap(&w.c), strs(&["pre-start", "m0", "post-stop"]));
    assert_eq!(snap(&w.p), strs(&["pre-start", "boom", "post-stop"]));
}

/// s04-two-children-resume: every child of the failed actor is suspended and resumed, not only one.
#[test]
fn all_children_are_suspended_and_resumed() {
    let w = world_with(Directive::Resume, false, true);
    let c2 = w.c2_ref.clone().unwrap();
    tell(&c2, "n0");
    wait_until("n0", || snap(&w.c2).len() == 2);
    pause(&w);
    tell(&w.c_ref, "x1");
    tell(&c2, "y1");
    tell(&c2, "y2");
    assert!(stays_at(&w.c, 2) && stays_at(&w.c2, 2));
    w.open();
    wait_until("both recover", || snap(&w.c).len() == 3 && snap(&w.c2).len() == 4);
    assert_eq!(snap(&w.c2), strs(&["pre-start", "n0", "y1", "y2"]));
}

/// s05-grandchild-resume / s06-grandchild-restart: the suspension is recursive.
#[test]
fn the_suspension_reaches_the_grandchildren() {
    let w = world_with(Directive::Resume, true, false);
    let gc = w.gc_ref.clone().unwrap();
    tell(&gc, "g0");
    wait_until("g0", || snap(&w.gc).len() == 2);
    pause(&w);
    tell(&gc, "z1");
    tell(&gc, "z2");
    assert!(stays_at(&w.gc, 2), "the grandchild must be suspended too: {:?}", snap(&w.gc));
    w.open();
    wait_until("gc recovers", || snap(&w.gc).len() == 4);
    tell(&w.c_ref, "after");
    wait_until("c works again", || snap(&w.c).len() == 3);
}
