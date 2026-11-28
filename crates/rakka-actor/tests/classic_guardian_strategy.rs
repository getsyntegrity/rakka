// Derived from Apache Pekko 2.0.0 actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala:353-392 (Apache-2.0).
//! `pekko.actor.guardian-supervisor-strategy`: the strategy of `/user`, chosen by name in the configuration.

#[allow(dead_code)] // each test binary uses only some of the shared helpers
mod common;
use common::RunningSystem;

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use rakka_actor::actor::ActorRef;
use rakka_actor::actor::fault::{Throwable, throw};
use rakka_actor::actor::props::{Actor, ActorContext, AnyMessage, Props};
use rakka_actor::actor::supervision::{Directive, OneForOneStrategy};
use rakka_actor::actor::system::{ActorSystem, Settings};
use rakka_actor::config::Config;
use rakka_actor::dispatch::Terminated;
use rakka_actor::event::{Channel, ErrorEvent};

const WAIT: Duration = Duration::from_secs(10);

/// The settings of a system configured with `application` over `reference.conf`.
fn configured(application: &str) -> Settings {
    let config = Config::parse_string(application)
        .unwrap()
        .with_fallback(&Config::default_reference())
        .resolve()
        .unwrap();
    Settings::from_config(&rakka_actor::config::Settings::new("guardian", config).unwrap()).unwrap()
}

/// Throws `Exception("hello")` on every message.
struct Dies;

impl Actor for Dies {
    fn receive(&mut self, _: &mut ActorContext, _: &AnyMessage) -> bool {
        throw(Throwable::new("java.lang.Exception", Some("hello")))
    }
}

/// The spec's `testActor` watching `target`: says on `watching` that the watch is sent, then sends on every `Terminated`.
struct Probe {
    target: ActorRef,
    watching: Sender<()>,
    terminations: Sender<(ActorRef, bool, bool)>,
}

impl Actor for Probe {
    fn pre_start(&mut self, ctx: &mut ActorContext) {
        ctx.watch(&self.target);
        self.watching.send(()).unwrap();
    }
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(t) = msg.downcast_ref::<Terminated>() else {
            return false;
        };
        let seen = (t.actor.clone(), t.existence_confirmed, t.address_terminated);
        self.terminations.send(seen).unwrap();
        true
    }
}

/// The spec's `EventFilter[Exception]("hello")`: sends on the message of every published `Error` with that cause.
struct ErrorFilter(Sender<Option<String>>);

impl Actor for ErrorFilter {
    fn receive(&mut self, _: &mut ActorContext, msg: &AnyMessage) -> bool {
        let Some(error) = msg.downcast_ref::<ErrorEvent>() else {
            return false;
        };
        if error.cause.is_a("java.lang.Exception") && error.cause.message() == Some("hello") {
            self.0.send(error.message.clone()).unwrap();
        }
        true
    }
}

/// `ActorSystemSpec` "allow configuration of guardian supervisor strategy" (AC-SYS-008 A2, AC-SUP-005 A1).
#[test]
fn a_stopping_guardian_strategy_stops_the_failed_top_level_actor() {
    let settings = configured(
        "pekko.actor.guardian-supervisor-strategy=org.apache.pekko.actor.StoppingSupervisorStrategy",
    );
    let system = RunningSystem::create_with_settings("t-guardian-stopping", settings);
    let child = system.actor_of(Props::new(|| Dies)).unwrap();
    let (logged, errors) = channel();
    let filter = system
        .actor_of(Props::new(move || ErrorFilter(logged.clone())))
        .unwrap();
    system.event_stream().subscribe(&filter, Channel::Error);
    let (watching, watched) = channel();
    let (seen, terminations) = channel();
    let target = child.clone();
    let probe = move || Probe {
        target: target.clone(),
        watching: watching.clone(),
        terminations: seen.clone(),
    };
    system.actor_of(Props::new(probe)).unwrap();
    watched.recv_timeout(WAIT).expect("the probe watches");

    child.tell("die", None);

    assert_eq!(terminations.recv_timeout(WAIT), Ok((child, true, false)));
    // `occurrences = 1`
    assert_eq!(errors.recv_timeout(WAIT), Ok(Some("hello".to_owned())));
    assert!(errors.recv_timeout(Duration::from_millis(150)).is_err());
    assert!(!system.when_terminated().is_completed());
}

/// `ActorSystemSpec` "shut down when /user escalates" (AC-SYS-008 A3, AC-SUP-008 A2): the spec names its own configurator.
/// Its `EventFilter[Exception]("hello")` is not asserted here, since a listener under `/user` dies with it: the in-crate
/// `guardian_failure_tests` assert the logged errors against the oracle (MBX-GRD-05).
#[test]
fn a_registered_strategy_that_escalates_ends_the_system_when_a_top_level_actor_fails() {
    let name = "org.apache.pekko.actor.ActorSystemSpec$Strategy";
    let mut settings = configured(&format!(
        "pekko.actor.guardian-supervisor-strategy=\"{name}\""
    ));
    settings.supervisor_strategies.register(name, || {
        OneForOneStrategy::new(-1, |_| Directive::Escalate).into()
    });
    let system = ActorSystem::create_with_settings("t-guardian-escalate", settings).unwrap();
    let child = system.actor_of(Props::new(|| Dies)).unwrap();

    child.tell("die", None);

    let terminated = system.when_terminated().wait(WAIT).expect("ended");
    assert_eq!(
        terminated.expect("normally").actor.path().to_string(),
        "pekko://t-guardian-escalate/"
    );
}

/// Q-X-2: a name without a registered configurator is a `ConfigurationException` that carries it.
#[test]
fn an_unregistered_guardian_strategy_name_fails_the_creation_of_the_system() {
    let settings = configured("pekko.actor.guardian-supervisor-strategy=no.SuchStrategy");

    let error = ActorSystem::create_with_settings("t-guardian-unknown", settings)
        .err()
        .expect("the system is not created");

    assert_eq!(
        error.to_string(),
        "org.apache.pekko.ConfigurationException: no.SuchStrategy"
    );
}
