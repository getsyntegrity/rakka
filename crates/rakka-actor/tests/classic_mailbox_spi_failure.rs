// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:714-726, actor/dungeon/Children.scala and actor/RepointableActorRef.scala:112-121 (Apache-2.0).
//! A user's `MailboxType` is public SPI and may panic: `context.actorOf` throws, `system.actorOf` returns and the actor fails to
//! initialise. Expected texts: Pekko 2.0.0 for a `MailboxType` whose `create` throws (oracle cases `MBX-TT-01` and `MBX-TT-02`).

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

/// A mailbox type, and its queue, that fails where it says (`satisfies` and `class_name` run user code only in Rust).
#[derive(Clone, Copy, PartialEq)]
enum Fails {
    Create,
    OrdinaryPanic,
    /// While the mailbox is selected, before any queue is asked for.
    Promise,
    Satisfies,
    /// The queue says it does not satisfy the requirement, then throws when asked its class.
    ClassName,
}

fn boom(message: &str) -> ! {
    throw(Throwable::illegal_state(message))
}

impl MailboxType for Fails {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        match self {
            Fails::Create => boom("mailbox boom"),
            Fails::OrdinaryPanic => panic!("plain panic"),
            Fails::Promise => unreachable!("selection fails first"),
            _ => Box::new(*self),
        }
    }
    fn produced_queue(&self) -> ProducedQueue {
        if *self == Fails::Promise {
            boom("promise boom");
        }
        ProducedQueue::new("test.Queue", &[MessageQueueSemantics::UNBOUNDED])
    }
}

#[rustfmt::skip] // one stub per line
impl MessageQueue for Fails {
    fn enqueue(&self, _: &ActorRef, _: Envelope) -> bool { unreachable!("the actor is never made") }
    fn dequeue(&self) -> Option<Envelope> { None }
    fn number_of_messages(&self) -> usize { 0 }
    fn has_messages(&self) -> bool { false }
    fn clean_up(&self, _: &ActorRef, _: &dyn MessageQueue) {}
    fn class_name(&self) -> &'static str { boom("class_name boom") }
    fn satisfies(&self, _: MessageQueueSemantics) -> Option<bool> {
        if *self == Fails::Satisfies { boom("satisfies boom") } else { Some(false) }
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

/// Requires an unbounded queue, so `Dispatch.init` asks the created queue about itself.
struct Requiring;
#[rustfmt::skip]
impl Actor for Requiring {
    fn required_message_queue() -> Option<MessageQueueSemantics> { Some(MessageQueueSemantics::UNBOUNDED) }
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool { true }
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

const DEFAULT_ID: &str = Mailboxes::DEFAULT_MAILBOX_ID;

/// `context.actorOf` with `props` throws `ConfigurationException` naming `mailbox_id` and `cause`, adds no child and frees the name.
fn assert_creation_fails(name: &str, props: fn(Props) -> Props, mailbox_id: &str, cause: &str) {
    let mut settings = Settings::default();
    settings.mailboxes.register("unpromising", Fails::Promise);
    let system = RunningSystem::create_with_settings(name, settings);
    let log = Log::default();
    let parent = Props::new({
        let log = log.clone();
        move || Parent(log.clone())
    });
    let parent = system.actor_of_named(parent, "parent").unwrap();
    parent.tell(props(echo(&log)), None);
    wait_until("the parent to report", || snapshot_of(&log).len() == 2);
    let failure = format!(
        "org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://{name}/user/parent/kid] \
         with dispatcher [pekko.actor.default-dispatcher] and mailbox [{mailbox_id}] cause={cause}"
    );
    assert_eq!(snapshot_of(&log), [&failure, "children=0 reused=true"]);
}

#[test]
fn a_mailbox_type_that_throws_fails_context_actor_of_and_frees_the_name() {
    let cause = "java.lang.IllegalStateException: mailbox boom";
    let props = |p: Props| p.with_mailbox(Fails::Create);
    assert_creation_fails("spi-sync", props, DEFAULT_ID, cause);
}

#[test]
fn an_ordinary_panic_of_a_mailbox_type_is_reported_as_a_runtime_exception() {
    let cause = "java.lang.RuntimeException: plain panic";
    let props = |p: Props| p.with_mailbox(Fails::OrdinaryPanic);
    assert_creation_fails("spi-panic", props, DEFAULT_ID, cause);
}

#[test]
fn a_mailbox_type_that_throws_while_it_is_selected_fails_actor_of_the_same_way() {
    let cause = "java.lang.IllegalStateException: promise boom";
    let props = |p: Props| p.with_mailbox_id("unpromising");
    assert_creation_fails("spi-select", props, "unpromising", cause);
}

#[test]
fn a_queue_that_throws_in_satisfies_fails_the_creation() {
    let cause = "java.lang.IllegalStateException: satisfies boom";
    let props = |_| Props::new(|| Requiring).with_mailbox(Fails::Satisfies);
    assert_creation_fails("spi-satisfies", props, DEFAULT_ID, cause);
}

#[test]
fn a_queue_that_throws_in_class_name_fails_the_creation() {
    let cause = "java.lang.IllegalStateException: class_name boom";
    let props = |_| Props::new(|| Requiring).with_mailbox(Fails::ClassName);
    assert_creation_fails("spi-class-name", props, DEFAULT_ID, cause);
}

/// Panics when dropped.
struct Bomb;

impl Drop for Bomb {
    fn drop(&mut self) {
        panic!("drop");
    }
}

/// Props whose only clone holds a [`Bomb`]: letting go of them panics.
fn props_with_a_panicking_drop(_: Props) -> Props {
    let held = Bomb;
    let factory = move || {
        let _ = &held;
        Requiring
    };
    Props::new(factory).with_mailbox(Fails::Create)
}

/// Creates `kid` with props of its own, so that the failed creation holds their last clone.
struct SoleOwner(Log);
impl Actor for SoleOwner {
    fn receive(&mut self, ctx: &mut ActorContext, _: &AnyMessage) -> bool {
        let props = props_with_a_panicking_drop(echo(&self.0));
        let outcome = match ctx.actor_of_named(props, "kid") {
            Ok(_) => "created".to_owned(),
            Err(error) => error.cause().unwrap_or("none").to_owned(),
        };
        let children = ctx.children().len();
        let reused = ctx.actor_of_named(echo(&self.0), "kid").is_ok();
        let report = format!("{outcome} children={children} reused={reused}");
        self.0.lock().unwrap().push(report);
        true
    }
}

#[test]
fn props_whose_drop_panics_do_not_keep_the_name_of_a_failed_creation() {
    let system = RunningSystem::create("spi-props-drop");
    let log = Log::default();
    let parent = Props::new({
        let log = log.clone();
        move || SoleOwner(log.clone())
    });
    system.actor_of(parent).unwrap().tell((), None);
    wait_until("the parent to report", || !snapshot_of(&log).is_empty());
    let report = "java.lang.IllegalStateException: mailbox boom children=0 reused=true";
    assert_eq!(snapshot_of(&log), [report]);
}

#[test]
fn a_default_mailbox_that_panics_makes_the_system_creation_throw_a_throwable() {
    let mut settings = Settings::default();
    settings
        .mailboxes
        .register(DEFAULT_ID, Fails::OrdinaryPanic);
    let create = || ActorSystem::create_with_settings("spi-system", settings);
    let thrown = catch_unwind(AssertUnwindSafe(create)).err();
    let thrown = thrown.expect("ActorSystem.create throws");
    assert_eq!(
        thrown.downcast_ref::<Throwable>().map(ToString::to_string),
        Some("java.lang.RuntimeException: plain panic".to_owned())
    );
}

struct Errors(Log);
impl Actor for Errors {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(error) = msg.downcast_ref::<ErrorEvent>() {
            let message = error.message.as_deref().unwrap_or("null");
            let seen = format!("{}|{}|{message}", error.log_source, error.cause.class());
            self.0.lock().unwrap().push(seen);
        }
        true
    }
}

#[test]
fn system_actor_of_returns_and_the_actor_that_cannot_be_made_is_stopped() {
    let system = RunningSystem::create("spi-async");
    let (log, errors) = (Log::default(), Log::default());
    let listener = Props::new({
        let errors = errors.clone();
        move || Errors(errors.clone())
    });
    let listener = system.actor_of(listener).unwrap();
    system.event_stream().subscribe(&listener, Channel::Error);

    let failed = system
        .actor_of_named(echo(&log).with_mailbox(Fails::Create), "a")
        .expect("a reference is returned");

    wait_until("the actor to be stopped", || failed.is_terminated());
    let next = system.actor_of_named(echo(&log), "b").unwrap();
    next.tell("m".to_owned(), None);
    wait_until("the next actor of the guardian", || {
        snapshot_of(&log) == ["m"]
    });
    wait_until("the report", || !snapshot_of(&errors).is_empty());
    let report =
        "pekko://spi-async/user/a|org.apache.pekko.actor.ActorInitializationException|mailbox boom";
    assert_eq!(snapshot_of(&errors), [report]);
    // `RunningSystem` then requires the system to terminate normally
}
