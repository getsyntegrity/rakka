// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala:105,157-200,214-230, actor/ActorRefProvider.scala:690-725 and actor/dungeon/Children.scala:279-350 (Apache-2.0).
//! Selecting the mailbox of an actor by id (`Props::with_mailbox_id`) through the real creation path of `actorOf`.
//! The actor holds its first message inside `receive` until the test releases it, so the queue contents are fixed without sleeping.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{Gate, LifoMailbox, RunningSystem, wait_until};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::Settings;
use rakka_actor::actor::{ActorError, ErrorClass};
use rakka_actor::dispatch::message_queue::UnboundedMailbox;

/// Logs every string; `"first"` waits for the gate, `"fail"` throws (restart), `"spawn-with-unknown-mailbox"` tries to create a
/// child with a mailbox id nobody registered and logs the class of the error.
struct Worker {
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
}

impl Actor for Worker {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        self.received.lock().unwrap().push(text.clone());
        match text.as_str() {
            "first" => self.gate.hold_until_released(),
            "fail" => panic!("fail on purpose"),
            "spawn-with-unknown-mailbox" => {
                let child = Props::new(|| Idle).with_mailbox_id("nobody-registered-this");
                let outcome = match ctx.actor_of_named(child, "child") {
                    Err(error) => format!("{:?}", error.class()),
                    Ok(_) => "created".to_owned(),
                };
                let registered = ctx.child("child").is_some();
                self.received
                    .lock()
                    .unwrap()
                    .push(format!("{outcome} registered={registered}"));
            }
            _ => {}
        }
        true
    }
}

struct Idle;
impl Actor for Idle {}

/// One worker actor created through `system.actor_of_named` with the props `select` returns.
struct Scenario {
    system: RunningSystem,
    worker: ActorRef,
    gate: Arc<Gate>,
    received: Arc<Mutex<Vec<String>>>,
    instances_created: Arc<AtomicUsize>,
}

fn lifo_and_fifo_settings(lifo: &LifoMailbox) -> Settings {
    let mut settings = Settings::default();
    settings.mailboxes.register("lifo-mailbox", lifo.clone());
    settings
        .mailboxes
        .register("fifo-mailbox", UnboundedMailbox);
    settings
}

fn worker_props(
    gate: &Arc<Gate>,
    received: &Arc<Mutex<Vec<String>>>,
    instances_created: &Arc<AtomicUsize>,
) -> Props {
    let (gate, received, instances_created) =
        (gate.clone(), received.clone(), instances_created.clone());
    Props::new(move || {
        instances_created.fetch_add(1, Ordering::SeqCst);
        Worker {
            gate: gate.clone(),
            received: received.clone(),
        }
    })
}

impl Scenario {
    fn new(name: &str, settings: Settings, select: impl FnOnce(Props) -> Props) -> Scenario {
        let system = RunningSystem::create_with_settings(name, settings);
        let (gate, received) = (Arc::new(Gate::default()), Arc::new(Mutex::new(Vec::new())));
        let instances_created = Arc::new(AtomicUsize::new(0));
        let props = select(worker_props(&gate, &received, &instances_created));
        let worker = system.actor_of_named(props, "worker").unwrap();
        Scenario {
            system,
            worker,
            gate,
            received,
            instances_created,
        }
    }

    fn tell(&self, text: &str) {
        self.worker.tell(text.to_owned(), None);
    }

    fn occupy_the_worker(&self) {
        self.tell("first");
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

    /// With the worker busy, queues `a`, `b`, `c` and returns the order they are processed in.
    fn order_of_three_queued_messages(&self) -> Vec<String> {
        self.occupy_the_worker();
        for text in ["a", "b", "c"] {
            self.tell(text);
        }
        self.release_and_wait_for(4);
        self.received()[1..].to_vec()
    }
}

#[test]
fn an_id_resolves_to_the_registered_type_and_two_ids_to_their_own_types() {
    let lifo = LifoMailbox::default();
    let by_lifo_id = Scenario::new("mbxid-lifo", lifo_and_fifo_settings(&lifo), |props| {
        props.with_mailbox_id("lifo-mailbox")
    });
    let by_fifo_id = Scenario::new("mbxid-fifo", lifo_and_fifo_settings(&lifo), |props| {
        props.with_mailbox_id("fifo-mailbox")
    });
    assert_eq!(by_lifo_id.order_of_three_queued_messages(), ["c", "b", "a"]);
    assert_eq!(by_fifo_id.order_of_three_queued_messages(), ["a", "b", "c"]);
    assert_eq!(lifo.queues_created.load(Ordering::SeqCst), 1);
}

#[test]
fn the_selected_id_is_not_replaced_by_the_default_nor_by_an_earlier_type() {
    let lifo = LifoMailbox::default();
    let id_after_type = Scenario::new("mbxid-over-type", lifo_and_fifo_settings(&lifo), |props| {
        props
            .with_mailbox(UnboundedMailbox)
            .with_mailbox_id("lifo-mailbox")
    });
    assert_eq!(
        id_after_type.order_of_three_queued_messages(),
        ["c", "b", "a"]
    );
    let type_after_id = Scenario::new("mbxid-type-over-id", Settings::default(), |props| {
        props
            .with_mailbox_id("never-registered")
            .with_mailbox(UnboundedMailbox)
    });
    assert_eq!(
        type_after_id.order_of_three_queued_messages(),
        ["a", "b", "c"]
    );
}

#[test]
fn the_empty_id_resets_an_earlier_selection_to_the_default() {
    let lifo = LifoMailbox::default();
    let scenario = Scenario::new("mbxid-reset", lifo_and_fifo_settings(&lifo), |props| {
        props.with_mailbox_id("lifo-mailbox").with_mailbox_id("")
    });
    assert_eq!(scenario.order_of_three_queued_messages(), ["a", "b", "c"]);
    assert_eq!(lifo.queues_created.load(Ordering::SeqCst), 0);
}

#[test]
fn the_built_in_ids_and_the_empty_id_give_the_default_unbounded_fifo() {
    for (index, id) in ["unbounded", "pekko.actor.default-mailbox", ""]
        .into_iter()
        .enumerate()
    {
        let scenario = Scenario::new(
            &format!("mbxid-builtin-{index}"),
            Settings::default(),
            |props| props.with_mailbox_id(id),
        );
        assert_eq!(
            scenario.order_of_three_queued_messages(),
            ["a", "b", "c"],
            "id {id:?}"
        );
    }
}

#[test]
fn two_actors_with_the_same_id_each_get_their_own_queue() {
    let lifo = LifoMailbox::default();
    let first = Scenario::new("mbxid-own-1", lifo_and_fifo_settings(&lifo), |props| {
        props.with_mailbox_id("lifo-mailbox")
    });
    first.occupy_the_worker();
    let (gate, received) = (Arc::new(Gate::default()), Arc::new(Mutex::new(Vec::new())));
    let second = first
        .system
        .actor_of_named(
            worker_props(&gate, &received, &Arc::new(AtomicUsize::new(0)))
                .with_mailbox_id("lifo-mailbox"),
            "second",
        )
        .unwrap();
    first.tell("only-for-first");
    second.tell("only-for-second".to_owned(), None);
    wait_until("the second worker to receive", || {
        !common::snapshot_of(&received).is_empty()
    });
    assert_eq!(common::snapshot_of(&received), ["only-for-second"]);
    assert_eq!(lifo.queues_created.load(Ordering::SeqCst), 2);
    first.release_and_wait_for(2);
    assert_eq!(first.received(), ["first", "only-for-first"]);
}

#[test]
fn a_restart_keeps_the_queue_of_the_selected_id() {
    let lifo = LifoMailbox::default();
    let scenario = Scenario::new("mbxid-restart", lifo_and_fifo_settings(&lifo), |props| {
        props.with_mailbox_id("lifo-mailbox")
    });
    scenario.tell("fail");
    wait_until("the failed instance to be replaced", || {
        scenario.instances_created.load(Ordering::SeqCst) == 2
    });
    scenario.occupy_the_worker();
    for text in ["a", "b", "c"] {
        scenario.tell(text);
    }
    scenario.release_and_wait_for(5);
    assert_eq!(scenario.received(), ["fail", "first", "c", "b", "a"]);
    assert_eq!(lifo.queues_created.load(Ordering::SeqCst), 1);
}

fn expect_configuration_error(outcome: Result<ActorRef, ActorError>, path: &str, id: &str) {
    let error = outcome.expect_err("an unknown mailbox id must fail the creation");
    assert_eq!(error.class(), ErrorClass::Configuration);
    assert_eq!(
        error.message(),
        format!(
            "configuration problem while creating [{path}] with dispatcher [pekko.actor.default-dispatcher] and mailbox [{id}]"
        )
    );
    assert_eq!(
        error.cause(),
        Some(
            format!("org.apache.pekko.ConfigurationException: Mailbox Type [{id}] not configured")
                .as_str()
        )
    );
}

#[test]
fn an_unknown_id_fails_actor_of_without_creating_an_actor_and_frees_the_name() {
    let system = RunningSystem::create("mbxid-unknown");
    let created = Arc::new(AtomicUsize::new(0));
    let unused = (Arc::new(Gate::default()), Arc::new(Mutex::new(Vec::new())));
    let props = || worker_props(&unused.0, &unused.1, &created);
    expect_configuration_error(
        system.actor_of_named(props().with_mailbox_id("typo"), "worker"),
        "pekko://mbxid-unknown/user/worker",
        "typo",
    );
    assert_eq!(created.load(Ordering::SeqCst), 0, "no instance is made");
    // the reserved name was released: the same name is usable at once, with a registered id or the default
    system.actor_of_named(props(), "worker").unwrap();
    wait_until("the retried actor to start", || {
        created.load(Ordering::SeqCst) == 1
    });
}

#[test]
fn a_child_created_from_an_actor_with_an_unknown_id_fails_in_the_parent_call() {
    let scenario = Scenario::new("mbxid-child", Settings::default(), |props| props);
    scenario.tell("spawn-with-unknown-mailbox");
    wait_until("the parent to report the outcome", || {
        scenario.received().len() == 2
    });
    assert_eq!(scenario.received()[1], "Configuration registered=false");
}
