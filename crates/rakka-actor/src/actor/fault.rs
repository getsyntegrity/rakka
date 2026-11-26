// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! Failures of actor code: the `Throwable` an actor "throws" and how the runtime reports it.
//! Rust has no checked exceptions, so an actor fails by unwinding with a [`Throwable`] payload
//! ([`throw`]) or with any ordinary panic, which is reported as a `java.lang.RuntimeException`.

use std::fmt;

/// A JVM exception as seen by Pekko: class name, optional message and optional cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Throwable {
    /// The JVM class name, e.g. `java.lang.IllegalStateException`.
    pub class: String,
    /// `getMessage`; `None` when the exception has no message.
    pub message: Option<String>,
    /// `getCause`: what this exception wraps. Equality compares it too; [`Display`](fmt::Display), like `toString`, leaves it out.
    pub cause: Option<Box<Throwable>>,
}

impl Throwable {
    /// A throwable of `class` with an optional `message`.
    pub fn new(class: impl Into<String>, message: Option<&str>) -> Self {
        Throwable {
            class: class.into(),
            message: message.map(str::to_string),
            cause: None,
        }
    }

    /// The same exception caused by `cause` (the constructor argument of the same name on the JVM).
    pub fn with_cause(mut self, cause: Throwable) -> Self {
        self.cause = Some(Box::new(cause));
        self
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

#[cfg(test)]
mod tests {
    use super::Throwable;

    /// `Throwable.toString` leaves the cause out; equality of the Rust value does not.
    #[test]
    fn a_cause_changes_equality_but_not_the_text() {
        let plain = Throwable::illegal_state("outer");
        let caused = plain.clone().with_cause(Throwable::illegal_state("inner"));
        assert_eq!(caused.to_string(), "java.lang.IllegalStateException: outer");
        assert_ne!(caused, plain);
        assert_eq!(
            caused.cause.as_deref(),
            Some(&Throwable::illegal_state("inner"))
        );
    }
}
