// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/actor/ActorPath.scala and Address.scala (Apache-2.0).
//! Errors carrying the Java exception class Pekko throws and its exact message.

use std::fmt;

macro_rules! java_error {
    ($(#[$doc:meta])* $name:ident, $class:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            message: String,
        }

        impl $name {
            pub(crate) fn new(message: impl Into<String>) -> Self {
                Self { message: message.into() }
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
    /// `java.lang.ArithmeticException`, from `Math.addExact`/`multiplyExact` overflow.
    ArithmeticError,
    "ArithmeticException"
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
