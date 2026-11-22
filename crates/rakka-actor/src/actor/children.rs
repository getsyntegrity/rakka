// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/ChildrenContainer.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/Children.scala (Apache-2.0).
//! `ChildrenContainer`: the children of an actor and their termination state.
//! Mirrors `dungeon/ChildrenContainer.scala` (Normal / Terminating / Terminated, name reservation).

use std::collections::BTreeMap;

use crate::actor::actor_ref::ActorRef;
use crate::actor::{ActorError, ErrorClass};

#[derive(Clone)]
pub(crate) enum ChildStats {
    /// `ChildNameReserved`
    Reserved,
    Child(ActorRef),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SuspendReason {
    UserRequest,
    Termination,
    /// `Recreation(cause)`: waiting for the children to die before the new instance is created (the cause is kept by the cell)
    Recreation,
}

#[derive(Clone)]
pub(crate) enum Children {
    /// `EmptyChildrenContainer`
    Empty,
    Normal(BTreeMap<String, ChildStats>),
    Terminating {
        c: BTreeMap<String, ChildStats>,
        to_die: Vec<ActorRef>,
        reason: SuspendReason,
    },
    /// `TerminatedChildrenContainer`
    Terminated,
}

fn not_unique(name: &str) -> ActorError {
    ActorError::new(
        ErrorClass::InvalidActorName,
        format!("actor name [{name}] is not unique!"),
    )
}

impl Children {
    pub fn add(&mut self, name: &str, child: ActorRef) {
        match self {
            Children::Empty => {
                *self = Children::Normal(BTreeMap::from([(
                    name.to_string(),
                    ChildStats::Child(child),
                )]))
            }
            Children::Normal(c) | Children::Terminating { c, .. } => {
                c.insert(name.to_string(), ChildStats::Child(child));
            }
            Children::Terminated => {}
        }
    }

    fn map(&self) -> Option<&BTreeMap<String, ChildStats>> {
        match self {
            Children::Normal(c) | Children::Terminating { c, .. } => Some(c),
            _ => None,
        }
    }

    pub fn get_by_ref(&self, actor: &ActorRef) -> Option<ActorRef> {
        match self.map()?.get(actor.path().name()) {
            Some(ChildStats::Child(c)) if c == actor => Some(c.clone()),
            _ => None,
        }
    }

    /// `getByName(name)`: a real child (a reserved name is not one).
    pub fn get_by_name(&self, name: &str) -> Option<ActorRef> {
        match self.map()?.get(name) {
            Some(ChildStats::Child(c)) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn children(&self) -> Vec<ActorRef> {
        self.map()
            .map(|c| {
                c.values()
                    .filter_map(|s| {
                        if let ChildStats::Child(r) = s {
                            Some(r.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `removeChildAndGetStateChange(child)`: the reason the container was waiting for, once its last dying child is gone.
    pub fn remove(&mut self, child: &ActorRef) -> Option<SuspendReason> {
        let name = child.path().name().to_string();
        match self {
            Children::Normal(c) => {
                c.remove(&name);
                if c.is_empty() {
                    *self = Children::Empty;
                }
                None
            }
            Children::Terminating { c, to_die, reason } => {
                to_die.retain(|r| r != child);
                c.remove(&name);
                if !to_die.is_empty() {
                    return None;
                }
                let reason = *reason;
                *self = match reason {
                    SuspendReason::Termination => Children::Terminated,
                    SuspendReason::UserRequest | SuspendReason::Recreation => {
                        let c = std::mem::take(c);
                        if c.is_empty() {
                            Children::Empty
                        } else {
                            Children::Normal(c)
                        }
                    }
                };
                Some(reason)
            }
            Children::Empty | Children::Terminated => None,
        }
    }

    /// `shallDie(actor)`.
    pub fn shall_die(&mut self, actor: &ActorRef) {
        match self {
            Children::Normal(c) => {
                *self = Children::Terminating {
                    c: std::mem::take(c),
                    to_die: vec![actor.clone()],
                    reason: SuspendReason::UserRequest,
                }
            }
            Children::Terminating { to_die, .. } => {
                if !to_die.contains(actor) {
                    to_die.push(actor.clone());
                }
            }
            Children::Empty | Children::Terminated => {}
        }
    }

    /// `initChild(ref)` (`Children.scala:169-179`): the reserved name becomes a real child; a name that is already a child stays as
    /// it is; a name nobody reserved is not registered (false). The child may be gone already when this runs late.
    pub fn init_child(&mut self, child: &ActorRef) -> bool {
        let name = child.path().name();
        match self {
            Children::Normal(c) | Children::Terminating { c, .. } => match c.get(name) {
                Some(ChildStats::Child(_)) => true,
                Some(ChildStats::Reserved) => {
                    c.insert(name.to_string(), ChildStats::Child(child.clone()));
                    true
                }
                None => false,
            },
            Children::Empty | Children::Terminated => false,
        }
    }

    /// `reserve(name)`.
    pub fn reserve(&mut self, name: &str) -> Result<(), ActorError> {
        match self {
            Children::Empty => {
                *self =
                    Children::Normal(BTreeMap::from([(name.to_string(), ChildStats::Reserved)]));
                Ok(())
            }
            Children::Normal(c) => {
                if c.contains_key(name) {
                    return Err(not_unique(name));
                }
                c.insert(name.to_string(), ChildStats::Reserved);
                Ok(())
            }
            Children::Terminating { c, reason, .. } => {
                if *reason == SuspendReason::Termination {
                    return Err(ActorError::new(
                        ErrorClass::IllegalState,
                        format!("cannot reserve actor name '{name}': terminating"),
                    ));
                }
                if c.contains_key(name) {
                    return Err(not_unique(name));
                }
                c.insert(name.to_string(), ChildStats::Reserved);
                Ok(())
            }
            Children::Terminated => Err(ActorError::new(
                ErrorClass::IllegalState,
                format!("cannot reserve actor name '{name}': already terminated"),
            )),
        }
    }

    /// `unreserve(name)`; used when creating a child fails after its name was reserved.
    #[allow(dead_code)]
    pub fn unreserve(&mut self, name: &str) {
        match self {
            Children::Normal(c) => {
                if matches!(c.get(name), Some(ChildStats::Reserved)) {
                    c.remove(name);
                    if c.is_empty() {
                        *self = Children::Empty;
                    }
                }
            }
            Children::Terminating { c, .. } => {
                if matches!(c.get(name), Some(ChildStats::Reserved)) {
                    c.remove(name);
                }
            }
            Children::Empty | Children::Terminated => {}
        }
    }

    /// `setChildrenTerminationReason(Recreation(cause))`: true only if children are already terminating.
    pub fn set_recreation_reason(&mut self) -> bool {
        if let Children::Terminating { reason, .. } = self {
            *reason = SuspendReason::Recreation;
            true
        } else {
            false
        }
    }

    /// `isTerminating`: only a container that waits for children with reason `Termination`.
    pub fn is_terminating(&self) -> bool {
        match self {
            Children::Terminated => true,
            Children::Terminating { reason, .. } => *reason == SuspendReason::Termination,
            _ => false,
        }
    }

    /// `setChildrenTerminationReason(Termination)`: true only if children are already terminating.
    pub fn set_termination_reason(&mut self) -> bool {
        if let Children::Terminating { reason, .. } = self {
            *reason = SuspendReason::Termination;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(r: Result<(), ActorError>) -> (ErrorClass, String) {
        let e = r.unwrap_err();
        (e.class(), e.message().to_string())
    }

    fn not_unique_msg() -> (ErrorClass, String) {
        (
            ErrorClass::InvalidActorName,
            "actor name [a] is not unique!".to_string(),
        )
    }

    #[test]
    fn reserving_a_name_twice_is_rejected_and_unreserve_frees_it() {
        let mut c = Children::Empty;
        c.reserve("a").unwrap();
        c.reserve("b").unwrap();
        assert_eq!(msg(c.reserve("a")), not_unique_msg());
        c.unreserve("a");
        c.reserve("a").unwrap();
        c.unreserve("a");
        c.unreserve("b");
        assert!(
            matches!(c, Children::Empty),
            "last reservation removed returns to the empty container"
        );
    }

    #[test]
    fn unreserve_of_unknown_name_changes_nothing() {
        let mut c = Children::Empty;
        c.unreserve("a");
        assert!(matches!(c, Children::Empty));
        c.reserve("a").unwrap();
        c.unreserve("zzz");
        assert_eq!(msg(c.reserve("a")), not_unique_msg());
    }

    #[test]
    fn terminating_by_user_request_still_reserves_but_by_termination_does_not() {
        let mut c = Children::Terminating {
            c: BTreeMap::new(),
            to_die: vec![],
            reason: SuspendReason::UserRequest,
        };
        c.reserve("a").unwrap();
        assert_eq!(msg(c.reserve("a")), not_unique_msg());
        c.unreserve("a");
        c.reserve("a").unwrap();

        let mut t = Children::Terminating {
            c: BTreeMap::new(),
            to_die: vec![],
            reason: SuspendReason::Termination,
        };
        assert_eq!(
            msg(t.reserve("a")),
            (
                ErrorClass::IllegalState,
                "cannot reserve actor name 'a': terminating".to_string()
            )
        );
        t.unreserve("a");
    }

    #[test]
    fn a_terminated_container_refuses_names() {
        let mut c = Children::Terminated;
        assert_eq!(
            msg(c.reserve("a")),
            (
                ErrorClass::IllegalState,
                "cannot reserve actor name 'a': already terminated".to_string()
            )
        );
        c.unreserve("a");
        assert!(matches!(c, Children::Terminated));
    }
}
