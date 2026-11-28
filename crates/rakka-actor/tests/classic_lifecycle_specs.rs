// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala, SupervisorSpec.scala,
// Ticket669Spec.scala (Apache-2.0).
//! The lifecycle hook specs (AC-CELL-006, AC-CELL-020). The same sequences are recorded from Pekko 2.0.0 as the cases `lc-*`,
//! `l04-max-three-fourth-stops` and `cm-reply-in-pre-restart`. Pekko's `expectNoMessage` is the bounded wait of `Seen::nothing_else`.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::supervision::{
    AllForOneStrategy, Directive, OneForOneStrategy, SupervisorStrategy,
};

/// What the test actor of the spec receives.
struct Seen(Receiver<String>);

impl Seen {
    fn expect(&self, events: &[&str]) {
        for event in events {
            let seen = self.0.recv_timeout(Duration::from_secs(10));
            assert_eq!(seen.as_deref(), Ok(*event));
        }
    }
    fn nothing_else(&self) {
        let late = self.0.recv_timeout(Duration::from_millis(150));
        assert!(late.is_err(), "unexpected {late:?}");
    }
}

/// The test actor as a sender: reports the texts it is sent.
struct TestActor(Sender<String>);

impl Actor for TestActor {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0
            .send(msg.downcast_ref::<String>().unwrap().clone())
            .unwrap();
        true
    }
}

/// `Supervisor` of the spec: creates one child, hands out its reference and forwards to it.
struct Supervisor(
    Props,
    SupervisorStrategy,
    Sender<ActorRef>,
    Option<ActorRef>,
);

impl Actor for Supervisor {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let child = ctx.actor_of(self.0.clone()).unwrap();
        self.2.send(child.clone()).unwrap();
        self.3 = Some(child);
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        self.1.clone()
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        ctx.forward(self.3.as_ref().unwrap(), msg.clone());
        true
    }
}

fn supervised(
    system: &RunningSystem,
    strategy: impl Into<SupervisorStrategy>,
    child: Props,
) -> (ActorRef, ActorRef) {
    let (out, refs) = channel();
    let strategy = strategy.into();
    let supervisor =
        Props::new(move || Supervisor(child.clone(), strategy.clone(), out.clone(), None));
    let supervisor = system.actor_of(supervisor).unwrap();
    let child = refs.recv_timeout(Duration::from_secs(10)).unwrap();
    (supervisor, child)
}

fn three_retries() -> OneForOneStrategy {
    OneForOneStrategy::new(3, |_: &Throwable| Directive::Restart)
}

/// `LifeCycleTestActor`: reports `<event> <generation>` and leaves `pre_restart` and `post_restart` to the trait.
struct LifeCycle(Sender<String>, usize);

impl LifeCycle {
    /// `with_hooks` wraps every instance in the overrides of the first spec.
    fn props(out: Sender<String>, with_hooks: bool) -> Props {
        let generations = AtomicUsize::new(0);
        let next = move || LifeCycle(out.clone(), generations.fetch_add(1, Ordering::SeqCst));
        if with_hooks {
            Props::new(move || Records(next()))
        } else {
            Props::new(next)
        }
    }
    fn report(&self, event: &str) {
        self.0.send(format!("{event} {}", self.1)).unwrap();
    }
}

impl Actor for LifeCycle {
    fn pre_start(&mut self, _: &mut ActorContext) {
        self.report("preStart");
    }
    fn post_stop(&mut self, _: &mut ActorContext) {
        self.report("postStop");
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let status = msg.downcast_ref::<&str>() == Some(&"status");
        if status {
            self.report("OK");
        }
        status
    }
}

/// The same actor with the overrides of the first spec: they only report.
struct Records(LifeCycle);

#[rustfmt::skip] // one delegation per line
impl Actor for Records {
    fn pre_start(&mut self, ctx: &mut ActorContext) { self.0.pre_start(ctx) }
    fn post_stop(&mut self, ctx: &mut ActorContext) { self.0.post_stop(ctx) }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool { self.0.receive(ctx, msg) }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) { self.0.report("preRestart") }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) { self.0.report("postRestart") }
}

/// Kills the child four times under three retries; `restarted` is what each of the first three restarts reports.
fn killed_until_stopped(name: &str, with_hooks: bool, restarted: [[&str; 2]; 3]) {
    let system = RunningSystem::create(name);
    let (out, seen) = channel();
    let seen = Seen(seen);
    let child = LifeCycle::props(out, with_hooks);
    let (_, restarter) = supervised(&system, three_retries(), child);
    seen.expect(&["preStart 0"]);
    for (generation, events) in restarted.iter().enumerate() {
        restarter.tell(Kill, None);
        seen.expect(events);
        restarter.tell("status", None);
        seen.expect(&[&format!("OK {}", generation + 1)]);
    }
    restarter.tell(Kill, None);
    seen.expect(&["postStop 3"]);
    seen.nothing_else();
}

#[test]
#[rustfmt::skip] // one restart per line
fn invoke_pre_restart_pre_start_post_restart_when_using_one_for_one_strategy() {
    killed_until_stopped("t-life-custom", true, [
        ["preRestart 0", "postRestart 1"], ["preRestart 1", "postRestart 2"], ["preRestart 2", "postRestart 3"],
    ]);
}

#[test]
#[rustfmt::skip] // one restart per line
fn default_for_pre_restart_and_post_restart_is_to_call_post_stop_and_pre_start_respectively() {
    killed_until_stopped("t-life-default", false, [
        ["postStop 0", "preStart 1"], ["postStop 1", "preStart 2"], ["postStop 2", "preStart 3"],
    ]);
}

#[test]
fn not_invoke_pre_restart_and_post_restart_when_never_restarted() {
    let system = RunningSystem::create("t-life-never");
    let (out, seen) = channel();
    let seen = Seen(seen);
    let (_, child) = supervised(&system, three_retries(), LifeCycle::props(out, false));
    seen.expect(&["preStart 0"]);
    child.tell("status", None);
    seen.expect(&["OK 0"]);
    system.stop(&child);
    seen.expect(&["postStop 0"]);
    seen.nothing_else();
}

/// The one instance of "restart properly when same instance is returned". A Rust factory returns its actor by value, so what is
/// the same every time is the state its handles share: the counters of the four hooks.
#[derive(Clone)]
struct SameInstance(Sender<String>, Arc<[AtomicUsize; 4]>);

impl SameInstance {
    fn count(&self, hook: usize) {
        let times = self.1[hook].fetch_add(1, Ordering::SeqCst) + 1;
        let name = ["preStart", "preRestart", "postRestart", "postStop"][hook];
        self.0.send(format!("{name}{times}")).unwrap();
    }
}

#[rustfmt::skip] // one hook per line
impl Actor for SameInstance {
    fn pre_start(&mut self, _: &mut ActorContext) { self.count(0) }
    fn pre_restart(&mut self, _: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) { self.count(1) }
    fn post_restart(&mut self, _: &mut ActorContext, _: &Throwable) { self.count(2) }
    fn post_stop(&mut self, _: &mut ActorContext) { self.count(3) }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<&str>() == Some(&"crash") {
            self.0.send("crashed".into()).unwrap();
            throw(Throwable::new("java.lang.RuntimeException", Some("Expected")));
        }
        ctx.sender().tell("pong".to_string(), None);
        true
    }
}

#[test]
fn restart_properly_when_same_instance_is_returned() {
    let system = RunningSystem::create("t-life-same");
    let (out, seen) = channel();
    let seen = Seen(seen);
    let test_actor = out.clone();
    let test_actor = system
        .actor_of(Props::new(move || TestActor(test_actor.clone())))
        .unwrap();
    let instance = SameInstance(out, Arc::default());
    let child = Props::new(move || instance.clone());
    let (master, _) = supervised(&system, three_retries(), child);
    seen.expect(&["preStart1"]);
    let ping = || {
        master.tell("ping", Some(&test_actor));
        seen.expect(&["pong"]);
    };
    ping();
    for i in 1..=3 {
        master.tell("crash", Some(&test_actor));
        seen.expect(&[
            "crashed",
            &format!("preRestart{i}"),
            &format!("postRestart{i}"),
        ]);
        ping();
    }
    master.tell("crash", Some(&test_actor));
    seen.expect(&["crashed", "postStop1"]);
    seen.nothing_else();
}

/// `Supervised` of Ticket669Spec.
struct Replies;

impl Actor for Replies {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::new("java.lang.Exception", Some("test")));
    }
    fn pre_restart(&mut self, ctx: &mut ActorContext, _: &Throwable, _: Option<&AnyMessage>) {
        ctx.sender().tell("failure1".to_string(), None);
    }
}

#[test]
fn be_able_to_reply_on_failure_during_pre_restart() {
    let system = RunningSystem::create("t-life-reply");
    let (out, seen) = channel();
    let test_actor = system
        .actor_of(Props::new(move || TestActor(out.clone())))
        .unwrap();
    let strategy = AllForOneStrategy::new(5, |_: &Throwable| Directive::Restart);
    let (_, child) = supervised(&system, strategy, Props::new(|| Replies));
    child.tell("test", Some(&test_actor));
    Seen(seen).expect(&["failure1"]);
}
