// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Props.scala, actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala (Apache-2.0).
//! `Actor`, `Props`, `ActorContext` and the built-in messages (Classic).

use std::any::Any;
use std::sync::Arc;

use crate::ConfigurationException;
use crate::actor::actor_ref::ActorRef;
use crate::actor::cell::ActorCell;
use crate::actor::deploy::{Deploy, NO_MAILBOX_GIVEN};
use crate::actor::fault::Throwable;
use crate::actor::supervision::{OneForOneStrategy, SupervisorStrategy};
use crate::dispatch::mailboxes::Mailboxes;
use crate::dispatch::message_queue::{MailboxType, MessageQueue, UnboundedMailbox};

/// A user-level message (`Any` in Pekko). Messages are immutable and shared, so the same message can
/// be fanned out (event stream, dead letters) without copying.
pub type AnyMessage = Arc<dyn Any + Send + Sync>;

/// `org.apache.pekko.actor.PoisonPill`: stops the actor once all earlier messages were processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoisonPill;

/// `org.apache.pekko.actor.Kill`: makes the actor fail with `ActorKilledException`, which the default strategy answers with a stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kill;

/// `org.apache.pekko.actor.Actor`. Hooks default to no-ops; `receive` returns whether the message was
/// handled (`false` is Pekko's `unhandled`, which publishes an `UnhandledMessage` on the event stream).
pub trait Actor: Send + 'static {
    /// `getClass.getName` of the running instance, as the runtime reports it in `logClass`. A JVM class name has no Rust
    /// equivalent: the default is the type path of the implementor.
    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Called before the first message and after the actor is created (`preStart`); on a restart `post_restart` calls it by default.
    fn pre_start(&mut self, _ctx: &mut ActorContext) {}
    /// Called when the actor stops, for any reason (`postStop`).
    fn post_stop(&mut self, _ctx: &mut ActorContext) {}
    /// `supervisorStrategy`: how this actor handles the failures of its children.
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        OneForOneStrategy::default_strategy().into()
    }
    /// Called on the failed instance before a restart. Default: stop all children, then `post_stop`.
    fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        _reason: &Throwable,
        _message: Option<&AnyMessage>,
    ) {
        for child in ctx.children() {
            ctx.stop(&child);
        }
        self.post_stop(ctx);
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

/// `org.apache.pekko.actor.Props`: how to create an actor instance.
#[derive(Clone)]
pub struct Props {
    factory: Arc<dyn Fn() -> Box<dyn Actor> + Send + Sync>,
    /// `Props.mailbox`: the type that builds the queue of the actor's mailbox; `None` is the default `UnboundedMailbox`.
    mailbox: Option<Arc<dyn MailboxType>>,
    /// `Props.deploy.mailbox`: the id of the mailbox to resolve when the actor is created.
    mailbox_id: Option<String>,
}

impl Props {
    /// Props that build a new instance with `f` every time the actor is created or restarted.
    pub fn new<A: Actor, F: Fn() -> A + Send + Sync + 'static>(f: F) -> Props {
        Props {
            factory: Arc::new(move || Box::new(f())),
            mailbox: None,
            mailbox_id: None,
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

    /// `Mailboxes.getMailboxType` for a selected id: the props whose mailbox is the type registered under it. Props without an id
    /// selection come back unchanged.
    pub(crate) fn resolve_mailbox(
        mut self,
        mailboxes: &Mailboxes,
    ) -> Result<Props, ConfigurationException> {
        if let Some(id) = &self.mailbox_id {
            self.mailbox = Some(mailboxes.lookup(id)?);
        }
        Ok(self)
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

    /// `context.sender()`: the sender of the message being processed, `deadLetters` if there was none.
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

    /// `context.child(name)`.
    pub fn child(&self, name: &str) -> Option<ActorRef> {
        self.cell.child(name)
    }

    /// `context.children`.
    pub fn children(&self) -> Vec<ActorRef> {
        self.cell.children()
    }

    /// `context.watch(actor)`: a `Terminated` message is delivered when `actor` terminates.
    pub fn watch(&mut self, actor: &ActorRef) {
        self.cell.watch(actor);
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
