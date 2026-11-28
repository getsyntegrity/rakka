// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:421-452,707-724 (Apache-2.0).
//! `NonBlockingBoundedMailbox` behind real actors, through the public API (AC-MBX-008; twin of the queue contracts `NB-QU-01..04`,
//! `NB-QC-01` and of `MailboxConfigSpec`'s "20 enqueue / 10 dequeue => 10 DeadLetter" of the Pekko 2.0.0 oracle).
//!
//! The actor holds its first message inside `receive` until the test releases it, so the mailbox is full without sleeping.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, settle_for_absence, snapshot_of, wait_until};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use rakka_actor::dispatch::message_queue::Envelope;
use rakka_actor::event::{Channel, DeadLetter};

/// `(message, sender path, recipient path)` of a dead letter.
type DeadRecord = (String, String, String);

#[derive(Default)]
struct Gate {
    entered: AtomicBool,
    released: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    fn hold_until_released(&self) {
        self.entered.store(true, Ordering::SeqCst);
        let mut released = self.released.lock().unwrap();
        while !*released {
            released = self.changed.wait(released).unwrap();
        }
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.changed.notify_all();
    }
}

/// Logs every string; the message `"first"` keeps the actor busy until the gate is released.
struct HoldsFirstMessage {
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
}

impl Actor for HoldsFirstMessage {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        if text == "first" {
            self.gate.hold_until_released();
        }
        true
    }
}

struct DeadLetterListener(Arc<Mutex<Vec<DeadRecord>>>);

impl Actor for DeadLetterListener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>()
            && let Some(text) = dead.message.downcast_ref::<String>()
        {
            self.0.lock().unwrap().push((
                text.clone(),
                path_of(&dead.sender),
                path_of(&dead.recipient),
            ));
        }
        true
    }
}

struct Scenario {
    _system: RunningSystem, // terminated when the scenario ends
    actor: ActorRef,
    sender: ActorRef,
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
    dead: Arc<Mutex<Vec<DeadRecord>>>,
}

impl Scenario {
    fn new(name: &str, capacity: i32) -> Scenario {
        let system = RunningSystem::create(name);
        let (gate, received, dead) = (
            Arc::new(Gate::default()),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(Vec::new())),
        );
        let listener = system
            .actor_of_named(
                Props::new({
                    let dead = dead.clone();
                    move || DeadLetterListener(dead.clone())
                }),
                "listener",
            )
            .unwrap();
        system
            .event_stream()
            .subscribe(&listener, Channel::DeadLetter);
        let mailbox = NonBlockingBoundedMailbox::new(capacity).unwrap();
        let actor = system
            .actor_of_named(
                Props::new({
                    let (gate, received) = (gate.clone(), received.clone());
                    move || HoldsFirstMessage {
                        gate: gate.clone(),
                        received: received.clone(),
                    }
                })
                .with_mailbox(mailbox),
                "bounded",
            )
            .unwrap();
        let sender = system
            .actor_of_named(Props::new(|| DeadLetterListener(Arc::default())), "sender")
            .unwrap();
        Scenario {
            _system: system,
            actor,
            sender,
            gate,
            received,
            dead,
        }
    }

    fn tell(&self, text: &str) {
        self.actor.tell(text.to_owned(), Some(&self.sender));
    }

    /// Sends `"first"` and waits until the actor is inside `receive`, so the mailbox itself is empty.
    fn occupy_the_actor(&self) {
        self.tell("first");
        wait_until("the actor to hold its first message", || {
            self.gate.entered.load(Ordering::SeqCst)
        });
    }

    fn dead(&self) -> Vec<DeadRecord> {
        snapshot_of(&self.dead)
    }
}

fn dead_record(text: &str, scenario: &Scenario) -> DeadRecord {
    (
        text.to_owned(),
        path_of(&scenario.sender),
        path_of(&scenario.actor),
    )
}

#[test]
fn a_full_mailbox_refuses_at_once_and_the_dead_letter_names_message_sender_and_receiver() {
    let scenario = Scenario::new("nb-full", 2);
    scenario.occupy_the_actor();

    for text in ["m1", "m2", "m3", "m4"] {
        scenario.tell(text); // returns at once even though the mailbox is full: the sender is never blocked
    }

    wait_until("the two refused messages to be dead-lettered", || {
        scenario.dead().len() == 2
    });
    assert_eq!(
        scenario.dead(),
        vec![dead_record("m3", &scenario), dead_record("m4", &scenario)]
    );
    scenario.gate.release();
    wait_until("the accepted messages to be processed", || {
        scenario.received.lock().unwrap().len() == 3
    });
    assert_eq!(
        snapshot_of(&scenario.received),
        ["first", "m1", "m2"],
        "the first messages are kept, in order"
    );
    settle_for_absence();
    assert_eq!(
        scenario.dead().len(),
        2,
        "no refused message is delivered later or dead-lettered twice"
    );
}

/// `MailboxConfigSpec.scala:85-87,142-192` as `SingleConsumerOnlyMailboxSpec` (`:269-277`) runs it: capacity 10, one producer of 20
/// finished before one consumer starts; the queue is driven directly, with a real `deadLetters`.
#[test]
fn spec_twenty_enqueued_and_ten_dequeued_leave_ten_dead_letters() {
    let scenario = Scenario::new("nb-spec", 1); // its system, dead letter listener and actors; its mailbox is not used here
    let system = &scenario._system;
    let queue = NonBlockingBoundedMailbox::new(10)
        .unwrap()
        .create(system.dead_letters());
    assert_eq!(
        (queue.number_of_messages(), queue.has_messages()),
        (0, false)
    );
    let produced: Vec<String> = (1..=20).map(|i| i.to_string()).collect();

    thread::scope(|scope| {
        scope.spawn(|| {
            for text in &produced {
                let envelope = Envelope {
                    message: Arc::new(text.clone()),
                    sender: Some(scenario.sender.clone()),
                };
                queue.enqueue(&scenario.actor, envelope);
            }
        });
    });
    let consumed: Vec<String> = thread::scope(|scope| {
        let consumer = scope.spawn(|| {
            std::iter::from_fn(|| queue.dequeue())
                .map(|e| e.message.downcast_ref::<String>().unwrap().clone())
                .collect()
        });
        consumer.join().unwrap()
    });

    assert_eq!(consumed, produced[..10], "ten consumed, each once");
    wait_until("the ten dead letters", || scenario.dead().len() == 10);
    settle_for_absence();
    let refused: Vec<DeadRecord> = produced[10..]
        .iter()
        .map(|text| dead_record(text, &scenario))
        .collect();
    assert_eq!(scenario.dead(), refused);
}

struct Ping;

/// Answers each `Ping` to its sender and stops after `left` of them.
struct Ponger {
    left: usize,
}

impl Actor for Ponger {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<Ping>().is_none() {
            return false;
        }
        self.left -= 1;
        ctx.sender().tell(Ping, Some(&ctx.self_ref()));
        if self.left == 0 {
            ctx.stop(&ctx.self_ref());
        }
        true
    }
}

const PINGS: usize = 2_000_000;

/// Creates and watches the two players, serves the first `Ping`, and stops once both are gone.
#[derive(Default)]
struct Referee {
    players: Vec<ActorRef>,
}

impl Actor for Referee {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        for _ in 0..2 {
            let mailbox = NonBlockingBoundedMailbox::new(1).unwrap();
            let props = Props::new(|| Ponger { left: PINGS / 2 }).with_mailbox(mailbox);
            let player = ctx.actor_of(props).unwrap();
            ctx.watch(&player);
            self.players.push(player);
        }
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<Ping>().is_some() {
            self.players[0].tell(Ping, Some(&self.players[1]));
        } else if msg.downcast_ref::<Terminated>().is_some() && ctx.children().is_empty() {
            ctx.stop(&ctx.self_ref());
        }
        true
    }
}

/// `MailboxConfigSpec.scala:325-327`, "support pathological ping-ponging for the bounded case": capacity 1, throughput 1.
#[test]
fn spec_pathological_ping_pong_with_capacity_one_and_throughput_one() {
    let settings = Settings {
        throughput: 1,
        ..Settings::default()
    };
    let system = ActorSystem::create_with_settings("nb-ping-pong", settings).unwrap();
    let referee = system.actor_of(Props::new(Referee::default)).unwrap();

    referee.tell(Ping, None);

    let deadline = Instant::now() + Duration::from_secs(30);
    while !referee.is_terminated() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    let finished = referee.is_terminated();
    let _ = system.terminate().wait(Duration::from_secs(10));
    assert!(finished, "the game did not end within 30 s");
}

#[test]
fn a_mailbox_of_capacity_zero_refuses_every_message() {
    let scenario = Scenario::new("nb-zero", 0);
    for text in ["a", "b", "c"] {
        scenario.tell(text);
    }

    wait_until("every message to be dead-lettered", || {
        scenario.dead().len() == 3
    });
    assert_eq!(
        scenario.dead(),
        vec![
            dead_record("a", &scenario),
            dead_record("b", &scenario),
            dead_record("c", &scenario)
        ]
    );
    settle_for_absence();
    assert!(snapshot_of(&scenario.received).is_empty());
}

#[test]
fn concurrent_senders_lose_and_duplicate_nothing_and_never_exceed_the_capacity() {
    const CAPACITY: usize = 3;
    const PRODUCERS: usize = 4;
    const PER_PRODUCER: usize = 25;
    let scenario = Scenario::new("nb-concurrent", CAPACITY as i32);
    scenario.occupy_the_actor();

    thread::scope(|scope| {
        for producer in 0..PRODUCERS {
            let scenario = &scenario;
            scope.spawn(move || {
                for n in 0..PER_PRODUCER {
                    scenario.tell(&format!("{producer}:{n}"));
                }
            });
        }
    });
    wait_until("the refused messages to be dead-lettered", || {
        scenario.dead().len() == PRODUCERS * PER_PRODUCER - CAPACITY
    });
    scenario.gate.release();
    wait_until("the accepted messages to be processed", || {
        scenario.received.lock().unwrap().len() == 1 + CAPACITY
    });
    settle_for_absence();

    let mut everything: Vec<String> = snapshot_of(&scenario.received)
        .into_iter()
        .filter(|text| text != "first")
        .chain(scenario.dead().into_iter().map(|(text, _, _)| text))
        .collect();
    everything.sort();
    let mut sent: Vec<String> = (0..PRODUCERS)
        .flat_map(|p| (0..PER_PRODUCER).map(move |n| format!("{p}:{n}")))
        .collect();
    sent.sort();
    assert_eq!(
        everything, sent,
        "each message is either received or dead-lettered, exactly once"
    );
}
