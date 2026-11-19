// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/ByteString.scala (Apache-2.0).
//! The charsets `ByteString` can encode to and decode from (Q-UTIL-22: UTF-8, US-ASCII and ISO-8859-1,
//! with the names the oracle accepts). Decoding replaces malformed input with U+FFFD the way
//! `new String(bytes, charset)` does, as the recorded cases show.

use super::ByteStringError;

/// A charset known to the crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Charset {
    /// `UTF-8`.
    Utf8,
    /// `US-ASCII`.
    UsAscii,
    /// `ISO-8859-1`.
    Iso8859_1,
}

impl Charset {
    /// The charset called `name`, or `None` (`Charset.forName` ignores case and knows aliases).
    #[must_use]
    pub fn for_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "UTF-8" | "UTF8" => Some(Self::Utf8),
            "US-ASCII" | "ASCII" | "ISO646-US" => Some(Self::UsAscii),
            "ISO-8859-1" | "ISO8859_1" | "ISO8859-1" | "ISO_8859_1" | "LATIN1" | "L1" => {
                Some(Self::Iso8859_1)
            }
            _ => None,
        }
    }

    pub(crate) fn named(name: &str) -> Result<Self, ByteStringError> {
        Self::for_name(name).ok_or_else(|| ByteStringError::UnsupportedEncoding(name.to_owned()))
    }

    /// The bytes of `text`; a character the charset cannot represent becomes `?`.
    #[must_use]
    pub fn encode(self, text: &str) -> Vec<u8> {
        match self {
            Self::Utf8 => text.as_bytes().to_vec(),
            Self::UsAscii => text
                .chars()
                .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
                .collect(),
            Self::Iso8859_1 => text
                .chars()
                .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
                .collect(),
        }
    }

    /// The text of `bytes`; malformed input becomes U+FFFD.
    #[must_use]
    pub fn decode(self, bytes: &[u8]) -> String {
        match self {
            Self::Utf8 => decode_utf8(bytes),
            Self::UsAscii => bytes
                .iter()
                .map(|&b| {
                    if b.is_ascii() {
                        char::from(b)
                    } else {
                        '\u{FFFD}'
                    }
                })
                .collect(),
            Self::Iso8859_1 => bytes.iter().map(|&b| char::from(b)).collect(),
        }
    }
}

/// UTF-8 with one U+FFFD per maximal malformed prefix, except that a lead byte `ED` followed by
/// `A0..BF` (an encoded surrogate) is replaced as a unit together with its continuation byte.
pub(crate) fn decode_utf8(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        match std::str::from_utf8(&bytes[position..]) {
            Ok(rest) => {
                out.push_str(rest);
                break;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                out.push_str(
                    std::str::from_utf8(&bytes[position..position + valid]).unwrap_or_default(),
                );
                position += valid;
                let remaining = bytes.len() - position;
                let consumed = if bytes[position] == 0xED
                    && remaining > 1
                    && (0xA0..=0xBF).contains(&bytes[position + 1])
                {
                    if remaining > 2 && (0x80..=0xBF).contains(&bytes[position + 2]) {
                        3
                    } else {
                        2
                    }
                } else {
                    error.error_len().unwrap_or(remaining)
                };
                out.push('\u{FFFD}');
                position += consumed;
            }
        }
    }
    out
}
