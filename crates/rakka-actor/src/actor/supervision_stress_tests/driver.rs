// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! The spec's `StressTest` FSM, run by the test thread in phases. One deadline bounds the whole run of a seed (the spec's 90 s) and
//! every failure names the seed. `children` is only appended to, so the final "no undead" check covers every reference ever reported.

use super::{Failure, PingOfDeath, World, hierarchy, unwrapped};
use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::Throwable;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::ActorSystem;
use crate::dispatch::Terminated;
use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::SeqCst};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SIZE: usize = 500;
const ACTIONS: usize = 5000;
const SEEDS: [u64; 3] = [0x9E37_79B9, 0xD1B5_4A32, 0x2545_F491];
const DEADLINE: Duration = Duration::from_secs(90);
/// What the runs of the three seeds must have gone through, all together.
const BRANCHES: [&str; 9] = [
    "child created again",
    "constructor threw",
    "decided Restart",
    "decided Resume",
    "decided Stop",
    "escalated",
    "postRestart threw",
    "preRestart threw",
    "restarted",
];

/// What the driver is told. `NotResumed` (stopped while failed or suspended) is an error only while the tree is being stopped.
pub(super) enum Event {
    Ready(ActorRef),
    Died(String),
    Pong(ActorRef),
    NotResumed(String),
    Broken(String),
    HeadTerminated,
}

pub(super) struct Random(u64);

impl Random {
    pub(super) fn below(&mut self, bound: usize) -> usize {
        // xorshift
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }

    /// The spec's `random012`: 0 nine times in ten, 1 in 7 %, 2 in 3 %.
    fn rarely(&mut self) -> u32 {
        match self.below(100) {
            10.. => 0,
            3.. => 1,
            _ => 2,
        }
    }

    fn failure(&mut self) -> Failure {
        let directive = if self.below(2) == 0 {
            Directive::Restart
        } else {
            Directive::Resume
        };
        let stop = self.rarely() > 0;
        let depth = self.rarely();
        let fail_pre = AtomicU32::new(self.rarely());
        let fail_post = AtomicU32::new(self.rarely());
        let fail_constr = self.rarely();
        let stop_kids = match self.rarely() {
            0 => 0,
            1 => self.below(super::BREADTH / 2),
            _ => 1000,
        };
        Failure {
            directive,
            stop,
            depth,
            fail_pre,
            fail_post,
            fail_constr,
            stop_kids,
        }
    }
}

/// The spec's FSM actor as the supervisor of the head: it does not escalate.
struct Boss(Arc<World>);

impl Actor for Boss {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let props = hierarchy(&self.0, "head".into(), SIZE, 0);
        let head = ctx.actor_of_named(props, "head").unwrap();
        ctx.watch(&head);
    }

    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let world = self.0.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            unwrapped(cause).map_or(Directive::Stop, |id| world.failure(id).directive)
        })
        .into()
    }

    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let terminated = msg.is::<Terminated>();
        if terminated {
            self.0.send(Event::HeadTerminated);
        }
        terminated
    }
}

fn path_of(actor: &ActorRef) -> String {
    actor.path().to_string()
}

struct Driver {
    seed: u64,
    deadline: Instant,
    events: Receiver<Event>,
    /// Every reference ever reported; never removed.
    children: Vec<ActorRef>,
    active: Vec<ActorRef>,
    idle: Vec<ActorRef>,
    pinged: HashSet<String>,
}

impl Driver {
    /// Fails the seed once its deadline has passed. Called for every event taken, so a stream of events cannot outlast it.
    fn check_deadline(&self, phase: &str) {
        let left = self.deadline.saturating_duration_since(Instant::now());
        let pings = self.pinged.len();
        assert!(
            !left.is_zero(),
            "seed {:#x}: deadline while {phase}, {pings} pings",
            self.seed
        );
    }

    fn next(&self, phase: &str) -> Event {
        self.check_deadline(phase);
        let left = self.deadline.saturating_duration_since(Instant::now());
        self.events.recv_timeout(left).unwrap_or_else(|_| {
            self.check_deadline(phase);
            panic!("seed {:#x}: no more events while {phase}", self.seed)
        })
    }

    /// Takes what has already been reported.
    fn drain(&mut self, phase: &str) {
        while let Ok(event) = self.events.try_recv() {
            self.check_deadline(phase);
            self.take(event);
        }
    }

    /// Forgets the actor at `path` and everything below it, except in `children`.
    fn bury(&mut self, path: &str) {
        let below = format!("{path}/");
        let alive = |other: &str| !format!("{other}/").starts_with(&below);
        self.active.retain(|actor| alive(&path_of(actor)));
        self.idle.retain(|actor| alive(&path_of(actor)));
        self.pinged.retain(|other| alive(other));
    }

    fn take(&mut self, event: Event) {
        match event {
            Event::Ready(actor) => {
                self.children.push(actor.clone());
                self.active.push(actor.clone());
                self.idle.push(actor);
            }
            Event::Died(path) => self.bury(&path),
            Event::Pong(actor) => {
                self.pinged.remove(&path_of(&actor));
                self.idle.push(actor);
            }
            Event::NotResumed(_) => {}
            Event::Broken(what) => panic!("seed {:#x}: {what}", self.seed),
            Event::HeadTerminated => panic!("seed {:#x}: the head stopped by itself", self.seed),
        }
    }

    fn ping(&mut self, index: usize) {
        let actor = self.idle.swap_remove(index);
        self.pinged.insert(path_of(&actor));
        actor.tell("ping", None);
    }

    fn await_pongs(&mut self, phase: &str) {
        while !self.pinged.is_empty() {
            let event = self.next(phase);
            self.take(event);
        }
    }

    fn init(&mut self) -> ActorRef {
        while self.children.len() < SIZE {
            match self.next("creating the hierarchy") {
                Event::Ready(actor) if self.children.contains(&actor) => {
                    panic!("seed {:#x}: children not unique", self.seed)
                }
                event => self.take(event),
            }
        }
        let head = self.children.iter().find(|a| a.path().name() == "head");
        head.unwrap().clone()
    }

    fn stress(&mut self, world: &World, random: &mut Random) {
        for _ in 0..ACTIONS {
            self.drain("stressing");
            while self.idle.is_empty() {
                let event = self.next("waiting for an idle actor");
                self.take(event);
            }
            self.check_deadline("stressing");
            if random.below(2) == 0 {
                let pick = random.below(self.idle.len());
                self.ping(pick);
            } else {
                let target = self.active[random.below(self.active.len())].clone();
                target.tell(world.add(random.failure()), None);
            }
        }
    }

    fn stop(&mut self, head: &ActorRef) {
        // what was reported before belongs to the earlier phases
        self.drain("before stopping");
        head.tell(PingOfDeath, None);
        loop {
            match self.next("stopping") {
                Event::HeadTerminated => break,
                Event::Broken(what) => panic!("seed {:#x}: {what}", self.seed),
                Event::NotResumed(path) => {
                    panic!(
                        "seed {:#x}: {path} stopped without having been resumed",
                        self.seed
                    )
                }
                _ => {}
            }
        }
        let undead: Vec<_> = self
            .children
            .iter()
            .filter(|a| !a.is_terminated())
            .collect();
        assert!(undead.is_empty(), "seed {:#x}: {undead:?}", self.seed);
    }
}

/// One run of the spec; returns the branches it went through.
fn run(seed: u64) -> BTreeSet<&'static str> {
    let (out, events) = channel();
    let world = Arc::new(World {
        failures: Mutex::default(),
        states: Mutex::default(),
        hits: Mutex::default(),
        random: Mutex::new(Random(seed)),
        events: Mutex::new(out),
        ignore_fail_constr: AtomicBool::new(false),
    });
    let mut driver = Driver {
        seed,
        deadline: Instant::now() + DEADLINE,
        events,
        children: Vec::new(),
        active: Vec::new(),
        idle: Vec::new(),
        pinged: HashSet::new(),
    };
    let system = ActorSystem::create("t-stress").unwrap();
    let boss = world.clone();
    let props = Props::new(move || Boss(boss.clone()));
    system.actor_of_named(props, "stressTest").unwrap();
    let head = driver.init();
    driver.stress(&world, &mut Random(seed.rotate_left(32)));
    // Finishing
    if !driver.pinged.is_empty() {
        world.ignore_fail_constr.store(true, SeqCst);
        driver.await_pongs("finishing");
    }
    // LastPing
    while !driver.idle.is_empty() {
        driver.ping(0);
    }
    driver.await_pongs("in the last ping");
    driver.stop(&head);
    let left = driver.deadline.saturating_duration_since(Instant::now());
    let terminated = system.terminate().wait(left);
    assert!(
        matches!(terminated, Some(Ok(_))),
        "seed {seed:#x}: no termination"
    );
    let hits = world.hits.lock().unwrap();
    hits.clone()
}

#[test]
fn survive_being_stressed() {
    let reached: BTreeSet<_> = SEEDS.into_iter().flat_map(run).collect();
    assert_eq!(reached, BTreeSet::from(BRANCHES), "seeds {SEEDS:x?}");
}
