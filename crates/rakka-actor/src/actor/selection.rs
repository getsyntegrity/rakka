// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala and ActorRefProvider.scala (Apache-2.0).
//! `ActorSelection`: a path into the actor tree, looked up each time a message is sent through it.
//!
//! Not ported yet: what a selection that matches nothing does (the message goes to `deadLetters` itself for now), the `*` and `?`
//! patterns (such an element is looked up as a name), `resolveOne`, equality and the string forms, `actorSelection(ActorPath)`
//! and `/temp`. A `#uid` that is no number throws in Pekko and matches nothing here (Q-REF-8, open).

use std::sync::Arc;

use crate::actor::actor_ref::ActorRef;
use crate::actor::path::{actor_path_extractor, split_name_and_uid};
use crate::actor::props::{ActorContext, AnyMessage};
use crate::actor::relative_actor_path_elements;
use crate::actor::system::SystemInner;

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
        } else if let Some((address, names)) = actor_path_extractor(path) {
            let ours = address == *root.path().address();
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
        // `name#uid` (`getSingleChild`): only that incarnation; uid 0 is any. A uid that is no number matches nothing.
        let (name, uid) = split_name_and_uid(name).ok()?;
        let child = actor.0.single_child(name)?;
        (uid == 0 || uid == child.path().uid()).then_some(child)
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
    struct Recorder(Sender<String>, Arc<Mutex<Receiver<()>>>);

    impl Actor for Recorder {
        fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let message = *msg.downcast_ref::<&str>().unwrap();
            let heard = format!("{} {message}", ctx.self_ref().path().name());
            self.0.send(heard).unwrap();
            if message == "hold" {
                self.1.lock().unwrap().recv_timeout(WAIT).unwrap();
            }
            true
        }
    }

    #[test]
    fn a_child_that_has_not_started_is_found_and_has_a_parent() {
        let system = ActorSystem::create("unstarted").unwrap();
        let (open, gate) = channel();
        let gate = Arc::new(Mutex::new(gate));
        let (heard, hears) = channel();
        let props = Props::new(move || Recorder(heard.clone(), gate.clone()));
        let parent = system.actor_of_named(props.clone(), "p").unwrap();
        parent.tell("hold", None);
        let next = || hears.recv_timeout(WAIT).unwrap();
        assert_eq!(next(), "p hold");
        // the parent is parked, so it cannot process the `Supervise` that would start this child
        parent.attach_child_for_tests(props, "lazy").unwrap();

        let select = |path| system.actor_selection(path);
        select("/user/p/lazy/..").tell("to-parent", None);
        select("/user/p/lazy").tell("to-child", None);
        open.send(()).unwrap();

        let mut released = [next(), next()];
        released.sort();
        assert_eq!(released, ["lazy to-child", "p to-parent"]);
        let ended = system.terminate().wait(WAIT);
        ended.expect("terminated").expect("terminated normally");
    }
}
