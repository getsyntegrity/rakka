// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! Failures of actor code: the `Throwable` an actor "throws" and how the runtime reports it.
//! Rust has no checked exceptions, so an actor fails by unwinding with a [`Throwable`] payload
//! ([`throw`]) or with any ordinary panic, which is reported as a `java.lang.RuntimeException`.

use std::fmt;

/// A JVM exception as seen by Pekko: class name and optional message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Throwable {
    pub class: String,
    pub message: Option<String>,
}

impl Throwable {
    pub fn new(class: impl Into<String>, message: Option<&str>) -> Self {
        Throwable {
            class: class.into(),
            message: message.map(str::to_string),
        }
    }

    /// `new IllegalStateException(message)`.
    pub fn illegal_state(message: &str) -> Self {
        Throwable::new("java.lang.IllegalStateException", Some(message))
    }
}

impl fmt::Display for Throwable {
    /// `reason.getClass.getName` plus `": message"` when there is one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{}: {m}", self.class),
            None => f.write_str(&self.class),
        }
    }
}

/// `throw t`: unwinds the current actor callback without invoking the panic hook (nothing is printed).
pub fn throw(t: Throwable) -> ! {
    std::panic::resume_unwind(Box::new(t))
}
