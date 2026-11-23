// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:421-452,707-724 and Dispatcher.scala:80-171 (Apache-2.0).
//! `NonBlockingBoundedMailbox` inside the cell (AC-MBX-008): what only the runtime can show -- no consumption turn for a refused
//! message, what stop and restart do with the queue, and that the queue does not keep its actor alive.

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use crate::dispatch::dispatcher::{Executor, Rejected, ThreadPool};
use crate::event::{Channel, DeadLetter};
use crate::util::clock::NanoClock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// The thread pool, counting every mailbox turn submitted for the actor `worker` (the collaborator that makes turns observable).
struct CountsTurns {
    pool: ThreadPool,
    submissions: AtomicUsize,
}

impl Executor for CountsTurns {
    fn execute(&self, cell: Arc<ActorCell>) -> Result<(), Rejected> {
        if cell.self_ref().path().name() == "worker" {
            self.submissions.fetch_add(1, Ordering::SeqCst);
        }
        self.pool.execute(cell)
    }
    fn shutdown(&self) {
        self.pool.shutdown();
    }
}

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

type Log = Arc<Mutex<Vec<String>>>;
type DeadLog = Arc<Mutex<Vec<(String, String)>>>; // (message, recipient path)

/// Logs every string; `"first"` is held until the gate opens and `"fail"` throws.
struct Worker {
    gate: Arc<Gate>,
    received: Log,
}

impl Actor for Worker {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        match text.as_str() {
            "first" => self.gate.hold_until_released(),
            "fail" => panic!("fail on purpose"),
            _ => {}
        }
        true
    }
}

struct DeadLetterListener(DeadLog);

impl Actor for DeadLetterListener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>()
            && let Some(text) = dead.message.downcast_ref::<String>()
        {
            self.0
                .lock()
                .unwrap()
                .push((text.clone(), dead.recipient.path().to_string()));
        }
        true
    }
}

struct Fixture {
    system: ActorSystem,
    turns: Arc<CountsTurns>,
    worker: ActorRef,
    gate: Arc<Gate>,
    received: Log,
    instances: Arc<AtomicUsize>,
    dead: DeadLog,
}

impl Fixture {
    fn new(capacity: i32) -> Fixture {
        let turns = Arc::new(CountsTurns {
            pool: ThreadPool::start("counting", 2),
            submissions: AtomicUsize::new(0),
        });
        let settings = Settings {
            parallelism: 2,
            ..Settings::default()
        };
        let system =
            ActorSystem::create_with_executor("nb", settings, Arc::new(NanoClock), turns.clone())
                .unwrap();
        let dead = DeadLog::default();
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
        let (gate, received, instances): (Arc<Gate>, Log, Arc<AtomicUsize>) = Default::default();
        let worker = system
            .actor_of_named(
                Props::new({
                    let (gate, received, instances) =
                        (gate.clone(), received.clone(), instances.clone());
                    move || {
                        instances.fetch_add(1, Ordering::SeqCst);
                        Worker {
                            gate: gate.clone(),
                            received: received.clone(),
                        }
                    }
                })
                .with_mailbox(NonBlockingBoundedMailbox::new(capacity).unwrap()),
                "worker",
            )
            .unwrap();
        wait_until("the start-up turn to end", || {
            instances.load(Ordering::SeqCst) == 1
                && worker.mailbox_view().contains("scheduled=false")
        });
        Fixture {
            system,
            turns,
            worker,
            gate,
            received,
            instances,
            dead,
        }
    }

    fn tell(&self, text: &str) {
        self.worker.tell(text.to_owned(), None);
    }

    fn occupy_the_worker(&self) {
        self.tell("first");
        wait_until("the worker to hold its first message", || {
            self.gate.entered.load(Ordering::SeqCst)
        });
    }

    fn dead(&self) -> Vec<(String, String)> {
        self.dead.lock().unwrap().clone()
    }

    fn received(&self) -> Vec<String> {
        self.received.lock().unwrap().clone()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.gate.release();
        let _ = self.system.terminate().wait(Duration::from_secs(10));
    }
}

#[test]
fn a_refused_message_does_not_start_a_consumption_turn() {
    let fixture = Fixture::new(0);
    let turns_before = fixture.turns.submissions.load(Ordering::SeqCst);

    for text in ["a", "b", "c"] {
        fixture.tell(text);
    }

    assert_eq!(
        fixture.turns.submissions.load(Ordering::SeqCst),
        turns_before,
        "nothing was stored, so there is nothing to consume"
    );
    assert!(
        fixture
            .worker
            .mailbox_view()
            .contains("scheduled=false suspendCount=0 queued=0")
    );
    wait_until("the refused messages to be dead-lettered", || {
        fixture.dead().len() == 3
    });
}

#[test]
fn an_accepted_message_starts_a_consumption_turn() {
    let fixture = Fixture::new(1);
    let turns_before = fixture.turns.submissions.load(Ordering::SeqCst);

    fixture.tell("kept");

    assert!(fixture.turns.submissions.load(Ordering::SeqCst) > turns_before);
    wait_until("the message to be received", || {
        fixture.received() == ["kept"]
    });
}

#[test]
fn stopping_the_actor_dead_letters_what_is_still_queued_addressed_to_it() {
    let fixture = Fixture::new(2);
    fixture.occupy_the_worker();
    fixture.tell("queued-1");
    fixture.tell("queued-2");

    fixture.worker.stop(); // `Terminate` is queued before the held message ends
    fixture.gate.release();

    wait_until("the queued messages to be dead-lettered", || {
        fixture.dead().len() == 2
    });
    let worker_path = fixture.worker.path().to_string();
    assert_eq!(
        fixture.dead(),
        [
            ("queued-1".to_owned(), worker_path.clone()),
            ("queued-2".to_owned(), worker_path)
        ]
    );
    assert_eq!(fixture.received(), ["first"]);
}

#[test]
fn a_restart_keeps_the_queue_and_its_bound() {
    let fixture = Fixture::new(1);
    fixture.tell("fail");
    wait_until("the restart", || {
        fixture.instances.load(Ordering::SeqCst) == 2
            && fixture
                .worker
                .mailbox_view()
                .contains("scheduled=false suspendCount=0")
    });

    fixture.occupy_the_worker();
    fixture.tell("kept");
    fixture.tell("refused");

    wait_until("the refused message to be dead-lettered", || {
        fixture.dead().len() == 1
    });
    assert_eq!(fixture.dead()[0].0, "refused");
    fixture.gate.release();
    wait_until("the kept message", || {
        fixture.received().last().map(String::as_str) == Some("kept")
    });
}

#[test]
fn a_terminated_actor_with_a_bounded_mailbox_is_released() {
    let probe = Arc::new(());
    let system = ActorSystem::create("nb-release").unwrap();
    let worker = system
        .actor_of_named(
            Props::new({
                let probe = probe.clone();
                move || {
                    let _held_by_the_factory = &probe;
                    Worker {
                        gate: Arc::default(),
                        received: Arc::default(),
                    }
                }
            })
            .with_mailbox(NonBlockingBoundedMailbox::new(4).unwrap()),
            "released",
        )
        .unwrap();
    worker.tell("queued".to_owned(), None);
    wait_until("the worker to be running", || {
        worker.mailbox_view().contains("scheduled=false")
    });
    assert!(Arc::strong_count(&probe) >= 2, "the cell holds the factory");

    system.stop(&worker);
    wait_until("the worker to terminate", || worker.is_terminated());
    drop(worker);

    wait_until("the cell, its props and its queue to be freed", || {
        Arc::strong_count(&probe) == 1
    });
    let _ = system.terminate().wait(Duration::from_secs(10));
}
