// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala (Apache-2.0).
//! Supervision: directives, `OneForOneStrategy` / `AllForOneStrategy` and the restart limit of a child.
//! Follows `SupervisorStrategy.scala` (`OneForOneStrategy`, `defaultDecider`, `ChildRestartStats`).

use std::sync::Arc;

use crate::actor::fault::Throwable;

/// `SupervisorStrategy.Directive`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directive {
    /// Keep the instance and its state; the failed message is lost, the next one is processed.
    Resume,
    /// Replace the instance (`preRestart`, new instance, `postRestart`).
    Restart,
    /// Stop the child.
    Stop,
    /// The supervisor does not decide: it fails itself with the child's exception and its own supervisor decides about the supervisor.
    Escalate,
}

/// Maps what a child threw to a [`Directive`] (Pekko's `Decider`, a total function here).
pub type Decider = Arc<dyn Fn(&Throwable) -> Directive + Send + Sync>;

/// `SupervisorStrategy.defaultDecider`: `ActorInitializationException`, `ActorKilledException` and
/// `DeathPactException`, subtypes included, stop the child; any other `Exception` restarts it. What is not an `Exception`
/// escalates (Pekko's decider is not defined there and `handleFailure` applies `escalateDefault`; a [`Decider`] is total).
pub fn default_decider(cause: &Throwable) -> Directive {
    if cause.is_a("org.apache.pekko.actor.ActorInitializationException")
        || cause.is_a("org.apache.pekko.actor.ActorKilledException")
        || cause.is_a("org.apache.pekko.actor.DeathPactException")
    {
        Directive::Stop
    } else if cause.is_a("java.lang.Exception") {
        Directive::Restart
    } else {
        Directive::Escalate
    }
}

/// The decider of `SupervisorStrategy.stoppingStrategy`: every `Exception` stops the child, the rest escalates.
fn stopping_decider(cause: &Throwable) -> Directive {
    if cause.is_a("java.lang.Exception") {
        Directive::Stop
    } else {
        Directive::Escalate
    }
}

/// `OneForOneStrategy(maxNrOfRetries, decider)`: the directive applies to the failed child only.
/// `max_nr_of_retries < 0` means unlimited (Pekko's `-1`); `0` means the first restart already exceeds the limit.
/// (The `withinTimeRange` window is not ported yet.)
#[derive(Clone)]
pub struct OneForOneStrategy {
    pub max_nr_of_retries: i32,
    pub decider: Decider,
    /// `loggingEnabled`: whether the failures this strategy handles are published (default `true`).
    pub logging_enabled: bool,
}

impl OneForOneStrategy {
    pub fn new(
        max_nr_of_retries: i32,
        decider: impl Fn(&Throwable) -> Directive + Send + Sync + 'static,
    ) -> Self {
        OneForOneStrategy {
            max_nr_of_retries,
            decider: Arc::new(decider),
            logging_enabled: true,
        }
    }

    /// The same strategy with `loggingEnabled` set to `logging_enabled`.
    pub fn with_logging_enabled(mut self, logging_enabled: bool) -> Self {
        self.logging_enabled = logging_enabled;
        self
    }

    /// `SupervisorStrategy.stoppingStrategy`: unlimited, one for one, every failed `Exception` stops its child.
    pub fn stopping_strategy() -> Self {
        OneForOneStrategy::new(-1, stopping_decider)
    }

    /// `SupervisorStrategy.defaultStrategy`: unlimited retries, [`default_decider`].
    pub fn default_strategy() -> Self {
        OneForOneStrategy::new(-1, default_decider)
    }
}

/// `AllForOneStrategy(maxNrOfRetries, decider)`: Resume applies to the failed child only; Restart and Stop (and the stop that follows an exhausted
/// restart limit) apply to **all** children of the supervisor. Same parameters as [`OneForOneStrategy`].
#[derive(Clone)]
pub struct AllForOneStrategy {
    pub max_nr_of_retries: i32,
    pub decider: Decider,
    /// `loggingEnabled`: whether the failures this strategy handles are published (default `true`).
    pub logging_enabled: bool,
}

impl AllForOneStrategy {
    pub fn new(
        max_nr_of_retries: i32,
        decider: impl Fn(&Throwable) -> Directive + Send + Sync + 'static,
    ) -> Self {
        AllForOneStrategy {
            max_nr_of_retries,
            decider: Arc::new(decider),
            logging_enabled: true,
        }
    }

    /// The same strategy with `loggingEnabled` set to `logging_enabled`.
    pub fn with_logging_enabled(mut self, logging_enabled: bool) -> Self {
        self.logging_enabled = logging_enabled;
        self
    }
}

/// `SupervisorStrategy`: what [`Actor::supervisor_strategy`](crate::props::Actor::supervisor_strategy) returns.
#[derive(Clone)]
pub enum SupervisorStrategy {
    OneForOne(OneForOneStrategy),
    AllForOne(AllForOneStrategy),
}

impl SupervisorStrategy {
    pub fn max_nr_of_retries(&self) -> i32 {
        match self {
            SupervisorStrategy::OneForOne(s) => s.max_nr_of_retries,
            SupervisorStrategy::AllForOne(s) => s.max_nr_of_retries,
        }
    }

    pub fn decide(&self, cause: &Throwable) -> Directive {
        match self {
            SupervisorStrategy::OneForOne(s) => (s.decider)(cause),
            SupervisorStrategy::AllForOne(s) => (s.decider)(cause),
        }
    }

    /// `loggingEnabled` of the strategy.
    pub fn logging_enabled(&self) -> bool {
        match self {
            SupervisorStrategy::OneForOne(s) => s.logging_enabled,
            SupervisorStrategy::AllForOne(s) => s.logging_enabled,
        }
    }

    /// `getClass.getName`: the `logClass` of what the strategy logs.
    pub(crate) fn class_name(&self) -> &'static str {
        match self {
            SupervisorStrategy::OneForOne(_) => "org.apache.pekko.actor.OneForOneStrategy",
            SupervisorStrategy::AllForOne(_) => "org.apache.pekko.actor.AllForOneStrategy",
        }
    }

    /// `true` for [`AllForOneStrategy`]: a Restart or Stop affects every child.
    pub fn is_all_for_one(&self) -> bool {
        matches!(self, SupervisorStrategy::AllForOne(_))
    }
}

impl From<OneForOneStrategy> for SupervisorStrategy {
    fn from(s: OneForOneStrategy) -> Self {
        SupervisorStrategy::OneForOne(s)
    }
}

impl From<AllForOneStrategy> for SupervisorStrategy {
    fn from(s: AllForOneStrategy) -> Self {
        SupervisorStrategy::AllForOne(s)
    }
}

impl Default for SupervisorStrategy {
    fn default() -> Self {
        OneForOneStrategy::default_strategy().into()
    }
}

impl Default for OneForOneStrategy {
    fn default() -> Self {
        OneForOneStrategy::default_strategy()
    }
}

/// `ChildRestartStats`, restart counter only (the `withinTimeRange` window is not ported yet).
#[derive(Debug, Default, Clone)]
pub(crate) struct RestartStats {
    count: i32,
}

impl RestartStats {
    /// `requestRestartPermission`: `max_nr_of_retries < 0` is unlimited, `0` never allows a restart, otherwise the
    /// first `max_nr_of_retries` requests are granted and every later one is refused.
    pub fn request_restart_permission(&mut self, max_nr_of_retries: i32) -> bool {
        if max_nr_of_retries < 0 {
            return true; // `maxNrOfRetriesOption`: a negative limit is `None` (unlimited)
        }
        if max_nr_of_retries < 1 {
            return false;
        }
        self.count += 1;
        self.count <= max_nr_of_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grants(max: i32, requests: usize) -> Vec<bool> {
        let mut stats = RestartStats::default();
        (0..requests)
            .map(|_| stats.request_restart_permission(max))
            .collect()
    }

    #[test]
    fn zero_retries_refuse_the_first_restart() {
        assert_eq!(grants(0, 3), [false, false, false]);
    }

    #[test]
    fn the_last_allowed_restart_is_granted_and_the_first_beyond_it_is_refused() {
        assert_eq!(grants(1, 3), [true, false, false]);
        assert_eq!(grants(2, 4), [true, true, false, false]);
        assert_eq!(grants(3, 5), [true, true, true, false, false]);
    }

    #[test]
    fn a_negative_limit_is_unlimited() {
        assert!(grants(-1, 50).into_iter().all(|g| g));
        assert!(grants(-5, 50).into_iter().all(|g| g));
    }

    #[test]
    fn each_child_has_its_own_counter() {
        let (mut a, mut b) = (RestartStats::default(), RestartStats::default());
        assert!(a.request_restart_permission(1));
        assert!(!a.request_restart_permission(1));
        assert!(b.request_restart_permission(1), "b is unaffected by a");
    }

    #[test]
    fn a_strategy_logs_unless_it_is_disabled_and_names_its_own_class() {
        let one = OneForOneStrategy::default_strategy();
        let all = AllForOneStrategy::new(-1, default_decider);
        let quiet_one = SupervisorStrategy::from(one.clone().with_logging_enabled(false));
        let quiet_all = SupervisorStrategy::from(all.clone().with_logging_enabled(false));
        assert!(!quiet_one.logging_enabled() && !quiet_all.logging_enabled());
        for (strategy, class) in [(one.into(), "OneForOne"), (all.into(), "AllForOne")] {
            let strategy: SupervisorStrategy = strategy;
            assert!(strategy.logging_enabled());
            let expected = format!("org.apache.pekko.actor.{class}Strategy");
            assert_eq!(strategy.class_name(), expected);
        }
    }

    #[test]
    fn the_default_decider_escalates_what_is_not_an_exception() {
        for class in ["java.lang.Error", "java.lang.Throwable"] {
            let cause = Throwable::new(class, Some("boom"));
            assert_eq!(default_decider(&cause), Directive::Escalate, "{class}");
        }
    }

    #[test]
    fn the_default_decider_decides_by_subtype() {
        let of = |class: &str, superclass: &str| {
            Throwable::new(class, None).with_superclasses([superclass])
        };
        let actor = |name: &str| format!("org.apache.pekko.actor.{name}");
        for (cause, expected) in [
            (
                of("a.Init", &actor("ActorInitializationException")),
                Directive::Stop,
            ),
            (
                of("a.Kill", &actor("ActorKilledException")),
                Directive::Stop,
            ),
            (of("a.Pact", &actor("DeathPactException")), Directive::Stop),
            (Throwable::new("a.Unknown", None), Directive::Restart),
            (of("a.Checked", "java.lang.Exception"), Directive::Restart),
            (of("a.Assert", "java.lang.Error"), Directive::Escalate),
            (of("a.Bare", "java.lang.Throwable"), Directive::Escalate),
        ] {
            assert_eq!(default_decider(&cause), expected, "{}", cause.class);
        }
    }

    /// AC-SUP-005: `{ case _: Exception => Stop }`; what it does not cover escalates.
    #[test]
    fn the_stopping_strategy_stops_every_exception_and_escalates_the_rest() {
        let s = OneForOneStrategy::stopping_strategy();
        assert_eq!(s.max_nr_of_retries, -1);
        assert!(s.logging_enabled);
        assert_eq!((s.decider)(&Throwable::illegal_state("x")), Directive::Stop);
        assert_eq!(
            (s.decider)(&Throwable::new("a.Unknown", None)),
            Directive::Stop
        );
        assert_eq!(
            (s.decider)(&Throwable::new("java.lang.Error", None)),
            Directive::Escalate
        );
    }

    #[test]
    fn the_default_strategy_is_unlimited_and_decides_by_class() {
        let s = OneForOneStrategy::default();
        assert_eq!(s.max_nr_of_retries, -1);
        assert_eq!(
            (s.decider)(&Throwable::illegal_state("x")),
            Directive::Restart
        );
        for stop in [
            "ActorKilledException",
            "DeathPactException",
            "ActorInitializationException",
            "PostRestartException",
            "PreRestartException",
        ] {
            assert_eq!(
                default_decider(&Throwable::new(
                    format!("org.apache.pekko.actor.{stop}"),
                    None
                )),
                Directive::Stop,
                "{stop}"
            );
        }
    }
}
