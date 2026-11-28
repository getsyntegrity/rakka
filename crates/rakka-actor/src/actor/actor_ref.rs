// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala (Apache-2.0).
//! `ActorRef` and the internal reference kinds of a local system.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::actor::ActorPath;
use crate::actor::props::AnyMessage;
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::EventStream;

/// `InternalActorRef`: what the runtime can do with any reference.
pub(crate) trait InternalActorRef: Send + Sync {
    fn path(&self) -> &ActorPath;
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>);
    fn send_system(&self, msg: SystemMessage);
    /// `stop()`: asks the actor to terminate (a `Terminate` system message).
    fn stop(&self);
    /// `isTerminated` (internal API in Pekko): the mailbox is closed.
    fn is_terminated(&self) -> bool;
    /// For `deadLetters`, the stream it publishes into; `None` for every other reference. It counts as terminated but
    /// never answers a `Watch` for itself, so the event stream cannot learn of its death and keeps it subscribed.
    fn dead_letters_stream(&self) -> Option<&EventStream> {
        None
    }
    /// `getParent` of a reference with a cell (`ActorRefWithCell`); `None` for the others.
    fn parent(&self) -> Option<ActorRef> {
        None
    }
    /// `getSingleChild(name)`: `None` where Pekko answers `Nobody`.
    fn single_child(&self, _name: &str) -> Option<ActorRef> {
        None
    }
    /// `children` of a reference with a cell, in name order; empty for the others.
    fn children(&self) -> Vec<ActorRef> {
        Vec::new()
    }
    /// The reference as a `RepointableActorRef` when it is one (what `Supervise(async = true)` asks to `point`).
    fn as_repointable(&self) -> Option<&crate::actor::repointable::RepointableActorRef> {
        None
    }
    /// `attachChild(props, name)` on the actor behind this reference, for tests (the cell is not reachable from an `ActorRef`).
    #[cfg(test)]
    fn attach_child_for_tests(
        &self,
        _props: crate::actor::props::Props,
        _name: &str,
    ) -> Result<ActorRef, crate::actor::ActorError> {
        Err(crate::actor::ActorError::new(
            crate::actor::ErrorClass::IllegalState,
            "no cell behind this reference",
        ))
    }
    /// The mailbox state as a string, for tests of the dispatcher (empty for references that have no mailbox).
    #[cfg(test)]
    fn mailbox_view(&self) -> String {
        String::new()
    }
    #[cfg(test)]
    fn has_system_messages(&self) -> bool {
        false
    }
    /// `isFailed` of the cell behind the reference, for tests.
    #[cfg(test)]
    fn is_failed(&self) -> bool {
        false
    }
}

/// `org.apache.pekko.actor.ActorRef`. Equality follows Pekko: same path *and* same uid.
#[derive(Clone)]
pub struct ActorRef(pub(crate) Arc<dyn InternalActorRef>);

impl ActorRef {
    /// The path of the actor this reference points to.
    pub fn path(&self) -> &ActorPath {
        self.0.path()
    }

    /// `ActorRef.isTerminated` (internal API in Pekko, deprecated for users).
    pub fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }

    /// `actor.tell(msg, sender)`; without a sender the receiver sees `deadLetters`.
    pub fn tell<M: std::any::Any + Send + Sync>(&self, msg: M, sender: Option<&ActorRef>) {
        self.0.tell(std::sync::Arc::new(msg), sender.cloned());
    }

    /// Type-erased [`ActorRef::tell`]: the message is already an [`AnyMessage`] and the sender is owned.
    pub fn tell_any(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        self.0.tell(msg, sender);
    }

    pub(crate) fn send_system(&self, msg: SystemMessage) {
        self.0.send_system(msg);
    }

    pub(crate) fn stop(&self) {
        self.0.stop();
    }

    #[cfg(test)]
    pub(crate) fn mailbox_view(&self) -> String {
        self.0.mailbox_view()
    }

    #[cfg(test)]
    pub(crate) fn attach_child_for_tests(
        &self,
        props: crate::actor::props::Props,
        name: &str,
    ) -> Result<ActorRef, crate::actor::ActorError> {
        self.0.attach_child_for_tests(props, name)
    }

    #[cfg(test)]
    pub(crate) fn is_failed(&self) -> bool {
        self.0.is_failed()
    }

    #[cfg(test)]
    pub(crate) fn has_system_messages(&self) -> bool {
        self.0.has_system_messages()
    }
}

impl PartialEq for ActorRef {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (self.path(), other.path());
        a.uid() == b.uid() && a == b
    }
}

impl Eq for ActorRef {}

impl Hash for ActorRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().uid().hash(state);
        self.path().hash(state);
    }
}

/// `ActorRef.toString`: `Actor[path#uid]`, without the `#uid` when the uid is undefined (`ActorRef.scala:186-188`).
impl std::fmt::Debug for ActorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actor[{}]", self.path().to_serialization_format())
    }
}
