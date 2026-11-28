// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala,
// actor-tests/src/test/scala/org/apache/pekko/config/ConfigSpec.scala and
// actor-tests/src/test/scala/org/apache/pekko/actor/CoordinatedShutdownSpec.scala (Apache-2.0).
//! Integration: a real `ActorSystem` built from a `Config` (AC-CFG-003, AC-CFG-020). No component is replaced.

use std::time::Duration;

use rakka_actor::actor::CreateError;
use rakka_actor::actor::system::ActorSystem;
use rakka_actor::config::{
    Config, DEFAULT_LOGGER, DEFAULT_LOGGING_FILTER, SLF4J_LOGGER, SLF4J_LOGGING_FILTER,
};
use rakka_actor::pekko_version;

const WAIT: Duration = Duration::from_secs(10);

/// A system that is terminated when the test ends, also on failure.
struct Started(ActorSystem);

impl Drop for Started {
    fn drop(&mut self) {
        self.0.terminate().wait(WAIT);
    }
}

fn create(name: &str, app: &str) -> Result<Started, CreateError> {
    let config = Config::parse_string(app).expect("HOCON");
    ActorSystem::create_from_config(name, config).map(Started)
}

fn rejection(name: &str, app: &str) -> CreateError {
    match create(name, app) {
        Ok(_) => panic!("a system was created from: {app}"),
        Err(error) => error,
    }
}

#[test]
fn a_config_of_another_pekko_version_does_not_start_the_system() {
    let error = rejection("sys", "pekko.version = \"0.0.1\"");
    assert_eq!(
        error.to_string(),
        "ConfigurationException: Pekko JAR version [2.0.0] does not match the provided config version [0.0.1]"
    );
}

#[test]
fn coordinated_shutdown_cannot_run_on_terminate_without_terminating_the_system() {
    let error = rejection(
        "sys",
        "pekko.coordinated-shutdown.terminate-actor-system = off",
    );
    assert_eq!(error.simple_class_name(), "ConfigurationException");
    assert_eq!(
        error.message(),
        "pekko.coordinated-shutdown.run-by-actor-system-terminate=on and \
         pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination."
    );
}

#[test]
fn the_system_exposes_the_settings_read_from_the_config_over_the_reference() {
    let system = create("sys", "pekko.actor.creation-timeout = 7s").expect("created");
    let settings = system.0.config_settings().expect("settings");
    assert_eq!(settings.name, "sys");
    assert_eq!(settings.creation_timeout.as_nanos(), 7_000_000_000);
    assert_eq!(settings.config_version, pekko_version::CURRENT);
    assert_eq!(
        settings.config().get_string("pekko.version").unwrap(),
        pekko_version::CURRENT
    );
}

#[test]
fn the_runtime_settings_and_the_deployments_come_from_the_config() {
    let system = create(
        "sys",
        "pekko.actor.unstarted-push-timeout = 3s\npekko.actor.deployment./svc.dispatcher = d1",
    )
    .expect("created");
    let runtime = system.0.settings();
    assert_eq!(runtime.unstarted_push_timeout, Duration::from_secs(3));
    let deploy = runtime.deployer.lookup(["svc"]).expect("deployed");
    assert_eq!(deploy.dispatcher(), "d1");
}

// `amendSlf4jConfig` runs before the settings are read; rakka has no `Slf4jLogger`, so `use-slf4j` alone changes nothing.
#[test]
fn the_slf4j_config_is_amended_before_the_settings_are_read() {
    let listed = create("sys", &format!("pekko.loggers = [\"{SLF4J_LOGGER}\"]")).expect("created");
    let settings = listed.0.config_settings().expect("settings");
    assert_eq!(settings.logging_filter, SLF4J_LOGGING_FILTER);

    let asked = create("sys", "pekko.use-slf4j = on").expect("created");
    let settings = asked.0.config_settings().expect("settings");
    assert_eq!(settings.loggers, [DEFAULT_LOGGER]);
    assert_eq!(settings.logging_filter, DEFAULT_LOGGING_FILTER);
}

#[test]
fn an_error_amending_the_slf4j_config_is_a_settings_error() {
    let error = rejection("sys", "pekko.loggers = 5");
    assert_eq!(error.simple_class_name(), "WrongType");
}

#[test]
fn the_name_is_checked_before_the_settings_are_read() {
    let error = rejection("-sys", "pekko.version = \"0.0.1\"");
    assert_eq!(error.simple_class_name(), "IllegalArgumentException");
    assert!(
        error
            .message()
            .starts_with("invalid ActorSystem name [-sys]"),
        "{error}"
    );
}

#[test]
fn a_rejected_deployment_path_does_not_start_the_system() {
    let error = rejection("sys", "pekko.actor.deployment.\"/a b\".dispatcher = d1");
    assert_eq!(error.simple_class_name(), "InvalidActorNameException");
    assert!(matches!(error, CreateError::Deployer(_)), "{error}");
}

// Q-CFG-31 (proposed): Pekko throws `ClassNotFoundException` here and rakka a `ConfigurationException` (Q-X-2), so
// "the class and message are Pekko's" (Q-SYS-7) does not hold for this rejection.
#[test]
fn an_unregistered_guardian_strategy_name_does_not_start_the_system() {
    let error = rejection(
        "sys",
        "pekko.actor.guardian-supervisor-strategy = \"no.SuchStrategy\"",
    );
    assert_eq!(error.to_string(), "ConfigurationException: no.SuchStrategy");
    assert!(matches!(error, CreateError::Actor(_)), "{error}");
}

#[test]
fn a_system_created_without_a_config_has_no_config_settings() {
    let system = Started(ActorSystem::create("sys").expect("created"));
    assert!(system.0.config_settings().is_none());
}
