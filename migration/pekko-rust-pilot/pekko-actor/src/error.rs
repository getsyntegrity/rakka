//! The exceptions the Classic runtime can throw at its callers, with their JVM class names so that
//! behaviour can be compared with Pekko's.

use crate::{jstr, JStr};

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
    pub fn java_name(self) -> &'static str {
        match self {
            ErrorClass::InvalidActorName => "org.apache.pekko.actor.InvalidActorNameException",
            ErrorClass::IllegalState => "java.lang.IllegalStateException",
            ErrorClass::IllegalArgument => "java.lang.IllegalArgumentException",
        }
    }
}

/// An exception: class plus message (UTF-16, like a Java `String`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorError {
    pub class: ErrorClass,
    pub message: JStr,
}

impl ActorError {
    pub fn new(class: ErrorClass, message: impl AsRef<str>) -> Self {
        ActorError { class, message: jstr(message.as_ref()) }
    }
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class.java_name(), String::from_utf16_lossy(&self.message))
    }
}

impl std::error::Error for ActorError {}
