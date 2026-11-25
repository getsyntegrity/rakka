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
use rakka_actor::actor::props::{Actor, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
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

#[test]
fn a_guardian_whose_queue_is_not_unbounded_fails_to_initialise_and_the_system_shuts_down() {
    let promise = NonBlockingBoundedMailbox::new(5).unwrap().produced_queue();
    for (guardian, bad_on) in [("root", 1), ("user", 2), ("system", 3)] {
        let system = system_with_a_bad_guardian_queue(promise, bad_on);
        let terminated = system.when_terminated().wait(Duration::from_secs(10));
        assert!(terminated.is_some(), "the {guardian} guardian");
        // only the root guardian fails to the bubble walker, which fails `whenTerminated` with its exception (`causeOfTermination.tryFailure`,
        // ActorRefProvider.scala:485-487); `/user` and `/system` are stopped by the root's strategy (`:525-529`) and the system ends normally
        let cause = system.when_terminated().failure();
        if guardian == "root" {
            let cause = cause.expect("the root guardian's failure");
            assert_eq!(
                cause.class,
                "org.apache.pekko.actor.ActorInitializationException"
            );
            let message = cause.message.unwrap_or_default();
            assert!(message.contains("requires mailbox type [interface org.apache.pekko.dispatch.UnboundedMessageQueueSemantics] got ["), "{message}");
        } else {
            assert_eq!(cause, None, "the {guardian} guardian");
        }
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
        assert!(system.terminate().wait(Duration::from_secs(10)).is_some());
        assert!(system.when_terminated().failure().is_none());
    }
}
