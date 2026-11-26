// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala:428-490 and actor/dungeon/FaultHandling.scala:263-264 (Apache-2.0).
//! `whenTerminated` after the root guardian fails. The bubble walker records the cause in its own `causeOfTermination` and stops the root; that makes
//! it stop walking, so the death notification of the root is discarded, `stop()` is never reached and the public `terminationPromise` stays pending.
//! Read from the Scala, not probed on the JVM (where `ActorSystem.create` already fails by the logger start-up timeout).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::fault::Throwable;
use crate::actor::system::{ActorSystem, Settings};
use crate::actor::system::{BubbleWalker, Promise};
use crate::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use crate::dispatch::queue_semantics::ProducedQueue;
use crate::dispatch::sysmsg::SystemMessage;
use crate::dispatch::test_support::recorder;

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
    {
        let _cleanup = DispatchersGuard(walker.clone());
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
    }
    // the system never terminates by itself (that is the point of the test), so the dispatchers must have been closed by the test itself
    assert!(walker.dispatchers_are_shut_down());
}

/// A walker on its own with a recorder as root: its messages are driven one by one, so the order is the test's, not a thread's.
fn walker_with_a_root() -> (Arc<BubbleWalker>, ActorRef) {
    let (_, root) = recorder("pekko://sys/");
    let (_, bubble) = recorder("pekko://sys/bubble-walker");
    let walker = Arc::new(BubbleWalker::new(
        bubble.path().clone(),
        Promise::new(),
        vec![],
    ));
    walker.root.set(root.clone()).ok();
    (walker, root)
}

fn the_root_failed(root: &ActorRef) -> SystemMessage {
    SystemMessage::Failed {
        child: root.clone(),
        cause: Throwable::new(
            "org.apache.pekko.actor.ActorInitializationException",
            Some("root"),
        ),
    }
}

#[test]
fn stop_called_directly_after_the_failure_fails_the_public_future_with_the_cause() {
    // `stop()` tries to complete the internal promise with `Terminated` (it already failed) and links `terminationPromise` to it:
    // `completeWith(causeOfTermination.future)` (ActorRefProvider.scala:464-468)
    let (walker, root) = walker_with_a_root();
    walker.send_system(the_root_failed(&root));
    walker.stop();
    let outcome = walker
        .termination
        .wait(Duration::ZERO)
        .expect("completed by stop()");
    assert_eq!(outcome.unwrap_err().message.as_deref(), Some("root"));
}

#[test]
fn the_death_notification_after_the_failure_is_discarded_and_the_public_future_stays_pending() {
    let (walker, root) = walker_with_a_root();
    walker.send_system(the_root_failed(&root));
    walker.send_system(SystemMessage::DeathWatchNotification {
        actor: root,
        existence_confirmed: true,
        address_terminated: false,
    });
    assert!(!walker.termination.is_completed());
}

#[test]
fn only_the_death_notification_of_the_failed_root_is_the_synchronisation_signal() {
    let (walker, root) = walker_with_a_root();
    let (_, other) = recorder("pekko://sys/user");
    walker.send_system(the_root_failed(&root));
    // discarded, but neither of them is the root's death
    walker.send_system(SystemMessage::Supervise {
        child: other.clone(),
        is_async: false,
    });
    walker.send_system(SystemMessage::DeathWatchNotification {
        actor: other,
        existence_confirmed: true,
        address_terminated: false,
    });
    assert!(!walker.notification_discarded.is_completed());
    walker.send_system(SystemMessage::DeathWatchNotification {
        actor: root,
        existence_confirmed: true,
        address_terminated: false,
    });
    assert!(walker.notification_discarded.is_completed());
}

/// Shuts the walker's dispatchers down when dropped, whether the test returns or unwinds.
struct DispatchersGuard(Arc<BubbleWalker>);

impl Drop for DispatchersGuard {
    fn drop(&mut self) {
        self.0.shut_down_dispatchers();
    }
}

#[test]
fn the_guard_shuts_the_dispatchers_down_when_the_test_unwinds() {
    let system = ActorSystem::create("unwinds").unwrap();
    let walker = system
        .0
        .walker
        .get()
        .expect("the walker of the system")
        .clone();
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = DispatchersGuard(walker.clone());
        panic!("the body of the test fails");
    }));
    assert!(unwound.is_err());
    assert!(walker.dispatchers_are_shut_down());
}
