// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:74-78 (Apache-2.0).
//! The configurators of `oracle/src/main/java/pilot/GuardianStrategies.java`, registered under the same names.

use rakka_actor::actor::fault::Throwable;
use rakka_actor::actor::supervision::{
    AllForOneStrategy, Directive, OneForOneStrategy, SupervisorStrategies, default_decider,
};

/// The exception message names the directive; any other message gets the default decider.
fn by_message(cause: &Throwable) -> Directive {
    let message = cause.message().unwrap_or("null");
    [
        ("resume", Directive::Resume),
        ("restart", Directive::Restart),
        ("stop", Directive::Stop),
        ("escalate", Directive::Escalate),
    ]
    .into_iter()
    .find(|(prefix, _)| message.starts_with(prefix))
    .map_or_else(|| default_decider(cause), |(_, directive)| directive)
}

pub fn register(strategies: &mut SupervisorStrategies) {
    let name = |simple: &str| format!("pilot.GuardianStrategies${simple}");
    strategies.register(name("Escalate"), || {
        OneForOneStrategy::new(-1, |_| Directive::Escalate).into()
    });
    strategies.register(name("ByMessage"), || {
        OneForOneStrategy::new(-1, by_message).into()
    });
    strategies.register(name("OneRetry"), || {
        OneForOneStrategy::new(1, by_message).into()
    });
    strategies.register(name("AllForOne"), || {
        AllForOneStrategy::new(-1, by_message).into()
    });
    strategies.register(name("Quiet"), || {
        OneForOneStrategy::new(-1, by_message)
            .with_logging_enabled(false)
            .into()
    });
}
