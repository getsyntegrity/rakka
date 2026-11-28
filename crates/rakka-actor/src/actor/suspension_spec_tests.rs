// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorHierarchySpec.scala (Apache-2.0).
//! `SupervisorHierarchySpec` "suspend children while failing" (AC-SUP-010 A1): while the supervisor of an ancestor decides, the worker's
//! mailbox is suspended and it answers nothing. In the crate to read the mailbox state. Integration: real actors and dispatcher.

use crate::actor::actor_ref::ActorRef;
use crate::actor::async_creation_tests::wait_until;
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use crate::actor::system::ActorSystem;
use crate::event::{Channel, WarningEvent};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn resume_all() -> SupervisorStrategy {
    OneForOneStrategy::new(-1, |_: &Throwable| Directive::Resume).into()
}

/// The spec's `Resumer`.
struct Resumer;

impl Actor for Resumer {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        resume_all()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<&str>() {
            Some(&"spawn") => {
                let child = ctx.actor_of(Props::new(|| Resumer)).unwrap();
                ctx.sender().tell(child, None);
            }
            Some(&"fail") => throw(Throwable::new("java.lang.Exception", Some("expected"))),
            Some(&"ping") => ctx.sender().tell("pong", None),
            _ => return false,
        }
        true
    }
}

/// `slowResumer`: its decision waits for the latch (at most 4 s, as the spec), then resumes.
struct SlowResumer(Arc<Mutex<Receiver<()>>>);

impl Actor for SlowResumer {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let latch = self.0.clone();
        OneForOneStrategy::new(-1, move |_: &Throwable| {
            let waited = latch.lock().unwrap().recv_timeout(Duration::from_secs(4));
            assert!(!matches!(waited, Err(RecvTimeoutError::Timeout)));
            Directive::Resume
        })
        .into()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<&str>() != Some(&"spawn") {
            return false;
        }
        let child = ctx.actor_of(Props::new(|| Resumer)).unwrap();
        ctx.sender().tell(child, None);
        true
    }
}

/// The spec's `testActor`: forwards what it receives to the test.
struct Probe(Sender<AnyMessage>);

impl Actor for Probe {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0.send(msg.clone()).unwrap();
        true
    }
}

struct TestActor(ActorRef, Receiver<AnyMessage>);

impl TestActor {
    fn next(&self) -> AnyMessage {
        self.1.recv_timeout(Duration::from_secs(3)).unwrap()
    }

    fn spawn_below(&self, parent: &ActorRef) -> ActorRef {
        parent.tell("spawn", Some(&self.0));
        self.next().downcast_ref::<ActorRef>().unwrap().clone()
    }

    fn expect_pong(&self) {
        assert_eq!(self.next().downcast_ref::<&str>(), Some(&"pong"));
    }
}

#[test]
fn suspend_children_while_failing() {
    let system = ActorSystem::create("t-suspend").unwrap();
    let (out, replies) = channel();
    let probe = system.actor_of(Props::new(move || Probe(out.clone())));
    let test_actor = TestActor(probe.unwrap(), replies);
    // `EventFilter.warning("expected", occurrences = 1)`
    let (warned, warnings) = channel();
    let listener = system.actor_of(Props::new(move || Probe(warned.clone())));
    let events = system.event_stream();
    events.subscribe(&listener.unwrap(), Channel::Warning);
    let (latch, opened) = channel();
    let opened = Arc::new(Mutex::new(opened));
    let props = Props::new(move || SlowResumer(opened.clone()));
    let slow_resumer = system.actor_of_named(props, "slowResumer").unwrap();
    let boss = test_actor.spawn_below(&slow_resumer);
    let middle = test_actor.spawn_below(&boss);
    let worker = test_actor.spawn_below(&middle);
    worker.tell("ping", Some(&test_actor.0));
    test_actor.expect_pong();
    boss.tell("fail", None);
    wait_until("the worker is suspended", || {
        let view = worker.mailbox_view();
        view.contains("suspendCount=") && !view.contains("suspendCount=0 ")
    });
    worker.tell("ping", Some(&test_actor.0));
    // `expectNoMessage(2.seconds)`: a bounded wait for an absence
    let silence = test_actor.1.recv_timeout(Duration::from_secs(2));
    assert!(matches!(silence, Err(RecvTimeoutError::Timeout)));
    latch.send(()).unwrap();
    test_actor.expect_pong();
    let expected = |event: AnyMessage| {
        let warning = event.downcast_ref::<WarningEvent>().unwrap();
        warning.message.as_deref() == Some("expected")
    };
    assert!(expected(
        warnings.recv_timeout(Duration::from_secs(3)).unwrap()
    ));
    assert!(!warnings.try_iter().any(expected));
}
