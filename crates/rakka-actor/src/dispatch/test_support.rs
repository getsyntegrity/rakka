// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala (Apache-2.0).
//! Test double shared by the dispatch unit tests: an `ActorRef` that only records what it is told.

use crate::actor::ActorPath;
use crate::actor::actor_ref::{ActorRef, InternalActorRef};
use crate::actor::props::AnyMessage;
use crate::dispatch::sysmsg::SystemMessage;
use std::sync::{Arc, Mutex};

/// `deadLetters` (or any actor) stand-in: it records what it is told; the collaborator whose own behaviour is tested in `system.rs`.
pub(crate) struct Recorder {
    path: ActorPath,
    pub(crate) told: Mutex<Vec<(AnyMessage, Option<ActorRef>)>>,
}

impl InternalActorRef for Recorder {
    fn path(&self) -> &ActorPath {
        &self.path
    }
    fn tell(&self, msg: AnyMessage, sender: Option<ActorRef>) {
        self.told.lock().unwrap().push((msg, sender));
    }
    fn send_system(&self, _msg: SystemMessage) {}
    fn stop(&self) {}
    fn is_terminated(&self) -> bool {
        false
    }
}

pub(crate) fn recorder(path: &str) -> (Arc<Recorder>, ActorRef) {
    let recorder = Arc::new(Recorder {
        path: ActorPath::from_string(path).unwrap(),
        told: Mutex::new(Vec::new()),
    });
    let reference = ActorRef(recorder.clone());
    (recorder, reference)
}
