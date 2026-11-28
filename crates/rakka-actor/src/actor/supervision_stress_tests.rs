// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! The actors of `SupervisorHierarchySpec` "survive being stressed" (AC-SUP-010): a random tree of 500 actors that is pinged and failed
//! at random (see `driver`). A characterisation test (no production change). Integration: real actors and dispatcher.

mod driver;

use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::dispatch::Terminated;
use driver::{Event, Random};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::SeqCst};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

const BREADTH: usize = 6;
const FAILURE: &str = "org.apache.pekko.actor.SupervisorHierarchySpec$Failure";

/// The spec's `Failure`: one instance per `fail` action or escalation; `fail_pre` and `fail_post` are its `var`s.
struct Failure {
    directive: Directive,
    stop: bool,
    depth: u32,
    fail_pre: AtomicU32,
    fail_post: AtomicU32,
    fail_constr: u32,
    stop_kids: usize,
}

impl Failure {
    fn copy(&self, depth: u32) -> Failure {
        Failure {
            directive: self.directive,
            stop: self.stop,
            depth,
            fail_pre: AtomicU32::new(self.fail_pre.load(SeqCst)),
            fail_post: AtomicU32::new(self.fail_post.load(SeqCst)),
            fail_constr: self.fail_constr,
            stop_kids: self.stop_kids,
        }
    }
}

/// Takes one of the throws left in a counter.
fn take_throw(counter: &AtomicU32) -> bool {
    counter
        .try_update(SeqCst, SeqCst, |left| left.checked_sub(1))
        .is_ok()
}

/// What the spec's `stateCache` keeps of a path across restarts: its children with their subtree sizes, and the constructor failures
/// still to be thrown.
#[derive(Clone, Default)]
struct State {
    kids: BTreeMap<String, usize>,
    fail_constr: Option<(u32, usize)>,
}

struct World {
    failures: Mutex<Vec<Arc<Failure>>>,
    states: Mutex<HashMap<String, State>>,
    random: Mutex<Random>,
    events: Mutex<Sender<Event>>,
    ignore_fail_constr: AtomicBool,
}

impl World {
    fn send(&self, event: Event) {
        let _ = self.events.lock().unwrap().send(event);
    }

    fn states(&self) -> MutexGuard<'_, HashMap<String, State>> {
        self.states.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn failure(&self, id: usize) -> Arc<Failure> {
        self.failures.lock().unwrap()[id].clone()
    }

    fn add(&self, failure: Failure) -> usize {
        let mut failures = self.failures.lock().unwrap();
        failures.push(Arc::new(failure));
        failures.len() - 1
    }
}

fn throw_failure(id: usize) -> ! {
    throw(Throwable::new(FAILURE, Some(&id.to_string())))
}

/// The id of a thrown `Failure`.
fn failure_id(thrown: &Throwable) -> Option<usize> {
    (thrown.class() == FAILURE).then(|| thrown.message().unwrap().parse().unwrap())
}

/// The spec's `unwrap`: the failure itself, or the one a constructor, `preStart` or `postRestart` threw.
fn unwrapped(thrown: &Throwable) -> Option<usize> {
    failure_id(thrown).or_else(|| thrown.cause().and_then(failure_id))
}

/// The `failed` and `suspended` flags of one instance.
#[derive(Default)]
struct Flags {
    failed: AtomicBool,
    suspended: AtomicBool,
}

impl Flags {
    fn set(&self, directive: Directive) {
        match directive {
            Directive::Restart => self.failed.store(true, SeqCst),
            Directive::Resume => self.suspended.store(true, SeqCst),
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
        let due = world.states().get_mut(&path).and_then(|state| {
            let (left, id) = state.fail_constr.as_mut()?;
            let active = *left > 0 && !world.ignore_fail_constr.load(SeqCst);
            active.then(|| {
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
        let what = format!("{}: {what}", self.path);
        self.world.send(Event::Broken(what));
    }

    fn spawn(&self, ctx: &mut ActorContext, name: &str, size: usize) {
        let path = format!("{}/{name}", self.path);
        let props = hierarchy(&self.world, path, size, self.level + 1);
        let child = ctx.actor_of_named(props, name).unwrap();
        ctx.watch(&child);
    }

    /// Creates again a child that is gone, telling the driver first.
    fn replace(&self, ctx: &mut ActorContext, name: &str, size: usize) {
        let path = ctx.self_ref().path().to_string();
        self.world.send(Event::Died(format!("{path}/{name}")));
        self.spawn(ctx, name, size);
    }

    fn stop_and_report(&self, ctx: &mut ActorContext) {
        ctx.stop(&ctx.self_ref());
        self.reply_to.as_ref().unwrap().tell(PongOfDeath, None);
    }

    /// The failing branch of `receive`.
    fn fail(&self, id: usize) -> ! {
        let failure = self.world.failure(id);
        self.flags.set(failure.directive);
        let copy = self.world.add(failure.copy(failure.depth));
        let mut states = self.world.states();
        let state = states.get_mut(&self.path).unwrap();
        state.fail_constr = Some((failure.fail_constr, copy));
        drop(states);
        throw_failure(id)
    }
}

impl Actor for Hierarchy {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        self.world.send(Event::Ready(ctx.self_ref()));
        let rest = self.size - 1;
        let mut state = State::default();
        if rest > 0 {
            let most = BREADTH.min(rest);
            let kids = self.world.random.lock().unwrap().below(most) + 1;
            for id in 1..=kids {
                let size = rest / kids + usize::from(id <= rest % kids);
                self.spawn(ctx, &id.to_string(), size);
                state.kids.insert(id.to_string(), size);
            }
        }
        self.world.states().insert(self.path.clone(), state);
    }

    fn pre_restart(&mut self, ctx: &mut ActorContext, cause: &Throwable, _: Option<&AnyMessage>) {
        if std::mem::replace(&mut self.pre_restart_called, true) {
            return self.broken("preRestart called twice");
        }
        let Some(id) = failure_id(cause) else {
            return;
        };
        let failure = self.world.failure(id);
        for child in ctx.children().iter().take(failure.stop_kids) {
            ctx.unwatch(child);
            ctx.stop(child);
        }
        if take_throw(&failure.fail_pre) {
            throw_failure(id);
        }
    }

    fn post_restart(&mut self, ctx: &mut ActorContext, cause: &Throwable) {
        let kids = self.world.states()[&self.path].kids.clone();
        for (name, size) in &kids {
            if ctx.child(name).is_none() {
                self.replace(ctx, name, *size);
            }
        }
        if ctx.children().len() != kids.len() {
            self.broken("invariant violated: children lost or added by the restart");
        }
        // a failure, or the `PostRestartException` of this very actor around one
        let own = format!("{}: ", ctx.self_ref().path());
        let wrapped = cause.message().is_some_and(|text| text.starts_with(&own));
        let thrown = failure_id(cause).or_else(|| unwrapped(cause).filter(|_| wrapped));
        if let Some(id) = thrown
            && take_throw(&self.world.failure(id).fail_post)
        {
            throw_failure(id);
        }
    }

    fn post_stop(&mut self, _: &mut ActorContext) {
        if self.flags.failed.load(SeqCst) || self.flags.suspended.load(SeqCst) {
            self.world.send(Event::NotResumed(self.path.clone()));
        } else {
            self.world
                .states()
                .insert(self.path.clone(), State::default());
        }
    }

    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (world, flags, path) = (self.world.clone(), self.flags.clone(), self.path.clone());
        let (level, dying) = (self.level, self.pongs_to_go > 0);
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            let Some(id) = unwrapped(cause) else {
                let what = format!("{path}: unhandled exception {cause}");
                world.send(Event::Broken(what));
                return Directive::Resume;
            };
            if dying {
                return Directive::Resume;
            }
            let failure = world.failure(id);
            if failure.depth > 0 {
                flags.set(failure.directive);
                throw_failure(world.add(failure.copy(failure.depth - 1)));
            }
            if level > 3 && failure.fail_post.load(SeqCst) == 0 && failure.stop {
                Directive::Stop
            } else {
                failure.directive
            }
        })
        .into()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        // `check`
        self.flags.suspended.store(false, SeqCst);
        if self.flags.failed.load(SeqCst) {
            self.broken("processing message while failed");
            return true;
        }
        if !ctx.self_ref().mailbox_view().contains("suspendCount=0 ") {
            self.broken("processing message while suspended");
            return true;
        }
        if let Some(&id) = msg.downcast_ref::<usize>() {
            self.fail(id);
        } else if msg.downcast_ref::<&str>() == Some(&"ping") {
            // as the spec: about 3 % of the pings take a millisecond
            let slow = self.world.random.lock().unwrap().below(103) >= 100;
            if slow {
                std::thread::sleep(Duration::from_millis(1));
            }
            self.world.send(Event::Pong(ctx.self_ref()));
        } else if let Some(terminated) = msg.downcast_ref::<Terminated>() {
            let name = terminated.actor.path().name().to_string();
            // otherwise it is the `Terminated` of an old child
            let current = ctx.child(&name);
            if self.pongs_to_go == 0 && current.is_none_or(|child| child == terminated.actor) {
                let size = self.world.states()[&self.path].kids[&name];
                self.replace(ctx, &name, size);
            }
        } else if msg.is::<PingOfDeath>() {
            self.reply_to = Some(ctx.sender());
            if self.size > 1 {
                let children = ctx.children();
                self.pongs_to_go = children.len();
                for child in &children {
                    child.tell(PingOfDeath, Some(&ctx.self_ref()));
                }
            } else {
                self.stop_and_report(ctx);
            }
        } else if msg.is::<PongOfDeath>() {
            self.pongs_to_go -= 1;
            if self.pongs_to_go == 0 {
                self.stop_and_report(ctx);
            }
        } else {
            return false;
        }
        true
    }
}
