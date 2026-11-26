// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala (Apache-2.0).
//! `context.watchWith` with a user message type through the public API (AC-DW-005). The equality that differs on purpose from
//! Scala (Q-DW-3 (a)) is unit tested in `actor::watching`; the parity traces are `cases/scenario-watch-with.jsonl`.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, settle_for_absence, snapshot_of, wait_until};

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::Throwable;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};

#[derive(PartialEq)]
struct JobDone(&'static str);

type Log = Arc<Mutex<Vec<String>>>;

/// "ok", or the message of the `IllegalStateException` that was thrown.
fn outcome(registration: impl FnOnce()) -> String {
    match catch_unwind(AssertUnwindSafe(registration)) {
        Ok(()) => "ok".to_owned(),
        Err(thrown) => {
            let thrown = thrown.downcast_ref::<Throwable>().expect("a Throwable");
            assert_eq!(thrown.class, "java.lang.IllegalStateException");
            thrown.message.clone().unwrap_or_default()
        }
    }
}

/// Told its subject, registers it several times; then logs what it receives.
struct Watcher(Log);

impl Actor for Watcher {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let mut log = self.0.lock().unwrap();
        if let Some(subject) = msg.downcast_ref::<ActorRef>() {
            log.push(outcome(|| ctx.watch_with(subject, JobDone("a"))));
            // an equal message built separately
            log.push(outcome(|| ctx.watch_with(subject, JobDone("a"))));
            log.push(outcome(|| ctx.watch_with(subject, JobDone("b"))));
            log.push(outcome(|| ctx.watch(subject)));
        } else {
            let done = msg.downcast_ref::<JobDone>();
            log.push(done.map_or("other".to_owned(), |done| format!("done {}", done.0)));
        }
        true
    }
}

#[test]
fn an_incompatible_registration_throws_and_the_first_message_is_delivered_once() {
    let system = RunningSystem::create("t-watch-with");
    let log = Log::default();
    let recorded = log.clone();
    let watcher = system
        .actor_of(Props::new(move || Watcher(recorded.clone())))
        .unwrap();
    let subject = system
        .actor_of(Props::new(|| Watcher(Log::default())))
        .unwrap();

    watcher.tell(subject.clone(), None);
    wait_until("the registrations", || log.lock().unwrap().len() == 4);
    system.stop(&subject);
    wait_until("the termination message", || log.lock().unwrap().len() == 5);
    settle_for_absence();

    let job = std::any::type_name::<JobDone>();
    let reference = |r: &ActorRef| format!("Actor[{}]", r.path().to_serialization_format());
    let thrown = |new: &str| {
        format!(
            "Watch({}, {}) termination message was not overwritten from [Some({job})] to [{new}]. \
             If this was intended, unwatch first before using `watch` / `watchWith` with another message.",
            reference(&watcher),
            reference(&subject),
        )
    };
    let (other, plain) = (thrown(&format!("Some({job})")), thrown("None"));
    assert_eq!(snapshot_of(&log), ["ok", "ok", &other, &plain, "done a"]);
}
