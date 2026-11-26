// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala, ActorCell.scala and ActorRef.scala (Apache-2.0).
//! `Identify` / `ActorIdentity` through the public API. Expected values were observed from Pekko 2.0.0
//! (see `cases/scenario-identify.jsonl`, `rust/app-cases/identify-*`).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, ActorIdentity, AnyMessage, Identify, Props};
use rakka_actor::actor::system::ActorSystem;

/// (correlation id, identified actor, sender of the reply)
type Replies = Arc<Mutex<Vec<(String, Option<ActorRef>, String)>>>;

/// Records every `ActorIdentity` and counts any other message as `receive` seeing it.
struct Asker(Replies);
impl Actor for Asker {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(reply) = msg.downcast_ref::<ActorIdentity>() else {
            return false;
        };
        let correlation = reply.correlation_id.downcast_ref::<&str>().unwrap();
        self.0.lock().unwrap().push((
            correlation.to_string(),
            reply.actor_ref.clone(),
            path_of(&ctx.sender()),
        ));
        true
    }
}

/// Records whether an `Identify` ever reached its `receive`.
struct Target(Arc<Mutex<Vec<bool>>>);
impl Actor for Target {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        self.0.lock().unwrap().push(msg.is::<Identify>());
        true
    }
}

fn identify(correlation: &'static str) -> Identify {
    Identify {
        message_id: Arc::new(correlation),
    }
}

fn asker(system: &ActorSystem) -> (ActorRef, Replies) {
    let replies = Replies::default();
    let recorded = replies.clone();
    let asker = system.actor_of_named(Props::new(move || Asker(recorded.clone())), "asker");
    (asker.unwrap(), replies)
}

#[test]
fn a_live_actor_answers_with_its_own_reference_and_never_receives_the_identify() {
    let system = RunningSystem::create("t-identify-live");
    let (asker, replies) = asker(&system);
    let seen: Arc<Mutex<Vec<bool>>> = Arc::default();
    let recorded = seen.clone();
    let target = system
        .actor_of(Props::new(move || Target(recorded.clone())))
        .unwrap();

    target.tell(identify("live"), Some(&asker));
    target.tell("after", None);

    wait_until("reply and next message", || {
        replies.lock().unwrap().len() == 1 && seen.lock().unwrap().len() == 1
    });
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert_eq!(
        *replies.lock().unwrap(),
        vec![(
            "live".to_string(),
            Some(target.clone()),
            "pekko://t-identify-live/deadLetters".to_string()
        )]
    );
}

#[test]
fn a_stopped_actor_and_dead_letters_answer_that_there_is_no_actor() {
    let system = RunningSystem::create("t-identify-gone");
    let (asker, replies) = asker(&system);
    let target = system
        .actor_of(Props::new(|| Target(Arc::default())))
        .unwrap();
    system.stop(&target);
    wait_until("target stopped", || target.is_terminated());

    target.tell(identify("stopped"), Some(&asker));
    wait_until("first reply", || replies.lock().unwrap().len() == 1);
    system.dead_letters().tell(identify("dead"), Some(&asker));
    wait_until("second reply", || replies.lock().unwrap().len() == 2);

    // `deadLetters ! Identify` replies with the asker itself as sender; a stopped actor replies with none.
    assert_eq!(
        *replies.lock().unwrap(),
        vec![
            (
                "stopped".to_string(),
                None,
                "pekko://t-identify-gone/deadLetters".to_string()
            ),
            ("dead".to_string(), None, path_of(&asker)),
        ]
    );
}
