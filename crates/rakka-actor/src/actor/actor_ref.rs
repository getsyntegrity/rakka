// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala (Apache-2.0).
//! `ActorRef` and the internal reference kinds of a local system.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::actor::ActorPath;
use crate::actor::props::AnyMessage;
use crate::dispatch::sysmsg::SystemMessage;

/// `InternalActorRef`: what the runtime can do with any reference.
pub(crate) trait InternalActorRef: Send + Sync {
    fn path(&self) -> &ActorPath;
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>);
    fn send_system(&self, msg: SystemMessage);
    /// `stop()`: asks the actor to terminate (a `Terminate` system message).
    fn stop(&self);
    /// `isTerminated` (internal API in Pekko): the mailbox is closed.
    fn is_terminated(&self) -> bool;
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

impl std::fmt::Debug for ActorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actor[{}]", self.path())
    }
}
