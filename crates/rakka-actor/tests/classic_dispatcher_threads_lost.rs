// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Dispatcher.scala (Apache-2.0).
//! What is left of the default dispatcher once every worker thread is dead: what was queued is dropped, not run and not kept for
//! ever, and later work is refused. Public API only, with real threads; a characterisation of behaviour that already holds.
//!
//! A scheduled send to `deadLetters` drops the user's message on a worker, outside every guard of the runtime: a `Drop` that
//! panics with a payload whose own `Drop` panics ends that thread. The single dispatcher thread makes it the last one.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::panic::panic_any;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use rakka_actor::actor::system::Settings;

const LIMIT: Duration = Duration::from_secs(10);

struct PanicsWhenDropped;

impl Drop for PanicsWhenDropped {
    fn drop(&mut self) {
        panic!("the panic payload of a user");
    }
}

/// Reports that a worker is dropping it, waits until the test lets go and ends that worker.
struct EndsItsThread(Sender<()>, Mutex<Receiver<()>>);

impl Drop for EndsItsThread {
    fn drop(&mut self) {
        self.0.send(()).unwrap();
        self.1.lock().unwrap().recv_timeout(LIMIT).unwrap();
        panic_any(PanicsWhenDropped);
    }
}

/// Reports, when dropped, whether its thread was dying then: only work abandoned by a lost thread is dropped that way.
struct ReportsDrop(Sender<bool>);

impl Drop for ReportsDrop {
    fn drop(&mut self) {
        self.0.send(std::thread::panicking()).unwrap();
    }
}

#[test]
fn losing_every_dispatcher_thread_drops_the_queued_work_and_refuses_more() {
    let settings = Settings {
        parallelism: 1,
        ..Settings::default()
    };
    let system = RunningSystem::create_with_settings("threads-lost", settings);
    let (scheduler, nowhere) = (system.scheduler(), system.dead_letters());
    let (entered, dropping) = channel();
    let (release, released) = channel();
    let ender = EndsItsThread(entered, Mutex::new(released));
    scheduler
        .schedule_once(Duration::ZERO, &nowhere, ender, None)
        .unwrap();
    dropping
        .recv_timeout(LIMIT)
        .expect("the only worker is busy");

    let (queued, abandoned) = channel();
    scheduler
        .schedule_once(Duration::ZERO, &nowhere, ReportsDrop(queued), None)
        .unwrap();
    release.send(()).unwrap();
    assert_eq!(abandoned.recv_timeout(LIMIT), Ok(true));

    // a refused task is dropped before `schedule_once` returns, by the caller, which is not dying
    let (late, refused) = channel();
    scheduler
        .schedule_once(Duration::ZERO, &nowhere, ReportsDrop(late), None)
        .unwrap();
    assert_eq!(refused.try_recv(), Ok(false));

    let terminated = system.terminate().wait(LIMIT);
    terminated.expect("system terminated").expect("normally");
}
