// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! "survive being stressed" (AC-SUP-010): a random tree is failed and pinged at random. No actor runs a message while it is failed or
//! suspended, no instance gets `preRestart` twice, a restarted actor has all its children, every survivor answers a last ping and the
//! whole tree stops. Bounded: 100 actors and 4000 actions per fixed seed (Pekko: 500 and 500000), the test thread in place of the FSM,
//! no sleeps. It lives in the crate because "suspended" is read from the actor's own mailbox.

use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::ActorSystem;
use crate::dispatch::Terminated;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SIZE: usize = 100;
const BREADTH: usize = 6;
const ACTIONS: usize = 4000;
const FAILURE: &str = "org.apache.pekko.actor.SupervisorHierarchySpec$Failure";

/// The spec's `Failure`; the thrown `Throwable` carries its index in `World::failures`, where the counters live.
#[derive(Clone, Copy)]
struct Failure {
    directive: Directive,
    stop: bool,
    depth: u32,
    fail_pre: u32,
    fail_post: u32,
    fail_constr: u32,
    stop_kids: usize,
}

/// What survives a restart: the children with their subtree sizes, and the constructor failures still to be thrown.
#[derive(Clone, Default)]
struct State {
    kids: BTreeMap<String, usize>,
    fail_constr: Option<(u32, usize)>,
}

enum Event {
    Ready(ActorRef),
    Died(String),
    Pong(ActorRef),
    /// Stopped while still failed or suspended; an error only while the tree is being stopped.
    NotResumed(String),
    Broken(String),
    HeadTerminated,
}

struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
    /// 0 nine times in ten, 1 in 7 %, 2 in 3 % (`random012`).
    fn rarely(&mut self) -> u32 {
        match self.below(100) {
            10.. => 0,
            3.. => 1,
            _ => 2,
        }
    }
}

struct World {
    failures: Mutex<Vec<Failure>>,
    states: Mutex<HashMap<String, State>>,
    random: Mutex<Random>,
    events: Mutex<Sender<Event>>,
    ignore_fail_constr: AtomicBool,
}

impl World {
    fn send(&self, event: Event) {
        let _ = self.events.lock().unwrap().send(event);
    }
    fn add(&self, failure: Failure) -> usize {
        let mut failures = self.failures.lock().unwrap();
        failures.push(failure);
        failures.len() - 1
    }
    fn failure(&self, id: usize) -> Failure {
        self.failures.lock().unwrap()[id]
    }
    /// Takes one of the throws left in a counter; actors that share a failure take them concurrently.
    fn take_throw(&self, id: usize, counter: fn(&mut Failure) -> &mut u32) -> bool {
        let mut failures = self.failures.lock().unwrap();
        let left = counter(&mut failures[id]);
        let due = *left > 0;
        *left = left.saturating_sub(1);
        due
    }
}

fn throw_failure(id: usize) -> ! {
    throw(Throwable::new(FAILURE, Some(&id.to_string())))
}

fn failure_id(thrown: &Throwable) -> Option<usize> {
    (thrown.class() == FAILURE).then(|| thrown.message().unwrap().parse().unwrap())
}

/// The failure itself, or the one a constructor, `preStart` or `postRestart` threw.
fn unwrapped(thrown: &Throwable) -> Option<usize> {
    failure_id(thrown).or_else(|| thrown.cause().and_then(failure_id))
}

#[derive(Default)]
struct Flags {
    failed: AtomicBool,
    suspended: AtomicBool,
}

impl Flags {
    fn set(&self, directive: Directive) {
        match directive {
            Directive::Restart => self.failed.store(true, Ordering::SeqCst),
            Directive::Resume => self.suspended.store(true, Ordering::SeqCst),
            _ => {}
        }
    }
}

struct PingOfDeath;
struct PongOfDeath;

struct Hierarchy {
    world: Arc<World>,
    path: String,
    size: usize,
    level: usize,
    flags: Arc<Flags>,
    pre_restart_called: bool,
    pongs_to_go: usize,
    reply_to: Option<ActorRef>,
}

/// The factory is the constructor: it throws while the state of its path says so.
fn hierarchy(world: &Arc<World>, path: String, size: usize, level: usize) -> Props {
    let world = world.clone();
    Props::new(move || {
        let due = world
            .states
            .lock()
            .unwrap()
            .get_mut(&path)
            .and_then(|state| {
                let (left, id) = state.fail_constr.as_mut()?;
                (*left > 0 && !world.ignore_fail_constr.load(Ordering::SeqCst)).then(|| {
                    *left -= 1;
                    *id
                })
            });
        if let Some(id) = due {
            throw_failure(id);
        }
        Hierarchy {
            world: world.clone(),
            path: path.clone(),
            size,
            level,
            flags: Arc::default(),
            pre_restart_called: false,
            pongs_to_go: 0,
            reply_to: None,
        }
    })
}

impl Hierarchy {
    fn broken(&self, what: &str) {
        self.world
            .send(Event::Broken(format!("{}: {what}", self.path)));
    }

    fn spawn(&self, ctx: &mut ActorContext, name: &str, size: usize) {
        let path = format!("{}/{name}", self.path);
        let props = hierarchy(&self.world, path, size, self.level + 1);
        let child = ctx.actor_of_named(props, name).unwrap();
        ctx.watch(&child);
    }
}

impl Actor for Hierarchy {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.world.send(Event::Ready(ctx.self_ref()));
        let rest = self.size - 1;
        let mut state = State::default();
        if rest > 0 {
            let kids = self.world.random.lock().unwrap().below(BREADTH.min(rest)) + 1;
            for id in 1..=kids {
                let size = rest / kids + usize::from(id <= rest % kids);
                self.spawn(ctx, &id.to_string(), size);
                state.kids.insert(id.to_string(), size);
            }
        }
        self.world
            .states
            .lock()
            .unwrap()
            .insert(self.path.clone(), state);
    }

    fn pre_restart(&mut self, ctx: &mut ActorContext, cause: &Throwable, _: Option<&AnyMessage>) {
        if std::mem::replace(&mut self.pre_restart_called, true) {
            return self.broken("preRestart called twice");
        }
        let Some(id) = failure_id(cause) else { return };
        let failure = self.world.failure(id);
        for child in ctx.children().iter().take(failure.stop_kids) {
            ctx.unwatch(child);
            ctx.stop(child);
        }
        if self.world.take_throw(id, |failure| &mut failure.fail_pre) {
            throw_failure(id);
        }
    }

    fn post_restart(&mut self, ctx: &mut ActorContext, cause: &Throwable) {
        let kids = self.world.states.lock().unwrap()[&self.path].kids.clone();
        for (name, size) in &kids {
            if ctx.child(name).is_none() {
                self.world
                    .send(Event::Died(format!("{}/{name}", self.path)));
                self.spawn(ctx, name, *size);
            }
        }
        if ctx.children().len() != kids.len() {
            self.broken("invariant violated: children lost or added by the restart");
        }
        let own = cause
            .message()
            .is_some_and(|text| text.contains(&format!("{}: ", self.path)));
        if let Some(id) = failure_id(cause).or_else(|| unwrapped(cause).filter(|_| own))
            && self.world.take_throw(id, |failure| &mut failure.fail_post)
        {
            throw_failure(id);
        }
    }

    fn post_stop(&mut self, _: &mut ActorContext) {
        let unresolved =
            self.flags.failed.load(Ordering::SeqCst) || self.flags.suspended.load(Ordering::SeqCst);
        if unresolved {
            self.world.send(Event::NotResumed(self.path.clone()));
        } else {
            self.world
                .states
                .lock()
                .unwrap()
                .insert(self.path.clone(), State::default());
        }
    }

    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (world, flags, level, dying) = (
            self.world.clone(),
            self.flags.clone(),
            self.level,
            self.pongs_to_go > 0,
        );
        let path = self.path.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            if dying {
                return Directive::Resume;
            }
            let Some(id) = unwrapped(cause) else {
                world.send(Event::Broken(format!(
                    "{path}: unhandled exception {cause}"
                )));
                return Directive::Resume;
            };
            let failure = world.failure(id);
            if failure.depth > 0 {
                flags.set(failure.directive);
                throw_failure(world.add(Failure {
                    depth: failure.depth - 1,
                    ..failure
                }));
            }
            if level > 3 && failure.fail_post == 0 && failure.stop {
                Directive::Stop
            } else {
                failure.directive
            }
        })
        .into()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.flags.suspended.store(false, Ordering::SeqCst);
        if self.flags.failed.load(Ordering::SeqCst) {
            self.broken("processing message while failed");
        } else if !ctx.self_ref().mailbox_view().contains("suspendCount=0 ") {
            self.broken("processing message while suspended");
        } else if let Some(&id) = msg.downcast_ref::<usize>() {
            let failure = self.world.failure(id);
            self.flags.set(failure.directive);
            let copy = self.world.add(failure);
            let due = Some((failure.fail_constr, copy));
            self.world
                .states
                .lock()
                .unwrap()
                .get_mut(&self.path)
                .unwrap()
                .fail_constr = due;
            throw_failure(id);
        } else if msg.downcast_ref::<&str>() == Some(&"ping") {
            self.world.send(Event::Pong(ctx.self_ref()));
        } else if let Some(terminated) = msg.downcast_ref::<Terminated>() {
            let name = terminated.actor.path().name().to_string();
            // otherwise it is a `Terminated` of an old child
            if self.pongs_to_go == 0
                && ctx
                    .child(&name)
                    .is_none_or(|child| child == terminated.actor)
            {
                self.world
                    .send(Event::Died(format!("{}/{name}", self.path)));
                let size = self.world.states.lock().unwrap()[&self.path].kids[&name];
                self.spawn(ctx, &name, size);
            }
        } else if msg.is::<PingOfDeath>() {
            self.reply_to = Some(ctx.sender());
            if self.size > 1 {
                let children = ctx.children();
                self.pongs_to_go = children.len();
                children
                    .iter()
                    .for_each(|child| child.tell(PingOfDeath, Some(&ctx.self_ref())));
            } else {
                ctx.stop(&ctx.self_ref());
                self.reply_to.as_ref().unwrap().tell(PongOfDeath, None);
            }
        } else if msg.is::<PongOfDeath>() {
            self.pongs_to_go -= 1;
            if self.pongs_to_go == 0 {
                ctx.stop(&ctx.self_ref());
                self.reply_to.as_ref().unwrap().tell(PongOfDeath, None);
            }
        } else {
            return false;
        }
        true
    }
}

/// The spec's FSM actor as a supervisor of the head: it does not escalate.
struct Boss(Arc<World>);

impl Actor for Boss {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let head = hierarchy(&self.0, "head".into(), SIZE, 0);
        let head = ctx.actor_of_named(head, "head").unwrap();
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

/// The references the FSM keeps: those it may fail, those it may ping and those that owe a pong.
#[derive(Default)]
struct Driver {
    all: Vec<ActorRef>,
    idle: Vec<ActorRef>,
    pinged: HashSet<String>,
    died: usize,
}

fn path_of(actor: &ActorRef) -> String {
    actor.path().to_string()
}

impl Driver {
    fn bury(&mut self, path: &str) {
        let below = format!("{path}/");
        let alive = |other: &str| !format!("{other}/").contains(&below);
        self.all.retain(|actor| alive(&path_of(actor)));
        self.idle.retain(|actor| alive(&path_of(actor)));
        self.pinged.retain(|other| alive(other));
    }

    fn take(&mut self, event: Event) {
        match event {
            // an actor made again by `Create` stops its children without a `Died`: its `Ready` forgets what was there
            Event::Ready(actor) => {
                self.bury(&path_of(&actor));
                self.all.push(actor.clone());
                self.idle.push(actor);
            }
            Event::Died(path) => {
                self.died += 1;
                self.bury(&path);
            }
            Event::Pong(actor) => {
                self.pinged.remove(&path_of(&actor));
                self.idle.push(actor);
            }
            Event::NotResumed(_) => {}
            Event::Broken(what) => panic!("{what}"),
            Event::HeadTerminated => panic!("the hierarchy stopped by itself"),
        }
    }

    fn ping(&mut self, index: usize) {
        let actor = self.idle.swap_remove(index);
        self.pinged.insert(path_of(&actor));
        actor.tell("ping", None);
    }
}

fn next(events: &Receiver<Event>, phase: &str) -> Event {
    let event = events.recv_timeout(Duration::from_secs(10));
    event.unwrap_or_else(|_| panic!("no progress while {phase}"))
}

fn stress(seed: u64) {
    let (out, events) = channel();
    let world = Arc::new(World {
        failures: Mutex::default(),
        states: Mutex::default(),
        random: Mutex::new(Random(seed)),
        events: Mutex::new(out),
        ignore_fail_constr: AtomicBool::new(false),
    });
    let system = ActorSystem::create("t-stress").unwrap();
    let boss = world.clone();
    system
        .actor_of_named(Props::new(move || Boss(boss.clone())), "stressTest")
        .unwrap();
    let mut driver = Driver::default();
    while driver.all.len() < SIZE {
        match next(&events, "creating the hierarchy") {
            Event::Ready(actor) if driver.all.contains(&actor) => panic!("children not unique"),
            event => driver.take(event),
        }
    }
    let head = driver
        .all
        .iter()
        .find(|actor| actor.path().name() == "head")
        .unwrap()
        .clone();
    let mut random = Random(seed.rotate_left(32));
    for _ in 0..ACTIONS {
        while let Ok(event) = events.try_recv() {
            driver.take(event);
        }
        while driver.idle.is_empty() {
            driver.take(next(&events, "waiting for an idle actor"));
        }
        if random.below(2) == 0 {
            driver.ping(random.below(driver.idle.len()));
        } else {
            let target = driver.all[random.below(driver.all.len())].clone();
            let failure = Failure {
                directive: if random.below(2) == 0 {
                    Directive::Restart
                } else {
                    Directive::Resume
                },
                stop: random.rarely() > 0,
                depth: random.rarely(),
                fail_pre: random.rarely(),
                fail_post: random.rarely(),
                fail_constr: random.rarely(),
                stop_kids: [0, random.below(BREADTH / 2), 1000][random.rarely() as usize],
            };
            target.tell(world.add(failure), None);
        }
    }
    world.ignore_fail_constr.store(true, Ordering::SeqCst);
    while !driver.pinged.is_empty() {
        driver.take(next(&events, "finishing"));
    }
    while !driver.idle.is_empty() {
        driver.ping(0);
    }
    while !driver.pinged.is_empty() {
        driver.take(next(&events, "waiting for the last pongs"));
    }
    head.tell(PingOfDeath, None);
    loop {
        match next(&events, "stopping") {
            Event::HeadTerminated => break,
            Event::Broken(what) => panic!("{what}"),
            Event::NotResumed(path) => panic!("{path} stopped without having been resumed"),
            _ => {}
        }
    }
    let undead: Vec<_> = driver
        .all
        .iter()
        .filter(|actor| !actor.is_terminated())
        .map(path_of)
        .collect();
    assert_eq!(undead, Vec::<String>::new());
    assert!(driver.died > 0, "the stress made nobody be created again");
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .unwrap()
        .unwrap();
}

#[test]
fn survive_being_stressed() {
    for seed in [
        0x9E37_79B9_7F4A_7C15,
        0xD1B5_4A32_D192_ED03,
        0x2545_F491_4F6C_DD1D,
    ] {
        stress(seed);
    }
}
