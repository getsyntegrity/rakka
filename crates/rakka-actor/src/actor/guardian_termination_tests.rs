// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:428-490 and actor/dungeon/FaultHandling.scala:263-264 (Apache-2.0).
//! `whenTerminated` after the root guardian fails. The bubble walker records the cause in its own `causeOfTermination` and stops the root; that makes
//! it stop walking, so the death notification of the root is discarded, `stop()` is never reached and the public `terminationPromise` stays pending.
//! Read from the Scala, not probed on the JVM (where `ActorSystem.create` already fails by the logger start-up timeout).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::actor::actor_ref::ActorRef;
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use crate::dispatch::queue_semantics::ProducedQueue;

/// A default mailbox that promises a bounded queue and creates one only for the first guardian (the root), unbounded ones for the rest.
struct BoundedForTheRoot(AtomicUsize);

impl MailboxType for BoundedForTheRoot {
    fn create(&self, dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        match self.0.fetch_add(1, Ordering::SeqCst) {
            0 => NonBlockingBoundedMailbox::new(5)
                .unwrap()
                .create(dead_letters.clone()),
            _ => UnboundedMailbox.create(),
        }
    }

    fn produced_queue(&self) -> ProducedQueue {
        NonBlockingBoundedMailbox::new(5).unwrap().produced_queue()
    }
}

#[test]
fn a_failed_root_guardian_leaves_when_terminated_pending() {
    let mut settings = Settings::default();
    settings.mailboxes.register(
        Mailboxes::DEFAULT_MAILBOX_ID,
        BoundedForTheRoot(AtomicUsize::new(0)),
    );
    let system = ActorSystem::create_with_settings("root-fails", settings).unwrap();
    let walker = system
        .0
        .walker
        .get()
        .expect("the walker of the system")
        .clone();
    // the walker discarded the death notification of the root: the failure is processed, so a pending future is not just "not yet"
    walker
        .notification_discarded
        .wait(Duration::from_secs(10))
        .expect("the death notification of the root reaches the walker");
    // `causeOfTermination` holds the failure of the root, which stopped the walker from walking
    let cause = walker
        .cause_of_termination
        .wait(Duration::ZERO)
        .expect("recorded");
    let cause = cause.expect_err("a failure");
    assert_eq!(
        cause.class,
        "org.apache.pekko.actor.ActorInitializationException"
    );
    assert!(!system.when_terminated().is_completed());
    walker.shut_down_dispatchers();
}
