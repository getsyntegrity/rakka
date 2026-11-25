// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala (Apache-2.0).
//! `deadLetterMailbox` (`Mailboxes.scala:54-72`): where everything addressed to a closed mailbox ends up.
//!
//! Pekko keeps one shared, permanently closed `Mailbox` per system. Its queues never store anything: a user message is turned into
//! `DeadLetter(msg, sender, receiver)` and sent to `deadLetters` (a message that already is a `DeadLetter` is dropped, so a
//! subscriber to dead letters cannot loop), and a system message into `DeadLetter(handle, receiver, receiver)`. Rust has no shared
//! `Mailbox` object (every cell owns its state, queue and system queue), so only the two forwarding behaviours remain; the
//! rest of that closed mailbox (`dequeue` is `null`, `hasMessages` false, `systemDrain` empty, `cleanUp` does nothing) is
//! "nothing is stored", which this type does by not having any storage.

use std::sync::Arc;

use crate::actor::actor_ref::ActorRef;
use crate::dispatch::message_queue::Envelope;
use crate::dispatch::sysmsg::SystemMessage;
use crate::event::DeadLetter;

/// `deadLetters.tell(DeadLetter(message, sender, receiver), sender)`; `Envelope` replaces a missing sender by `deadLetters`.
pub(crate) fn tell_dead_letter(dead_letters: &ActorRef, receiver: &ActorRef, envelope: Envelope) {
    let sender = envelope.sender.unwrap_or_else(|| dead_letters.clone());
    dead_letters.tell_any(
        Arc::new(DeadLetter {
            message: envelope.message,
            sender: sender.clone(),
            recipient: receiver.clone(),
        }),
        Some(sender),
    );
}

/// The forwarding half of Pekko's `deadLetterMailbox`.
pub(crate) struct DeadLetterMailbox {
    dead_letters: ActorRef,
}

impl DeadLetterMailbox {
    pub(crate) fn new(dead_letters: ActorRef) -> Self {
        DeadLetterMailbox { dead_letters }
    }

    /// `MessageQueue.enqueue(receiver, envelope)` of the dead-letter mailbox (`onDeadLetterMailboxEnqueue`).
    pub(crate) fn enqueue(&self, receiver: &ActorRef, envelope: Envelope) {
        if envelope.message.is::<DeadLetter>() {
            return; // an actor subscribed to DeadLetter: drop it instead of looping
        }
        tell_dead_letter(&self.dead_letters, receiver, envelope);
    }

    /// `systemEnqueue(receiver, handle)`: `deadLetters ! DeadLetter(handle, receiver, receiver)`.
    pub(crate) fn system_enqueue(&self, receiver: &ActorRef, handle: SystemMessage) {
        self.dead_letters.tell_any(
            Arc::new(DeadLetter {
                message: Arc::new(handle),
                sender: receiver.clone(),
                recipient: receiver.clone(),
            }),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::test_support::{Recorder, recorder};

    fn dead_letter_told(recorder: &Recorder, index: usize) -> DeadLetter {
        let told = recorder.told.lock().unwrap();
        told[index]
            .0
            .downcast_ref::<DeadLetter>()
            .expect("the dead-letter mailbox sends a DeadLetter")
            .clone()
    }

    #[test]
    fn a_user_message_is_forwarded_as_a_dead_letter_with_its_sender_and_receiver() {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let (_, receiver) = recorder("pekko://sys/user/receiver");
        let (_, sender) = recorder("pekko://sys/user/sender");
        let mailbox = DeadLetterMailbox::new(dead_letters_ref);

        mailbox.enqueue(
            &receiver,
            Envelope {
                message: Arc::new("late".to_owned()),
                sender: Some(sender.clone()),
            },
        );

        let forwarded = dead_letter_told(&dead_letters, 0);
        assert_eq!(forwarded.message.downcast_ref::<String>().unwrap(), "late");
        assert!(forwarded.sender == sender && forwarded.recipient == receiver);
        assert!(dead_letters.told.lock().unwrap()[0].1.as_ref() == Some(&sender));
    }

    #[test]
    fn a_user_message_without_sender_is_sent_by_dead_letters() {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let (_, receiver) = recorder("pekko://sys/user/receiver");
        let mailbox = DeadLetterMailbox::new(dead_letters_ref.clone());

        mailbox.enqueue(
            &receiver,
            Envelope {
                message: Arc::new(1_u8),
                sender: None,
            },
        );

        assert!(dead_letter_told(&dead_letters, 0).sender == dead_letters_ref);
    }

    #[test]
    fn a_message_that_already_is_a_dead_letter_is_dropped() {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let (_, receiver) = recorder("pekko://sys/user/receiver");
        let mailbox = DeadLetterMailbox::new(dead_letters_ref.clone());
        let again = DeadLetter {
            message: Arc::new("x".to_owned()),
            sender: dead_letters_ref,
            recipient: receiver.clone(),
        };

        mailbox.enqueue(
            &receiver,
            Envelope {
                message: Arc::new(again),
                sender: None,
            },
        );

        assert!(dead_letters.told.lock().unwrap().is_empty());
    }

    #[test]
    fn a_system_message_is_forwarded_as_a_dead_letter_from_and_to_the_receiver() {
        let (dead_letters, dead_letters_ref) = recorder("pekko://sys/deadLetters");
        let (_, receiver) = recorder("pekko://sys/user/receiver");
        let mailbox = DeadLetterMailbox::new(dead_letters_ref);

        mailbox.system_enqueue(&receiver, SystemMessage::Suspend);

        let forwarded = dead_letter_told(&dead_letters, 0);
        assert!(matches!(
            forwarded.message.downcast_ref::<SystemMessage>(),
            Some(SystemMessage::Suspend)
        ));
        assert!(forwarded.sender == receiver && forwarded.recipient == receiver);
    }
}

/// The dead-letter mailbox with a real system: what happens to system messages that reach an actor that already terminated
/// (oracle case `MBX-OR-05`, the line of `cases/oracle-only/mailbox.golden` that the Pekko 2.0.0 JVM printed).
#[cfg(test)]
mod with_real_actors {
    use super::*;
    use crate::actor::fault::Throwable;
    use crate::actor::props::{Actor, ActorContext, AnyMessage, Props};
    use crate::actor::system::ActorSystem;
    use crate::dispatch::sysmsg::Terminated;
    use crate::event::Channel;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    const ORACLE_GOLDEN: &str = include_str!("../../../../cases/oracle-only/mailbox.golden");

    type Log = Arc<Mutex<Vec<String>>>;

    fn name_of(message: &SystemMessage) -> &'static str {
        match message {
            SystemMessage::Create(_) => "Create",
            SystemMessage::Terminate => "Terminate",
            SystemMessage::Failed { .. } => "Failed",
            SystemMessage::Recreate { .. } => "Recreate",
            SystemMessage::Resume { .. } => "Resume",
            SystemMessage::Suspend => "Suspend",
            SystemMessage::Supervise { .. } => "Supervise",
            SystemMessage::Watch { .. } => "Watch",
            SystemMessage::Unwatch { .. } => "Unwatch",
            SystemMessage::DeathWatchNotification { .. } => "DeathWatchNotification",
        }
    }

    struct Idle;
    impl Actor for Idle {
        fn receive(&mut self, _ctx: &mut ActorContext, _msg: &AnyMessage) -> bool {
            true
        }
    }

    /// Records the system messages of every `DeadLetter` it receives.
    struct RecordsDeadLetters(Log);
    impl Actor for RecordsDeadLetters {
        fn receive(&mut self, _ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            let Some(dead) = msg.downcast_ref::<DeadLetter>() else {
                return false;
            };
            let name = dead
                .message
                .downcast_ref::<SystemMessage>()
                .map_or("user message", name_of);
            self.0.lock().unwrap().push(name.to_owned());
            true
        }
    }

    /// Watches `target` when told to, and records its `Terminated`.
    struct WatchesOnDemand {
        target: ActorRef,
        log: Log,
    }
    impl Actor for WatchesOnDemand {
        fn receive(&mut self, ctx: &mut ActorContext, msg: &AnyMessage) -> bool {
            if msg.is::<Terminated>() {
                self.log.lock().unwrap().push("terminated".to_owned());
            } else {
                ctx.watch(&self.target);
            }
            true
        }
    }

    fn wait_until(what: &str, condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition() {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// A system with a stopped actor `stopped` and a recorder subscribed to dead letters.
    fn system_with_a_stopped_actor(name: &str) -> (ActorSystem, ActorRef, Log) {
        let system = ActorSystem::create(name).unwrap();
        let dead: Log = Log::default();
        let recorder = system
            .actor_of(Props::new({
                let dead = dead.clone();
                move || RecordsDeadLetters(dead.clone())
            }))
            .unwrap();
        system
            .event_stream()
            .subscribe(&recorder, Channel::DeadLetter);
        let stopped = system
            .actor_of_named(Props::new(|| Idle), "stopped")
            .unwrap();
        system.stop(&stopped);
        wait_until("the actor to terminate", || stopped.is_terminated());
        (system, stopped, dead)
    }

    #[test]
    fn system_messages_after_close_are_dead_letters_and_a_watch_is_answered() {
        let (system, stopped, dead) = system_with_a_stopped_actor("or05");
        let watcher_log: Log = Log::default();
        let watcher = system
            .actor_of(Props::new({
                let (target, log) = (stopped.clone(), watcher_log.clone());
                move || WatchesOnDemand {
                    target: target.clone(),
                    log: log.clone(),
                }
            }))
            .unwrap();

        stopped.send_system(SystemMessage::Suspend);
        stopped.send_system(SystemMessage::Resume { cause: None });
        watcher.tell("watch", None);

        wait_until("two dead letters and the Terminated", || {
            dead.lock().unwrap().len() == 2 && !watcher_log.lock().unwrap().is_empty()
        });
        let observed = format!(
            "MBX-OR-05 system messages after close go to dead letters (Watch is answered) -> deadLetters=[{}] watcherLog=[{}]",
            dead.lock().unwrap().join(", "),
            watcher_log.lock().unwrap().join(", ")
        );
        let recorded = ORACLE_GOLDEN
            .lines()
            .find(|line| line.starts_with("MBX-OR-05 "))
            .expect("the golden has MBX-OR-05");
        assert_eq!(observed, recorded);
        system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap()
            .expect("terminated normally");
    }

    #[test]
    fn the_other_system_messages_are_dead_letters_too_but_unwatch_and_the_suppressed_ones_are_not()
    {
        let (system, stopped, dead) = system_with_a_stopped_actor("or05-rest");
        let cause = || Throwable::illegal_state("boom");
        let unwatch = SystemMessage::Unwatch {
            watchee: stopped.clone(),
            watcher: system.dead_letters(),
        };

        stopped.send_system(unwatch); // ignored by `deadLetters`
        stopped.send_system(SystemMessage::Terminate); // DeadLetterSuppression
        stopped.send_system(SystemMessage::DeathWatchNotification {
            actor: system.dead_letters(),
            existence_confirmed: true,
            address_terminated: false,
        }); // DeadLetterSuppression
        stopped.send_system(SystemMessage::Recreate { cause: cause() });
        stopped.send_system(SystemMessage::Failed {
            child: stopped.clone(),
            cause: cause(),
        });
        stopped.send_system(SystemMessage::Create(None));
        stopped.send_system(SystemMessage::Supervise {
            child: stopped.clone(),
            is_async: false,
        });

        wait_until("four dead letters", || dead.lock().unwrap().len() >= 4);
        std::thread::sleep(Duration::from_millis(100)); // nothing else may follow
        assert_eq!(
            *dead.lock().unwrap(),
            ["Recreate", "Failed", "Create", "Supervise"]
        );
        system
            .terminate()
            .wait(Duration::from_secs(10))
            .unwrap()
            .expect("terminated normally");
    }
}
