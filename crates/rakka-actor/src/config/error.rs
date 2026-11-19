// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
//! Error categories and messages of the configuration library (Q-CFG-4 (a): the categories and texts measured with the oracle).

use std::error::Error;
use std::fmt;

/// The `ConfigException` subclass the oracle reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConfigErrorKind {
    /// `ConfigException.Missing`.
    Missing,
    /// `ConfigException.Null`.
    Null,
    /// `ConfigException.WrongType`.
    WrongType,
    /// `ConfigException.BadValue`.
    BadValue,
    /// `ConfigException.Parse`.
    Parse,
    /// `ConfigException.UnresolvedSubstitution`.
    UnresolvedSubstitution,
    /// `ConfigException.NotResolved`.
    NotResolved,
    /// `ConfigException.IO`.
    Io,
    /// `ConfigException.ValidationFailed`.
    ValidationFailed,
    /// `ConfigException.BadPath`.
    BadPath,
}

impl ConfigErrorKind {
    /// Simple name of the Java exception class.
    pub fn java_name(self) -> &'static str {
        match self {
            ConfigErrorKind::Missing => "Missing",
            ConfigErrorKind::Null => "Null",
            ConfigErrorKind::WrongType => "WrongType",
            ConfigErrorKind::BadValue => "BadValue",
            ConfigErrorKind::Parse => "Parse",
            ConfigErrorKind::UnresolvedSubstitution => "UnresolvedSubstitution",
            ConfigErrorKind::NotResolved => "NotResolved",
            ConfigErrorKind::Io => "IO",
            ConfigErrorKind::ValidationFailed => "ValidationFailed",
            ConfigErrorKind::BadPath => "BadPath",
        }
    }
}

/// A configuration error: a category and the message of the configuration library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    kind: ConfigErrorKind,
    message: String,
}

impl ConfigError {
    pub(crate) fn new(kind: ConfigErrorKind, message: impl Into<String>) -> ConfigError {
        ConfigError {
            kind,
            message: message.into(),
        }
    }

    /// The category.
    pub fn kind(&self) -> ConfigErrorKind {
        self.kind
    }

    /// The message, as the configuration library words it.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.java_name(), self.message)
    }
}

impl Error for ConfigError {}
