// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! `ActorSystem(name, config)`: an actor system built from a `Config` (AC-CFG-003, AC-CFG-020). The shape of this API is
//! the proposed decision Q-SYS-7.

use std::error::Error;
use std::fmt;

use crate::actor::system::{self, ActorSystem, check_name};
use crate::actor::{ActorError, DeployerError};
use crate::config::{Config, Settings, SettingsError, amend_slf4j_config};

/// Why [`ActorSystem::create_from_config`] did not create the system.
#[derive(Clone, Debug)]
pub enum CreateError {
    /// The name of the system, or what starting the system throws.
    Actor(ActorError),
    /// `new ActorSystem.Settings(..)`.
    Settings(SettingsError),
    /// A deployment of `pekko.actor.deployment`.
    Deployer(DeployerError),
}

impl CreateError {
    /// Simple name of the Java exception class.
    pub fn simple_class_name(&self) -> &'static str {
        match self {
            Self::Actor(e) => e.class().java_name().rsplit('.').next().unwrap_or_default(),
            Self::Settings(e) => e.java_name(),
            Self::Deployer(e) => e.simple_class_name(),
        }
    }

    /// `getMessage`.
    pub fn message(&self) -> &str {
        match self {
            Self::Actor(e) => e.message(),
            Self::Settings(e) => e.message(),
            Self::Deployer(e) => e.message(),
        }
    }
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.simple_class_name(), self.message())
    }
}

impl Error for CreateError {}

impl ActorSystem {
    /// `ActorSystem(name, config)`: `config` over the reference configuration, amended by `amendSlf4jConfig`, read by
    /// `ActorSystem.Settings` and then by the runtime settings and the deployer. The first error stops the creation.
    ///
    /// rakka has no `Slf4jLogger`, so it is not on the classpath of `amendSlf4jConfig`. The dispatcher's keys are not
    /// read yet ([`Settings::from_config`](crate::actor::system::Settings::from_config)).
    pub fn create_from_config(name: &str, config: Config) -> Result<ActorSystem, CreateError> {
        // The name is checked before the configuration is read.
        check_name(name).map_err(CreateError::Actor)?;
        let config = config.with_fallback(&Config::default_reference());
        let config =
            amend_slf4j_config(&config, |_| false).map_err(|e| CreateError::Settings(e.into()))?;
        let settings = Settings::new(name, config).map_err(CreateError::Settings)?;
        let runtime = system::Settings::from_config(&settings).map_err(CreateError::Deployer)?;
        let system = Self::create_with_settings(name, runtime).map_err(CreateError::Actor)?;
        // No other handle of the system exists yet, so this is the only write.
        system.0.config_settings.get_or_init(|| Box::new(settings));
        Ok(system)
    }

    /// `system.settings` of a system created with [`ActorSystem::create_from_config`]; `None` for one created without a
    /// configuration.
    pub fn config_settings(&self) -> Option<&Settings> {
        self.0.config_settings.get().map(|settings| &**settings)
    }
}
