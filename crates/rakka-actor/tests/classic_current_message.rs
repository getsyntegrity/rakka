// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala (Apache-2.0).
//! `currentMessage` in the lifecycle hooks (AC-CELL-007.1, AC-CELL-020): a sibling `s` restarts `p` without `p` running a message.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, wait_until};

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Identify, PoisonPill, Props};
use rakka_actor::actor::supervision::{
    AllForOneStrategy, Directive, OneForOneStrategy, SupervisorStrategy,
};
use rakka_actor::dispatch::Failed;
use rakka_actor::event::{Channel, ErrorEvent};

struct Fragile;

#[rustfmt::skip] // one stub per line
impl Drop for Fragile {
    fn drop(&mut self) { panic!("drop of a user message") }
}

#[rustfmt::skip]
fn fail(text: &str) -> ! { throw(Throwable::illegal_state(text)) }

struct Throws;

#[rustfmt::skip] // one stub per line
impl Actor for Throws {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool { fail(msg.downcast_ref::<&str>().unwrap()) }
}

/// Reports what it hears: the identity of an `Identify`, and the `PreRestartException` of an `Error`.
struct Probe(Sender<String>);

#[rustfmt::skip] // one case per line
impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<ErrorEvent>() {
            Some(e) if e.cause.class.ends_with("PreRestartException") => self.0.send(e.cause.to_string()).unwrap(),
            Some(_) => {}
            None => self.0.send("identity".into()).unwrap(),
        }
        true
    }
}

/// `p`: reports what its hooks see. The flag makes `preRestart` throw.
struct Subject(Sender<String>, bool);

impl Subject {
    fn report(&self, what: &str, ctx: &ActorContext) {
        let sender = ctx.sender().path().name().to_string();
        self.0.send(format!("{what} {sender}")).unwrap();
    }
}

#[rustfmt::skip] // one hook per line
impl Actor for Subject {
    fn pre_start(&mut self, ctx: &mut ActorContext) { ctx.actor_of_named(Props::new(|| Throws), "k").unwrap(); }
    fn post_restart(&mut self, ctx: &mut ActorContext, _: &Throwable) { self.report("postRestart", ctx) }
    fn post_stop(&mut self, ctx: &mut ActorContext) { self.report("postStop", ctx) }
    fn pre_restart(&mut self, ctx: &mut ActorContext, _: &Throwable, message: Option<&AnyMessage>) {
        let message = message.map_or("None", |message| match message.is::<Failed>() {
            true => "Failed",
            false => message.downcast_ref::<&str>().copied().unwrap_or("?"),
        });
        self.report(&format!("preRestart {message}"), ctx);
        if self.1 {
            fail("hook");
        }
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<&str>().copied() {
            None => fail("fragile"),
            Some("boom") => fail("boom"),
            Some("fail child") => ctx.child("k").unwrap().tell("kboom", None),
            Some(other) => {
                self.0.send(other.into()).unwrap();
                return other != "unhandled";
            }
        }
        true
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let out = self.0.clone();
        let decide = move |_: &Throwable| {
            out.send("decided".into()).unwrap();
            Directive::Resume
        };
        OneForOneStrategy::new(-1, decide).into()
    }
}

/// `g`: all-for-one; restarts for the failure of `s` and answers any other with the directive.
struct Supervisor(Sender<String>, Sender<[ActorRef; 2]>, Directive, bool);

impl Actor for Supervisor {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        let (out, hook_throws) = (self.0.clone(), self.3);
        let p = Props::new(move || Subject(out.clone(), hook_throws));
        let p = ctx.actor_of_named(p, "p").unwrap();
        let s = ctx.actor_of_named(Props::new(|| Throws), "s").unwrap();
        self.1.send([p, s]).unwrap();
    }
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let (out, other) = (self.0.clone(), self.2);
        let decide = move |cause: &Throwable| {
            out.send("supervised".into()).unwrap();
            let sibling = cause.message.as_deref() == Some("sibling");
            if sibling { Directive::Restart } else { other }
        };
        AllForOneStrategy::new(-1, decide).into()
    }
}

const WAIT: Duration = Duration::from_secs(10);
#[rustfmt::skip] // the parts of one fixture
struct Family { system: RunningSystem, p: ActorRef, s: ActorRef, probe: ActorRef, seen: Receiver<String> }

#[rustfmt::skip] // one field per line is not wanted in the constructor
impl Family {
    fn new(name: &str, directive: Directive, hook_throws: bool) -> Self {
        let system = RunningSystem::create(name);
        let ((out, seen), (refs, children)) = (channel(), channel());
        let heard = out.clone();
        let probe = system.actor_of_named(Props::new(move || Probe(heard.clone())), "probe").unwrap();
        let g = Props::new(move || Supervisor(out.clone(), refs.clone(), directive, hook_throws));
        system.actor_of_named(g, "g").unwrap();
        let [p, s] = children.recv_timeout(WAIT).unwrap();
        Family { system, p, s, probe, seen }
    }

    fn next(&self) -> String {
        self.seen.recv_timeout(WAIT).expect("event")
    }

    fn expect(&self, events: &[&str]) {
        events.iter().for_each(|event| assert_eq!(self.next(), *event));
    }

    /// `p` handles the failure of its child and stays healthy; the decision is taken while the `Failed` is being processed.
    fn handled_failure(name: &str, hook_throws: bool) -> Self {
        let family = Family::new(name, Directive::Resume, hook_throws);
        family.p.tell("fail child", None);
        family.expect(&["decided"]);
        family
    }

    fn sibling_fails(&self) {
        self.s.tell("sibling", None);
        self.expect(&["supervised"]);
    }
}

#[test]
fn a_handled_failure_stays_current_until_a_restart_forgets_it() {
    let family = Family::handled_failure("t-current-handled", false);
    family.sibling_fails();
    family.expect(&["preRestart Failed k", "postRestart deadLetters"]);
}

#[test]
fn post_stop_reads_the_sender_of_the_current_message() {
    let family = Family::handled_failure("t-current-stop", false);
    family.system.stop(&family.p);
    family.expect(&["postStop k"]);

    let family = Family::handled_failure("t-current-pill", false);
    family.p.tell(PoisonPill, Some(&family.probe));
    family.expect(&["postStop deadLetters"]);

    let family = Family::new("t-current-stopped", Directive::Stop, false);
    let message: AnyMessage = Arc::new("boom");
    let held = Arc::downgrade(&message);
    family.p.tell_any(message, Some(family.probe.clone()));
    family.expect(&["supervised", "postStop probe"]);
    wait_until("the stopped actor to release the message", || {
        held.upgrade().is_none()
    });
}

#[test]
fn a_successful_message_forgets_the_current_one() {
    #[rustfmt::skip] // message and what shows it arrived
    let messages: [(AnyMessage, &str); 3] = [(Arc::new("ok"), "ok"), (Arc::new("unhandled"), "unhandled"),
        (Arc::new(Identify { message_id: Arc::new(1) }), "identity")];
    for (message, heard) in messages {
        let family = Family::handled_failure("t-current-success", false);
        family.p.tell_any(message, Some(family.probe.clone()));
        family.expect(&[heard]);
        family.sibling_fails();
        family.expect(&["preRestart None deadLetters"]);
    }
}

#[test]
fn resume_keeps_the_failed_message_and_its_sender() {
    let family = Family::new("t-current-resume", Directive::Resume, false);
    family.p.tell("boom", Some(&family.probe));
    family.expect(&["supervised"]);
    family.sibling_fails();
    family.expect(&["preRestart boom probe"]);
}

#[test]
fn a_throwing_pre_restart_names_the_class_of_the_current_message() {
    let family = Family::handled_failure("t-current-hook", true);
    let events = family.system.event_stream();
    events.subscribe(&family.probe, Channel::Error);
    family.sibling_fails();
    family.expect(&["preRestart Failed k"]);
    let mut unordered = [family.next(), family.next()]; // the `Error` reaches the probe while `p` restarts
    unordered.sort();
    let error = "org.apache.pekko.actor.PreRestartException: pekko://t-current-hook/user/g/p: exception in preRestart(\
                 class java.lang.IllegalStateException, class org.apache.pekko.dispatch.sysmsg.Failed)";
    assert_eq!(unordered, [error, "postRestart deadLetters"]);
}

/// Rust only: the cell forgets the message outside its state lock, and a panicking `Drop` stops there.
#[test]
fn a_failed_message_whose_drop_panics_does_not_wedge_the_actor() {
    for directive in [Directive::Restart, Directive::Resume] {
        let family = Family::new("t-current-drop", directive, false);
        family.p.tell(Fragile, None);
        family.expect(&["supervised"]);
        if directive == Directive::Restart {
            family.expect(&["preRestart ? deadLetters", "postRestart deadLetters"]);
        }
        family.p.tell("ping", None);
        family.expect(&["ping"]);
    }
}
