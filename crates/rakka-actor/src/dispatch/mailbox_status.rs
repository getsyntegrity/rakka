// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala (Apache-2.0).
//! `MailboxStatus`: the atomic state of a mailbox (`Mailbox.scala:40-56,125-260`).
//!
//! One word encodes the primary status (`Open` = 0 or `Closed` = 1), the `Scheduled` bit (2) and the suspend count in units of 4.
//! Every transition is a compare-and-swap loop, so concurrent callers never lose an update. `Closed` is exactly the value 1:
//! closing clears the `Scheduled` bit and the suspend count, and nothing leaves it. This type only models the state; the queues,
//! the dispatcher and the actor cell are separate pieces that arrive in later steps of ADR-0001.

use std::sync::atomic::{AtomicU32, Ordering};

/// The status word of a mailbox. The suspend count is held in `u32`: more than 2^30 nested suspensions is out of scope.
#[derive(Debug, Default)]
pub struct MailboxStatus(AtomicU32);

impl MailboxStatus {
    /// Primary status: the mailbox accepts and processes messages.
    pub const OPEN: u32 = 0;
    /// Primary status: terminal; the word is exactly this value.
    pub const CLOSED: u32 = 1;
    /// Secondary status bit: a turn is scheduled or running; may accompany `OPEN` and a suspend count.
    pub const SCHEDULED: u32 = 2;
    /// One suspension, as added to the word.
    pub const SUSPEND_UNIT: u32 = 4;
    const SHOULD_SCHEDULE_MASK: u32 = 3;
    const SHOULD_NOT_PROCESS_MASK: u32 = !2;
    const SUSPEND_MASK: u32 = !3;

    /// An open, idle, unsuspended mailbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// The raw word (`Mailbox.currentStatus`).
    pub fn current_status(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }

    /// `Mailbox.shouldProcessMessage`: neither suspended nor closed (the `Scheduled` bit is irrelevant).
    pub fn should_process_message(&self) -> bool {
        self.current_status() & Self::SHOULD_NOT_PROCESS_MASK == 0
    }

    /// `Mailbox.suspendCount`.
    pub fn suspend_count(&self) -> u32 {
        self.current_status() / Self::SUSPEND_UNIT
    }

    /// `Mailbox.isSuspended`.
    pub fn is_suspended(&self) -> bool {
        self.current_status() & Self::SUSPEND_MASK != 0
    }

    /// `Mailbox.isClosed`: the word is exactly `CLOSED`.
    pub fn is_closed(&self) -> bool {
        self.current_status() == Self::CLOSED
    }

    /// `Mailbox.isScheduled`.
    pub fn is_scheduled(&self) -> bool {
        self.current_status() & Self::SCHEDULED != 0
    }

    fn update(&self, old: u32, new: u32) -> bool {
        self.0
            .compare_exchange(old, new, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Reduces the suspend count by one, never below zero (`Mailbox.resume`). Returns whether the count is now zero; `false` on a
    /// closed mailbox, which does not change.
    pub fn resume(&self) -> bool {
        let mut status = self.current_status();
        loop {
            if status == Self::CLOSED {
                return false;
            }
            let next = if status < Self::SUSPEND_UNIT {
                status
            } else {
                status - Self::SUSPEND_UNIT
            };
            if self.update(status, next) {
                return next < Self::SUSPEND_UNIT;
            }
            std::hint::spin_loop();
            status = self.current_status();
        }
    }

    /// Increments the suspend count (`Mailbox.suspend`). Returns whether the previous count was zero; `false` on a closed mailbox.
    pub fn suspend(&self) -> bool {
        let mut status = self.current_status();
        loop {
            if status == Self::CLOSED {
                return false;
            }
            if self.update(status, status + Self::SUSPEND_UNIT) {
                return status < Self::SUSPEND_UNIT;
            }
            std::hint::spin_loop();
            status = self.current_status();
        }
    }

    /// Makes the mailbox `Closed` for good (`Mailbox.becomeClosed`). Returns `true` only for the call that closed it.
    pub fn become_closed(&self) -> bool {
        let mut status = self.current_status();
        loop {
            if status == Self::CLOSED {
                return false;
            }
            if self.update(status, Self::CLOSED) {
                return true;
            }
            std::hint::spin_loop();
            status = self.current_status();
        }
    }

    /// Sets the `Scheduled` bit if the mailbox is neither closed nor already scheduled; a suspended mailbox can be scheduled
    /// (`Mailbox.setAsScheduled`). The only door to a turn: at most one caller wins until `set_as_idle`.
    pub fn set_as_scheduled(&self) -> bool {
        let mut status = self.current_status();
        loop {
            if status & Self::SHOULD_SCHEDULE_MASK != Self::OPEN {
                return false;
            }
            if self.update(status, status | Self::SCHEDULED) {
                return true;
            }
            std::hint::spin_loop();
            status = self.current_status();
        }
    }

    /// Clears the `Scheduled` bit, whatever the primary status (`Mailbox.setAsIdle`); always `true`.
    pub fn set_as_idle(&self) -> bool {
        self.0.fetch_and(!Self::SCHEDULED, Ordering::SeqCst);
        true
    }

    /// `Mailbox.canBeScheduledForExecution`. The queue facts are closures, as in Pekko they are only consulted when the answer
    /// needs them: never for a closed mailbox, and only the system queue for a suspended one.
    pub fn can_be_scheduled_for_execution(
        &self,
        has_message_hint: bool,
        has_system_message_hint: bool,
        has_system_messages: impl FnOnce() -> bool,
        has_messages: impl FnOnce() -> bool,
    ) -> bool {
        match self.current_status() {
            Self::OPEN | Self::SCHEDULED => {
                has_message_hint
                    || has_system_message_hint
                    || has_system_messages()
                    || has_messages()
            }
            Self::CLOSED => false,
            _ => has_system_message_hint || has_system_messages(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn status_with(scheduled: bool, suspensions: u32, closed: bool) -> MailboxStatus {
        let status = MailboxStatus::new();
        if scheduled {
            assert!(status.set_as_scheduled());
        }
        for _ in 0..suspensions {
            status.suspend();
        }
        if closed {
            assert!(status.become_closed());
        }
        status
    }

    fn run_on_threads(
        threads: usize,
        work: impl Fn(usize) -> bool + Send + Sync + 'static,
    ) -> usize {
        let work = Arc::new(work);
        let start = Arc::new(Barrier::new(threads));
        let handles: Vec<_> = (0..threads)
            .map(|n| {
                let (work, start) = (Arc::clone(&work), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    work(n)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(|&won| won)
            .count()
    }

    #[test]
    fn a_new_mailbox_is_open_idle_and_not_suspended() {
        let status = MailboxStatus::new();
        assert_eq!(status.current_status(), MailboxStatus::OPEN);
        assert!(!status.is_closed() && !status.is_scheduled() && !status.is_suspended());
        assert_eq!(status.suspend_count(), 0);
        assert!(status.should_process_message());
    }

    #[test]
    fn set_as_scheduled_succeeds_once_until_it_is_idle_again() {
        let status = MailboxStatus::new();
        assert!(status.set_as_scheduled());
        assert!(!status.set_as_scheduled());
        assert_eq!(status.current_status(), MailboxStatus::SCHEDULED);
        assert!(status.set_as_idle());
        assert!(status.set_as_idle(), "set_as_idle is unconditional");
        assert_eq!(status.current_status(), MailboxStatus::OPEN);
        assert!(status.set_as_scheduled());
    }

    #[test]
    fn suspend_counts_in_units_of_four_and_reports_only_the_first() {
        let status = MailboxStatus::new();
        assert!(status.suspend());
        assert!(!status.suspend());
        assert_eq!((status.suspend_count(), status.current_status()), (2, 8));
        assert!(status.is_suspended() && !status.should_process_message());
    }

    #[test]
    fn resume_reports_only_the_last_and_never_goes_below_zero() {
        let status = status_with(false, 2, false);
        assert!(!status.resume());
        assert!(status.resume());
        assert_eq!(status.suspend_count(), 0);
        assert!(
            status.resume(),
            "resume at zero reports zero and keeps zero"
        );
        assert_eq!(status.current_status(), MailboxStatus::OPEN);
    }

    #[test]
    fn a_suspended_mailbox_can_still_be_scheduled_and_keeps_the_bit_on_resume() {
        let status = status_with(false, 1, false);
        assert!(status.set_as_scheduled());
        assert_eq!(status.current_status(), 6);
        assert!(!status.can_be_scheduled_for_execution(false, false, || false, || true));
        assert!(status.can_be_scheduled_for_execution(false, true, || false, || false));
        assert!(status.resume());
        assert_eq!(status.current_status(), MailboxStatus::SCHEDULED);
    }

    #[test]
    fn become_closed_is_terminal_and_clears_the_scheduled_bit_and_the_suspend_count() {
        let status = status_with(true, 2, false);
        assert_eq!(status.current_status(), 10);
        assert!(status.become_closed());
        assert!(!status.become_closed());
        assert_eq!(status.current_status(), MailboxStatus::CLOSED);
        assert!(status.is_closed() && !status.is_scheduled() && !status.is_suspended());
        assert!(!status.should_process_message());
    }

    #[test]
    fn a_closed_mailbox_ignores_suspend_resume_and_scheduling_and_stays_closed_when_set_idle() {
        let status = status_with(false, 0, true);
        assert!(!status.resume());
        assert!(!status.suspend());
        assert!(!status.set_as_scheduled());
        assert!(status.set_as_idle());
        assert_eq!(status.current_status(), MailboxStatus::CLOSED);
        assert!(!status.can_be_scheduled_for_execution(true, true, || true, || true));
    }

    #[test]
    fn can_be_scheduled_for_execution_follows_the_state_and_the_hints() {
        // (scheduled, suspensions, closed) -> answers for hints (msg, sys) = FF, FT, TF, TT with empty queues
        let table: [((bool, u32, bool), [bool; 4]); 5] = [
            ((false, 0, false), [false, true, true, true]),
            ((true, 0, false), [false, true, true, true]),
            ((false, 1, false), [false, true, false, true]),
            ((true, 1, false), [false, true, false, true]),
            ((false, 0, true), [false; 4]),
        ];
        for ((scheduled, suspensions, closed), expected) in table {
            let status = status_with(scheduled, suspensions, closed);
            let answers =
                [(false, false), (false, true), (true, false), (true, true)].map(|(msg, sys)| {
                    status.can_be_scheduled_for_execution(msg, sys, || false, || false)
                });
            assert_eq!(
                answers, expected,
                "state ({scheduled}, {suspensions}, {closed})"
            );
        }
    }

    #[test]
    fn queued_work_makes_an_open_mailbox_schedulable_but_a_suspended_one_only_for_system_messages()
    {
        let open = MailboxStatus::new();
        assert!(open.can_be_scheduled_for_execution(false, false, || false, || true));
        assert!(open.can_be_scheduled_for_execution(false, false, || true, || false));
        let suspended = status_with(false, 1, false);
        assert!(!suspended.can_be_scheduled_for_execution(false, false, || false, || true));
        assert!(suspended.can_be_scheduled_for_execution(false, false, || true, || false));
    }

    #[test]
    fn the_queues_are_not_consulted_when_the_answer_does_not_need_them() {
        let closed = status_with(false, 0, true);
        assert!(!closed.can_be_scheduled_for_execution(
            false,
            false,
            || panic!("system queue"),
            || panic!("user queue")
        ));
        let open = MailboxStatus::new();
        assert!(open.can_be_scheduled_for_execution(
            true,
            false,
            || panic!("system queue"),
            || panic!("user queue")
        ));
        let suspended = status_with(false, 1, false);
        assert!(suspended.can_be_scheduled_for_execution(
            false,
            false,
            || true,
            || panic!("user queue")
        ));
    }

    #[test]
    fn concurrent_suspends_lose_no_increment_and_exactly_one_is_the_first() {
        let status = Arc::new(MailboxStatus::new());
        let shared = Arc::clone(&status);
        let first = run_on_threads(8, move |_| {
            (0..1000).map(|_| shared.suspend()).filter(|&f| f).count() == 1
        });
        assert_eq!(first, 1);
        assert_eq!(status.suspend_count(), 8000);
    }

    #[test]
    fn concurrent_resumes_balance_the_suspends_and_exactly_one_reaches_zero() {
        let status = Arc::new(status_with(false, 8000, false));
        let shared = Arc::clone(&status);
        let last = run_on_threads(8, move |_| {
            (0..1000).map(|_| shared.resume()).filter(|&z| z).count() == 1
        });
        assert_eq!(last, 1);
        assert_eq!(status.current_status(), MailboxStatus::OPEN);
    }

    #[test]
    fn exactly_one_of_many_concurrent_schedulers_wins() {
        let status = Arc::new(MailboxStatus::new());
        let shared = Arc::clone(&status);
        assert_eq!(run_on_threads(16, move |_| shared.set_as_scheduled()), 1);
        assert!(status.is_scheduled());
    }

    #[test]
    fn exactly_one_of_many_concurrent_closers_wins() {
        let status = Arc::new(status_with(true, 3, false));
        let shared = Arc::clone(&status);
        assert_eq!(run_on_threads(16, move |_| shared.become_closed()), 1);
        assert_eq!(status.current_status(), MailboxStatus::CLOSED);
    }

    #[test]
    fn closing_while_others_suspend_resume_and_schedule_always_ends_closed() {
        for _ in 0..200 {
            let status = Arc::new(MailboxStatus::new());
            let shared = Arc::clone(&status);
            run_on_threads(4, move |n| {
                match n {
                    0 => return shared.become_closed(),
                    1 => (0..50).for_each(|_| {
                        shared.suspend();
                    }),
                    2 => (0..50).for_each(|_| {
                        shared.resume();
                    }),
                    _ => (0..50).for_each(|_| {
                        shared.set_as_scheduled();
                        shared.set_as_idle();
                    }),
                }
                false
            });
            assert_eq!(status.current_status(), MailboxStatus::CLOSED);
        }
    }
}
