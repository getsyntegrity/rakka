//! `system.scheduler().schedule_once` and `Cancellable` with real actors (AC-SCH-001, 005, 008.2, 009; `SchedulerSpec`).
//! Absence is shown by order, not by sleeping: a later marker arrives first or alone. Its margin (the cancel or the
//! `terminate` runs within the delay of the message that must not arrive) is far above what a loaded machine needs.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::actor::{ActorRef, ErrorClass};

const WAIT: Duration = Duration::from_secs(10);
const ENQUEUE_AFTER_SHUTDOWN: &str = "cannot enqueue after timer shutdown";

/// Reports every text it receives with the name of its sender.
struct Probe(Mutex<Sender<(String, String)>>);

impl Actor for Probe {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<&str>() else {
            return false;
        };
        let sender = ctx.sender().path().name().to_string();
        self.0.lock().unwrap().send((text.to_string(), sender)).ok();
        true
    }
}

/// A system that is terminated when the test ends, also by a failure.
struct System(ActorSystem);

impl Drop for System {
    fn drop(&mut self) {
        self.0.terminate().wait(WAIT);
    }
}

fn system(name: &str) -> System {
    System(ActorSystem::create(name).unwrap())
}

fn probe(system: &System, name: &str) -> (ActorRef, Receiver<(String, String)>) {
    let (tx, rx) = channel();
    let props = Props::new(move || Probe(Mutex::new(tx.clone())));
    (system.0.actor_of_named(props, name).unwrap(), rx)
}

fn next(rx: &Receiver<(String, String)>) -> (String, String) {
    rx.recv_timeout(WAIT).expect("a message")
}

#[test]
fn a_message_arrives_once_never_early_with_the_given_sender_and_cannot_be_cancelled_afterwards() {
    let system = system("once");
    let (target, rx) = probe(&system, "target");
    let (sender, _) = probe(&system, "asker");
    let delay = Duration::from_millis(50);
    let start = Instant::now();
    let task = system
        .0
        .scheduler()
        .schedule_once(delay, &target, "tick", Some(&sender))
        .unwrap();
    assert_eq!(next(&rx), ("tick".into(), "asker".into()));
    assert!(start.elapsed() >= delay);
    assert_eq!((task.cancel(), task.is_cancelled()), (false, false));
    system
        .0
        .scheduler()
        .schedule_once(delay, &target, "marker", None)
        .unwrap();
    assert_eq!(next(&rx), ("marker".into(), "deadLetters".into()));
}

#[test]
fn a_cancelled_message_is_never_sent_and_only_the_first_cancel_succeeds() {
    let system = system("cancel");
    let (target, rx) = probe(&system, "target");
    let scheduler = system.0.scheduler();
    let task = scheduler
        .schedule_once(Duration::from_secs(1), &target, "cancelled", None)
        .unwrap();
    assert!(!task.is_cancelled());
    let copy = task.clone();
    assert_eq!((task.cancel(), copy.is_cancelled()), (true, true));
    assert_eq!((copy.cancel(), task.is_cancelled()), (false, true));
    scheduler
        .schedule_once(Duration::from_millis(1200), &target, "marker", None)
        .unwrap();
    assert_eq!(next(&rx).0, "marker");
}

#[test]
fn a_zero_delay_sends_at_once_and_returns_a_task_that_is_not_cancellable() {
    let system = system("zero");
    let (target, rx) = probe(&system, "target");
    let task = system
        .0
        .scheduler()
        .schedule_once(Duration::ZERO, &target, "now", None)
        .unwrap();
    assert_eq!((task.cancel(), task.is_cancelled()), (false, false));
    assert_eq!(next(&rx).0, "now");
}

#[test]
fn a_delay_beyond_the_reach_of_the_wheel_is_rejected_with_the_pekko_text() {
    let system = system("far");
    let (target, _rx) = probe(&system, "target");
    let schedule = |delay| {
        system
            .0
            .scheduler()
            .schedule_once(delay, &target, "far", None)
    };
    // `Int.MaxValue` ticks of 10 ms is the largest delay; it is rounded up to the tick before the check
    let largest = Duration::from_millis(21_474_836_470);
    let task = schedule(largest).unwrap();
    assert_eq!(format!("{task:?}"), "Cancellable(is_cancelled: false)");
    assert!(task.cancel());
    let error = schedule(largest + Duration::from_nanos(1)).unwrap_err();
    assert_eq!(error.class(), ErrorClass::IllegalArgument);
    assert_eq!(
        error.message(),
        "Task scheduled with [21474836] seconds delay, which is too far in future, maximum delay is [21474835] seconds"
    );
    assert_eq!(
        schedule(Duration::MAX).unwrap_err().class(),
        ErrorClass::IllegalArgument
    );
}

#[test]
fn termination_drops_the_pending_messages_and_rejects_new_ones() {
    let closing = system("closing");
    let other = system("other");
    let (target, rx) = probe(&other, "target");
    let pending = closing
        .0
        .scheduler()
        .schedule_once(Duration::from_secs(1), &target, "pending", None)
        .unwrap();
    closing.0.terminate().wait(WAIT).expect("terminated").ok();
    other
        .0
        .scheduler()
        .schedule_once(Duration::from_millis(1200), &target, "marker", None)
        .unwrap();
    assert_eq!(next(&rx).0, "marker");
    assert_eq!((pending.cancel(), pending.is_cancelled()), (true, true));
    for delay in [Duration::ZERO, Duration::from_secs(1), Duration::MAX] {
        let error = closing
            .0
            .scheduler()
            .schedule_once(delay, &target, "late", None)
            .unwrap_err();
        assert_eq!(error.class(), ErrorClass::IllegalState);
        assert_eq!(error.message(), ENQUEUE_AFTER_SHUTDOWN);
    }
}

struct PanicsOnDrop(Sender<()>);

impl Drop for PanicsOnDrop {
    fn drop(&mut self) {
        self.0.send(()).ok();
        panic!("drop of a scheduled message");
    }
}

#[test]
fn a_pending_message_whose_drop_panics_does_not_stop_the_termination() {
    let system = system("panics");
    let (target, _rx) = probe(&system, "target");
    let (tx, dropped) = channel();
    system
        .0
        .scheduler()
        .schedule_once(Duration::from_secs(60), &target, PanicsOnDrop(tx), None)
        .unwrap();
    assert!(system.0.terminate().wait(WAIT).is_some());
    dropped.recv_timeout(WAIT).expect("the message was dropped");
}
