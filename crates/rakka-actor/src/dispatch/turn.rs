// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala and actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala (Apache-2.0).
//! The limits of one mailbox turn: `throughput` and `throughput-deadline-time` (`Mailbox.processMailbox`, `Mailbox.scala:272-292`).
//!
//! A turn processes user messages while the mailbox may process them. After each message (and the system messages that
//! follow it) the turn goes on only if more than one message was left in its budget *and*, when a deadline is defined, the
//! clock has not reached the deadline computed when the turn began. The first message of a turn is always processed.
//! The clock is `System.nanoTime` in Pekko; here it is an injected [`Clock`], so tests need no sleeps.

use std::time::Duration;

use crate::util::clock::Clock;

/// What a dispatcher gives its mailboxes for one turn: `throughput` (at least 1 is used) and the optional deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TurnPolicy {
    throughput: usize,
    deadline_nanos: Option<i64>,
}

impl TurnPolicy {
    /// `isThroughputDeadlineTimeDefined = throughputDeadlineTime.toMillis > 0` (`AbstractDispatcher.scala:372`): a deadline
    /// shorter than a millisecond counts as no deadline.
    pub(crate) fn new(throughput: usize, deadline: Duration) -> Self {
        let deadline_nanos = (deadline.as_millis() > 0)
            .then(|| i64::try_from(deadline.as_nanos()).unwrap_or(i64::MAX));
        TurnPolicy {
            throughput,
            deadline_nanos,
        }
    }

    /// Starts a turn: the budget is `max(throughput, 1)` and the deadline is `now + throughputDeadlineTime`.
    pub(crate) fn begin(&self, clock: &dyn Clock) -> Turn {
        Turn {
            left: self.throughput.max(1),
            deadline: self
                .deadline_nanos
                .map(|nanos| clock.current_time().wrapping_add(nanos)),
        }
    }
}

/// One mailbox turn in progress.
#[derive(Debug)]
pub(crate) struct Turn {
    left: usize,
    deadline: Option<i64>,
}

impl Turn {
    /// Called after a message was processed: whether the turn may take another one.
    pub(crate) fn allows_another(&mut self, clock: &dyn Clock) -> bool {
        if self.left <= 1 {
            return false;
        }
        // `(System.nanoTime - deadlineNs) < 0`: a difference, so the wrap-around of the counter does not matter
        if let Some(deadline) = self.deadline
            && clock.current_time().wrapping_sub(deadline) >= 0
        {
            return false;
        }
        self.left -= 1;
        true
    }
}

#[cfg(test)]
pub(crate) mod test_clock {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    /// A clock tests move by hand; `earlier_time` is never used by the mailbox.
    #[derive(Debug, Default)]
    pub(crate) struct ManualClock(AtomicI64);

    impl ManualClock {
        pub(crate) fn new(start_nanos: i64) -> Self {
            ManualClock(AtomicI64::new(start_nanos))
        }
        pub(crate) fn advance(&self, by: Duration) {
            self.0
                .fetch_add(i64::try_from(by.as_nanos()).unwrap(), Ordering::SeqCst);
        }
    }

    impl Clock for ManualClock {
        fn current_time(&self) -> i64 {
            self.0.load(Ordering::SeqCst)
        }
        fn earlier_time(&self, duration_nanos: i64) -> i64 {
            self.current_time() - duration_nanos
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_clock::ManualClock;
    use super::*;

    const NO_DEADLINE: Duration = Duration::ZERO;

    /// How many messages a turn processes when every one of them takes `each` of (fake) time.
    fn messages_processed(policy: TurnPolicy, clock: &ManualClock, each: Duration) -> usize {
        let mut turn = policy.begin(clock);
        let mut processed = 1; // the first message is always processed
        clock.advance(each);
        while turn.allows_another(clock) {
            processed += 1;
            clock.advance(each);
        }
        processed
    }

    #[test]
    fn a_turn_processes_throughput_messages_when_there_is_no_deadline() {
        let clock = ManualClock::new(0);
        assert_eq!(
            messages_processed(
                TurnPolicy::new(5, NO_DEADLINE),
                &clock,
                Duration::from_secs(60)
            ),
            5
        );
    }

    #[test]
    fn a_throughput_of_zero_or_one_still_processes_one_message() {
        let clock = ManualClock::new(0);
        assert_eq!(
            messages_processed(TurnPolicy::new(0, NO_DEADLINE), &clock, Duration::ZERO),
            1
        );
        assert_eq!(
            messages_processed(TurnPolicy::new(1, NO_DEADLINE), &clock, Duration::ZERO),
            1
        );
    }

    #[test]
    fn the_deadline_ends_the_turn_even_with_throughput_left() {
        let clock = ManualClock::new(0);
        let policy = TurnPolicy::new(100, Duration::from_millis(30));
        // 20 ms per message: after message 1 (t=20) and 2 (t=40) the deadline at t=30 has passed -> 2 messages
        assert_eq!(
            messages_processed(policy, &clock, Duration::from_millis(20)),
            2
        );
    }

    #[test]
    fn the_first_message_is_processed_even_if_it_alone_exceeds_the_deadline() {
        let clock = ManualClock::new(0);
        let policy = TurnPolicy::new(100, Duration::from_millis(30));
        assert_eq!(
            messages_processed(policy, &clock, Duration::from_millis(60)),
            1
        );
    }

    #[test]
    fn reaching_the_deadline_exactly_ends_the_turn() {
        let clock = ManualClock::new(1_000);
        let policy = TurnPolicy::new(100, Duration::from_millis(30));
        let mut turn = policy.begin(&clock);
        clock.advance(Duration::from_millis(29));
        assert!(
            turn.allows_another(&clock),
            "one nanosecond-millisecond before the deadline"
        );
        clock.advance(Duration::from_millis(1)); // now == deadline: `now - deadline < 0` is false
        assert!(!turn.allows_another(&clock));
    }

    #[test]
    fn the_throughput_limit_wins_when_the_deadline_is_far() {
        let clock = ManualClock::new(0);
        let policy = TurnPolicy::new(3, Duration::from_secs(10));
        assert_eq!(
            messages_processed(policy, &clock, Duration::from_millis(1)),
            3
        );
    }

    #[test]
    fn a_deadline_shorter_than_a_millisecond_is_no_deadline() {
        let clock = ManualClock::new(0);
        let policy = TurnPolicy::new(4, Duration::from_micros(999));
        assert_eq!(policy, TurnPolicy::new(4, NO_DEADLINE));
        assert_eq!(
            messages_processed(policy, &clock, Duration::from_secs(1)),
            4
        );
    }

    #[test]
    fn the_deadline_survives_the_wrap_around_of_the_clock() {
        let clock = ManualClock::new(i64::MAX - 10_000_000); // 10 ms before the counter wraps
        let policy = TurnPolicy::new(100, Duration::from_millis(30));
        assert_eq!(
            messages_processed(policy, &clock, Duration::from_millis(20)),
            2
        );
    }

    #[test]
    fn the_clock_is_not_read_once_the_throughput_is_spent() {
        struct Panicking;
        impl Clock for Panicking {
            fn current_time(&self) -> i64 {
                panic!("the clock must not be read");
            }
            fn earlier_time(&self, _: i64) -> i64 {
                panic!("unused")
            }
        }
        let mut turn = TurnPolicy::new(1, NO_DEADLINE).begin(&Panicking);
        assert!(!turn.allows_another(&Panicking));
    }
}
