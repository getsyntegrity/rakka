// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/event/EventStream.scala, actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala and actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! The event stream (`EventStream`), reduced to the two channels the Classic runtime publishes on: dead letters
//! and unhandled messages.

use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::actor::actor_ref::ActorRef;
use crate::actor::fault::Throwable;
use crate::actor::props::AnyMessage;

/// `org.apache.pekko.actor.DeadLetter`
#[derive(Clone)]
pub struct DeadLetter {
    /// The undeliverable message.
    pub message: AnyMessage,
    /// Who sent it (`deadLetters` when nobody did).
    pub sender: ActorRef,
    /// The actor it was addressed to.
    pub recipient: ActorRef,
}

/// `org.apache.pekko.actor.UnhandledMessage`
#[derive(Clone)]
pub struct UnhandledMessage {
    /// The message the actor did not handle.
    pub message: AnyMessage,
    /// Who sent it.
    pub sender: ActorRef,
    /// The actor that left it unhandled.
    pub recipient: ActorRef,
}

/// `org.apache.pekko.event.Logging.Error`: an error the runtime reports on the event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorEvent {
    /// The exception (`cause`).
    pub cause: Throwable,
    /// `logSource`: the path of the actor or the class name of the component that reports it.
    pub log_source: String,
    /// `logClass`: the class of the reporter.
    pub log_class: String,
    /// The text of the event.
    pub message: String,
}

/// `org.apache.pekko.event.Logging.Warning`: a warning the runtime reports on the event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarningEvent {
    /// `logSource`: the path of the actor that reports it.
    pub log_source: String,
    /// `logClass`: the class of the reporter.
    pub log_class: String,
    /// The text of the event.
    pub message: String,
}

/// The event classes a subscriber can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Subscribes to [`DeadLetter`] events.
    DeadLetter,
    /// Subscribes to [`UnhandledMessage`] events.
    UnhandledMessage,
    /// Subscribes to [`ErrorEvent`] events.
    Error,
    /// Subscribes to [`WarningEvent`] events.
    Warning,
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
        self.subscribers
            .lock()
            .unwrap()
            .retain(|(_, s)| s != subscriber);
    }

    fn deliver(&self, channel: Channel, event: AnyMessage) {
        let targets: Vec<ActorRef> = {
            let mut subs = self.subscribers.lock().unwrap();
            // dead subscribers are dropped (Pekko unsubscribes terminated subscribers)
            subs.retain(|(_, s)| !s.is_terminated());
            subs.iter()
                .filter(|(c, _)| *c == channel)
                .map(|(_, s)| s.clone())
                .collect()
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

    pub(crate) fn publish_error(&self, e: ErrorEvent) {
        self.deliver(Channel::Error, Arc::new(e));
    }

    pub(crate) fn publish_warning(&self, w: WarningEvent) {
        self.deliver(Channel::Warning, Arc::new(w));
    }
}

thread_local! {
    /// The threads that are publishing a dispatch failure right now (`DispatchFailureReporting.reporting`).
    static REPORTING: Cell<bool> = const { Cell::new(false) };
}

/// `DispatchFailureReporting.report` (`AbstractDispatcher.scala:91-103`): publishing an error tells its subscribers, which can
/// fail to be scheduled and report again from inside the publication. A report made while this thread is already publishing one
/// goes to the standard output (`StandardOutLogger`) instead of the event stream, so it cannot recurse.
pub(crate) fn report_dispatch_failure(events: &EventStream, event: ErrorEvent) {
    if REPORTING.replace(true) {
        println!(
            "[ERROR] [{}] {} ({})",
            event.log_source, event.message, event.cause
        );
        return;
    }
    struct Done;
    impl Drop for Done {
        fn drop(&mut self) {
            REPORTING.set(false); // the `finally { reporting.remove(thread) }`
        }
    }
    let _done = Done;
    events.publish_error(event);
}
