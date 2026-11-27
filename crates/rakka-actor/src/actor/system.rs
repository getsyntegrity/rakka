// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala and actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala (Apache-2.0).
//! `ActorSystem` (local, Classic): provider, guardians and the termination chain.
//! Follows `ActorSystem.scala` and `LocalActorRefProvider` in `ActorRefProvider.scala`.

use crate::actor::fault::Throwable;
use crate::dispatch::message_queue::UnboundedMailbox;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, Weak};
use std::time::Duration;

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::cell::{ActorCell, LocalActorRef};
use crate::actor::props::{
    Actor, ActorContext, ActorIdentity, AnyMessage, Identify, PoisonPill, Props,
};
use crate::actor::scheduler::{ParkWaiter, Scheduler, TimerHooks, Waiter};
use crate::actor::selection::ActorSelection;
use crate::actor::{ActorError, ErrorClass};
use crate::actor::{ActorPath, Address, Deployer, DeployerError};
use crate::dispatch::dead_letter_mailbox::DeadLetterMailbox;
use crate::dispatch::dispatcher::{Dispatcher, Executor};
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::queue_semantics::MessageQueueSemantics;
use crate::dispatch::sysmsg::{StopChild, SystemMessage, Terminated};
use crate::dispatch::turn::TurnPolicy;
use crate::event::{DeadLetter, EventStream};
use crate::util::clock::{Clock, NanoClock};

/// A write-once value other threads can wait for (`Promise`/`Future`).
pub struct Promise<T> {
    value: Mutex<Option<T>>,
    cv: Condvar,
}

impl<T: Clone> Promise<T> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Promise {
            value: Mutex::new(None),
            cv: Condvar::new(),
        })
    }

    /// `trySuccess`: idempotent.
    fn complete(&self, v: T) {
        let mut g = self.value.lock().unwrap();
        if g.is_none() {
            *g = Some(v);
            self.cv.notify_all();
        }
    }

    pub fn is_completed(&self) -> bool {
        self.value.lock().unwrap().is_some()
    }

    /// Blocks until completed or the timeout elapses.
    pub fn wait(&self, timeout: Duration) -> Option<T> {
        let g = self.value.lock().unwrap();
        let (g, _) = self
            .cv
            .wait_timeout_while(g, timeout, |v| v.is_none())
            .unwrap();
        g.clone()
    }
}

/// `whenTerminated`: a `Future[Terminated]`, so one outcome: `wait` gives `None` on timeout, `Some(Ok(Terminated))` or `Some(Err(cause))`. It can only
/// fail through the walker's `stop()` after a guardian failed (`terminationPromise.completeWith(causeOfTermination.future)`, `ActorRefProvider.scala:467`).
pub type TerminationFuture = Arc<Promise<Result<Terminated, Throwable>>>;

/// The few `pekko.actor.default-dispatcher` settings the port supports (no HOCON `Config` yet).
#[derive(Debug, Clone)]
pub struct Settings {
    /// Number of dispatcher threads (Pekko's default is a fork-join pool with `parallelism-min = 8`).
    pub parallelism: usize,
    /// Messages an actor processes per turn (`pekko.actor.default-dispatcher.throughput`).
    pub throughput: usize,
    /// `pekko.actor.default-dispatcher.throughput-deadline-time`: a turn also ends once this much time has passed since it began.
    /// Zero (the default) or less than a millisecond means no deadline.
    pub throughput_deadline_time: Duration,
    /// `pekko.actor.unstarted-push-timeout`: how long a send to an actor that is being started waits for the reference's lock before
    /// the message becomes a dead letter. Zero does not wait (the JVM treats a negative timeout like zero; `Duration` cannot be negative).
    pub unstarted_push_timeout: Duration,
    /// `pekko.actor.default-dispatcher.mailbox-requirement`: the semantics the default dispatcher requires of the queue of every actor
    /// that runs on it (all of them until `Props` can select a dispatcher). `None` is the empty string: no requirement.
    pub mailbox_requirement: Option<MessageQueueSemantics>,
    /// The mailbox ids that `Props::with_mailbox_id` can select (`Mailboxes`).
    pub mailboxes: Mailboxes,
    /// The deployments (`pekko.actor.deployment`, or added with [`Deployer::deploy`]) that `actorOf` looks up by the path of the new actor.
    /// Clones of the settings share this one deployer, as one system shares its own.
    pub deployer: Arc<Deployer>,
}

impl Settings {
    /// The runtime settings that the configuration decides: `pekko.actor.unstarted-push-timeout` and the deployments of
    /// `pekko.actor.deployment` (an invalid one is an error, as at the start of a Pekko system). The dispatcher's keys
    /// (`parallelism`, `throughput`, `throughput-deadline-time`) keep their defaults until the system is created from a
    /// configuration. A negative timeout is zero, as the JVM's `tryLock` treats it.
    pub fn from_config(config: &crate::config::Settings) -> Result<Settings, DeployerError> {
        let unstarted_push_timeout = u64::try_from(config.unstarted_push_timeout.as_nanos())
            .map_or(Duration::ZERO, Duration::from_nanos);
        Ok(Settings {
            unstarted_push_timeout,
            deployer: Arc::new(Deployer::from_settings(config)?),
            ..Settings::default()
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            parallelism: 8,
            throughput: 5,
            throughput_deadline_time: Duration::ZERO,
            unstarted_push_timeout: Duration::from_secs(10),
            mailbox_requirement: None,
            mailboxes: Mailboxes::new(),
            deployer: Arc::new(Deployer::new()),
        }
    }
}

pub(crate) struct SystemInner {
    pub name: String,
    pub settings: Settings,
    pub events: Arc<EventStream>,
    pub dispatcher: Arc<Dispatcher>,
    scheduler: Arc<Scheduler>,
    root_path: ActorPath,
    root_guardian: OnceLock<Arc<ActorCell>>,
    pub(crate) guardian: OnceLock<Arc<ActorCell>>,
    pub(crate) system_guardian: OnceLock<Arc<ActorCell>>,
    pub dead_letters: ActorRef,
    /// `Mailboxes.deadLetterMailbox`: where a closed mailbox sends its user and system messages.
    pub dead_letter_mailbox: DeadLetterMailbox,
    termination: TerminationFuture,
    terminating: AtomicBool,
    /// The bubble walker, for the tests that look at its state.
    #[cfg(test)]
    pub(crate) walker: OnceLock<Arc<BubbleWalker>>,
}

impl SystemInner {
    pub(crate) fn root_guardian(&self) -> ActorRef {
        self.root_guardian.get().expect("initialised").self_ref()
    }
}

/// Messages marked `DeadLetterSuppression` in Pekko (`Terminated`, `PoisonPill`): never published as `DeadLetter`.
pub(crate) fn is_dead_letter_suppressed(msg: &AnyMessage) -> bool {
    msg.is::<Terminated>() || msg.is::<PoisonPill>()
}

/// `ActorIdentity(messageId, None)`: what `deadLetters` answers to the `Identify` of an actor that is not there.
fn tell_no_such_actor(identify: &Identify, asker: &ActorRef, reply_sender: Option<ActorRef>) {
    let identity = ActorIdentity {
        correlation_id: identify.message_id.clone(),
        actor_ref: None,
    };
    asker.tell_any(Arc::new(identity), reply_sender);
}

/// A local Classic actor system.
#[derive(Clone)]
pub struct ActorSystem(pub(crate) Arc<SystemInner>);

// ---- the three special references of the provider ----

/// `deadLetters`: every message sent to it (and to dead actors) is published as a `DeadLetter`.
/// It answers `Watch` as "already dead".
struct DeadLetters {
    path: ActorPath,
    events: Arc<EventStream>,
    me: Weak<DeadLetters>,
}

impl DeadLetters {
    /// `DeadLetterActorRef.specialHandle` for system messages: true when `deadLetters` dealt with the message itself.
    /// A `Watch` is answered with a `DeathWatchNotification` (unless the watchee or the watcher is `deadLetters`), an `Unwatch` is
    /// ignored, and `Terminate` and `DeathWatchNotification` (`DeadLetterSuppression`) are never published as dead letters.
    fn special_handle(&self, message: &SystemMessage, me: &ActorRef) -> bool {
        match message {
            SystemMessage::Watch { watchee, watcher } => {
                if watchee != me && watcher != me {
                    watcher.send_system(SystemMessage::DeathWatchNotification {
                        actor: watchee.clone(),
                        existence_confirmed: false,
                        address_terminated: false,
                    });
                }
                true
            }
            SystemMessage::Unwatch { .. }
            | SystemMessage::Terminate
            | SystemMessage::DeathWatchNotification { .. } => true,
            _ => false,
        }
    }
}

impl InternalActorRef for DeadLetters {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        let me = ActorRef(self.me.upgrade().expect("alive"));
        if is_dead_letter_suppressed(&msg) {
            return;
        }
        if let Some(d) = msg.downcast_ref::<DeadLetter>() {
            // `DeadLetterActorRef.!`: `if (!specialHandle(d.message, d.sender)) eventStream.publish(d)`
            let handled = if let Some(system_message) = d.message.downcast_ref::<SystemMessage>() {
                self.special_handle(system_message, &me)
            } else if let Some(identify) = d.message.downcast_ref::<Identify>() {
                tell_no_such_actor(identify, &d.sender, None);
                true
            } else {
                is_dead_letter_suppressed(&d.message) // Pekko publishes a `SuppressedDeadLetter`; not modelled yet
            };
            if !handled {
                self.events.publish_dead_letter(d.clone());
            }
        } else if let (Some(identify), Some(asker)) = (msg.downcast_ref::<Identify>(), &sender) {
            // told to `deadLetters` itself, the reply carries the asker as its sender. Without a sender Pekko throws a
            // NullPointerException (Q-REF-7, open): the message is published like any other, as before.
            tell_no_such_actor(identify, asker, Some(asker.clone()));
        } else {
            self.events.publish_dead_letter(DeadLetter {
                message: msg,
                sender: sender.unwrap_or_else(|| me.clone()),
                recipient: me,
            });
        }
    }
    fn send_system(&self, msg: SystemMessage) {
        // `EmptyLocalActorRef.sendSystemMessage`: `specialHandle(message, deadLetters)`; what it does not handle is ignored
        let me = ActorRef(self.me.upgrade().expect("alive"));
        self.special_handle(&msg, &me);
    }
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        true // `EmptyLocalActorRef.isTerminated`
    }
}

/// `theOneWhoWalksTheBubblesOfSpaceTime`: the root guardian's supervisor; its death ends the system.
pub(crate) struct BubbleWalker {
    path: ActorPath,
    /// The provider's `terminationPromise`, which `whenTerminated` exposes: it follows `cause_of_termination` only in `stop()`.
    pub(crate) termination: TerminationFuture,
    /// `causeOfTermination`: the walker's own promise. It completes with the cause when a guardian fails, or with `Terminated` in `stop()`;
    /// while it is pending the walker is walking (`isWalking`) and handles messages.
    pub(crate) cause_of_termination: Arc<Promise<Result<Terminated, Throwable>>>,
    dispatchers: Vec<Arc<Dispatcher>>,
    /// Closed before `whenTerminated` completes (`registerOnTermination(stopScheduler())`).
    scheduler: OnceLock<Arc<Scheduler>>,
    pub(crate) root: OnceLock<ActorRef>,
    /// Completes when the walker discards, after it stopped walking, the `DeathWatchNotification` of the root guardian (and of no other actor).
    #[cfg(test)]
    pub(crate) notification_discarded: Arc<Promise<()>>,
}

impl BubbleWalker {
    pub(crate) fn new(
        path: ActorPath,
        termination: TerminationFuture,
        dispatchers: Vec<Arc<Dispatcher>>,
    ) -> Self {
        BubbleWalker {
            path,
            termination,
            cause_of_termination: Promise::new(),
            dispatchers,
            scheduler: OnceLock::new(),
            root: OnceLock::new(),
            #[cfg(test)]
            notification_discarded: Promise::new(),
        }
    }

    /// `isWalking`.
    fn is_walking(&self) -> bool {
        !self.cause_of_termination.is_completed()
    }

    #[cfg(test)]
    pub(crate) fn dispatchers_are_shut_down(&self) -> bool {
        self.dispatchers.iter().all(|d| d.is_shut_down())
    }

    #[cfg(test)]
    pub(crate) fn shut_down_dispatchers(&self) {
        for d in &self.dispatchers {
            d.shutdown();
        }
    }
}

impl InternalActorRef for BubbleWalker {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, _msg: AnyMessage, _sender: Option<ActorRef>) {}
    fn send_system(&self, msg: SystemMessage) {
        // `sendSystemMessage` only while `isWalking` (`ActorRefProvider.scala:483`)
        if !self.is_walking() {
            #[cfg(test)]
            if matches!(&msg, SystemMessage::DeathWatchNotification { actor, .. } if self.root.get() == Some(actor))
            {
                self.notification_discarded.complete(());
            }
            return;
        }
        match msg {
            SystemMessage::DeathWatchNotification { .. } => self.stop(),
            // `causeOfTermination.tryFailure(ex)` then `child.stop()`: the walker stops walking, so the death of the root is discarded above, `stop()`
            // never runs and the public `terminationPromise` stays pending (the cause is not read by anything else)
            SystemMessage::Failed { child, cause } => {
                self.cause_of_termination.complete(Err(cause));
                child.stop();
            }
            _ => {}
        }
    }
    fn stop(&self) {
        if let Some(root) = self.root.get() {
            // `causeOfTermination.trySuccess(Terminated(root))` (a no-op once a guardian failed), then
            // `terminationPromise.completeWith(causeOfTermination.future)`: the public future gets whatever the internal one holds, success or failure
            self.cause_of_termination.complete(Ok(Terminated {
                actor: root.clone(),
                existence_confirmed: true,
                address_terminated: true,
            }));
            if let Some(scheduler) = self.scheduler.get() {
                scheduler.close();
            }
            if let Some(outcome) = self.cause_of_termination.wait(Duration::ZERO) {
                self.termination.complete(outcome);
            }
        }
        for d in &self.dispatchers {
            d.shutdown();
        }
    }
    fn is_terminated(&self) -> bool {
        !self.is_walking()
    }
}

// ---- guardian actors ----

/// `LocalActorRefProvider.Guardian` (root and `/user`).
struct Guardian;

impl Actor for Guardian {
    fn required_message_queue() -> Option<MessageQueueSemantics> {
        Some(MessageQueueSemantics::UNBOUNDED)
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.is::<Terminated>() {
            let me = ctx.self_ref();
            ctx.stop(&me);
            true
        } else if let Some(stop) = msg.downcast_ref::<StopChild>() {
            ctx.stop(&stop.0);
            true
        } else {
            false
        }
    }
}

/// `LocalActorRefProvider.SystemGuardian`. Termination hooks are not supported yet.
struct SystemGuardian {
    guardian: ActorRef,
}

impl Actor for SystemGuardian {
    fn required_message_queue() -> Option<MessageQueueSemantics> {
        Some(MessageQueueSemantics::UNBOUNDED)
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(t) = msg.downcast_ref::<Terminated>() {
            if t.actor == self.guardian {
                // no termination hooks registered: time for the system guardian to stop
                let me = ctx.self_ref();
                ctx.stop(&me);
            }
            true
        } else if let Some(stop) = msg.downcast_ref::<StopChild>() {
            ctx.stop(&stop.0);
            true
        } else {
            false
        }
    }
}

impl ActorSystem {
    /// `ActorSystem.create(name)` with default settings.
    pub fn create(name: &str) -> Result<ActorSystem, ActorError> {
        Self::create_with_settings(name, Settings::default())
    }

    /// `ActorSystem.create(name, config)`, with the supported settings given directly.
    pub fn create_with_settings(name: &str, settings: Settings) -> Result<ActorSystem, ActorError> {
        Self::create_with_clock(name, settings, Arc::new(NanoClock))
    }

    /// Like [`ActorSystem::create_with_settings`] with the clock injected: it measures the throughput deadline and drives the scheduler.
    pub(crate) fn create_with_clock(
        name: &str,
        settings: Settings,
        clock: Arc<dyn Clock>,
    ) -> Result<ActorSystem, ActorError> {
        Self::create_with(name, settings, clock, Arc::new(ParkWaiter), None)
    }

    /// Like [`ActorSystem::create_with_clock`] with the executor of the default dispatcher replaced (the internal dispatcher
    /// that runs the guardians keeps its pool): the seam that lets tests make the executor reject tasks.
    #[cfg(test)]
    pub(crate) fn create_with_executor(
        name: &str,
        settings: Settings,
        clock: Arc<dyn Clock>,
        executor: Arc<dyn Executor>,
    ) -> Result<ActorSystem, ActorError> {
        Self::create_with(name, settings, clock, Arc::new(ParkWaiter), Some(executor))
    }

    /// Like [`ActorSystem::create_with_clock`] with the `Waiter` of the scheduler injected too, so that tests move all the time.
    #[cfg(test)]
    pub(crate) fn create_with_time(
        name: &str,
        settings: Settings,
        clock: Arc<dyn Clock>,
        waiter: Arc<dyn Waiter>,
    ) -> Result<ActorSystem, ActorError> {
        Self::create_with(name, settings, clock, waiter, None)
    }

    fn create_with(
        name: &str,
        settings: Settings,
        clock: Arc<dyn Clock>,
        waiter: Arc<dyn Waiter>,
        executor: Option<Arc<dyn Executor>>,
    ) -> Result<ActorSystem, ActorError> {
        let valid = {
            let mut chars = name.chars();
            matches!(chars.next(), Some(c) if c.is_ascii_alphanumeric())
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        };
        if !valid {
            return Err(ActorError::new(
                ErrorClass::IllegalArgument,
                format!(
                    "invalid ActorSystem name [{name}], must contain only word characters (i.e. [a-zA-Z0-9] plus non-leading '-' or '_')"
                ),
            ));
        }
        let root_path = ActorPath::root(Address::local("pekko", name));
        let turn_policy = TurnPolicy::new(settings.throughput, settings.throughput_deadline_time);
        let dispatcher = match executor {
            Some(executor) => Dispatcher::with_executor(executor, turn_policy, clock.clone()),
            None => Dispatcher::start_with(name, settings.parallelism, turn_policy, clock.clone()),
        };
        let events = EventStream::new();
        let internal_dispatcher =
            Dispatcher::start_with(&format!("{name}-internal"), 4, turn_policy, clock.clone());
        let scheduler = Arc::new(Scheduler::start(
            name,
            dispatcher.clone(),
            TimerHooks::new(clock, waiter),
        ));
        let termination = Promise::new();
        let dead_letters = ActorRef(Arc::new_cyclic(|me| DeadLetters {
            path: root_path.new_child("deadLetters", 0),
            events: events.clone(),
            me: me.clone(),
        }));
        let sys = Arc::new(SystemInner {
            name: name.to_string(),
            settings,
            events,
            dispatcher: dispatcher.clone(),
            scheduler: scheduler.clone(),
            root_path: root_path.clone(),
            root_guardian: OnceLock::new(),
            guardian: OnceLock::new(),
            system_guardian: OnceLock::new(),
            dead_letter_mailbox: DeadLetterMailbox::new(dead_letters.clone()),
            dead_letters,
            termination: termination.clone(),
            terminating: AtomicBool::new(false),
            #[cfg(test)]
            walker: OnceLock::new(),
        });

        // `defaultMailbox = mailboxes.lookup(DefaultMailboxId)`: the guardians take it directly, not through `getMailboxType`
        let default_mailbox = sys
            .settings
            .mailboxes
            .lookup(Mailboxes::DEFAULT_MAILBOX_ID)
            .unwrap_or_else(|_| Arc::new(UnboundedMailbox));
        // rootGuardian, supervised by the bubble walker
        let bubble = Arc::new(BubbleWalker::new(
            root_path.new_child("bubble-walker", 0),
            termination,
            vec![dispatcher, internal_dispatcher.clone()],
        ));
        bubble.scheduler.set(scheduler).ok();
        #[cfg(test)]
        sys.walker.set(bubble.clone()).ok();
        let walker = ActorRef(bubble.clone());
        let root = ActorCell::new_local(
            &sys,
            internal_dispatcher.clone(),
            Props::new(|| Guardian).with_resolved_mailbox(default_mailbox.clone()),
            walker,
            root_path.clone(),
        );
        bubble.root.set(root.self_ref()).ok();
        sys.root_guardian.set(root.clone()).ok();

        // /user and /system are children of the root guardian that bypass actorOf (reserveChild + initChild)
        let make_guardian = |name: &str, props: Props| {
            root.reserve_child(name);
            let cell = ActorCell::new_local(
                &sys,
                internal_dispatcher.clone(),
                props,
                root.self_ref(),
                root_path.new_child(name, 0),
            );
            root.init_child(name, cell.self_ref());
            cell
        };
        let user = make_guardian(
            "user",
            Props::new(|| Guardian).with_resolved_mailbox(default_mailbox.clone()),
        );
        sys.guardian.set(user.clone()).ok();
        let user_ref = user.self_ref();
        let system_guardian = make_guardian(
            "system",
            Props::new(move || SystemGuardian {
                guardian: user_ref.clone(),
            })
            .with_resolved_mailbox(default_mailbox.clone()),
        );
        sys.system_guardian.set(system_guardian.clone()).ok();

        // provider.init: start the root, then chain death watchers so that killing the guardian stops everything
        // the guardians are attached synchronously by the provider; their behaviour is unchanged (no evidence yet)
        let _ = root.start();
        let _ = user.start();
        let _ = system_guardian.start();
        let (sg, ug, rg) = (system_guardian.self_ref(), user.self_ref(), root.self_ref());
        sg.send_system(SystemMessage::Watch {
            watchee: ug,
            watcher: sg.clone(),
        });
        rg.send_system(SystemMessage::Watch {
            watchee: sg,
            watcher: rg.clone(),
        });
        Ok(ActorSystem(sys))
    }

    /// `ActorSystem.name`.
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// `provider.rootPath`.
    pub fn root_path(&self) -> &ActorPath {
        &self.0.root_path
    }

    /// The `/user` guardian.
    pub fn guardian(&self) -> ActorRef {
        self.0.guardian.get().expect("initialised").self_ref()
    }

    /// The `/system` guardian.
    pub fn system_guardian(&self) -> ActorRef {
        self.0
            .system_guardian
            .get()
            .expect("initialised")
            .self_ref()
    }

    /// `system.actorSelection(path)`: a relative path starts at the root guardian.
    pub fn actor_selection(&self, path: &str) -> ActorSelection {
        ActorSelection::new(&self.0, &self.0.root_guardian(), path)
    }

    pub fn dead_letters(&self) -> ActorRef {
        self.0.dead_letters.clone()
    }

    /// `system.eventStream`.
    pub fn event_stream(&self) -> &EventStream {
        &self.0.events
    }

    pub fn settings(&self) -> &Settings {
        &self.0.settings
    }

    /// `system.scheduler`.
    pub fn scheduler(&self) -> &Scheduler {
        &self.0.scheduler
    }

    /// `system.actorOf(props)`: a top-level actor with a generated name (`$a`, `$b`, ...).
    pub fn actor_of(&self, props: Props) -> Result<ActorRef, ActorError> {
        self.0
            .guardian
            .get()
            .expect("initialised")
            .attach_child(props, None, true)
    }

    /// `system.actorOf(props, name)`.
    pub fn actor_of_named(&self, props: Props, name: &str) -> Result<ActorRef, ActorError> {
        self.0
            .guardian
            .get()
            .expect("initialised")
            .attach_child(props, Some(name), true)
    }

    /// `system.stop(actor)`: top-level actors are stopped by their guardian, others directly.
    pub fn stop(&self, actor: &ActorRef) {
        let parent = actor.path().parent();
        let guardian = self.guardian();
        let sys_guardian = self.system_guardian();
        if parent == *guardian.path() {
            guardian.tell_any(Arc::new(StopChild(actor.clone())), None);
        } else if parent == *sys_guardian.path() {
            sys_guardian.tell_any(Arc::new(StopChild(actor.clone())), None);
        } else {
            actor.stop();
        }
    }

    /// `system.terminate()`.
    pub fn terminate(&self) -> TerminationFuture {
        self.0.terminating.store(true, Ordering::SeqCst);
        // finalTerminate: stop the /user guardian; the rest follows through death watch
        self.0
            .guardian
            .get()
            .expect("initialised")
            .self_ref()
            .stop();
        self.when_terminated()
    }

    /// `system.whenTerminated`.
    pub fn when_terminated(&self) -> TerminationFuture {
        self.0.termination.clone()
    }
}

impl std::fmt::Display for ActorSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.root_path.address())
    }
}

#[allow(dead_code)]
fn _assert_send_sync() {
    fn is<T: Send + Sync>() {}
    is::<ActorSystem>();
    let _ = LocalActorRef::is_terminated;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Noop;
    impl Actor for Noop {}

    const WAIT: Duration = Duration::from_secs(10);

    #[test]
    fn invalid_system_names_are_rejected() {
        for bad in ["", "-a", "_a", "a b", "a.b", "é"] {
            let e = ActorSystem::create(bad).err().expect(bad);
            assert_eq!(e.class(), ErrorClass::IllegalArgument);
        }
        assert!(ActorSystem::create("Sys-1_x").is_ok());
    }

    #[test]
    fn spawn_names_and_duplicates() {
        let sys = ActorSystem::create("t1").unwrap();
        let a = sys.actor_of_named(Props::new(|| Noop), "a").unwrap();
        assert_eq!(a.path().to_string(), "pekko://t1/user/a");
        assert_eq!(
            sys.actor_of(Props::new(|| Noop)).unwrap().path().name(),
            "$a"
        );
        let dup = sys.actor_of_named(Props::new(|| Noop), "a").unwrap_err();
        assert_eq!(dup.class(), ErrorClass::InvalidActorName);
        sys.terminate()
            .wait(WAIT)
            .expect("ended")
            .expect("terminated normally");
    }

    #[test]
    fn stop_and_terminate_complete_the_chain() {
        let sys = ActorSystem::create("t2").unwrap();
        let a = sys.actor_of_named(Props::new(|| Noop), "a").unwrap();
        sys.stop(&a);
        let end = std::time::Instant::now() + WAIT;
        while !a.is_terminated() && std::time::Instant::now() < end {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(a.is_terminated());
        let t = sys
            .terminate()
            .wait(WAIT)
            .expect("ended")
            .expect("terminated normally");
        assert!(t.existence_confirmed && t.address_terminated);
        let late = sys.actor_of_named(Props::new(|| Noop), "late").unwrap_err();
        assert_eq!(late.class(), ErrorClass::IllegalState);
    }

    #[test]
    fn terminating_an_empty_system_closes_the_guardian_to_new_children() {
        // regression: `setTerminated()` must run when there are no children to wait for
        let sys = ActorSystem::create("t3").unwrap();
        sys.terminate()
            .wait(WAIT)
            .expect("ended")
            .expect("terminated normally");
        assert_eq!(
            sys.actor_of(Props::new(|| Noop)).unwrap_err().class(),
            ErrorClass::IllegalState
        );
    }
}
