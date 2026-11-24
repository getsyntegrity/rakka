// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala (Apache-2.0).
//! `UnstartedPushTimeout` (`RepointableActorRef.scala:208-325`): while `replaceWith` hands the queued messages to the real cell it keeps
//! the lock of the unstarted cell, so a user message sent meanwhile waits for it at most `pekko.actor.unstarted-push-timeout` and
//! otherwise becomes a dead letter with a `Warning`; a system message waits for as long as it takes.
//!
//! The JVM blocks the hand-over with a mailbox whose `enqueue` waits (oracle cases `MBX-PT-01..05` of `cases/oracle-only/mailbox.golden`).
//! Rakka has no bounded mailbox yet (PR 7a), so the real cell is replaced by one whose `tell` waits for the same reason: a message to
//! `m1` blocks until the test releases it. Everything else is the real reference, system and event stream; the same script runs here
//! and its line is compared with the golden one.

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::props::AnyMessage;
use crate::actor::repointable::{PushPolicy, RepointableActorRef};
use crate::actor::system::{ActorSystem, Settings};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::{Channel, DeadLetter, WarningEvent};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");
const REFERENCE_PATH: &str = "pekko://mbx/user/spawner/c";

fn recorded(case: &str) -> &'static str {
    ORACLE_GOLDEN
        .lines()
        .find(|line| line.starts_with(case))
        .and_then(|line| line.split_once(" -> "))
        .map(|(_, observed)| observed)
        .expect("the golden has the case")
}

fn listed(items: &Mutex<Vec<String>>) -> String {
    format!("[{}]", items.lock().unwrap().join(", "))
}

/// The real cell as far as the hand-over is concerned: it logs what it is given and waits inside `tell("m1")` until released.
struct BlockingCell {
    path: ActorPath,
    log: Mutex<Vec<String>>,
    suspends: AtomicUsize,
    entered: Mutex<Sender<()>>,
    release: Mutex<Receiver<()>>,
}

impl InternalActorRef for BlockingCell {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        let text = msg.downcast_ref::<String>().cloned().unwrap_or_default();
        if text == "m1" {
            self.entered.lock().unwrap().send(()).unwrap();
            self.release.lock().unwrap().recv().unwrap();
        }
        self.log.lock().unwrap().push(format!("r:{text}"));
    }
    fn send_system(&self, msg: SystemMessage) {
        if matches!(msg, SystemMessage::Suspend) {
            self.suspends.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

/// A subscriber that is not an actor: records the dead letters (their text) and the warnings (`logSource|logClass|message`).
struct EventRecorder {
    path: ActorPath,
    dead_letters: Mutex<Vec<String>>,
    warnings: Mutex<Vec<String>>,
}

impl InternalActorRef for EventRecorder {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>() {
            let text = dead.message.downcast_ref::<String>().cloned();
            self.dead_letters
                .lock()
                .unwrap()
                .push(text.unwrap_or_default());
        } else if let Some(warning) = msg.downcast_ref::<WarningEvent>() {
            self.warnings.lock().unwrap().push(format!(
                "{}|{}|{}",
                warning.log_source, warning.log_class, warning.message
            ));
        }
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

/// An unstarted reference with "m1" queued, whose hand-over has started and is waiting inside the real cell.
struct BlockedHandOver {
    _system: ActorSystem,
    reference: Arc<RepointableActorRef>,
    cell: Arc<BlockingCell>,
    recorder: Arc<EventRecorder>,
    release: Sender<()>,
    hand_over: Option<JoinHandle<()>>,
}

/// A system, an unstarted reference with "m1" queued and a recorder of the dead letters and warnings.
struct Unstarted {
    system: ActorSystem,
    path: ActorPath,
    reference: Arc<RepointableActorRef>,
    recorder: Arc<EventRecorder>,
}

impl Unstarted {
    fn new(unstarted_push_timeout: Duration) -> Unstarted {
        let system = ActorSystem::create_with_settings(
            "mbx",
            Settings {
                parallelism: 1,
                unstarted_push_timeout,
                ..Settings::default()
            },
        )
        .unwrap();
        let path = ActorPath::from_string(REFERENCE_PATH).unwrap();
        let recorder = Arc::new(EventRecorder {
            path: ActorPath::from_string("pekko://mbx/system/event-recorder").unwrap(),
            dead_letters: Mutex::default(),
            warnings: Mutex::default(),
        });
        let subscriber = ActorRef(recorder.clone());
        system
            .event_stream()
            .subscribe(&subscriber, Channel::DeadLetter);
        system
            .event_stream()
            .subscribe(&subscriber, Channel::Warning);
        let reference = RepointableActorRef::for_actor(
            path.clone(),
            Box::new(|_| unreachable!("the real cell is handed over directly")),
            PushPolicy {
                timeout: unstarted_push_timeout,
                events: system.0.events.clone(),
                dead_letters: system.0.dead_letters.clone(),
            },
        );
        reference.tell(Arc::new("m1".to_owned()), None);
        Unstarted {
            system,
            path,
            reference,
            recorder,
        }
    }
}

impl BlockedHandOver {
    fn start(unstarted_push_timeout: Duration) -> BlockedHandOver {
        let Unstarted {
            system,
            path,
            reference,
            recorder,
        } = Unstarted::new(unstarted_push_timeout);
        let (entered_tx, entered_rx) = channel();
        let (release, release_rx) = channel();
        let cell = Arc::new(BlockingCell {
            path,
            log: Mutex::default(),
            suspends: AtomicUsize::new(0),
            entered: Mutex::new(entered_tx),
            release: Mutex::new(release_rx),
        });
        let hand_over = {
            let (reference, cell) = (reference.clone(), cell.clone());
            std::thread::spawn(move || {
                reference.replace_with(cell);
            })
        };
        entered_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the hand-over reaches the real cell");
        BlockedHandOver {
            _system: system,
            reference,
            cell,
            recorder,
            release,
            hand_over: Some(hand_over),
        }
    }

    /// Lets "m1" through and waits for the hand-over to end.
    fn release_and_finish(&mut self) {
        self.release.send(()).unwrap();
        self.hand_over.take().unwrap().join().unwrap();
    }

    /// Sends "m2" from another thread and says how long the send took; a send that has not returned after five seconds fails the test
    /// instead of hanging it.
    fn time_a_send(&self, text: &'static str) -> Duration {
        self.time_a_send_of(text.to_owned())
    }

    fn time_a_send_of<M: std::any::Any + Send + Sync>(&self, message: M) -> Duration {
        let reference = self.reference.clone();
        let (elapsed_tx, elapsed_rx) = channel();
        std::thread::spawn(move || {
            let started = Instant::now();
            reference.tell(Arc::new(message), None);
            let _ = elapsed_tx.send(started.elapsed());
        });
        elapsed_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the send returned within five seconds")
    }

    fn send_from_another_thread(&self, text: &'static str) -> JoinHandle<()> {
        let reference = self.reference.clone();
        std::thread::spawn(move || reference.tell(Arc::new(text.to_owned()), None))
    }

    fn wait_until_a_sender_waits(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.reference.waiting_senders() == 0 {
            assert!(
                Instant::now() < deadline,
                "no sender is waiting for the lock"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn wait_for_the_drop(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.recorder.dead_letters.lock().unwrap().is_empty()
            || self.recorder.warnings.lock().unwrap().is_empty()
        {
            assert!(Instant::now() < deadline, "the message was not dropped");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn finish_within_five_seconds(thread: JoinHandle<()>) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !thread.is_finished() {
            assert!(Instant::now() < deadline, "the sender is still waiting");
            std::thread::sleep(Duration::from_millis(1));
        }
        thread.join().unwrap();
    }

    fn absence_pause() {
        std::thread::sleep(Duration::from_millis(300)); // a bounded wait for an absence, as the oracle does
    }
}

#[test]
fn a_send_that_waits_longer_than_the_timeout_is_a_dead_letter_with_a_warning() {
    let mut blocked = BlockedHandOver::start(Duration::from_millis(300));
    let waited = blocked.time_a_send("m2");
    blocked.wait_for_the_drop();
    blocked.release_and_finish();
    BlockedHandOver::absence_pause();
    let observed = format!(
        "waitedTheTimeout={} returnedWithinBound={} deadLetters={} warnings={} log={}",
        waited >= Duration::from_millis(300),
        waited < Duration::from_secs(5),
        listed(&blocked.recorder.dead_letters),
        listed(&blocked.recorder.warnings),
        listed(&blocked.cell.log)
    );
    assert_eq!(observed, recorded("MBX-PT-01"));
}

#[test]
fn a_send_that_waits_while_the_hand_over_ends_before_the_timeout_is_delivered_after_the_queued_messages()
 {
    let mut blocked = BlockedHandOver::start(Duration::from_secs(10));
    let sender = blocked.send_from_another_thread("m2");
    blocked.wait_until_a_sender_waits();
    blocked.release_and_finish();
    BlockedHandOver::finish_within_five_seconds(sender);
    BlockedHandOver::absence_pause();
    let observed = format!(
        "log={} deadLetters={} warnings={}",
        listed(&blocked.cell.log),
        listed(&blocked.recorder.dead_letters),
        listed(&blocked.recorder.warnings)
    );
    assert_eq!(observed, recorded("MBX-PT-02"));
}

#[test]
fn a_system_message_waits_for_the_lock_without_a_timeout_and_is_delivered_once_the_hand_over_ends()
{
    let mut blocked = BlockedHandOver::start(Duration::from_millis(300));
    let reference = blocked.reference.clone();
    let sender = std::thread::spawn(move || reference.send_system(SystemMessage::Suspend));
    blocked.wait_until_a_sender_waits();
    std::thread::sleep(Duration::from_millis(1000)); // three timeouts: the wait is not bounded by them
    let still_blocked = !sender.is_finished();
    blocked.release_and_finish();
    BlockedHandOver::finish_within_five_seconds(sender);
    let observed = format!(
        "stillBlockedAfterThreeTimeouts={still_blocked} suspendCount={} deadLetters={} warnings={}",
        blocked.cell.suspends.load(Ordering::SeqCst),
        listed(&blocked.recorder.dead_letters),
        listed(&blocked.recorder.warnings)
    );
    assert_eq!(observed, recorded("MBX-PT-03"));
}

/// A zero timeout does not wait. The oracle's `-1s` gives the same line (`MBX-PT-05`): the JVM treats a negative `tryLock` timeout as
/// zero, and Rakka's `Duration` cannot be negative.
#[test]
fn a_zero_timeout_drops_a_send_that_finds_the_lock_held_at_once() {
    for case in ["MBX-PT-04", "MBX-PT-05"] {
        let mut blocked = BlockedHandOver::start(Duration::ZERO);
        let waited = blocked.time_a_send("m2");
        blocked.wait_for_the_drop();
        blocked.release_and_finish();
        BlockedHandOver::absence_pause();
        let observed = format!(
            "returnedWithinBound={} deadLetters={} warnings={} log={}",
            waited < Duration::from_secs(5),
            listed(&blocked.recorder.dead_letters),
            listed(&blocked.recorder.warnings),
            listed(&blocked.cell.log)
        );
        assert_eq!(observed, recorded(case));
    }
}

/// The real cell sends "m2" to the reference while it takes "m1" (the oracle's `ReentrantHooks`).
struct ReentrantCell {
    path: ActorPath,
    log: Mutex<Vec<String>>,
    reference: std::sync::OnceLock<std::sync::Weak<RepointableActorRef>>,
}

impl InternalActorRef for ReentrantCell {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
        let text = msg.downcast_ref::<String>().cloned().unwrap_or_default();
        if text == "m1" {
            let reference = self.reference.get().and_then(std::sync::Weak::upgrade);
            reference
                .expect("the test sets the reference")
                .tell(Arc::new("m2".to_owned()), None);
        }
        self.log.lock().unwrap().push(format!("r:{text}"));
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

/// `ReentrantLock`: the thread that is handing the queue over can send to the reference; its message joins the queue instead of
/// waiting for a lock that it holds itself (which would be a dead letter after the timeout, or a deadlock for a system message).
#[test]
fn the_thread_that_hands_the_queue_over_can_send_to_the_same_reference() {
    let unstarted = Unstarted::new(Duration::from_millis(300));
    let cell = Arc::new(ReentrantCell {
        path: unstarted.path.clone(),
        log: Mutex::default(),
        reference: std::sync::OnceLock::new(),
    });
    cell.reference
        .set(Arc::downgrade(&unstarted.reference))
        .unwrap();
    assert!(unstarted.reference.replace_with(cell.clone()));
    BlockedHandOver::absence_pause();
    let observed = format!(
        "log={} deadLetters={} warnings={}",
        listed(&cell.log),
        listed(&unstarted.recorder.dead_letters),
        listed(&unstarted.recorder.warnings)
    );
    assert_eq!(observed, recorded("MBX-PT-06"));
}

/// A timeout too long to be added to the clock is a wait without a limit, not a panic in `tell` (`tryLock(Long.MaxValue, NANOSECONDS)`
/// simply never expires).
#[test]
fn a_timeout_too_long_for_the_clock_does_not_panic_the_sender() {
    let unstarted = Unstarted::new(Duration::MAX);
    unstarted.reference.tell(Arc::new("m2".to_owned()), None);
    assert_eq!(unstarted.reference.queued_user_messages(), 2);
}

/// Pekko prints the JVM class of the dropped message. A Rust `String` is the JVM `String`; for any other Rust type the class is not
/// invented: the text says the type is unknown (the one place the warning cannot match the JVM for every message type).
#[test]
fn a_dropped_message_of_a_type_without_a_jvm_class_reports_an_unknown_type_not_an_invented_class() {
    let mut blocked = BlockedHandOver::start(Duration::ZERO);
    blocked.time_a_send_of(42_u32);
    blocked.wait_for_the_drop();
    blocked.release_and_finish();
    assert_eq!(
        *blocked.recorder.warnings.lock().unwrap(),
        [format!(
            "{REFERENCE_PATH}|org.apache.pekko.actor.UnstartedCell|dropping message of typeclass <unknown Rust type> due to lock timeout"
        )]
    );
}

fn runtime_settings_configured_with(application_conf: &str) -> Settings {
    let config = crate::config::Config::parse_string(application_conf)
        .unwrap()
        .with_fallback(&crate::config::Config::default_reference())
        .resolve()
        .unwrap();
    Settings::from_config(&crate::config::Settings::new("configured", config).unwrap()).unwrap()
}

/// The value of `pekko.actor.unstarted-push-timeout` reaches the runtime `Settings`. The JVM reads it with `getMillisDuration`
/// (recorded cases `cs-0099..0102`); a negative timeout makes `tryLock` behave as zero (`MBX-PT-05`).
#[test]
fn the_configured_unstarted_push_timeout_reaches_the_runtime_settings() {
    for (configured, expected) in [
        ("", Duration::from_secs(10)),
        (
            "pekko.actor.unstarted-push-timeout = 300ms",
            Duration::from_millis(300),
        ),
        (
            "pekko.actor.unstarted-push-timeout = 1 minute",
            Duration::from_secs(60),
        ),
        ("pekko.actor.unstarted-push-timeout = 0s", Duration::ZERO),
        ("pekko.actor.unstarted-push-timeout = -1s", Duration::ZERO),
        ("pekko.actor.unstarted-push-timeout = -5ms", Duration::ZERO),
    ] {
        assert_eq!(
            runtime_settings_configured_with(configured).unstarted_push_timeout,
            expected,
            "{configured:?}"
        );
    }
}

/// The configured value is the one the references of the actors made by `actorOf` use (the real creation path).
#[test]
fn an_actor_of_reference_uses_the_configured_unstarted_push_timeout() {
    let system = ActorSystem::create_with_settings(
        "configured",
        runtime_settings_configured_with("pekko.actor.unstarted-push-timeout = 300ms"),
    )
    .unwrap();
    let actor = system
        .actor_of(crate::actor::props::Props::new(|| NoOp))
        .unwrap();
    let timeout = actor.0.as_repointable().unwrap().push_timeout();
    assert_eq!(timeout, Some(Duration::from_millis(300)));
}

struct NoOp;
impl crate::actor::props::Actor for NoOp {}

/// `isStarted` while the hand-over is in progress (oracle case `MBX-UT-09`: `isStarted=false` during, `true` after). Pekko reads a
/// volatile field; Rakka reads under the reference's own mutex, which the hand-over does not hold while it delivers. The test does not
/// assume either: the read runs on its own thread with a bounded wait, and the value it returns is what is asserted.
#[test]
fn is_started_is_false_while_the_hand_over_is_in_progress_and_true_after() {
    let mut blocked = BlockedHandOver::start(Duration::from_secs(10));
    let reader = {
        let reference = blocked.reference.clone();
        let (answer_tx, answer) = channel();
        std::thread::spawn(move || {
            let _ = answer_tx.send(reference.is_started());
        });
        answer
    };
    let during = reader.recv_timeout(Duration::from_secs(2));
    blocked.release_and_finish(); // never leave the hand-over blocked, whatever the read did
    let during =
        during.unwrap_or_else(|_| reader.recv().expect("the read ends after the hand-over"));
    assert!(!during, "isStarted during the hand-over");
    assert!(
        blocked.reference.is_started(),
        "isStarted after the hand-over"
    );
    assert!(
        recorded("MBX-UT-09").contains("duringHandOver: isStarted=false")
            && recorded("MBX-UT-09").contains("afterHandOver: isStarted=true"),
        "the golden pins the same two values"
    );
}
