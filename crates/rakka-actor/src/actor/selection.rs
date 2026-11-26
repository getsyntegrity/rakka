// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRefProvider.scala and ActorRef.scala (Apache-2.0).
//! `ActorSelection`: a path into the actor tree, looked up each time a message is sent through it.
//!
//! Not ported yet: the `*` and `?` patterns (such an element is looked up as a name), `resolveOne`, equality and the string
//! forms, `actorSelection(ActorPath)`, `/temp`, and death watch on the recipient of a missed selection. A `#uid` that is no
//! number throws in Pekko and matches nothing here (Q-REF-8, open).

use std::sync::{Arc, Weak};

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::path::{actor_path_extractor, split_name_and_uid};
use crate::actor::props::{ActorContext, ActorIdentity, AnyMessage, Identify};
use crate::actor::system::{SystemInner, is_dead_letter_suppressed};
use crate::actor::{ActorPath, relative_actor_path_elements};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::DeadLetter;

/// `SelectionPathElement`.
#[derive(Clone)]
enum Element {
    /// `SelectChildName`
    Name(String),
    /// `SelectParent`
    Parent,
}

/// `org.apache.pekko.actor.ActorSelection`, made by `actor_selection` of an [`ActorSystem`](crate::actor::system::ActorSystem) or
/// an [`ActorContext`]. Nothing is looked up until a message is sent. A message that reaches no actor becomes a [`DeadLetter`]
/// whose recipient is the selected path, or `deadLetters` for a URI of another system, unparseable text and anything under
/// `/deadLetters`; an [`Identify`] is answered with an [`ActorIdentity`] without a reference.
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
            // `deadLetters` is the one reference without a cell that a walk can stand on
            if current == self.system.dead_letters {
                return answer_not_found(&self.system, current, msg, sender);
            }
            let next = match element {
                Element::Parent => self.parent_of(&current),
                Element::Name(name) => self.child_of(&current, name),
            };
            let Some(next) = next else {
                return answer_not_found(&self.system, self.empty_ref(), msg, sender);
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

    /// The `EmptyLocalActorRef` of a miss: the anchor's path followed by every element of the selection.
    fn empty_ref(&self) -> ActorRef {
        let anchor = self.anchor.path().clone();
        let path = self.elements.iter().fold(anchor, |path, element| {
            let name = match element {
                Element::Name(name) => name,
                Element::Parent => "..",
            };
            // `path / name` reads a `#uid`
            path.child(name).unwrap_or_else(|_| path.new_child(name, 0))
        });
        ActorRef(Arc::new_cyclic(|me| EmptyLocalActorRef {
            path,
            system: Arc::downgrade(&self.system),
            me: me.clone(),
        }))
    }
}

/// What `EmptyLocalActorRef` and `DeadLetterActorRef` do with a selection that ends at them: an `Identify` is answered with no
/// actor and no sender; anything else is a dead letter addressed to `recipient`. For a suppressed message Pekko publishes a
/// `SuppressedDeadLetter`, which is not ported: nothing is published (Q-REF-10, open).
fn answer_not_found(
    system: &SystemInner,
    recipient: ActorRef,
    msg: AnyMessage,
    sender: Option<ActorRef>,
) {
    if let (Some(identify), Some(asker)) = (msg.downcast_ref::<Identify>(), &sender) {
        let identity = ActorIdentity {
            correlation_id: identify.message_id.clone(),
            actor_ref: None,
        };
        asker.tell_any(Arc::new(identity), None);
    } else if !is_dead_letter_suppressed(&msg) {
        // (without a sender Pekko throws on an `Identify`; here it is published: Q-REF-9, open)
        system.events.publish_dead_letter(DeadLetter {
            message: msg,
            sender: sender.unwrap_or_else(|| system.dead_letters.clone()),
            recipient,
        });
    }
}

/// `EmptyLocalActorRef`: the recipient of a missed selection.
struct EmptyLocalActorRef {
    path: ActorPath,
    system: Weak<SystemInner>,
    me: Weak<EmptyLocalActorRef>,
}

impl InternalActorRef for EmptyLocalActorRef {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        let (Some(system), Some(me)) = (self.system.upgrade(), self.me.upgrade()) else {
            return;
        };
        // a `DeadLetter` is never published again (that would loop): only an `Identify` inside it is answered
        let (msg, sender) = match msg.downcast_ref::<DeadLetter>() {
            Some(dead) if dead.message.is::<Identify>() => {
                (dead.message.clone(), Some(dead.sender.clone()))
            }
            Some(_) => return,
            None => (msg, sender),
        };
        answer_not_found(&system, ActorRef(me), msg, sender);
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        true
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
