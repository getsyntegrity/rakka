// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:535-549,851-865 (Apache-2.0).
//! An actor whose mailbox is `UnboundedDequeBasedMailbox` (AC-MBX-011): the real cell, dispatcher and dead letters over the deque
//! queue. Putting envelopes at the head is not reachable from an actor until `Stash` exists, so that part is tested on the queue.
//!
//! The actor holds its first message inside `receive` until the test releases it, so the queue contents are fixed without sleeping.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, RunningSystem, path_of, snapshot_of, wait_until};

use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::Settings;
use rakka_actor::dispatch::deque_mailbox::UnboundedDequeBasedMailbox;
use rakka_actor::dispatch::queue_semantics::MessageQueueSemantics;
use rakka_actor::event::{Channel, DeadLetter};

type Log = Arc<Mutex<Vec<String>>>;

/// Logs every string and holds on the first one; `"first-then-stop"` also stops the actor once released.
struct Worker {
    gate: Arc<Gate>,
    received: Log,
}

impl Actor for Worker {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        if text.starts_with("first") {
            self.gate.hold_until_released();
        }
        if text == "first-then-stop" {
            ctx.stop(&ctx.self_ref());
        }
        true
    }
}

/// Logs `message@recipient` of every string dead letter.
struct DeadLetterListener(Log);

impl Actor for DeadLetterListener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>()
            && let Some(text) = dead.message.downcast_ref::<String>()
        {
            let entry = format!("{text}@{}", path_of(&dead.recipient));
            self.0.lock().unwrap().push(entry);
        }
        true
    }
}

fn busy_deque_worker(system: &RunningSystem, first_message: &str) -> (ActorRef, Arc<Gate>, Log) {
    let (gate, received) = (Arc::new(Gate::default()), Log::default());
    let props = Props::new({
        let (gate, received) = (gate.clone(), received.clone());
        move || Worker {
            gate: gate.clone(),
            received: received.clone(),
        }
    })
    .with_mailbox(UnboundedDequeBasedMailbox);
    let worker = system.actor_of_named(props, "worker").unwrap();
    worker.tell(first_message.to_owned(), None);
    wait_until("the worker to hold its first message", || gate.entered());
    (worker, gate, received)
}

#[test]
fn an_actor_with_a_deque_mailbox_receives_in_the_order_the_messages_arrived() {
    let system = RunningSystem::create("mbxdq-fifo");
    let (worker, gate, received) = busy_deque_worker(&system, "first");
    for text in ["a", "b", "c"] {
        worker.tell(text.to_owned(), None);
    }
    gate.release();
    wait_until("the queued messages", || snapshot_of(&received).len() == 4);
    assert_eq!(snapshot_of(&received), ["first", "a", "b", "c"]);
}

#[test]
fn stopping_the_actor_dead_letters_what_its_deque_still_holds_in_order() {
    let system = RunningSystem::create("mbxdq-stop");
    let dead = Log::default();
    let listener_props = Props::new({
        let dead = dead.clone();
        move || DeadLetterListener(dead.clone())
    });
    let listener = system.actor_of_named(listener_props, "listener").unwrap();
    system
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    let (worker, gate, received) = busy_deque_worker(&system, "first-then-stop");
    worker.tell("queued-1".to_owned(), None);
    worker.tell("queued-2".to_owned(), None);
    gate.release();
    wait_until("both queued messages to be dead-lettered", || {
        snapshot_of(&dead).len() == 2
    });
    let path = path_of(&worker);
    assert_eq!(
        snapshot_of(&dead),
        [format!("queued-1@{path}"), format!("queued-2@{path}")]
    );
    assert_eq!(snapshot_of(&received), ["first-then-stop"]);
}

/// Requires a bounded queue, which the deque mailbox does not produce.
struct NeedsBounded;

impl Actor for NeedsBounded {
    fn required_message_queue() -> Option<MessageQueueSemantics> {
        Some(MessageQueueSemantics::BOUNDED)
    }

    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

/// Selected by its registered id (what Pekko's `withMailbox(id)` does; a type given directly to `with_mailbox` is not verified).
/// The error names the queue class of the type, so this also proves which mailbox was selected.
#[test]
fn a_requirement_the_deque_queue_does_not_fulfill_fails_naming_its_queue_class() {
    let mut settings = Settings::default();
    settings
        .mailboxes
        .register("deque-mailbox", UnboundedDequeBasedMailbox);
    let system = RunningSystem::create_with_settings("mbxdq-requirement", settings);
    let props = Props::new(|| NeedsBounded).with_mailbox_id("deque-mailbox");
    let Err(error) = system.actor_of_named(props, "bounded") else {
        panic!("the deque queue is not bounded");
    };
    let report = format!("{} | {:?}", error.message(), error.cause());
    let queue = "[class org.apache.pekko.dispatch.UnboundedDequeBasedMailbox$MessageQueue]";
    assert!(report.contains(queue), "{report}");
}
