// Derived from Apache Pekko 1.7.1 actor/src/main/scala/org/apache/pekko/PekkoException.scala (Apache-2.0).
//! `PekkoException` and `ConfigurationException`.
//!
//! Two separate types: Pekko's `ConfigurationException extends PekkoException`
//! has no counterpart in the type system (Q-UTIL-4), and `OnlyCauseStackTrace`
//! is not ported because Rust errors carry no stack trace.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

macro_rules! pekko_exception {
    ($(#[$doc:meta])* $name:ident, $class:literal) => {
        $(#[$doc])*
        ///
        /// The cause is shared, so a clone sees the same cause the way every
        /// holder of one Java exception instance does.
        #[derive(Clone, Debug)]
        pub struct $name {
            message: String,
            cause: Option<Arc<dyn Error + Send + Sync>>,
        }

        impl $name {
            /// The Java constructor taking only a message.
            pub fn new(message: impl Into<String>) -> Self {
                Self { message: message.into(), cause: None }
            }

            /// The Java constructor taking a message and a cause.
            pub fn with_cause(
                message: impl Into<String>,
                cause: impl Error + Send + Sync + 'static,
            ) -> Self {
                Self { message: message.into(), cause: Some(Arc::new(cause)) }
            }

            /// `getMessage`.
            pub fn message(&self) -> &str {
                &self.message
            }

            /// `getCause`.
            pub fn cause(&self) -> Option<&(dyn Error + Send + Sync + 'static)> {
                self.cause.as_deref()
            }

            /// Simple name of the Java class.
            pub fn java_class(&self) -> &'static str {
                $class
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.message)
            }
        }

        impl Error for $name {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                self.cause().map(|c| c as &(dyn Error + 'static))
            }
        }
    };
}

pekko_exception!(
    /// `PekkoException`: the base error of the library.
    PekkoException,
    "PekkoException"
);
pekko_exception!(
    /// `ConfigurationException`: Pekko detected a problem with the provided configuration.
    ConfigurationException,
    "ConfigurationException"
);
