// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/sysmsg/SystemMessage.scala and actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! System messages (`org.apache.pekko.dispatch.sysmsg`) and the user-level messages the runtime
//! itself sends (`Terminated`, `StopChild`).

use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::Throwable;

#[allow(dead_code)] // `Supervise` fields are stored for parity with Pekko but nothing reads them yet
pub(crate) enum SystemMessage {
    /// `Create(failure)`: start the actor, or, when the cell could not be made, fail with the `ActorInitializationException` it carries
    Create(Option<Throwable>),
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

/// `ActorCell.calculateState`. The order matters: the cell takes back what it deferred when its state goes down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DeliveryState {
    Default,
    /// the mailbox is suspended: the actor failed, or was suspended with its failed supervisor
    Suspended,
    /// a restart or a retried creation waits for the children to die
    WaitingForChildren,
}

impl SystemMessage {
    /// `shouldStash`: `StashWhenFailed` (`Failed`) while suspended, `StashWhenWaitingForChildren` while the children die.
    pub(crate) fn is_deferred_in(&self, state: DeliveryState) -> bool {
        match state {
            DeliveryState::Default => false,
            DeliveryState::Suspended => matches!(self, Self::Failed { .. }),
            DeliveryState::WaitingForChildren => matches!(
                self,
                Self::Failed { .. } | Self::Recreate { .. } | Self::Suspend | Self::Resume { .. }
            ),
        }
    }
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
#[derive(Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::ActorPath;
    use crate::actor::actor_ref::InternalActorRef;
    use crate::actor::props::AnyMessage;
    use std::sync::Arc;

    /// Stands for the failed child: only the message that carries it is under test.
    struct NoChild;
    impl InternalActorRef for NoChild {
        fn path(&self) -> &ActorPath {
            unreachable!()
        }
        fn tell(&self, _: AnyMessage, _: Option<ActorRef>) {}
        fn send_system(&self, _: SystemMessage) {}
        fn stop(&self) {}
        fn is_terminated(&self) -> bool {
            true
        }
    }

    #[test]
    fn a_suspended_cell_defers_only_failed_and_a_waiting_one_also_recreate_suspend_and_resume() {
        use DeliveryState::{Default, Suspended, WaitingForChildren};
        let failed = SystemMessage::Failed {
            child: ActorRef(Arc::new(NoChild)),
            cause: Throwable::illegal_state("child"),
        };
        let deferred = |m: &SystemMessage| {
            [Default, Suspended, WaitingForChildren].map(|s| m.is_deferred_in(s))
        };
        assert_eq!(deferred(&failed), [false, true, true]);
        assert_eq!(deferred(&SystemMessage::Suspend), [false, false, true]);
        let resume = SystemMessage::Resume { cause: None };
        assert_eq!(deferred(&resume), [false, false, true]);
        assert_eq!(deferred(&SystemMessage::Terminate), [false, false, false]);
    }
}
