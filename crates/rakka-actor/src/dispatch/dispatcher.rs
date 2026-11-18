// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala and actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! The default dispatcher: a fixed pool of threads that run mailboxes.
//! (Pekko's default is a fork-join pool with `parallelism-min = 8`; scheduling policy is not observable.)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::actor::cell::ActorCell;

pub(crate) struct Dispatcher {
    tx: Mutex<Option<Sender<Arc<ActorCell>>>>,
    stopped: AtomicBool,
}

impl Dispatcher {
    pub fn start(system_name: &str, parallelism: usize) -> Arc<Dispatcher> {
        let (tx, rx) = channel::<Arc<ActorCell>>();
        let rx: Arc<Mutex<Receiver<Arc<ActorCell>>>> = Arc::new(Mutex::new(rx));
        for i in 0..parallelism.max(1) {
            let rx = rx.clone();
            thread::Builder::new()
                .name(format!("{system_name}-pekko.actor.default-dispatcher-{i}"))
                .spawn(move || {
                    loop {
                        // The lock is held only while waiting for the next job, never while running one.
                        let next = rx.lock().unwrap().recv();
                        match next {
                            Ok(cell) => ActorCell::run_mailbox(&cell),
                            Err(_) => return, // sender dropped: dispatcher shut down
                        }
                    }
                })
                .expect("spawn dispatcher thread");
        }
        Arc::new(Dispatcher {
            tx: Mutex::new(Some(tx)),
            stopped: AtomicBool::new(false),
        })
    }

    /// `registerForExecution`: queue a mailbox run.
    pub fn execute(&self, cell: Arc<ActorCell>) {
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.send(cell);
        }
    }

    /// Lets the worker threads exit once the queued runs are done.
    pub fn shutdown(&self) {
        if !self.stopped.swap(true, Ordering::SeqCst) {
            self.tx.lock().unwrap().take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_is_idempotent() {
        let d = Dispatcher::start("t-dispatch", 2);
        d.shutdown();
        d.shutdown();
        assert!(
            d.tx.lock().unwrap().is_none(),
            "worker channel closed so the threads can exit"
        );
    }
}
