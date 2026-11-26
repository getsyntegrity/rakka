// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRef.scala and actor-tests/src/test/scala/org/apache/pekko/actor/ActorSelectionSpec.scala (Apache-2.0).
//! What an actor selection that matches nothing does, and `Identify` through a selection. Expected values were observed from
//! Pekko 2.0.0.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{
    Actor, ActorContext, ActorIdentity, AnyMessage, Identify, PoisonPill, Props,
};
use rakka_actor::event::{Channel, DeadLetter, UnhandledMessage};

type Log = Arc<Mutex<Vec<String>>>;

fn short(actor: &ActorRef) -> String {
    path_of(actor).replace("pekko://sel", "")
}

fn text(msg: &AnyMessage) -> &str {
    msg.downcast_ref::<&str>().copied().unwrap_or("?")
}

/// Records dead letters, unhandled messages and `ActorIdentity` replies; tells the recipient of the dead letter `first` two wrapped dead letters and then `barrier`.
struct Recorder(Log);

impl Actor for Recorder {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let line = if let Some(dead) = msg.downcast_ref::<DeadLetter>() {
            if text(&dead.message) == "first" {
                let wrapped = |message: AnyMessage| DeadLetter {
                    message,
                    sender: ctx.self_ref(),
                    recipient: dead.recipient.clone(),
                };
                dead.recipient.tell(wrapped(Arc::new("wrapped")), None);
                let identify = Identify {
                    message_id: Arc::new("inner"),
                };
                dead.recipient.tell(wrapped(Arc::new(identify)), None);
                dead.recipient.tell("barrier", None);
            }
            let (message, from) = (text(&dead.message), short(&dead.sender));
            let uid = dead.recipient.path().uid();
            let mut to = short(&dead.recipient);
            if uid != 0 && to != "/deadLetters" {
                to = format!("{to} uid={uid}");
            }
            format!("dead {message} from {from} to {to}")
        } else if let Some(unhandled) = msg.downcast_ref::<UnhandledMessage>() {
            let (message, from) = (text(&unhandled.message), short(&unhandled.sender));
            format!(
                "unhandled {message} from {from} to {}",
                short(&unhandled.recipient)
            )
        } else {
            let identity = msg.downcast_ref::<ActorIdentity>().unwrap();
            let found = identity.actor_ref.as_ref().map(short);
            format!(
                "{} {} from {}",
                text(&identity.correlation_id),
                found.as_deref().unwrap_or("-"),
                short(&ctx.sender())
            )
        };
        self.0.lock().unwrap().push(line);
        true
    }
}

/// A system with `/user/r`, which records the replies sent to it and what the event stream `channel` publishes.
fn probe(channel: Channel) -> (RunningSystem, ActorRef, Log) {
    let system = RunningSystem::create("sel");
    let log = Log::default();
    let recorded = log.clone();
    let probe = system
        .actor_of_named(Props::new(move || Recorder(recorded.clone())), "r")
        .unwrap();
    system.event_stream().subscribe(&probe, channel);
    (system, probe, log)
}

fn lines(log: &Log, count: usize) -> Vec<String> {
    wait_until("recorded lines", || log.lock().unwrap().len() >= count);
    log.lock().unwrap().clone()
}

#[test]
fn what_matches_nothing_is_a_dead_letter_and_a_suppressed_message_is_not_one() {
    let (system, probe, log) = probe(Channel::DeadLetter);
    for (path, message) in [
        ("/user/missing", "a"),
        ("/user/r/missing/deeper", "b"),
        ("user/r/../x", "c"),
        ("pekko://other/user/r", "d"),
        ("pekko://sys2/user/r", "e"),
        ("not a path", "f"),
        ("/deadLetters/x", "g"),
        ("/deadLetters", "h"),
        // a URI reference whose authority is `user`: the path is `/r`
        ("//user/r", "i"),
        // another incarnation of an existing actor
        ("/user/r#999999999", "stale"),
    ] {
        system.actor_selection(path).tell(message, Some(&probe));
    }
    let missing = system.actor_selection("/user/missing");
    missing.tell(PoisonPill, Some(&probe));
    missing.tell("last", None);

    assert_eq!(
        lines(&log, 11),
        [
            "dead a from /user/r to /user/missing",
            "dead b from /user/r to /user/r/missing/deeper",
            "dead c from /user/r to /user/r/../x",
            "dead d from /user/r to /deadLetters",
            "dead e from /user/r to /deadLetters",
            "dead f from /user/r to /deadLetters",
            "dead g from /user/r to /deadLetters",
            "dead h from /user/r to /deadLetters",
            "dead i from /user/r to /r",
            "dead stale from /user/r to /user/r uid=999999999",
            "dead last from /deadLetters to /user/missing",
        ]
    );
}

#[test]
fn the_recipient_of_a_missed_selection_dead_letters_what_it_is_told_but_not_a_dead_letter() {
    let (system, _probe, log) = probe(Channel::DeadLetter);
    system.actor_selection("/user/no/one").tell("first", None);

    assert_eq!(
        lines(&log, 3),
        [
            "dead first from /deadLetters to /user/no/one",
            "inner - from /deadLetters",
            "dead barrier from /deadLetters to /user/no/one",
        ]
    );
}

#[test]
fn the_empty_path_is_the_root_guardian() {
    let (system, probe, log) = probe(Channel::UnhandledMessage);
    system.actor_selection("").tell("root", Some(&probe));

    assert_eq!(lines(&log, 1), ["unhandled root from /user/r to /"]);
}

#[test]
fn identify_through_a_selection_answers_with_the_actor_or_none() {
    let (system, probe, log) = probe(Channel::DeadLetter);
    let paths = [
        ("/user/r", "found"),
        ("/user/missing", "none"),
        ("/user/r/no/deeper", "deep"),
        ("pekko://other/user/r", "foreign"),
        ("not a path", "unparsed"),
        ("/deadLetters", "dead"),
        ("/deadLetters/x", "under-dead"),
    ];
    for (answered, (path, correlation)) in paths.into_iter().enumerate() {
        let identify = Identify {
            message_id: Arc::new(correlation),
        };
        system.actor_selection(path).tell(identify, Some(&probe));
        lines(&log, answered + 1);
    }

    // a path that ends at `deadLetters` itself answers with the asker as sender, as `deadLetters ! Identify` does
    assert_eq!(
        lines(&log, 7),
        [
            "found /user/r from /deadLetters",
            "none - from /deadLetters",
            "deep - from /deadLetters",
            "foreign - from /deadLetters",
            "unparsed - from /user/r",
            "dead - from /user/r",
            "under-dead - from /deadLetters",
        ]
    );
}
