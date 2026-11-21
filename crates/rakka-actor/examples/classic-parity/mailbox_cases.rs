// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! Cases of the `Mailbox` oracle (`MailboxOracle.java`) that Rust can answer: the status transitions ST-01 to ST-08 (`MailboxStatus`)
//! and the default queue MQ-01 to MQ-03 (`UnboundedMailbox`). Each script drives the Rust type exactly as the oracle drives Pekko's.

use std::sync::Arc;

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
        "needs the user queue and the dispatcher on the actor cell (ADR-0001 steps 3-6)",
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
        "MBX-SC-01 throughput 2 on one thread interleaves two actors in blocks of two",
        "needs the per-dispatcher throughput and deadline (ADR-0001 step 6)",
    ),
    (
        "MBX-SC-02 throughput-deadline-time ends the turn after a slow message",
        "needs the per-dispatcher throughput and deadline (ADR-0001 step 6)",
    ),
    (
        "MBX-SC-03 actor suspended with queued work does not reschedule itself",
        "needs the per-dispatcher throughput and deadline (ADR-0001 step 6)",
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
