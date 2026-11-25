// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:318-345,543-606 and actor/dungeon/Dispatch.scala:78-103 (Apache-2.0).
//! The root, `/user` and `/system` guardians take the registered default mailbox directly (`mailboxes.lookup(DefaultMailboxId)`, not
//! `getMailboxType`); all three declare `RequiresMessageQueue[UnboundedMessageQueueSemantics]`, which is checked only in `Dispatch.init` and
//! only when the default type promises its queue. A guardian that fails to initialise shuts the system down.
//!
//! Oracle probe (JDK 17, Pekko 2.0.0): a default mailbox that produces a bounded queue makes the root guardian fail with
//! `ActorInitializationException ... requires mailbox type [interface ...UnboundedMessageQueueSemantics] got [...BoundedNodeMessageQueue]`
//! and the system shuts down; a type that promises nothing, or an unbounded queue, starts normally. What `ActorSystem.create` then throws on the
//! JVM comes from the logger start-up timeout, which Rakka does not have: not claimed here.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::Throwable;
use rakka_actor::actor::props::{Actor, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use rakka_actor::dispatch::mailboxes::Mailboxes;
use rakka_actor::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::dispatch::queue_semantics::ProducedQueue;

/// A default mailbox that counts the queues it creates and declares `promise`; its `bad_on`-th queue (1 is the root guardian's, 2 `/user`'s, 3
/// `/system`'s, the order Pekko creates them in) is bounded, the others unbounded.
struct CountingDefault {
    created: Arc<AtomicUsize>,
    promise: ProducedQueue,
    bad_on: Option<usize>,
}

impl MailboxType for CountingDefault {
    fn create(&self, dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        let nth = self.created.fetch_add(1, Ordering::SeqCst) + 1;
        if self.bad_on == Some(nth) {
            NonBlockingBoundedMailbox::new(5)
                .unwrap()
                .create(dead_letters.clone())
        } else {
            UnboundedMailbox.create()
        }
    }

    fn produced_queue(&self) -> ProducedQueue {
        self.promise
    }
}

fn settings_with_default(default: impl MailboxType) -> Settings {
    let mut settings = Settings::default();
    settings
        .mailboxes
        .register(Mailboxes::DEFAULT_MAILBOX_ID, default);
    settings
}

struct Idle;
impl Actor for Idle {}

#[test]
fn the_three_guardians_take_the_registered_default_mailbox() {
    let created = Arc::new(AtomicUsize::new(0));
    let default = CountingDefault {
        created: created.clone(),
        promise: UnboundedMailbox.produced_queue(),
        bad_on: None,
    };
    let system = common::RunningSystem::create_with_settings("g", settings_with_default(default));
    // root, /user and /system: nothing else is created before the first actorOf
    assert_eq!(created.load(Ordering::SeqCst), 3);
    system
        .actor_of(Props::new(|| Idle))
        .expect("an actor under the user guardian");
}

/// A default mailbox that promises `promise` and creates a bounded queue for guardian number `bad_on` (root, user, system).
fn system_with_a_bad_guardian_queue(promise: ProducedQueue, bad_on: usize) -> ActorSystem {
    let default = CountingDefault {
        created: Arc::default(),
        promise,
        bad_on: Some(bad_on),
    };
    ActorSystem::create_with_settings("g", settings_with_default(default)).unwrap()
}

/// How the system whose guardian number `bad_on` (1 root, 2 `/user`, 3 `/system`) gets a bounded queue ended: `whenTerminated` is one `Future[Terminated]`.
fn how_a_system_with_a_bad_guardian_queue_ends(
    bad_on: usize,
) -> Option<Result<Terminated, Throwable>> {
    let promise = NonBlockingBoundedMailbox::new(5).unwrap().produced_queue();
    let system = system_with_a_bad_guardian_queue(promise, bad_on);
    system.when_terminated().wait(Duration::from_secs(10))
}

#[test]
fn a_root_guardian_that_fails_to_initialise_fails_when_terminated_with_its_exception() {
    // the bubble walker fails `causeOfTermination` (ActorRefProvider.scala:485-487); the `Terminated` that follows cannot overwrite it
    let ended = how_a_system_with_a_bad_guardian_queue_ends(1).expect("the system ends");
    let cause = ended.expect_err("a failure, not a Terminated");
    assert_eq!(
        cause.class,
        "org.apache.pekko.actor.ActorInitializationException"
    );
    let message = cause.message.unwrap_or_default();
    assert!(message.contains("requires mailbox type [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics] got ["), "{message}");
}

#[test]
fn the_user_and_system_guardians_that_fail_to_initialise_end_the_system_normally() {
    // the root's strategy stops them without recording a cause (ActorRefProvider.scala:525-529)
    for (guardian, bad_on) in [("user", 2), ("system", 3)] {
        let ended = how_a_system_with_a_bad_guardian_queue_ends(bad_on).expect("the system ends");
        assert!(ended.is_ok(), "the {guardian} guardian: {ended:?}");
    }
}

#[test]
fn a_default_mailbox_that_promises_nothing_leaves_the_guardians_unchecked() {
    for bad_on in 1..=3 {
        let system = system_with_a_bad_guardian_queue(ProducedQueue::PLAIN, bad_on);
        assert!(!system.when_terminated().is_completed());
        system
            .actor_of(Props::new(|| Idle))
            .expect("the system keeps working");
        // a requested termination is a normal one
        let ended = system.terminate().wait(Duration::from_secs(10));
        assert!(matches!(ended, Some(Ok(_))), "{ended:?}");
    }
}
