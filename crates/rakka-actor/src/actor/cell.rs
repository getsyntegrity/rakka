// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala (Apache-2.0).
//! `ActorCell`: the engine behind a local actor — mailbox, system-message processing, children,
//! death watch and the termination sequence. Follows `ActorCell.scala` and `dungeon/*`.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError, Weak};

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::children::{Children, SuspendReason};
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{
    Actor, ActorContext, ActorIdentity, AnyMessage, Identify, Kill, PoisonPill, Props,
};
use crate::actor::repointable::{PushPolicy, RepointableActorRef};
use crate::actor::supervision::{Directive, RestartStats, SupervisorStrategy};
use crate::actor::system::SystemInner;
use crate::actor::validate_path_element;
use crate::actor::watching::{CustomTermination, check_watching_same};
use crate::actor::{ActorError, ConfigurationError, ErrorClass};
use crate::dispatch::dispatcher::Dispatcher;
use crate::dispatch::dispatcher::Rejected;
use crate::dispatch::mailbox_close::enqueue_unless_closed;
use crate::dispatch::mailbox_status::MailboxStatus;
use crate::dispatch::message_queue::{Envelope, MessageQueue, UnboundedMailbox};
use crate::dispatch::sysmsg::{DeliveryState, Failed, SystemMessage, Terminated};
use crate::dispatch::system_queue::SystemQueue;
use crate::event::UnhandledMessage;
use crate::event::{ErrorEvent, EventStream, WarningEvent, report_dispatch_failure};
use crate::util::helpers::base64_append;

#[derive(Default)]
struct CellState {
    actor: Option<Box<dyn Actor>>,
    /// `clazz(actor)` of the current instance, kept while the instance is taken out to run a callback; `None` without an instance
    actor_class: Option<&'static str>,
    /// `watching`: the watched actors, with the message of a `watch_with` (`None`: a `Terminated` is delivered)
    watching: Vec<(ActorRef, Option<CustomTermination>)>,
    watched_by: Vec<ActorRef>,
    /// `terminatedQueued`: a `Terminated` that was put in our mailbox and not delivered yet, with what is delivered in its place;
    /// `unwatch` cancels it.
    terminated_queued: HashMap<ActorRef, Option<CustomTermination>>,
    /// `_failed`
    failed: FailedInfo,
    /// `currentMessage`: what `preRestart` is told and `context.sender()` reads. Only a successful user message, a restart and the end
    /// of the instance forget it: a failed one is kept across `Resume`, and a child's `Failed` stays after it was handled.
    current: Option<Envelope>,
    /// the instance a restart replaced: Pekko keeps it in `_actor` until the factory has produced the new one
    replaced_actor: Option<Box<dyn Actor>>,
    /// the cause of a `Recreate` that waits for the children to die (`Recreation(cause)`)
    recreate_cause: Option<Throwable>,
    /// `ChildRestartStats` of the children this actor supervises, by child
    restart_stats: std::collections::HashMap<ActorRef, RestartStats>,
    /// `sysmsgStash`: system messages deferred while the cell is suspended or waits for its children to die, oldest first
    deferred: Vec<SystemMessage>,
}

/// `FaultHandling.FailedInfo`: what a cell remembers of its own failure.
#[derive(Default)]
pub(crate) enum FailedInfo {
    #[default]
    NotFailed,
    /// `FailedRef(perpetrator)`: a failure waits for its decision, or the actor is terminating. The perpetrator is the child whose failure
    /// was escalated, the one child that gets the cause back on `Resume`; `None` stands for `self`.
    FailedRef(Option<ActorRef>),
    /// a restart failed. Never cleared: no later restart calls `preRestart`, and `Resume` with a cause restarts
    FailedFatally,
}

impl FailedInfo {
    pub(crate) fn is_failed(&self) -> bool {
        matches!(self, Self::FailedRef(_))
    }
    fn perpetrator(&self) -> Option<ActorRef> {
        match self {
            Self::FailedRef(perpetrator) => perpetrator.clone(),
            _ => None,
        }
    }
    pub(crate) fn set_failed(&mut self, perpetrator: Option<ActorRef>) {
        if !matches!(self, Self::FailedFatally) {
            *self = Self::FailedRef(perpetrator);
        }
    }
    pub(crate) fn clear_failed(&mut self) {
        if self.is_failed() {
            *self = Self::NotFailed;
        }
    }
}

/// What a callback threw: a [`Throwable`] payload, or an ordinary panic reported as a `RuntimeException`.
fn throwable_of(payload: Box<dyn std::any::Any + Send>) -> Throwable {
    match payload.downcast::<Throwable>() {
        Ok(t) => *t,
        Err(p) => {
            let msg = p
                .downcast_ref::<&str>()
                .map(|m| m.to_string())
                .or_else(|| p.downcast_ref::<String>().cloned());
            drop_guarded(p);
            Throwable::new("java.lang.RuntimeException", msg.as_deref())
        }
    }
}

/// Drops a value of the user (an instance, the current message, a strategy, a panic payload): a panic of its `Drop` stops here, once the
/// panic hook has reported it, so it cannot cut a transition of the cell short. Never call it with a lock of the cell held.
fn drop_guarded<T>(value: T) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(value)));
}

pub(crate) struct ActorCell {
    pub(crate) path: ActorPath,
    pub(crate) system: Arc<SystemInner>,
    parent: ActorRef,
    props: Props,
    /// the dispatcher this actor's mailbox runs on (`internal-dispatcher` for guardians)
    dispatcher: Arc<Dispatcher>,
    me: Weak<ActorCell>,
    /// what the actor knows as `self` when it is not the cell's own reference: the `RepointableActorRef` of an actor made by `actorOf`
    outer: Mutex<Option<ActorRef>>,
    /// the atomic mailbox state (Open/Closed, Scheduled, suspend count)
    status: MailboxStatus,
    system_queue: SystemQueue<SystemMessage>,
    /// the user messages (`Mailbox.messageQueue`): `UnboundedMailbox` unless `Props::with_mailbox` chose another queue
    queue: Box<dyn MessageQueue>,
    children: Mutex<Children>,
    name_counter: AtomicU64,
    st: Mutex<CellState>,
}

/// The class of the dispatcher that reports a rejected registration (`getClass.getName`).
const DISPATCHER_CLASS: &str = "org.apache.pekko.dispatch.Dispatcher";
/// `classOf[Actor]`, what `clazz(actor)` answers while the cell has no instance (before `Create`, after termination).
const ACTOR_CLASS: &str = "org.apache.pekko.actor.Actor";

/// `clazz(actor)`: the class of the cell's instance, or `Actor` when there is none.
fn reporter_class(instance_class: Option<&'static str>) -> String {
    instance_class.unwrap_or(ACTOR_CLASS).to_owned()
}

/// `getClass.getName` of a message, for `PreRestartException`: strings and the messages below; any other type is `?` (Q-CELL-6).
fn message_class(message: &AnyMessage) -> &'static str {
    match () {
        _ if message.is::<&str>() || message.is::<String>() => "java.lang.String",
        _ if message.is::<Kill>() => "org.apache.pekko.actor.Kill$",
        _ if message.is::<ActorIdentity>() => "org.apache.pekko.actor.ActorIdentity",
        _ if message.is::<Terminated>() => "org.apache.pekko.actor.Terminated",
        _ if message.is::<Failed>() => "org.apache.pekko.dispatch.sysmsg.Failed",
        _ => "?",
    }
}

/// `ActorInitializationException` and its two subclasses.
const INITIALIZATION_FAILURES: [&str; 3] = [
    "org.apache.pekko.actor.ActorInitializationException",
    "org.apache.pekko.actor.PreRestartException",
    "org.apache.pekko.actor.PostRestartException",
];

/// The text `logFailure` publishes: `getMessage` of the failure, or of its cause when it is an `ActorInitializationException`
/// that has one (looking through the wrapper of a reflective call).
fn logged_message(failure: &Throwable) -> Option<String> {
    let cause = match &failure.cause {
        Some(cause) if INITIALIZATION_FAILURES.contains(&failure.class.as_str()) => cause,
        _ => return failure.message.clone(),
    };
    match &cause.cause {
        Some(target) if cause.class == "java.lang.reflect.InvocationTargetException" => {
            target.message.clone()
        }
        _ => cause.message.clone(),
    }
}

/// `new RejectedExecutionException`.
fn rejected_execution() -> Throwable {
    Throwable::new("java.util.concurrent.RejectedExecutionException", None)
}

/// `LocalActorRef`.
pub(crate) struct LocalActorRef(pub Arc<ActorCell>);

impl InternalActorRef for LocalActorRef {
    fn path(&self) -> &ActorPath {
        &self.0.path
    }
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        self.0.enqueue_user(Envelope {
            message: msg,
            sender,
        });
    }
    fn send_system(&self, msg: SystemMessage) {
        self.0.system_dispatch(msg);
    }
    fn stop(&self) {
        self.0.system_dispatch(SystemMessage::Terminate);
    }
    fn parent(&self) -> Option<ActorRef> {
        Some(self.0.parent.clone())
    }
    fn single_child(&self, name: &str) -> Option<ActorRef> {
        self.0.child(name)
    }
    fn children(&self) -> Vec<ActorRef> {
        self.0.children()
    }
    #[cfg(test)]
    fn attach_child_for_tests(&self, props: Props, name: &str) -> Result<ActorRef, ActorError> {
        self.0.attach_child(props, Some(name), true)
    }
    #[cfg(test)]
    fn mailbox_view(&self) -> String {
        self.0.mailbox_view()
    }
    #[cfg(test)]
    fn has_system_messages(&self) -> bool {
        self.0.has_system_messages()
    }
    #[cfg(test)]
    fn is_failed(&self) -> bool {
        self.0.st.lock().unwrap().failed.is_failed()
    }
    fn is_terminated(&self) -> bool {
        self.0.status.is_closed()
    }
}

/// `ActorCell.newUid()`: a random non-zero uid (splitmix64 over a counter, seeded from the clock).
pub(crate) fn new_uid() -> i32 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    static SEED: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    let seed = *SEED.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678_9ABC_DEF0)
    });
    loop {
        let mut z = COUNTER
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed)
            .wrapping_add(seed);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        if z as i32 != 0 {
            return z as i32;
        }
    }
}

/// `Dispatchers.DefaultDispatcherId`: the dispatcher every actor runs on until `Props` can select another one.
pub(crate) const DEFAULT_DISPATCHER_ID: &str = "pekko.actor.default-dispatcher";

/// How the real cell of an asynchronously created actor is made (`newCell`): the system, the actor's props, its parent, its path and the
/// reference it knows as its own. `Err` is the exception Pekko's constructor or `init` would throw.
pub(crate) type CellMaker = fn(
    &Arc<SystemInner>,
    Props,
    ActorRef,
    ActorPath,
    &ActorRef,
) -> Result<Arc<ActorCell>, Throwable>;

impl ActorCell {
    /// Creates the cell and its `LocalActorRef` (`ActorCell.init`): `Create` is queued first,
    /// then the parent is told to supervise. Nothing runs until [`ActorCell::start`].
    pub(crate) fn new_local(
        system: &Arc<SystemInner>,
        dispatcher: Arc<Dispatcher>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
    ) -> Arc<ActorCell> {
        Self::build(system, dispatcher, props, parent, path, None, None)
    }

    /// `newCell` of a `RepointableActorRef`: `init(sendSupervise = false)` (the supervisor was told when the reference was made) and
    /// `outer` is the reference the actor knows as its own.
    pub(crate) fn new_repointed(
        system: &Arc<SystemInner>,
        dispatcher: Arc<Dispatcher>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
        outer: ActorRef,
    ) -> Arc<ActorCell> {
        Self::build(system, dispatcher, props, parent, path, Some(outer), None)
    }

    /// `initWithFailure(failure)`: the replacement for a cell that could not be made. It runs on the system's default dispatcher with
    /// an unbounded mailbox, and its only message is `Create` carrying `ActorInitializationException(self, "failure while creating
    /// ActorCell", failure)`, whose cause is `failure`; the supervisor was already told about the actor when the reference was made.
    pub(crate) fn new_with_failure(
        system: &Arc<SystemInner>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
        outer: ActorRef,
        failure: Throwable,
    ) -> Arc<ActorCell> {
        let initialization_failure = Throwable::new(
            "org.apache.pekko.actor.ActorInitializationException",
            Some(&format!("{path}: failure while creating ActorCell")),
        )
        .with_cause(failure);
        Self::build(
            system,
            system.dispatcher.clone(),
            props,
            parent,
            path,
            Some(outer),
            Some(initialization_failure),
        )
    }

    fn build(
        system: &Arc<SystemInner>,
        dispatcher: Arc<Dispatcher>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
        outer: Option<ActorRef>,
        creation_failure: Option<Throwable>,
    ) -> Arc<ActorCell> {
        // the replacement of a cell that could not be made always has the default unbounded mailbox
        let queue = if creation_failure.is_some() {
            UnboundedMailbox.create()
        } else {
            props.new_message_queue(&system.dead_letters)
        };
        let cell = Arc::new_cyclic(|me| ActorCell {
            path,
            system: system.clone(),
            parent: parent.clone(),
            props,
            dispatcher,
            me: me.clone(),
            outer: Mutex::new(outer.clone()),
            status: {
                let status = MailboxStatus::new();
                status.set_as_scheduled(); // not attached yet: nothing runs before `start`
                status
            },
            system_queue: SystemQueue::new(),
            queue,
            children: Mutex::new(Children::Empty),
            name_counter: AtomicU64::new(0),
            st: Mutex::new(CellState::default()),
        });
        // `Dispatch.init`: what the type promised is checked against the queue it really created; the failure is delayed into `Create`
        let creation_failure = creation_failure.or_else(|| {
            let (required, got) = cell.props.unmet_queue_requirement(&*cell.queue)?;
            let me = cell.self_ref();
            Some(Throwable::new(
                "org.apache.pekko.actor.ActorInitializationException",
                Some(&format!(
                    "{}: Actor [{me:?}] requires mailbox type [{required}] got [{got}]",
                    cell.path
                )),
            ))
        });
        assert!(
            cell.system_queue
                .enqueue(SystemMessage::Create(creation_failure))
                .is_ok(),
            "a new mailbox accepts system messages"
        );
        if outer.is_none() {
            parent.send_system(SystemMessage::Supervise {
                child: cell.self_ref(),
                is_async: false,
            });
        }
        cell
    }

    /// `ActorCell.start()`: attach to the dispatcher. `Dispatcher.attach` lets the executor's rejection propagate to the caller
    /// (after `register_for_execution` has already published its `Error`).
    pub(crate) fn start(self: &Arc<Self>) -> Result<(), Rejected> {
        self.status.set_as_idle();
        self.register_for_execution(false, false).map(drop)
    }

    /// The state of the mailbox as the oracle prints it: `open|closed scheduled=<b> suspendCount=<n> queued=<n>`.
    #[cfg(test)]
    pub(crate) fn mailbox_view(&self) -> String {
        format!(
            "{} scheduled={} suspendCount={} queued={}",
            if self.status.is_closed() {
                "closed"
            } else {
                "open"
            },
            self.status.is_scheduled(),
            self.status.suspend_count(),
            self.queue.number_of_messages()
        )
    }

    /// Whether a system message is waiting (`hasSystemMessages`).
    #[cfg(test)]
    pub(crate) fn has_system_messages(&self) -> bool {
        self.system_queue.has_messages()
    }

    /// `self`: the reference the actor knows as its own (the repointable one for an actor made by `actorOf`).
    pub(crate) fn self_ref(&self) -> ActorRef {
        let outer = self.outer.lock().unwrap_or_else(PoisonError::into_inner);
        match &*outer {
            Some(outer) => outer.clone(),
            None => self.local_ref(),
        }
    }

    /// The reference to this cell itself, what a `RepointableActorRef` points to once it is installed.
    pub(crate) fn local_ref(&self) -> ActorRef {
        ActorRef(Arc::new(LocalActorRef(
            self.me.upgrade().expect("cell alive"),
        )))
    }

    // ---- dispatch ----

    /// `MessageDispatcher.registerForExecution` (`Dispatcher.scala:146-171`): schedules a turn when the state allows it.
    /// `set_as_scheduled` is the only door, so at most one turn runs at a time. If the executor rejects the mailbox it is
    /// submitted once more; a second rejection makes the mailbox idle again, publishes an `Error` and returns the rejection.
    /// `Ok(true)` means a turn was submitted.
    fn register_for_execution(
        self: &Arc<Self>,
        message_hint: bool,
        system_message_hint: bool,
    ) -> Result<bool, Rejected> {
        let can_run = self.status.can_be_scheduled_for_execution(
            message_hint,
            system_message_hint,
            || self.system_queue.has_messages(),
            || self.queue.has_messages(),
        );
        if !can_run || !self.status.set_as_scheduled() {
            return Ok(false);
        }
        if self.dispatcher.execute(self.clone()).is_ok() {
            return Ok(true);
        }
        match self.dispatcher.execute(self.clone()) {
            Ok(()) => Ok(true), // retry once
            Err(rejected) => {
                self.status.set_as_idle();
                report_dispatch_failure(
                    &self.system.events,
                    ErrorEvent {
                        cause: rejected_execution(),
                        log_source: DISPATCHER_CLASS.to_owned(),
                        log_class: DISPATCHER_CLASS.to_owned(),
                        message: Some("registerForExecution was rejected twice!".to_owned()),
                    },
                );
                Err(rejected)
            }
        }
    }

    /// The cell takes `actor` as its instance (`_actor = newActor()`), remembering its class for reports.
    fn install_actor(&self, actor: Box<dyn Actor>) {
        let mut state = self.st.lock().unwrap();
        state.actor_class = Some(actor.class_name());
        state.actor = Some(actor);
    }

    /// The cell has no instance any more (`clearActorFields`).
    fn remove_actor(&self) {
        self.set_current(None);
        let (actor, replaced) = {
            let mut state = self.st.lock().unwrap();
            state.actor_class = None;
            // `replaced_actor` too: an actor stopped while it restarts still holds the instance it was replacing
            (state.actor.take(), state.replaced_actor.take())
        };
        drop_guarded(actor); // one at a time: a second panic while the first unwinds would abort
        drop_guarded(replaced);
    }

    /// `currentMessage = ...`. The user's value it replaces is dropped outside the state lock, and a panic of its `Drop` stops here.
    fn set_current(&self, current: Option<Envelope>) {
        let replaced = std::mem::replace(&mut self.st.lock().unwrap().current, current);
        drop_guarded(replaced);
    }

    /// `clazz(actor)`. A call of its own, so the state lock is released before the event that carries it is published.
    fn log_class(&self) -> String {
        reporter_class(self.st.lock().unwrap().actor_class)
    }

    /// `ActorCell.handleException` for a failed dispatch (`Dispatch.scala:136-148`): the sender swallows the exception and reports it.
    fn swallow_failed_send(&self, _rejected: Rejected) {
        report_dispatch_failure(
            &self.system.events,
            ErrorEvent {
                cause: rejected_execution(),
                log_source: self.path.to_string(),
                log_class: self.log_class(),
                message: Some("swallowing exception during message send".to_owned()),
            },
        );
    }

    pub(crate) fn system_dispatch(self: &Arc<Self>, msg: SystemMessage) {
        if let Err(msg) = self.system_queue.enqueue(msg) {
            self.dead_system_message(msg); // the queue is closed: the actor terminated
            return;
        }
        if let Err(rejected) = self.register_for_execution(false, true) {
            self.swallow_failed_send(rejected);
        }
    }

    fn enqueue_user(self: &Arc<Self>, env: Envelope) {
        let receiver = self.self_ref();
        let enqueued = enqueue_unless_closed(&self.status, &*self.queue, &receiver, env);
        // a message the queue refused is already dead-lettered: it needs no turn
        if enqueued.awaits_consumption
            && let Err(rejected) = self.register_for_execution(true, false)
        {
            self.swallow_failed_send(rejected);
        }
        for env in enqueued.dead {
            self.system.dead_letter_mailbox.enqueue(&receiver, env);
        }
    }

    /// System messages that reach a terminated actor go to the `deadLetterMailbox` (`systemEnqueue(self, handle)`).
    fn dead_system_message(&self, msg: SystemMessage) {
        self.system
            .dead_letter_mailbox
            .system_enqueue(&self.self_ref(), msg);
    }

    /// `Mailbox.run`: all system messages, then user messages up to the throughput limit, system messages being drained again
    /// after each user message; then idle again and re-register if work is left.
    pub(crate) fn run_mailbox(cell: &Arc<ActorCell>) {
        if !cell.status.is_closed() {
            cell.process_all_system_messages();
            cell.process_mailbox();
        }
        cell.status.set_as_idle();
        // in Pekko a rejection here escapes `Mailbox.run` into the pool; it has already been reported
        let _ = cell.register_for_execution(false, false);
    }

    /// `Mailbox.processAllSystemMessages`: a message found after the mailbox closed is handled as a dead one.
    fn process_all_system_messages(self: &Arc<Self>) {
        loop {
            let batch = self.system_queue.drain();
            if batch.is_empty() {
                break;
            }
            let mut todo = VecDeque::from(batch);
            while let Some(m) = todo.pop_front() {
                let before = self.delivery_state();
                if self.status.is_closed() {
                    self.dead_system_message(m);
                } else if m.is_deferred_in(before) {
                    self.st.lock().unwrap().deferred.push(m);
                } else {
                    self.system_invoke(m);
                }
                // the state went down: what was deferred goes before the rest of the batch
                if self.delivery_state() < before {
                    let deferred = std::mem::take(&mut self.st.lock().unwrap().deferred);
                    for earlier in deferred.into_iter().rev() {
                        todo.push_front(earlier);
                    }
                }
            }
        }
    }

    /// `calculateState`
    fn delivery_state(&self) -> DeliveryState {
        if self.waits_for_children() {
            DeliveryState::WaitingForChildren
        } else if self.status.is_suspended() {
            DeliveryState::Suspended
        } else {
            DeliveryState::Default
        }
    }

    /// `waitingForChildrenOrNull ne null`: the children that are dying hold back a `Recreation` or a `Creation`.
    fn waits_for_children(&self) -> bool {
        matches!(
            *self.children.lock().unwrap(),
            Children::Terminating {
                reason: SuspendReason::Recreation | SuspendReason::Creation,
                ..
            }
        )
    }

    /// `Mailbox.processMailbox`: up to `throughput` user messages while the mailbox is neither suspended nor closed.
    fn process_mailbox(self: &Arc<Self>) {
        let clock = self.dispatcher.clock();
        let mut turn = self.dispatcher.turn_policy().begin(clock);
        while self.status.should_process_message() {
            let Some(env) = self.queue.dequeue() else {
                break;
            };
            self.invoke(env);
            self.process_all_system_messages();
            if !turn.allows_another(clock) {
                break;
            }
        }
    }

    /// Runs `f` on the actor instance (taken out of the cell while the callback runs).
    fn with_actor(
        self: &Arc<Self>,
        f: impl FnOnce(&mut dyn Actor, &mut ActorContext),
    ) -> Result<(), Throwable> {
        let Some(mut actor) = self.st.lock().unwrap().actor.take() else {
            return Ok(());
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut ctx = ActorContext { cell: self };
            f(actor.as_mut(), &mut ctx);
        }));
        self.st.lock().unwrap().actor = Some(actor);
        result.map_err(throwable_of)
    }

    /// `handleInvokeFailure`: suspend the mailbox and every child, mark the actor failed and report to the parent, which decides.
    fn handle_invoke_failure(self: &Arc<Self>, cause: Throwable) {
        self.fail(cause, None, &[]);
    }

    /// `handleInvokeFailure(childrenNotToSuspend, t)`: it has no `isFailed` guard, so every failure suspends and reports. `failed_child`
    /// sent the `Failed` being processed: unless this actor was already failed it becomes the perpetrator and, having suspended itself, is
    /// not suspended again. `still_suspended` are the survivors of a restart that failed: not resumed since the failure that caused it.
    fn fail(
        self: &Arc<Self>,
        cause: Throwable,
        failed_child: Option<ActorRef>,
        still_suspended: &[ActorRef],
    ) {
        let skipped = {
            let mut st = self.st.lock().unwrap();
            if st.failed.is_failed() {
                None
            } else {
                st.failed.set_failed(failed_child.clone());
                failed_child
            }
        };
        self.status.suspend(); // suspendNonRecursive
        for k in self
            .children
            .lock()
            .unwrap()
            .children()
            .into_iter()
            .filter(|k| Some(k) != skipped.as_ref() && !still_suspended.contains(k))
        {
            k.send_system(SystemMessage::Suspend); // suspendChildren
        }
        self.parent.send_system(SystemMessage::Failed {
            child: self.self_ref(),
            cause,
        });
    }

    pub(crate) fn current_sender(&self) -> Option<ActorRef> {
        let state = self.st.lock().unwrap();
        state
            .current
            .as_ref()
            .and_then(|current| current.sender.clone())
    }

    /// `invoke`: the message is the current one while it is processed, and stays so if processing it fails.
    fn invoke(self: &Arc<Self>, env: Envelope) {
        self.set_current(Some(Envelope {
            message: env.message.clone(),
            sender: env.sender.clone(),
        }));
        match self.process_message(env) {
            Ok(()) => self.set_current(None),
            Err(cause) => self.handle_invoke_failure(cause),
        }
    }

    fn process_message(self: &Arc<Self>, env: Envelope) -> Result<(), Throwable> {
        // auto-received messages are handled by the cell itself
        if env.message.is::<PoisonPill>() {
            self.self_ref().stop();
            return Ok(());
        }
        // `Kill` is auto-received: it makes the actor throw `ActorKilledException`
        if env.message.is::<Kill>() {
            let class = "org.apache.pekko.actor.ActorKilledException";
            return Err(Throwable::new(class, Some("Kill")));
        }
        // `Identify` is auto-received: the cell answers, with no sender, and `receive` never sees it
        if let Some(identify) = env.message.downcast_ref::<Identify>() {
            let identity = ActorIdentity {
                correlation_id: identify.message_id.clone(),
                actor_ref: Some(self.self_ref()),
            };
            let asker = env
                .sender
                .unwrap_or_else(|| self.system.dead_letters.clone());
            asker.tell_any(Arc::new(identity), None);
            return Ok(());
        }
        let mut msg = env.message;
        // `receivedTerminated`: only a Terminated that is still queued for us is delivered (unwatch cancels it). The message of a
        // `watch_with` takes its place as the current message; it is not auto-received, whatever its type.
        if let Some(t) = msg.downcast_ref::<Terminated>() {
            let queued = self.st.lock().unwrap().terminated_queued.remove(&t.actor);
            let Some(custom) = queued else {
                return Ok(());
            };
            if let Some(custom) = custom {
                msg = custom.message.clone();
                self.set_current(Some(Envelope {
                    message: msg.clone(),
                    sender: env.sender.clone(),
                }));
            }
        }
        let mut handled = true;
        self.with_actor(|a, ctx| handled = a.receive(ctx, &msg))?;
        if handled {
            return Ok(());
        }
        // `unhandled(Terminated)` throws `DeathPactException`
        if let Some(t) = msg.downcast_ref::<Terminated>() {
            let dead = format!("Monitored actor [{:?}] terminated", t.actor);
            let class = "org.apache.pekko.actor.DeathPactException";
            return Err(Throwable::new(class, Some(&dead)));
        }
        let sender = env
            .sender
            .unwrap_or_else(|| self.system.dead_letters.clone());
        self.system.events.publish_unhandled(UnhandledMessage {
            message: msg,
            sender,
            recipient: self.self_ref(),
        });
        Ok(())
    }

    // ---- system messages ----

    fn system_invoke(self: &Arc<Self>, msg: SystemMessage) {
        match msg {
            SystemMessage::Create(failure) => self.create(failure),
            SystemMessage::Terminate => self.terminate(),
            SystemMessage::Failed { child, cause } => self.handle_failure(child, cause),
            SystemMessage::Recreate { cause } => self.fault_recreate(cause),
            SystemMessage::Resume { cause } => self.fault_resume(cause),
            SystemMessage::Suspend => self.fault_suspend(),
            SystemMessage::Supervise { child, is_async } => self.supervise(child, is_async),
            SystemMessage::Watch { watchee, watcher } => self.add_watcher(watchee, watcher),
            SystemMessage::Unwatch { watchee, watcher } => self.rem_watcher(watchee, watcher),
            SystemMessage::DeathWatchNotification {
                actor,
                existence_confirmed,
                address_terminated,
            } => self.watched_actor_terminated(actor, existence_confirmed, address_terminated),
        }
    }

    /// `create(failure)`: a cell that could not be made fails with the exception it was given (`failure.foreach { throw _ }`).
    fn create(self: &Arc<Self>, failure: Option<Throwable>) {
        if let Some(failure) = failure {
            self.handle_invoke_failure(failure);
            return;
        }
        // `newActor()` runs the user's factory: a throw there is an `ActorInitializationException`, like one in `preStart`
        let actor = match catch_unwind(AssertUnwindSafe(|| self.props.new_actor())) {
            Ok(actor) => actor,
            Err(payload) => {
                let cause = self.creation_failure(throwable_of(payload));
                self.handle_invoke_failure(cause);
                return;
            }
        };
        self.install_actor(actor);
        if let Err(thrown) = self.with_actor(|a, ctx| a.pre_start(ctx)) {
            // the half-built actor is dropped (so `postStop` never runs) and the supervisor decides (default: stop)
            self.remove_actor();
            let cause = self.creation_failure(thrown);
            self.handle_invoke_failure(cause);
        }
    }

    /// `ActorInitializationException(self, "exception during creation, root cause message: [..]", cause)` (`ActorCell.create`).
    fn creation_failure(&self, thrown: Throwable) -> Throwable {
        let mut root_cause = &thrown; // `rootCauseOf`
        while let Some(cause) = &root_cause.cause {
            root_cause = cause;
        }
        let root_message = root_cause
            .message
            .as_deref()
            .unwrap_or("No message available");
        Throwable::new(
            "org.apache.pekko.actor.ActorInitializationException",
            Some(&format!(
                "{}: exception during creation, root cause message: [{root_message}]",
                self.path
            )),
        )
        .with_cause(thrown)
    }

    /// `PreRestartException(self, e, originalCause, optionalMessage)`: its message names the class of the failure that caused the
    /// restart and that of the message being processed then. Its cause is `e`, what `preRestart` threw.
    fn pre_restart_failure(
        &self,
        thrown: Throwable,
        cause: &Throwable,
        message: Option<&AnyMessage>,
    ) -> Throwable {
        let message = message.map_or_else(
            || "None".to_owned(),
            |message| format!("class {}", message_class(message)),
        );
        let (path, class) = (&self.path, &cause.class);
        let text = format!("{path}: exception in preRestart(class {class}, {message})");
        Throwable::new("org.apache.pekko.actor.PreRestartException", Some(&text)).with_cause(thrown)
    }

    /// `PostRestartException(self, e, originalCause)`: its message names the class of the failure that caused the restart, its
    /// cause is `e`, what the factory or `postRestart` threw.
    fn post_restart_failure(&self, thrown: Throwable, original_cause: &Throwable) -> Throwable {
        Throwable::new(
            "org.apache.pekko.actor.PostRestartException",
            Some(&format!(
                "{}: exception post restart (class {})",
                self.path, original_cause.class
            )),
        )
        .with_cause(thrown)
    }

    /// `actor.supervisorStrategy` (once per handled `Failed` and once per child terminated): asked of the instance of the cell, the
    /// one being replaced while it restarts. `None` without an instance; `Err` is what the user's code panicked with.
    fn current_strategy(self: &Arc<Self>) -> Option<Result<SupervisorStrategy, Throwable>> {
        let (actor, restarting) = {
            let mut st = self.st.lock().unwrap();
            match st.actor.take() {
                Some(actor) => (Some(actor), false),
                None => (st.replaced_actor.take(), true),
            }
        };
        let actor = actor?;
        let strategy = catch_unwind(AssertUnwindSafe(|| actor.supervisor_strategy()));
        let mut st = self.st.lock().unwrap();
        if restarting {
            st.replaced_actor = Some(actor);
        } else {
            st.actor = Some(actor);
        }
        drop(st); // before the panic's payload is dropped
        Some(strategy.map_err(throwable_of))
    }

    /// The child whose `Failed` is still the current message: `handleInvokeFailure` takes it as the perpetrator.
    fn current_failed_child(&self) -> Option<ActorRef> {
        let st = self.st.lock().unwrap();
        let failed = st.current.as_ref()?.message.downcast_ref::<Failed>()?;
        Some(failed.child.clone())
    }

    /// `handleFailure`: ask the strategy's decider what to do with the failed child and apply it to that child only.
    fn handle_failure(self: &Arc<Self>, child: ActorRef, cause: Throwable) {
        let failed = Failed {
            child: child.clone(),
            cause: cause.clone(),
        };
        // set before the child is looked up, and not cleared afterwards
        self.set_current(Some(Envelope {
            message: Arc::new(failed),
            sender: Some(child.clone()),
        }));
        if self.children.lock().unwrap().get_by_ref(&child).is_none() {
            return; // not (or no longer) our child: the failure is dropped
        }
        // a strategy or a decider that throws is handled like `Escalate` (Pekko: the exception leaves `handleFailure`): this actor
        // fails with it, and the child that failed, which has suspended itself already, is not suspended again
        let answer = self
            .current_strategy()
            .unwrap_or_else(|| Ok(SupervisorStrategy::default()))
            .and_then(|strategy| {
                match catch_unwind(AssertUnwindSafe(|| strategy.decide(&cause))) {
                    Ok(decision) => Ok((decision, strategy)),
                    Err(payload) => {
                        drop_guarded(strategy); // it holds the decider's captures
                        Err(throwable_of(payload))
                    }
                }
            });
        let (decision, strategy) = match answer {
            Ok(answer) => answer,
            Err(thrown) => {
                self.fail(thrown, Some(child), &[]);
                return;
            }
        };
        self.log_failure(&strategy, &child, &cause, decision);
        match decision {
            Directive::Resume => child.send_system(SystemMessage::Resume { cause: Some(cause) }),
            Directive::Restart => self.process_failure(true, child, cause, &strategy),
            Directive::Stop => self.process_failure(false, child, cause, &strategy),
            Directive::Escalate => self.escalate(child, cause),
        }
        drop_guarded(strategy);
    }

    /// `SupervisorStrategy.logFailure`: before a directive is applied, `Resume` publishes a `Warning` and `Restart` and `Stop` an `Error`,
    /// both with the failed child as source and the strategy as class. `Escalate` is left to a higher level.
    fn log_failure(
        &self,
        strategy: &SupervisorStrategy,
        child: &ActorRef,
        cause: &Throwable,
        decision: Directive,
    ) {
        if !strategy.logging_enabled() || decision == Directive::Escalate {
            return;
        }
        let log_source = child.path().to_string();
        let log_class = strategy.class_name().to_owned();
        let message = logged_message(cause);
        self.publish_unguarded(|events| {
            if decision == Directive::Resume {
                events.publish_warning(WarningEvent {
                    log_source,
                    log_class,
                    message,
                });
            } else {
                events.publish_error(ErrorEvent {
                    cause: cause.clone(),
                    log_source,
                    log_class,
                    message,
                });
            }
        });
    }

    /// The publications of supervision and lifecycle: `ActorCell.publish` (`ActorCell.scala:747-750`, swallows NonFatal) and the
    /// plain `eventStream.publish` of `FaultHandling`. Without the recursion guard of `report_dispatch_failure`, so the reports of a
    /// subscriber that cannot be scheduled still reach the subscribers. What a subscriber panics with stops here.
    fn publish_unguarded(&self, publish: impl FnOnce(&EventStream)) {
        let _ =
            catch_unwind(AssertUnwindSafe(|| publish(&self.system.events))).map_err(drop_guarded);
    }

    /// `processFailure`: a restart is only granted while every affected child is within its limit, otherwise they are stopped.
    /// OneForOne affects the failed child; AllForOne affects all children, for Restart and for Stop alike.
    fn process_failure(
        self: &Arc<Self>,
        restart: bool,
        child: ActorRef,
        cause: Throwable,
        strategy: &SupervisorStrategy,
    ) {
        let affected = if strategy.is_all_for_one() {
            self.children.lock().unwrap().children()
        } else {
            vec![child.clone()]
        };
        let max = strategy.max_nr_of_retries();
        // `children.forall(_.requestRestartPermission(...))` short-circuits at the first refusal
        let granted = restart && {
            let mut st = self.st.lock().unwrap();
            affected.iter().all(|k| {
                st.restart_stats
                    .entry(k.clone())
                    .or_default()
                    .request_restart_permission(max)
            })
        };
        if granted {
            for k in &affected {
                if *k != child {
                    k.send_system(SystemMessage::Suspend); // `restartChild(suspendFirst = true)` for the healthy ones
                }
                k.send_system(SystemMessage::Recreate {
                    cause: cause.clone(),
                });
            }
        } else {
            for k in &affected {
                self.stop_actor(k);
            }
        }
    }

    /// `Escalate` (`handleFailure` returns false, so the supervisor rethrows the child's exception and `handleInvokeFailure` runs for it):
    /// this actor fails with the very same cause; the failed child is not suspended again.
    fn escalate(self: &Arc<Self>, child: ActorRef, cause: Throwable) {
        self.fail(cause, Some(child), &[]);
    }

    /// `faultSuspend`: suspend this actor and, recursively, its children (the failed parent's whole subtree waits for the decision).
    fn fault_suspend(self: &Arc<Self>) {
        self.status.suspend();
        for k in self.children.lock().unwrap().children() {
            k.send_system(SystemMessage::Suspend);
        }
    }

    /// `faultResume`: the same instance carries on, and the failed message stays the current one. `resumeChildren`: the child whose failure was escalated gets
    /// the cause back (it clears its own failure), every other child only loses one suspension. `cause` is `None` for an actor that was merely suspended
    /// along with its failed parent.
    fn fault_resume(self: &Arc<Self>, cause: Option<Throwable>) {
        if self.has_no_instance() {
            // there is no instance to resume (it never got created): `Resume` becomes `Create`
            self.change_into_create("Resume", cause.as_ref());
            return;
        }
        let failed_fatally = matches!(self.st.lock().unwrap().failed, FailedInfo::FailedFatally);
        if let Some(cause) = cause.clone().filter(|_| failed_fatally) {
            self.publish_error(format!("changing Resume into Restart after {cause}"));
            self.fault_recreate(cause);
            return;
        }
        let perp = self.st.lock().unwrap().failed.perpetrator();
        self.resume_after_failure(cause.is_some());
        for k in self.children.lock().unwrap().children() {
            let own = if Some(&k) == perp.as_ref() {
                cause.clone()
            } else {
                None
            };
            k.send_system(SystemMessage::Resume { cause: own });
        }
    }

    /// `faultRecreate`: `preRestart` on the failed instance, wait for the children to die, then `finishRecreate`.
    fn fault_recreate(self: &Arc<Self>, cause: Throwable) {
        if self.has_no_instance() {
            // there is no instance to restart (it never got created): `Recreate` becomes `Create`
            self.change_into_create("Recreate", Some(&cause));
            return;
        }
        if self.children.lock().unwrap().is_terminating() {
            // the actor is shutting down anyway: `faultResume(null)` keeps the suspend counters balanced and the failure
            self.fault_resume(None);
            return;
        }
        let (message, failed_fatally) = {
            let st = self.st.lock().unwrap();
            let fatally = matches!(st.failed, FailedInfo::FailedFatally);
            let message = st.current.as_ref().map(|current| current.message.clone());
            (message, fatally)
        };
        if !failed_fatally
            && let Err(thrown) =
                self.with_actor(|a, ctx| a.pre_restart(ctx, &cause, message.as_ref()))
        {
            let text = thrown.message.clone();
            let failure = self.pre_restart_failure(thrown, &cause, message.as_ref());
            self.publish_caused_error(failure, text);
        }
        drop(message); // so that the cell's own drop, which is guarded, is the last one
        self.set_current(None); // `clearActorFields`
        {
            let mut st = self.st.lock().unwrap();
            st.replaced_actor = st.actor.take();
        }
        if self.children.lock().unwrap().set_recreation_reason() {
            self.st.lock().unwrap().recreate_cause = Some(cause);
        } else {
            self.finish_recreate(cause);
        }
    }

    /// `actor eq null`: the cell has no instance, and none waits to be replaced (an actor that restarts while its children die
    /// keeps `_actor` in Pekko, here it is parked in `replaced_actor`).
    fn has_no_instance(&self) -> bool {
        let state = self.st.lock().unwrap();
        state.actor.is_none() && state.replaced_actor.is_none()
    }

    /// `changing <directive> into Create after <cause>`: publishes the error, then `faultCreate`.
    fn change_into_create(self: &Arc<Self>, directive: &str, cause: Option<&Throwable>) {
        let cause = cause.map_or_else(|| "null".to_owned(), ToString::to_string);
        self.publish_error(format!("changing {directive} into Create after {cause}"));
        self.fault_create();
    }

    /// `eventStream.publish(Error(self.path.toString, clazz(actor), message))`: an `Error` with no cause.
    fn publish_error(&self, message: String) {
        let no_cause = Throwable::new("org.apache.pekko.event.Logging$Error$NoCause$", None);
        self.publish_caused_error(no_cause, Some(message));
    }

    /// `eventStream.publish(Error(cause, self.path.toString, clazz(actor), message))`.
    fn publish_caused_error(&self, cause: Throwable, message: Option<String>) {
        let error = ErrorEvent {
            cause,
            log_source: self.path.to_string(),
            log_class: self.log_class(),
            message,
        };
        self.publish_unguarded(|events| events.publish_error(error));
    }

    /// `faultCreate`: stop the children, then create the actor again (`finishCreate`) once they are gone.
    fn fault_create(self: &Arc<Self>) {
        let children = self.children.lock().unwrap().children();
        for child in &children {
            self.stop_actor(child);
        }
        if !self.children.lock().unwrap().set_creation_reason() {
            self.finish_create();
        }
    }

    /// `finishCreate`: resume the mailbox, forget the failure and run `Create` again.
    fn finish_create(self: &Arc<Self>) {
        self.resume_after_failure(true);
        self.create(None);
    }

    /// `resumeNonRecursive` + `clearFailed` (a child that was only suspended along with its parent keeps its own failure state).
    fn resume_after_failure(&self, clear_failed: bool) {
        self.status.resume();
        if clear_failed {
            let mut st = self.st.lock().unwrap();
            st.failed.clear_failed();
        }
    }

    /// `finishRecreate`: resume the mailbox, create the fresh instance, run its `postRestart` and only then restart the children that
    /// `preRestart` did not stop.
    fn finish_recreate(self: &Arc<Self>, cause: Throwable) {
        // taken before the new instance exists: the children it creates are not restarted
        let survivors = self.children.lock().unwrap().children();
        self.resume_after_failure(true);
        let replaced = self.st.lock().unwrap().replaced_actor.take();
        let fresh = match catch_unwind(AssertUnwindSafe(|| self.props.new_actor())) {
            Ok(fresh) => fresh,
            Err(payload) => {
                // the old instance stays in place, so it is stopped (and post-stopped) with the cell
                self.st.lock().unwrap().actor = replaced;
                self.fail_restart(throwable_of(payload), &cause, &survivors);
                return;
            }
        };
        self.install_actor(fresh);
        drop_guarded(replaced);
        if let Err(thrown) = self.with_actor(|a, ctx| a.post_restart(ctx, &cause)) {
            // the new instance stays in place (Pekko keeps `_actor`): it gets its `postStop` when the actor stops
            self.fail_restart(thrown, &cause, &survivors);
            return;
        }
        for survivor in survivors {
            survivor.send_system(SystemMessage::Recreate {
                cause: cause.clone(),
            });
        }
    }

    /// `setFailedFatally` and `handleInvokeFailure(survivors, PostRestartException)`: the survivors are still suspended.
    fn fail_restart(
        self: &Arc<Self>,
        thrown: Throwable,
        original_cause: &Throwable,
        survivors: &[ActorRef],
    ) {
        self.st.lock().unwrap().failed = FailedInfo::FailedFatally; // setFailedFatally
        let failure = self.post_restart_failure(thrown, original_cause);
        self.fail(failure, None, survivors);
    }

    /// `FaultHandling.terminate`
    fn terminate(self: &Arc<Self>) {
        self.unwatch_watched_actors();
        let kids = self.children.lock().unwrap().children();
        for k in &kids {
            self.stop_actor(k);
        }
        let (was_terminating, waiting) = {
            let mut ch = self.children.lock().unwrap();
            (ch.is_terminating(), ch.set_termination_reason())
        };
        if waiting {
            if !was_terminating {
                self.status.suspend(); // suspendNonRecursive
                // `setFailed(self)`: no failure is propagated during shutdown, and no child is the perpetrator any more
                self.st.lock().unwrap().failed.set_failed(None);
            }
        } else {
            *self.children.lock().unwrap() = Children::Terminated; // setTerminated()
            self.finish_terminate();
        }
    }

    /// `FaultHandling.finishTerminate` — the order of the steps matters.
    fn finish_terminate(self: &Arc<Self>) {
        if let Err(thrown) = self.with_actor(|a, ctx| a.post_stop(ctx)) {
            let message = thrown.message.clone();
            self.publish_caused_error(thrown, message);
        }
        self.remove_actor();
        // `clearFieldsForTermination`: what is still deferred is forgotten, it does not even become a dead letter
        let forgotten = std::mem::take(&mut self.st.lock().unwrap().deferred);
        drop(forgotten); // outside the lock: it holds references
        // dispatcher.detach: close the mailbox, hand what is left to dead letters
        let leftovers = UnboundedMailbox.create();
        self.status.become_closed();
        let receiver = self.self_ref();
        self.queue.clean_up(&receiver, &*leftovers); // `cleanUp`; a producer that raced with the close drains for itself (Q-MBX-8)
        let sys = self.system_queue.clean_up();
        for m in sys {
            self.dead_system_message(m);
        }
        while let Some(env) = leftovers.dequeue() {
            self.system.dead_letter_mailbox.enqueue(&receiver, env);
        }
        self.parent
            .send_system(SystemMessage::DeathWatchNotification {
                actor: self.self_ref(),
                existence_confirmed: true,
                address_terminated: false,
            });
        self.tell_watchers_we_died();
        self.unwatch_watched_actors();
        // the repointable reference holds this cell: let go of it so the pair can be freed (Pekko relies on the GC)
        let released = self
            .outer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        drop(released); // outside the lock: it may be the last reference to the repointable
    }

    // ---- children ----

    /// `reserveChild(name)` for the provider's guardians.
    pub(crate) fn reserve_child(&self, name: &str) {
        self.children
            .lock()
            .unwrap()
            .reserve(name)
            .expect("guardian names are unique");
    }

    /// `initChild(ref)`.
    pub(crate) fn init_child(&self, name: &str, child: ActorRef) {
        self.children.lock().unwrap().add(name, child);
    }

    fn random_name(&self) -> String {
        let mut name = String::from("$");
        base64_append(
            self.name_counter.fetch_add(1, Ordering::SeqCst) as i64,
            &mut name,
        );
        name
    }

    /// `makeChild` (`Children.scala:279-352`). `asynchronous` is Pekko's `async`: `context.actorOf` is synchronous (a started
    /// `LocalActorRef`, `async = false`) while `attachChild`, which `system.actorOf` uses, hands out a `RepointableActorRef` that
    /// this actor starts when it processes `Supervise` (`async = true`).
    pub(crate) fn attach_child(
        self: &Arc<Self>,
        props: Props,
        name: Option<&str>,
        asynchronous: bool,
    ) -> Result<ActorRef, ActorError> {
        self.attach_child_made_by(props, name, asynchronous, Self::make_repointed)
    }

    /// `newCell` of an actor made by `actorOf`. Nothing in Rakka can make this fail yet (Pekko: the mailbox type, the dispatcher), so
    /// the production maker never does; `attach_child_made_by` lets tests supply one that does.
    fn make_repointed(
        system: &Arc<SystemInner>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
        outer: &ActorRef,
    ) -> Result<Arc<ActorCell>, Throwable> {
        Ok(Self::new_repointed(
            system,
            system.dispatcher.clone(),
            props,
            parent,
            path,
            outer.clone(),
        ))
    }

    pub(crate) fn attach_child_made_by(
        self: &Arc<Self>,
        props: Props,
        name: Option<&str>,
        asynchronous: bool,
        make_cell: CellMaker,
    ) -> Result<ActorRef, ActorError> {
        let name = match name {
            Some("") => {
                return Err(ActorError::new(
                    ErrorClass::InvalidActorName,
                    "actor name must not be empty",
                ));
            }
            Some(n) => {
                validate_path_element(n, None).map_err(ActorError::from)?;
                n.to_string()
            }
            None => self.random_name(),
        };
        // makeChild
        if self.children.lock().unwrap().is_terminating() {
            return Err(ActorError::new(
                ErrorClass::IllegalState,
                "cannot create children while terminating or terminated",
            ));
        }
        self.children.lock().unwrap().reserve(&name)?;
        let path = self.path.new_child(&name, new_uid());
        // `getMailboxType` runs before any reference exists (`LocalActorRefProvider.actorOf`); a failure releases the name (`makeChild`)
        let props = props.deployed_over(self.system.settings.deployer.lookup_path(&path).as_ref());
        let mailbox_id = props.mailbox_id().to_owned();
        let props = match props.resolve_mailbox(
            &self.system.settings.mailboxes,
            self.system.settings.mailbox_requirement,
        ) {
            Ok(resolved) => resolved,
            Err(cause) => {
                self.children.lock().unwrap().unreserve(&name);
                return Err(ConfigurationError::caused_by(
                    format!(
                        "configuration problem while creating [{path}] with dispatcher [{DEFAULT_DISPATCHER_ID}] and mailbox [{mailbox_id}]"
                    ),
                    &cause,
                )
                .into());
            }
        };
        if asynchronous {
            // `provider.actorOf(..., async = true)`: a `RepointableActorRef`, which tells the supervisor (this actor) to `Supervise` it
            let child_ref = ActorRef(RepointableActorRef::for_actor(
                path.clone(),
                self.self_ref(),
                {
                    let (system, parent) = (self.system.clone(), self.self_ref());
                    Box::new(move |outer: &ActorRef| {
                        // `point(catchFailures = true)`: what `newCell` throws is not lost, the actor fails to initialise instead
                        make_cell(&system, props.clone(), parent.clone(), path.clone(), outer)
                            .unwrap_or_else(|failure| {
                                ActorCell::new_with_failure(
                                    &system,
                                    props,
                                    parent,
                                    path,
                                    outer.clone(),
                                    failure,
                                )
                            })
                    })
                },
                PushPolicy {
                    timeout: self.system.settings.unstarted_push_timeout,
                    events: self.system.events.clone(),
                    dead_letters: self.system.dead_letters.clone(),
                },
            ));
            self.system_dispatch(SystemMessage::Supervise {
                child: child_ref.clone(),
                is_async: true,
            });
            self.inherit_suspension(&child_ref);
            self.children.lock().unwrap().init_child(&child_ref);
            Ok(child_ref)
        } else {
            // `async = false`: a `LocalActorRef` whose cell is attached to the dispatcher right now
            let child = ActorCell::new_local(
                &self.system,
                self.system.dispatcher.clone(),
                props,
                self.self_ref(),
                path,
            );
            let child_ref = child.self_ref();
            self.inherit_suspension(&child_ref);
            self.children.lock().unwrap().init_child(&child_ref);
            // `actor.start()` is after `initChild` and outside the `try` that unreserves the name: the caller gets the exception,
            // the name stays taken and the child stays registered (`Children.scala:317-350`)
            if child.start().is_err() {
                throw(rejected_execution());
            }
            Ok(child_ref)
        }
    }

    /// `makeChild`: the child gets one `Suspend` for every suspension of this actor (`if (mailbox ne null) { ... actor.suspend() }`).
    fn inherit_suspension(&self, child: &ActorRef) {
        for _ in 0..self.status.suspend_count() {
            child.send_system(SystemMessage::Suspend);
        }
    }

    /// `ActorCell.supervise`: the first thing a new child sends. A child nobody reserved a name for is reported; a registered
    /// asynchronous one is started now, in this actor's turn (`handleSupervise` → `point`). Ignored while this actor terminates.
    fn supervise(self: &Arc<Self>, child: ActorRef, is_async: bool) {
        let registered = {
            let mut children = self.children.lock().unwrap();
            if children.is_terminating() {
                return;
            }
            children.init_child(&child)
        };
        if !registered {
            self.publish_error(format!(
                "received Supervise from unregistered child {child:?}, this will not end well"
            ));
        } else if is_async && let Some(repointable) = child.0.as_repointable() {
            // `handleSupervise` → `point`: `cell.start()` is outside `catchFailures`, so a rejected attach fails this actor
            // (`systemInvoke`'s `handleInvokeFailure(Nil, e)`), not the new one
            if repointable.point().is_err() {
                self.handle_invoke_failure(rejected_execution());
            }
        }
    }

    /// `ActorCell.stop(actor)` (`Children.scala:132-145`): mark a child as about to die, then ask the actor to terminate. A child that
    /// has not started (`RepointableRef.isStarted` is false) is not marked: it only gets its `Terminate`, so this actor never waits for a
    /// termination that cannot happen until the child is pointed (oracle cases `MBX-UT-03`, `MBX-UT-06`).
    pub(crate) fn stop_actor(&self, actor: &ActorRef) {
        let may_wait_for_death = actor.0.as_repointable().is_none_or(|r| r.is_started());
        {
            let mut ch = self.children.lock().unwrap();
            if may_wait_for_death && ch.get_by_ref(actor).is_some() {
                ch.shall_die(actor);
            }
        }
        actor.stop();
    }

    fn handle_child_terminated(self: &Arc<Self>, child: &ActorRef) {
        self.st.lock().unwrap().restart_stats.remove(child);
        let change = self.children.lock().unwrap().remove(child);
        // `strategy.handleChildTerminated` does nothing (it cannot be overridden yet), but asking for the strategy is user code;
        // what it throws fails this actor and the pending stop, restart or creation goes on
        match self.current_strategy() {
            Some(Ok(strategy)) => drop_guarded(strategy),
            Some(Err(thrown)) => {
                let message = Some("handleChildTerminated failed".to_owned());
                self.publish_caused_error(thrown.clone(), message);
                self.fail(thrown, self.current_failed_child(), &[]);
            }
            None => {}
        }
        match change {
            Some(SuspendReason::Termination) => self.finish_terminate(),
            Some(SuspendReason::Recreation) => {
                let cause = self.st.lock().unwrap().recreate_cause.take(); // the guard must not outlive this statement
                if let Some(cause) = cause {
                    self.finish_recreate(cause);
                }
            }
            Some(SuspendReason::Creation) => self.finish_create(),
            _ => {}
        }
    }

    // ---- death watch ----

    /// `context.watch(subject)`, or `watchWith` when there is a `custom` termination message.
    pub(crate) fn watch(self: &Arc<Self>, subject: &ActorRef, custom: Option<CustomTermination>) {
        let me = self.self_ref();
        if *subject == me {
            return;
        }
        let previous = self
            .st
            .lock()
            .unwrap()
            .watching
            .iter()
            .find(|(w, _)| w == subject)
            .map(|(_, previous)| previous.clone());
        if let Some(previous) = previous {
            // outside the lock: it runs the `PartialEq` of the user, and may throw
            check_watching_same(&me, subject, previous.as_ref(), custom.as_ref());
            return;
        }
        self.st
            .lock()
            .unwrap()
            .watching
            .push((subject.clone(), custom));
        subject.send_system(SystemMessage::Watch {
            watchee: subject.clone(),
            watcher: me,
        });
    }

    /// Removes `subject` from `watching`: `None` if it was not watched, otherwise its custom message, if any.
    fn stop_watching(&self, subject: &ActorRef) -> Option<Option<CustomTermination>> {
        let mut st = self.st.lock().unwrap();
        let index = st.watching.iter().position(|(w, _)| w == subject)?;
        Some(st.watching.remove(index).1)
    }

    /// `context.unwatch(subject)`: also cancels a `Terminated` that is already queued.
    pub(crate) fn unwatch(self: &Arc<Self>, subject: &ActorRef) {
        let me = self.self_ref();
        if *subject != me && self.stop_watching(subject).is_some() {
            subject.send_system(SystemMessage::Unwatch {
                watchee: subject.clone(),
                watcher: me,
            });
        }
        // bound, not discarded: a user's message is dropped only after the lock is released
        let _cancelled = self.st.lock().unwrap().terminated_queued.remove(subject);
    }

    fn add_watcher(self: &Arc<Self>, watchee: ActorRef, watcher: ActorRef) {
        let me = self.self_ref();
        let (watchee_self, watcher_self) = (watchee == me, watcher == me);
        if watchee_self && !watcher_self {
            let mut st = self.st.lock().unwrap();
            if !st.watched_by.contains(&watcher) {
                st.watched_by.push(watcher);
            }
        } else if !watchee_self && watcher_self {
            self.watch(&watchee, None);
        }
        // anything else is a bug in Pekko ("illegal Watch") and is ignored here
    }

    fn rem_watcher(self: &Arc<Self>, watchee: ActorRef, watcher: ActorRef) {
        let me = self.self_ref();
        let (watchee_self, watcher_self) = (watchee == me, watcher == me);
        if watchee_self && !watcher_self {
            self.st.lock().unwrap().watched_by.retain(|w| w != &watcher);
        } else if !watchee_self && watcher_self {
            self.unwatch(&watchee);
        }
    }

    // ---- children lookup ----

    /// `context.child(name)`.
    pub(crate) fn child(&self, name: &str) -> Option<ActorRef> {
        self.children.lock().unwrap().get_by_name(name)
    }

    /// `context.children`.
    pub(crate) fn children(&self) -> Vec<ActorRef> {
        self.children.lock().unwrap().children()
    }

    fn watched_actor_terminated(
        self: &Arc<Self>,
        actor: ActorRef,
        existence_confirmed: bool,
        address_terminated: bool,
    ) {
        let watched = self.stop_watching(&actor);
        if let Some(custom) = watched
            && !self.children.lock().unwrap().is_terminating()
        {
            // `terminatedQueuedFor`: a registration made while one is already queued does not replace it
            let _refused = match self
                .st
                .lock()
                .unwrap()
                .terminated_queued
                .entry(actor.clone())
            {
                Entry::Vacant(slot) => {
                    slot.insert(custom);
                    None
                }
                Entry::Occupied(_) => Some(custom),
            };
            self.self_ref().tell_any(
                Arc::new(Terminated {
                    actor: actor.clone(),
                    existence_confirmed,
                    address_terminated,
                }),
                Some(actor.clone()),
            );
        }
        let is_child = self.children.lock().unwrap().get_by_ref(&actor).is_some();
        if is_child {
            self.handle_child_terminated(&actor);
        }
    }

    fn tell_watchers_we_died(self: &Arc<Self>) {
        let watchers = std::mem::take(&mut self.st.lock().unwrap().watched_by);
        let me = self.self_ref();
        for w in watchers {
            // the parent receives a DeathWatchNotification by default
            if w != self.parent {
                w.send_system(SystemMessage::DeathWatchNotification {
                    actor: me.clone(),
                    existence_confirmed: true,
                    address_terminated: false,
                });
            }
        }
    }

    fn unwatch_watched_actors(self: &Arc<Self>) {
        let (watching, _queued) = {
            let mut st = self.st.lock().unwrap();
            let queued = std::mem::take(&mut st.terminated_queued);
            (std::mem::take(&mut st.watching), queued)
        };
        let me = self.self_ref();
        for (w, _) in watching {
            w.send_system(SystemMessage::Unwatch {
                watchee: w.clone(),
                watcher: me.clone(),
            });
        }
    }
}

#[cfg(test)]
mod reporter_class_tests {
    use super::*;

    #[test]
    fn the_reporter_is_the_class_of_the_instance_when_there_is_one() {
        assert_eq!(reporter_class(Some("my::Actor")), "my::Actor");
    }

    #[test]
    fn without_an_instance_the_reporter_is_the_actor_trait() {
        assert_eq!(reporter_class(None), "org.apache.pekko.actor.Actor");
    }

    #[test]
    fn a_message_is_named_by_the_jvm_class_it_stands_for() {
        let system = crate::actor::system::ActorSystem::create("t-message-class").unwrap();
        let (actor, cause) = (system.dead_letters(), Throwable::illegal_state("x"));
        let (correlation_id, actor_ref) = (Arc::new(1_u8), None);
        #[rustfmt::skip] // one case per line
        let cases: [(AnyMessage, &str); 7] = [
            (Arc::new("text"), "java.lang.String"),
            (Arc::new(String::new()), "java.lang.String"),
            (Arc::new(Kill), "org.apache.pekko.actor.Kill$"),
            (Arc::new(ActorIdentity { correlation_id, actor_ref }), "org.apache.pekko.actor.ActorIdentity"),
            (Arc::new(Terminated { actor: actor.clone(), existence_confirmed: true, address_terminated: false }), "org.apache.pekko.actor.Terminated"),
            (Arc::new(Failed { child: actor, cause }), "org.apache.pekko.dispatch.sysmsg.Failed"),
            (Arc::new(1_u8), "?"), // a user type has no JVM class (Q-CELL-6)
        ];
        for (message, class) in cases {
            assert_eq!(message_class(&message), class);
        }
    }
}

#[cfg(test)]
mod terminating_survivor_tests {
    use super::*;
    use crate::actor::system::ActorSystem;
    use std::sync::atomic::AtomicUsize;

    /// A chain of actors, one per depth: "fail" throws, anything else blocks until the test lets go of the gate of its depth.
    #[derive(Clone)]
    struct Chain {
        depth: usize,
        born: Arc<Mutex<Vec<ActorRef>>>,
        gates: Arc<[Mutex<()>; 4]>,
        blocked: Arc<AtomicUsize>,
    }

    impl Actor for Chain {
        fn pre_start(&mut self, ctx: &mut ActorContext) {
            self.born.lock().unwrap().push(ctx.self_ref());
            if self.depth > 0 {
                let next = Chain {
                    depth: self.depth - 1,
                    ..self.clone()
                };
                ctx.actor_of(Props::new(move || next.clone())).unwrap();
            }
        }
        fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.downcast_ref::<&str>() == Some(&"fail") {
                throw(Throwable::illegal_state("fail"));
            }
            self.blocked.fetch_add(1, Ordering::SeqCst);
            drop(self.gates[self.depth].lock().unwrap());
            true
        }
    }

    fn wait_for(what: &str, condition: impl Fn() -> bool) {
        let started = std::time::Instant::now();
        while !condition() {
            assert!(started.elapsed().as_secs() < 10, "timed out: {what}");
            std::thread::yield_now();
        }
    }

    /// An actor stopped while its parent still decides about its failure gets `Recreate` as a terminating one: `faultResume(null)`,
    /// which resumes its children too and does not clear `failed`.
    #[test]
    fn recreate_of_a_terminating_actor_resumes_its_children_and_keeps_it_failed() {
        let system = ActorSystem::create("t-terminating-survivor").unwrap();
        let root = Chain {
            depth: 3,
            born: Arc::default(),
            gates: Arc::default(),
            blocked: Arc::default(),
        };
        let (born, gates, blocked) = (root.born.clone(), root.gates.clone(), root.blocked.clone());
        let top = system.actor_of(Props::new(move || root.clone())).unwrap();
        wait_for("chain", || born.lock().unwrap().len() == 4);
        let born = born.lock().unwrap().clone();
        let (failing, child, leaf) = (&born[1], &born[2], &born[3]);
        let suspended = |actor: &ActorRef, times: u8| {
            let view = actor.mailbox_view();
            view.contains(&format!("suspendCount={times} "))
        };
        let deciding = gates[3].lock().unwrap();
        let held = gates[0].lock().unwrap();
        leaf.tell("block", None);
        top.tell("block", None);
        wait_for("top and leaf busy", || blocked.load(Ordering::SeqCst) == 2);
        failing.tell("fail", None);
        wait_for("failed", || suspended(failing, 1) && suspended(child, 1));
        failing.stop();
        wait_for("stopping", || suspended(failing, 2) && suspended(child, 2));
        drop(deciding); // `top` now decides: restart
        wait_for("resumed", || suspended(failing, 1) && suspended(child, 1));
        assert!(failing.is_failed());
        drop(held);
    }
}
