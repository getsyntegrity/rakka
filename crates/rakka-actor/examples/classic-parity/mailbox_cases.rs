// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! Cases of the `Mailbox` oracle (`MailboxOracle.java`) that Rust can answer: the status transitions ST-01 to ST-08 (`MailboxStatus`)
//! the default queue MQ-01 to MQ-03 (`UnboundedMailbox`), and DL-01 (what `deadLetters` answers to a `Watch`, with a real system).
//! Each script drives the Rust type exactly as the oracle drives Pekko's.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy, SupervisorStrategy};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::config::Config;
use rakka_actor::dispatch::Terminated;
use rakka_actor::dispatch::bounded_node_queue::NonBlockingBoundedMailbox;
use rakka_actor::dispatch::deque_mailbox::UnboundedDequeBasedMailbox;
use rakka_actor::dispatch::mailbox_status::MailboxStatus;
use rakka_actor::dispatch::mailboxes::Mailboxes;
use rakka_actor::dispatch::message_queue::{Envelope, MailboxType, MessageQueue, UnboundedMailbox};
use rakka_actor::dispatch::queue_semantics::{MessageQueueSemantics, ProducedQueue};
use rakka_actor::dispatch::single_consumer_mailbox::SingleConsumerOnlyUnboundedMailbox;
use rakka_actor::dispatch::system_queue::SystemQueue;
use rakka_actor::event::{Channel, DeadLetter};

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
        "MBX-GRD-02 Kill to /user: the strategy of the root guardian logs and stops it, and the system terminates",
        "needs an error subscriber that is not an actor (internal): produced in-crate by actor::guardian_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0",
    ),
    (
        "MBX-GRD-03 Kill to the root guardian: the bubble walker logs, /user stops, and whenTerminated stays pending",
        "needs an error subscriber that is not an actor (internal): produced in-crate by actor::guardian_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0",
    ),
    (
        "MBX-GRD-04 Kill to /system: the strategy of the root guardian logs and stops it, and the system terminates",
        "needs an error subscriber that is not an actor (internal): produced in-crate by actor::guardian_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0",
    ),
    (
        "MBX-RJ-08 an attach rejected twice while /user supervises: the root guardian stops /user and the system terminates",
        "needs an executor that rejects tasks (internal): produced in-crate by dispatch::executor_rejection_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0",
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
        "MBX-IF-06 the supervisor's log of a cell that cannot be made publishes the message of what made it fail",
        "needs a cell that cannot be made (a mailbox type that throws in Pekko) and an executor that runs nothing by itself (internal): produced in-crate by actor::init_failure_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-SUP-3)",
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
        "MBX-UT-06 a restart with only an unstarted child does not wait for it: the new instance starts first, then the child starts, is terminated",
        "needs the unstarted reference and the real cell (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-05 a parent already waiting for a started child to die ignores the Supervise of a new child: the child is never pointed and the parent still terminates",
        "needs the unstarted reference and the real cell, and to hold the death of a started child (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
    ),
    (
        "MBX-UT-07 a restart waiting for a started child: the Supervise of an unstarted child is processed meanwhile, the child starts and terminates, the restart completes when the started child is gone",
        "needs the unstarted reference and the real cell, and to hold the death of a started child (internal): produced in-crate by actor::unstarted_termination_tests and compared with this line of cases/oracle-only/mailbox.golden, printed by Pekko 2.0.0 (Q-MBX-11, slice C4)",
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

/// The receiver the oracle's `UnboundedMailbox` is given (`enqueue(receiver, handle)`); the unbounded queue only passes it on.
fn receiver() -> ActorRef {
    static SYSTEM: std::sync::OnceLock<ActorSystem> = std::sync::OnceLock::new();
    SYSTEM
        .get_or_init(|| ActorSystem::create("mailbox-receiver").expect("a system"))
        .dead_letters()
}

fn put(queue: &dyn MessageQueue, text: &str) {
    queue.enqueue(
        &receiver(),
        Envelope {
            message: Arc::new(text.to_owned()),
            sender: None,
        },
    );
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
    queue.clean_up(&receiver(), &*dead_letters);
    format!(
        "source {} | target {} order {}",
        counts(&*queue),
        counts(&*dead_letters),
        drain(&*dead_letters)
    )
}

fn put_first(queue: &dyn MessageQueue, text: &str) {
    let envelope = Envelope {
        message: Arc::new(text.to_owned()),
        sender: None,
    };
    queue
        .as_deque()
        .expect("a deque based queue")
        .enqueue_first(&receiver(), envelope);
}

/// `MBX-DQ-01`: `enqueueFirst` goes to the head, ahead of what is queued; `cleanUp` drains from the head.
fn the_unbounded_deque_based_queue_takes_envelopes_at_its_head() -> String {
    let queue = UnboundedDequeBasedMailbox.create(&receiver());
    let dead_letters = UnboundedMailbox.create();
    let is = |semantics| queue.satisfies(semantics) == Some(true);
    let kind = format!(
        "class={} unbounded={} multipleConsumer={} bounded={}",
        queue.class_name(),
        is(MessageQueueSemantics::UNBOUNDED),
        is(MessageQueueSemantics::MULTIPLE_CONSUMER),
        is(MessageQueueSemantics::BOUNDED),
    );
    put(&*queue, "a");
    put(&*queue, "b");
    put_first(&*queue, "c");
    put_first(&*queue, "d");
    let before = counts(&*queue);
    let drained = drain(&*queue);
    let then = if queue.dequeue().is_none() {
        "null"
    } else {
        "message"
    };
    let mixed = format!("{before} {drained} then {then} {}", counts(&*queue));
    put_first(&*queue, "x");
    put_first(&*queue, "y");
    let only_first = drain(&*queue);
    put(&*queue, "a");
    put_first(&*queue, "c");
    put(&*queue, "b");
    queue.clean_up(&receiver(), &*dead_letters);
    format!(
        "{kind} | mixed {mixed} | onlyFirst {only_first} | cleanUp source {} target {}",
        counts(&*queue),
        drain(&*dead_letters)
    )
}

/// `MBX-SCQ-01`: the single consumer only queue, driven directly; the class and the semantics it declares.
fn the_single_consumer_only_queue_is_first_in_first_out_and_declares_its_semantics() -> String {
    let queue = SingleConsumerOnlyUnboundedMailbox.create(&receiver());
    let dead_letters = UnboundedMailbox.create();
    let is = |semantics| queue.satisfies(semantics) == Some(true);
    let kind = format!(
        "class={} unbounded={} multipleConsumer={} bounded={}",
        queue.class_name(),
        is(MessageQueueSemantics::UNBOUNDED),
        is(MessageQueueSemantics::MULTIPLE_CONSUMER),
        is(MessageQueueSemantics::BOUNDED),
    );
    let none = if queue.dequeue().is_none() {
        "null"
    } else {
        "message"
    };
    let empty = format!("{} {none}", counts(&*queue));
    put(&*queue, "a");
    put(&*queue, "b");
    let first = queue
        .dequeue()
        .and_then(|e| e.message.downcast_ref::<String>().cloned())
        .expect("a string");
    put(&*queue, "c");
    let before = counts(&*queue);
    let rest = drain(&*queue);
    let mixed = format!("{before} first={first} rest={rest} {}", counts(&*queue));
    put(&*queue, "x");
    put(&*queue, "y");
    queue.clean_up(&receiver(), &*dead_letters);
    format!(
        "{kind} | new {empty} | {mixed} | cleanUp source {} target {}",
        counts(&*queue),
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
        .expect("terminated")
        .expect("terminated normally");
    line
}

/// Occupies the only dispatcher thread until released (the oracle's `Block` message) and logs `r:block` afterwards.
struct Blocker {
    entered: std::sync::mpsc::Sender<()>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
    log: Arc<Mutex<Vec<String>>>,
    started: Arc<AtomicUsize>,
}
impl Actor for Blocker {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.started.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
        self.entered.send(()).expect("entered");
        self.release.lock().unwrap().recv().expect("release");
        self.log.lock().unwrap().push("r:block".to_owned());
        true
    }
}

/// Logs `r:<text>`; a message starting with `sleep` takes 60 ms, like the oracle's `Worker`.
struct Worker {
    log: Arc<Mutex<Vec<String>>>,
    started: Arc<AtomicUsize>,
}
impl Actor for Worker {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.started.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        if text.starts_with("sleep") {
            std::thread::sleep(Duration::from_millis(60));
        }
        self.log.lock().unwrap().push(format!("r:{text}"));
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
    // like the oracle's `spawn`, each actor is started (`preStart` ran) before the next is created and before anything is sent:
    // a message sent to an actor whose cell is not installed yet is handed over later, in an order the scenario does not control
    let started = Arc::new(AtomicUsize::new(0));
    let await_started = |count: usize| wait_until(|| started.load(Ordering::SeqCst) == count);
    let blocker = system
        .actor_of(Props::new({
            let (log, started) = (log.clone(), started.clone());
            move || {
                let (entered, release) = gate.lock().unwrap().take().expect("one instance");
                Blocker {
                    entered,
                    release: Mutex::new(release),
                    log: log.clone(),
                    started: started.clone(),
                }
            }
        }))
        .expect("blocker");
    await_started(1);
    let spawn = |name: &str, count: usize| {
        let (log, started) = (log.clone(), started.clone());
        let worker = system
            .actor_of_named(
                Props::new(move || Worker {
                    log: log.clone(),
                    started: started.clone(),
                }),
                name,
            )
            .expect("worker");
        await_started(count);
        worker
    };
    let (x, y) = (spawn("x", 2), spawn("y", 3));
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
        .expect("terminated")
        .expect("terminated normally");
    line
}

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "condition not reached");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Records its lifecycle in the shared log as the oracle's `LifeActor` does: `label:preStart`, `label:r:<message>`, `label:postStop`.
struct LogsItsLife {
    label: &'static str,
    log: Arc<Mutex<Vec<String>>>,
}

impl Actor for LogsItsLife {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.log
            .lock()
            .unwrap()
            .push(format!("{}:preStart", self.label));
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.log
            .lock()
            .unwrap()
            .push(format!("{}:postStop", self.label));
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let text = msg.downcast_ref::<String>().expect("string messages");
        self.log
            .lock()
            .unwrap()
            .push(format!("{}:r:{text}", self.label));
        true
    }
}

/// The oracle's `UnstartedParent` without `attachChild`: logs `P:preStart` and `P:postStop`, and on `go` creates the child `c`
/// (synchronously, as `ActorContext::actor_of` does) and sends it the messages of the case.
struct ParentOfLifeActor {
    log: Arc<Mutex<Vec<String>>>,
    child: Arc<Mutex<Option<ActorRef>>>,
    messages: Vec<String>,
}

impl Actor for ParentOfLifeActor {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:preStart".to_owned());
    }
    fn post_stop(&mut self, _ctx: &mut ActorContext) {
        self.log.lock().unwrap().push("P:postStop".to_owned());
    }
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg.downcast_ref::<String>().is_some_and(|m| m == "go") {
            let log = self.log.clone();
            let child = ctx
                .actor_of_named(
                    Props::new(move || LogsItsLife {
                        label: "c",
                        log: log.clone(),
                    }),
                    "c",
                )
                .expect("child");
            *self.child.lock().unwrap() = Some(child.clone());
            for message in &self.messages {
                child.tell(message.clone(), None);
            }
        }
        true
    }
}

/// Records every `DeadLetter` as `message<-sender->recipient`, the way the oracle's listener does.
struct RecordsDeadLetters(Arc<Mutex<Vec<String>>>);

impl Actor for RecordsDeadLetters {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>() {
            let text = dead
                .message
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_default();
            self.0.lock().unwrap().push(format!(
                "{text}<-{}->{}",
                dead.sender.path().name(),
                dead.recipient.path().name()
            ));
        }
        true
    }
}

fn listed(items: &[String]) -> String {
    format!("[{}]", items.join(", "))
}

/// `MBX-UT-04`: a parent stopped after its child started waits for the child, which dies first. The child is created
/// synchronously here, so the case needs no unstarted reference.
fn a_parent_stopped_after_its_child_started_waits_for_it() -> String {
    let system = ActorSystem::create("mbx").expect("system");
    let (log, dead, child) = (
        Arc::new(Mutex::new(Vec::new())),
        Arc::new(Mutex::new(Vec::new())),
        Arc::new(Mutex::new(None)),
    );
    let listener = system
        .actor_of(Props::new({
            let dead = dead.clone();
            move || RecordsDeadLetters(dead.clone())
        }))
        .expect("listener");
    system
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    let parent = system
        .actor_of_named(
            Props::new({
                let (log, child) = (log.clone(), child.clone());
                move || ParentOfLifeActor {
                    log: log.clone(),
                    child: child.clone(),
                    messages: vec!["m1".to_owned()],
                }
            }),
            "p",
        )
        .expect("parent");
    wait_until(|| log.lock().unwrap().contains(&"P:preStart".to_owned()));
    parent.tell("go".to_owned(), None);
    wait_until(|| log.lock().unwrap().contains(&"c:r:m1".to_owned()));
    system.stop(&parent);
    wait_until(|| parent.is_terminated());
    let child_terminated = child
        .lock()
        .unwrap()
        .as_ref()
        .expect("the child")
        .is_terminated();
    let line = format!(
        "parentTerminated={} childTerminated={child_terminated} deadLetters={} log={}",
        parent.is_terminated(),
        listed(&dead.lock().unwrap()),
        listed(&log.lock().unwrap())
    );
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated")
        .expect("terminated normally");
    line
}

/// `MBX-UT-08`: two senders race with the installation of the real cell of an asynchronously created actor
/// (`ActorSystem::actor_of` returns a reference that starts unstarted): nothing lost, nothing repeated, per-sender order kept.
fn senders_racing_with_the_installation_of_the_real_cell() -> String {
    let system = ActorSystem::create("mbx").expect("system");
    let (log, dead) = (
        Arc::new(Mutex::new(Vec::new())),
        Arc::new(Mutex::new(Vec::new())),
    );
    let listener = system
        .actor_of(Props::new({
            let dead = dead.clone();
            move || RecordsDeadLetters(dead.clone())
        }))
        .expect("listener");
    system
        .event_stream()
        .subscribe(&listener, Channel::DeadLetter);
    let child = system
        .actor_of_named(
            Props::new({
                let log = log.clone();
                move || LogsItsLife {
                    label: "c",
                    log: log.clone(),
                }
            }),
            "c",
        )
        .expect("child");
    let go = Arc::new(std::sync::Barrier::new(3));
    let senders: Vec<_> = ["a", "b"]
        .into_iter()
        .map(|who| {
            let (child, go) = (child.clone(), go.clone());
            std::thread::spawn(move || {
                go.wait();
                for i in 0..200 {
                    child.tell(format!("{who}:{i}"), None);
                }
            })
        })
        .collect();
    go.wait();
    senders
        .into_iter()
        .for_each(|sender| sender.join().expect("sender"));
    wait_until(|| {
        log.lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.starts_with("c:r:"))
            .count()
            == 400
    });
    std::thread::sleep(Duration::from_millis(500)); // a bounded wait for an absence: nothing more may arrive
    let log = log.lock().unwrap().clone();
    let (mut next, mut ordered) = (std::collections::HashMap::new(), true);
    let mut distinct = std::collections::HashSet::new();
    let mut delivered = 0;
    for entry in log.iter().filter(|entry| entry.starts_with("c:r:")) {
        delivered += 1;
        distinct.insert(entry.clone());
        let (who, index) = entry["c:r:".len()..].split_once(':').expect("who:index");
        let expected = next.entry(who.to_owned()).or_insert(0_usize);
        ordered &= *expected == index.parse::<usize>().expect("index");
        *expected += 1;
    }
    let pre_starts = log.iter().filter(|entry| *entry == "c:preStart").count();
    let line = format!(
        "delivered={delivered} distinct={} perSenderOrdered={ordered} preStarts={pre_starts} deadLetters={}",
        distinct.len(),
        listed(&dead.lock().unwrap())
    );
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated")
        .expect("terminated normally");
    line
}

/// A worker of the `MBX-ID` cases: logs `label:message`; `block` holds its turn until the shared gate opens.
struct IdWorker {
    label: &'static str,
    env: Arc<IdEnv>,
}
impl Actor for IdWorker {
    fn pre_start(&mut self, _ctx: &mut ActorContext) {
        self.env.started.fetch_add(1, Ordering::SeqCst);
    }
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(text) = msg.downcast_ref::<String>() else {
            return false;
        };
        if text == "block" {
            self.env.entered.fetch_add(1, Ordering::SeqCst);
            let mut open = self.env.gate_open.lock().unwrap();
            while !*open {
                open = self.env.gate_changed.wait(open).unwrap();
            }
        }
        self.env
            .log
            .lock()
            .unwrap()
            .push(format!("{}:{text}", self.label));
        true
    }
}

struct IdDeadListener(Arc<IdEnv>);
impl Actor for IdDeadListener {
    fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(dead) = msg.downcast_ref::<DeadLetter>()
            && let Some(text) = dead.message.downcast_ref::<String>()
        {
            self.0
                .dead
                .lock()
                .unwrap()
                .push(format!("{}:{text}", dead.recipient.path().name()));
        }
        true
    }
}

/// The environment of the `MBX-ID` cases (the oracle's `IdEnv`): ids `cap2-mailbox` and `cap4-mailbox` are non-blocking bounded
/// mailboxes of capacity 2 and 4.
struct IdEnv {
    log: Mutex<Vec<String>>,
    dead: Mutex<Vec<String>>,
    started: AtomicUsize,
    entered: AtomicUsize,
    kid: Mutex<Option<ActorRef>>,
    gate_open: Mutex<bool>,
    gate_changed: std::sync::Condvar,
}

/// Creates the child `kid` (props with the capacity-4 id) when told `spawn`.
struct IdParent(Arc<IdEnv>);
impl Actor for IdParent {
    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if msg
            .downcast_ref::<String>()
            .is_some_and(|text| text == "spawn")
        {
            let kid = ctx
                .actor_of_named(self.0.props("kid", Some("cap4-mailbox")), "kid")
                .expect("kid");
            *self.0.kid.lock().unwrap() = Some(kid);
        }
        true
    }
}

impl IdEnv {
    fn new() -> Arc<IdEnv> {
        Arc::new(IdEnv {
            kid: Mutex::new(None),
            log: Mutex::default(),
            dead: Mutex::default(),
            started: AtomicUsize::new(0),
            entered: AtomicUsize::new(0),
            gate_open: Mutex::new(false),
            gate_changed: std::sync::Condvar::new(),
        })
    }

    fn system(self: &Arc<Self>) -> ActorSystem {
        let mut settings = Settings::default();
        for (id, capacity) in [("cap2-mailbox", 2), ("cap4-mailbox", 4)] {
            settings.mailboxes.register(
                id,
                NonBlockingBoundedMailbox::new(capacity).expect("capacity"),
            );
        }
        self.system_with(settings)
    }

    fn system_with(self: &Arc<Self>, settings: Settings) -> ActorSystem {
        let system = ActorSystem::create_with_settings("mbx", settings).expect("system");
        let listener = system
            .actor_of(
                Props::new({
                    let env = self.clone();
                    move || IdDeadListener(env.clone())
                })
                .with_mailbox_id("unbounded"),
            )
            .expect("listener");
        system
            .event_stream()
            .subscribe(&listener, Channel::DeadLetter);
        system
    }

    fn props(self: &Arc<Self>, label: &'static str, mailbox_id: Option<&str>) -> Props {
        let env = self.clone();
        let props = Props::new(move || IdWorker {
            label,
            env: env.clone(),
        });
        match mailbox_id {
            Some(id) => props.with_mailbox_id(id),
            None => props,
        }
    }

    fn spawn(
        self: &Arc<Self>,
        system: &ActorSystem,
        label: &'static str,
        mailbox_id: Option<&str>,
    ) -> ActorRef {
        self.spawn_props(system, label, self.props(label, mailbox_id))
    }

    fn spawn_props(
        self: &Arc<Self>,
        system: &ActorSystem,
        label: &'static str,
        props: Props,
    ) -> ActorRef {
        let before = self.started.load(Ordering::SeqCst);
        let worker = system.actor_of_named(props, label).expect("worker");
        wait_until(|| self.started.load(Ordering::SeqCst) > before);
        worker
    }

    /// The props of an `IdWorker` wrapped in `wrap`, which declares what the actor requires of its queue.
    fn requiring<A: Actor>(
        self: &Arc<Self>,
        label: &'static str,
        wrap: fn(IdWorker) -> A,
    ) -> Props {
        let env = self.clone();
        Props::new(move || {
            wrap(IdWorker {
                label,
                env: env.clone(),
            })
        })
    }

    fn open_gate(&self) {
        *self.gate_open.lock().unwrap() = true;
        self.gate_changed.notify_all();
    }

    fn per_actor(&self, labels: &[&str]) -> String {
        let (log, dead) = (self.log.lock().unwrap(), self.dead.lock().unwrap());
        let of = |entries: &[String], label: &str| {
            let prefix = format!("{label}:");
            let found: Vec<&str> = entries
                .iter()
                .filter_map(|entry| entry.strip_prefix(&prefix))
                .collect();
            format!("[{}]", found.join(", "))
        };
        labels
            .iter()
            .map(|label| {
                format!(
                    "{label} received={} deadLettered={};",
                    of(&log, label),
                    of(&dead, label)
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn finish_id_case(env: &IdEnv, system: ActorSystem, line: String) -> String {
    env.open_gate();
    system
        .terminate()
        .wait(Duration::from_secs(10))
        .expect("terminated")
        .expect("terminated normally");
    line
}

/// `MBX-ID-01`: ids of capacity 2 and 4 select their own queue; a second actor with the capacity-2 id has a queue of its own.
fn two_ids_select_their_queues_and_actors_keep_their_own() -> String {
    let env = IdEnv::new();
    let system = env.system();
    let actors = [
        env.spawn(&system, "a", Some("cap2-mailbox")),
        env.spawn(&system, "b", Some("cap4-mailbox")),
        env.spawn(&system, "c", Some("cap2-mailbox")),
    ];
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 3);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| env.log.lock().unwrap().len() == 3 + 8 && env.dead.lock().unwrap().len() == 10);
    let line = env.per_actor(&["a", "b", "c"]);
    finish_id_case(&env, system, line)
}

/// `MBX-ID-02`: an unknown id fails `actorOf` at the call; no actor is made and the name can be used again.
fn an_unknown_id_fails_actor_of_and_frees_the_name() -> String {
    let env = IdEnv::new();
    let system = env.system();
    let outcome = match system.actor_of_named(env.props("w", Some("typo")), "w") {
        Ok(_) => "created".to_owned(),
        Err(error) => format!(
            "{}: {} | cause {}",
            error.class().java_name(),
            error.message(),
            error.cause().unwrap_or("none")
        ),
    };
    env.spawn(&system, "w", None);
    let line = format!(
        "actorOf: {outcome} | instancesStartedAfterRetryWithSameName={}",
        env.started.load(Ordering::SeqCst)
    );
    finish_id_case(&env, system, line)
}

/// `MBX-ID-03`: no selection, the built-in ids and the empty id all give the unbounded FIFO queue.
fn the_default_and_built_in_ids_are_unbounded_fifo() -> String {
    let env = IdEnv::new();
    let system = env.system();
    let ids = [
        None,
        Some("unbounded"),
        Some("pekko.actor.default-mailbox"),
        Some(""),
    ];
    let labels = ["w0", "w1", "w2", "w3"];
    let actors: Vec<ActorRef> = labels
        .iter()
        .zip(ids)
        .map(|(label, id)| env.spawn(&system, label, id))
        .collect();
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 4);
    for actor in &actors {
        for number in 1..=10 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| env.log.lock().unwrap().len() == 4 * 11);
    let line = env.per_actor(&labels);
    finish_id_case(&env, system, line)
}

/// The oracle's `LifoType`: hands out the newest envelope first and loses none.
#[derive(Default)]
struct LifoQueue(Mutex<Vec<Envelope>>);

impl MessageQueue for LifoQueue {
    fn satisfies(&self, requirement: MessageQueueSemantics) -> Option<bool> {
        Some(
            requirement == MessageQueueSemantics::MESSAGE_QUEUE
                || requirement == MessageQueueSemantics::UNBOUNDED,
        )
    }
    fn enqueue(&self, _receiver: &ActorRef, envelope: Envelope) -> bool {
        self.0.lock().unwrap().push(envelope);
        true
    }
    fn dequeue(&self) -> Option<Envelope> {
        self.0.lock().unwrap().pop()
    }
    fn number_of_messages(&self) -> usize {
        self.0.lock().unwrap().len()
    }
    fn has_messages(&self) -> bool {
        !self.0.lock().unwrap().is_empty()
    }
    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        while let Some(envelope) = self.dequeue() {
            dead_letters.enqueue(owner, envelope);
        }
    }
}

struct LifoMailbox;
impl MailboxType for LifoMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::<LifoQueue>::default()
    }

    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new(
            "pilot.MailboxOracle$LifoQueue",
            &[MessageQueueSemantics::UNBOUNDED],
        )
    }
}

/// `MBX-CFG-02`: a type registered as the default mailbox serves the actors without a selection (no id, the empty id and the
/// default id); `unbounded` and another id keep their own queues.
fn the_registered_default_serves_the_actors_without_a_selection() -> String {
    let env = IdEnv::new();
    let mut settings = Settings::default();
    settings.mailboxes.register(
        "cap4-mailbox",
        NonBlockingBoundedMailbox::new(4).expect("capacity"),
    );
    settings
        .mailboxes
        .register(Mailboxes::DEFAULT_MAILBOX_ID, LifoMailbox);
    let system = env.system_with(settings);
    let ids = [
        None,
        Some(""),
        Some("pekko.actor.default-mailbox"),
        Some("unbounded"),
        Some("cap4-mailbox"),
    ];
    let labels = ["w0", "w1", "w2", "w3", "w4"];
    let actors: Vec<ActorRef> = labels
        .iter()
        .zip(ids)
        .map(|(label, id)| env.spawn(&system, label, id))
        .collect();
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 5);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| {
        env.log.lock().unwrap().len() == 3 * 7 + 7 + 5 && env.dead.lock().unwrap().len() == 2
    });
    let line = env.per_actor(&labels);
    finish_id_case(&env, system, line)
}

/// `MBX-CFG-04`: the deployment of the actor path (read from `pekko.actor.deployment`) decides the mailbox over the props, by the
/// full path of a child too; the id it names must exist, and a failed `actorOf` leaves the name free.
fn a_deployment_of_the_path_decides_the_mailbox_over_the_props() -> String {
    let env = IdEnv::new();
    let application = r#"pekko.actor.deployment {
        "/d-lifo" { mailbox = lifo-mailbox }
        "/d-lifo-b" { mailbox = lifo-mailbox }
        "/d-empty" { mailbox = "" }
        "/d-bad" { mailbox = typo }
        "/kid" { mailbox = typo }
        "/d-parent/kid" { mailbox = lifo-mailbox } }"#;
    let config = Config::parse_string(application)
        .and_then(|parsed| parsed.with_fallback(&Config::default_reference()).resolve())
        .expect("configuration");
    let mut settings = Settings::from_config(
        &rakka_actor::config::Settings::new("mbx", config).expect("settings"),
    )
    .expect("deployments");
    settings.mailboxes.register("lifo-mailbox", LifoMailbox);
    settings.mailboxes.register(
        "cap4-mailbox",
        NonBlockingBoundedMailbox::new(4).expect("capacity"),
    );
    let system = env.system_with(settings);
    let spawned = [
        env.spawn(&system, "d-lifo", Some("cap4-mailbox")),
        env.spawn(&system, "d-lifo-b", Some("never-registered")),
        env.spawn(&system, "d-empty", Some("cap4-mailbox")),
        env.spawn(&system, "other", Some("cap4-mailbox")),
    ];
    let parent = system
        .actor_of_named(
            Props::new({
                let env = env.clone();
                move || IdParent(env.clone())
            }),
            "d-parent",
        )
        .expect("parent");
    parent.tell("spawn".to_owned(), None);
    wait_until(|| env.started.load(Ordering::SeqCst) == 5 && env.kid.lock().unwrap().is_some());
    let mut bad = String::new();
    for _ in 0..2 {
        bad.push_str(
            &match system.actor_of_named(env.props("d-bad", Some("cap4-mailbox")), "d-bad") {
                Ok(_) => " | created".to_owned(),
                Err(error) => format!(
                    " | {}: {} cause {}",
                    error.class().java_name(),
                    error.message(),
                    error.cause().unwrap_or("none")
                ),
            },
        );
    }
    let kid = env.kid.lock().unwrap().clone().expect("kid");
    let actors: Vec<ActorRef> = spawned.into_iter().chain([kid]).collect();
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 5);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| {
        env.log.lock().unwrap().len() == 3 * 7 + 2 * 5 && env.dead.lock().unwrap().len() == 4
    });
    let line = format!(
        "{}{bad}",
        env.per_actor(&["d-lifo", "d-lifo-b", "d-empty", "other", "kid"])
    );
    finish_id_case(&env, system, line)
}

/// The oracle's application-defined semantics: one bound to a mailbox, one bound to none.
const AUDIT_TRAIL: MessageQueueSemantics =
    MessageQueueSemantics::new("pilot.MailboxOracle$AuditTrail");
const UNMAPPED: MessageQueueSemantics = MessageQueueSemantics::new("pilot.MailboxOracle$Unmapped");

/// An unbounded FIFO queue that has exactly the semantics in `has` (the oracle's `AuditTrailQueue`, or what a lying type really creates).
struct SemanticsQueue {
    inner: Box<dyn MessageQueue>,
    has: &'static [MessageQueueSemantics],
    class: &'static str,
}

impl MessageQueue for SemanticsQueue {
    fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool {
        self.inner.enqueue(receiver, envelope)
    }
    fn dequeue(&self) -> Option<Envelope> {
        self.inner.dequeue()
    }
    fn number_of_messages(&self) -> usize {
        self.inner.number_of_messages()
    }
    fn has_messages(&self) -> bool {
        self.inner.has_messages()
    }
    fn clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue) {
        self.inner.clean_up(owner, dead_letters)
    }
    fn satisfies(&self, requirement: MessageQueueSemantics) -> Option<bool> {
        Some(requirement == MessageQueueSemantics::MESSAGE_QUEUE || self.has.contains(&requirement))
    }
    fn class_name(&self) -> &'static str {
        self.class
    }
}

/// The oracle's `AuditTrailType`: an unbounded FIFO queue that also has the `AuditTrail` semantics.
struct AuditTrailMailbox;
impl MailboxType for AuditTrailMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(SemanticsQueue {
            inner: UnboundedMailbox.create(),
            has: &[MessageQueueSemantics::UNBOUNDED, AUDIT_TRAIL],
            class: "pilot.MailboxOracle$AuditTrailQueue",
        })
    }

    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new(
            "pilot.MailboxOracle$AuditTrailQueue",
            &[MessageQueueSemantics::UNBOUNDED, AUDIT_TRAIL],
        )
    }
}

/// An `IdWorker` that declares `RequiresMessageQueue[T]`, as the oracle's `UnboundedWorker`, `BoundedWorker`, ...
macro_rules! requiring_worker {
    ($name:ident, $semantics:expr) => {
        struct $name(IdWorker);
        impl Actor for $name {
            fn required_message_queue() -> Option<MessageQueueSemantics> {
                Some($semantics)
            }
            fn pre_start(&mut self, ctx: &mut ActorContext) {
                self.0.pre_start(ctx);
            }
            fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
                self.0.receive(ctx, msg)
            }
        }
    };
}
requiring_worker!(UnboundedWorker, MessageQueueSemantics::UNBOUNDED);
requiring_worker!(BoundedWorker, MessageQueueSemantics::BOUNDED);
requiring_worker!(AuditTrailWorker, AUDIT_TRAIL);
requiring_worker!(UnmappedWorker, UNMAPPED);
requiring_worker!(AnyQueueWorker, MessageQueueSemantics::MESSAGE_QUEUE);

/// `MBX-REQ-01`: the actors that require a queue semantics get the mailbox bound to it (the table of `reference.conf` for the
/// unbounded one, the bindings of the configuration for the others), not the registered default, which the actors that require nothing keep.
fn an_actor_requiring_a_queue_gets_the_mailbox_bound_to_it() -> String {
    let env = IdEnv::new();
    let mut settings = Settings::default();
    let mailboxes = &mut settings.mailboxes;
    mailboxes.register(Mailboxes::DEFAULT_MAILBOX_ID, LifoMailbox);
    mailboxes.register(
        "cap4-mailbox",
        NonBlockingBoundedMailbox::new(4).expect("capacity"),
    );
    mailboxes.register("audit-mailbox", AuditTrailMailbox);
    mailboxes.bind_requirement(MessageQueueSemantics::BOUNDED, "cap4-mailbox");
    mailboxes.bind_requirement(AUDIT_TRAIL, "audit-mailbox");
    let system = env.system_with(settings);
    let actors = [
        env.spawn(&system, "plain", None),
        env.spawn_props(
            &system,
            "unbounded",
            env.requiring("unbounded", UnboundedWorker),
        ),
        env.spawn_props(&system, "bounded", env.requiring("bounded", BoundedWorker)),
        env.spawn_props(&system, "audit", env.requiring("audit", AuditTrailWorker)),
        env.spawn(&system, "plainAfter", None),
    ];
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 5);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| {
        env.log.lock().unwrap().len() == 4 * 7 + 5 && env.dead.lock().unwrap().len() == 2
    });
    let line = env.per_actor(&["plain", "unbounded", "bounded", "audit", "plainAfter"]);
    finish_id_case(&env, system, line)
}

/// `MBX-REQ-02`: a semantics with no mailbox bound to it fails `actorOf` at the call, and the name can be used again.
fn an_actor_requiring_an_unbound_queue_fails_actor_of_and_frees_the_name() -> String {
    let env = IdEnv::new();
    let system = env.system();
    let outcome = match system.actor_of_named(env.requiring("w", UnmappedWorker), "w") {
        Ok(_) => "created".to_owned(),
        Err(error) => format!(
            "{}: {} | cause {}",
            error.class().java_name(),
            error.message(),
            error.cause().unwrap_or("none")
        ),
    };
    env.spawn(&system, "w", None);
    let line = format!(
        "actorOf: {outcome} | instancesStartedAfterRetryWithSameName={}",
        env.started.load(Ordering::SeqCst)
    );
    finish_id_case(&env, system, line)
}

/// `MBX-REQ-03`: an id selected by the props or by a deployment decides the mailbox over the semantics the actor requires; an empty
/// id (`NoMailboxGiven`) in the props or in the deployment leaves the requirement.
fn a_selected_id_decides_the_mailbox_over_the_requirement() -> String {
    let env = IdEnv::new();
    let application = r#"pekko.actor.deployment {
        "/deployed" { mailbox = lifo-mailbox }
        "/emptyDeploy" { mailbox = "" } }"#;
    let config = Config::parse_string(application)
        .and_then(|parsed| parsed.with_fallback(&Config::default_reference()).resolve())
        .expect("configuration");
    let mut settings = Settings::from_config(
        &rakka_actor::config::Settings::new("mbx", config).expect("settings"),
    )
    .expect("deployments");
    let mailboxes = &mut settings.mailboxes;
    mailboxes.register("lifo-mailbox", LifoMailbox);
    for (id, capacity) in [("cap2-mailbox", 2), ("cap4-mailbox", 4)] {
        mailboxes.register(
            id,
            NonBlockingBoundedMailbox::new(capacity).expect("capacity"),
        );
    }
    mailboxes.bind_requirement(MessageQueueSemantics::BOUNDED, "cap4-mailbox");
    let system = env.system_with(settings);
    let unbounded = |label| env.requiring(label, UnboundedWorker);
    let bounded = |label| env.requiring(label, BoundedWorker);
    let labels = [
        "byProps",
        "byPropsBounded",
        "deployed",
        "emptyDeploy",
        "emptyProps",
        "required",
    ];
    let props = [
        unbounded("byProps").with_mailbox_id("lifo-mailbox"),
        bounded("byPropsBounded").with_mailbox_id("cap2-mailbox"),
        unbounded("deployed"),
        unbounded("emptyDeploy"),
        unbounded("emptyProps").with_mailbox_id(""),
        bounded("required"),
    ];
    let actors: Vec<ActorRef> = labels
        .iter()
        .zip(props)
        .map(|(label, props)| env.spawn_props(&system, label, props))
        .collect();
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 6);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| {
        env.log.lock().unwrap().len() == 4 * 7 + 3 + 5 && env.dead.lock().unwrap().len() == 4 + 2
    });
    let line = env.per_actor(&labels);
    finish_id_case(&env, system, line)
}

/// The oracle's `PlainType`: a mailbox whose queue has the `AuditTrail` semantics but whose type does not promise it.
struct PlainMailbox;
impl MailboxType for PlainMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        UnboundedMailbox.create()
    }
}

/// One attempt of `MBX-CMP-01`: `actorOf` with the mailbox `mailbox` selected. The class of the actor is the only difference in the
/// text: the JVM names it `java_class`, Rust by its type path.
fn attempt_to_create<A: Actor>(
    env: &Arc<IdEnv>,
    system: &ActorSystem,
    label: &'static str,
    props: Props,
    java_class: &str,
) -> String {
    let before = env.started.load(Ordering::SeqCst);
    let outcome = match system.actor_of_named(props, label) {
        Ok(_) => {
            wait_until(|| env.started.load(Ordering::SeqCst) > before);
            "created".to_owned()
        }
        Err(error) => format!(
            "{}: {} | cause {}",
            error.class().java_name(),
            error.message(),
            error.cause().unwrap_or("none")
        )
        .replace(std::any::type_name::<A>(), java_class),
    };
    format!("{label}: {outcome}; ")
}

/// The `attempts` of the oracle: `actorOf` of `$worker` (an `IdWorker` that declares what it requires) with `$mailbox` selected, or of the
/// plain worker when `$worker` is `IdWorker`; appends what it throws, or "created", to `$line`.
macro_rules! attempt {
    ($line:ident, $env:ident, $system:ident, $label:literal, IdWorker, $java:literal, $mailbox:expr) => {
        $line.push_str(&attempt_to_create::<IdWorker>(
            &$env,
            &$system,
            $label,
            $env.props($label, $mailbox),
            $java,
        ))
    };
    ($line:ident, $env:ident, $system:ident, $label:literal, $worker:ident, $java:literal, $mailbox:expr) => {{
        let props = $env.requiring($label, $worker);
        let props = match $mailbox {
            Some(id) => props.with_mailbox_id(id),
            None => props,
        };
        $line.push_str(&attempt_to_create::<$worker>(
            &$env,
            &$system,
            $label,
            props,
            &format!("pilot.MailboxOracle${}", $java),
        ))
    }};
}

/// `MBX-CMP-01`: the queue the selected mailbox promises must fulfill the requirement of the actor (`verifyRequirements`); a type that does
/// not promise its queue produces a plain `MessageQueue` even when the queue it creates has the capability.
fn the_selected_mailbox_must_fulfill_the_requirement_of_the_actor() -> String {
    let env = IdEnv::new();
    let mut settings = Settings::default();
    settings.mailboxes.register("plain-mailbox", PlainMailbox);
    settings
        .mailboxes
        .register("audit-mailbox", AuditTrailMailbox);
    let system = env.system_with(settings);
    let mut line = String::new();
    attempt!(
        line,
        env,
        system,
        "unboundedOnPlain",
        UnboundedWorker,
        "UnboundedWorker",
        Some("plain-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "auditOnUnbounded",
        AuditTrailWorker,
        "AuditTrailWorker",
        Some("unbounded")
    );
    attempt!(
        line,
        env,
        system,
        "boundedOnUnbounded",
        BoundedWorker,
        "BoundedWorker",
        Some("unbounded")
    );
    attempt!(
        line,
        env,
        system,
        "unboundedOnAudit",
        UnboundedWorker,
        "UnboundedWorker",
        Some("audit-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "auditOnAudit",
        AuditTrailWorker,
        "AuditTrailWorker",
        Some("audit-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "anyOnPlain",
        AnyQueueWorker,
        "AnyQueueWorker",
        Some("plain-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "nothingOnPlain",
        IdWorker,
        "unused",
        Some("plain-mailbox")
    );
    finish_id_case(&env, system, line.trim().to_owned())
}

/// `MBX-CMP-02`: with a dispatcher that requires the unbounded semantics, the selected mailbox must fulfill that requirement and the actor's,
/// the dispatcher's first; an actor binding that does not fit gives way to the dispatcher's binding, which is verified too.
fn the_selected_mailbox_must_fulfill_the_requirements_of_the_dispatcher_and_the_actor() -> String {
    let env = IdEnv::new();
    let mut settings = Settings {
        mailbox_requirement: Some(MessageQueueSemantics::UNBOUNDED),
        ..Settings::default()
    };
    let mailboxes = &mut settings.mailboxes;
    mailboxes.register(
        "cap4-mailbox",
        NonBlockingBoundedMailbox::new(4).expect("capacity"),
    );
    mailboxes.register("audit-mailbox", AuditTrailMailbox);
    mailboxes.bind_requirement(MessageQueueSemantics::BOUNDED, "cap4-mailbox");
    mailboxes.bind_requirement(AUDIT_TRAIL, "audit-mailbox");
    let system = env.system_with(settings);
    let mut line = String::new();
    attempt!(
        line,
        env,
        system,
        "dispatcherUnmetBySelected",
        IdWorker,
        "unused",
        Some("cap4-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "dispatcherMetBySelected",
        IdWorker,
        "unused",
        Some("audit-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "boundedBindingFallsBack",
        BoundedWorker,
        "BoundedWorker",
        None::<&str>
    );
    attempt!(
        line,
        env,
        system,
        "bothMetByAudit",
        AuditTrailWorker,
        "AuditTrailWorker",
        None::<&str>
    );
    attempt!(
        line,
        env,
        system,
        "actorUnmetOnly",
        BoundedWorker,
        "BoundedWorker",
        Some("unbounded")
    );
    attempt!(
        line,
        env,
        system,
        "bothUnmet",
        BoundedWorker,
        "BoundedWorker",
        Some("cap4-mailbox")
    );
    attempt!(
        line,
        env,
        system,
        "plainOnDefault",
        IdWorker,
        "unused",
        None::<&str>
    );
    finish_id_case(&env, system, line.trim().to_owned())
}

/// The oracle's `LiarType`: promises the `AuditTrail` queue but creates one that does not have that semantics.
struct LiarMailbox;
impl MailboxType for LiarMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        Box::new(SemanticsQueue {
            inner: UnboundedMailbox.create(),
            has: &[MessageQueueSemantics::UNBOUNDED],
            class: "pilot.MailboxOracle$LifoQueue",
        })
    }

    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new(
            "pilot.MailboxOracle$AuditTrailQueue",
            &[MessageQueueSemantics::UNBOUNDED, AUDIT_TRAIL],
        )
    }
}

/// The oracle's `InitParent`: creates `kid` from each `Props` it is sent and logs what it sees: the failure its supervision gets, the death.
struct InitParent {
    env: Arc<IdEnv>,
    attempts: usize,
}

impl Actor for InitParent {
    fn supervisor_strategy(&self) -> SupervisorStrategy {
        let env = self.env.clone();
        OneForOneStrategy::new(10, move |failure| {
            let message = failure.message.clone().unwrap_or_default();
            let without_uid = message.split('#').next().unwrap_or_default().to_owned();
            env.log
                .lock()
                .unwrap()
                .push(format!("failure:{}: {without_uid}", failure.class));
            Directive::Stop
        })
        .into()
    }

    fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
        if let Some(props) = msg.downcast_ref::<Props>() {
            let label = if self.attempts == 0 {
                "created"
            } else {
                "again"
            };
            self.attempts += 1;
            let entry = match ctx.actor_of_named(props.clone(), "kid") {
                Ok(kid) => {
                    ctx.watch(&kid);
                    label.to_owned()
                }
                Err(error) => format!("{label}:{}: {}", error.class().java_name(), error.message()),
            };
            self.env.log.lock().unwrap().push(entry);
            true
        } else if msg.downcast_ref::<Terminated>().is_some() {
            self.env.log.lock().unwrap().push("terminated".to_owned());
            true
        } else {
            false
        }
    }
}

/// `MBX-CMP-03`: what a type promises is checked again against the queue it really creates when the actor is made (`Dispatch.init`): `actorOf`
/// returns, the failure is delayed into `Create`, the actor stops and its name is free.
fn a_type_that_promises_its_queue_is_checked_against_the_queue_it_creates() -> String {
    let env = IdEnv::new();
    let mut settings = Settings::default();
    settings.mailboxes.register("liar-mailbox", LiarMailbox);
    let system = env.system_with(settings);
    let parent = system
        .actor_of_named(
            Props::new({
                let env = env.clone();
                move || InitParent {
                    env: env.clone(),
                    attempts: 0,
                }
            })
            .with_mailbox_id("unbounded"),
            "parent",
        )
        .expect("parent");
    let has_logged = |entry: &str| env.log.lock().unwrap().iter().any(|l| l.starts_with(entry));
    parent.tell(
        env.requiring("liar", AuditTrailWorker)
            .with_mailbox_id("liar-mailbox"),
        None,
    );
    wait_until(|| has_logged("terminated"));
    parent.tell(env.props("again", None), None);
    wait_until(|| has_logged("again") && env.started.load(Ordering::SeqCst) == 1);
    let line = format!(
        "{} | started={}",
        env.log.lock().unwrap().join(" | "),
        env.started.load(Ordering::SeqCst)
    );
    finish_id_case(&env, system, line)
}

/// The oracle's `CountingDefaultType`: a default mailbox that counts its queues and promises an unbounded one.
struct CountingDefaultMailbox(Arc<AtomicUsize>);
impl MailboxType for CountingDefaultMailbox {
    fn create(&self, _dead_letters: &ActorRef) -> Box<dyn MessageQueue> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::new(SemanticsQueue {
            inner: UnboundedMailbox.create(),
            has: &[MessageQueueSemantics::UNBOUNDED],
            class: "pilot.MailboxOracle$LifoQueue",
        })
    }

    fn produced_queue(&self) -> ProducedQueue {
        ProducedQueue::new(
            "pilot.MailboxOracle$LifoQueue",
            &[MessageQueueSemantics::UNBOUNDED],
        )
    }
}

/// `MBX-GRD-01`: the guardians take the registered default mailbox directly (`mailboxes.lookup(DefaultMailboxId)`): three queues of it exist
/// before any actor is created.
fn the_guardians_take_the_registered_default_mailbox() -> String {
    let env = IdEnv::new();
    let queues = Arc::new(AtomicUsize::new(0));
    let mut settings = Settings::default();
    settings.mailboxes.register(
        Mailboxes::DEFAULT_MAILBOX_ID,
        CountingDefaultMailbox(queues.clone()),
    );
    let system = env.system_with(settings);
    // Pekko also makes `deadLetterListener` and `eventStreamUnsubscriber`, which Rakka does not have: only the guardians are counted
    let line = format!("guardianQueues={}", queues.load(Ordering::SeqCst));
    finish_id_case(&env, system, line)
}

/// `MBX-REQ-04`: the semantics the dispatcher requires (`default-dispatcher.mailbox-requirement`) selects the mailbox of the actors that
/// require nothing and of those whose own requirement has no binding; the binding of the actor itself and a selected id come first.
fn the_requirement_of_the_dispatcher_selects_the_mailbox_after_the_actor_and_the_id() -> String {
    let env = IdEnv::new();
    let mut settings = Settings {
        mailbox_requirement: Some(MessageQueueSemantics::UNBOUNDED),
        ..Settings::default()
    };
    let mailboxes = &mut settings.mailboxes;
    mailboxes.register("lifo-mailbox", LifoMailbox);
    mailboxes.register("audit-mailbox", AuditTrailMailbox);
    mailboxes.bind_requirement(MessageQueueSemantics::UNBOUNDED, "lifo-mailbox");
    mailboxes.bind_requirement(AUDIT_TRAIL, "audit-mailbox");
    let system = env.system_with(settings);
    let actors = [
        env.spawn(&system, "plain", None),
        env.spawn_props(
            &system,
            "anyQueue",
            env.requiring("anyQueue", AnyQueueWorker),
        ),
        env.spawn_props(&system, "audit", env.requiring("audit", AuditTrailWorker)),
        env.spawn(&system, "selected", Some("unbounded")),
    ];
    for actor in &actors {
        actor.tell("block".to_owned(), None);
    }
    wait_until(|| env.entered.load(Ordering::SeqCst) == 4);
    for actor in &actors {
        for number in 1..=6 {
            actor.tell(format!("m{number}"), None);
        }
    }
    env.open_gate();
    wait_until(|| env.log.lock().unwrap().len() == 4 * 7);
    let line = env.per_actor(&["plain", "anyQueue", "audit", "selected"]);
    finish_id_case(&env, system, line)
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
            "MBX-ID-01 two ids select their own queue and two actors with the same id keep separate queues",
            two_ids_select_their_queues_and_actors_keep_their_own(),
        ),
        (
            "MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again",
            an_unknown_id_fails_actor_of_and_frees_the_name(),
        ),
        (
            "MBX-ID-03 no selection, the built-in ids and the empty id all give the unbounded FIFO queue",
            the_default_and_built_in_ids_are_unbounded_fifo(),
        ),
        (
            "MBX-CFG-02 a mailbox configured as the default serves the actors without a selection, not the ids that name another",
            the_registered_default_serves_the_actors_without_a_selection(),
        ),
        (
            "MBX-CFG-04 a deployment of the actor path decides the mailbox over the props, by the full path of a child too, and its unknown id fails actorOf",
            a_deployment_of_the_path_decides_the_mailbox_over_the_props(),
        ),
        (
            "MBX-REQ-01 an actor that requires a queue semantics gets the mailbox bound to it, not the default, and the actors that do not require it keep the default",
            an_actor_requiring_a_queue_gets_the_mailbox_bound_to_it(),
        ),
        (
            "MBX-REQ-02 an actor that requires a queue semantics with no mailbox bound to it fails actorOf at the call and its name is free again",
            an_actor_requiring_an_unbound_queue_fails_actor_of_and_frees_the_name(),
        ),
        (
            "MBX-REQ-03 an id selected by the props or by a deployment decides the mailbox over the queue semantics the actor requires; an empty id leaves the requirement",
            a_selected_id_decides_the_mailbox_over_the_requirement(),
        ),
        (
            "MBX-REQ-04 the queue semantics the dispatcher requires selects the mailbox of the actors that require nothing, and of those whose own requirement has no binding; the actor's own binding and a selected id come first",
            the_requirement_of_the_dispatcher_selects_the_mailbox_after_the_actor_and_the_id(),
        ),
        (
            "MBX-CMP-01 an actor whose requirement the queue promised by the selected mailbox does not fulfill fails actorOf at the call; one it fulfills, or an actor that requires nothing, is created",
            the_selected_mailbox_must_fulfill_the_requirement_of_the_actor(),
        ),
        (
            "MBX-CMP-02 the queue of the selected mailbox must fulfill the requirement of the dispatcher and that of the actor, the dispatcher's reported first; an actor binding that does not fit gives way to the dispatcher's binding",
            the_selected_mailbox_must_fulfill_the_requirements_of_the_dispatcher_and_the_actor(),
        ),
        (
            "MBX-CMP-03 a type that promises its queue is checked against the queue it creates when the actor is made: actorOf returns, the actor fails to initialise and stops, and its name is free",
            a_type_that_promises_its_queue_is_checked_against_the_queue_it_creates(),
        ),
        (
            "MBX-GRD-01 the root, user and system guardians take the registered default mailbox directly: three queues of it exist before any actor is created",
            the_guardians_take_the_registered_default_mailbox(),
        ),
        (
            "MBX-DQ-01 the unbounded deque based queue takes an envelope enqueued first at its head, counts it and cleans up from the head",
            the_unbounded_deque_based_queue_takes_envelopes_at_its_head(),
        ),
        (
            "MBX-SCQ-01 the single consumer only unbounded queue is first in first out, counts and cleans up in order, and declares the semantics of Pekko's NodeMessageQueue",
            the_single_consumer_only_queue_is_first_in_first_out_and_declares_its_semantics(),
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
        (
            "MBX-UT-04 control: a parent stopped after its child started waits for it, and the child dies first",
            a_parent_stopped_after_its_child_started_waits_for_it(),
        ),
        (
            "MBX-UT-08 senders racing with the installation of the real cell: nothing lost, nothing repeated, per-sender order kept",
            senders_racing_with_the_installation_of_the_real_cell(),
        ),
    ]
    .into_iter()
    .map(|(id, result)| (id.to_owned(), format!("{id} -> {result}")))
    .collect()
}
