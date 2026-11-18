//! The event stream (just what slice 2b needs): dead letters and unhandled messages.

use std::sync::{Arc, Mutex};

use crate::identity::actor_ref::ActorRef;
use crate::actor::props::AnyMessage;

/// `org.apache.pekko.actor.DeadLetter`
#[derive(Clone)]
pub struct DeadLetter {
    pub message: AnyMessage,
    pub sender: ActorRef,
    pub recipient: ActorRef,
}

/// `org.apache.pekko.actor.UnhandledMessage`
#[derive(Clone)]
pub struct UnhandledMessage {
    pub message: AnyMessage,
    pub sender: ActorRef,
    pub recipient: ActorRef,
}

/// The event classes a subscriber can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    DeadLetter,
    UnhandledMessage,
}

/// `system.eventStream`.
#[derive(Default)]
pub struct EventStream {
    subscribers: Mutex<Vec<(Channel, ActorRef)>>,
}

impl EventStream {
    pub(crate) fn new() -> Arc<EventStream> {
        Arc::new(EventStream::default())
    }

    /// `eventStream.subscribe(subscriber, channel)`.
    pub fn subscribe(&self, subscriber: &ActorRef, channel: Channel) {
        let mut subs = self.subscribers.lock().unwrap();
        if !subs.iter().any(|(c, s)| *c == channel && s == subscriber) {
            subs.push((channel, subscriber.clone()));
        }
    }

    /// `eventStream.unsubscribe(subscriber)`.
    pub fn unsubscribe(&self, subscriber: &ActorRef) {
        self.subscribers.lock().unwrap().retain(|(_, s)| s != subscriber);
    }

    fn deliver(&self, channel: Channel, event: AnyMessage) {
        let targets: Vec<ActorRef> = {
            let mut subs = self.subscribers.lock().unwrap();
            // dead subscribers are dropped (Pekko unsubscribes terminated subscribers)
            subs.retain(|(_, s)| !s.is_terminated());
            subs.iter().filter(|(c, _)| *c == channel).map(|(_, s)| s.clone()).collect()
        };
        for t in targets {
            t.tell_any(event.clone(), None);
        }
    }

    pub(crate) fn publish_dead_letter(&self, d: DeadLetter) {
        self.deliver(Channel::DeadLetter, Arc::new(d));
    }

    pub(crate) fn publish_unhandled(&self, u: UnhandledMessage) {
        self.deliver(Channel::UnhandledMessage, Arc::new(u));
    }
}
