// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala and actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! System messages (`org.apache.pekko.dispatch.sysmsg`) and the user-level messages the runtime
//! itself sends (`Terminated`, `StopChild`).

use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::Throwable;

#[allow(dead_code)] // `Supervise` fields are stored for parity with Pekko but nothing reads them yet
pub(crate) enum SystemMessage {
    Create,
    Terminate,
    /// a child tells its parent that it failed (`Failed(child, cause, uid)`)
    Failed {
        child: ActorRef,
        cause: Throwable,
    },
    /// the supervisor tells the failed actor to restart (`Recreate(cause)`)
    Recreate {
        cause: Throwable,
    },
    /// the supervisor tells the failed actor to carry on with its current instance (`Resume(causedByFailure)`)
    /// `cause` is `None` for a child that was only suspended along with its failed parent (`resumeChildren`).
    Resume {
        cause: Option<Throwable>,
    },
    /// `Suspend`: a parent that fails suspends its other children until its own supervisor decides
    Suspend,
    Supervise {
        child: ActorRef,
        is_async: bool,
    },
    Watch {
        watchee: ActorRef,
        watcher: ActorRef,
    },
    Unwatch {
        watchee: ActorRef,
        watcher: ActorRef,
    },
    DeathWatchNotification {
        actor: ActorRef,
        existence_confirmed: bool,
        address_terminated: bool,
    },
}

/// `org.apache.pekko.dispatch.sysmsg.Failed`: the message a supervisor was processing when it escalated (`preRestart` receives it).
#[derive(Clone)]
pub struct Failed {
    /// The child that failed.
    pub child: ActorRef,
    /// What the child threw.
    pub cause: Throwable,
}

/// `org.apache.pekko.actor.Terminated`, delivered to watchers as an ordinary message.
#[derive(Clone)]
pub struct Terminated {
    /// The actor that terminated.
    pub actor: ActorRef,
    /// `existenceConfirmed`: the termination was observed, not inferred from a missing actor.
    pub existence_confirmed: bool,
    /// `addressTerminated`: the whole address went away (remoting; always `false` for local actors).
    pub address_terminated: bool,
}

/// Sent to a guardian to make it stop one of its children (`ActorSystem.stop`).
pub(crate) struct StopChild(pub ActorRef);
