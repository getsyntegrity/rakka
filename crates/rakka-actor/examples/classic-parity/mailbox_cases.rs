// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! Cases of the `Mailbox` oracle (`MailboxOracle.java`) that Rust can answer: the status transitions ST-01 to ST-08 (`MailboxStatus`)
//! the default queue MQ-01 to MQ-03 (`UnboundedMailbox`), and DL-01 (what `deadLetters` answers to a `Watch`, with a real system).
//! Each script drives the Rust type exactly as the oracle drives Pekko's.

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::mailbox_status::MailboxStatus;
use rakka_actor::dispatch::message_queue::{Envelope, MessageQueue, UnboundedMailbox};
use rakka_actor::dispatch::system_queue::SystemQueue;

/// Oracle cases Rust does not produce yet, with the reason: they need pieces of ADR-0001 that arrive in later steps.
#[allow(dead_code)] // read by tests/recorded_pekko.rs
pub const PENDING: &[(&str, &str)] = &[
    (
        "MBX-TK-01 Mailbox is a plain Runnable, not a ForkJoinTask",
        "JVM class property (Mailbox is not a ForkJoinTask): no Rust counterpart",
    ),
    (
        "MBX-OR-01 user messages are FIFO",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-OR-02 system message queued behind user messages runs before them, drained after the turn's message",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-OR-03 suspension keeps user messages pending; nested suspend needs balanced resume",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-OR-04 messages sent while terminating and after termination go to dead letters",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-OR-05 system messages after close go to dead letters (Watch is answered)",
        "the case sends Suspend/Resume, internal system messages the public API cannot send: produced in-crate by dispatch::dead_letter_mailbox (with_real_actors) and compared with the MBX-OR-05 line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 5)",
    ),
    (
        "MBX-CL-01 closed mailbox of a stopped actor",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-CC-01 eight producers on a one-thread pool: no loss, per-sender order, serial execution",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-CC-02 message enqueued at the last empty dequeue of a turn is not stranded",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-CC-03 concurrent Suspend/Resume system messages lose no increments",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-CC-04 dead-letter forwarding during cleanUp",
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
    ),
    (
        "MBX-CC-05 producer that read the mailbox before it closed strands its message",
        "Q-MBX-8 (accepted): Pekko strands the late message, Rust dead-letters it by design; covered by dispatch::mailbox_close and tests/classic_mailbox_close.rs",
    ),
    (
        "MBX-CC-05a producer that enqueues before cleanUp drains the old queue is dead-lettered",
        "Q-MBX-8 (accepted): same dead-letter outcome as Pekko; the forced interleaving is a unit test of dispatch::mailbox_close (the cell does not accept a hook queue yet)",
    ),
    (
        "MBX-RJ-01 one rejection of the executor is retried once and the message is delivered",
        "needs an executor that rejects tasks and internal mailbox state: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 6b)",
    ),
    (
        "MBX-RJ-02 two rejections: Error published, mailbox idle, message kept and delivered by the next send",
        "needs an executor that rejects tasks and internal mailbox state: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 6b)",
    ),
    (
        "MBX-RJ-03 a system message whose scheduling is rejected twice stays queued and runs with the next send",
        "needs an executor that rejects tasks and internal mailbox state: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 6b)",
    ),
    (
        "MBX-RJ-04 a Terminate whose scheduling is rejected twice terminates the actor with the next send; the message is dead-lettered",
        "needs an executor that rejects tasks and internal mailbox state: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 6b)",
    ),
    (
        "MBX-RJ-05 an actor whose attach is rejected twice: actorOf returns, the supervisor fails and its children stop, the new actor never starts",
        "needs an executor that rejects tasks: produced in-crate by dispatch::executor_rejection_tests (through the asynchronous creation of slice B) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C2)",
    ),
    (
        "MBX-RJ-06 a synchronous context.actorOf whose attach is rejected twice: the caller gets the exception and the name stays taken",
        "needs an executor that rejects tasks and a parent actor: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C2)",
    ),
    (
        "MBX-RJ-07 a synchronous context.actorOf whose attach is rejected twice and whose exception escapes: the parent fails and restarts",
        "needs an executor that rejects tasks and a parent actor: produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C2)",
    ),
    (
        "MBX-PT-01 a send that waits for the unstarted cell's lock longer than unstarted-push-timeout is a dead letter with a Warning",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-PT-02 a send that waits for the lock while the hand-over ends before the timeout is delivered after the queued messages",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-PT-03 a system message waits for the lock without a timeout and is delivered once the hand-over ends",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-PT-06 the thread that hands the queue over can send to the same reference: its message joins the queue, not the timeout",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-PT-04 unstarted-push-timeout = 0s: a send that finds the lock held is dropped at once",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-PT-05 unstarted-push-timeout = -1s: a send that finds the lock held is dropped at once",
        "needs the unstarted reference whose hand-over blocks (a real cell stand-in): produced in-crate by actor::repointable (unstarted_push_timeout) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C3)",
    ),
    (
        "MBX-UC-01 messages sent to an unstarted actor: the system message goes first, then the user messages in order",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::repointable (with_real_cells) and, through the real attach_child, by actor::async_creation_tests; both compare with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slices A and B)",
    ),
    (
        "MBX-UC-02 a Watch and a Terminate sent to an unstarted actor are applied once it starts: the watcher gets Terminated",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::repointable (with_real_cells) and, through the real attach_child, by actor::async_creation_tests; both compare with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slices A and B)",
    ),
    (
        "MBX-UC-03 a Watch undone by an Unwatch before the actor starts: the watcher is not told",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::repointable (with_real_cells) and, through the real attach_child, by actor::async_creation_tests; both compare with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slices A and B)",
    ),
    (
        "MBX-UC-04 a child created under a suspended parent inherits the suspension",
        "needs the parent's cell to be suspended and attachChild (internal): produced in-crate by actor::async_creation_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice B)",
    ),
    (
        "MBX-IF-01 the cell of an asynchronously created actor cannot be made: actorOf returns, the supervisor stops the actor, which never starts",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C1)",
    ),
    (
        "MBX-IF-02 messages sent to an actor whose cell cannot be made, before and after it is stopped, are dead letters",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C1)",
    ),
    (
        "MBX-IF-03 a supervisor that restarts on ActorInitializationException: the failed creation becomes a Create and the actor starts",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C1)",
    ),
    (
        "MBX-IF-04 an actor that fails to start after creating a child is restarted as a Create: its children are stopped first",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C1)",
    ),
    (
        "MBX-IF-05 a supervisor that resumes on ActorInitializationException: the failed creation becomes a Create and the actor starts",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C1)",
    ),
    (
        "MBX-AC-01 actorOf returns before the child starts: it is found by name and its name is taken at once",
        "needs attachChild inside the parent's turn and an executor that runs nothing by itself (internal): produced in-crate by actor::async_creation_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice B)",
    ),
    (
        "MBX-SC-03 actor suspended with queued work does not reschedule itself",
        "sends the internal Suspend system message and reads the mailbox state, neither reachable from the public API: produced in-crate by dispatch::dispatcher (turn_limits_with_real_actors) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (ADR-0001 step 6c)",
    ),
    (
        "MBX-UT-01 swapLookup: before the real cell is installed the reference is still unstarted, but a child the actor created in preStart already resolves through the real cell",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-02 an unstarted reference: not started, not terminated, counts its queued user messages, queues Suspend and Resume and hands everything over in order",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-03 a parent that terminates before the Supervise of its child: Supervise is dead-lettered, the child never starts and keeps its queue",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-04 control: a parent stopped after its child started waits for it, and the child dies first",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-05 a parent already waiting for a started child to die ignores the Supervise of a new child: the child is never pointed and the parent still terminates",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-06 a restart with only an unstarted child does not wait for it: the new instance starts first, then the child starts, is terminated",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-07 a restart waiting for a started child: the Supervise of an unstarted child is processed meanwhile, the child starts and terminates, the restart completes when the started child is gone",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-08 senders racing with the installation of the real cell: nothing lost, nothing repeated, per-sender order kept",
        "needs the unstarted reference and the real cell (internal): produced in-crate by tests/classic_unstarted_races.rs and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-09 a child created in preStart while the hand-over is in progress waits for the hand-over to end: the reference's lock is held across replaceWith",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_push_timeout_tests (is_started during the hand-over) and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
];

fn state(status: &MailboxStatus) -> String {
    format!(
        "{} scheduled={} suspendCount={} raw={}",
        if status.is_closed() { "closed" } else { "open" },
        status.is_scheduled(),
        status.suspend_count(),
        status.current_status()
    )
}

fn can_be_scheduled(status: &MailboxStatus, message_hint: bool, system_hint: bool) -> bool {
    status.can_be_scheduled_for_execution(message_hint, system_hint, || false, || false)
}

fn open_to_scheduled() -> String {
    let status = MailboxStatus::new();
    let (first, second) = (status.set_as_scheduled(), status.set_as_scheduled());
    format!("first={first} second={second} {}", state(&status))
}

fn scheduled_to_idle() -> String {
    let status = MailboxStatus::new();
    status.set_as_scheduled();
    let idle = status.set_as_idle();
    let after_idle = state(&status);
    let again = status.set_as_idle();
    let after_again = state(&status);
    format!(
        "idle={idle} {after_idle} | again={again} {after_again} | reschedule={}",
        status.set_as_scheduled()
    )
}

fn suspended_and_resumed() -> String {
    let status = MailboxStatus::new();
    let (suspend1, suspend2) = (status.suspend(), status.suspend());
    let mut line = format!("suspend1={suspend1} suspend2={suspend2} {}", state(&status));
    let resume1 = status.resume();
    line.push_str(&format!(" | resume1={resume1} {}", state(&status)));
    let resume2 = status.resume();
    line.push_str(&format!(" resume2={resume2} {}", state(&status)));
    let at_zero = status.resume();
    line.push_str(&format!(" | resumeAtZero={at_zero} {}", state(&status)));
    line
}

fn suspended_and_scheduled() -> String {
    let status = MailboxStatus::new();
    status.suspend();
    let schedule = status.set_as_scheduled();
    let mut line = format!(
        "schedule={schedule} {} canBe(f,f)={} canBe(f,sys)={}",
        state(&status),
        can_be_scheduled(&status, false, false),
        can_be_scheduled(&status, false, true)
    );
    let resume = status.resume();
    line.push_str(&format!(" | resume={resume} {}", state(&status)));
    line
}

fn open_to_closed() -> String {
    let status = MailboxStatus::new();
    status.set_as_scheduled();
    status.suspend();
    status.suspend();
    let before = state(&status);
    let close1 = status.become_closed();
    let after_close1 = state(&status);
    let close2 = status.become_closed();
    format!(
        "before {before} | close1={close1} {after_close1} close2={close2} {}",
        state(&status)
    )
}

fn closed_ignores_resume_and_suspend() -> String {
    let status = MailboxStatus::new();
    status.become_closed();
    let (resume, suspend) = (status.resume(), status.suspend());
    format!("resume={resume} suspend={suspend} {}", state(&status))
}

fn closed_refuses_scheduling() -> String {
    let status = MailboxStatus::new();
    status.become_closed();
    let (schedule, idle) = (status.set_as_scheduled(), status.set_as_idle());
    format!(
        "schedule={schedule} idle={idle} {} canBe={}",
        state(&status),
        can_be_scheduled(&status, true, true)
    )
}

fn can_be_scheduled_matrix() -> String {
    let mut line = String::new();
    for (name, scheduled, suspended, closed) in [
        ("open", false, false, false),
        ("scheduled", true, false, false),
        ("suspended", false, true, false),
        ("suspended+scheduled", true, true, false),
        ("closed", false, false, true),
    ] {
        let status = MailboxStatus::new();
        if scheduled {
            status.set_as_scheduled();
        }
        if suspended {
            status.suspend();
        }
        if closed {
            status.become_closed();
        }
        line.push_str(name);
        line.push_str(": ");
        for (message_hint, system_hint) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            line.push(if can_be_scheduled(&status, message_hint, system_hint) {
                'T'
            } else {
                'F'
            });
        }
        // the oracle enqueues a system message here; the queue fact is what changes
        let with_system_message =
            status.can_be_scheduled_for_execution(false, false, || true, || false);
        line.push_str(&format!(" withSystemMessage={with_system_message}; "));
    }
    line.trim().to_owned()
}

fn names(names: Vec<&str>) -> String {
    format!("[{}]", names.join(", "))
}

fn system_queue_drains_earliest_first() -> String {
    let queue = SystemQueue::new();
    for message in ["Suspend", "Resume", "Terminate"] {
        queue
            .enqueue(message)
            .expect("an open queue stores the message");
    }
    let has = queue.has_messages();
    let first = names(queue.drain());
    format!(
        "has={has} drain={first} | after has={} drain={}",
        queue.has_messages(),
        names(queue.drain())
    )
}

fn sentinel_closes_the_system_queue() -> String {
    let queue = SystemQueue::new();
    queue
        .enqueue("Suspend")
        .expect("an open queue stores the message");
    let drained = names(queue.clean_up());
    // the oracle's mailbox has no actor, so the late message is dropped there; here `enqueue` hands it back to the caller
    let late = queue.enqueue("Resume");
    assert_eq!(late, Err("Resume"));
    format!(
        "drain={drained} | afterSentinel has={} drain={} (actor==null: dropped, not forwarded)",
        queue.has_messages(),
        names(queue.drain())
    )
}

fn put(queue: &dyn MessageQueue, text: &str) {
    queue.enqueue(Envelope {
        message: Arc::new(text.to_owned()),
        sender: None,
    });
}

fn counts(queue: &dyn MessageQueue) -> String {
    format!(
        "n={} has={}",
        queue.number_of_messages(),
        queue.has_messages()
    )
}

fn drain(queue: &dyn MessageQueue) -> String {
    let texts: Vec<String> = std::iter::from_fn(|| queue.dequeue())
        .map(|e| {
            e.message
                .downcast_ref::<String>()
                .expect("string messages")
                .clone()
        })
        .collect();
    format!("[{}]", texts.join(", "))
}

fn default_queue_is_fifo_and_counts() -> String {
    let queue = UnboundedMailbox.create();
    let mut line = format!("new: {} | enqueue", counts(&*queue));
    for text in ["a", "b", "c"] {
        put(&*queue, text);
        line.push_str(&format!(" {}", counts(&*queue)));
    }
    let drained = drain(&*queue);
    let after_empty = if queue.dequeue().is_none() {
        "null"
    } else {
        "message"
    };
    format!(
        "{line} | dequeue {drained} then {after_empty} | after {}",
        counts(&*queue)
    )
}

fn default_queue_interleaved() -> String {
    let queue = UnboundedMailbox.create();
    put(&*queue, "a");
    put(&*queue, "b");
    let first = queue
        .dequeue()
        .and_then(|e| e.message.downcast_ref::<String>().cloned())
        .expect("a message");
    put(&*queue, "c");
    format!("first={first} rest={}", drain(&*queue))
}

fn clean_up_moves_to_dead_letters() -> String {
    let (queue, dead_letters) = (UnboundedMailbox.create(), UnboundedMailbox.create());
    for text in ["a", "b", "c"] {
        put(&*queue, text);
    }
    queue.clean_up(&*dead_letters);
    format!(
        "source {} | target {} order {}",
        counts(&*queue),
        counts(&*dead_letters),
        drain(&*dead_letters)
    )
}

/// `(case id, line)` for every oracle case of the family that Rust produces.
struct Idle;
impl Actor for Idle {
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        true
    }
}

/// Watches every `ActorRef` it is told about, in order, and logs each `Terminated` it receives.
struct WatchesWhatItIsTold(Arc<Mutex<Vec<String>>>);
impl Actor for WatchesWhatItIsTold {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(target) = msg.downcast_ref::<ActorRef>() {
            ctx.watch(target);
        } else if msg.is::<Terminated>() {
            self.0.lock().unwrap().push("terminated".to_owned());
        }
        true
    }
}

/// `MBX-DL-01`: `deadLetters` ignores a `Watch` of itself, a stopped actor's mailbox answers it.
fn watching_dead_letters_is_not_answered() -> String {
    let system = ActorSystem::create("mbx").expect("system");
    let log = Arc::new(Mutex::new(Vec::new()));
    let stopped = system
        .actor_of_named(Props::new(|| Idle), "w")
        .expect("actor");
    system.stop(&stopped);
    wait_until(|| stopped.is_terminated());
    let watcher = system
        .actor_of(Props::new({
            let log = log.clone();
            move || WatchesWhatItIsTold(log.clone())
        }))
        .expect("watcher");
    watcher.tell(system.dead_letters(), None);
    watcher.tell(stopped, None);
    wait_until(|| !log.lock().unwrap().is_empty());
    std::thread::sleep(Duration::from_millis(300)); // a bounded wait for an absence: no second `terminated` may follow
    let line = format!("watcherLog=[{}]", log.lock().unwrap().join(", "));
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated");
    line
}

/// Occupies the only dispatcher thread until released (the oracle's `Block` message) and logs `r:block` afterwards.
struct Blocker {
    entered: std::sync::mpsc::Sender<()>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
    log: Arc<Mutex<Vec<String>>>,
}
impl Actor for Blocker {
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        self.entered.send(()).expect("entered");
        self.release.lock().unwrap().recv().expect("release");
        self.log.lock().unwrap().push("r:block".to_owned());
        true
    }
}

/// Logs `r:<text>`; a message starting with `sleep` takes 60 ms, like the oracle's `Worker`.
struct Worker(Arc<Mutex<Vec<String>>>);
impl Actor for Worker {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        if text.starts_with("sleep") {
            std::thread::sleep(Duration::from_millis(60));
        }
        self.0.lock().unwrap().push(format!("r:{text}"));
        true
    }
}

/// `MBX-SC-01` / `MBX-SC-02`: one dispatcher thread, a blocked actor, then six messages queued on `x` and six on `y` (`sleep-` prefixed
/// and three each for the deadline case). The order in which the two actors take turns is the observable.
fn turn_interleaving(
    throughput: usize,
    deadline: Duration,
    prefix: &str,
    per_actor: usize,
) -> String {
    let system = ActorSystem::create_with_settings(
        "mbx",
        Settings {
            parallelism: 1,
            throughput,
            throughput_deadline_time: deadline,
            ..Settings::default()
        },
    )
    .expect("system");
    let log = Arc::new(Mutex::new(Vec::new()));
    let (entered_tx, entered_rx) = channel();
    let (release_tx, release_rx) = channel();
    let gate = Mutex::new(Some((entered_tx, release_rx)));
    let blocker = system
        .actor_of(Props::new({
            let log = log.clone();
            move || {
                let (entered, release) = gate.lock().unwrap().take().expect("one instance");
                Blocker {
                    entered,
                    release: Mutex::new(release),
                    log: log.clone(),
                }
            }
        }))
        .expect("blocker");
    let spawn = |name: &str| {
        let log = log.clone();
        system
            .actor_of_named(Props::new(move || Worker(log.clone())), name)
            .expect("worker")
    };
    let (x, y) = (spawn("x"), spawn("y"));
    blocker.tell("block".to_owned(), None);
    entered_rx.recv().expect("blocker running");
    for i in 1..=per_actor {
        x.tell(format!("{prefix}x{i}"), None);
    }
    for i in 1..=per_actor {
        y.tell(format!("{prefix}y{i}"), None);
    }
    release_tx.send(()).expect("release");
    wait_until(|| log.lock().unwrap().len() == 1 + 2 * per_actor);
    let line = format!("[{}]", log.lock().unwrap().join(", "));
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated");
    line
}

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "condition not reached");
        std::thread::sleep(Duration::from_millis(2));
    }
}

pub fn rust_lines() -> Vec<(String, String)> {
    [
        ("MBX-ST-01 open to scheduled", open_to_scheduled()),
        ("MBX-ST-02 scheduled to idle", scheduled_to_idle()),
        (
            "MBX-ST-03 open to suspended and resumed",
            suspended_and_resumed(),
        ),
        (
            "MBX-ST-04 suspended and scheduled",
            suspended_and_scheduled(),
        ),
        ("MBX-ST-05 open to closed", open_to_closed()),
        (
            "MBX-ST-06 closed ignores resume and suspend",
            closed_ignores_resume_and_suspend(),
        ),
        (
            "MBX-ST-07 closed refuses scheduling",
            closed_refuses_scheduling(),
        ),
        (
            "MBX-ST-08 canBeScheduledForExecution matrix",
            can_be_scheduled_matrix(),
        ),
        (
            "MBX-SQ-01 system queue drains earliest first",
            system_queue_drains_earliest_first(),
        ),
        (
            "MBX-SQ-02 sentinel NoMessage closes the system queue",
            sentinel_closes_the_system_queue(),
        ),
        (
            "MBX-SC-01 throughput 2 on one thread interleaves two actors in blocks of two",
            turn_interleaving(2, Duration::ZERO, "", 6),
        ),
        (
            "MBX-SC-02 throughput-deadline-time ends the turn after a slow message",
            turn_interleaving(100, Duration::from_millis(30), "sleep-", 3),
        ),
        (
            "MBX-DL-01 watching deadLetters is not answered, watching a stopped actor is",
            watching_dead_letters_is_not_answered(),
        ),
        (
            "MBX-MQ-01 default queue is FIFO and counts",
            default_queue_is_fifo_and_counts(),
        ),
        (
            "MBX-MQ-02 default queue stays FIFO when enqueue and dequeue interleave",
            default_queue_interleaved(),
        ),
        (
            "MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order",
            clean_up_moves_to_dead_letters(),
        ),
    ]
    .into_iter()
    .map(|(id, result)| (id.to_owned(), format!("{id} -> {result}")))
    .collect()
}
