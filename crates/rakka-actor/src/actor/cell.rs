// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/DeathWatch.scala, actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala (Apache-2.0).
//! `ActorCell`: the engine behind a local actor — mailbox, system-message processing, children,
//! death watch and the termination sequence. Follows `ActorCell.scala` and `dungeon/*`.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::children::{Children, SuspendReason};
use crate::actor::fault::Throwable;
use crate::actor::props::{Actor, ActorContext, AnyMessage, Kill, PoisonPill, Props};
use crate::actor::supervision::{Directive, OneForOneStrategy, RestartStats, SupervisorStrategy};
use crate::actor::system::SystemInner;
use crate::actor::validate_path_element;
use crate::actor::{ActorError, ErrorClass};
use crate::dispatch::dispatcher::Dispatcher;
use crate::dispatch::mailbox_close::enqueue_unless_closed;
use crate::dispatch::mailbox_status::MailboxStatus;
use crate::dispatch::message_queue::{Envelope, MessageQueue, UnboundedMailbox};
use crate::dispatch::sysmsg::{Failed, SystemMessage, Terminated};
use crate::dispatch::system_queue::SystemQueue;
use crate::event::UnhandledMessage;
use crate::util::helpers::base64_append;

#[derive(Default)]
struct CellState {
    actor: Option<Box<dyn Actor>>,
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
    /// the atomic mailbox state (Open/Closed, Scheduled, suspend count)
    status: MailboxStatus,
    system_queue: SystemQueue<SystemMessage>,
    /// the user messages (`Mailbox.messageQueue`): the default `UnboundedMailbox` queue
    queue: Box<dyn MessageQueue>,
    children: Mutex<Children>,
    name_counter: AtomicU64,
    st: Mutex<CellState>,
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
        let cell = Arc::new_cyclic(|me| ActorCell {
            path,
            system: system.clone(),
            parent: parent.clone(),
            props,
            dispatcher,
            me: me.clone(),
            status: {
                let status = MailboxStatus::new();
                status.set_as_scheduled(); // not attached yet: nothing runs before `start`
                status
            },
            system_queue: SystemQueue::new(),
            queue: UnboundedMailbox.create(),
            children: Mutex::new(Children::Empty),
            name_counter: AtomicU64::new(0),
            st: Mutex::new(CellState::default()),
        });
        assert!(
            cell.system_queue.enqueue(SystemMessage::Create).is_ok(),
            "a new mailbox accepts system messages"
        );
        parent.send_system(SystemMessage::Supervise {
            child: cell.self_ref(),
            is_async: false,
        });
        cell
    }

    /// `ActorCell.start()`: attach to the dispatcher.
    pub(crate) fn start(self: &Arc<Self>) {
        self.status.set_as_idle();
        self.register_for_execution(false, false);
    }

    pub(crate) fn self_ref(&self) -> ActorRef {
        ActorRef(Arc::new(LocalActorRef(
            self.me.upgrade().expect("cell alive"),
        )))
    }

    // ---- dispatch ----

    /// `MessageDispatcher.registerForExecution`: schedules a turn when the state allows it. `set_as_scheduled` is the only door,
    /// so at most one turn runs at a time.
    fn register_for_execution(self: &Arc<Self>, message_hint: bool, system_message_hint: bool) {
        let can_run = self.status.can_be_scheduled_for_execution(
            message_hint,
            system_message_hint,
            || self.system_queue.has_messages(),
            || self.queue.has_messages(),
        );
        if can_run && self.status.set_as_scheduled() {
            self.dispatcher.execute(self.clone());
        }
    }

    pub(crate) fn system_dispatch(self: &Arc<Self>, msg: SystemMessage) {
        if let Err(msg) = self.system_queue.enqueue(msg) {
            self.dead_system_message(msg); // the queue is closed: the actor terminated
            return;
        }
        self.register_for_execution(false, true);
    }

    fn enqueue_user(self: &Arc<Self>, env: Envelope) {
        let dead = enqueue_unless_closed(&self.status, &*self.queue, env);
        if dead.is_empty() {
            self.register_for_execution(true, false);
        }
        for env in dead {
            self.system
                .dead_letter_mailbox
                .enqueue(&self.self_ref(), env);
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
        cell.register_for_execution(false, false);
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
            SystemMessage::Create => self.create(),
            SystemMessage::Terminate => self.terminate(),
            SystemMessage::Failed { child, cause } => self.handle_failure(child, cause),
            SystemMessage::Recreate { cause } => self.fault_recreate(cause),
            SystemMessage::Resume { cause } => self.fault_resume(cause),
            SystemMessage::Suspend => self.fault_suspend(),
            // The child was already registered by `attach_child` (`initChild`); nothing else to track yet.
            SystemMessage::Supervise { .. } => {}
            SystemMessage::Watch { watchee, watcher } => self.add_watcher(watchee, watcher),
            SystemMessage::Unwatch { watchee, watcher } => self.rem_watcher(watchee, watcher),
            SystemMessage::DeathWatchNotification {
                actor,
                existence_confirmed,
                address_terminated,
            } => self.watched_actor_terminated(actor, existence_confirmed, address_terminated),
        }
    }

    fn create(self: &Arc<Self>) {
        // `newActor()` runs the user's factory: a throw there is an `ActorInitializationException`, like one in `preStart`
        let actor = match catch_unwind(AssertUnwindSafe(|| self.props.new_actor())) {
            Ok(actor) => actor,
            Err(payload) => {
                let cause = self.creation_failure(&throwable_of(payload));
                self.handle_invoke_failure(cause, None);
                return;
            }
        };
        self.st.lock().unwrap().actor = Some(actor);
        if let Err(root_cause) = self.with_actor(|a, ctx| a.pre_start(ctx)) {
            // the half-built actor is dropped (so `postStop` never runs) and the supervisor decides (default: stop)
            self.st.lock().unwrap().actor = None;
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
        self.st.lock().unwrap().actor = Some(fresh);
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
        self.st.lock().unwrap().actor = None;
        // dispatcher.detach: close the mailbox, hand what is left to dead letters
        let leftovers = UnboundedMailbox.create();
        self.status.become_closed();
        self.queue.clean_up(&*leftovers); // `cleanUp`; a producer that raced with the close drains for itself (Q-MBX-8)
        let sys = self.system_queue.clean_up();
        for m in sys {
            self.dead_system_message(m);
        }
        while let Some(env) = leftovers.dequeue() {
            self.system
                .dead_letter_mailbox
                .enqueue(&self.self_ref(), env);
        }
        self.parent
            .send_system(SystemMessage::DeathWatchNotification {
                actor: self.self_ref(),
                existence_confirmed: true,
                address_terminated: false,
            });
        self.tell_watchers_we_died();
        self.unwatch_watched_actors();
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

    /// `attachChild(props, name)` / `attachChild(props)`: create a child of this actor.
    pub(crate) fn attach_child(
        self: &Arc<Self>,
        props: Props,
        name: Option<&str>,
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
        let child = ActorCell::new_local(
            &self.system,
            self.system.dispatcher.clone(),
            props,
            self.self_ref(),
            self.path.new_child(&name, new_uid()),
        );
        let child_ref = child.self_ref();
        self.children.lock().unwrap().add(&name, child_ref.clone()); // initChild
        child.start();
        Ok(child_ref)
    }

    /// `ActorCell.stop(actor)`: mark a child as about to die, then ask the actor to terminate.
    pub(crate) fn stop_actor(&self, actor: &ActorRef) {
        {
            let mut ch = self.children.lock().unwrap();
            if ch.get_by_ref(actor).is_some() {
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
