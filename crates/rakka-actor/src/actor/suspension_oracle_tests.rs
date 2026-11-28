// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala and dispatch/Mailbox.scala (Apache-2.0).
//! The suspend counter of failure and recursive suspension (AC-SUP-010), each script compared with the line of
//! `cases/oracle-only/mailbox.golden` that the Pekko 2.0.0 JVM printed (`MBX-OR-03`, `MBX-CC-03`). In the crate because the cases
//! read the mailbox state and send the internal `Suspend`/`Resume`. Integration: real actors and dispatcher.

use crate::actor::actor_ref::ActorRef;
use crate::actor::async_creation_tests::{Log, joined, recorded, wait_until};
use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
use crate::actor::system::ActorSystem;
use crate::dispatch::sysmsg::SystemMessage;
use std::sync::{Arc, Barrier, Mutex};

/// Logs `r:<text>`; `"block"` holds its turn until the gate opens.
struct Worker(Log, Arc<Mutex<()>>);

impl Actor for Worker {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = *msg.downcast_ref::<&str>().unwrap();
        if text == "block" {
            self.0.lock().unwrap().push("entered".into());
            drop(self.1.lock().unwrap());
            self.0.lock().unwrap().retain(|entry| entry != "entered");
        }
        self.0.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

fn worker() -> (ActorSystem, ActorRef, Log, Arc<Mutex<()>>) {
    let system = ActorSystem::create("mbx").unwrap();
    let (log, gate) = (Log::default(), Arc::new(Mutex::new(())));
    let props = {
        let (log, gate) = (log.clone(), gate.clone());
        Props::new(move || Worker(log.clone(), gate.clone()))
    };
    let actor = system.actor_of_named(props, "w").unwrap();
    wait_until("w started", || actor.mailbox_view().starts_with("open"));
    (system, actor, log, gate)
}

/// Waits for `count` suspensions and an idle mailbox, then returns the view the oracle prints.
fn settled(actor: &ActorRef, count: u32) -> String {
    let expected = format!("open scheduled=false suspendCount={count} ");
    wait_until(&expected, || actor.mailbox_view().starts_with(&expected));
    actor.mailbox_view()
}

fn suspend(actor: &ActorRef) {
    actor.send_system(SystemMessage::Suspend);
}

fn resume(actor: &ActorRef) {
    actor.send_system(SystemMessage::Resume { cause: None });
}

#[test]
fn nested_suspensions_need_as_many_resumes() {
    let (_system, actor, log, _) = worker();
    suspend(&actor);
    suspend(&actor);
    actor.tell("u1", None);
    actor.tell("u2", None);
    let suspended = format!("{} log=[{}]", settled(&actor, 2), joined(&log));
    resume(&actor);
    let half = format!("{} log=[{}]", settled(&actor, 1), joined(&log));
    resume(&actor);
    wait_until("both", || log.lock().unwrap().len() == 2);
    let done = format!("{} log=[{}]", settled(&actor, 0), joined(&log));
    let observed = format!("suspended: {suspended} | oneResume: {half} | twoResumes: {done}");
    assert_eq!(observed, recorded("MBX-OR-03"));
}

/// Two threads released together, each running `send` fifty times on `actor`.
fn hammer(actor: &ActorRef, send: fn(&ActorRef)) {
    let go = Barrier::new(2);
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                go.wait();
                (0..50).for_each(|_| send(actor));
            });
        }
    });
}

#[test]
fn concurrent_suspend_and_resume_lose_no_increment() {
    let (_system, actor, log, gate) = worker();
    let held = gate.lock().unwrap();
    actor.tell("block", None);
    wait_until("blocked", || joined(&log) == "entered");
    actor.tell("u", None);
    hammer(&actor, suspend);
    drop(held);
    let suspended = format!("{} log=[{}]", settled(&actor, 100), joined(&log));
    hammer(&actor, resume);
    wait_until("u", || log.lock().unwrap().len() == 2);
    let resumed = format!("{} log=[{}]", settled(&actor, 0), joined(&log));
    let observed = format!("afterSuspends: {suspended} | afterResumes: {resumed}");
    assert_eq!(observed, recorded("MBX-CC-03"));
}
