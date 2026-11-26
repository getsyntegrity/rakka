// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala, actor/src/main/scala/org/apache/pekko/actor/Address.scala and actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! Errors carrying the Java exception class Pekko throws and its exact message.

use std::fmt;

use crate::dispatch::mailboxes::MailboxSelectionError;

macro_rules! java_error {
    ($(#[$doc:meta])* $name:ident, $class:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            message: String,
            /// The message as UTF-16 code units when it holds a lone surrogate that `message` cannot (Q-PATH-2).
            exact_utf16: Option<Vec<u16>>,
        }

        impl $name {
            pub(crate) fn new(message: impl Into<String>) -> Self {
                Self { message: message.into(), exact_utf16: None }
            }

            /// A message made of UTF-16 code units, as the JVM builds it (may hold a lone surrogate).
            #[allow(dead_code)]
            pub(crate) fn from_utf16(units: Vec<u16>) -> Self {
                Self { message: String::from_utf16_lossy(&units), exact_utf16: Some(units) }
            }

            /// The exact message as UTF-16 code units, like `Throwable.getMessage` on the JVM.
            pub fn message_utf16(&self) -> Vec<u16> {
                self.exact_utf16.clone().unwrap_or_else(|| self.message.encode_utf16().collect())
            }

            /// Simple name of the Java exception class Pekko throws.
            pub fn java_class(&self) -> &'static str {
                $class
            }

            /// The exact message Pekko puts in the exception.
            pub fn message(&self) -> &str {
                &self.message
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.message)
            }
        }

        impl std::error::Error for $name {}
    };
}

java_error!(
    /// `java.net.MalformedURLException`.
    MalformedUrlError,
    "MalformedURLException"
);
java_error!(
    /// `java.lang.NumberFormatException`, from `Integer.valueOf` on an uid.
    NumberFormatError,
    "NumberFormatException"
);
java_error!(
    /// `java.lang.IllegalArgumentException`.
    IllegalArgumentError,
    "IllegalArgumentException"
);
java_error!(
    /// `java.lang.UnsupportedOperationException`.
    UnsupportedOperationError,
    "UnsupportedOperationException"
);
java_error!(
    /// `java.lang.IllegalStateException`.
    IllegalStateError,
    "IllegalStateException"
);
java_error!(
    /// `org.apache.pekko.actor.InvalidActorNameException`.
    InvalidActorNameError,
    "InvalidActorNameException"
);

macro_rules! java_error_enum {
    ($(#[$doc:meta])* $name:ident { $($variant:ident($inner:ident)),+ }) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $(#[doc = concat!("The `", stringify!($variant), "` case.")] $variant($inner)),+
        }

        impl $name {
            /// The JVM class name the wrapped error stands for, e.g. `IllegalArgumentException`.
            pub fn java_class(&self) -> &'static str {
                match self { $(Self::$variant(e) => e.java_class()),+ }
            }

            /// The message of the wrapped error (`getMessage`).
            pub fn message(&self) -> &str {
                match self { $(Self::$variant(e) => e.message()),+ }
            }

            /// The message as UTF-16 code units, which keeps lone surrogates that a Rust `str` cannot hold.
            pub fn message_utf16(&self) -> Vec<u16> {
                match self { $(Self::$variant(e) => e.message_utf16()),+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.message())
            }
        }

        impl std::error::Error for $name {}

        $(impl From<$inner> for $name {
            fn from(e: $inner) -> Self {
                Self::$variant(e)
            }
        })+
    };
}

java_error_enum!(
    /// Errors of `ActorPath::from_string`.
    FromStringError {
        MalformedUrl(MalformedUrlError),
        NumberFormat(NumberFormatError)
    }
);
java_error_enum!(
    /// Errors of `ActorPath::child` and `ActorPath::descendant`.
    ChildError {
        NumberFormat(NumberFormatError),
        IllegalArgument(IllegalArgumentError)
    }
);
/// `org.apache.pekko.ConfigurationException` as `actorOf` throws it: the message and the `Throwable.toString` of the cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationError {
    message: String,
    cause: Option<String>,
}

impl ConfigurationError {
    pub(crate) fn caused_by(message: String, cause: &MailboxSelectionError) -> Self {
        Self {
            message,
            cause: Some(cause.to_string_like_throwable()),
        }
    }

    /// `getMessage`.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// `getCause.toString`: the class and message of what made the configuration unusable.
    pub fn cause(&self) -> Option<&str> {
        self.cause.as_deref()
    }
}

/// What the Classic runtime throws at its callers (`ActorSystem.create`, `actorOf`). Unlike the other errors it
/// renders like `Throwable.toString`: `<JVM class name>: <message>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActorError {
    /// `InvalidActorNameException`.
    InvalidActorName(InvalidActorNameError),
    /// `java.lang.IllegalStateException`.
    IllegalState(IllegalStateError),
    /// `java.lang.IllegalArgumentException`.
    IllegalArgument(IllegalArgumentError),
    /// `org.apache.pekko.ConfigurationException`.
    Configuration(ConfigurationError),
}

impl ActorError {
    /// The message of the wrapped error (`getMessage`).
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidActorName(e) => e.message(),
            Self::IllegalState(e) => e.message(),
            Self::IllegalArgument(e) => e.message(),
            Self::Configuration(e) => e.message(),
        }
    }

    /// The message as UTF-16 code units.
    pub fn message_utf16(&self) -> Vec<u16> {
        match self {
            Self::InvalidActorName(e) => e.message_utf16(),
            Self::IllegalState(e) => e.message_utf16(),
            Self::IllegalArgument(e) => e.message_utf16(),
            Self::Configuration(e) => e.message().encode_utf16().collect(),
        }
    }

    /// `getCause.toString`, for the errors that wrap one.
    pub fn cause(&self) -> Option<&str> {
        match self {
            Self::Configuration(e) => e.cause(),
            _ => None,
        }
    }
}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.class().java_name(), self.message())
    }
}

impl std::error::Error for ActorError {}

impl From<InvalidActorNameError> for ActorError {
    fn from(e: InvalidActorNameError) -> Self {
        Self::InvalidActorName(e)
    }
}

impl From<IllegalStateError> for ActorError {
    fn from(e: IllegalStateError) -> Self {
        Self::IllegalState(e)
    }
}

impl From<IllegalArgumentError> for ActorError {
    fn from(e: IllegalArgumentError) -> Self {
        Self::IllegalArgument(e)
    }
}

impl From<ConfigurationError> for ActorError {
    fn from(e: ConfigurationError) -> Self {
        Self::Configuration(e)
    }
}

/// The exception classes of [`ActorError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// `org.apache.pekko.actor.InvalidActorNameException`
    InvalidActorName,
    /// `java.lang.IllegalStateException`
    IllegalState,
    /// `java.lang.IllegalArgumentException`
    IllegalArgument,
    /// `org.apache.pekko.ConfigurationException`
    Configuration,
}

impl ErrorClass {
    /// Fully qualified JVM class name.
    pub fn java_name(self) -> &'static str {
        match self {
            ErrorClass::InvalidActorName => "org.apache.pekko.actor.InvalidActorNameException",
            ErrorClass::IllegalState => "java.lang.IllegalStateException",
            ErrorClass::IllegalArgument => "java.lang.IllegalArgumentException",
            ErrorClass::Configuration => "org.apache.pekko.ConfigurationException",
        }
    }
}

impl ActorError {
    /// Which JVM exception class this error stands for.
    pub fn class(&self) -> ErrorClass {
        match self {
            ActorError::InvalidActorName(_) => ErrorClass::InvalidActorName,
            ActorError::IllegalState(_) => ErrorClass::IllegalState,
            ActorError::IllegalArgument(_) => ErrorClass::IllegalArgument,
            ActorError::Configuration(_) => ErrorClass::Configuration,
        }
    }

    /// Builds the error of `class` with `message`.
    pub fn new(class: ErrorClass, message: impl Into<String>) -> ActorError {
        let message = message.into();
        match class {
            ErrorClass::InvalidActorName => InvalidActorNameError::new(message).into(),
            ErrorClass::IllegalState => IllegalStateError::new(message).into(),
            ErrorClass::IllegalArgument => IllegalArgumentError::new(message).into(),
            ErrorClass::Configuration => ConfigurationError {
                message,
                cause: None,
            }
            .into(),
        }
    }
}

#[cfg(test)]
mod configuration_error_tests {
    use super::*;
    use crate::ConfigurationException;

    #[test]
    fn a_configuration_error_without_cause_renders_like_throwable_to_string() {
        let error = ActorError::new(ErrorClass::Configuration, "bad setting");
        assert_eq!(error.class(), ErrorClass::Configuration);
        assert_eq!(error.cause(), None);
        assert_eq!(
            error.message_utf16(),
            "bad setting".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            error.to_string(),
            "org.apache.pekko.ConfigurationException: bad setting"
        );
    }

    #[test]
    fn a_configuration_error_keeps_the_class_and_message_of_its_cause() {
        let cause = MailboxSelectionError::from(ConfigurationException::new(
            "Mailbox Type [x] not configured",
        ));
        let error: ActorError = ConfigurationError::caused_by("outer".to_owned(), &cause).into();
        assert_eq!(error.message(), "outer");
        assert_eq!(
            error.cause(),
            Some("org.apache.pekko.ConfigurationException: Mailbox Type [x] not configured")
        );
    }

    #[test]
    fn other_errors_have_no_cause() {
        assert_eq!(ActorError::new(ErrorClass::IllegalState, "s").cause(), None);
    }
}
