// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! The spec's `StressTest` FSM, run by the test thread in phases. One deadline bounds the whole run of a seed (the spec's 90 s) and
//! every failure names the seed. `children` is only appended to, so the final "no undead" check covers every reference ever reported.

use super::{PingOfDeath, World, hierarchy};
use crate::actor::actor_ref::ActorRef;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::ActorSystem;
use crate::dispatch::Terminated;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SIZE: usize = 500;
const SEEDS: [u64; 3] = [0x9E37_79B9, 0xD1B5_4A32, 0x2545_F491];
const DEADLINE: Duration = Duration::from_secs(90);

/// What the driver is told.
pub(super) enum Event {
    Ready(ActorRef),
    Pong(ActorRef),
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
}

/// The spec's FSM actor as the supervisor of the head.
struct Boss(Arc<World>);

impl Actor for Boss {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let props = hierarchy(&self.0, SIZE);
        let head = ctx.actor_of_named(props, "head").unwrap();
        ctx.watch(&head);
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

    fn take(&mut self, event: Event) {
        match event {
            Event::Ready(actor) => {
                self.children.push(actor.clone());
                self.idle.push(actor);
            }
            Event::Pong(actor) => {
                self.pinged.remove(&path_of(&actor));
                self.idle.push(actor);
            }
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

    fn stop(&mut self, head: &ActorRef) {
        head.tell(PingOfDeath, None);
        while !matches!(self.next("stopping"), Event::HeadTerminated) {}
        let undead: Vec<_> = self
            .children
            .iter()
            .filter(|a| !a.is_terminated())
            .collect();
        assert!(undead.is_empty(), "seed {:#x}: {undead:?}", self.seed);
    }
}

/// One run of the spec.
fn run(seed: u64) {
    let (out, events) = channel();
    let world = Arc::new(World {
        random: Mutex::new(Random(seed)),
        events: Mutex::new(out),
    });
    let mut driver = Driver {
        seed,
        deadline: Instant::now() + DEADLINE,
        events,
        children: Vec::new(),
        idle: Vec::new(),
        pinged: HashSet::new(),
    };
    let system = ActorSystem::create("t-stress").unwrap();
    let boss = world.clone();
    let props = Props::new(move || Boss(boss.clone()));
    system.actor_of_named(props, "stressTest").unwrap();
    let head = driver.init();
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
}

#[test]
fn a_random_tree_answers_every_ping_and_stops_without_an_undead() {
    SEEDS.into_iter().for_each(run);
}
