// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala and actor/src/main/scala/org/apache/pekko/util/Helpers.scala (Apache-2.0).
//! `ActorSystem.Settings` (AC-CFG-002 and the key groups AC-CFG-004/005/006/007/009/010/011/020) and
//! `Settings.amendSlf4jConfig` (AC-CFG-018).
//!
//! `Settings::new` reads every key the core uses, once, in the order of the Scala constructor, and fails with the first
//! error: either the configuration library's (`ConfigError`) or a `ConfigurationException`. Values keep Pekko's domain:
//! `log_dead_letters` is the `Int` Pekko computes (`off` = 0, `on` = `i32::MAX`), a timeout may be negative or zero.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use super::{Config, ConfigError};
use crate::ConfigurationException;
use crate::util::Timeout;
use crate::util::helpers::to_root_lower_case;

/// Name of the Slf4j logger class Pekko looks for (`ActorSystem.scala:344`).
pub const SLF4J_LOGGER: &str = "org.apache.pekko.event.slf4j.Slf4jLogger";
/// Name of the Slf4j logging filter class.
pub const SLF4J_LOGGING_FILTER: &str = "org.apache.pekko.event.slf4j.Slf4jLoggingFilter";
/// `classOf[DefaultLogger].getName`.
pub const DEFAULT_LOGGER: &str = "org.apache.pekko.event.Logging$DefaultLogger";
/// `classOf[DefaultLoggingFilter].getName`.
pub const DEFAULT_LOGGING_FILTER: &str = "org.apache.pekko.event.DefaultLoggingFilter";

/// Why a `Settings` could not be built.
#[derive(Clone, Debug)]
pub enum SettingsError {
    /// A key is missing, has the wrong type or is not a valid value (`ConfigException`).
    Config(ConfigError),
    /// `ConfigurationException`: an unsupported combination, a version that does not match, or a feature rakka
    /// does not have yet (`serialize-messages` and `serialize-creators` on, Q-CFG-11).
    Configuration(ConfigurationException),
}

impl SettingsError {
    /// Simple name of the Java exception class.
    pub fn java_name(&self) -> &'static str {
        match self {
            SettingsError::Config(e) => e.kind().java_name(),
            SettingsError::Configuration(e) => e.java_class(),
        }
    }

    /// `getMessage`.
    pub fn message(&self) -> &str {
        match self {
            SettingsError::Config(e) => e.message(),
            SettingsError::Configuration(e) => e.message(),
        }
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.java_name(), self.message())
    }
}

impl Error for SettingsError {}

impl From<ConfigError> for SettingsError {
    fn from(e: ConfigError) -> SettingsError {
        SettingsError::Config(e)
    }
}

impl From<ConfigurationException> for SettingsError {
    fn from(e: ConfigurationException) -> SettingsError {
        SettingsError::Configuration(e)
    }
}

/// `LogDeadLettersSuspendDuration`: `infinite` or a finite duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspendDuration {
    /// `Duration.Inf`.
    Infinite,
    /// A duration read in whole milliseconds (it may be zero or negative, like Pekko's).
    Finite(Timeout),
}

/// The settings of an actor system, read from a `Config` (`ActorSystem.Settings`).
#[derive(Clone, Debug)]
pub struct Settings {
    /// The name of the system.
    pub name: String,
    config: Config,
    /// `ConfigVersion` (`pekko.version`).
    pub config_version: String,
    /// `SupervisorStrategyClass` (`pekko.actor.guardian-supervisor-strategy`): a name for the registry (Q-X-2).
    pub supervisor_strategy_class: String,
    /// `CreationTimeout`.
    pub creation_timeout: Timeout,
    /// `UnstartedPushTimeout`.
    pub unstarted_push_timeout: Timeout,
    /// `AllowJavaSerialization`.
    pub allow_java_serialization: bool,
    /// `SerializeAllMessages`; `on` is rejected by `Settings::new` until AC-SER exists (Q-CFG-11).
    pub serialize_all_messages: bool,
    /// `SerializeAllCreators`; `on` is rejected by `Settings::new` until AC-SER exists (Q-CFG-11).
    pub serialize_all_creators: bool,
    /// `NoSerializationVerificationNeededClassPrefix`.
    pub no_serialization_verification_needed_class_prefix: BTreeSet<String>,
    /// `LogLevel`.
    pub log_level: String,
    /// `StdoutLogLevel`.
    pub stdout_log_level: String,
    /// `Loggers`: names for the registry (Q-X-2).
    pub loggers: Vec<String>,
    /// `LoggersDispatcher`.
    pub loggers_dispatcher: String,
    /// `LoggingFilter`: a name for the registry (Q-X-2).
    pub logging_filter: String,
    /// `LoggerStartTimeout`.
    pub logger_start_timeout: Timeout,
    /// `LogConfigOnStart`.
    pub log_config_on_start: bool,
    /// `LogDeadLetters`: the `Int` of Pekko (`off` = 0, `on` = `i32::MAX`).
    pub log_dead_letters: i32,
    /// `LogDeadLettersDuringShutdown`.
    pub log_dead_letters_during_shutdown: bool,
    /// `LogDeadLettersSuspendDuration`.
    pub log_dead_letters_suspend_duration: SuspendDuration,
    /// `AddLoggingReceive` (`pekko.actor.debug.receive`).
    pub add_logging_receive: bool,
    /// `DebugAutoReceive`.
    pub debug_auto_receive: bool,
    /// `DebugLifecycle`.
    pub debug_lifecycle: bool,
    /// `FsmDebugEvent`.
    pub fsm_debug_event: bool,
    /// `DebugEventStream`.
    pub debug_event_stream: bool,
    /// `DebugUnhandledMessage`.
    pub debug_unhandled_message: bool,
    /// `DebugRouterMisconfiguration`.
    pub debug_router_misconfiguration: bool,
    /// `Home`: `None` when `pekko.home` is the empty string.
    pub home: Option<String>,
    /// `SchedulerClass`: a name for the registry (Q-X-2).
    pub scheduler_class: String,
    /// `Daemonicity` (`pekko.daemonic`); accepted and kept, with no effect (Q-CFG-7 (a)).
    pub daemonicity: bool,
    /// `JvmExitOnFatalError`; accepted and kept, with no effect yet (Q-CFG-7 (a)).
    pub jvm_exit_on_fatal_error: bool,
    /// `JvmShutdownHooks`; accepted and kept, with no effect yet (Q-CFG-7 (a)).
    pub jvm_shutdown_hooks: bool,
    /// `FailMixedVersions`; accepted and kept, with no effect (Q-CFG-7 (a)).
    pub fail_mixed_versions: bool,
    /// `CoordinatedShutdownTerminateActorSystem`.
    pub coordinated_shutdown_terminate_actor_system: bool,
    /// `CoordinatedShutdownRunByActorSystemTerminate`.
    pub coordinated_shutdown_run_by_actor_system_terminate: bool,
    /// `DefaultVirtualNodesFactor`.
    pub default_virtual_nodes_factor: i32,
}

impl Settings {
    /// `new ActorSystem.Settings(classLoader, config, name)`.
    ///
    /// The configuration is validated against `Config::default_reference()` with `pekko.actor.internal-dispatcher` left
    /// out (AC-CFG-019), every key is read in the order of the Scala constructor, and the first problem is returned.
    /// With `serialize-messages` or `serialize-creators` on the result is a `ConfigurationException` (Q-CFG-11).
    pub fn new(name: &str, config: Config) -> Result<Settings, SettingsError> {
        let reference =
            Config::default_reference().without_path("pekko.actor.internal-dispatcher")?;
        config.check_valid(&reference, &["pekko"])?;

        let config_version = config.get_string("pekko.version")?;
        let supervisor_strategy_class =
            config.get_string("pekko.actor.guardian-supervisor-strategy")?;
        let creation_timeout = config.get_millis_duration("pekko.actor.creation-timeout")?;
        let unstarted_push_timeout =
            config.get_millis_duration("pekko.actor.unstarted-push-timeout")?;

        let allow_java_serialization =
            config.get_boolean("pekko.actor.allow-java-serialization")?;
        let serialize_all_messages = config.get_boolean("pekko.actor.serialize-messages")?;
        let serialize_all_creators = config.get_boolean("pekko.actor.serialize-creators")?;
        let no_serialization_verification_needed_class_prefix = config
            .get_string_list("pekko.actor.no-serialization-verification-needed-class-prefix")?
            .into_iter()
            .collect();

        let log_level = config.get_string("pekko.loglevel")?;
        let stdout_log_level = config.get_string("pekko.stdout-loglevel")?;
        let loggers = config.get_string_list("pekko.loggers")?;
        let loggers_dispatcher = config.get_string("pekko.loggers-dispatcher")?;
        let logging_filter = config.get_string("pekko.logging-filter")?;
        let logger_start_timeout = config.get_millis_duration("pekko.logger-startup-timeout")?;
        let log_config_on_start = config.get_boolean("pekko.log-config-on-start")?;
        let log_dead_letters =
            match to_root_lower_case(&config.get_string("pekko.log-dead-letters")?).as_str() {
                "off" | "false" => 0,
                "on" | "true" => i32::MAX,
                _ => config.get_int("pekko.log-dead-letters")?,
            };
        let log_dead_letters_during_shutdown =
            config.get_boolean("pekko.log-dead-letters-during-shutdown")?;
        let log_dead_letters_suspend_duration = {
            let key = "pekko.log-dead-letters-suspend-duration";
            match to_root_lower_case(&config.get_string(key)?).as_str() {
                "infinite" => SuspendDuration::Infinite,
                _ => SuspendDuration::Finite(config.get_millis_duration(key)?),
            }
        };

        let add_logging_receive = config.get_boolean("pekko.actor.debug.receive")?;
        let debug_auto_receive = config.get_boolean("pekko.actor.debug.autoreceive")?;
        let debug_lifecycle = config.get_boolean("pekko.actor.debug.lifecycle")?;
        let fsm_debug_event = config.get_boolean("pekko.actor.debug.fsm")?;
        let debug_event_stream = config.get_boolean("pekko.actor.debug.event-stream")?;
        let debug_unhandled_message = config.get_boolean("pekko.actor.debug.unhandled")?;
        let debug_router_misconfiguration =
            config.get_boolean("pekko.actor.debug.router-misconfiguration")?;

        let home = match config.get_string("pekko.home")?.as_str() {
            "" => None,
            other => Some(other.to_owned()),
        };

        let scheduler_class = config.get_string("pekko.scheduler.implementation")?;
        let daemonicity = config.get_boolean("pekko.daemonic")?;
        let jvm_exit_on_fatal_error = config.get_boolean("pekko.jvm-exit-on-fatal-error")?;
        let jvm_shutdown_hooks = config.get_boolean("pekko.jvm-shutdown-hooks")?;
        let fail_mixed_versions = config.get_boolean("pekko.fail-mixed-versions")?;

        let terminate = config.get_boolean("pekko.coordinated-shutdown.terminate-actor-system")?;
        let run_by_terminate =
            config.get_boolean("pekko.coordinated-shutdown.run-by-actor-system-terminate")?;
        if run_by_terminate && !terminate {
            return Err(ConfigurationException::new(
                "pekko.coordinated-shutdown.run-by-actor-system-terminate=on and \
                 pekko.coordinated-shutdown.terminate-actor-system=off is not a supported configuration combination.",
            )
            .into());
        }

        let default_virtual_nodes_factor =
            config.get_int("pekko.actor.deployment.default.virtual-nodes-factor")?;

        super::check_pekko_version(&config)?;

        // Q-CFG-11: the checks these two keys switch on need AC-SER; "on" is an error, not a silent no-op.
        for (on, key) in [
            (serialize_all_messages, "pekko.actor.serialize-messages"),
            (serialize_all_creators, "pekko.actor.serialize-creators"),
        ] {
            if on {
                return Err(ConfigurationException::new(format!(
                    "{key}=on is not supported yet: the serialization checks need the serialization module (AC-SER)."
                ))
                .into());
            }
        }

        Ok(Settings {
            name: name.to_owned(),
            config,
            config_version,
            supervisor_strategy_class,
            creation_timeout,
            unstarted_push_timeout,
            allow_java_serialization,
            serialize_all_messages,
            serialize_all_creators,
            no_serialization_verification_needed_class_prefix,
            log_level,
            stdout_log_level,
            loggers,
            loggers_dispatcher,
            logging_filter,
            logger_start_timeout,
            log_config_on_start,
            log_dead_letters,
            log_dead_letters_during_shutdown,
            log_dead_letters_suspend_duration,
            add_logging_receive,
            debug_auto_receive,
            debug_lifecycle,
            fsm_debug_event,
            debug_event_stream,
            debug_unhandled_message,
            debug_router_misconfiguration,
            home,
            scheduler_class,
            daemonicity,
            jvm_exit_on_fatal_error,
            jvm_shutdown_hooks,
            fail_mixed_versions,
            coordinated_shutdown_terminate_actor_system: terminate,
            coordinated_shutdown_run_by_actor_system_terminate: run_by_terminate,
            default_virtual_nodes_factor,
        })
    }

    /// `settings.config`: the configuration the settings were read from.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// `settings.toString`: the configuration with the environment values and the user name redacted.
    pub fn render(&self) -> String {
        self.config.render_with_redactions()
    }

    /// The extensions to load, library extensions first and then the user's, in list order (AC-CFG-006).
    /// Q-CFG-8 (b): the trailing `$` of a Scala object name is dropped and a name that repeats is kept once.
    pub fn extension_names(&self) -> Result<Vec<String>, ConfigError> {
        let mut names: Vec<String> = Vec::new();
        for key in ["pekko.library-extensions", "pekko.extensions"] {
            for raw in self.config.get_string_list(key)? {
                let name = normalize_extension_name(&raw).to_owned();
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        Ok(names)
    }
}

/// Q-CFG-8 (b): `org.apache.pekko.serialization.SerializationExtension$` and `...SerializationExtension` are the same name.
pub fn normalize_extension_name(name: &str) -> &str {
    name.strip_suffix('$').unwrap_or(name)
}

/// `Settings.amendSlf4jConfig(config, dynamicAccess)` (AC-CFG-018, Q-X-7 (A), Q-CFG-10): `is_registered` stands for
/// `dynamicAccess.classIsOnClasspath`, that is, whether a factory with that name is registered.
///
/// - `pekko.loggers` already lists the Slf4j logger: unchanged, except that a default filter becomes the Slf4j one.
/// - `pekko.use-slf4j` is on and the Slf4j logger is registered: the Slf4j logger goes first, the default logger is
///   dropped, and a default filter becomes the Slf4j one.
/// - anything else: unchanged.
pub fn amend_slf4j_config(
    config: &Config,
    is_registered: impl Fn(&str) -> bool,
) -> Result<Config, ConfigError> {
    let loggers_key = "pekko.loggers";
    let filter_key = "pekko.logging-filter";
    let configured_loggers = config.get_string_list(loggers_key)?;
    let configured_filter = config.get_string(filter_key)?;
    let filter_already_configured =
        configured_filter == SLF4J_LOGGING_FILTER || configured_filter != DEFAULT_LOGGING_FILTER;
    let new_filter = format!("{filter_key} = \"{SLF4J_LOGGING_FILTER}\"");

    if configured_loggers.iter().any(|l| l == SLF4J_LOGGER) {
        if filter_already_configured {
            Ok(config.clone())
        } else {
            Ok(Config::parse_string(&new_filter)?.with_fallback(config))
        }
    } else {
        let use_key = "pekko.use-slf4j";
        if config.has_path(use_key)? && config.get_boolean(use_key)? && is_registered(SLF4J_LOGGER)
        {
            let mut loggers = vec![SLF4J_LOGGER.to_owned()];
            loggers.extend(
                configured_loggers
                    .into_iter()
                    .filter(|l| l != DEFAULT_LOGGER),
            );
            let new_loggers = format!("{loggers_key} = [\"{}\"]", loggers.join("\", \""));
            let text = if filter_already_configured {
                new_loggers
            } else {
                format!("{new_loggers}\n{new_filter}")
            };
            Ok(Config::parse_string(&text)?.with_fallback(config))
        } else {
            Ok(config.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigErrorKind;
    use crate::util::TimeUnit;

    fn typed_base() -> Config {
        Config::default_reference_with_typed()
    }

    fn settings_with(app: &str) -> Result<Settings, SettingsError> {
        Settings::new(
            "test",
            Config::parse_string(app)
                .unwrap()
                .with_fallback(&typed_base())
                .resolve()
                .unwrap(),
        )
    }

    #[test]
    fn the_reference_configuration_gives_the_defaults_of_config_spec() {
        let s = Settings::new("sys", Config::default_reference()).unwrap();
        assert_eq!(s.name, "sys");
        assert_eq!(s.config_version, "2.0.0");
        assert_eq!(s.log_dead_letters, 10);
        assert_eq!(
            s.log_dead_letters_suspend_duration,
            SuspendDuration::Finite(Timeout::of(5, TimeUnit::Minutes).unwrap())
        );
        assert_eq!(s.loggers, [DEFAULT_LOGGER]);
        assert_eq!(s.logging_filter, DEFAULT_LOGGING_FILTER);
        assert!(s.home.is_none());
        assert!(
            !s.daemonicity
                && s.jvm_exit_on_fatal_error
                && s.jvm_shutdown_hooks
                && s.fail_mixed_versions
        );
        assert_eq!(s.config().get_string("pekko.version").unwrap(), "2.0.0");
        assert_eq!(s.render(), s.config().render_with_redactions());
    }

    #[test]
    fn serialize_messages_or_creators_on_is_a_configuration_exception_until_ac_ser_exists() {
        for (app, key) in [
            (
                "pekko.actor.serialize-messages = on",
                "pekko.actor.serialize-messages",
            ),
            (
                "pekko.actor.serialize-creators = on",
                "pekko.actor.serialize-creators",
            ),
            (
                "pekko.actor.serialize-messages = on\npekko.actor.serialize-creators = on",
                "pekko.actor.serialize-messages",
            ),
        ] {
            let e = settings_with(app).unwrap_err();
            assert_eq!(e.java_name(), "ConfigurationException", "{app}");
            assert!(
                e.message()
                    .starts_with(&format!("{key}=on is not supported yet")),
                "{e}"
            );
        }
    }

    #[test]
    fn a_problem_pekko_reports_comes_before_the_serialization_error() {
        let e = settings_with("pekko.actor.serialize-messages = on\npekko.version = \"1.0.0\"")
            .unwrap_err();
        assert_eq!(
            e.to_string(),
            "ConfigurationException: Pekko JAR version [2.0.0] does not match the provided config version [1.0.0]"
        );
    }

    #[test]
    fn a_key_missing_from_the_configuration_fails_validation_with_the_key_in_the_message() {
        let without = typed_base()
            .without_path("pekko.actor.creation-timeout")
            .unwrap();
        let e = Settings::new("test", without).unwrap_err();
        assert_eq!(e.java_name(), "ValidationFailed");
        assert!(
            e.message()
                .contains("No setting at 'pekko.actor.creation-timeout'"),
            "{e}"
        );
    }

    #[test]
    fn a_wrong_type_is_a_config_error_and_converts_into_a_settings_error() {
        let e: SettingsError = ConfigError::new(ConfigErrorKind::Missing, "m").into();
        assert_eq!(e.to_string(), "Missing: m");
        let c: SettingsError = ConfigurationException::new("c").into();
        assert_eq!(
            (c.java_name(), c.message()),
            ("ConfigurationException", "c")
        );
    }

    #[test]
    fn extension_names_are_library_first_without_the_object_suffix_and_without_repeats() {
        let s = settings_with(
            "pekko.library-extensions += \"my.Ext\"\npekko.extensions = [\"a.B\", \"c.D$\", \"c.D\", \"my.Ext$\"]",
        )
        .unwrap();
        assert_eq!(
            s.extension_names().unwrap(),
            [
                "org.apache.pekko.serialization.SerializationExtension",
                "org.apache.pekko.actor.typed.internal.adapter.ActorSystemAdapter$LoadTypedExtensions",
                "my.Ext",
                "a.B",
                "c.D",
            ]
        );
        assert_eq!(normalize_extension_name("x.Y$"), "x.Y");
        assert_eq!(normalize_extension_name("x.Y"), "x.Y");
        assert_eq!(normalize_extension_name("x$Y$"), "x$Y");
    }

    #[test]
    fn extension_names_of_a_configuration_without_the_lists_is_a_config_error() {
        let config = Config::parse_string("pekko.extensions = 5").unwrap();
        let s = Settings::new("test", Config::default_reference()).unwrap();
        let broken = Settings { config, ..s };
        assert_eq!(
            broken.extension_names().unwrap_err().kind(),
            ConfigErrorKind::Missing
        );
    }

    #[test]
    fn a_duration_beyond_the_long_nanoseconds_saturates_before_it_is_converted() {
        let c =
            Config::parse_string("a = \"106752 d\"\nb = \"-106752 d\"\nc = \"1e20 d\"").unwrap();
        assert_eq!(
            c.get_duration("a", TimeUnit::Milliseconds).unwrap(),
            9_223_372_036_854
        );
        assert_eq!(
            c.get_duration("b", TimeUnit::Milliseconds).unwrap(),
            -9_223_372_036_854
        );
        assert_eq!(
            c.get_duration("c", TimeUnit::Seconds).unwrap(),
            9_223_372_036
        );
    }

    #[test]
    fn amend_asks_the_registry_about_the_slf4j_logger_only_when_use_slf4j_is_on() {
        let reference = Config::default_reference();
        let asked = std::cell::RefCell::new(Vec::new());
        let ask = |name: &str| {
            asked.borrow_mut().push(name.to_owned());
            true
        };
        amend_slf4j_config(&reference, ask).unwrap();
        assert!(asked.borrow().is_empty());
        let on = Config::parse_string("pekko.use-slf4j = on")
            .unwrap()
            .with_fallback(&reference);
        let amended = amend_slf4j_config(&on, ask).unwrap();
        assert_eq!(*asked.borrow(), [SLF4J_LOGGER]);
        assert_eq!(
            amended.get_string_list("pekko.loggers").unwrap(),
            [SLF4J_LOGGER]
        );
        assert_eq!(
            amended.get_string("pekko.logging-filter").unwrap(),
            SLF4J_LOGGING_FILTER
        );
        // The input is not modified.
        assert_eq!(
            on.get_string_list("pekko.loggers").unwrap(),
            [DEFAULT_LOGGER]
        );
    }
}
