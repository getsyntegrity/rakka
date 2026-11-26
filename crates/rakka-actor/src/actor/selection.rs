// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala and ActorRefProvider.scala (Apache-2.0).
//! `ActorSelection`: a path into the actor tree, looked up each time a message is sent through it.
//!
//! Not ported yet: what a selection that matches nothing does (the message goes to `deadLetters` itself for now), the `*` and `?`
//! patterns (such an element is looked up as a name), `resolveOne`, equality and the string forms, `actorSelection(ActorPath)`,
//! `#uid` fragments and `/temp`.

use std::sync::Arc;

use crate::actor::actor_ref::ActorRef;
use crate::actor::props::{ActorContext, AnyMessage};
use crate::actor::system::SystemInner;
use crate::actor::{ActorPath, relative_actor_path_elements};

/// `SelectionPathElement`.
#[derive(Clone)]
enum Element {
    /// `SelectChildName`
    Name(String),
    /// `SelectParent`
    Parent,
}

/// `org.apache.pekko.actor.ActorSelection`, made by `actor_selection` of an [`ActorSystem`](crate::actor::system::ActorSystem) or
/// an [`ActorContext`]. Nothing is looked up until a message is sent.
#[derive(Clone)]
pub struct ActorSelection {
    system: Arc<SystemInner>,
    anchor: ActorRef,
    elements: Vec<Element>,
}

impl ActorSelection {
    /// `ActorRefFactory.actorSelection(path)`: a relative path starts at `lookup_root`, one that begins with `/` or is a URI of
    /// this system at the root guardian; a URI of another system, or text that is no path, selects under `deadLetters`.
    pub(crate) fn new(system: &Arc<SystemInner>, lookup_root: &ActorRef, path: &str) -> Self {
        let root = system.root_guardian();
        let dead_letters = system.dead_letters.clone();
        let (anchor, names) = if let Some(names) = relative_actor_path_elements(path) {
            let absolute = names.first().is_some_and(String::is_empty);
            let anchor = if absolute { root } else { lookup_root.clone() };
            (anchor, names)
        } else if let Ok(parsed) = ActorPath::from_string(path) {
            let ours = parsed.address() == root.path().address();
            let names = parsed.elements().into_iter().map(str::to_owned).collect();
            (if ours { root } else { dead_letters }, names)
        } else {
            (dead_letters, Vec::new())
        };
        let elements = names.into_iter().filter(|name| !name.is_empty());
        ActorSelection {
            system: system.clone(),
            anchor,
            elements: elements
                .map(|name| match name.as_str() {
                    ".." => Element::Parent,
                    _ => Element::Name(name),
                })
                .collect(),
        }
    }

    /// `selection.tell(msg, sender)`; without a sender the receiver sees `deadLetters`.
    pub fn tell<M: std::any::Any + Send + Sync>(&self, msg: M, sender: Option<&ActorRef>) {
        self.tell_any(Arc::new(msg), sender.cloned());
    }

    /// Type-erased [`ActorSelection::tell`].
    pub fn tell_any(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        let mut current = self.anchor.clone();
        for element in &self.elements {
            let next = match element {
                Element::Parent => self.parent_of(&current),
                Element::Name(name) => self.child_of(&current, name),
            };
            let Some(next) = next else {
                return self.system.dead_letters.tell_any(msg, sender);
            };
            current = next;
        }
        current.tell_any(msg, sender);
    }

    /// `selection.forward(msg)`: sends with the sender of the message the context is processing (`deadLetters` if it has none).
    pub fn forward(&self, msg: AnyMessage, context: &ActorContext) {
        self.tell_any(msg, Some(context.sender()));
    }

    fn parent_of(&self, actor: &ActorRef) -> Option<ActorRef> {
        // the root guardian is its own parent
        if *actor == self.system.root_guardian() {
            return Some(actor.clone());
        }
        actor.0.parent()
    }

    fn child_of(&self, actor: &ActorRef, name: &str) -> Option<ActorRef> {
        if name == "deadLetters" && *actor == self.system.root_guardian() {
            return Some(self.system.dead_letters.clone());
        }
        actor.0.single_child(name)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
    use crate::actor::system::ActorSystem;

    const WAIT: Duration = Duration::from_secs(10);

    /// Reports each string it receives; `"hold"` then parks its thread until the gate is signalled.
    struct Recorder(Sender<&'static str>, Arc<Mutex<Receiver<()>>>);

    impl Actor for Recorder {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let message = *msg.downcast_ref::<&str>().unwrap();
            self.0.send(message).unwrap();
            if message == "hold" {
                self.1.lock().unwrap().recv_timeout(WAIT).unwrap();
            }
            true
        }
    }

    fn recorder(gate: &Arc<Mutex<Receiver<()>>>) -> (Props, Receiver<&'static str>) {
        let (heard, hears) = channel();
        let gate = gate.clone();
        let props = Props::new(move || Recorder(heard.clone(), gate.clone()));
        (props, hears)
    }

    #[test]
    fn a_child_that_has_not_started_is_found_and_has_a_parent() {
        let system = ActorSystem::create("unstarted").unwrap();
        let (open, gate) = channel();
        let gate = Arc::new(Mutex::new(gate));
        let (props, parent_hears) = recorder(&gate);
        let parent = system.actor_of_named(props, "p").unwrap();
        parent.tell("hold", None);
        assert_eq!(parent_hears.recv_timeout(WAIT), Ok("hold"));
        // the parent is parked, so it cannot process the `Supervise` that would start this child
        let (props, child_hears) = recorder(&gate);
        parent.attach_child_for_tests(props, "lazy").unwrap();

        let select = |path| system.actor_selection(path);
        select("/user/p/lazy/..").tell("to-parent", None);
        select("/user/p/lazy").tell("to-child", None);
        open.send(()).unwrap();

        assert_eq!(parent_hears.recv_timeout(WAIT), Ok("to-parent"));
        assert_eq!(child_hears.recv_timeout(WAIT), Ok("to-child"));
        let ended = system.terminate().wait(WAIT);
        ended.expect("terminated").expect("terminated normally");
    }
}
