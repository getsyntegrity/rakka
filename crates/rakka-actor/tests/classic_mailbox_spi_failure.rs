// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:714-726, actor/dungeon/Children.scala and actor/RepointableActorRef.scala:112-121 (Apache-2.0).
//! A user's `MailboxType` is public SPI and may panic. `context.actorOf` then throws `ConfigurationException` and releases the name;
//! `system.actorOf` returns and the new actor fails to initialise. Expected texts: what Pekko 2.0.0 prints for a `MailboxType` whose
//! `create` throws (the recorded cases come in a follow-up change).

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::{RunningSystem, snapshot_of, wait_until};

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::dispatch::mailboxes::Mailboxes;
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue};
use rakka_actor::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};
use rakka_actor::event::{Channel, ErrorEvent};

type Log = Arc<Mutex<Vec<String>>>;

/// `create` throws `IllegalStateException("mailbox boom")`.
struct ThrowingMailbox;
impl MailboxType for ThrowingMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        throw(Throwable::illegal_state("mailbox boom"))
    }
}

/// `create` panics the ordinary Rust way.
struct PanickingMailbox;
impl MailboxType for PanickingMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        panic!("plain panic")
    }
}

/// Fails while the mailbox is selected, before any queue is asked for.
struct UnpromisingMailbox;
impl MailboxType for UnpromisingMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        unreachable!("selection fails first")
    }
    fn produced_queue(&self) -> ProducedQueue {
        throw(Throwable::illegal_state("promise boom"))
    }
}

struct Echo(Log);
impl Actor for Echo {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(text) = msg.downcast_ref::<String>() {
            self.0.lock().unwrap().push(text.clone());
        }
        true
    }
}

fn echo(log: &Log) -> Props {
    let log = log.clone();
    Props::new(move || Echo(log.clone()))
}

/// Creates `kid` with the props it is sent and logs the outcome and its children, then reuses the name.
struct Parent(Log);
impl Actor for Parent {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(props) = msg.downcast_ref::<Props>() else {
            return false;
        };
        let outcome = match ctx.actor_of_named(props.clone(), "kid") {
            Ok(_) => "created".to_owned(),
            Err(error) => format!("{error} cause={}", error.cause().unwrap_or("none")),
        };
        let children = ctx.children().len();
        let reused = ctx.actor_of_named(echo(&self.0), "kid");
        let mut log = self.0.lock().unwrap();
        log.push(outcome);
        log.push(format!("children={children} reused={}", reused.is_ok()));
        true
    }
}

fn outcome_of_creating_a_child_with(system_name: &str, props: fn(Props) -> Props) -> Vec<String> {
    let mut settings = Settings::default();
    settings
        .mailboxes
        .register("unpromising", UnpromisingMailbox);
    let system = RunningSystem::create_with_settings(system_name, settings);
    let log = Log::default();
    let parent = system
        .actor_of_named(
            Props::new({
                let log = log.clone();
                move || Parent(log.clone())
            }),
            "parent",
        )
        .unwrap();
    parent.tell(props(echo(&log)), None);
    wait_until("the parent to report", || snapshot_of(&log).len() == 2);
    snapshot_of(&log)
}

fn configuration_failure(system_name: &str, mailbox_id: &str, cause: &str) -> Vec<String> {
    vec![
        format!(
            "org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://{system_name}/user/parent/kid] \
             with dispatcher [pekko.actor.default-dispatcher] and mailbox [{mailbox_id}] cause={cause}"
        ),
        "children=0 reused=true".to_owned(),
    ]
}

#[test]
fn a_mailbox_type_that_throws_fails_context_actor_of_and_frees_the_name() {
    let log =
        outcome_of_creating_a_child_with("spi-sync", |props| props.with_mailbox(ThrowingMailbox));
    assert_eq!(
        log,
        configuration_failure(
            "spi-sync",
            "pekko.actor.default-mailbox",
            "java.lang.IllegalStateException: mailbox boom"
        )
    );
}

#[test]
fn an_ordinary_panic_of_a_mailbox_type_is_reported_as_a_runtime_exception() {
    let log =
        outcome_of_creating_a_child_with("spi-panic", |props| props.with_mailbox(PanickingMailbox));
    assert_eq!(
        log,
        configuration_failure(
            "spi-panic",
            "pekko.actor.default-mailbox",
            "java.lang.RuntimeException: plain panic"
        )
    );
}

#[test]
fn a_mailbox_type_that_throws_while_it_is_selected_fails_actor_of_the_same_way() {
    let log = outcome_of_creating_a_child_with("spi-select", |props| {
        props.with_mailbox_id("unpromising")
    });
    assert_eq!(
        log,
        configuration_failure(
            "spi-select",
            "unpromising",
            "java.lang.IllegalStateException: promise boom"
        )
    );
}

/// An actor that requires an unbounded queue, so `Dispatch.init` asks the created queue about itself.
struct Requiring;
impl Actor for Requiring {
    fn required_message_queue() -> Option<MessageQueueSemantics> {
        Some(MessageQueueSemantics::UNBOUNDED)
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

/// Promises an unbounded queue; the queue panics in `satisfies`, or says no and panics in `class_name`. Rust-only SPI methods: in
/// Pekko neither question runs user code.
#[derive(Clone, Copy)]
struct SelfDescribing {
    satisfies_throws: bool,
}
impl MailboxType for SelfDescribing {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(*self)
    }
    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new("test.Queue", &[MessageQueueSemantics::UNBOUNDED])
    }
}
impl MessageQueue for SelfDescribing {
    fn enqueue(&self, _receiver: &ActorRef, _envelope: Envelope) -> bool {
        unreachable!("the actor is never made")
    }
    fn dequeue(&self) -> Option<Envelope> {
        None
    }
    fn number_of_messages(&self) -> usize {
        0
    }
    fn has_messages(&self) -> bool {
        false
    }
    fn clean_up(&self, _owner: &ActorRef, _dead_letters: &dyn MessageQueue) {}
    fn satisfies(&self, _requirement: MessageQueueSemantics) -> Option<bool> {
        if self.satisfies_throws {
            throw(Throwable::illegal_state("satisfies boom"));
        }
        Some(false)
    }
    fn class_name(&self) -> &'static str {
        throw(Throwable::illegal_state("class_name boom"))
    }
}

#[test]
fn a_queue_that_throws_in_satisfies_fails_the_creation() {
    let log = outcome_of_creating_a_child_with("spi-satisfies", |_| {
        Props::new(|| Requiring).with_mailbox(SelfDescribing {
            satisfies_throws: true,
        })
    });
    assert_eq!(
        log,
        configuration_failure(
            "spi-satisfies",
            "pekko.actor.default-mailbox",
            "java.lang.IllegalStateException: satisfies boom"
        )
    );
}

#[test]
fn a_queue_that_throws_in_class_name_fails_the_creation() {
    let log = outcome_of_creating_a_child_with("spi-class-name", |_| {
        Props::new(|| Requiring).with_mailbox(SelfDescribing {
            satisfies_throws: false,
        })
    });
    assert_eq!(
        log,
        configuration_failure(
            "spi-class-name",
            "pekko.actor.default-mailbox",
            "java.lang.IllegalStateException: class_name boom"
        )
    );
}

#[test]
fn a_default_mailbox_that_panics_makes_the_system_creation_throw_a_throwable() {
    let mut settings = Settings::default();
    settings
        .mailboxes
        .register(Mailboxes::DEFAULT_MAILBOX_ID, PanickingMailbox);
    let thrown = catch_unwind(AssertUnwindSafe(|| {
        ActorSystem::create_with_settings("spi-system", settings)
    }))
    .err()
    .expect("ActorSystem.create throws");
    assert_eq!(
        thrown.downcast_ref::<Throwable>().map(ToString::to_string),
        Some("java.lang.RuntimeException: plain panic".to_owned())
    );
}

struct Errors(Log);
impl Actor for Errors {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
            self.0.lock().unwrap().push(format!(
                "{}|{}|{}",
                error.log_source,
                error.cause.class,
                error.message.as_deref().unwrap_or("null")
            ));
        }
        true
    }
}

#[test]
fn system_actor_of_returns_and_the_actor_that_cannot_be_made_is_stopped() {
    let system = RunningSystem::create("spi-async");
    let (log, errors) = (Log::default(), Log::default());
    let listener = system
        .actor_of(Props::new({
            let errors = errors.clone();
            move || Errors(errors.clone())
        }))
        .unwrap();
    system.event_stream().subscribe(&listener, Channel::Error);

    let failed = system
        .actor_of_named(echo(&log).with_mailbox(ThrowingMailbox), "a")
        .expect("a reference is returned");

    wait_until("the actor to be stopped", || failed.is_terminated());
    system
        .actor_of_named(echo(&log), "b")
        .unwrap()
        .tell("m".to_owned(), None);
    wait_until("the next actor of the guardian", || {
        snapshot_of(&log) == ["m"]
    });
    wait_until("the report", || !snapshot_of(&errors).is_empty());
    assert_eq!(
        snapshot_of(&errors),
        [
            "pekko://spi-async/user/a|org.apache.pekko.actor.ActorInitializationException|mailbox boom"
        ]
    );
    // `RunningSystem` then requires the system to terminate normally
}
