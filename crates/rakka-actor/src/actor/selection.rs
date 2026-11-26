// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSelection.scala, ActorRefProvider.scala and ActorRef.scala (Apache-2.0).
//! `ActorSelection`: a path into the actor tree, looked up each time a message is sent through it.
//!
//! An element with `*` or `?` is a pattern over the names of the children. In a string path `?` starts the URI query, which is
//! dropped, so only `*` can be written there. Not ported yet: `resolveOne`, equality and the string forms,
//! `actorSelection(ActorPath)`, `/temp`, and death watch on the recipient of a missed selection. A `#uid` that is no number
//! throws in Pekko and matches nothing here (Q-REF-8, open).

use std::sync::{Arc, Weak};

use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::path::{actor_path_extractor, split_name_and_uid};
use crate::actor::props::{ActorContext, ActorIdentity, AnyMessage, Identify};
use crate::actor::system::{SystemInner, is_dead_letter_suppressed};
use crate::actor::{ActorPath, relative_actor_path_elements};
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::DeadLetter;
use crate::util::glob_matches;

/// `SelectionPathElement`.
#[derive(Clone)]
enum Element {
    /// `SelectChildName`
    Name(String),
    /// `SelectChildPattern`
    Pattern(String),
    /// `SelectParent`
    Parent,
}

/// `org.apache.pekko.actor.ActorSelection`, made by `actor_selection` of an [`ActorSystem`](crate::actor::system::ActorSystem) or
/// an [`ActorContext`]. Nothing is looked up until a message is sent. A message that reaches no actor becomes a [`DeadLetter`]
/// whose recipient is the selected path, or `deadLetters` for a URI of another system, unparseable text and anything under
/// `/deadLetters`; an [`Identify`] is answered with an [`ActorIdentity`] without a reference. A pattern reaches every child it
/// matches. After a single match a miss is addressed below that actor; after two or more it is silent.
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
        Self::from_elements(system, anchor, names)
    }

    /// `ActorSelection(anchorRef, elements)`: empty elements are dropped.
    fn from_elements(
        system: &Arc<SystemInner>,
        anchor: ActorRef,
        names: impl IntoIterator<Item = String>,
    ) -> Self {
        let elements = names.into_iter().filter(|name| !name.is_empty());
        ActorSelection {
            system: system.clone(),
            anchor,
            elements: elements
                .map(|name| match name.as_str() {
                    _ if name.contains(['*', '?']) => Element::Pattern(name),
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
        self.deliver(&self.anchor, &self.elements, msg, sender, false);
    }

    /// `deliverSelection`: walks `elements` from `anchor`. `fanned_out` is set once a pattern matched two or more actors: from
    /// then on a miss is silent.
    fn deliver(
        &self,
        anchor: &ActorRef,
        elements: &[Element],
        msg: AnyMessage,
        sender: Option<ActorRef>,
        fanned_out: bool,
    ) {
        let mut current = anchor.clone();
        for (index, element) in elements.iter().enumerate() {
            // `deadLetters` is the one reference without a cell that a walk can stand on
            if current == self.system.dead_letters {
                return answer_not_found(&self.system, current, msg, sender, fanned_out);
            }
            let next = match element {
                Element::Parent => self.parent_of(&current),
                Element::Name(name) => self.child_of(&current, name),
                Element::Pattern(pattern) => {
                    let mut matching = current.0.children();
                    matching.retain(|child| glob_matches(pattern, child.path().name()));
                    if !matching.is_empty() {
                        let rest = &elements[index + 1..];
                        let fanned_out = fanned_out || matching.len() > 1;
                        for child in matching {
                            // with nothing left this tells the child
                            self.deliver(&child, rest, msg.clone(), sender.clone(), fanned_out);
                        }
                        return;
                    }
                    None
                }
            };
            let Some(next) = next else {
                if !fanned_out {
                    // after a single match the recipient is below that match, not below the selection's anchor
                    let missed = self.empty_ref(anchor, elements);
                    answer_not_found(&self.system, missed, msg, sender, false);
                }
                return;
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

    /// The `EmptyLocalActorRef` of a miss: the anchor's path followed by every element that was to be walked from it.
    fn empty_ref(&self, anchor: &ActorRef, elements: &[Element]) -> ActorRef {
        let path = elements
            .iter()
            .fold(anchor.path().clone(), |path, element| {
                let name = match element {
                    Element::Name(name) | Element::Pattern(name) => name,
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
/// actor and no sender (not at all once the selection fanned out); anything else is a dead letter addressed to `recipient`. For a suppressed message Pekko publishes a
/// `SuppressedDeadLetter`, which is not ported: nothing is published (Q-REF-10, open).
fn answer_not_found(
    system: &SystemInner,
    recipient: ActorRef,
    msg: AnyMessage,
    sender: Option<ActorRef>,
    fanned_out: bool,
) {
    if fanned_out && msg.is::<Identify>() {
        return;
    }
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
        answer_not_found(&system, ActorRef(me), msg, sender, false);
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

    /// A string path cannot carry `?` (it starts the URI query), so the elements are given directly, as `ActorSelection(ref, "b?")`
    /// does; Pekko 2.0.0 was probed through `actorSelection(ActorPath)`.
    #[test]
    fn a_question_mark_matches_exactly_one_character() {
        let system = ActorSystem::create("question").unwrap();
        let (heard, hears) = channel();
        let (_open, gate) = channel();
        let gate = Arc::new(Mutex::new(gate));
        let props = Props::new(move || Recorder(heard.clone(), gate.clone()));
        for name in ["b1", "b2", "b33"] {
            system.actor_of_named(props.clone(), name).unwrap();
        }
        let select = |pattern: &str| {
            let names = ["user".to_owned(), pattern.to_owned()];
            super::ActorSelection::from_elements(&system.0, system.0.root_guardian(), names)
        };
        select("b?").tell("one", None);
        select("b??").tell("two", None);

        let next = || hears.recv_timeout(WAIT).unwrap();
        let mut received = [next(), next(), next()];
        received.sort();
        assert_eq!(received, ["b1 one", "b2 one", "b33 two"]);
        let ended = system.terminate().wait(WAIT);
        ended.expect("terminated").expect("terminated normally");
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
