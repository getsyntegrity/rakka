// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala, actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala and actor-tests/src/test/scala/org/apache/pekko/actor/LocalActorRefProviderSpec.scala (Apache-2.0).
//! The text of an `ActorRef` and the messages that carry it, with real actors.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::Throwable;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Kill, Props};
use rakka_actor::actor::supervision::{OneForOneStrategy, SupervisorStrategy, default_decider};
use rakka_actor::dispatch::Terminated;
use rakka_actor::event::{Channel, UnhandledMessage};

fn next<T>(receiver: &Receiver<T>) -> T {
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("sent")
}

struct Silent;
impl Actor for Silent {}

/// `ActorRef.toString` (`ActorRef.scala:186-188`).
#[test]
fn the_text_of_a_ref_carries_its_uid_only_when_it_has_one() {
    let system = RunningSystem::create("t-ref-text");
    let child = system
        .actor_of_named(Props::new(|| Silent), "child")
        .unwrap();
    let uid = child.path().uid();

    assert_ne!(uid, 0);
    assert_eq!(
        format!("{child:?}"),
        format!("Actor[pekko://t-ref-text/user/child#{uid}]")
    );
    assert_eq!(
        format!("{:?}", system.dead_letters()),
        "Actor[pekko://t-ref-text/deadLetters]"
    );
    assert_eq!(
        format!("{:?}", system.guardian()),
        "Actor[pekko://t-ref-text/user]"
    );
}

/// Asks the supervisor for a child.
struct Spawn(Props, Sender<ActorRef>);

struct Supervisor(Sender<String>);
impl Actor for Supervisor {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let seen = self.0.clone();
        OneForOneStrategy::new(-1, move |cause: &Throwable| {
            let message = cause.message().unwrap_or_default();
            seen.send(format!("{}: {message}", cause.class())).unwrap();
            default_decider(cause)
        })
        .into()
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(Spawn(props, born)) = msg.downcast_ref::<Spawn>() else {
            return false;
        };
        born.send(ctx.actor_of(props.clone()).unwrap()).unwrap();
        true
    }
}

/// Watches `subject`; with `report` it handles the `Terminated`, without it (`EmptyWatcher`) it does not.
struct Watcher {
    subject: ActorRef,
    watching: Sender<()>,
    report: Option<Sender<String>>,
}
impl Actor for Watcher {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        ctx.watch(&self.subject);
        self.watching.send(()).unwrap();
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        match (&self.report, msg.downcast_ref::<Terminated>()) {
            (Some(report), Some(t)) => {
                report.send(format!("terminated {:?}", t.actor)).unwrap();
                true
            }
            _ => false,
        }
    }
}

fn watcher(subject: &ActorRef, watching: &Sender<()>, report: Option<Sender<String>>) -> Props {
    let (subject, watching) = (subject.clone(), watching.clone());
    Props::new(move || Watcher {
        subject: subject.clone(),
        watching: watching.clone(),
        report: report.clone(),
    })
}

/// `DeathWatchSpec` "fail a monitor which does not handle Terminated()" (`:215-245`, AC-DW-007, AC-CELL-004).
#[test]
fn a_monitor_that_does_not_handle_terminated_fails_with_a_death_pact_and_stops() {
    let system = RunningSystem::create("t-death-pact");
    let (seen, order) = channel();
    let supervisor = {
        let seen = seen.clone();
        system
            .actor_of(Props::new(move || Supervisor(seen.clone())))
            .unwrap()
    };
    let child = |props| {
        let (born, child) = channel();
        supervisor.tell(Spawn(props, born), None);
        next(&child)
    };
    let (watching, ready) = channel();
    let failed = child(Props::new(|| Silent));
    let brother = child(watcher(&failed, &watching, None));
    next(&ready);
    system
        .actor_of(watcher(&brother, &watching, Some(seen.clone())))
        .unwrap();
    next(&ready);

    failed.tell(Kill, None);

    assert_eq!(
        next(&order),
        "org.apache.pekko.actor.ActorKilledException: Kill"
    );
    assert_eq!(
        next(&order),
        format!(
            "org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[{}#{}]] terminated",
            failed.path(),
            failed.path().uid()
        )
    );
    assert_eq!(next(&order), format!("terminated {brother:?}"));
}

struct Unhandled(Sender<String>);
impl Actor for Unhandled {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(u) = msg.downcast_ref::<UnhandledMessage>() else {
            return false;
        };
        let message = u.message.downcast_ref::<&str>().expect("a text");
        // the line `TestEventListener` logs, which the spec filters on
        let line = format!("unhandled message from {:?}: {message}", u.sender);
        self.0
            .send(format!("{} | {line}", u.recipient.path()))
            .unwrap();
        true
    }
}

/// `LocalActorRefProviderSpec` the three guardian specs (`:68-106`, AC-CELL-004).
#[test]
fn a_message_no_guardian_acts_upon_is_published_as_unhandled() {
    let system = RunningSystem::create("t-guardians");
    let (published, events) = channel();
    let subscriber = system
        .actor_of(Props::new(move || Unhandled(published.clone())))
        .unwrap();
    system
        .event_stream()
        .subscribe(&subscriber, Channel::UnhandledMessage);

    for (path, recipient, message) in [
        ("/", "pekko://t-guardians/", "Hello, Mr. Root Guardian"),
        (
            "/user",
            "pekko://t-guardians/user",
            "Hello, Mr. User Guardian",
        ),
        (
            "/system",
            "pekko://t-guardians/system",
            "Hello, Mr. System Guardian",
        ),
    ] {
        system.actor_selection(path).tell(message, None);

        let line =
            format!("unhandled message from Actor[pekko://t-guardians/deadLetters]: {message}");
        assert_eq!(next(&events), format!("{recipient} | {line}"));
    }
}
