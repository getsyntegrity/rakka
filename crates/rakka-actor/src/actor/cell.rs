// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala (Apache-2.0).
//! `ActorCell`: the engine behind a local actor — mailbox, system-message processing, children,
//! death watch and the termination sequence. Follows `ActorCell.scala` and `dungeon/*`.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError, Weak};

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::children::{Children, SuspendReason};
use crate::actor::fault::{Throwable, throw};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Kill, PoisonPill, Props};
use crate::actor::repointable::{PushPolicy, RepointableActorRef};
use crate::actor::supervision::{Directive, OneForOneStrategy, RestartStats, SupervisorStrategy};
use crate::actor::system::SystemInner;
use crate::actor::validate_path_element;
use crate::actor::{ActorError, ConfigurationError, ErrorClass};
use crate::dispatch::dispatcher::Dispatcher;
use crate::dispatch::dispatcher::Rejected;
use crate::dispatch::mailbox_close::enqueue_unless_closed;
use crate::dispatch::mailbox_status::MailboxStatus;
use crate::dispatch::message_queue::{Envelope, MessageQueue, UnboundedMailbox};
use crate::dispatch::sysmsg::{Failed, SystemMessage, Terminated};
use crate::dispatch::system_queue::SystemQueue;
use crate::event::UnhandledMessage;
use crate::event::{ErrorEvent, report_dispatch_failure};
use crate::util::helpers::base64_append;

#[derive(Default)]
struct CellState {
    actor: Option<Box<dyn Actor>>,
    /// `clazz(actor)` of the current instance, kept while the instance is taken out to run a callback; `None` without an instance
    actor_class: Option<&'static str>,
    /// sender of the message being processed (`context.sender()`)
    sender: Option<ActorRef>,
    watching: Vec<ActorRef>,
    watched_by: Vec<ActorRef>,
    /// `terminatedQueued`: a `Terminated` that was put in our mailbox and not delivered yet; `unwatch` cancels it.
    terminated_queued: HashSet<ActorRef>,
    /// `isFailed`: the actor reported a failure and waits for its supervisor's decision
    failed: bool,
    /// the message being processed when it failed (`currentMessage`), handed to `preRestart`
    failed_message: Option<AnyMessage>,
    /// the instance a restart replaced: Pekko keeps it in `_actor` until the factory has produced the new one
    replaced_actor: Option<Box<dyn Actor>>,
    /// the cause of a `Recreate` that waits for the children to die (`Recreation(cause)`)
    recreate_cause: Option<Throwable>,
    /// `ChildRestartStats` of the children this actor supervises, by child
    restart_stats: std::collections::HashMap<ActorRef, RestartStats>,
    /// the child whose failure this actor escalated (`FailedRef(perp)`); it is the one child not suspended by this actor's own failure
    perpetrator: Option<ActorRef>,
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
            Throwable::new("java.lang.RuntimeException", msg.as_deref())
        }
    }
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
    /// ActorCell", failure)`; the supervisor was already told about the actor when the reference was made. Pekko keeps `failure` as the exception's
    /// cause; `Throwable` has no cause chain, so it is not kept (the supervisor's `logFailure` text, which reads it, is another contract).
    pub(crate) fn new_with_failure(
        system: &Arc<SystemInner>,
        props: Props,
        parent: ActorRef,
        path: ActorPath,
        outer: ActorRef,
    ) -> Arc<ActorCell> {
        let initialization_failure = Throwable::new(
            "org.apache.pekko.actor.ActorInitializationException",
            Some(&format!("{path}: failure while creating ActorCell")),
        );
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
                        message: "registerForExecution was rejected twice!".to_owned(),
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
        let mut state = self.st.lock().unwrap();
        state.actor = None;
        state.actor_class = None;
    }

    /// `ActorCell.handleException` for a failed dispatch (`Dispatch.scala:129-141`): the sender swallows the exception and reports it.
    fn swallow_failed_send(&self, _rejected: Rejected) {
        report_dispatch_failure(
            &self.system.events,
            ErrorEvent {
                cause: rejected_execution(),
                log_source: self.path.to_string(),
                log_class: reporter_class(self.st.lock().unwrap().actor_class),
                message: "swallowing exception during message send".to_owned(),
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
            for m in batch {
                if self.status.is_closed() {
                    self.dead_system_message(m);
                } else {
                    self.system_invoke(m);
                }
            }
        }
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
    fn handle_invoke_failure(self: &Arc<Self>, cause: Throwable, message: Option<AnyMessage>) {
        self.fail(cause, message, None);
    }

    /// `handleInvokeFailure(childrenNotToSuspend, t)`. `perpetrator` is the failed child whose `Failed` is being escalated: it already
    /// suspended itself, so it is not suspended again (and it is the one child that gets the cause back on `Resume`).
    fn fail(
        self: &Arc<Self>,
        cause: Throwable,
        message: Option<AnyMessage>,
        perpetrator: Option<ActorRef>,
    ) {
        {
            let mut st = self.st.lock().unwrap();
            if st.failed {
                return;
            }
            st.failed = true;
            st.failed_message = message;
            st.perpetrator = perpetrator.clone();
        }
        self.status.suspend(); // suspendNonRecursive
        for k in self
            .children
            .lock()
            .unwrap()
            .children()
            .into_iter()
            .filter(|k| Some(k) != perpetrator.as_ref())
        {
            k.send_system(SystemMessage::Suspend); // suspendChildren
        }
        self.parent.send_system(SystemMessage::Failed {
            child: self.self_ref(),
            cause,
        });
    }

    pub(crate) fn current_sender(&self) -> Option<ActorRef> {
        self.st.lock().unwrap().sender.clone()
    }

    fn invoke(self: &Arc<Self>, env: Envelope) {
        // auto-received messages are handled by the cell itself
        if env.message.is::<PoisonPill>() {
            self.self_ref().stop();
            return;
        }
        // `Kill` is auto-received: it makes the actor throw `ActorKilledException`
        if env.message.is::<Kill>() {
            let cause = Throwable::new("org.apache.pekko.actor.ActorKilledException", Some("Kill"));
            self.handle_invoke_failure(cause, Some(env.message));
            return;
        }
        // `receivedTerminated`: only a Terminated that is still queued for us is delivered (unwatch cancels it)
        if let Some(t) = env.message.downcast_ref::<Terminated>()
            && !self.st.lock().unwrap().terminated_queued.remove(&t.actor)
        {
            return;
        }
        self.st.lock().unwrap().sender = env.sender.clone();
        let msg = env.message;
        let mut handled = true;
        let result = self.with_actor(|a, ctx| handled = a.receive(ctx, &msg));
        self.st.lock().unwrap().sender = None;
        if let Err(cause) = result {
            self.handle_invoke_failure(cause, Some(msg));
        } else if !handled && msg.is::<Terminated>() {
            // `unhandled(Terminated)` throws `DeathPactException`
            let dead = msg
                .downcast_ref::<Terminated>()
                .map(|t| format!("{:?}", t.actor))
                .unwrap_or_default();
            let cause = Throwable::new(
                "org.apache.pekko.actor.DeathPactException",
                Some(&format!("Monitored actor [{dead}] terminated")),
            );
            self.handle_invoke_failure(cause, Some(msg));
        } else if !handled {
            let sender = env
                .sender
                .unwrap_or_else(|| self.system.dead_letters.clone());
            self.system.events.publish_unhandled(UnhandledMessage {
                message: msg,
                sender,
                recipient: self.self_ref(),
            });
        }
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
            self.handle_invoke_failure(failure, None);
            return;
        }
        // `newActor()` runs the user's factory: a throw there is an `ActorInitializationException`, like one in `preStart`
        let actor = match catch_unwind(AssertUnwindSafe(|| self.props.new_actor())) {
            Ok(actor) => actor,
            Err(payload) => {
                let cause = self.creation_failure(&throwable_of(payload));
                self.handle_invoke_failure(cause, None);
                return;
            }
        };
        self.install_actor(actor);
        if let Err(root_cause) = self.with_actor(|a, ctx| a.pre_start(ctx)) {
            // the half-built actor is dropped (so `postStop` never runs) and the supervisor decides (default: stop)
            self.remove_actor();
            let cause = self.creation_failure(&root_cause);
            self.handle_invoke_failure(cause, None);
        }
    }

    /// `ActorInitializationException(self, "exception during creation, root cause message: [..]", cause)` (`ActorCell.create`).
    fn creation_failure(&self, root_cause: &Throwable) -> Throwable {
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
    }

    /// `PostRestartException(self, e, originalCause)`: its message names the class of the failure that caused the restart.
    fn post_restart_failure(&self, original_cause: &Throwable) -> Throwable {
        Throwable::new(
            "org.apache.pekko.actor.PostRestartException",
            Some(&format!(
                "{}: exception post restart (class {})",
                self.path, original_cause.class
            )),
        )
    }

    /// `supervisorStrategy` of the current instance (the default one while there is none, e.g. during a restart).
    fn current_strategy(self: &Arc<Self>) -> SupervisorStrategy {
        let actor = self.st.lock().unwrap().actor.take();
        let strategy = match &actor {
            Some(a) => {
                catch_unwind(AssertUnwindSafe(|| a.supervisor_strategy())).unwrap_or_default()
            }
            None => OneForOneStrategy::default_strategy().into(),
        };
        if let Some(a) = actor {
            self.st.lock().unwrap().actor = Some(a);
        }
        strategy
    }

    /// `handleFailure`: ask the strategy's decider what to do with the failed child and apply it to that child only.
    fn handle_failure(self: &Arc<Self>, child: ActorRef, cause: Throwable) {
        if self.children.lock().unwrap().get_by_ref(&child).is_none() {
            return; // not (or no longer) our child: the failure is dropped
        }
        let strategy = self.current_strategy();
        // a decider that throws is handled like `Escalate` (Pekko: `throw f.cause` inside `handleFailure`): this actor fails with
        // the exception the decider threw, and the child that failed is not suspended again
        let decision = match catch_unwind(AssertUnwindSafe(|| strategy.decide(&cause))) {
            Ok(decision) => decision,
            Err(payload) => {
                self.fail_while_deciding(child, cause, throwable_of(payload));
                return;
            }
        };
        match decision {
            Directive::Resume => child.send_system(SystemMessage::Resume { cause: Some(cause) }),
            Directive::Restart => self.process_failure(true, child, cause, &strategy),
            Directive::Stop => self.process_failure(false, child, cause, &strategy),
            Directive::Escalate => self.escalate(child, cause),
        }
    }

    /// `handleInvokeFailure` after the exception `thrown` escaped `handleFailure(Failed(child, cause))`.
    fn fail_while_deciding(self: &Arc<Self>, child: ActorRef, cause: Throwable, thrown: Throwable) {
        let failed: AnyMessage = Arc::new(Failed {
            child: child.clone(),
            cause,
        });
        self.fail(thrown, Some(failed), Some(child));
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
        let message: AnyMessage = Arc::new(Failed {
            child: child.clone(),
            cause: cause.clone(),
        });
        self.fail(cause, Some(message), Some(child));
    }

    /// `faultSuspend`: suspend this actor and, recursively, its children (the failed parent's whole subtree waits for the decision).
    fn fault_suspend(self: &Arc<Self>) {
        self.status.suspend();
        for k in self.children.lock().unwrap().children() {
            k.send_system(SystemMessage::Suspend);
        }
    }

    /// `faultResume`: the same instance carries on; the failed message is already gone. `resumeChildren`: the child whose failure was escalated gets
    /// the cause back (it clears its own failure), every other child only loses one suspension. `cause` is `None` for an actor that was merely suspended
    /// along with its failed parent.
    fn fault_resume(self: &Arc<Self>, cause: Option<Throwable>) {
        if self.has_no_instance() {
            // there is no instance to resume (it never got created): `Resume` becomes `Create`
            self.change_into_create("Resume", cause.as_ref());
            return;
        }
        let perp = self.st.lock().unwrap().perpetrator.clone();
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
            // the actor is shutting down anyway: only keep the suspend counter balanced
            self.resume_after_failure(true);
            return;
        }
        let message = self.st.lock().unwrap().failed_message.clone();
        let _ = self.with_actor(|a, ctx| a.pre_restart(ctx, &cause, message.as_ref())); // failures here are only logged in Pekko
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
        report_dispatch_failure(
            &self.system.events,
            ErrorEvent {
                cause: Throwable::new("org.apache.pekko.event.Logging$Error$NoCause$", None),
                log_source: self.path.to_string(),
                log_class: reporter_class(self.st.lock().unwrap().actor_class),
                message: format!("changing {directive} into Create after {cause}"),
            },
        );
        self.fault_create();
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
            st.failed = false;
            st.failed_message = None;
            st.perpetrator = None;
        }
    }

    /// `finishRecreate`: resume the mailbox, create the fresh instance and run its `postRestart`.
    fn finish_recreate(self: &Arc<Self>, cause: Throwable) {
        self.resume_after_failure(true);
        let replaced = self.st.lock().unwrap().replaced_actor.take();
        let fresh = match catch_unwind(AssertUnwindSafe(|| self.props.new_actor())) {
            Ok(fresh) => fresh,
            Err(_) => {
                // the old instance stays in place, so it is stopped (and post-stopped) with the cell
                self.st.lock().unwrap().actor = replaced;
                let failure = self.post_restart_failure(&cause);
                self.handle_invoke_failure(failure, None);
                return;
            }
        };
        self.install_actor(fresh);
        if self
            .with_actor(|a, ctx| a.post_restart(ctx, &cause))
            .is_err()
        {
            // the new instance stays in place (Pekko keeps `_actor`): it gets its `postStop` when the actor stops
            let failure = self.post_restart_failure(&cause);
            self.handle_invoke_failure(failure, None);
        }
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
            }
        } else {
            *self.children.lock().unwrap() = Children::Terminated; // setTerminated()
            self.finish_terminate();
        }
    }

    /// `FaultHandling.finishTerminate` — the order of the steps matters.
    fn finish_terminate(self: &Arc<Self>) {
        let _ = self.with_actor(|a, ctx| a.post_stop(ctx)); // post-stop failures are only logged in Pekko
        self.remove_actor();
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
                {
                    let (system, parent) = (self.system.clone(), self.self_ref());
                    Box::new(move |outer: &ActorRef| {
                        // `point(catchFailures = true)`: what `newCell` throws is not lost, the actor fails to initialise instead
                        make_cell(&system, props.clone(), parent.clone(), path.clone(), outer)
                            .unwrap_or_else(|_failure| {
                                ActorCell::new_with_failure(
                                    &system,
                                    props,
                                    parent,
                                    path,
                                    outer.clone(),
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
            report_dispatch_failure(
                &self.system.events,
                ErrorEvent {
                    cause: Throwable::new("org.apache.pekko.event.Logging$Error$NoCause$", None),
                    log_source: self.path.to_string(),
                    log_class: reporter_class(self.st.lock().unwrap().actor_class),
                    message: format!(
                        "received Supervise from unregistered child {child:?}, this will not end well"
                    ),
                },
            );
        } else if is_async && let Some(repointable) = child.0.as_repointable() {
            // `handleSupervise` → `point`: `cell.start()` is outside `catchFailures`, so a rejected attach fails this actor
            // (`systemInvoke`'s `handleInvokeFailure(Nil, e)`), not the new one
            if repointable.point().is_err() {
                self.handle_invoke_failure(rejected_execution(), None);
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

    /// `context.watch(subject)`.
    pub(crate) fn watch(self: &Arc<Self>, subject: &ActorRef) {
        let me = self.self_ref();
        if *subject == me {
            return;
        }
        let newly = {
            let mut st = self.st.lock().unwrap();
            if st.watching.contains(subject) {
                false
            } else {
                st.watching.push(subject.clone());
                true
            }
        };
        if newly {
            subject.send_system(SystemMessage::Watch {
                watchee: subject.clone(),
                watcher: me,
            });
        }
    }

    /// `context.unwatch(subject)`: also cancels a `Terminated` that is already queued.
    pub(crate) fn unwatch(self: &Arc<Self>, subject: &ActorRef) {
        let me = self.self_ref();
        if *subject != me {
            let was = {
                let mut st = self.st.lock().unwrap();
                let before = st.watching.len();
                st.watching.retain(|w| w != subject);
                st.watching.len() != before
            };
            if was {
                subject.send_system(SystemMessage::Unwatch {
                    watchee: subject.clone(),
                    watcher: me,
                });
            }
        }
        self.st.lock().unwrap().terminated_queued.remove(subject);
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
            self.watch(&watchee);
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
        let was_watching = {
            let mut st = self.st.lock().unwrap();
            let before = st.watching.len();
            st.watching.retain(|w| w != &actor);
            st.watching.len() != before
        };
        if was_watching && !self.children.lock().unwrap().is_terminating() {
            self.st
                .lock()
                .unwrap()
                .terminated_queued
                .insert(actor.clone());
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
        let watching = {
            let mut st = self.st.lock().unwrap();
            st.terminated_queued.clear();
            std::mem::take(&mut st.watching)
        };
        let me = self.self_ref();
        for w in watching {
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
}
