//! Slice 2b through the public API: tell, sender, ordering, PoisonPill, dead letters, unhandled, throughput.
//! Expected values were observed from Pekko 2.0.0 (see `cases/scenario-2b.jsonl`, `rust/app-cases/`).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, PoisonPill, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::event::{Channel, DeadLetter, UnhandledMessage};

type Log = Arc<Mutex<Vec<(String, String)>>>; // (message, sender path)

/// Records every `String` it receives with its sender.
struct Recorder(Log);
impl Actor for Recorder {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(m) = msg.downcast_ref::<String>() {
            self.0
                .lock()
                .unwrap()
                .push((m.clone(), path_of(&ctx.sender())));
            true
        } else {
            false
        }
    }
}

fn recorder(sys: &ActorSystem, name: &str) -> (ActorRef, Log) {
    let log: Log = Log::default();
    let l = log.clone();
    (
        sys.actor_of_named(Props::new(move || Recorder(l.clone())), name)
            .unwrap(),
        log,
    )
}

#[test]
fn receiver_sees_explicit_sender_or_dead_letters() {
    let sys = RunningSystem::create("t-sender");
    let (a, log) = recorder(&sys, "a");
    let (b, _) = recorder(&sys, "b");
    a.tell("no-sender".to_string(), None);
    a.tell("from-b".to_string(), Some(&b));
    wait_until("2 messages", || log.lock().unwrap().len() == 2);
    assert_eq!(
        *log.lock().unwrap(),
        vec![
            (
                "no-sender".to_string(),
                "pekko://t-sender/deadLetters".to_string()
            ),
            ("from-b".to_string(), "pekko://t-sender/user/b".to_string()),
        ]
    );
}

#[test]
fn messages_from_one_sender_arrive_in_order() {
    let sys = RunningSystem::create("t-order");
    let (a, log) = recorder(&sys, "a");
    for i in 0..2000 {
        a.tell(format!("m-{i}"), None);
    }
    wait_until("2000 messages", || log.lock().unwrap().len() == 2000);
    let got: Vec<String> = log.lock().unwrap().iter().map(|(m, _)| m.clone()).collect();
    assert_eq!(got, (0..2000).map(|i| format!("m-{i}")).collect::<Vec<_>>());
}

#[test]
fn poison_pill_stops_after_earlier_messages_and_later_ones_are_dead_letters() {
    let sys = RunningSystem::create("t-pill");
    let (a, log) = recorder(&sys, "a");
    let (collector, dead) = recorder(&sys, "dl");
    // dead letters arrive as `DeadLetter` events: a tiny adapter actor turns them into strings
    struct DlAdapter(ActorRef);
    impl Actor for DlAdapter {
        fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
            if let Some(d) = msg.downcast_ref::<DeadLetter>() {
                let text = d
                    .message
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_default();
                let note = format!("{text}|{}|{}", path_of(&d.sender), path_of(&d.recipient));
                self.0.tell(note, None);
                true
            } else {
                false
            }
        }
    }
    let c2 = collector.clone();
    let adapter = sys
        .actor_of(Props::new(move || DlAdapter(c2.clone())))
        .unwrap();
    sys.event_stream().subscribe(&adapter, Channel::DeadLetter);

    a.tell("m1".to_string(), None);
    a.tell("m2".to_string(), None);
    a.tell(PoisonPill, None);
    wait_until("termination", || a.is_terminated());
    a.tell("late".to_string(), None);
    a.tell(PoisonPill, None); // DeadLetterSuppression: no event
    wait_until("dead letter", || !dead.lock().unwrap().is_empty());
    assert_eq!(log.lock().unwrap().len(), 2);
    assert_eq!(
        dead.lock().unwrap()[0].0,
        "late|pekko://t-pill/deadLetters|pekko://t-pill/user/a"
    );
}

#[test]
fn unhandled_messages_are_published_with_sender_and_recipient() {
    struct Picky;
    impl Actor for Picky {} // handles nothing
    let sys = RunningSystem::create("t-unh");
    let seen: Arc<(Mutex<Vec<String>>, Condvar)> = Arc::default();
    struct Watcher(Arc<(Mutex<Vec<String>>, Condvar)>);
    impl Actor for Watcher {
        fn receive(&mut self, _c: &mut ActorContext, msg: &AnyMessage) -> bool {
            let Some(u) = msg.downcast_ref::<UnhandledMessage>() else {
                return false;
            };
            self.0.0.lock().unwrap().push(format!(
                "{}|{}",
                path_of(&u.sender),
                path_of(&u.recipient)
            ));
            self.0.1.notify_all();
            true
        }
    }
    let s2 = seen.clone();
    let w = sys
        .actor_of(Props::new(move || Watcher(s2.clone())))
        .unwrap();
    sys.event_stream().subscribe(&w, Channel::UnhandledMessage);
    let p = sys.actor_of_named(Props::new(|| Picky), "p").unwrap();
    p.tell("x".to_string(), None);
    wait_until("unhandled event", || !seen.0.lock().unwrap().is_empty());
    assert_eq!(
        seen.0.lock().unwrap()[0],
        "pekko://t-unh/deadLetters|pekko://t-unh/user/p"
    );
}

#[test]
fn single_thread_dispatcher_runs_throughput_messages_per_turn() {
    // observed on Pekko 2.0.0: with throughput 3, two actors alternate in turns of 3 messages
    struct Gate(Arc<(Mutex<bool>, Condvar)>, Arc<AtomicUsize>);
    impl Actor for Gate {
        fn pre_start(&mut self, _c: &mut ActorContext) {
            self.1.fetch_add(1, Ordering::SeqCst);
        }
        fn receive(&mut self, _c: &mut ActorContext, _m: &AnyMessage) -> bool {
            let g = self.0.0.lock().unwrap();
            drop(self.0.1.wait_while(g, |open| !*open).unwrap());
            true
        }
    }
    struct Tagged(&'static str, Arc<Mutex<Vec<String>>>, Arc<AtomicUsize>);
    impl Actor for Tagged {
        fn pre_start(&mut self, _c: &mut ActorContext) {
            self.2.fetch_add(1, Ordering::SeqCst);
        }
        fn receive(&mut self, _c: &mut ActorContext, m: &AnyMessage) -> bool {
            self.1.lock().unwrap().push(format!(
                "{}{}",
                self.0,
                m.downcast_ref::<usize>().unwrap()
            ));
            true
        }
    }
    let sys = RunningSystem::create_with_settings(
        "t-tput",
        Settings {
            parallelism: 1,
            throughput: 3,
            ..Settings::default()
        },
    );
    let order = Arc::new(Mutex::new(Vec::new()));
    let gate: Arc<(Mutex<bool>, Condvar)> = Arc::default();
    let g2 = gate.clone();
    let started = Arc::new(AtomicUsize::new(0));
    let (s_g, s_a, s_b) = (started.clone(), started.clone(), started.clone());
    let g = sys
        .actor_of(Props::new(move || Gate(g2.clone(), s_g.clone())))
        .unwrap();
    let (o_a, o_b) = (order.clone(), order.clone());
    let a = sys
        .actor_of(Props::new(move || Tagged("a", o_a.clone(), s_a.clone())))
        .unwrap();
    let b = sys
        .actor_of(Props::new(move || Tagged("b", o_b.clone(), s_b.clone())))
        .unwrap();
    // pre-start barrier: all three actors are created before the first message (as in the recorded scenarios)
    wait_until("three actors started", || {
        started.load(Ordering::SeqCst) == 3
    });
    g.tell(0usize, None);
    for i in 0..7usize {
        a.tell(i, None);
    }
    for i in 0..7usize {
        b.tell(i, None);
    }
    *gate.0.lock().unwrap() = true;
    gate.1.notify_all();
    wait_until("14 messages", || order.lock().unwrap().len() == 14);
    assert_eq!(
        order.lock().unwrap().join(" "),
        "a0 a1 a2 b0 b1 b2 a3 a4 a5 b3 b4 b5 a6 b6"
    );
}
