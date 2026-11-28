// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and dispatch/Mailbox.scala (Apache-2.0).
//! The suspend counter of failure and recursive suspension (AC-SUP-010), each script compared with the line of
//! `cases/oracle-only/mailbox.golden` that the Pekko 2.0.0 JVM printed (`MBX-OR-03`, `MBX-CC-03`, `MBX-SU-01`, `MBX-SU-02`). In the
//! crate because the cases read the mailbox state and send the internal `Suspend`/`Resume`. Integration: real actors and dispatcher.

use crate::actor::actor_ref::ActorRef;
use crate::actor::async_creation_tests::{Log, joined, recorded, wait_until};
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::ActorSystem;
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, ErrorEvent};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

/// Logs `r:<text>`; `"block"` holds its turn until the gate opens.
struct Worker(Log, Arc<Mutex<()>>);

impl Actor for Worker {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = *msg.downcast_ref::<&str>().unwrap();
        if text == "block" {
            self.0.lock().unwrap().push("entered".into());
            drop(self.1.lock().unwrap());
            self.0.lock().unwrap().retain(|entry| entry != "entered");
        }
        self.0.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

fn worker() -> (ActorSystem, ActorRef, Log, Arc<Mutex<()>>) {
    let system = ActorSystem::create("mbx").unwrap();
    let (log, gate) = (Log::default(), Arc::new(Mutex::new(())));
    let props = {
        let (log, gate) = (log.clone(), gate.clone());
        Props::new(move || Worker(log.clone(), gate.clone()))
    };
    let actor = system.actor_of_named(props, "w").unwrap();
    wait_until("w started", || actor.mailbox_view().starts_with("open"));
    (system, actor, log, gate)
}

/// Waits for `count` suspensions and an idle mailbox, then returns the view the oracle prints.
fn settled(actor: &ActorRef, count: u32) -> String {
    let expected = format!("open scheduled=false suspendCount={count} ");
    wait_until(&expected, || actor.mailbox_view().starts_with(&expected));
    actor.mailbox_view()
}

fn suspend(actor: &ActorRef) {
    actor.send_system(SystemMessage::Suspend);
}

fn resume(actor: &ActorRef) {
    actor.send_system(SystemMessage::Resume { cause: None });
}

#[test]
fn nested_suspensions_need_as_many_resumes() {
    let (_system, actor, log, _) = worker();
    suspend(&actor);
    suspend(&actor);
    actor.tell("u1", None);
    actor.tell("u2", None);
    let suspended = format!("{} log=[{}]", settled(&actor, 2), joined(&log));
    resume(&actor);
    let half = format!("{} log=[{}]", settled(&actor, 1), joined(&log));
    resume(&actor);
    wait_until("both", || log.lock().unwrap().len() == 2);
    let done = format!("{} log=[{}]", settled(&actor, 0), joined(&log));
    let observed = format!("suspended: {suspended} | oneResume: {half} | twoResumes: {done}");
    assert_eq!(observed, recorded("MBX-OR-03"));
}

/// Two threads released together, each running `send` fifty times on `actor`.
fn hammer(actor: &ActorRef, send: fn(&ActorRef)) {
    let go = Barrier::new(2);
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                go.wait();
                (0..50).for_each(|_| send(actor));
            });
        }
    });
}

#[test]
fn concurrent_suspend_and_resume_lose_no_increment() {
    let (_system, actor, log, gate) = worker();
    let held = gate.lock().unwrap();
    actor.tell("block", None);
    wait_until("blocked", || joined(&log) == "entered");
    actor.tell("u", None);
    hammer(&actor, suspend);
    drop(held);
    let suspended = format!("{} log=[{}]", settled(&actor, 100), joined(&log));
    hammer(&actor, resume);
    wait_until("u", || log.lock().unwrap().len() == 2);
    let resumed = format!("{} log=[{}]", settled(&actor, 0), joined(&log));
    let observed = format!("afterSuspends: {suspended} | afterResumes: {resumed}");
    assert_eq!(observed, recorded("MBX-CC-03"));
}

/// What the oracle's `Refailing` shares: the log, the permits of the held decisions and the switch of the throwing strategy.
#[derive(Clone)]
struct Shared {
    log: Log,
    permits: Arc<Mutex<Receiver<()>>>,
    fail_next: Arc<AtomicBool>,
    born: Sender<ActorRef>,
}

/// `s`: creates `p`, records each cause, holds each decision until a permit arrives and resumes.
struct Holder(Shared);

impl Actor for Holder {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let shared = self.0.clone();
        let props = Props::new(move || Fragile(shared.clone()));
        let parent = ctx.actor_of_named(props, "p").unwrap();
        self.0.born.send(parent).unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let shared = self.0.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            let text = cause.message.clone().unwrap();
            shared.log.lock().unwrap().push(format!("decide:{text}"));
            let permits = shared.permits.lock().unwrap();
            let _ = permits.recv_timeout(Duration::from_secs(10));
            Directive::Resume
        })
        .into()
    }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        false
    }
}

/// `p`: fails on `"boom"`; asked for its strategy when a child dies, it throws once `fail_next` is set.
struct Fragile(Shared);

impl Actor for Fragile {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        for name in ["c", "d"] {
            let child = ctx.actor_of_named(Props::new(|| Silent), name).unwrap();
            self.0.born.send(child).unwrap();
        }
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        if self.0.fail_next.swap(false, SeqCst) {
            throw(Throwable::illegal_state("strategy"));
        }
        SupervisorStrategy::default()
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = *msg.downcast_ref::<&str>().unwrap();
        if text == "boom" {
            throw(Throwable::illegal_state("boom"));
        }
        self.0.log.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

struct Silent;

impl Actor for Silent {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        false
    }
}

/// Collects the errors published by the cell of `p`.
struct Errors(Log);

impl Actor for Errors {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>()
            && error.log_source == "pekko://mbx/user/s/p"
        {
            let message = error.message.clone().unwrap_or_default();
            let line = format!("{}|{message}", error.cause.class);
            self.0.lock().unwrap().push(line);
        }
        true
    }
}

struct Refailing {
    _system: ActorSystem,
    shared: Shared,
    decide: Sender<()>,
    errors: Log,
    p: ActorRef,
    c: ActorRef,
    d: ActorRef,
}

impl Refailing {
    fn start() -> Refailing {
        let system = ActorSystem::create("mbx").unwrap();
        let errors = Log::default();
        let collected = errors.clone();
        let listener = system
            .actor_of(Props::new(move || Errors(collected.clone())))
            .unwrap();
        system.event_stream().subscribe(&listener, Channel::Error);
        let ((decide, permits), (born, births)) = (channel(), channel());
        let shared = Shared {
            log: Log::default(),
            permits: Arc::new(Mutex::new(permits)),
            fail_next: Arc::default(),
            born,
        };
        let holder = shared.clone();
        let props = Props::new(move || Holder(holder.clone()));
        system.actor_of_named(props, "s").unwrap();
        let mut refs: Vec<ActorRef> = (0..3)
            .map(|_| births.recv_timeout(Duration::from_secs(5)).unwrap())
            .collect();
        // `s` and `p` report from different turns, so the three arrive in no fixed order: pick by name
        let mut named = |name: &str| {
            let at = refs.iter().position(|r| r.path().name() == name).unwrap();
            refs.swap_remove(at)
        };
        let (p, c, d) = (named("p"), named("c"), named("d"));
        for actor in [&p, &c, &d] {
            settled(actor, 0);
        }
        Refailing {
            _system: system,
            shared,
            decide,
            errors,
            p,
            c,
            d,
        }
    }

    /// Waits for `event` in the log and for the counts of `p` and `d`; returns their views.
    fn views(&self, count: u32, event: &str) -> String {
        wait_until(event, || {
            self.shared.log.lock().unwrap().iter().any(|e| e == event)
        });
        let (p, d) = (settled(&self.p, count), settled(&self.d, count));
        format!("p=[{p}] d=[{d}]")
    }

    /// Stops `c` with the strategy of `p` armed: its `DeathWatchNotification` is processed while `p` is suspended.
    fn fail_inside_the_suspension(&self) {
        self.shared.fail_next.store(true, SeqCst);
        self.c.stop();
    }

    fn cell_errors(&self) -> String {
        wait_until("the error", || !self.errors.lock().unwrap().is_empty());
        joined(&self.errors)
    }
}

#[test]
fn a_failed_actor_that_fails_again_counts_again_recursively() {
    let f = Refailing::start();
    f.p.tell("boom", None);
    wait_until("deciding", || joined(&f.shared.log) == "decide:boom");
    f.p.tell("u", None);
    let first = f.views(1, "decide:boom");
    let first = format!("{first} c={}", settled(&f.c, 1));
    f.fail_inside_the_suspension();
    let second = f.views(2, "decide:boom");
    f.decide.send(()).unwrap();
    let half = f.views(1, "decide:strategy");
    f.decide.send(()).unwrap();
    let done = f.views(0, "r:u");
    let (log, errors) = (joined(&f.shared.log), f.cell_errors());
    let observed = format!(
        "afterFirstFailure: {first} | afterSecondFailure: {second} | afterOneResume: {half} \
         | afterTwoResumes: {done} log=[{log}] errors=[{errors}]"
    );
    assert_eq!(observed, recorded("MBX-SU-01"));
}

#[test]
fn a_failure_inside_a_suspension_by_another_counts_again_recursively() {
    let f = Refailing::start();
    suspend(&f.p);
    for actor in [&f.p, &f.c, &f.d] {
        settled(actor, 1);
    }
    f.p.tell("u", None);
    let suspended = format!(
        "p=[{}] d=[{}] c={}",
        settled(&f.p, 1),
        settled(&f.d, 1),
        settled(&f.c, 1)
    );
    f.fail_inside_the_suspension();
    let failed = f.views(2, "decide:strategy");
    f.decide.send(()).unwrap();
    let half = f.views(1, "decide:strategy");
    let half = format!("{half} log=[{}]", joined(&f.shared.log));
    resume(&f.p);
    let done = f.views(0, "r:u");
    let (log, errors) = (joined(&f.shared.log), f.cell_errors());
    let observed = format!(
        "suspended: {suspended} | afterFailure: {failed} | afterSupervisorResume: {half} \
         | afterResume: {done} log=[{log}] errors=[{errors}]"
    );
    assert_eq!(observed, recorded("MBX-SU-02"));
}
