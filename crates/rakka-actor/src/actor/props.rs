// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Props.scala, actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala (Apache-2.0).
//! `Actor`, `Props`, `ActorContext` and the built-in messages (Classic).

use std::any::Any;
use std::sync::Arc;

use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::{ActorCell, DEFAULT_DISPATCHER_ID};
use crate::actor::deploy::{Deploy, NO_MAILBOX_GIVEN};
use crate::actor::errors::IllegalArgumentError;
use crate::actor::fault::Throwable;
use crate::actor::supervision::{OneForOneStrategy, SupervisorStrategy};
use crate::actor::watching::CustomTermination;
use crate::dispatch::mailboxes::{MailboxSelectionError, Mailboxes};
use crate::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};
use crate::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};

/// A user-level message (`Any` in Pekko). Messages are immutable and shared, so the same message can
/// be fanned out (event stream, dead letters) without copying.
pub type AnyMessage = Arc<dyn Any + Send + Sync>;

/// `org.apache.pekko.actor.PoisonPill`: stops the actor once all earlier messages were processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoisonPill;

/// `org.apache.pekko.actor.Kill`: makes the actor fail with `ActorKilledException`, which the default strategy answers with a stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kill;

/// `org.apache.pekko.actor.Identify`: every actor answers it with an [`ActorIdentity`] without it reaching `receive`.
#[derive(Debug, Clone)]
pub struct Identify {
    /// `messageId`: returned untouched as the correlation id of the reply.
    pub message_id: AnyMessage,
}

/// `org.apache.pekko.actor.ActorIdentity`: the reply to [`Identify`].
#[derive(Debug, Clone)]
pub struct ActorIdentity {
    /// `correlationId`: the `message_id` of the request.
    pub correlation_id: AnyMessage,
    /// `ref`: the actor that answered, `None` when there is none (stopped actor or `deadLetters`).
    pub actor_ref: Option<ActorRef>,
}

/// `org.apache.pekko.actor.Actor`. Hooks default to no-ops; `receive` returns whether the message was
/// handled (`false` is Pekko's `unhandled`, which publishes an `UnhandledMessage` on the event stream).
pub trait Actor: Send + 'static {
    /// `getClass.getName` of the running instance, as the runtime reports it in `logClass`. A JVM class name has no Rust
    /// equivalent: the default is the type path of the implementor.
    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// `RequiresMessageQueue[T]`: the semantics this actor needs of its message queue, which selects its mailbox when neither the props
    /// nor a deployment name one (`Mailboxes.getRequiredType`). Declared on the type, so `Props` know it without making an instance.
    fn required_message_queue() -> Option<MessageQueueSemantics>
    where
        Self: Sized,
    {
        None
    }
    /// Called before the first message and after the actor is created (`preStart`); on a restart `post_restart` calls it by default.
    fn pre_start(&mut self, _ctx: &mut ActorContext) {}
    /// Called when the actor stops, for any reason (`postStop`).
    fn post_stop(&mut self, _ctx: &mut ActorContext) {}
    /// `supervisorStrategy`: how this actor handles the failures of its children.
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        OneForOneStrategy::default_strategy().into()
    }
    /// Called on the failed instance before a restart. Default: [`default_pre_restart`]. An override replaces all of it: one that also
    /// wants the default calls that function.
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        reason: &Throwable,
        message: Option<&AnyMessage>,
    ) {
        default_pre_restart(self, ctx, reason, message);
    }
    /// Called on the new instance after a restart. Default: `pre_start`.
    fn post_restart(&mut self, ctx: &mut ActorContext, _reason: &Throwable) {
        self.pre_start(ctx);
    }
    /// Handles one message and returns whether it was handled; `false` is Pekko's `unhandled`, which publishes an `UnhandledMessage`. A panic inside `receive` is the actor's failure and goes to the supervisor.
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        false
    }
}

/// The default [`Actor::pre_restart`], for an override to call (`super.preRestart`): `unwatch` and `stop` every child, so no termination
/// of those children is told to the new instance, then call the `post_stop` of `actor`, an override of it included.
pub fn default_pre_restart<A: Actor + ?Sized>(
    actor: &mut A,
    ctx: &mut ActorContext,
    _reason: &Throwable,
    _message: Option<&AnyMessage>,
) {
    for child in ctx.children() {
        ctx.unwatch(&child);
        ctx.stop(&child);
    }
    actor.post_stop(ctx);
}

/// `org.apache.pekko.actor.Props`: how to create an actor instance.
#[derive(Clone)]
pub struct Props {
    factory: Arc<dyn Fn() -> Box<dyn Actor> + Send + Sync>,
    /// `Props.mailbox`: the type that builds the queue of the actor's mailbox; `None` is the default `UnboundedMailbox`.
    mailbox: Option<Arc<dyn MailboxType>>,
    /// `Props.deploy.mailbox`: the id of the mailbox to resolve when the actor is created.
    mailbox_id: Option<String>,
    /// `RequiresMessageQueue[T]` of the actor class.
    required_queue: Option<MessageQueueSemantics>,
    /// The class of the actor, which `verifyRequirements` names when its queue does not fit.
    actor_class: &'static str,
}

impl Props {
    /// Props that build a new instance with `f` every time the actor is created or restarted.
    pub fn new<A: Actor, F: Fn() -> A + Send + Sync + 'static>(f: F) -> Props {
        Props {
            factory: Arc::new(move || Box::new(f())),
            mailbox: None,
            mailbox_id: None,
            required_queue: A::required_message_queue(),
            actor_class: std::any::type_name::<A>(),
        }
    }

    /// `Props.withMailbox(id)`: the mailbox is the one registered under `id` in [`Mailboxes`], resolved when the actor is created;
    /// an unknown id makes `actorOf` fail. The empty id is `NoMailboxGiven`: the default mailbox. The last selection wins.
    pub fn with_mailbox_id(mut self, id: impl Into<String>) -> Props {
        self.mailbox = None;
        self.mailbox_id = Some(id.into()).filter(|id| !id.is_empty());
        self
    }

    /// `Props.withMailbox` with the type given directly: the actor's user messages go to the queue `mailbox` creates (for example
    /// `NonBlockingBoundedMailbox`, which never blocks the sender and dead-letters what does not fit). The last selection wins.
    pub fn with_mailbox(mut self, mailbox: impl MailboxType) -> Props {
        self.mailbox = Some(Arc::new(mailbox));
        self.mailbox_id = None;
        self
    }

    /// The props of a guardian: the mailbox type is the one given, whatever the props select (the provider hands the guardians
    /// `mailboxes.lookup(DefaultMailboxId)` directly, without `getMailboxType`).
    pub(crate) fn with_resolved_mailbox(mut self, mailbox: Arc<dyn MailboxType>) -> Props {
        self.mailbox = Some(mailbox);
        self
    }

    /// `Props.mailbox`: the id selected, or the default mailbox id when none was given.
    pub(crate) fn mailbox_id(&self) -> &str {
        self.mailbox_id
            .as_deref()
            .unwrap_or(Mailboxes::DEFAULT_MAILBOX_ID)
    }

    /// `ActorRefProvider.actorOf` (`:692-701`): the mailbox of the deployment found for the path overrides the one of the props
    /// (`props.withMailbox(mbx)`); a deployment without a mailbox leaves them alone.
    pub(crate) fn deployed_over(self, deploy: Option<&Deploy>) -> Props {
        match deploy.map(Deploy::mailbox) {
            Some(mailbox) if mailbox != NO_MAILBOX_GIVEN => self.with_mailbox_id(mailbox),
            _ => self,
        }
    }

    /// `Mailboxes.getMailboxType`: the props whose mailbox is the type registered under the selected id; else the one bound to the
    /// queue semantics the actor requires (or, when that has no usable binding, to the one the dispatcher requires); else the one bound
    /// to the semantics the dispatcher requires; else the one registered under `pekko.actor.default-mailbox`. Props that carry a
    /// mailbox type directly come back unchanged.
    pub(crate) fn resolve_mailbox(
        mut self,
        mailboxes: &Mailboxes,
        dispatcher_requirement: Option<MessageQueueSemantics>,
    ) -> Result<Props, MailboxSelectionError> {
        if self.mailbox.is_none() {
            // `hasMailboxRequirement`: a dispatcher that requires `MessageQueue` requires nothing (`Mailboxes.scala:165`)
            let dispatcher_requirement = dispatcher_requirement
                .filter(|semantics| *semantics != MessageQueueSemantics::MESSAGE_QUEUE);
            let dispatcher_requirement_as_fallback = |error| match dispatcher_requirement {
                Some(semantics) => mailboxes.lookup_by_queue_type(semantics),
                None => Err(error),
            };
            self.mailbox = Some(
                match (
                    &self.mailbox_id,
                    self.required_queue,
                    dispatcher_requirement,
                ) {
                    (Some(_), _, _) => mailboxes.lookup(self.mailbox_id())?,
                    (None, Some(semantics), _) => mailboxes
                        .lookup_by_queue_type(semantics)
                        .or_else(dispatcher_requirement_as_fallback)?,
                    (None, None, Some(semantics)) => mailboxes.lookup_by_queue_type(semantics)?,
                    (None, None, None) => mailboxes.lookup(self.mailbox_id())?,
                },
            );
            if let Some(selected) = self.mailbox.take() {
                self.mailbox = Some(self.verified(selected, mailboxes, dispatcher_requirement)?);
            }
        }
        Ok(self)
    }

    /// `verifyRequirements` of the selected type. Only the binding of the actor's own requirement has a fallback: when it does not
    /// fit, the binding of the dispatcher's requirement is tried and verified (`getMailboxType` catches the failure of the first).
    fn verified(
        &self,
        selected: Arc<dyn MailboxType>,
        mailboxes: &Mailboxes,
        dispatcher_requirement: Option<MessageQueueSemantics>,
    ) -> Result<Arc<dyn MailboxType>, MailboxSelectionError> {
        let unfit = match self.verify_requirements(&*selected, dispatcher_requirement) {
            Ok(()) => return Ok(selected),
            Err(unfit) => unfit,
        };
        match (
            &self.mailbox_id,
            self.required_queue,
            dispatcher_requirement,
        ) {
            (None, Some(_), Some(semantics)) => {
                let fallback = mailboxes.lookup_by_queue_type(semantics)?;
                self.verify_requirements(&*fallback, dispatcher_requirement)?;
                Ok(fallback)
            }
            _ => Err(unfit),
        }
    }

    /// The queue `mailbox` promises must satisfy what the dispatcher requires and what the actor requires (dispatcher first).
    fn verify_requirements(
        &self,
        mailbox: &dyn MailboxType,
        dispatcher_requirement: Option<MessageQueueSemantics>,
    ) -> Result<(), MailboxSelectionError> {
        let produced = mailbox.produced_queue();
        let check = |requirement: Option<MessageQueueSemantics>, owner: String| match requirement {
            Some(required) if !produced.satisfies(required) => Err(
                MailboxSelectionError::IncompatibleQueue(IllegalArgumentError::new(format!(
                    "produced message queue type [{produced}] does not fulfill requirement for {owner}. Must be a subclass of [{required}]."
                ))),
            ),
            _ => Ok(()),
        };
        check(
            dispatcher_requirement,
            format!("dispatcher [{DEFAULT_DISPATCHER_ID}]"),
        )?;
        check(
            self.required_queue,
            format!("actor class [class {}]", self.actor_class),
        )
    }

    /// `Dispatch.init`: a mailbox type that promises its queue is checked again against the queue it really created, when the actor requires
    /// a type. The requirement that queue does not satisfy and its class (`Dispatch.init`: `requires mailbox type [req] got [class]`).
    pub(crate) fn unmet_queue_requirement(
        &self,
        queue: &dyn MessageQueue,
    ) -> Option<(MessageQueueSemantics, &'static str)> {
        let promise = self
            .mailbox
            .as_ref()
            .map(|mailbox| mailbox.produced_queue())
            .filter(|promise| *promise != ProducedQueue::PLAIN);
        let unmet = |required: &MessageQueueSemantics| {
            promise.is_some_and(|promise| {
                !queue
                    .satisfies(*required)
                    .unwrap_or_else(|| promise.satisfies(*required))
            })
        };
        self.required_queue
            .filter(unmet)
            .map(|required| (required, queue.class_name()))
    }

    /// The queue for one mailbox of this actor; what a bounded queue refuses is dead-lettered to `dead_letters`.
    pub(crate) fn new_message_queue(&self, dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        match &self.mailbox {
            Some(mailbox) => mailbox.create(dead_letters),
            None => UnboundedMailbox.create(),
        }
    }

    pub(crate) fn new_actor(&self) -> Box<dyn Actor> {
        (self.factory)()
    }
}

/// `org.apache.pekko.actor.ActorContext`, valid only inside a callback of the owning actor.
pub struct ActorContext<'a> {
    pub(crate) cell: &'a Arc<ActorCell>,
}

impl ActorContext<'_> {
    /// A reference to the actor this context belongs to (`self`).
    pub fn self_ref(&self) -> ActorRef {
        self.cell.self_ref()
    }

    /// `context.sender()`: the sender of `currentMessage`, `deadLetters` if there is none. In a lifecycle hook that can be an earlier
    /// message: one that failed, or the `Failed` of a child that was already handled.
    pub fn sender(&self) -> ActorRef {
        self.cell
            .current_sender()
            .unwrap_or_else(|| self.cell.system.dead_letters.clone())
    }

    /// `context.actorOf(props)`: a child with a generated name.
    pub fn actor_of(&mut self, props: Props) -> Result<ActorRef, crate::actor::ActorError> {
        self.cell.attach_child(props, None, false)
    }

    /// `context.actorOf(props, name)`.
    pub fn actor_of_named(
        &mut self,
        props: Props,
        name: &str,
    ) -> Result<ActorRef, crate::actor::ActorError> {
        self.cell.attach_child(props, Some(name), false)
    }

    /// `context.actorSelection(path)`: a relative path starts at this actor.
    pub fn actor_selection(&self, path: &str) -> crate::actor::selection::ActorSelection {
        crate::actor::selection::ActorSelection::new(&self.cell.system, &self.self_ref(), path)
    }

    /// `context.child(name)`.
    pub fn child(&self, name: &str) -> Option<ActorRef> {
        self.cell.child(name)
    }

    /// `context.children`.
    pub fn children(&self) -> Vec<ActorRef> {
        self.cell.children()
    }

    /// `context.watch(actor)`: a `Terminated` message is delivered when `actor` terminates.
    ///
    /// Throws (`fault::throw`) `java.lang.IllegalStateException` if `actor` is watched with [`watch_with`](Self::watch_with).
    pub fn watch(&mut self, actor: &ActorRef) {
        self.cell.watch(actor, None);
    }

    /// `context.watchWith(actor, message)`: `message` is delivered through `receive`, as an ordinary message with `actor` as sender,
    /// when `actor` terminates. Watching oneself does nothing.
    ///
    /// Registering again is accepted only with an equal message; otherwise, or if `actor` is watched with [`watch`](Self::watch), it
    /// throws (`fault::throw`) `java.lang.IllegalStateException` and the first registration stays. Call
    /// [`unwatch`](Self::unwatch) first to change it.
    ///
    /// Equality is that of the concrete Rust type and value (`PartialEq`), which differs from Scala's `==`: `1_i32` and `1_i64`, or
    /// `"a"` and `String::from("a")`, are different messages, and a value that is not equal to itself (`NaN`) is never accepted twice.
    /// The exception text shows `String` and `&str` messages by value and any other type by its Rust type name.
    pub fn watch_with<M: Any + PartialEq + Send + Sync>(&mut self, actor: &ActorRef, message: M) {
        self.cell
            .watch(actor, Some(CustomTermination::new(message)));
    }

    /// `context.unwatch(actor)`: no `Terminated` for `actor` is delivered after this returns.
    pub fn unwatch(&mut self, actor: &ActorRef) {
        self.cell.unwatch(actor);
    }

    /// `context.stop(actor)`.
    pub fn stop(&mut self, actor: &ActorRef) {
        self.cell.stop_actor(actor);
    }

    /// `target.forward(msg)`: sends keeping the original sender.
    pub fn forward(&self, target: &ActorRef, msg: AnyMessage) {
        target.tell_any(msg, self.cell.current_sender());
    }
}
