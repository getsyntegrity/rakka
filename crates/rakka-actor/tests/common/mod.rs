//! Helpers shared by the `classic_*` integration tests (each test binary uses a subset).

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::system::{ActorSystem, Settings};

const WAIT_LIMIT: Duration = Duration::from_secs(10);

/// Polls `condition` every 2 ms and fails the test after 10 s, naming `what` was awaited.
pub fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + WAIT_LIMIT;
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Fixed pause used only to let an *absence* of events show up (nothing can be awaited for).
pub fn settle_for_absence() {
    std::thread::sleep(Duration::from_millis(150));
}

/// A copy of what a shared log holds right now.
pub fn snapshot_of<T: Clone>(log: &Arc<Mutex<Vec<T>>>) -> Vec<T> {
    log.lock().unwrap().clone()
}

/// Owned strings from literals, for comparing against recorded logs.
pub fn owned_strings(literals: &[&str]) -> Vec<String> {
    literals.iter().map(ToString::to_string).collect()
}

/// The full path of `actor_ref` rendered as a string.
pub fn path_of(actor_ref: &ActorRef) -> String {
    actor_ref.path().to_string()
}

/// An `ActorSystem` that is terminated (and awaited) when the test ends, also when an assertion fails,
/// so a failing test does not leak the dispatcher threads.
pub struct RunningSystem(ActorSystem);

impl RunningSystem {
    pub fn create(name: &str) -> Self {
        Self(ActorSystem::create(name).unwrap())
    }

    pub fn create_with_settings(name: &str, settings: Settings) -> Self {
        Self(ActorSystem::create_with_settings(name, settings).unwrap())
    }
}

impl Deref for RunningSystem {
    type Target = ActorSystem;
    fn deref(&self) -> &ActorSystem {
        &self.0
    }
}

impl Drop for RunningSystem {
    fn drop(&mut self) {
        let terminated = self.0.terminate().wait(WAIT_LIMIT);
        if !std::thread::panicking() {
            terminated.expect("system terminated");
        }
    }
}
