//! System messages (`org.apache.pekko.dispatch.sysmsg`) and the user-level messages the runtime
//! itself sends (`Terminated`, `StopChild`).

use crate::actor::fault::Throwable;
use crate::identity::actor_ref::ActorRef;

#[allow(dead_code)] // `Supervise` carries data the supervision slice will consume
pub(crate) enum SystemMessage {
    Create,
    Terminate,
    /// a child tells its parent that it failed (`Failed(child, cause, uid)`)
    Failed { child: ActorRef, cause: Throwable },
    /// the supervisor tells the failed actor to restart (`Recreate(cause)`)
    Recreate { cause: Throwable },
    /// the supervisor tells the failed actor to carry on with its current instance (`Resume(causedByFailure)`)
    /// `cause` is `None` for a child that was only suspended along with its failed parent (`resumeChildren`).
    Resume { cause: Option<Throwable> },
    /// `Suspend`: a parent that fails suspends its other children until its own supervisor decides
    Suspend,
    Supervise { child: ActorRef, is_async: bool },
    Watch { watchee: ActorRef, watcher: ActorRef },
    Unwatch { watchee: ActorRef, watcher: ActorRef },
    DeathWatchNotification { actor: ActorRef, existence_confirmed: bool, address_terminated: bool },
}

/// `org.apache.pekko.dispatch.sysmsg.Failed`: the message a supervisor was processing when it escalated (`preRestart` receives it).
#[derive(Clone)]
pub struct Failed {
    pub child: ActorRef,
    pub cause: Throwable,
}

/// `org.apache.pekko.actor.Terminated`, delivered to watchers as an ordinary message.
#[derive(Clone)]
pub struct Terminated {
    pub actor: ActorRef,
    pub existence_confirmed: bool,
    pub address_terminated: bool,
}

/// Sent to a guardian to make it stop one of its children (`ActorSystem.stop`).
pub(crate) struct StopChild(pub ActorRef);
