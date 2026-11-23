// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/RepointableActorRef.scala (Apache-2.0).
//! `RepointableActorRef` and `UnstartedCell` (`RepointableActorRef.scala:38-325`): the reference an asynchronous `actorOf` hands out
//! before the actor's real cell exists.
//!
//! The reference is stable: its path and uid never change. Until the real cell is installed ([`RepointableActorRef::replace_with`],
//! Pekko's `point` → `UnstartedCell.replaceWith`), user messages wait in a queue and system messages in another, and nothing
//! reaches any actor. Installing delivers the system messages first (oldest first), then the user messages in the order they were
//! sent, to the real cell, and from then on every message goes straight to it. This is why a `Suspend` sent after some user
//! messages still takes effect before them (oracle cases `MBX-UC-01..03`).
//!
//! `ActorSystem::actor_of` creates one through `ActorCell::attach_child(.., asynchronous = true)` (Q-MBX-11, slice B).
//! A failure to make the real cell is not lost: the actor fails to initialise instead (slice C1, `init_failure_tests`).
//! Differences from Pekko, to be settled in the next slices of C: the lock is a plain `Mutex` (no `UnstartedPushTimeout`: a send that
//! waits for the lock more than that long is a dead letter in Pekko), and it is held while the queues are handed over, as in Pekko
//! but without re-entrancy: the real cell must not send to this reference from inside `tell`/`send_system`.

#![allow(dead_code)] // some accessors are only used by tests

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};

use crate::actor::ActorPath;
use crate::actor::ActorRef;
use crate::actor::actor_ref::InternalActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::fault::Throwable;
use crate::actor::props::AnyMessage;
use crate::dispatch::message_queue::Envelope;
use crate::dispatch::sysmsg::SystemMessage;

/// `UnstartedCell`: what has been sent to an actor whose real cell does not exist yet.
#[derive(Default)]
struct UnstartedCell {
    /// user messages, oldest first (`queue`)
    user_messages: VecDeque<Envelope>,
    /// system messages, oldest first (`sysmsgQueue` keeps the newest first and is reversed when drained)
    system_messages: Vec<SystemMessage>,
}

enum Underlying {
    Unstarted(UnstartedCell),
    Started(Arc<dyn InternalActorRef>),
}

/// How the real cell of an actor is made (`newCell`), given the reference the actor will know as its own (`self`).
type NewCell = Box<dyn FnOnce(&ActorRef) -> Arc<ActorCell> + Send>;

/// `RepointableActorRef`.
pub(crate) struct RepointableActorRef {
    path: ActorPath,
    underlying: Mutex<Underlying>,
    me: Weak<RepointableActorRef>,
    /// `newCell`: taken by the first `point`
    new_cell: Mutex<Option<NewCell>>,
}

impl RepointableActorRef {
    /// The state is always consistent (it switches in one step), so a panic of the cell while it takes the messages, which poisons
    /// the lock, must not stop later senders.
    fn lock(&self) -> std::sync::MutexGuard<'_, Underlying> {
        self.underlying
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// A reference to an actor that has not started (`initialize`: `swapCell(new UnstartedCell(...))`).
    pub(crate) fn new(path: ActorPath) -> Arc<RepointableActorRef> {
        Self::build(path, None)
    }

    /// A reference whose cell is made by `new_cell` when the supervisor processes `Supervise` (`point`).
    pub(crate) fn for_actor(path: ActorPath, new_cell: NewCell) -> Arc<RepointableActorRef> {
        Self::build(path, Some(new_cell))
    }

    fn build(path: ActorPath, new_cell: Option<NewCell>) -> Arc<RepointableActorRef> {
        Arc::new_cyclic(|me| RepointableActorRef {
            path,
            underlying: Mutex::new(Underlying::Unstarted(UnstartedCell::default())),
            me: me.clone(),
            new_cell: Mutex::new(new_cell),
        })
    }

    /// `point(catchFailures)` (`RepointableActorRef.scala:112-135`): makes the real cell, starts it (the dispatcher attaches it) and
    /// installs it. On a started reference, or one that has no cell factory, it does nothing. The factory already catches a failure of
    /// `newCell` (`catchFailures = true`: every `RepointableActorRef` is made for an asynchronous child, `Supervise(async = true)`) and answers `ActorCell::new_with_failure`.
    pub(crate) fn point(&self) {
        let Some(me) = self.me.upgrade() else {
            return;
        };
        let Some(new_cell) = self
            .new_cell
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        else {
            return;
        };
        let cell = new_cell(&ActorRef(me));
        cell.start();
        self.replace_with(cell.local_ref().0);
    }

    /// `point`/`replaceWith`: installs the real cell, handing it everything that was sent before. Returns false, doing nothing, when a
    /// cell was already installed. If the cell panics while taking the messages, the reference still points to it (Pekko's `finally
    /// { swapCell(cell) }`) and what was not handed over is lost.
    pub(crate) fn replace_with(&self, cell: Arc<dyn InternalActorRef>) -> bool {
        // the lock stays held while the messages are handed over: a sender that arrives meanwhile waits, then finds the cell
        let mut underlying = self.lock();
        if matches!(&*underlying, Underlying::Started(_)) {
            return false; // `point` on a started reference answers `this`
        }
        let Underlying::Unstarted(unstarted) =
            std::mem::replace(&mut *underlying, Underlying::Started(cell.clone()))
        else {
            unreachable!("checked just above, under the same lock");
        };
        let UnstartedCell {
            user_messages,
            system_messages,
        } = unstarted;
        for message in system_messages {
            cell.send_system(message);
        }
        for envelope in user_messages {
            cell.tell(envelope.message, envelope.sender);
        }
        true
    }

    /// `isStarted`.
    pub(crate) fn is_started(&self) -> bool {
        matches!(&*self.lock(), Underlying::Started(_))
    }

    /// A weak handle to the installed cell, to check in tests that nothing keeps it alive after termination.
    #[cfg(test)]
    pub(crate) fn installed_cell_for_tests(&self) -> Option<std::sync::Weak<dyn InternalActorRef>> {
        match &*self.lock() {
            Underlying::Started(cell) => Some(Arc::downgrade(cell)),
            Underlying::Unstarted(_) => None,
        }
    }

    /// `suspend()`: a `Suspend` system message.
    pub(crate) fn suspend(&self) {
        self.send_system(SystemMessage::Suspend);
    }

    /// `resume(causedByFailure)`: a `Resume` system message.
    pub(crate) fn resume(&self, caused_by_failure: Option<Throwable>) {
        self.send_system(SystemMessage::Resume {
            cause: caused_by_failure,
        });
    }

    /// `restart(cause)`: a `Recreate` system message.
    pub(crate) fn restart(&self, cause: Throwable) {
        self.send_system(SystemMessage::Recreate { cause });
    }

    /// `UnstartedCell.numberOfMessages` while unstarted: the user messages waiting.
    pub(crate) fn queued_user_messages(&self) -> usize {
        match &*self.lock() {
            Underlying::Unstarted(unstarted) => unstarted.user_messages.len(),
            Underlying::Started(_) => 0,
        }
    }

    /// The system messages waiting.
    pub(crate) fn queued_system_messages(&self) -> usize {
        match &*self.lock() {
            Underlying::Unstarted(unstarted) => unstarted.system_messages.len(),
            Underlying::Started(_) => 0,
        }
    }
}

impl InternalActorRef for RepointableActorRef {
    fn path(&self) -> &ActorPath {
        &self.path
    }

    fn as_repointable(&self) -> Option<&RepointableActorRef> {
        Some(self)
    }

    #[cfg(test)]
    fn mailbox_view(&self) -> String {
        match &*self.lock() {
            Underlying::Unstarted(unstarted) => format!(
                "unstarted queued={} system={}",
                unstarted.user_messages.len(),
                unstarted.system_messages.len()
            ),
            Underlying::Started(cell) => cell.mailbox_view(),
        }
    }

    #[cfg(test)]
    fn has_system_messages(&self) -> bool {
        match &*self.lock() {
            Underlying::Unstarted(unstarted) => !unstarted.system_messages.is_empty(),
            Underlying::Started(cell) => cell.has_system_messages(),
        }
    }

    #[cfg(test)]
    fn attach_child_for_tests(
        &self,
        props: crate::actor::props::Props,
        name: &str,
    ) -> Result<ActorRef, crate::actor::ActorError> {
        let started = match &*self.lock() {
            Underlying::Started(cell) => Some(cell.clone()),
            Underlying::Unstarted(_) => None,
        };
        match started {
            Some(cell) => cell.attach_child_for_tests(props, name),
            None => Err(crate::actor::ActorError::new(
                crate::actor::ErrorClass::IllegalState,
                "the actor has not started",
            )),
        }
    }

    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        let started = {
            let mut underlying = self.lock();
            match &mut *underlying {
                Underlying::Unstarted(unstarted) => {
                    unstarted.user_messages.push_back(Envelope {
                        message: msg,
                        sender,
                    });
                    return;
                }
                Underlying::Started(cell) => cell.clone(),
            }
        };
        started.tell(msg, sender);
    }

    fn send_system(&self, msg: SystemMessage) {
        let started = {
            let mut underlying = self.lock();
            match &mut *underlying {
                Underlying::Unstarted(unstarted) => {
                    unstarted.system_messages.push(msg);
                    return;
                }
                Underlying::Started(cell) => cell.clone(),
            }
        };
        started.send_system(msg);
    }

    fn stop(&self) {
        self.send_system(SystemMessage::Terminate);
    }

    fn is_terminated(&self) -> bool {
        match &*self.lock() {
            Underlying::Unstarted(_) => false,
            Underlying::Started(cell) => cell.is_terminated(),
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Barrier, Mutex as StdMutex};
    use std::thread;

    fn name_of(message: &SystemMessage) -> &'static str {
        match message {
            SystemMessage::Create(_) => "Create",
            SystemMessage::Terminate => "Terminate",
            SystemMessage::Failed { .. } => "Failed",
            SystemMessage::Recreate { .. } => "Recreate",
            SystemMessage::Resume { .. } => "Resume",
            SystemMessage::Suspend => "Suspend",
            SystemMessage::Supervise { .. } => "Supervise",
            SystemMessage::Watch { .. } => "Watch",
            SystemMessage::Unwatch { .. } => "Unwatch",
            SystemMessage::DeathWatchNotification { .. } => "DeathWatchNotification",
        }
    }

    /// The real cell's stand-in: it records what it is handed, in order (the one fact these tests are about).
    #[derive(Default)]
    struct RecordingCell {
        path: Option<ActorPath>,
        events: StdMutex<Vec<String>>,
        terminated: AtomicBool,
        panic_on: Option<&'static str>,
    }

    impl RecordingCell {
        fn events(&self) -> Vec<String> {
            self.events.lock().unwrap().clone()
        }
    }

    impl InternalActorRef for RecordingCell {
        fn path(&self) -> &ActorPath {
            self.path
                .as_ref()
                .expect("the double is only asked for its path in one test")
        }
        fn tell(&self, msg: AnyMessage, _sender: Option<ActorRef>) {
            let text = msg.downcast_ref::<String>().cloned().unwrap_or_default();
            if self.panic_on == Some(text.as_str()) {
                panic!("the cell fails on {text}");
            }
            self.events.lock().unwrap().push(format!("user:{text}"));
        }
        fn send_system(&self, msg: SystemMessage) {
            self.events
                .lock()
                .unwrap()
                .push(format!("system:{}", name_of(&msg)));
        }
        fn stop(&self) {}
        fn is_terminated(&self) -> bool {
            self.terminated.load(Ordering::SeqCst)
        }
    }

    fn unstarted(name: &str) -> Arc<RepointableActorRef> {
        let root = ActorPath::from_string("pekko://sys/user").unwrap();
        RepointableActorRef::new(root.new_child(name, 7))
    }

    fn say(reference: &RepointableActorRef, text: &str) {
        reference.tell(Arc::new(text.to_owned()), None);
    }

    #[test]
    fn the_reference_keeps_its_path_and_identity_when_the_cell_is_installed() {
        let reference = unstarted("a");
        let handle = ActorRef(reference.clone());
        let before = (handle.path().to_string(), handle.path().uid());
        assert!(reference.replace_with(Arc::new(RecordingCell::default())));
        assert_eq!((handle.path().to_string(), handle.path().uid()), before);
        assert!(
            handle == ActorRef(reference.clone()),
            "same path and uid, same actor"
        );
        assert_eq!(before, ("pekko://sys/user/a".to_owned(), 7));
    }

    #[test]
    fn what_is_sent_before_the_cell_exists_waits_and_reaches_nobody() {
        let reference = unstarted("a");
        say(&reference, "m1");
        say(&reference, "m2");
        reference.suspend();
        assert_eq!(
            (
                reference.queued_user_messages(),
                reference.queued_system_messages(),
                reference.is_started()
            ),
            (2, 1, false)
        );
    }

    #[test]
    fn installing_delivers_the_system_messages_first_then_the_user_messages_in_order() {
        let reference = unstarted("a");
        say(&reference, "m1");
        reference.suspend();
        say(&reference, "m2");
        reference.resume(None);
        let cell = Arc::new(RecordingCell::default());
        assert!(reference.replace_with(cell.clone()));
        assert_eq!(
            cell.events(),
            ["system:Suspend", "system:Resume", "user:m1", "user:m2"],
            "the interleaving of arrival is not kept: Suspend goes before m1"
        );
        assert_eq!(
            (
                reference.queued_user_messages(),
                reference.queued_system_messages(),
                reference.is_started()
            ),
            (0, 0, true)
        );
    }

    #[test]
    fn watch_and_unwatch_are_ordinary_system_messages_while_unstarted() {
        let reference = unstarted("a");
        let peer = ActorRef(unstarted("b"));
        reference.send_system(SystemMessage::Watch {
            watchee: peer.clone(),
            watcher: peer.clone(),
        });
        reference.send_system(SystemMessage::Unwatch {
            watchee: peer.clone(),
            watcher: peer,
        });
        reference.stop();
        let cell = Arc::new(RecordingCell::default());
        reference.replace_with(cell.clone());
        assert_eq!(
            cell.events(),
            ["system:Watch", "system:Unwatch", "system:Terminate"]
        );
    }

    #[test]
    fn suspend_resume_restart_and_stop_are_system_messages_before_the_cell_exists() {
        let reference = unstarted("a");
        reference.suspend();
        reference.resume(Some(Throwable::illegal_state("boom")));
        reference.restart(Throwable::illegal_state("boom"));
        reference.stop();
        let cell = Arc::new(RecordingCell::default());
        reference.replace_with(cell.clone());
        assert_eq!(
            cell.events(),
            [
                "system:Suspend",
                "system:Resume",
                "system:Recreate",
                "system:Terminate"
            ]
        );
    }

    #[test]
    fn after_installation_messages_go_straight_to_the_cell_behind_the_transferred_ones() {
        let reference = unstarted("a");
        say(&reference, "early");
        let cell = Arc::new(RecordingCell::default());
        reference.replace_with(cell.clone());
        say(&reference, "late");
        reference.send_system(SystemMessage::Suspend);
        assert_eq!(cell.events(), ["user:early", "user:late", "system:Suspend"]);
        assert_eq!(reference.queued_user_messages(), 0);
    }

    #[test]
    fn an_unstarted_actor_is_not_terminated_and_a_started_one_answers_for_its_cell() {
        let reference = unstarted("a");
        assert!(!reference.is_terminated());
        let cell = Arc::new(RecordingCell::default());
        reference.replace_with(cell.clone());
        assert!(!reference.is_terminated());
        cell.terminated.store(true, Ordering::SeqCst);
        assert!(reference.is_terminated());
    }

    #[test]
    fn installing_a_second_cell_is_ignored() {
        let reference = unstarted("a");
        let first = Arc::new(RecordingCell::default());
        let second = Arc::new(RecordingCell::default());
        assert!(reference.replace_with(first.clone()));
        assert!(!reference.replace_with(second.clone()));
        say(&reference, "m");
        assert_eq!(
            (first.events(), second.events()),
            (vec!["user:m".to_owned()], vec![])
        );
    }

    #[test]
    fn a_cell_that_panics_while_taking_the_messages_still_replaces_the_unstarted_one() {
        let reference = unstarted("a");
        say(&reference, "ok");
        say(&reference, "bad");
        say(&reference, "lost");
        let cell = Arc::new(RecordingCell {
            panic_on: Some("bad"),
            ..RecordingCell::default()
        });
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reference.replace_with(cell.clone())
        }));
        assert!(attempt.is_err());
        assert!(reference.is_started(), "`finally {{ swapCell(cell) }}`");
        assert_eq!(
            cell.events(),
            ["user:ok"],
            "what was not handed over is lost, as in Pekko"
        );
        say(&reference, "after");
        assert_eq!(cell.events(), ["user:ok", "user:after"]);
    }

    #[test]
    fn senders_racing_the_installation_lose_and_duplicate_nothing_and_keep_their_own_order() {
        const PRODUCERS: usize = 4;
        const EACH: usize = 200;
        let reference = unstarted("a");
        let cell = Arc::new(RecordingCell::default());
        let go = Arc::new(Barrier::new(PRODUCERS + 1));
        let producers: Vec<_> = (0..PRODUCERS)
            .map(|producer| {
                let (reference, go) = (reference.clone(), go.clone());
                thread::spawn(move || {
                    go.wait();
                    for n in 0..EACH {
                        say(&reference, &format!("{producer}:{n}"));
                    }
                })
            })
            .collect();
        go.wait();
        reference.replace_with(cell.clone());
        for producer in producers {
            producer.join().unwrap();
        }
        let delivered = cell.events();
        assert_eq!(
            delivered.len(),
            PRODUCERS * EACH,
            "no loss and no duplicate"
        );
        for producer in 0..PRODUCERS {
            let own: Vec<usize> = delivered
                .iter()
                .filter_map(|event| event.strip_prefix(&format!("user:{producer}:")))
                .map(|n| n.parse().unwrap())
                .collect();
            assert_eq!(
                own,
                (0..EACH).collect::<Vec<_>>(),
                "producer {producer} keeps its order"
            );
        }
    }

    #[test]
    fn a_terminate_racing_the_installation_reaches_the_cell_exactly_once_with_every_message() {
        const PRODUCERS: usize = 3;
        const EACH: usize = 100;
        let reference = unstarted("a");
        let cell = Arc::new(RecordingCell::default());
        let go = Arc::new(Barrier::new(PRODUCERS + 2));
        let mut racers: Vec<thread::JoinHandle<()>> = (0..PRODUCERS)
            .map(|producer| {
                let (reference, go) = (reference.clone(), go.clone());
                thread::spawn(move || {
                    go.wait();
                    for n in 0..EACH {
                        say(&reference, &format!("{producer}:{n}"));
                    }
                })
            })
            .collect();
        racers.push({
            let (reference, go) = (reference.clone(), go.clone());
            thread::spawn(move || {
                go.wait();
                reference.stop();
            })
        });
        racers.push({
            let (reference, cell, go) = (reference.clone(), cell.clone(), go.clone());
            thread::spawn(move || {
                go.wait();
                assert!(reference.replace_with(cell));
            })
        });
        for racer in racers {
            racer.join().unwrap();
        }
        let delivered = cell.events();
        let count = |wanted: &str| {
            delivered
                .iter()
                .filter(|event| event.as_str() == wanted)
                .count()
        };
        assert_eq!(
            count("system:Terminate"),
            1,
            "the Terminate is neither lost nor duplicated"
        );
        assert_eq!(delivered.len(), PRODUCERS * EACH + 1);
        for producer in 0..PRODUCERS {
            let own: Vec<usize> = delivered
                .iter()
                .filter_map(|event| event.strip_prefix(&format!("user:{producer}:")))
                .map(|n| n.parse().unwrap())
                .collect();
            assert_eq!(
                own,
                (0..EACH).collect::<Vec<_>>(),
                "producer {producer} keeps its order"
            );
        }
        assert_eq!(
            (
                reference.queued_user_messages(),
                reference.queued_system_messages()
            ),
            (0, 0)
        );
    }

    #[test]
    fn two_installations_racing_leave_exactly_one_cell_that_gets_everything() {
        let reference = unstarted("a");
        for n in 0..50 {
            say(&reference, &format!("m{n}"));
        }
        let cells = [
            Arc::new(RecordingCell::default()),
            Arc::new(RecordingCell::default()),
        ];
        let go = Arc::new(Barrier::new(2));
        let racers: Vec<_> = cells
            .iter()
            .map(|cell| {
                let (reference, cell, go) = (reference.clone(), cell.clone(), go.clone());
                thread::spawn(move || {
                    go.wait();
                    reference.replace_with(cell)
                })
            })
            .collect();
        let winners: Vec<bool> = racers
            .into_iter()
            .map(|racer| racer.join().unwrap())
            .collect();
        assert_eq!(
            winners.iter().filter(|won| **won).count(),
            1,
            "only one installation succeeds"
        );
        let winner = winners.iter().position(|won| *won).unwrap();
        assert_eq!(cells[winner].events().len(), 50);
        assert!(
            cells[1 - winner].events().is_empty(),
            "the other cell is never used"
        );
        say(&reference, "after");
        assert_eq!(cells[winner].events().len(), 51);
    }
}

/// The unstarted reference with a real `ActorCell` as the cell that replaces it (nothing is wired to `actor_of`): twins of oracle
/// cases `MBX-UC-01..03`, compared with the lines of `cases/oracle-only/mailbox.golden` that the Pekko 2.0.0 JVM printed. `MBX-UC-04`
/// (a child created under a suspended parent inherits the suspension) is evidence for slice B: it needs `attach_child` to be asynchronous.
#[cfg(test)]
mod with_real_cells {
    use super::*;
    use crate::actor::cell::ActorCell;
    use crate::actor::props::{Actor, ActorContext, Props};
    use crate::actor::system::ActorSystem;
    use crate::dispatch::sysmsg::Terminated;
    use std::time::{Duration, Instant};

    const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");

    type Log = Arc<Mutex<Vec<String>>>;

    fn recorded(case: &str) -> &'static str {
        ORACLE_GOLDEN
            .lines()
            .find(|line| line.starts_with(case))
            .and_then(|line| line.split_once(" -> "))
            .map(|(_, observed)| observed)
            .expect("the golden has the case")
    }

    fn wait_until(what: &str, condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !condition() {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Logs `r:<text>` for the strings it receives (the oracle's `Worker`).
    struct Worker(Log);
    impl Actor for Worker {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let Some(text) = msg.downcast_ref::<String>() else {
                return false;
            };
            self.0.lock().unwrap().push(format!("r:{text}"));
            true
        }
    }

    enum Command {
        Watch(ActorRef),
        Unwatch(ActorRef),
    }

    /// Watches or unwatches what it is told to, and logs `terminated` for every `Terminated`.
    struct Watcher(Log);
    impl Actor for Watcher {
        fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.is::<Terminated>() {
                self.0.lock().unwrap().push("terminated".to_owned());
            } else if let Some(command) = msg.downcast_ref::<Command>() {
                match command {
                    Command::Watch(target) => ctx.watch(target),
                    Command::Unwatch(target) => ctx.unwatch(target),
                }
            }
            true
        }
    }

    fn joined(log: &Log) -> String {
        log.lock().unwrap().join(", ")
    }

    struct Setup {
        system: ActorSystem,
        reference: Arc<RepointableActorRef>,
        handle: ActorRef,
        path: ActorPath,
        worker_log: Log,
    }

    impl Setup {
        fn new() -> Setup {
            let system = ActorSystem::create("uc").unwrap();
            let path = system.root_path().new_child("user", 0).new_child("c", 7);
            let reference = RepointableActorRef::new(path.clone());
            Setup {
                system,
                handle: ActorRef(reference.clone()),
                reference,
                path,
                worker_log: Log::default(),
            }
        }

        /// The real cell of the actor (`newCell`): created and started, then it replaces the unstarted one (`point`).
        fn start_real_cell(&self) -> Arc<ActorCell> {
            let log = self.worker_log.clone();
            let cell = ActorCell::new_local(
                &self.system.0,
                self.system.0.dispatcher.clone(),
                Props::new(move || Worker(log.clone())),
                self.system.dead_letters(),
                self.path.clone(),
            );
            cell.start();
            assert!(self.reference.replace_with(cell.self_ref().0.clone()));
            cell
        }

        fn watcher(&self, log: &Log) -> ActorRef {
            let log = log.clone();
            self.system
                .actor_of(Props::new(move || Watcher(log.clone())))
                .unwrap()
        }
    }

    #[test]
    fn system_messages_sent_before_the_cell_exists_go_first_and_user_messages_keep_their_order() {
        let setup = Setup::new();
        setup.handle.tell("m1".to_owned(), None);
        setup.handle.tell("m2".to_owned(), None);
        setup.reference.suspend();
        let cell = setup.start_real_cell();
        wait_until("the Suspend to be applied", || {
            let view = cell.mailbox_view();
            view.contains("suspendCount=1") && view.contains("scheduled=false")
        });
        let after_start = format!(
            "{} log=[{}]",
            cell.mailbox_view(),
            joined(&setup.worker_log)
        );
        setup.reference.resume(None);
        wait_until("the messages to be processed", || {
            setup.worker_log.lock().unwrap().len() == 2
                && cell.mailbox_view().contains("scheduled=false")
        });
        let observed = format!(
            "afterStart: {after_start} | afterResume: {} log=[{}]",
            cell.mailbox_view(),
            joined(&setup.worker_log)
        );
        assert_eq!(observed, recorded("MBX-UC-01"));
        setup
            .system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap();
    }

    #[test]
    fn a_watch_and_a_terminate_sent_before_the_cell_exists_are_applied_when_it_starts() {
        let setup = Setup::new();
        let watcher_log = Log::default();
        let watcher = setup.watcher(&watcher_log);
        watcher.tell(Command::Watch(setup.handle.clone()), None);
        wait_until("the Watch to reach the unstarted reference", || {
            setup.reference.queued_system_messages() == 1
        });
        setup.reference.stop();
        setup.start_real_cell();
        wait_until("the watcher to be told", || {
            !watcher_log.lock().unwrap().is_empty()
        });
        wait_until("the actor to terminate", || setup.handle.is_terminated());
        let observed = format!(
            "watcherLog=[{}] childTerminated={}",
            joined(&watcher_log),
            setup.handle.is_terminated()
        );
        assert_eq!(observed, recorded("MBX-UC-02"));
        setup
            .system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap();
    }

    #[test]
    fn a_watch_undone_by_an_unwatch_before_the_cell_exists_tells_the_watcher_nothing() {
        let setup = Setup::new();
        let watcher_log = Log::default();
        let watcher = setup.watcher(&watcher_log);
        watcher.tell(Command::Watch(setup.handle.clone()), None);
        wait_until("the Watch to reach the unstarted reference", || {
            setup.reference.queued_system_messages() == 1
        });
        watcher.tell(Command::Unwatch(setup.handle.clone()), None);
        wait_until("the Unwatch to reach the unstarted reference", || {
            setup.reference.queued_system_messages() == 2
        });
        setup.reference.stop();
        setup.start_real_cell();
        wait_until("the actor to terminate", || setup.handle.is_terminated());
        std::thread::sleep(Duration::from_millis(100)); // a bounded wait for an absence: no Terminated may follow
        let observed = format!(
            "watcherLog=[{}] childTerminated={}",
            joined(&watcher_log),
            setup.handle.is_terminated()
        );
        assert_eq!(observed, recorded("MBX-UC-03"));
        setup
            .system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap();
    }
}
