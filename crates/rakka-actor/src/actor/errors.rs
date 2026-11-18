// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala, actor/src/main/scala/org/apache/pekko/actor/Address.scala and actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! Errors carrying the Java exception class Pekko throws and its exact message.

use std::fmt;

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
            $($variant($inner)),+
        }

        impl $name {
            pub fn java_class(&self) -> &'static str {
                match self { $(Self::$variant(e) => e.java_class()),+ }
            }

            pub fn message(&self) -> &str {
                match self { $(Self::$variant(e) => e.message()),+ }
            }

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
/// What the Classic runtime throws at its callers (`ActorSystem.create`, `actorOf`). Unlike the other errors it
/// renders like `Throwable.toString`: `<JVM class name>: <message>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActorError {
    InvalidActorName(InvalidActorNameError),
    IllegalState(IllegalStateError),
    IllegalArgument(IllegalArgumentError),
}

impl ActorError {
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidActorName(e) => e.message(),
            Self::IllegalState(e) => e.message(),
            Self::IllegalArgument(e) => e.message(),
        }
    }

    pub fn message_utf16(&self) -> Vec<u16> {
        match self {
            Self::InvalidActorName(e) => e.message_utf16(),
            Self::IllegalState(e) => e.message_utf16(),
            Self::IllegalArgument(e) => e.message_utf16(),
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

/// The exception classes of [`ActorError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// `org.apache.pekko.actor.InvalidActorNameException`
    InvalidActorName,
    /// `java.lang.IllegalStateException`
    IllegalState,
    /// `java.lang.IllegalArgumentException`
    IllegalArgument,
}

impl ErrorClass {
    /// Fully qualified JVM class name.
    pub fn java_name(self) -> &'static str {
        match self {
            ErrorClass::InvalidActorName => "org.apache.pekko.actor.InvalidActorNameException",
            ErrorClass::IllegalState => "java.lang.IllegalStateException",
            ErrorClass::IllegalArgument => "java.lang.IllegalArgumentException",
        }
    }
}

impl ActorError {
    pub fn class(&self) -> ErrorClass {
        match self {
            ActorError::InvalidActorName(_) => ErrorClass::InvalidActorName,
            ActorError::IllegalState(_) => ErrorClass::IllegalState,
            ActorError::IllegalArgument(_) => ErrorClass::IllegalArgument,
        }
    }

    pub fn new(class: ErrorClass, message: impl Into<String>) -> ActorError {
        let message = message.into();
        match class {
            ErrorClass::InvalidActorName => InvalidActorNameError::new(message).into(),
            ErrorClass::IllegalState => IllegalStateError::new(message).into(),
            ErrorClass::IllegalArgument => IllegalArgumentError::new(message).into(),
        }
    }
}
