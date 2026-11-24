// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:376-410,677-696 and Mailboxes.scala (Apache-2.0).
//! Selecting the mailbox of an actor through `Props::with_mailbox` with any `MailboxType` (AC-MBX-008, ADR-0001 PR 7/10; Q-MBX-2 is not
//! involved: there is no `Mailboxes` registry or `requirements` here). A `MailboxType` the crate does not know -- a LIFO queue defined
//! in this test -- must be usable, and the default must stay the unbounded FIFO.
//!
//! The actor holds its first message inside `receive` until the test releases it, so the queue contents are fixed without sleeping.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, path_of, wait_until};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::event::{Channel, DeadLetter};

/// A queue that delivers the newest envelope first.
#[derive(Default)]
struct LifoQueue(Mutex<Vec<Envelope>>);

impl MessageQueue for LifoQueue {
    fn enqueue(&self, _receiver: &ActorRef, envelope: Envelope) -> bool {
        self.0.lock().unwrap().push(envelope);
        true
    }
    fn dequeue(&self) -> Option<Envelope> {
        self.0.lock().unwrap().pop()
    }
    fn number_of_messages(&self) -> usize {
        self.0.lock().unwrap().len()
    }
    fn has_messages(&self) -> bool {
        !self.0.lock().unwrap().is_empty()
    }
    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(owner, envelope);
        }
    }
}

/// `MailboxType` of `LifoQueue`; counts the queues it creates.
#[derive(Clone, Default)]
struct LifoMailbox {
    queues_created: Arc<AtomicUsize>,
}

impl MailboxType for LifoMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        self.queues_created.fetch_add(1, Ordering::SeqCst);
        Box::<LifoQueue>::default()
    }
}

#[derive(Default)]
struct Gate {
    released: Mutex<bool>,
    changed: Condvar,
    entered: Mutex<bool>,
}

impl Gate {
    fn hold_until_released(&self) {
        *self.entered.lock().unwrap() = true;
        let mut released = self.released.lock().unwrap();
        while !*released {
            released = self.changed.wait(released).unwrap();
        }
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.changed.notify_all();
    }
    fn entered(&self) -> bool {
        *self.entered.lock().unwrap()
    }
}

/// Logs every string; `"first"` waits for the gate, `"first-then-stop"` also stops the actor afterwards, `"fail"` throws (restart).
struct Worker {
    instances_dropped: Arc<AtomicUsize>,
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.instances_dropped.fetch_add(1, Ordering::SeqCst);
    }
}

impl Actor for Worker {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        match text.as_str() {
            "first" => self.gate.hold_until_released(),
            "first-then-stop" => {
                self.gate.hold_until_released();
                ctx.stop(&ctx.self_ref()); // `Terminate` is queued before this turn ends, so the queue is swept at once
            }
            "fail" => panic!("fail on purpose"),
            _ => {}
        }
        true
    }
}

struct Scenario {
    _system: RunningSystem,
    worker: ActorRef,
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
    /// Actor instances discarded so far: a restart drops the failed instance.
    instances_dropped: Arc<AtomicUsize>,
}

impl Scenario {
    fn new(name: &str, mailbox: Option<Box<dyn Fn(Props) -> Props>>) -> Scenario {
        let system = RunningSystem::create(name);
        let (gate, received) = (Arc::new(Gate::default()), Arc::new(Mutex::new(Vec::new())));
        let instances_dropped = Arc::new(AtomicUsize::new(0));
        let props = Props::new({
            let (gate, received) = (gate.clone(), received.clone());
            let instances_dropped = instances_dropped.clone();
            move || Worker {
                instances_dropped: instances_dropped.clone(),
                gate: gate.clone(),
                received: received.clone(),
            }
        });
        let props = match mailbox {
            Some(select) => select(props),
            None => props,
        };
        let worker = system.actor_of_named(props, "worker").unwrap();
        Scenario {
            _system: system,
            worker,
            gate,
            received,
            instances_dropped,
        }
    }

    fn tell(&self, text: &str) {
        self.worker.tell(text.to_owned(), None);
    }

    fn occupy_the_worker(&self) {
        self.occupy_the_worker_with("first");
    }

    fn occupy_the_worker_with(&self, first_message: &str) {
        self.tell(first_message);
        wait_until("the worker to hold its first message", || {
            self.gate.entered()
        });
    }

    fn received(&self) -> Vec<String> {
        common::snapshot_of(&self.received)
    }

    fn release_and_wait_for(&self, count: usize) {
        self.gate.release();
        wait_until("the queued messages", || self.received().len() == count);
    }
}

#[test]
fn a_mailbox_type_unknown_to_the_crate_supplies_the_queue_of_the_actor() {
    let mailbox = LifoMailbox::default();
    let scenario = Scenario::new(
        "mbxsel-lifo",
        Some(Box::new({
            let mailbox = mailbox.clone();
            move |props| props.with_mailbox(mailbox.clone())
        })),
    );
    scenario.occupy_the_worker();
    for text in ["a", "b", "c"] {
        scenario.tell(text);
    }
    scenario.release_and_wait_for(4);
    assert_eq!(scenario.received(), ["first", "c", "b", "a"]);
    assert_eq!(mailbox.queues_created.load(Ordering::SeqCst), 1);
}

#[test]
fn a_restart_keeps_the_queue_the_mailbox_type_created() {
    let mailbox = LifoMailbox::default();
    let scenario = Scenario::new(
        "mbxsel-restart",
        Some(Box::new({
            let mailbox = mailbox.clone();
            move |props| props.with_mailbox(mailbox.clone())
        })),
    );
    scenario.tell("fail");
    wait_until("the failed instance to be replaced", || {
        scenario.instances_dropped.load(Ordering::SeqCst) == 1
    });
    assert_eq!(scenario.received(), ["fail"]);
    scenario.occupy_the_worker();
    scenario.tell("x");
    scenario.tell("y");
    scenario.release_and_wait_for(4);
    assert_eq!(scenario.received(), ["fail", "first", "y", "x"]);
    assert_eq!(mailbox.queues_created.load(Ordering::SeqCst), 1);
}

struct DeadLetterListener(Arc<Mutex<Vec<(String, String)>>>);

impl Actor for DeadLetterListener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>()
            && let Some(text) = dead.message.downcast_ref::<String>()
        {
            self.0
                .lock()
                .unwrap()
                .push((text.clone(), path_of(&dead.recipient)));
        }
        true
    }
}

#[test]
fn stopping_the_actor_sends_what_its_custom_queue_still_holds_to_dead_letters_once() {
    let scenario = Scenario::new(
        "mbxsel-stop",
        Some(Box::new(|props| props.with_mailbox(LifoMailbox::default()))),
    );
    let dead = Arc::new(Mutex::new(Vec::new()));
    let listener = scenario
        ._system
        .actor_of_named(
            Props::new({
                let dead = dead.clone();
                move || DeadLetterListener(dead.clone())
            }),
            "listener",
        )
        .unwrap();
    scenario
        ._system
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    scenario.occupy_the_worker_with("first-then-stop");
    scenario.tell("queued-1");
    scenario.tell("queued-2");
    scenario.gate.release();

    wait_until("both queued messages to be dead-lettered", || {
        common::snapshot_of(&dead).len() == 2
    });
    let worker_path = path_of(&scenario.worker);
    // the LIFO queue is swept newest first; each envelope arrives exactly once, addressed to the stopped actor
    assert_eq!(
        common::snapshot_of(&dead),
        [
            ("queued-2".to_owned(), worker_path.clone()),
            ("queued-1".to_owned(), worker_path)
        ]
    );
    common::settle_for_absence();
    assert_eq!(common::snapshot_of(&dead).len(), 2);
    assert_eq!(scenario.received(), ["first-then-stop"]);
}

#[test]
fn selecting_unbounded_mailbox_explicitly_keeps_fifo_order() {
    let scenario = Scenario::new(
        "mbxsel-unbounded",
        Some(Box::new(|props| props.with_mailbox(UnboundedMailbox))),
    );
    scenario.occupy_the_worker();
    for text in ["a", "b", "c"] {
        scenario.tell(text);
    }
    scenario.release_and_wait_for(4);
    assert_eq!(scenario.received(), ["first", "a", "b", "c"]);
}

#[test]
fn without_with_mailbox_the_default_is_unbounded_and_loses_nothing() {
    const QUEUED: usize = 5_000;
    let scenario = Scenario::new("mbxsel-default", None);
    scenario.occupy_the_worker();
    for number in 0..QUEUED {
        scenario.tell(&number.to_string());
    }
    scenario.release_and_wait_for(QUEUED + 1);
    let received = scenario.received();
    assert_eq!(received[0], "first");
    assert!(
        received[1..]
            .iter()
            .enumerate()
            .all(|(index, text)| *text == index.to_string())
    );
}
