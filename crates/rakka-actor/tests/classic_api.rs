//! Public-API contracts of the Classic runtime that the recorded scenarios do not reach directly.
//! Expected values follow Pekko 2.0.0 sources (`DeadLetterActorRef`, `EventStream`, exception formats).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, PoisonPill, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::actor::{ActorError, ErrorClass};
use rakka_actor::dispatch::Terminated;
use rakka_actor::event::{Channel, DeadLetter};

fn wait_until(what: &str, cond: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn settle() {
    std::thread::sleep(Duration::from_millis(150));
}

/// Stores every message of type `T` it receives.
struct Collect<T>(Arc<Mutex<Vec<T>>>);
impl<T: Clone + Send + Sync + 'static> Actor for Collect<T> {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        match msg.downcast_ref::<T>() {
            Some(m) => {
                self.0.lock().unwrap().push(m.clone());
                true
            }
            None => false,
        }
    }
}

fn collector<T: Clone + Send + Sync + 'static>(
    sys: &ActorSystem,
    name: &str,
) -> (ActorRef, Arc<Mutex<Vec<T>>>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    (
        sys.actor_of_named(Props::new(move || Collect(l.clone())), name)
            .unwrap(),
        log,
    )
}

fn shutdown(sys: &ActorSystem) {
    sys.terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated");
}

#[test]
fn exceptions_render_like_java_with_their_jvm_class_names() {
    let cases = [
        (
            ErrorClass::InvalidActorName,
            "org.apache.pekko.actor.InvalidActorNameException",
        ),
        (ErrorClass::IllegalState, "java.lang.IllegalStateException"),
        (
            ErrorClass::IllegalArgument,
            "java.lang.IllegalArgumentException",
        ),
    ];
    for (class, name) in cases {
        assert_eq!(class.java_name(), name);
        assert_eq!(
            ActorError::new(class, "boom").to_string(),
            format!("{name}: boom")
        );
    }
}

#[test]
fn actor_ref_debug_shows_the_full_path() {
    let sys = ActorSystem::create("t-debug").unwrap();
    let (a, _) = collector::<String>(&sys, "a");
    let shown = format!("{a:?}");
    assert!(
        shown.starts_with("Actor[pekko://t-debug/user/a") && shown.ends_with(']'),
        "{shown}"
    );
    assert_eq!(sys.to_string(), "pekko://t-debug");
    shutdown(&sys);
}

#[test]
fn system_exposes_its_name_root_path_settings_and_dead_letters() {
    let settings = Settings {
        parallelism: 3,
        throughput: 7,
    };
    let sys = ActorSystem::create_with_settings("t-meta", settings).unwrap();
    assert_eq!(sys.name(), "t-meta");
    assert_eq!(sys.root_path().to_string(), "pekko://t-meta/");
    assert_eq!(
        (sys.settings().parallelism, sys.settings().throughput),
        (3, 7)
    );
    assert_eq!(
        sys.dead_letters().path().to_string(),
        "pekko://t-meta/deadLetters"
    );
    assert!(
        !sys.dead_letters().is_terminated(),
        "deadLetters never terminates"
    );
    shutdown(&sys);
}

#[test]
fn unsubscribed_actors_stop_receiving_events() {
    let sys = ActorSystem::create("t-unsub").unwrap();
    let (sub, got) = collector::<DeadLetter>(&sys, "sub");
    sys.event_stream().subscribe(&sub, Channel::DeadLetter);
    let ghost = sys
        .actor_of_named(
            Props::new(|| Collect::<String>(Default::default())),
            "ghost",
        )
        .unwrap();
    ghost.tell(PoisonPill, None);
    wait_until("ghost to stop", || ghost.is_terminated());
    ghost.tell("one".to_string(), None);
    wait_until("first dead letter", || got.lock().unwrap().len() == 1);
    sys.event_stream().unsubscribe(&sub);
    ghost.tell("two".to_string(), None);
    settle();
    assert_eq!(got.lock().unwrap().len(), 1, "no event after unsubscribe");
    shutdown(&sys);
}

#[test]
fn dead_letters_republishes_a_dead_letter_as_is_and_ignores_suppressed_messages() {
    let sys = ActorSystem::create("t-dl").unwrap();
    let (sub, got) = collector::<DeadLetter>(&sys, "sub");
    sys.event_stream().subscribe(&sub, Channel::DeadLetter);
    // suppressed (DeadLetterSuppression): PoisonPill is not published
    sys.dead_letters().tell(PoisonPill, None);
    // a DeadLetter sent to deadLetters is published unwrapped
    let inner = DeadLetter {
        message: Arc::new("payload".to_string()),
        sender: sub.clone(),
        recipient: sub.clone(),
    };
    sys.dead_letters().tell(inner, None);
    // anything else is wrapped, with the sender (or deadLetters itself) and deadLetters as recipient
    sys.dead_letters().tell("plain".to_string(), Some(&sub));
    sys.dead_letters().tell("anon".to_string(), None);
    wait_until("3 dead letters", || got.lock().unwrap().len() == 3);
    settle();
    let got = got.lock().unwrap();
    assert_eq!(got.len(), 3, "PoisonPill must not be published");
    let text = |d: &DeadLetter| d.message.downcast_ref::<String>().cloned().unwrap();
    let path = |r: &ActorRef| r.path().to_string();
    assert_eq!(text(&got[0]), "payload");
    assert_eq!(
        (path(&got[0].sender), path(&got[0].recipient)),
        (
            "pekko://t-dl/user/sub".into(),
            "pekko://t-dl/user/sub".into()
        )
    );
    assert_eq!(
        (text(&got[1]), path(&got[1].sender), path(&got[1].recipient)),
        (
            "plain".into(),
            "pekko://t-dl/user/sub".into(),
            "pekko://t-dl/deadLetters".into()
        )
    );
    assert_eq!(
        (text(&got[2]), path(&got[2].sender), path(&got[2].recipient)),
        (
            "anon".into(),
            "pekko://t-dl/deadLetters".into(),
            "pekko://t-dl/deadLetters".into()
        )
    );
    drop(got);
    shutdown(&sys);
}

/// Watches the target it is told about (`Watch(ref)` as a `ActorRef` message).
struct Watcher(Arc<Mutex<Vec<Terminated>>>);
impl Actor for Watcher {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(target) = msg.downcast_ref::<ActorRef>() {
            ctx.watch(target);
            true
        } else if let Some(t) = msg.downcast_ref::<Terminated>() {
            self.0.lock().unwrap().push(t.clone());
            true
        } else {
            false
        }
    }
}

#[test]
fn watching_dead_letters_is_answered_with_an_unconfirmed_terminated() {
    let sys = ActorSystem::create("t-watch-dl").unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let w = sys
        .actor_of_named(Props::new(move || Watcher(l.clone())), "w")
        .unwrap();
    w.tell(sys.dead_letters(), None);
    wait_until("Terminated for deadLetters", || {
        !log.lock().unwrap().is_empty()
    });
    let t = log.lock().unwrap()[0].clone();
    assert_eq!(t.actor, sys.dead_letters());
    assert!(!t.existence_confirmed && !t.address_terminated);
    shutdown(&sys);
}

#[test]
fn stopping_an_actor_that_is_not_top_level_stops_it_directly() {
    struct Parent(Arc<Mutex<Option<ActorRef>>>);
    impl Actor for Parent {
        fn pre_start(&mut self, ctx: &mut ActorContext) {
            *self.0.lock().unwrap() = Some(
                ctx.actor_of_named(Props::new(|| Collect::<String>(Default::default())), "kid")
                    .unwrap(),
            );
        }
    }
    let sys = ActorSystem::create("t-stop-nested").unwrap();
    let slot = Arc::new(Mutex::new(None));
    let s = slot.clone();
    let _p = sys
        .actor_of_named(Props::new(move || Parent(s.clone())), "p")
        .unwrap();
    wait_until("child", || slot.lock().unwrap().is_some());
    let kid = slot.lock().unwrap().clone().unwrap();
    sys.stop(&kid);
    wait_until("child stopped", || kid.is_terminated());
    shutdown(&sys);
}
