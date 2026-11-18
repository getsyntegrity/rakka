// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! Errors of the byte-string family. Like the rest of the port (Q-UTIL-17), each error carries the
//! Java class Pekko throws (`java_class()`), while `Display` shows only the message.

use std::fmt;

/// The failure of a `ByteString`, `ByteIterator`, `ByteStringBuilder` or stream operation, named after the
/// exception Pekko 2.0.0 throws in the same situation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByteStringError {
    /// `java.lang.IndexOutOfBoundsException`; the message is absent when Pekko throws it without one.
    IndexOutOfBounds(Option<String>),
    /// `java.lang.ArrayIndexOutOfBoundsException`.
    ArrayIndexOutOfBounds(String),
    /// `java.lang.IllegalArgumentException`.
    IllegalArgument(String),
    /// `java.util.NoSuchElementException`.
    NoSuchElement(String),
    /// `java.io.UnsupportedEncodingException`: the charset name is unknown.
    UnsupportedEncoding(String),
    /// `java.io.EOFException`: a stream ended before the bytes asked for.
    EndOfFile,
}

impl ByteStringError {
    /// The fully qualified name of the Java exception class.
    #[must_use]
    pub fn java_class(&self) -> &'static str {
        match self {
            Self::IndexOutOfBounds(_) => "java.lang.IndexOutOfBoundsException",
            Self::ArrayIndexOutOfBounds(_) => "java.lang.ArrayIndexOutOfBoundsException",
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::NoSuchElement(_) => "java.util.NoSuchElementException",
            Self::UnsupportedEncoding(_) => "java.io.UnsupportedEncodingException",
            Self::EndOfFile => "java.io.EOFException",
        }
    }

    /// The exception message, `None` where Pekko throws without one.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::IndexOutOfBounds(message) => message.as_deref(),
            Self::EndOfFile => None,
            Self::ArrayIndexOutOfBounds(message)
            | Self::IllegalArgument(message)
            | Self::NoSuchElement(message)
            | Self::UnsupportedEncoding(message) => Some(message),
        }
    }

    pub(crate) fn exhausted() -> Self {
        Self::NoSuchElement("next on empty iterator".to_owned())
    }
}

impl fmt::Display for ByteStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message().unwrap_or_default())
    }
}

impl std::error::Error for ByteStringError {}

impl From<ByteStringError> for std::io::Error {
    fn from(error: ByteStringError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
    }
}
